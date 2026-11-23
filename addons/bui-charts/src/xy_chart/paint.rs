use blenderui::hooks::Signal;
use blenderui::style::TextAlign;
use blenderui::{App, Hsla, MouseButton, MouseDownEvent, MouseMoveEvent, Painter, ScrollWheelEvent};

use super::layout::{Hit, Layout};
use super::model::{Cache, Columns, Plotted};
use super::style::PointShape;
use crate::geom::{Bounds, Point, point, size};
use crate::path::Path;
use crate::plot::{
    AxisColorMode, AxisConfig, AxisPlacement, Drag, LABEL_HEIGHT, PADDING, Scale, SelectionMode, TEXT_GAP,
    TEXT_HEIGHT, TEXT_SIZE, Zoom, fraction, wheel_zoom,
};
use crate::surface::Handle;
use crate::text;

const HOVER_OPACITY: f32 = 0.4;

#[derive(Clone, Copy)]
pub(crate) struct Palette {
    pub text: Hsla,
    pub grid: Hsla,
    pub border: Hsla,
    pub hover: Hsla,
}

#[derive(Clone, Copy)]
pub(crate) struct Pointer {
    pub surface: Handle<Layout>,
    pub cache: Signal<Cache>,
    pub hover: Signal<Option<Hit>>,
    pub zoom: Signal<Zoom>,
    pub drag: Signal<Option<Drag>>,
    pub tips: bool,
}

pub(crate) fn paint(layout: &Layout, pointer: Pointer, palette: Palette, painter: &mut Painter) {
    let hovered = pointer.hover.get();
    let dragging = pointer.drag.get();
    let locked = layout.model.equal.is_some();
    paint_grid(layout, palette, painter);
    pointer.cache.with(|cache| {
        let columns = &cache.columns;
        painter.with_clip(layout.plot.into(), |painter| {
            for series in layout.model.series.iter().filter(|series| !series.hidden) {
                let data = &columns[series.index];
                if series.style.show().points() {
                    paint_points(layout, series, data, painter);
                }
                if series.style.show().lines() {
                    paint_line(layout, series, data, painter);
                }
            }
            if let Some(hit) = hovered {
                paint_hover(layout, columns, hit, palette.hover, painter);
            }
            if let Some(drag) = dragging {
                let drag = if locked { drag.locked(layout.plot) } else { drag };
                drag.paint(layout.plot, SelectionMode::XY, palette.hover, painter);
            }
        });
    });
    paint_labels(layout, palette, painter);
}

/// The last layout's plot, its scales and whether its zoom keeps the aspect.
fn frame(pointer: Pointer) -> Option<(Bounds, Scale, Scale, bool)> {
    pointer.surface.with(|layout| (layout.plot, layout.model.x, layout.model.y, layout.model.equal.is_some()))
}

pub(crate) fn on_down(pointer: Pointer, event: &MouseDownEvent) {
    let at = Point::from(event.position);
    if event.button != MouseButton::Left || !frame(pointer).is_some_and(|(plot, ..)| plot.contains(at)) {
        return;
    }
    if event.click_count >= 2 {
        pointer.zoom.set(Zoom::default());
        pointer.drag.set(None);
        return;
    }
    pointer.drag.set(Some(Drag { from: at, to: at }));
}

pub(crate) fn on_move(pointer: Pointer, event: &MouseMoveEvent) {
    let at = Point::from(event.position);
    if let Some(current) = pointer.drag.get() {
        // a release outside the chart ends the drag when the pointer comes back
        pointer.drag.set(event.pressed_button.map(|_| Drag { to: at, ..current }));
    }
    if !pointer.tips {
        return;
    }
    let hit = pointer.surface.update(|layout| pointer.cache.with(|cache| layout.hit(at, &cache.columns))).flatten();
    if pointer.hover.get() != hit {
        pointer.hover.set(hit);
    }
}

pub(crate) fn on_leave(pointer: Pointer) {
    if pointer.hover.get().is_some() {
        pointer.hover.set(None);
    }
}

/// A drag ended: the area it spans zooms both axes.
pub(crate) fn on_up(pointer: Pointer) {
    let Some(done) = pointer.drag.get() else { return };
    pointer.drag.set(None);
    let Some((plot, x, y, locked)) = frame(pointer) else { return };
    let done = if locked { done.locked(plot) } else { done };
    let Some(range) = done.range(plot, SelectionMode::XY, &x, &y) else { return };
    pointer.zoom.update(|zoom| {
        zoom.set_x(range.x, &x);
        if let Some(range) = range.y {
            zoom.set_y(range, &y);
        }
    });
}

pub(crate) fn on_wheel(pointer: Pointer, event: &ScrollWheelEvent, cx: &mut App) {
    wheel_zoom(event, pointer.zoom, cx, |at| {
        frame(pointer).map(|(plot, x, y, _)| {
            let lens = (
                Some((x, fraction(at.x, plot.left(), plot.size.width))),
                Some((y, 1. - fraction(at.y, plot.top(), plot.size.height))),
            );
            (plot, lens)
        })
    });
}
fn paint_grid(layout: &Layout, palette: Palette, painter: &mut Painter) {
    let (plot, origin) = (layout.plot, layout.bounds.origin);
    let (x_axis, y_axis) = (&layout.model.x_axis, &layout.model.y_axis);
    let mut line = |from: Point, width: f32, height: f32, color: Hsla| {
        painter.fill(Bounds::new(from, size(width, height)).into(), color);
    };
    let (width, height, thin) = (plot.size.width, plot.size.height, 1.);
    if x_axis.grid {
        for x in layout.x_labels.positions() {
            line(
                point(origin.x + x, plot.top()),
                thin,
                height,
                palette.grid,
            );
        }
    }
    if y_axis.grid {
        for y in layout.y_labels.positions() {
            line(
                point(plot.left(), origin.y + y),
                width,
                thin,
                palette.grid,
            );
        }
    }
    if x_axis.visible() && x_axis.border {
        let y = if x_axis.placement == AxisPlacement::Top {
            plot.top()
        } else {
            plot.bottom()
        };
        line(point(plot.left(), y), width, thin, palette.border);
    }
    if y_axis.visible() && y_axis.border {
        let x = if y_axis.placement == AxisPlacement::Right {
            plot.right()
        } else {
            plot.left()
        };
        line(point(x, plot.top()), thin, height, palette.border);
    }
}

