use blenderui::hooks::Signal;
use blenderui::style::TextAlign;
use blenderui::{
    App, Hsla, MouseButton, MouseDownEvent, MouseMoveEvent, Painter, ScrollWheelEvent, SharedString, Window, black,
    white,
};

use super::grid::Grid;
use super::model::{Cache, Format, Model};
use super::options::BucketLayout;
use crate::color::Hsl;
use crate::geom::{Bounds, Point, Size, point, size};
use crate::plot::{
    AxisPlacement, Drag, LABEL_HEIGHT, Labels, PADDING, SelectSlot, SelectionMode, TEXT_GAP, TEXT_HEIGHT, TEXT_SIZE,
    VisibilityMode, X_TICK_SPACING, Y_TICK_SPACING, Zoom, fraction, gutter, wheel_zoom,
};
use crate::surface::{Arrange, Handle};
use crate::text;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Hover {
    pub column: usize,
    pub row: usize,
    pub at: Point,
    pub area: Size,
}

#[derive(Clone, Copy)]
pub(crate) struct Palette {
    pub text: Hsla,
    pub border: Hsla,
    pub accent: Hsla,
}

#[derive(Clone, Copy)]
pub(crate) struct Pointer {
    pub surface: Handle<Layout>,
    pub cache: Signal<Cache>,
    pub hover: Signal<Option<Hover>>,
    pub drag: Signal<Option<Drag>>,
    pub zoom: Signal<Zoom>,
    pub format: Signal<Format>,
    pub select: Signal<SelectSlot>,
}

/// The model and the cache its grid is in (the labels follow the grid's buckets).
pub(crate) struct Plan {
    pub model: Model,
    pub cache: Signal<Cache>,
}

pub(crate) struct Layout {
    pub model: Model,
    cache: Signal<Cache>,
    pub bounds: Bounds,
    pub plot: Bounds,
    x_labels: Labels,
    y_labels: Labels,
}

fn ordinal(grid: &Grid, index: usize) -> f64 {
    let offset = match grid.layout {
        Some(BucketLayout::Le) => 1.,
        Some(BucketLayout::Ge) => 0.,
        _ => 0.5,
    };
    index as f64 + offset
}

impl Arrange for Layout {
    type Model = Plan;

    fn arrange(plan: Plan, bounds: Bounds) -> Self {
        let Plan { model, cache } = plan;
        let (x_labels, y_labels, plot) = cache.with(|cache| place(&model, &cache.grid, bounds));
        Self { model, cache, bounds, plot, x_labels, y_labels }
    }

    fn into_model(self) -> Plan {
        Plan { model: self.model, cache: self.cache }
    }

    fn bounds(&self) -> Bounds {
        self.bounds
    }
}

/// The axis labels and the plot of `model` over `grid` in `bounds`.
fn place(model: &Model, grid: &Grid, bounds: Bounds) -> (Labels, Labels, Bounds) {
    let (x_axis, y_axis) = (&model.x_axis, &model.y_axis);
    let y_title = if y_axis.visible() && y_axis.label.is_some() { LABEL_HEIGHT } else { 0. };
    let top = PADDING + y_title;
    let bottom = PADDING + gutter(x_axis);
    let height = (bounds.size.height - top - bottom).max(0.);
    let y_at = |fraction: f32| top + height * (1. - fraction);
    let y_labels = if grid.layout.is_some() {
        let items = grid
            .labels
            .iter()
            .enumerate()
            .map(|(index, label)| (y_at(model.y.fraction(ordinal(grid, index))), label.clone()));
        Labels::explicit(items, TEXT_HEIGHT)
    } else {
        Labels::new(&model.y, y_axis, height, Y_TICK_SPACING, y_at)
    };
    let y_width = y_labels.width(y_axis);
    let right_axis = y_axis.placement == AxisPlacement::Right;
    let left = PADDING + if right_axis { 0. } else { y_width };
    let right = PADDING + if right_axis { y_width } else { 0. };
    let width = (bounds.size.width - left - right).max(0.);
    let x_labels = Labels::new(&model.x, x_axis, width, X_TICK_SPACING, |fraction| left + width * fraction);
    (x_labels, y_labels, Bounds::new(bounds.origin + point(left, top), size(width, height)))
}

impl Layout {
    fn x(&self, value: f64) -> f32 {
        self.plot.left() + self.plot.size.width * self.model.x.fraction(value)
    }

    fn y(&self, value: f64) -> f32 {
        self.plot.bottom() - self.plot.size.height * self.model.y.fraction(value)
    }

