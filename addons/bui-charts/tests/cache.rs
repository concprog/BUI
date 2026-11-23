//! A chart derives its columns once per revision of its data: renders that change nothing else reuse
//! them, and a write to the data signal makes it derive them again.

use std::sync::atomic::{AtomicUsize, Ordering};

use blenderui::hooks::Signal;
use blenderui::prelude::*;
use bui_charts::{Series, XYChart};

static CALLS: AtomicUsize = AtomicUsize::new(0);

struct ChartView {
    data: Signal<Vec<f64>>,
    /// written to re-render without touching the data
    tick: Signal<u32>,
}

impl Render for ChartView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let (data, _) = (self.data, self.tick.get());
        let x = |value: &f64| {
            CALLS.fetch_add(1, Ordering::Relaxed);
            *value
        };
        rsx! {
            <div class="size-full">
                <XYChart data={data} class="size-full">
                    <Series x={x} y={|value: &f64| *value} />
                </XYChart>
            </div>
        }
    }
}

#[test]
fn columns_are_reused_until_the_data_changes() {
    Application::headless().run(|cx| {
        let data = Signal::new(vec![1., 2., 3.]);
        let tick = Signal::new(0);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds { origin: point(px(0.), px(0.)), size: size(px(400.), px(300.)) })),
            ..Default::default()
        };
        let win = cx.open_window(options, |_, cx| cx.new(|_| ChartView { data, tick })).unwrap();
        cx.draw(win.id);
        assert_eq!(CALLS.load(Ordering::Relaxed), 3);

        for _ in 0..2 {
            tick.update(|tick| *tick += 1);
            cx.draw(win.id);
        }
        assert_eq!(CALLS.load(Ordering::Relaxed), 3, "re-renders reuse the columns");

        data.update(|rows| rows.push(4.));
        cx.draw(win.id);
        assert_eq!(CALLS.load(Ordering::Relaxed), 7, "a write derives them again");
    });
}