fn paint_points(layout: &Layout, series: &Plotted, data: &Columns, painter: &mut Painter) {
    let style = &series.style;
    let (stroke, opacity) = (style.point_stroke_width(), style.fill_opacity());
    let square = style.point_shape() == PointShape::Square;
    for (index, at, diameter) in layout.points(data) {
        let color = data.fill(index, series.color);
        let background = if color.a < 1. {
            color
        } else {
            color.opacity(opacity)
        };
        marker(painter, at, diameter + stroke, square, background, stroke, color.alpha(1.));
    }
}

fn paint_line(layout: &Layout, series: &Plotted, data: &Columns, painter: &mut Painter) {
    let width = series.style.line_width();
    if width <= 0. {
        return;
    }
    let mut builder = Path::with_capacity(data.xs.len());
    let (mut open, mut any) = (false, false);
    for (x, y) in data.xs.iter().zip(&data.ys) {
        if !x.is_finite() || !y.is_finite() {
            open = false;
            continue;
        }
        let at = layout.position(*x, *y);
        if open {
            builder.line_to(at);
            any = true;
        } else {
            builder.move_to(at);
            open = true;
        }
    }
    if !any {
        return;
    }
    builder.stroke_styled(painter, series.color, width, series.style.line_style());
}

fn paint_hover(layout: &Layout, columns: &[Columns], hit: Hit, color: Hsla, painter: &mut Painter) {
    let Some(series) = layout
        .model
        .series
        .iter()
        .find(|series| series.index == hit.series)
    else {
        return;
    };
    let data = &columns[series.index];
    let (Some(x), Some(y)) = (data.xs.get(hit.index), data.ys.get(hit.index)) else {
        return;
    };
    let side = data.diameter(hit.index) + series.style.point_stroke_width();
    let square = series.style.point_shape() == PointShape::Square;
    let at = layout.position(*x, *y);
    marker(painter, at, side, square, color.opacity(HOVER_OPACITY), 0., Hsla::TRANSPARENT);
}

fn marker(
    painter: &mut Painter,
    center: Point,
    side: f32,
    square: bool,
    background: Hsla,
    border: f32,
    border_color: Hsla,
) {
    let radius = if square { 0. } else { side / 2. };
    painter.quad(Bounds::centered_at(center, size(side, side)).into(), radius, background, border_color, border);
}

fn paint_labels(layout: &Layout, palette: Palette, painter: &mut Painter) {
    let origin = layout.bounds.origin;
    let width = layout.bounds.size.width;
    let plot = Bounds::new(layout.plot.origin - origin, layout.plot.size);
    let (left, top, right, bottom) = (
        plot.left(),
        plot.top(),
        plot.right(),
        plot.bottom(),
    );
    let model = &layout.model;
    let tick_color = |axis: &AxisConfig| match axis.color_mode {
        AxisColorMode::Series => model
            .series
            .iter()
            .find(|series| !series.hidden)
            .map_or(palette.text, |series| series.color),
        AxisColorMode::Text => palette.text,
    };
    let (x_axis, y_axis) = (&model.x_axis, &model.y_axis);
    let (x_color, y_color) = (tick_color(x_axis), tick_color(y_axis));
    let (x_visible, y_visible) = (x_axis.visible(), y_axis.visible());
    let x_title = x_axis.label.as_ref().filter(|_| x_visible);
    let y_title = y_axis.label.as_ref().filter(|_| y_visible);
    let (x_on_top, y_on_right) = (
        x_axis.placement == AxisPlacement::Top,
        y_axis.placement == AxisPlacement::Right,
    );
    let mut text = |label: &str, x: f32, y: f32, align: TextAlign, color: Hsla| {
        text::draw(painter, label, origin + point(x, y), align, TEXT_SIZE, color);
    };

    if x_visible {
        let y = if x_on_top {
            top - TEXT_GAP - TEXT_SIZE
        } else {
            bottom + TEXT_GAP
        };
        for (x, tick) in layout.x_labels.iter() {
            text(tick, crate::trend::inside(tick, x, layout.bounds.size.width), y, TextAlign::Center, x_color);
        }
        if let Some(title) = x_title {
            let y = if x_on_top {
                y - LABEL_HEIGHT
            } else {
                y + TEXT_HEIGHT + TEXT_GAP
            };
            text(title, width / 2., y, TextAlign::Center, palette.text);
        }
    }
    if y_visible {
        let (x, align) = if y_on_right {
            (TEXT_GAP.mul_add(2., right), TextAlign::Left)
        } else {
            (TEXT_GAP.mul_add(-2., left), TextAlign::Right)
        };
        for (y, tick) in layout.y_labels.iter() {
            text(tick, x, y - TEXT_SIZE / 2., align, y_color);
        }
        if let Some(title) = y_title {
            let (x, align) = if y_on_right {
                (width - PADDING, TextAlign::Right)
            } else {
                (PADDING, TextAlign::Left)
            };
            text(title, x, PADDING, align, palette.text);
        }
    }
}