    fn cell(&self, grid: &Grid, column: usize, row: usize) -> Bounds {
        let (x0, x1) = (self.x(grid.x_edges[column]), self.x(grid.x_edges[column + 1]));
        let (y0, y1) = (self.y(grid.y_edges[row]), self.y(grid.y_edges[row + 1]));
        let gap = self.model.gap / 2.;
        let shrink = |low: f32, high: f32| {
            let (low, high) = if low <= high { (low, high) } else { (high, low) };
            if high - low > gap * 2. { (low + gap, high - gap) } else { (low, high) }
        };
        let ((left, right), (top, bottom)) = (shrink(x0, x1), shrink(y1, y0));
        Bounds::from_corners(point(left, top), point(right, bottom))
    }

    fn locate(&self, grid: &Grid, at: Point) -> Option<(usize, usize)> {
        if !self.plot.contains(at) {
            return None;
        }
        let x = self.model.x.value((at.x - self.plot.left()) / self.plot.size.width);
        let y = self.model.y.value((self.plot.bottom() - at.y) / self.plot.size.height);
        let find = |edges: &[f64], value: f64| {
            let (low, high) = (edges.first()?.min(*edges.last()?), edges.first()?.max(*edges.last()?));
            if value < low || value > high {
                return None;
            }
            let index = edges.partition_point(|edge| *edge <= value).checked_sub(1)?;
            Some(index.min(edges.len().saturating_sub(2)))
        };
        Some((find(&grid.x_edges, x)?, find(&grid.y_edges, y)?))
    }
}

pub(crate) fn paint(layout: &Layout, pointer: Pointer, palette: Palette, painter: &mut Painter) {
    let hovered = pointer.hover.get();
    let drag = pointer.drag.get();
    let mut values: Vec<(SharedString, Point, Hsla)> = Vec::new();
    pointer.cache.with(|cache| {
        pointer.format.with(|format| {
            let (grid, colors) = (&cache.grid, &cache.palette);
            let model = &layout.model;
            let plot = layout.plot;
            painter.with_clip(plot.into(), |painter| {
                let radius = model.radius;
                for column in 0..grid.columns() {
                    for row in 0..grid.rows() {
                        let value = grid.value(column, row);
                        if model.hidden(value) {
                            continue;
                        }
                        let Some(color) = model.tone(value, colors) else { continue };
                        let bounds = layout.cell(grid, column, row);
                        if bounds.is_empty() {
                            continue;
                        }
                        painter.quad(bounds.into(), radius, color, Hsla::TRANSPARENT, 0.);
                        if model.show_value != VisibilityMode::Never {
                            let label = model.cell_text(value, format);
                            let fits = bounds.size.width >= text::measure(&label, TEXT_SIZE) + 4.
                                && bounds.size.height >= TEXT_SIZE + 2.;
                            if fits || model.show_value == VisibilityMode::Always {
                                let ink = if Hsl::from(color).l > 0.55 || color.a < 0.5 { black() } else { white() };
                                let center = bounds.center();
                                values.push((label, point(center.x, center.y - TEXT_SIZE / 2.), ink));
                            }
                        }
                    }
                }
                for (x, y) in &model.exemplars {
                    let center = point(layout.x(*x), layout.y(*y));
                    let side = 6.;
                    let dot = Bounds::centered_at(center, size(side, side));
                    painter.quad(dot.into(), side / 2., model.exemplar_color, Hsla::TRANSPARENT, 0.);
                }
                if let Some(hover) = hovered
                    && hover.column < grid.columns()
                    && hover.row < grid.rows()
                {
                    let bounds = layout.cell(grid, hover.column, hover.row);
                    painter.quad(bounds.into(), radius, Hsla::TRANSPARENT, palette.text, 1.);
                }
                if let Some(drag) = drag {
                    drag.paint(plot, model.selection, palette.accent, painter);
                }
            });
        });
    });
    for (label, at, ink) in &values {
        text::draw(painter, label, *at, TextAlign::Center, TEXT_SIZE, *ink);
    }
    paint_axes(layout, palette, painter);
}

