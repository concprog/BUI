//! What a view type builds (gpui's `AnyView`): blenderui renders one root view per window, so a
//! panel's content is any [`Render`] entity, drawn into the panel each frame.

use blenderui::prelude::{AnyElement, App, Entity, IntoElement, Render, Window};

trait Erased {
    fn render(&self, window: &mut Window, cx: &mut App) -> AnyElement;
    fn release(&self, cx: &mut App);
}

impl<V: Render> Erased for Entity<V> {
    fn render(&self, window: &mut Window, cx: &mut App) -> AnyElement {
        self.update(cx, |view, cx| view.render(window, cx).into_any_element())
    }

    fn release(&self, cx: &mut App) {
        cx.release(*self);
    }
}

/// A view of any type: `cx.new(|_| MyView { .. }).into()`.
pub struct AnyView(Box<dyn Erased>);

impl<V: Render> From<Entity<V>> for AnyView {
    fn from(entity: Entity<V>) -> Self {
        Self(Box::new(entity))
    }
}

impl AnyView {
    pub(crate) fn render(&self, window: &mut Window, cx: &mut App) -> AnyElement {
        self.0.render(window, cx)
    }

    /// Frees the entity: the workspace drops a view whose tab closed.
    pub(crate) fn release(self, cx: &mut App) {
        self.0.release(cx);
    }
}
