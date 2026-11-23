use std::hash::{DefaultHasher, Hash, Hasher};

use blenderui::hooks::Signal;
use blenderui::style::TextAlign;
use blenderui::{
    App, Hsla, MouseButton, MouseDownEvent, MouseMoveEvent, Painter, ScrollWheelEvent, SharedString,
    Window, rgb,
};

use super::layout::{Hover, Layout, unrotate};
use super::marks::{Mark, MarkHover};
use super::model::{Cache, Data, Line, Model, Plotted};
use super::options::{
    BarAlignment, DrawStyle, GradientMode, Interpolation, Part, PointsMode, Style, ThresholdsStyle, ValueTransform,
};
use super::shade::{self, Shade, Stops, hue, with_alpha};
use crate::candlestick::{CandleStyle, ColorStrategy};
use crate::geom::{Bounds, Point, point, size};
use crate::path::Path;
use crate::plot::{
    AxisColorMode, AxisPlacement, Drag, Extent, LABEL_HEIGHT, LineStyle, PADDING, PathLayer, SelectSlot,
    SelectionMode, TEXT_GAP, TEXT_HEIGHT, TEXT_SIZE, Zoom, fraction, wheel_zoom,
};
use crate::surface::Handle;
use crate::thresholds::{Fallback, fallback, steps};
use crate::{ChartColors, ChartTheme, text};

const BAND_OPACITY: f32 = 0.35;
const THRESHOLD_LINE_ALPHA: f32 = 0.7;
const THRESHOLD_AREA_ALPHA: f32 = 0.15;
const HOVER_ALPHA: f32 = 0.4;

#[derive(Clone, Copy)]
pub(crate) struct Palette {
    pub text: Hsla,
    pub grid: Hsla,
    pub border: Hsla,
    pub cursor: Hsla,
    pub background: Hsla,
}

/// How a kept path is drawn: filled, or stroked at a width in a line style.
#[derive(Clone, Copy)]
pub(crate) struct Ink {
    shade: Shade,
    stroke: Option<(f32, LineStyle)>,
}