fn paint_axes(layout: &Layout, palette: Palette, painter: &mut Painter) {
    let origin = layout.bounds.origin;
    let plot = layout.plot;
    let (x_axis, y_axis) = (&layout.model.x_axis, &layout.model.y_axis);
    if x_axis.visible() && x_axis.border {
        painter.fill(Bounds::new(point(plot.left(), plot.bottom()), size(plot.size.width, 1.)).into(), palette.border);
    }
    let right_axis = y_axis.placement == AxisPlacement::Right;
    if y_axis.visible() && y_axis.border {
        let x = if right_axis { plot.right() } else { plot.left() };
        painter.fill(Bounds::new(point(x, plot.top()), size(1., plot.size.height)).into(), palette.border);
    }
    let x_visible = x_axis.visible();
    let y_visible = y_axis.visible();
    let local = Bounds::new(plot.origin - origin, plot.size);
    let (left, right, bottom) = (local.left(), local.right(), local.bottom());
    let mut draw = |label: &str, x: f32, y: f32, align: TextAlign| {
        text::draw(painter, label, origin + point(x, y), align, TEXT_SIZE, palette.text);
    };
    if x_visible {
        let y = bottom + TEXT_GAP;
        for (x, tick) in layout.x_labels.iter() {
            draw(tick, crate::trend::inside(tick, x, layout.bounds.size.width), y, TextAlign::Center);
        }
        if let Some(title) = &x_axis.label {
            draw(title, f32::midpoint(left, right), y + TEXT_HEIGHT + TEXT_GAP, TextAlign::Center);
        }
    }
    if y_visible {
        let (x, align) =
            if right_axis { (TEXT_GAP.mul_add(2., right), TextAlign::Left) } else { (TEXT_GAP.mul_add(-2., left), TextAlign::Right) };
        for (y, tick) in layout.y_labels.iter() {
            draw(tick, x, y - TEXT_SIZE / 2., align);
        }
        if let Some(title) = &y_axis.label {
            draw(
                title,
                if right_axis { right } else { left },
                PADDING,
                if right_axis { TextAlign::Right } else { TextAlign::Left },
            );
        }
    }
}

pub(crate) fn on_move(pointer: Pointer, event: &MouseMoveEvent) {
    let at = Point::from(event.position);
    if let Some(current) = pointer.drag.get() {
        // a release outside the chart ends the drag when the pointer comes back
        pointer.drag.set(event.pressed_button.map(|_| Drag { to: at, ..current }));
    }
    let hit = pointer
        .surface
        .with(|layout| {
            pointer.cache.with(|cache| layout.locate(&cache.grid, at)).map(|(column, row)| Hover {
                column,
                row,
                at: at - layout.bounds.origin,
                area: layout.bounds.size,
            })
        })
        .flatten();
    if pointer.hover.get() != hit {
        pointer.hover.set(hit);
    }
}

pub(crate) fn on_leave(pointer: Pointer) {
    if pointer.hover.get().is_some() {
        pointer.hover.set(None);
    }
}

pub(crate) fn on_down(pointer: Pointer, event: &MouseDownEvent) {
    let at = Point::from(event.position);
    if event.button != MouseButton::Left || pointer.surface.with(|layout| layout.plot.contains(at)) != Some(true) {
        return;
    }
    if event.click_count >= 2 {
        pointer.zoom.set(Zoom::default());
        pointer.drag.set(None);
        return;
    }
    pointer.drag.set(Some(Drag { from: at, to: at }));
}

/// A drag ended: its range zooms the heatmap and goes to `on_select`.
pub(crate) fn on_up(pointer: Pointer, window: &mut Window, cx: &mut App) {
    let Some(done) = pointer.drag.get() else { return };
    pointer.drag.set(None);
    let picked = pointer.surface.with(|layout| {
        let (x, y, mode) = (layout.model.x, layout.model.y, layout.model.selection);
        done.range(layout.plot, mode, &x, &y).map(|range| (range, x, y, mode))
    });
    let Some((range, x, y, mode)) = picked.flatten() else { return };
    pointer.zoom.update(|zoom| {
        if mode != SelectionMode::Y {
            zoom.set_x(range.x, &x);
        }
        if let Some(range) = range.y {
            zoom.set_y(range, &y);
        }
    });
    SelectSlot::call(pointer.select, range, window, cx);
}

pub(crate) fn on_wheel(pointer: Pointer, event: &ScrollWheelEvent, cx: &mut App) {
    wheel_zoom(event, pointer.zoom, cx, |at| {
        pointer.surface.with(|layout| {
            let (plot, x, y, mode) = (layout.plot, layout.model.x, layout.model.y, layout.model.selection);
            (
                plot,
                (
                    (mode != SelectionMode::Y).then(|| (x, fraction(at.x, plot.left(), plot.size.width))),
                    (mode != SelectionMode::X).then(|| (y, 1. - fraction(at.y, plot.top(), plot.size.height))),
                ),
            )
        })
    });
}
