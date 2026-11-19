//! A title bar the app draws: `window_control_area` / CSS `app-region` mark its parts, and the
//! window asks which part is under the pointer (Windows' WM_NCHITTEST, a press elsewhere).

use std::cell::Cell;
use std::rc::Rc;

use blenderui::prelude::*;

struct Bar {
    clicks: Rc<Cell<u32>>,
}

impl Render for Bar {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let clicks = self.clicks.clone();
        body()
            .child(style().child("html, body { margin: 0 }
                #bar { display: flex; height: 30px }
                #bar > * { width: 40px; flex: none }
                #title { flex: 1 }
                #search { -webkit-app-region: no-drag }"))
            .child(div().id("bar").window_control_area(WindowControlArea::Drag)
                .child(div().id("menu").on_click(move |_, _, _| clicks.set(clicks.get() + 1)))
                .child(div().id("title").child("Title"))
                .child(div().id("search"))
                .child(div().id("min").window_control_area(WindowControlArea::Min))
                .child(div().id("max").window_control_area(WindowControlArea::Max)
                    .child(div().id("icon").size(px(10.0))))
                .child(div().id("close").window_control_area(WindowControlArea::Close)))
            .child(div().id("page").h(px(100.0)))
    }
}

fn centre(cx: &mut App, w: WindowId, id: &str) -> (f32, f32) {
    let b = cx.element_bounds(w, id).unwrap();
    (b.origin.x + b.size.width / 2.0, b.origin.y + b.size.height / 2.0)
}

#[test]
fn each_part_of_the_bar_is_what_it_is_marked() {
    Application::headless().run(|cx| {
        let opts = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds { origin: point(px(0.0), px(0.0)),
                                                                size: size(px(400.0), px(200.0)) })),
            window_decorations: WindowDecorations::Client,
            ..Default::default()
        };
        let clicks = Rc::new(Cell::new(0));
        let h = cx.open_window(opts, |_, cx| cx.new(|_| Bar { clicks: clicks.clone() })).unwrap();
        cx.draw(h.id);
        let at = |cx: &mut App, id: &str| {
            let (x, y) = centre(cx, h.id, id);
            cx.window_control_area(h.id, x, y)
        };
        assert_eq!(at(cx, "title"), Some(WindowControlArea::Drag), "text in the bar drags");
        assert_eq!(at(cx, "menu"), None, "an element that takes clicks is the app's");
        assert_eq!(at(cx, "search"), None, "no-drag is the app's");
        assert_eq!(at(cx, "min"), Some(WindowControlArea::Min));
        assert_eq!(at(cx, "icon"), Some(WindowControlArea::Max), "a button's icon is the button");
        assert_eq!(at(cx, "close"), Some(WindowControlArea::Close));
        assert_eq!(at(cx, "page"), None, "below the bar");

        // a press on the drag area is the window manager's: the bar's buttons still click
        let (x, y) = centre(cx, h.id, "title");
        cx.simulate_click(h.id, x, y);
        let (x, y) = centre(cx, h.id, "menu");
        cx.simulate_click(h.id, x, y);
        assert_eq!(clicks.get(), 1);
    });
}
