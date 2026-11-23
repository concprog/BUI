use std::borrow::Cow;
use std::ops::Range;

use blenderui::hooks::Signal;
use blenderui::style::TextAlign;
use blenderui::{App, Hsla, MouseButton, MouseDownEvent, MouseMoveEvent, Painter, ScrollWheelEvent, SharedString, Window, white};

use super::model::Cache;
use super::options::ValueAlign;
use crate::color::Hsl;
use crate::geom::{Bounds, Point, Size, point, size};
use crate::plot::{
    AxisConfig, AxisPlacement, Drag, Labels, PADDING, Scale, SelectSlot, SelectionMode, TEXT_GAP, TEXT_HEIGHT,
    TEXT_SIZE, VisibilityMode, X_TICK_SPACING, Zoom, fraction, gutter, wheel_zoom,
};
use crate::surface::{Arrange, Handle};
use crate::text;

const TEXT_PADDING: f32 = 4.;
const ELLIPSIS: &str = "…";

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Hover {
    pub lane: usize,
    pub x: f64,
    pub at: Point,
    pub area: Size,
}

#[derive(Clone, Copy)]
pub(crate) struct Palette {
    pub text: Hsla,
    pub ink: Hsla,
    pub grid: Hsla,
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
    pub select: Signal<SelectSlot>,
    pub multi: bool,
}

pub(crate) struct Settings {
    pub x: Scale,
    pub x_axis: AxisConfig,
    pub y_axis: AxisConfig,
    pub page: Range<usize>,
    pub row_height: f32,
    pub col_width: f32,
    pub show_value: VisibilityMode,
    pub align: ValueAlign,
}

/// The settings and the cache the lane names come from.
pub(crate) struct Plan {
    pub settings: Settings,
    pub cache: Signal<Cache>,
}

pub(crate) struct Layout {
    settings: Settings,
    cache: Signal<Cache>,
    bounds: Bounds,
    plot: Bounds,
    names: Vec<SharedString>,
    x_labels: Labels,
}

impl Arrange for Layout {
    type Model = Plan;

    fn arrange(plan: Plan, bounds: Bounds) -> Self {
        let Plan { settings, cache } = plan;
        let names: Vec<SharedString> = cache.with(|cache| {
            let page = settings.page.start.min(cache.lanes.len())..settings.page.end.min(cache.lanes.len());
            cache.lanes[page].iter().map(|lane| lane.name.clone()).collect()
        });
        let y_axis = &settings.y_axis;
        let column = if y_axis.visible() {
            y_axis.width.map_or_else(
                || names.iter().map(|name| text::measure(name, TEXT_SIZE)).fold(0., f32::max) + TEXT_GAP * 3.,
                |width| width.0,
            )
        } else {
            0.
        };
        let right = y_axis.placement == AxisPlacement::Right;
        let top = PADDING;
        let bottom = PADDING + gutter(&settings.x_axis);
        let left = PADDING + if right { 0. } else { column };
        let width = (bounds.size.width - left - PADDING - if right { column } else { 0. }).max(0.);
        let height = (bounds.size.height - top - bottom).max(0.);
        let x_labels = Labels::new(&settings.x, &settings.x_axis, width, X_TICK_SPACING, |fraction| left + width * fraction);
        Self {
            plot: Bounds::new(bounds.origin + point(left, top), size(width, height)),
            bounds,
            names,
            x_labels,
            settings,
            cache,
        }
    }

    fn into_model(self) -> Plan {
        Plan { settings: self.settings, cache: self.cache }
    }

    fn bounds(&self) -> Bounds {
        self.bounds
    }
}

impl Layout {
    fn lane_height(&self) -> f32 {
        self.plot.size.height / self.settings.page.len().max(1) as f32
    }

    fn x(&self, value: f64) -> f32 {
        self.plot.left() + self.plot.size.width * self.settings.x.fraction(value)
    }

