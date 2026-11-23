//! What `#[component]` (bui-charts-macros) expands to: the builder's prop states and the hand-off of
//! typed children. Not API.

use std::any::Any;
use std::cell::Cell;

use blenderui::element::AnyElement;

/// A required prop not set yet.
pub struct Missing;

/// A required prop that was set.
pub struct Set<V>(pub V);

/// An optional callback prop, set (`Option<F>`) or not (`Option<fn(..)>`).
pub trait Optional {
    type Inner;
    fn into_option(self) -> Option<Self::Inner>;
}

impl<F> Optional for Option<F> {
    type Inner = F;
    fn into_option(self) -> Self {
        self
    }
}

thread_local! {
    /// Typed children built but not yet taken by their parent, in markup order.
    static OFFERED: Cell<Vec<Box<dyn Any>>> = const { Cell::new(Vec::new()) };
}

/// A typed child (`<TrendSeries .. />`): blenderui's `rsx!` turns every child into an element
/// right away, so the value waits here for its parent's `.children(..)`, which `rsx!` calls next.
pub fn offer<C: 'static>(child: C) -> AnyElement {
    OFFERED.with(|cell| {
        let mut offered = cell.take();
        offered.push(Box::new(child));
        cell.set(offered);
    });
    AnyElement::Text(std::borrow::Cow::Borrowed(""))
}

/// The children of type `C` offered since the last parent took its own (the elements `rsx!` built
/// for them carry nothing).
#[must_use]
pub fn collect<C: 'static>(_elements: Vec<AnyElement>) -> Vec<C> {
    OFFERED.with(|cell| {
        let offered = cell.take();
        let mut mine = Vec::with_capacity(offered.len());
        let mut rest = Vec::new();
        for child in offered {
            match child.downcast::<C>() {
                Ok(child) => mine.push(*child),
                Err(other) => rest.push(other),
            }
        }
        cell.set(rest);
        mine
    })
}
