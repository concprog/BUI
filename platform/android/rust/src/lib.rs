//! A BUI app in Rust on Android: the same code as on a desktop. `bui_app_main` is what the
//! NativeActivity shell (platform/android/bui_android.c) calls; the window is the kit's Android host
//! (kit/intern/kit_window_android.c), so `Application::run` needs nothing Android-specific.

use blenderui::prelude::*;

struct Demo {
    taps: u32,
}

impl Render for Demo {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut rows = div().flex().flex_col().gap_1();
        for i in 1..=30 {
            rows = rows.child(div().child(format!("Row {i}")));
        }
        div()
            .id("root")
            .css("height", "100vh") // a definite height: this is the scroll container
            .overflow_y_scroll()
            .flex()
            .flex_col()
            .gap_2()
            .p_4()
            .child(h1().child("BUI + Rust on Android"))
            .child(button().child(format!("Tapped {} times", self.taps)).on_click(cx.listener(
                |this, _, _, cx| {
                    this.taps += 1;
                    cx.notify();
                },
            )))
            .child(input().attr("placeholder", "tap to type"))
            .child(rows)
    }
}

/// The app: open the window and run until the activity ends.
#[unsafe(no_mangle)]
pub extern "C" fn bui_app_main() {
    Application::new().run(|cx: &mut App| {
        cx.open_window(WindowOptions::default(), |_, cx| cx.new(|_| Demo { taps: 0 })).unwrap();
    });
}