    fn hover(&self, at: Point) -> Option<Hover> {
        if !self.plot.contains(at) {
            return None;
        }
        let lane = ((at.y - self.plot.top()) / self.lane_height()).floor() as usize;
        let x = self.settings.x.value((at.x - self.plot.left()) / self.plot.size.width);
        Some(Hover {
            lane: self.settings.page.start + lane.min(self.settings.page.len().saturating_sub(1)),
            x,
            at: at - self.bounds.origin,
            area: self.bounds.size,
        })
    }
}

/// `label`, cut with an ellipsis to fit `room` (`None`: too little room to show anything useful).
fn fit(label: &str, room: f32, mode: VisibilityMode) -> Option<Cow<'_, str>> {
    let width = text::measure(label, TEXT_SIZE);
    if width <= room {
        return Some(Cow::Borrowed(label));
    }
    let count = label.chars().count().max(1);
    let fits = ((room / (width / count as f32)).floor() as usize).saturating_sub(1);
    if fits < 2 && mode == VisibilityMode::Auto {
        return None;
    }
    let mut short: String = label.chars().take(fits.max(1)).collect();
    short.push_str(ELLIPSIS);
    Some(Cow::Owned(short))
}

#[allow(clippy::too_many_lines)]
pub(crate) fn paint(layout: &Layout, pointer: Pointer, palette: Palette, painter: &mut Painter) {
    let hovered = pointer.hover.get();
    let drag = pointer.drag.get();
    let plot = layout.plot;
    let lane_height = layout.lane_height();
    if layout.settings.x_axis.grid {
        for x in layout.x_labels.positions() {
            let line = Bounds::new(point(layout.bounds.origin.x + x, plot.top()), size(1., plot.size.height));
            painter.fill(line.into(), palette.grid);
        }
    }
    pointer.cache.with(|cache| {
        let settings = &layout.settings;
        let page = settings.page.start.min(cache.lanes.len())..settings.page.end.min(cache.lanes.len());
        let mut labels: Vec<(Cow<'_, str>, Point, TextAlign, Hsla)> = Vec::new();
        painter.with_clip(plot.into(), |painter| {
            for (offset, lane) in cache.lanes[page.clone()].iter().enumerate() {
                let lane_top = plot.top() + lane_height * offset as f32;
                let box_height = lane_height * settings.row_height.clamp(0., 1.);
                let top = lane_top + (lane_height - box_height) / 2.;
                let stroke = lane.line_width;
                let hovered_lane = hovered.filter(|hover| hover.lane == settings.page.start + offset);
                for segment in &lane.segments {
                    let (left, right) = (layout.x(segment.start), layout.x(segment.end));
                    let inset = (right - left) * (1. - settings.col_width.clamp(0., 1.)) / 2.;
                    let (left, right) = (left + inset, right - inset);
                    if right < plot.left() || left > plot.right() {
                        continue;
                    }
                    let bounds = Bounds::new(point(left, top), size((right - left).max(1.), box_height));
                    let background =
                        if segment.color.a < 1. { segment.color } else { segment.color.opacity(lane.fill_opacity) };
                    painter.quad(bounds.into(), 0., background, segment.color, stroke);
                    if hovered_lane.is_some_and(|hover| segment.start <= hover.x && hover.x < segment.end) {
                        painter.quad(bounds.into(), 0., Hsla::TRANSPARENT, palette.text, 1.);
                    }
                    if settings.show_value == VisibilityMode::Never {
                        continue;
                    }
                    let (visible_left, visible_right) = (left.max(plot.left()), right.min(plot.right()));
                    let room = stroke.mul_add(-2., visible_right - visible_left) - TEXT_PADDING * 2.;
                    if room <= 0. || box_height < TEXT_SIZE {
                        continue;
                    }
                    let Some(label) = fit(&segment.text, room, settings.show_value) else { continue };
                    let ink = if background.a >= 0.5 && Hsl::from(background).l < 0.6 { white() } else { palette.ink };
                    let middle = top + box_height / 2. - TEXT_SIZE / 2.;
                    let (x, align) = match settings.align {
                        ValueAlign::Left => (visible_left + stroke + TEXT_PADDING, TextAlign::Left),
                        ValueAlign::Right => (visible_right - stroke - TEXT_PADDING, TextAlign::Right),
                        ValueAlign::Center => (f32::midpoint(visible_left, visible_right), TextAlign::Center),
                    };
                    labels.push((label, point(x, middle), align, ink));
                }
            }
            if let Some(hover) = hovered.filter(|_| pointer.multi) {
                let x = layout.x(hover.x);
                painter.fill(Bounds::new(point(x, plot.top()), size(1., plot.size.height)).into(), palette.text.opacity(0.6));
            }
            if let Some(drag) = drag {
                drag.paint(plot, SelectionMode::X, palette.accent, painter);
            }
        });
        for (label, at, align, ink) in &labels {
            text::draw(painter, label, *at, *align, TEXT_SIZE, *ink);
        }
    });
    let origin = layout.bounds.origin;
    if layout.settings.y_axis.visible() {
        let right = layout.settings.y_axis.placement == AxisPlacement::Right;
        let (x, align) =
            if right { (plot.right() + TEXT_GAP * 2., TextAlign::Left) } else { (plot.left() - TEXT_GAP * 2., TextAlign::Right) };
        for (offset, name) in layout.names.iter().enumerate() {
            let y = lane_height.mul_add(offset as f32 + 0.5, plot.top()) - TEXT_SIZE / 2.;
            text::draw(painter, name, point(x, y), align, TEXT_SIZE, palette.text);
        }
    }
    if layout.settings.x_axis.visible() {
        let y = plot.bottom() + TEXT_GAP;
        for (x, tick) in layout.x_labels.iter() {
            let x = crate::trend::inside(tick, x, layout.bounds.size.width);
            text::draw(painter, tick, point(origin.x + x, y), TextAlign::Center, TEXT_SIZE, palette.text);
        }
        if let Some(title) = &layout.settings.x_axis.label {
            let y = y + TEXT_HEIGHT + TEXT_GAP;
            text::draw(painter, title, point(plot.center().x, y), TextAlign::Center, TEXT_SIZE, palette.text);
        }
    }
    if layout.settings.x_axis.border {
        painter.fill(Bounds::new(point(plot.left(), plot.bottom()), size(plot.size.width, 1.)).into(), palette.border);
    }
}

pub(crate) fn on_move(pointer: Pointer, event: &MouseMoveEvent) {
    let at = Point::from(event.position);
    if let Some(current) = pointer.drag.get() {
        // a release outside the chart ends the drag when the pointer comes back
        pointer.drag.set(event.pressed_button.map(|_| Drag { to: at, ..current }));
    }
    let hit = pointer.surface.with(|layout| layout.hover(at)).flatten();
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

/// A drag ended: its x range zooms the timeline and goes to `on_select`.
pub(crate) fn on_up(pointer: Pointer, window: &mut Window, cx: &mut App) {
    let Some(done) = pointer.drag.get() else { return };
    pointer.drag.set(None);
    let picked = pointer.surface.with(|layout| {
        let x = layout.settings.x;
        done.range(layout.plot, SelectionMode::X, &x, &x).map(|range| (range, x))
    });
    let Some((range, x)) = picked.flatten() else { return };
    pointer.zoom.update(|zoom| zoom.set_x(range.x, &x));
    SelectSlot::call(pointer.select, range, window, cx);
}

pub(crate) fn on_wheel(pointer: Pointer, event: &ScrollWheelEvent, cx: &mut App) {
    wheel_zoom(event, pointer.zoom, cx, |at| {
        pointer.surface.with(|layout| {
            let (plot, x) = (layout.plot, layout.settings.x);
            (plot, (Some((x, fraction(at.x, plot.left(), plot.size.width))), None))
        })
    });
}
