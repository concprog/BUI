//! blenderui's components run without the window or the app (`#[component]` bodies take props
//! only), but a `<Workspace>` owns an entity and draws it. [`scope`] lends the window and the
//! app to the components rendered inside it, as zopra's components got them as arguments.

use std::{cell::Cell, ptr::NonNull};

use blenderui::prelude::{AnyElement, App, IntoElement, Window};

type Host = (NonNull<Window>, NonNull<App>);

thread_local! {
    static HOST: Cell<Option<Host>> = const { Cell::new(None) };
}

/// Puts back what was lent before, also when a component panics.
struct Restore(Option<Host>);

impl Drop for Restore {
    fn drop(&mut self) {
        HOST.set(self.0.take());
    }
}

/// Render `f`'s markup with the window and app lent to the `<Workspace>`s in it. Call it from a
/// view's `render`:
///
/// ```ignore
/// impl Render for IdeView {
///     fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
///         bui_panels::ui::scope(window, cx, || rsx! { <Ide /> })
///     }
/// }
/// ```
pub fn scope<E: IntoElement>(window: &mut Window, cx: &mut App, f: impl FnOnce() -> E) -> AnyElement {
    let _restore = Restore(HOST.replace(Some((NonNull::from(window), NonNull::from(cx)))));
    f().into_any_element()
}

/// The window and app lent by the [`scope`] being rendered.
pub(crate) fn with_host<R>(f: impl FnOnce(&mut Window, &mut App) -> R) -> R {
    let (mut window, mut cx) = HOST.take().expect(
        "a <Workspace> renders inside bui_panels::ui::scope(window, cx, || ..) (from a view's render)",
    );
    let _restore = Restore(Some((window, cx)));
    // SAFETY: `scope` turned its exclusive borrows into these pointers and does not touch the
    // window or the app until `f` (its closure) returns; `take` above leaves the slot empty, so
    // no second reference to them exists while this `f` runs.
    unsafe { f(window.as_mut(), cx.as_mut()) }
}