impl Ink {
    fn paint(&self, path: &Path, plot: Bounds, vertical: bool, painter: &mut Painter) {
        match self.stroke {
            Some((width, style)) => shade::stroke_along(path, &self.shade, plot, vertical, width, style, painter),
            None => shade::paint_along(path, &self.shade, plot, vertical, painter),
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Pointer {
    pub surface: Handle<Layout>,
    pub cache: Signal<Cache>,
    pub hover: Signal<Option<Hover>>,
    pub drag: Signal<Option<Drag>>,
    pub zoom: Signal<Zoom>,
    pub select: Signal<SelectSlot>,
    pub marks: Signal<Option<MarkHover>>,
    pub paths: Signal<PathLayer<Ink>>,
    pub single: bool,
}

type Recorded = Vec<(Path, Ink)>;

struct Scene<'a> {
    layout: &'a Layout,
    data: &'a Data,
    colors: &'a ChartColors,
    fallback: Fallback,
    dark: bool,
    column: f32,
}

/// Where the scene draws: the painter, and the paths kept for the next paint.
struct Out<'p> {
    painter: &'p mut Painter,
    record: Option<Recorded>,
}

pub(crate) fn paint(layout: &Layout, pointer: Pointer, palette: Palette, painter: &mut Painter) {
    let hovered = pointer.hover.get();
    paint_grid(layout, palette, painter);
    let mut values: Vec<(SharedString, Point)> = Vec::new();
    pointer.cache.with(|cache| {
        let data = &cache.data;
        let colors = ChartTheme::colors();
        let scene = Scene {
            layout,
            data,
            colors: &colors,
            fallback: fallback(&colors),
            dark: crate::color::Hsl::from(palette.background).l < 0.5,
            column: column(layout, &data.xs),
        };
        let plot = layout.plot;
        let layer = layout.model.layer.map(|key| {
            let mut state = DefaultHasher::new();
            (key, layout.logical.size.width.to_bits(), layout.logical.size.height.to_bits(), scene.column.to_bits())
                .hash(&mut state);
            state.finish()
        });
        let plain = || layout.model.visible().filter(|series| series.part.is_none());
        pointer.paths.update_untracked(|paths| {
            let warm = layer.is_some_and(|key| paths.fresh(key));
            painter.with_clip(plot.into(), |painter| {
                let mut out = Out { painter, record: None };
                if warm {
                    let vertical = layout.vertical_axis();
                    for (path, ink) in paths.replay(plot.origin) {
                        ink.paint(path, plot, vertical, out.painter);
                    }
                } else {
                    out.record = layer.map(|_| Vec::new());
                    scene.thresholds(&mut out);
                    scene.candles(palette.background, &mut out);
                    let mut scratch = Scratch::default();
                    for series in plain() {
                        scene.series(series, &mut scratch, &mut out);
                    }
                    if let (Some(key), Some(items)) = (layer, out.record.take()) {
                        paths.store(key, plot.origin, items);
                    }
                }
                for series in plain() {
                    scene.points(series, &mut values, out.painter);
                }
                scene.marks(&mut out);
                if let Some(hover) = hovered {
                    scene.hover(hover, palette, out.painter);
                }
                if let Some(drag) = pointer.drag.get() {
                    let mode = if layout.vertical_axis() { SelectionMode::Y } else { SelectionMode::X };
                    drag.paint(plot, mode, palette.cursor, out.painter);
                }
            });
        });
    });
    for (label, at) in &values {
        text::draw(painter, label, *at, TextAlign::Center, TEXT_SIZE, palette.text);
    }
    paint_labels(layout, palette, painter);
}

/// The pointer moved over the chart: hover (the nearest x; in single mode the nearest series),
/// annotation marks, and a drag in progress.
pub(crate) fn on_move(pointer: Pointer, event: &MouseMoveEvent) {
    let at = Point::from(event.position);
    if let Some(current) = pointer.drag.get() {
        // a release outside the chart ends the drag when the pointer comes back
        pointer.drag.set(event.pressed_button.map(|_| Drag { to: at, ..current }));
    }
    let (mark, hit) = pointer
        .surface
        .with(|layout| {
            let mark = find_mark(layout, at).map(|mark| MarkHover {
                mark,
                at: at - layout.bounds.origin,
                area: layout.bounds.size,
            });
            let hit = if mark.is_none() {
                pointer.cache.with(|cache| layout.hover(at, &cache.data, pointer.single))
            } else {
                None
            };
            (mark, hit)
        })
        .unwrap_or_default();
    if pointer.marks.get() != mark {
        pointer.marks.set(mark);
    }
    if pointer.hover.get() != hit {
        pointer.hover.set(hit);
    }
}

pub(crate) fn on_leave(pointer: Pointer) {
    if pointer.hover.get().is_some() {
        pointer.hover.set(None);
    }
    if pointer.marks.get().is_some() {
        pointer.marks.set(None);
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

/// A drag ended: its x range zooms the chart and goes to `on_select`.
pub(crate) fn on_up(pointer: Pointer, window: &mut Window, cx: &mut App) {
    let Some(done) = pointer.drag.get() else { return };
    pointer.drag.set(None);
    let picked = pointer.surface.with(|layout| {
        let (plot, logical, vertical) = (layout.plot, layout.logical, layout.vertical_axis());
        let flat = Drag { from: unrotate(plot, logical, vertical, done.from), to: unrotate(plot, logical, vertical, done.to) };
        let x = layout.model.x;
        flat.range(logical, SelectionMode::X, &x, &x).map(|range| (range, x))
    });
    let Some((range, x)) = picked.flatten() else { return };
    pointer.zoom.update(|zoom| zoom.set_x(range.x, &x));
    SelectSlot::call(pointer.select, range, window, cx);
}

pub(crate) fn on_wheel(pointer: Pointer, event: &ScrollWheelEvent, cx: &mut App) {
    wheel_zoom(event, pointer.zoom, cx, |at| {
        pointer.surface.with(|layout| {
            let (plot, logical, vertical, x) = (layout.plot, layout.logical, layout.vertical_axis(), layout.model.x);
            let flat = unrotate(plot, logical, vertical, at);
            (plot, (Some((x, fraction(flat.x, logical.left(), logical.size.width))), None))
        })
    });
}
const MARK_REACH: f32 = 6.;
const MARKER: f32 = 8.;
const NOTE: u32 = 0x00d3ff;

fn find_mark(layout: &Layout, cursor: Point) -> Option<Mark> {
    let at = layout.logical_point(cursor);
    let plot = layout.logical;
    let model = &layout.model;
    if (plot.bottom() - at.y) <= MARKER + MARK_REACH {
        let note = model.notes.iter().position(|note| {
            let start = layout.x(note.time);
            note.end.map_or_else(
                || (start - at.x).abs() <= MARK_REACH,
                |end| at.x >= start && at.x <= layout.x(end),
            )
        });
        if let Some(index) = note {
            return Some(Mark::Note(index));
        }
    }
    if model.axes.is_empty() {
        return None;
    }
    model
        .exemplars
        .iter()
        .position(|(x, y)| {
            let (dx, dy) = (layout.x(*x) - at.x, layout.y(0, *y) - at.y);
            dx.abs().max(dy.abs()) <= MARK_REACH
        })
        .map(Mark::Exemplar)
}

fn rect(left: f32, right: f32, top: f32, bottom: f32) -> Path {
    let mut builder = Path::new();
    builder.add_polygon(
        &[
            point(left, top),
            point(right, top),
            point(right, bottom),
            point(left, bottom),
        ],
        true,
    );
    builder
}

fn column(layout: &Layout, xs: &[f64]) -> f32 {
    let mut previous: Option<f32> = None;
    let mut gap = layout.logical.size.width;
    for x in xs.iter().filter(|x| x.is_finite()) {
        let at = layout.x(*x);
        if let Some(previous) = previous {
            let distance = at - previous;
            if distance > 0. {
                gap = gap.min(distance);
            }
        }
        previous = Some(at);
    }
    gap
}

#[derive(Default)]
struct Scratch {
    indices: Vec<usize>,
    top: Vec<Point>,
    bottom: Vec<Point>,
}

impl Scene<'_> {
    const fn model(&self) -> &Model {
        &self.layout.model
    }

    fn draw(&self, out: &mut Out<'_>, mut path: Path, ink: &Ink) {
        if path.is_empty() {
            return;
        }
        if self.layout.vertical_axis() {
            path.map(|at| self.layout.screen(at));
        }
        ink.paint(&path, self.layout.plot, self.layout.vertical_axis(), out.painter);
        if let Some(record) = &mut out.record {
            record.push((path, *ink));
        }
    }

    fn fill(&self, out: &mut Out<'_>, path: Path, shade: &Shade) {
        self.draw(out, path, &Ink { shade: *shade, stroke: None });
    }

    fn stroke(&self, out: &mut Out<'_>, path: Path, shade: &Shade, width: f32, style: LineStyle) {
        self.draw(out, path, &Ink { shade: *shade, stroke: Some((width, style)) });
    }

    fn steps_for<'a>(&'a self, series: &'a Plotted) -> crate::thresholds::Steps<'a> {
        steps(
            series
                .thresholds
                .as_ref()
                .or_else(|| self.model().thresholds.as_ref()),
            &self.fallback,
        )
    }

    fn range_for(&self, series: &Plotted) -> (f64, f64) {
        let config = &self.model().axes[series.axis].config;
        let line = &self.data.lines[series.index];
        let (low, high) = Extent::of(&line.values)
            .for_scale(config.scale)
            .unwrap_or((0., 100.));
        (
            config.min.or(config.soft_min).unwrap_or(low),
            config.max.or(config.soft_max).unwrap_or(high),
        )
    }

    fn scheme_stops(&self, series: &Plotted, alpha: f32, force_thresholds: bool) -> Stops {
        let scale = &self.model().axes[series.axis].scale;
        let range = self.range_for(series);
        let scheme = series
            .scheme
            .as_ref()
            .or_else(|| self.model().scheme.as_ref());
        match scheme.filter(|_| !force_thresholds) {
            Some(scheme) => Stops::scheme(scheme.stops(self.colors), range, scale, alpha),
            None => Stops::thresholds(self.steps_for(series), range, scale, alpha),
        }
    }

    fn line_shade(&self, series: &Plotted) -> Shade {
        let style = &series.style;
        if style.thresholds_style() == ThresholdsStyle::Series {
            return Shade::Stops(self.scheme_stops(series, 1., true));
        }
        match style.gradient() {
            GradientMode::Scheme => Shade::Stops(self.scheme_stops(series, 1., false)),
            GradientMode::Hue => {
                let (top, bottom) = hue(series.color, 1., self.dark);
                Shade::Vertical(top, bottom)
            }
            GradientMode::None | GradientMode::Opacity => Shade::Solid(series.color),
        }
    }

    fn fill_shade(&self, series: &Plotted, opacity: f32) -> Option<Shade> {
        let style = &series.style;
        if style.thresholds_style() == ThresholdsStyle::Series {
            return (opacity > 0.).then(|| Shade::Stops(self.scheme_stops(series, opacity, true)));
        }
        match style.gradient() {
            GradientMode::Opacity => Some(Shade::Vertical(
                series.color.alpha(opacity),
                series.color.alpha(0.),
            )),
            GradientMode::Hue => {
                let (top, bottom) = hue(series.color, opacity, self.dark);
                Some(Shade::Vertical(top, bottom))
            }
            GradientMode::Scheme => Some(Shade::Stops(self.scheme_stops(series, opacity, false))),
            GradientMode::None => {
                (opacity > 0.).then(|| Shade::Solid(with_alpha(series.color, opacity)))
            }
        }
    }

    fn thresholds(&self, out: &mut Out<'_>) {
        let layout = self.layout;
        let plot = layout.logical;
        for (axis, _) in self.model().axes.iter().enumerate() {
            let Some(series) = self.model().visible().find(|series| {
                series.axis == axis && series.style.thresholds_style() != ThresholdsStyle::Off
            }) else {
                continue;
            };
            let mode = series.style.thresholds_style();
            let scale = &self.model().axes[axis].scale;
            let range = self.range_for(series);
            let steps = self.steps_for(series);
            if mode.area() {
                let stops = Stops::thresholds(steps, range, scale, THRESHOLD_AREA_ALPHA);
                let builder = rect(plot.left(), plot.right(), plot.top(), plot.bottom());
                self.fill(out, builder, &Shade::Stops(stops));
            }
            let Some(dash) = mode.line() else { continue };
            for (value, color) in steps.steps.iter().skip(1) {
                let y = layout.y(axis, steps.value(*value, range));
                let style = if dash {
                    LineStyle::DASH
                } else {
                    LineStyle::Solid
                };
                let builder = Path::polyline([point(plot.left(), y), point(plot.right(), y)]);
                self.stroke(out, builder, &Shade::Solid(with_alpha(*color, THRESHOLD_LINE_ALPHA)), 2., style);
            }
        }
    }

    fn series(&self, series: &Plotted, scratch: &mut Scratch, out: &mut Out<'_>) {
        let style = &series.style;
        let line = &self.data.lines[series.index];
        match style.draw_style() {
            DrawStyle::Points => {}
            DrawStyle::Bars => self.bars(series, line, out),
            DrawStyle::Line if style.transform() == ValueTransform::Constant => {
                self.constant(series, line, out);
            }
            DrawStyle::Line => self.lines(series, line, scratch, out),
        }
    }

    fn constant(&self, series: &Plotted, line: &Line, out: &mut Out<'_>) {
        let Some(value) = line.values.iter().copied().find(|value| value.is_finite()) else {
            return;
        };
        let plot = self.layout.logical;
        let y = self.layout.y(series.axis, value);
        let style = &series.style;
        let width = style.line_width();
        if width > 0. {
            let builder = Path::polyline([point(plot.left(), y), point(plot.right(), y)]);
            self.stroke(out, builder, &self.line_shade(series), width, style.line_style());
        }
    }

    fn lines(&self, series: &Plotted, line: &Line, scratch: &mut Scratch, out: &mut Out<'_>) {
        let layout = self.layout;
        let style = series.style;
        let interpolation = style.interpolation();
        let below = series.below.map(|position| &self.model().series[position]);
        let mut opacity = style.fill_opacity();
        if below.is_some() && opacity == 0. {
            opacity = BAND_OPACITY;
        }
        let fill = self.fill_shade(series, opacity);
        let stroke = self.line_shade(series);
        let width = style.line_width();
        let xs = &self.data.xs;
        let baseline = self.model().axes[series.axis].scale.baseline();
        let lower = |index: usize| -> f64 {
            match below {
                Some(other) => self.data.lines[other.index]
                    .values
                    .get(index)
                    .copied()
                    .unwrap_or(f64::NAN),
                None => line.base.get(index).copied().unwrap_or(baseline),
            }
        };
        runs(xs, &line.values, &style, &mut scratch.indices, |indices| {
            scratch.top.clear();
            scratch.top.extend(indices.iter().map(|&index| {
                point(
                    layout.x(xs[index]),
                    layout.y(series.axis, line.values[index]),
                )
            }));
            if let Some(fill) = fill {
                scratch.bottom.clear();
                let mut bounded = true;
                for &index in indices.iter().rev() {
                    let value = lower(index);
                    if !value.is_finite() {
                        bounded = false;
                        break;
                    }
                    let axis = below.map_or(series.axis, |other| other.axis);
                    scratch
                        .bottom
                        .push(point(layout.x(xs[index]), layout.y(axis, value)));
                }
                if bounded && !scratch.top.is_empty() {
                    let mut builder = Path::with_capacity(scratch.top.len() + scratch.bottom.len() + 1);
                    builder.move_to(scratch.top[0]);
                    trace(&mut builder, &scratch.top, interpolation);
                    builder.line_to(scratch.bottom[0]);
                    trace(&mut builder, &scratch.bottom, interpolation.reversed());
                    builder.close();
                    self.fill(out, builder, &fill);
                }
            }
            if width > 0. && scratch.top.len() > 1 {
                let mut builder = Path::with_capacity(scratch.top.len());
                builder.move_to(scratch.top[0]);
                trace(&mut builder, &scratch.top, interpolation);
                self.stroke(out, builder, &stroke, width, style.line_style());
            }
        });
    }

    fn bars(&self, series: &Plotted, line: &Line, out: &mut Out<'_>) {
        let layout = self.layout;
        let style = &series.style;
        let bar = style
            .bar_max_width()
            .min(style.bar_width_factor() * self.column)
            .max(1.);
        let shift = match style.bar_alignment() {
            BarAlignment::Before => -bar,
            BarAlignment::Center => -bar / 2.,
            BarAlignment::After => 0.,
        };
        let baseline = self.model().axes[series.axis].scale.baseline();
        let mut body = Path::new();
        let width = style.line_width();
        let mut any = false;
        for (index, (x, value)) in self.data.xs.iter().zip(&line.values).enumerate() {
            if !x.is_finite() || !value.is_finite() {
                continue;
            }
            let (left, right) = if let Some(end) = self.data.ends.get(index) {
                (layout.x(*x), layout.x(*end))
            } else {
                let left = layout.x(*x) + shift;
                (left, left + bar)
            };
            let base = line.base.get(index).copied().unwrap_or(baseline);
            let (top, bottom) = (layout.y(series.axis, *value), layout.y(series.axis, base));
            let corners = [
                point(left, top),
                point(right, top),
                point(right, bottom),
                point(left, bottom),
            ];
            body.add_polygon(&corners, true);
            any = true;
        }
        if !any {
            return;
        }
        let opacity = style.fill_opacity();
        if width > 0. {
            self.stroke(out, body.clone(), &self.line_shade(series), width, LineStyle::Solid);
        }
        if let Some(fill) = self.fill_shade(series, opacity) {
            self.fill(out, body, &fill);
        }
    }

    fn points(
        &self,
        series: &Plotted,
        labels: &mut Vec<(SharedString, Point)>,
        painter: &mut Painter,
    ) {
        let layout = self.layout;
        let style = &series.style;
        let line = &self.data.lines[series.index];
        let xs = &self.data.xs;
        let diameter = style.point_size().max(style.line_width());
        let all = match (style.draw_style(), style.show_points()) {
            (DrawStyle::Points, _) | (_, PointsMode::Always) => true,
            (_, PointsMode::Never) | (DrawStyle::Bars, PointsMode::Auto) => false,
            (DrawStyle::Line, PointsMode::Auto) => {
                let shown = xs
                    .iter()
                    .zip(&line.values)
                    .filter(|(x, y)| x.is_finite() && y.is_finite())
                    .count();
                shown as f32 <= layout.logical.size.width / (diameter * 2.)
            }
        };
        let isolated = !all
            && style.show_points() == PointsMode::Auto
            && style.draw_style() == DrawStyle::Line
            && !matches!(style.span_nulls(), super::options::NullSpan::Always);
        let finite = |index: usize| {
            line.values
                .get(index)
                .is_some_and(|value| value.is_finite())
        };
        let mut last_label: Option<f32> = None;
        let line_shade = self.line_shade(series);
        for (index, (x, value)) in xs.iter().zip(&line.values).enumerate() {
            if !x.is_finite() || !value.is_finite() {
                continue;
            }
            let at = point(
                layout.x(self.data.center(index)),
                layout.y(series.axis, *value),
            );
            let lonely = isolated && !finite(index.wrapping_sub(1)) && !finite(index + 1);
            let drawn = all || lonely;
            if drawn {
                let color = match line_shade {
                    Shade::Solid(color) | Shade::Vertical(color, _) => color,
                    Shade::Stops(stops) => {
                        let fraction =
                            1. - ((at.y - layout.logical.top()) / layout.logical.size.height);
                        stops.color(fraction).unwrap_or(series.color)
                    }
                };
                let side = diameter;
                let dot = Bounds::centered_at(layout.screen(at), size(side, side));
                painter.quad(dot.into(), side / 2., color, color, 0.);
            }
            let labelled = style.show_values() && (drawn || style.draw_style() == DrawStyle::Bars);
            if labelled {
                let label = self.model().axes[series.axis]
                    .config
                    .text(line.raw.get(index).copied().unwrap_or(*value), None);
                let half = text::measure(&label, TEXT_SIZE) / 2.;
                if last_label.is_none_or(|edge| at.x - half > edge + TEXT_GAP) {
                    last_label = Some(at.x + half);
                    let spot = layout.screen(at);
                    let offset = diameter / 2. + TEXT_GAP;
                    let place = if layout.vertical_axis() {
                        point(spot.x + offset + half, spot.y - (TEXT_SIZE / 2.))
                    } else {
                        point(spot.x, spot.y - (offset + TEXT_SIZE))
                    };
                    labels.push((label, place));
                }
            }
        }
    }

    fn candles(&self, background: Hsla, out: &mut Out<'_>) {
        let Some(look) = self.model().candles else {
            return;
        };
        let layout = self.layout;
        let find = |part: Part| {
            self.model()
                .visible()
                .find(|series| series.part == Some(part))
        };
        let raw =
            |series: Option<&Plotted>| series.map(|series| &self.data.lines[series.index].raw);
        let (open, close) = (find(Part::Open), find(Part::Close));
        let Some(axis) = open.or(close).map(|series| series.axis) else {
            return;
        };
        let (opens, closes) = (raw(open), raw(close));
        let (highs, lows) = (raw(find(Part::High)), raw(find(Part::Low)));
        let volume = find(Part::Volume).filter(|_| look.mode.volume());
        let volumes = raw(volume);
        let at = |values: Option<&Vec<f64>>, index: usize| {
            values
                .and_then(|values| values.get(index))
                .copied()
                .unwrap_or(f64::NAN)
        };
        let open_at = |index: usize| {
            if opens.is_some() {
                at(opens, index)
            } else {
                at(closes, index.saturating_sub(1))
            }
        };
        let close_at = |index: usize| {
            if closes.is_some() {
                at(closes, index)
            } else {
                at(opens, (index + 1).min(self.data.xs.len().saturating_sub(1)))
            }
        };
        let width = (0.6 * self.column).round().max(1.);
        let (stick, outline): (f32, f32) = if width <= 12. { (1., 1.) } else { (2., 2.) };
        let half = (width / 2.).floor();
        let mut bodies = [Path::new(), Path::new(), Path::new()];
        let mut bars = [Path::new(), Path::new(), Path::new()];
        let mut hollow = Path::new();
        let px_rect = |builder: &mut Path, left: f32, top: f32, w: f32, h: f32| {
            builder.add_polygon(
                &[
                    point(left, top),
                    point(left + w, top),
                    point(left + w, top + h),
                    point(left, top + h),
                ],
                true,
            );
        };
        for (index, x) in self.data.xs.iter().enumerate() {
            let (o, c) = (open_at(index), close_at(index));
            if !x.is_finite() || !o.is_finite() || !c.is_finite() {
                continue;
            }
            let t = layout.x(*x).round();
            let inter = if index == 0 {
                0.
            } else {
                (c - close_at(index - 1)).signum()
            };
            let intra = if (c - o).abs() <= f64::EPSILON {
                0.
            } else {
                (c - o).signum()
            };
            let direction = match look.strategy {
                ColorStrategy::CloseClose => inter,
                ColorStrategy::OpenClose => intra,
            };
            let flat_as_up = !(index == 0 && look.strategy == ColorStrategy::CloseClose);
            let slot = if direction > 0. || (direction == 0. && flat_as_up) {
                0
            } else if direction < 0. {
                1
            } else {
                2
            };
            if let Some(series) = volume {
                let value = at(volumes, index);
                if value.is_finite() {
                    let top = layout.y(series.axis, value).round();
                    let zero = layout.y(series.axis, 0.).round();
                    px_rect(&mut bars[slot], t - half, top, width, zero - top);
                }
            }
            if !look.mode.price() {
                continue;
            }
            let high = at(highs, index);
            let low = at(lows, index);
            let (high, low) = (
                if high.is_finite() { high } else { o.max(c) },
                if low.is_finite() { low } else { o.min(c) },
            );
            let y = |value: f64| layout.y(axis, value).round();
            let (high_px, low_px) = (y(high), y(low));
            px_rect(
                &mut bodies[slot],
                t - (stick / 2.).floor(),
                high_px,
                stick,
                low_px - high_px,
            );
            let (open_px, close_px) = (y(o), y(c));
            match look.style {
                CandleStyle::Candles => {
                    let top = open_px.min(close_px);
                    let height = (open_px.max(close_px) - top).max(1.);
                    px_rect(&mut bodies[slot], t - half, top, width, height);
                    if look.strategy == ColorStrategy::CloseClose
                        && intra >= 0.
                        && height > outline * 2.
                    {
                        px_rect(
                            &mut hollow,
                            t - half + outline,
                            top + outline,
                            outline.mul_add(-2., width),
                            outline.mul_add(-2., height),
                        );
                    }
                }
                CandleStyle::OhlcBars => {
                    px_rect(&mut bodies[slot], t - half, open_px, half, stick);
                    px_rect(&mut bodies[slot], t, close_px, half, stick);
                }
            }
        }
        let colors = [look.up, look.down, look.flat];
        for (builder, color) in bars.into_iter().zip(colors) {
            self.fill(out, builder, &Shade::Solid(color.opacity(look.volume_alpha)));
        }
        for (builder, color) in bodies.into_iter().zip(colors) {
            self.fill(out, builder, &Shade::Solid(color));
        }
        self.fill(out, hollow, &Shade::Solid(background));
    }

    fn marks(&self, out: &mut Out<'_>) {
        let layout = self.layout;
        let model = self.model();
        let plot = layout.logical;
        let marker_top = plot.bottom() - MARKER;
        let half = MARKER / 2. + 1.;
        for note in &model.notes {
            let color = note.color.unwrap_or_else(|| rgb(NOTE));
            let start = layout.x(note.time);
            if let Some(end) = note.end {
                let stop = layout.x(end);
                let band = rect(start, stop, plot.top(), plot.bottom());
                self.fill(out, band, &Shade::Solid(color.opacity(0.12)));
                let bar = rect(start, stop, marker_top, plot.bottom());
                self.fill(out, bar, &Shade::Solid(color));
            } else {
                let line = Path::polyline([point(start, plot.top()), point(start, plot.bottom())]);
                self.stroke(out, line, &Shade::Solid(color.opacity(0.8)), 1., LineStyle::Dash(&[4., 4.]));
                let mut marker = Path::new();
                marker.add_polygon(
                    &[
                        point(start, marker_top),
                        point(start + half, plot.bottom()),
                        point(start - half, plot.bottom()),
                    ],
                    true,
                );
                self.fill(out, marker, &Shade::Solid(color));
            }
        }
        if model.axes.is_empty() {
            return;
        }
        for (x, y) in &model.exemplars {
            if !x.is_finite() || !y.is_finite() {
                continue;
            }
            let center = point(layout.x(*x), layout.y(0, *y));
            let mut diamond = Path::new();
            diamond.add_polygon(
                &[
                    point(center.x, center.y - half),
                    point(center.x + half, center.y),
                    point(center.x, center.y + half),
                    point(center.x - half, center.y),
                ],
                true,
            );
            self.fill(out, diamond, &Shade::Solid(model.exemplar_color));
        }
    }

    fn hover(&self, hover: Hover, palette: Palette, painter: &mut Painter) {
        let layout = self.layout;
        if !self.data.xs.get(hover.index).is_some_and(|x| x.is_finite()) {
            return;
        }
        let plot = layout.logical;
        let at = layout.x(self.data.center(hover.index));
        let band = match self.data.ends.get(hover.index) {
            Some(end) => {
                let left = layout.x(self.data.xs[hover.index]);
                Bounds::new(
                    point(left, plot.top()),
                    size(layout.x(*end) - left, plot.size.height),
                )
            }
            None => Bounds::new(point(at, plot.top()), size(1., plot.size.height)),
        };
        let cursor = if self.data.ends.is_empty() { palette.cursor } else { palette.cursor.opacity(0.3) };
        painter.fill(layout.screen_bounds(band).into(), cursor);
        for series in layout.model.visible() {
            let Some(value) = self.data.lines[series.index]
                .values
                .get(hover.index)
                .filter(|value| value.is_finite())
            else {
                continue;
            };
            let focused = hover.series.is_none_or(|index| index == series.index);
            let side = series.style.point_size().max(4.) * if focused { 2. } else { 1.5 };
            let center = layout.screen(point(at, layout.y(series.axis, *value)));
            let dot = Bounds::centered_at(center, size(side, side));
            painter.quad(dot.into(), side / 2., series.color.alpha(HOVER_ALPHA), series.color, 1.);
        }
    }
}

impl Interpolation {
    const fn reversed(self) -> Self {
        match self {
            Self::StepBefore => Self::StepAfter,
            Self::StepAfter => Self::StepBefore,
            other => other,
        }
    }
}

fn runs(
    xs: &[f64],
    values: &[f64],
    style: &Style,
    indices: &mut Vec<usize>,
    mut each: impl FnMut(&[usize]),
) {
    indices.clear();
    let (span, limit) = (style.span_nulls(), style.insert_nulls());
    let mut last: Option<usize> = None;
    let mut gap = false;
    for (index, (x, value)) in xs.iter().zip(values).enumerate() {
        if !x.is_finite() {
            continue;
        }
        if !value.is_finite() {
            gap = true;
            continue;
        }
        if let Some(previous) = last {
            let distance = x - xs[previous];
            if distance > limit || (gap && !span.bridges(distance)) {
                each(indices);
                indices.clear();
            }
        }
        indices.push(index);
        last = Some(index);
        gap = false;
    }
    if !indices.is_empty() {
        each(indices);
    }
}

pub(crate) fn trace(
    builder: &mut Path,
    points: &[Point],
    interpolation: Interpolation,
) {
    match interpolation {
        Interpolation::Linear => points.iter().skip(1).for_each(|at| builder.line_to(*at)),
        Interpolation::StepAfter => {
            for pair in points.windows(2) {
                builder.line_to(point(pair[1].x, pair[0].y));
                builder.line_to(pair[1]);
            }
        }
        Interpolation::StepBefore => {
            for pair in points.windows(2) {
                builder.line_to(point(pair[0].x, pair[1].y));
                builder.line_to(pair[1]);
            }
        }
        Interpolation::Smooth => smooth(builder, points),
    }
}

fn smooth(builder: &mut Path, points: &[Point]) {
    let count = points.len();
    if count < 3 {
        points.iter().skip(1).for_each(|at| builder.line_to(*at));
        return;
    }
    let delta = |k: usize| {
        let (a, b) = (points[k], points[k + 1]);
        let dx = b.x - a.x;
        let slope = if dx == 0. {
            0.
        } else {
            (b.y - a.y) / dx
        };
        (dx, slope)
    };
    let tangent = |k: usize| -> f32 {
        if k == 0 {
            return delta(0).1;
        }
        if k == count - 1 {
            return delta(count - 2).1;
        }
        let ((h0, d0), (h1, d1)) = (delta(k - 1), delta(k));
        if d0 * d1 <= 0. {
            0.
        } else {
            3. * (h0 + h1) / ((2. * h1 + h0) / d0 + (h1 + 2. * h0) / d1)
        }
    };
    let mut previous = tangent(0);
    for k in 0..count - 1 {
        let next = tangent(k + 1);
        let (a, b) = (points[k], points[k + 1]);
        let third = (b.x - a.x) / 3.;
        builder.cubic_to(
            b,
            point(a.x + third, a.y + (previous * third)),
            point(b.x - third, b.y - (next * third)),
        );
        previous = next;
    }
}

fn paint_grid(layout: &Layout, palette: Palette, painter: &mut Painter) {
    let (plot, origin) = (layout.plot, layout.bounds.origin);
    let model = &layout.model;
    let vertical = model.vertical;
    let mut line = |from: Point, width: f32, height: f32, color: Hsla| {
        painter.fill(Bounds::new(from, size(width, height)).into(), color);
    };
    let (width, height, thin) = (plot.size.width, plot.size.height, 1.);
    let across = |at: f32| point(plot.left(), origin.y + at);
    let down = |at: f32| point(origin.x + at, plot.top());
    if model.x_axis.grid
        && let Some(row) = layout.x_rows.first()
    {
        for at in row.labels.positions() {
            if vertical {
                line(across(at), width, thin, palette.grid);
            } else {
                line(down(at), thin, height, palette.grid);
            }
        }
    }
    if let Some(labels) = layout
        .y_labels
        .iter()
        .find(|labels| model.axes[labels.axis].config.grid)
    {
        for at in labels.labels.positions() {
            if vertical {
                line(down(at), thin, height, palette.grid);
            } else {
                line(across(at), width, thin, palette.grid);
            }
        }
    }
    if model.x_axis.visible() && model.x_axis.border {
        if vertical {
            line(plot.origin, thin, height, palette.border);
        } else {
            let y = if model.x_axis.placement == AxisPlacement::Top {
                plot.top()
            } else {
                plot.bottom()
            };
            line(point(plot.left(), y), width, thin, palette.border);
        }
    }
    for labels in &layout.y_labels {
        if model.axes[labels.axis].config.border {
            if vertical {
                let y = if model.axes[labels.axis].right {
                    plot.top()
                } else {
                    plot.bottom()
                };
                line(point(plot.left(), y), width, thin, palette.border);
            } else {
                line(down(labels.edge), thin, height, palette.border);
            }
        }
    }
}

/// A centred label's x, moved so the label stays inside a canvas `width` wide.
pub(crate) fn inside(label: &str, x: f32, width: f32) -> f32 {
    let half = text::measure(label, TEXT_SIZE) / 2.;
    x.clamp(half, (width - half).max(half))
}

fn paint_labels(layout: &Layout, palette: Palette, painter: &mut Painter) {
    let origin = layout.bounds.origin;
    let plot = Bounds::new(layout.plot.origin - origin, layout.plot.size);
    let (left, top, right) = (
        plot.left(),
        plot.top(),
        plot.right(),
    );
    let mut text = |label: &str, x: f32, y: f32, align: TextAlign, color: Hsla| {
        text::draw(painter, label, origin + point(x, y), align, TEXT_SIZE, color);
    };
    let vertical = layout.model.vertical;
    let x_axis = &layout.model.x_axis;
    if x_axis.visible() {
        let on_top = x_axis.placement == AxisPlacement::Top;
        let row_height = TEXT_HEIGHT + TEXT_GAP;
        let mut last = 0.;
        for row in &layout.x_rows {
            if vertical {
                for (y, tick) in row.labels.iter() {
                    text(
                        tick,
                        TEXT_GAP.mul_add(-2., row.edge),
                        y - TEXT_SIZE / 2.,
                        TextAlign::Right,
                        palette.text,
                    );
                }
                continue;
            }
            let y = if on_top {
                row.edge + row_height - TEXT_GAP - TEXT_SIZE
            } else {
                row.edge + TEXT_GAP
            };
            last = y;
            for (x, tick) in row.labels.iter() {
                text(tick, inside(tick, x, layout.bounds.size.width), y, TextAlign::Center, palette.text);
            }
        }
        if let Some(title) = &x_axis.label {
            if vertical {
                text(
                    title,
                    PADDING,
                    top - LABEL_HEIGHT,
                    TextAlign::Left,
                    palette.text,
                );
            } else {
                let y = if on_top {
                    last - LABEL_HEIGHT
                } else {
                    last + TEXT_HEIGHT + TEXT_GAP
                };
                text(
                    title,
                    f32::midpoint(left, right),
                    y,
                    TextAlign::Center,
                    palette.text,
                );
            }
        }
    }
    let model = &layout.model;
    for labels in &layout.y_labels {
        let axis = &model.axes[labels.axis];
        let color = match axis.config.color_mode {
            AxisColorMode::Series => axis.color,
            AxisColorMode::Text => palette.text,
        };
        if vertical {
            let y = labels.edge + TEXT_GAP;
            for (x, tick) in labels.labels.iter() {
                text(tick, x, y, TextAlign::Center, color);
            }
            if let Some(title) = &axis.config.label {
                text(
                    title,
                    f32::midpoint(left, right),
                    y + TEXT_HEIGHT + TEXT_GAP,
                    TextAlign::Center,
                    palette.text,
                );
            }
            continue;
        }
        let (x, align) = if axis.right {
            (TEXT_GAP.mul_add(2., labels.edge), TextAlign::Left)
        } else {
            (TEXT_GAP.mul_add(-2., labels.edge), TextAlign::Right)
        };
        for (y, tick) in labels.labels.iter() {
            text(tick, x, y - TEXT_SIZE / 2., align, color);
        }
        if let Some(title) = &axis.config.label {
            // aligned to the axis, kept inside the canvas
            let width = text::measure(title, TEXT_SIZE);
            let x = if axis.right {
                labels.edge.min(layout.bounds.size.width - width)
            } else {
                (labels.edge - width).max(0.)
            };
            text(title, x, PADDING, TextAlign::Left, palette.text);
        }
    }
}
