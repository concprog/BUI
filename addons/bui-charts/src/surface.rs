//! A chart's drawing surface: what gpui's `canvas(prepaint, paint)` and its hitboxes did. A render
//! builds the chart's model; the first paint after it lays the model out for the canvas' bounds
//! (text measured, scales fitted) and draws; the pointer handlers ask the same layout what is under
//! the pointer. The model and its layout are owned by one hook signal and handed over by value:
//! nothing is shared or reference counted.

use blenderui::hooks::{Signal, use_signal};

use crate::geom::Bounds;

/// A chart layout: its model placed in the canvas' bounds.
pub(crate) trait Arrange: Sized + 'static {
    type Model: 'static;

    fn arrange(model: Self::Model, bounds: Bounds) -> Self;

    /// The model back, to be laid out again for other bounds.
    fn into_model(self) -> Self::Model;

    fn bounds(&self) -> Bounds;
}

enum Stage<L: Arrange> {
    Empty,
    Model(L::Model),
    Laid(L),
}

pub(crate) struct Surface<L: Arrange> {
    stage: Stage<L>,
    /// the bounds the canvas last painted in (handlers lay out with them before the next paint)
    bounds: Option<Bounds>,
}

impl<L: Arrange> Default for Surface<L> {
    fn default() -> Self {
        Self { stage: Stage::Empty, bounds: None }
    }
}

impl<L: Arrange> Surface<L> {
    /// The layout for `bounds`, laid out again when they changed.
    fn laid(&mut self, bounds: Bounds) -> Option<&mut L> {
        self.bounds = Some(bounds);
        let stage = std::mem::replace(&mut self.stage, Stage::Empty);
        self.stage = match stage {
            Stage::Laid(layout) if layout.bounds() == bounds => Stage::Laid(layout),
            Stage::Laid(layout) => Stage::Laid(L::arrange(layout.into_model(), bounds)),
            Stage::Model(model) => Stage::Laid(L::arrange(model, bounds)),
            Stage::Empty => Stage::Empty,
        };
        match &mut self.stage {
            Stage::Laid(layout) => Some(layout),
            _ => None,
        }
    }
}

/// The surface of the chart being rendered: a `Copy` handle for the canvas and its handlers.
pub(crate) struct Handle<L: Arrange>(Signal<Surface<L>>);

impl<L: Arrange> Clone for Handle<L> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<L: Arrange> Copy for Handle<L> {}

/// The surface of the calling chart (a hook: the same one every render).
#[track_caller]
pub(crate) fn use_surface<L: Arrange>() -> Handle<L> {
    Handle(use_signal(Surface::default()))
}

impl<L: Arrange> Handle<L> {
    /// This render's model; the next paint lays it out.
    pub(crate) fn set(self, model: L::Model) {
        self.0.update_untracked(|surface| surface.stage = Stage::Model(model));
    }

    /// Lays out for `bounds` (when needed) and paints with `f`.
    pub(crate) fn paint(self, bounds: Bounds, f: impl FnOnce(&mut L)) {
        self.0.update_untracked(|surface| {
            if let Some(layout) = surface.laid(bounds) {
                f(layout);
            }
        });
    }

    /// What the last layout says (a handler asking what is under the pointer); `None` before the
    /// first paint.
    pub(crate) fn with<R>(self, f: impl FnOnce(&L) -> R) -> Option<R> {
        self.0.update_untracked(|surface| {
            let bounds = surface.bounds?;
            surface.laid(bounds).map(|layout| f(layout))
        })
    }

    /// [`with`](Self::with), mutably.
    pub(crate) fn update<R>(self, f: impl FnOnce(&mut L) -> R) -> Option<R> {
        self.0.update_untracked(|surface| {
            let bounds = surface.bounds?;
            surface.laid(bounds).map(f)
        })
    }
}
