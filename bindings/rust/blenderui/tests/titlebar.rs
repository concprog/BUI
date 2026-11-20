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

struct NativeClose;

impl Render for NativeClose {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        body().child(button().id("close").child("x").window_control_area(WindowControlArea::Close))
    }
}

#[test]
fn a_native_close_button_closes_once_with_the_close_hooks() {
    Application::headless().run(|cx| {
        let opts = WindowOptions { window_decorations: WindowDecorations::Client, ..Default::default() };
        let h = cx.open_window(opts, |_, cx| cx.new(|_| NativeClose)).unwrap();
        let closed = Rc::new(Cell::new(0));
        let seen = closed.clone();
        cx.on_window_closed(move |id, _| {
            assert_eq!(id, h.id);
            seen.set(seen.get() + 1);
        });
        cx.draw(h.id);
        let (x, y) = centre(cx, h.id, "close");
        cx.simulate_click(h.id, x, y);
        assert_eq!(closed.get(), 1, "the widget's click ran the Close control and the close hooks");
        cx.close_window(h.id);
        assert_eq!(closed.get(), 1, "a closed window does not close again");
    });
}

struct RsxBar;

impl Render for RsxBar {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        // the website's rsx! title bar
        rsx! {
            <body m-0>
                <div flex h-8 style="app-region: drag">
                    <div id="file" onClick={|| {}}>"File"</div>
                    <div id="title" flex-1>"My app"</div>
                    <div id="min" style="width: 46px; app-region: minimize">"–"</div>
                    <div id="max" style="width: 46px; app-region: maximize">"□"</div>
                    <div id="close" style="width: 46px; app-region: close">"✕"</div>
                </div>
            </body>
        }
    }
}

#[test]
fn rsx_marks_the_bar_with_app_region() {
    Application::headless().run(|cx| {
        let opts = WindowOptions { window_decorations: WindowDecorations::Client, ..Default::default() };
        let h = cx.open_window(opts, |_, cx| cx.new(|_| RsxBar)).unwrap();
        cx.draw(h.id);
        let at = |cx: &mut App, id: &str| {
            let (x, y) = centre(cx, h.id, id);
            cx.window_control_area(h.id, x, y)
        };
        assert_eq!(at(cx, "file"), None);
        assert_eq!(at(cx, "title"), Some(WindowControlArea::Drag));
        assert_eq!(at(cx, "min"), Some(WindowControlArea::Min));
        assert_eq!(at(cx, "max"), Some(WindowControlArea::Max));
        assert_eq!(at(cx, "close"), Some(WindowControlArea::Close));
    });
}

#[test]
fn a_close_from_an_update_runs_the_close_hooks() {
    Application::headless().run(|cx| {
        let h = cx.open_window(WindowOptions::default(), |_, cx| cx.new(|_| NativeClose)).unwrap();
        let closed = Rc::new(Cell::new(0));
        let seen = closed.clone();
        cx.on_window_closed(move |_, _| seen.set(seen.get() + 1));
        h.update(cx, |_, window, _| window.remove_window());
        assert_eq!(closed.get(), 1, "an update that asks no render still finishes the close");
    });
}

/// The close hooks' calls: (hook, window).
struct Closed(Vec<(u8, WindowId)>);

#[test]
fn a_window_a_close_hook_closes_gets_its_hooks_once() {
    Application::headless().run(|cx| {
        let a = cx.open_window(WindowOptions::default(), |_, cx| cx.new(|_| NativeClose)).unwrap();
        let b = cx.open_window(WindowOptions::default(), |_, cx| cx.new(|_| NativeClose)).unwrap();
        cx.set_global(Closed(Vec::new()));
        cx.on_window_closed(move |id, cx| {
            cx.global_mut::<Closed>().0.push((1, id));
            if id == a.id {
                cx.close_window(b.id); // the other window, from inside a hook
            }
        });
        cx.on_window_closed(|id, cx| cx.global_mut::<Closed>().0.push((2, id)));
        cx.close_window(a.id);
        assert_eq!(cx.global::<Closed>().0, [(1, a.id), (2, a.id), (1, b.id), (2, b.id)]);
        cx.close_window(b.id);
        assert_eq!(cx.global::<Closed>().0.len(), 4, "a closed window does not close again");
    });
}
