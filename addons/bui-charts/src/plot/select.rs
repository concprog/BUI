use blenderui::hooks::Signal;
use blenderui::{App, Hsla, Modifiers, Painter, ScrollWheelEvent, Window};

use super::scale::Scale;
use crate::geom::{Bounds, Point, point};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum VisibilityMode {
    #[default]
    Auto,
    Always,
    Never,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SelectionMode {
    #[default]
    X,
    Y,
    XY,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SelectedRange {
    pub x: (f64, f64),
    pub y: Option<(f64, f64)>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Zoom {
    pub x: Option<(f64, f64)>,
    pub y: Option<(f64, f64)>,
    home_x: Option<(f64, f64)>,
    home_y: Option<(f64, f64)>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Drag {
    pub from: Point,
    pub to: Point,
}

type SelectFn = dyn Fn(SelectedRange, &mut Window, &mut App);

/// The chart's `on_select` callback, kept from render to render.
#[derive(Default)]
pub(crate) struct SelectSlot(pub Option<Box<SelectFn>>);

impl SelectSlot {
    /// Calls the callback (taken out of its signal for the call, which may render).
    pub(crate) fn call(slot: Signal<Self>, range: SelectedRange, window: &mut Window, cx: &mut App) {
        let mut taken = Self::default();
        slot.update_untracked(|current| std::mem::swap(current, &mut taken));
        if let Some(callback) = &taken.0 {
            callback(range, window, cx);
        }
        slot.update_untracked(|current| std::mem::swap(current, &mut taken));
    }
}

const MIN_DRAG: f32 = 4.;

impl Drag {
    pub(crate) fn area(self, plot: Bounds, mode: SelectionMode) -> Bounds {
        let (left, right) = (self.from.x.min(self.to.x), self.from.x.max(self.to.x));
        let (top, bottom) = (self.from.y.min(self.to.y), self.from.y.max(self.to.y));
        let (left, right) = if mode == SelectionMode::Y {
            (plot.left(), plot.right())
        } else {
            (left.max(plot.left()), right.min(plot.right()))
        };
        let (top, bottom) = if mode == SelectionMode::X {
            (plot.top(), plot.bottom())
        } else {
            (top.max(plot.top()), bottom.min(plot.bottom()))
        };
        Bounds::from_corners(point(left, top), point(right, bottom))
    }

    pub(crate) fn locked(self, plot: Bounds) -> Self {
        let area = self.area(plot, SelectionMode::XY);
        let (width, height) = (area.size.width, area.size.height);
        let (full_width, full_height) = (plot.size.width, plot.size.height);
        if width < MIN_DRAG || height < MIN_DRAG || full_width <= 0. || full_height <= 0. {
            return self;
        }
        let scale = (width / full_width).max(height / full_height);
        let (fitted_width, fitted_height) = (full_width * scale, full_height * scale);
        let center = area.center();
        let left = (center.x - fitted_width / 2.).clamp(plot.left(), plot.right() - fitted_width);
        let top = (center.y - fitted_height / 2.).clamp(plot.top(), plot.bottom() - fitted_height);
        Self { from: point(left, top), to: point(left + fitted_width, top + fitted_height) }
    }

    pub(crate) fn paint(self, plot: Bounds, mode: SelectionMode, color: Hsla, painter: &mut Painter) {
        painter.fill(self.area(plot, mode).into(), color.opacity(0.2));
    }

    pub(crate) fn range(self, plot: Bounds, mode: SelectionMode, x: &Scale, y: &Scale) -> Option<SelectedRange> {
        let area = self.area(plot, mode);
        let wide = area.size.width >= MIN_DRAG;
        let tall = area.size.height >= MIN_DRAG;
        let usable = match mode {
            SelectionMode::X => wide,
            SelectionMode::Y => tall,
            SelectionMode::XY => wide && tall,
        };
        if !usable {
            return None;
        }
        let along_x = |at: f32| x.value(((at - plot.left()) / plot.size.width).clamp(0., 1.));
        let along_y = |at: f32| y.value(((plot.bottom() - at) / plot.size.height).clamp(0., 1.));
        let sorted = |a: f64, b: f64| if a <= b { (a, b) } else { (b, a) };
        Some(SelectedRange {
            x: sorted(along_x(area.left()), along_x(area.right())),
            y: (mode != SelectionMode::X).then(|| sorted(along_y(area.bottom()), along_y(area.top()))),
        })
    }
}

pub(crate) type Lens = (Option<(Scale, f32)>, Option<(Scale, f32)>);

/// The modifier that turns the wheel into zoom (BUI reports Cmd as Ctrl on macOS, and a trackpad
/// pinch as Ctrl + wheel, as browsers do).
pub(crate) const fn zoom_held(modifiers: Modifiers) -> bool {
    modifiers.control
}

type Span = Option<(f64, f64)>;

fn zoomed(current: Span, home: Span, (scale, fraction): (Scale, f32), factor: f32) -> (Span, Span) {
    let home = home.unwrap_or_else(|| current.unwrap_or_else(|| scale.range()));
    let low = scale.value(fraction - fraction * factor);
    let high = scale.value((1. - fraction).mul_add(factor, fraction));
    let (low, high) = if low <= high { (low, high) } else { (high, low) };
    let width = high - low;
    if !(width.is_finite() && width > 0.) || width >= home.1 - home.0 {
        return (None, None);
    }
    let shift = (home.0 - low).max(0.) - (high - home.1).max(0.);
    (Some((low + shift, high + shift)), Some(home))
}

impl Zoom {
    pub(crate) fn set_x(&mut self, range: (f64, f64), scale: &Scale) {
        self.home_x = self.home_x.or_else(|| Some(scale.range()));
        self.x = Some(range);
    }

    pub(crate) fn set_y(&mut self, range: (f64, f64), scale: &Scale) {
        self.home_y = self.home_y.or_else(|| Some(scale.range()));
        self.y = Some(range);
    }

    fn scaled(&mut self, (x, y): Lens, factor: f32) {
        if let Some(lens) = x {
            (self.x, self.home_x) = zoomed(self.x, self.home_x, lens, factor);
        }
        if let Some(lens) = y {
            (self.y, self.home_y) = zoomed(self.y, self.home_y, lens, factor);
        }
    }
}

/// The wheel with the zoom modifier over `plot` zooms around the pointer: what `lens` says is under
/// it. `plot` and `lens` come from the chart's last layout (`None` before its first paint).
pub(crate) fn wheel_zoom(
    event: &ScrollWheelEvent,
    zoom: Signal<Zoom>,
    cx: &mut App,
    lens: impl FnOnce(Point) -> Option<(Bounds, Lens)>,
) {
    if !zoom_held(event.modifiers) {
        return;
    }
    let at = Point::from(event.position);
    let Some((plot, lens)) = lens(at) else { return };
    if !plot.contains(at) {
        return;
    }
    let rise = event.pixels.unwrap_or(event.delta * 16.);
    let factor = (-rise * 0.004).clamp(-0.5, 0.5).exp();
    if (factor - 1.).abs() > f32::EPSILON {
        zoom.update(|zoom| zoom.scaled(lens, factor));
    }
    cx.stop_propagation();
    cx.prevent_default();
}

pub(crate) fn fraction(at: f32, from: f32, length: f32) -> f32 {
    if length > 0. { ((at - from) / length).clamp(0., 1.) } else { 0.5 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geom::size;

    fn plot() -> Bounds {
        Bounds::new(point(10., 10.), size(400., 200.))
    }

    #[test]
    fn locked_drag_keeps_the_plot_aspect_inside_the_plot() {
        let drag = Drag { from: point(380., 30.), to: point(405., 60.) }.locked(plot());
        let area = drag.area(plot(), SelectionMode::XY);
        let ratio = area.size.width / area.size.height;
        assert!((ratio - 2.).abs() < 1e-3);
        assert!(area.right() <= plot().right() && area.left() >= plot().left());
        assert!(area.bottom() <= plot().bottom() && area.top() >= plot().top());
    }

    #[test]
    fn tiny_locked_drag_is_left_alone() {
        let drag = Drag { from: point(50., 50.), to: point(52., 52.) };
        assert_eq!(drag.locked(plot()), drag);
    }
}
