use std::collections::BTreeSet;
use std::f32::consts::TAU;

use blenderui::element::{AnyElement, Div, FluentBuilder, IntoElement, ParentElement};
use blenderui::hooks::{Signal, use_signal};
use blenderui::html::div;
use blenderui::style::TextAlign;
use blenderui::{Hsla, MouseMoveEvent, Painter, SharedString, Styled, canvas};

use super::scene::{Camera, Zoom, camera_input};
use super::{Tip, card, scene};
use crate::derive::data_key;
use crate::geom::{Bounds, Point, Size, point, size};
use crate::legend::{LegendItem, legend_panel};
use crate::path::Path;
use crate::plot::{TEXT_SIZE, Tooltip, TooltipMode};
use crate::trend::TrendField;
use crate::{ChartTheme, Dataset, Legend, component, text};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SpiderGrid {
    #[default]
    Polygon,
    Circle,
}

#[derive(Clone, Copy, PartialEq)]
enum Shape {
    Spider(SpiderGrid),
    Rose,
    Pattern,
    Parallel,
}

#[derive(Default)]
struct Web {
    key: Option<u64>,
    axes: Vec<SharedString>,
    angles: Vec<f32>,
    groups: Vec<SharedString>,
    names: Vec<SharedString>,
    members: Vec<usize>,
    values: Vec<f64>,
    extents: Vec<(f64, f64)>,
}

impl Web {
    fn value(&self, row: usize, axis: usize) -> f64 {
        self.values
            .get(row * self.axes.len() + axis)
            .copied()
            .unwrap_or(f64::NAN)
    }

    fn uniform(mut self, offset: f32) -> Self {
        let count = self.axes.len().max(1) as f32;
        self.angles = (0..self.axes.len())
            .map(|axis| TAU * (axis as f32 + offset) / count)
            .collect();
        self
    }

    fn measured(mut self) -> Self {
        self.extents = (0..self.axes.len())
            .map(|axis| {
                let finite = (0..self.names.len())
                    .map(|row| self.value(row, axis))
                    .filter(|value| value.is_finite());
                let (low, high) = finite
                    .fold((f64::INFINITY, f64::NEG_INFINITY), |(low, high), value| {
                        (low.min(value), high.max(value))
                    });
                if low <= high { (low, high) } else { (0., 1.) }
            })
            .collect();
        self
    }
}

fn rows_by<T>(
    rows: &[T],
    axes: Vec<SharedString>,
    name: impl Fn(&T) -> &str,
    group: Option<&dyn Fn(&T) -> &str>,
    value: impl Fn(&T, usize) -> f64,
) -> Web {
    let mut groups: Vec<SharedString> = Vec::new();
    let members = rows
        .iter()
        .enumerate()
        .map(|(index, item)| {
            group.map_or(index, |group| {
                let label = group(item);
                groups
                    .iter()
                    .position(|known| known == label)
                    .unwrap_or_else(|| {
                        groups.push(SharedString::from(label.to_owned()));
                        groups.len() - 1
                    })
            })
        })
        .collect();
    let names: Vec<SharedString> = rows
        .iter()
        .map(|item| SharedString::from(name(item).to_owned()))
        .collect();
    if group.is_none() {
        groups.clone_from(&names);
    }
    let values = rows
        .iter()
        .flat_map(|item| {
            (0..axes.len())
                .map(|axis| value(item, axis))
                .collect::<Vec<_>>()
        })
        .collect();
    Web {
        axes,
        groups,
        names,
        members,
        values,
        ..Web::default()
    }
}

fn refresh(web: Signal<Web>, key: Option<u64>, build: impl FnOnce() -> Web) {
    if key.is_some() && web.with(|web| web.key) == key {
        return;
    }
    let mut next = build();
    next.key = key;
    web.update_untracked(|web| *web = next);
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Hover {
    axis: usize,
    row: Option<usize>,
    at: Point,
    area: Size,
}

#[derive(Clone, Copy)]
struct Look {
    shape: Shape,
    levels: usize,
    fill: f32,
    line: f32,
    min: f64,
    max: f64,
    text: Hsla,
    rule: Hsla,
}

impl Look {
    fn new(
        shape: Shape,
        levels: usize,
        (fill, line): (f32, f32),
        (min, max): (f64, f64),
    ) -> Self {
        let theme = crate::ui::theme();
        Self {
            shape,
            levels: levels.max(1),
            fill,
            line,
            min,
            max,
            text: theme.muted_foreground,
            rule: theme.chart_grid,
        }
    }

    fn fraction(&self, value: f64) -> f32 {
        let span = self.max - self.min;
        if span > 0. && value.is_finite() {
            ((value - self.min) / span).clamp(0., 1.) as f32
        } else {
            0.
        }
    }

    fn format(&self, value: f64) -> SharedString {
        let shown = crate::format::trimmed(value);
        if self.shape == Shape::Pattern {
            format!("{shown} dB").into()
        } else {
            shown
        }
    }
}

fn peak(web: Signal<Web>, stacked: bool) -> f64 {
    web.with(|web| peak_of(web, stacked))
}

fn peak_of(web: &Web, stacked: bool) -> f64 {
    let finite = |value: f64| if value.is_finite() { value.max(0.) } else { 0. };
    if stacked {
        (0..web.axes.len())
            .map(|axis| {
                (0..web.names.len())
                    .map(|row| finite(web.value(row, axis)))
                    .sum()
            })
            .fold(0., f64::max)
    } else {
        web.values.iter().copied().map(finite).fold(0., f64::max)
    }
}

const MARGIN: f32 = 48.;
const STEPS: usize = 72;
const PAD: (f32, f32, f32, f32) = (56., 56., 28., 48.);
const REACH: f32 = 12.;

fn radial(center: Point, radius: f32, angle: f32) -> Point {
    let at = scene::polar(point(0., 0.), radius, angle);
    center + point(at.x, at.y)
}

fn stroke(points: &[Point], closed: bool, width: f32, color: Hsla, painter: &mut Painter) {
    if points.len() < 2 {
        return;
    }
    let mut path = Path::with_capacity(points.len() + 1);
    path.add_polygon(points, closed);
    path.stroke(painter, color, width);
}

fn shade(points: &[Point], color: Hsla, painter: &mut Painter) {
    if points.len() >= 3 {
        Path::polygon(points).fill(painter, color);
    }
}

fn dot(at: Point, side: f32, color: Hsla, painter: &mut Painter) {
    painter.quad(Bounds::centered_at(at, size(side, side)).into(), side / 2., color, Hsla::TRANSPARENT, 0.);
}

fn ring(center: Point, radius: f32, angles: &[f32]) -> Vec<Point> {
    angles
        .iter()
        .map(|angle| radial(center, radius, *angle))
        .collect()
}

fn circle() -> Vec<f32> {
    (0..STEPS)
        .map(|step| TAU * step as f32 / STEPS as f32)
        .collect()
}

fn wedge(center: Point, radii: (f32, f32), angles: (f32, f32)) -> Vec<Point> {
    scene::wedge(point(0., 0.), radii, angles)
        .into_iter()
        .map(|at| center + point(at.x, at.y))
        .collect()
}

fn gap(angle: f32, angles: &[f32]) -> Option<usize> {
    angles
        .iter()
        .enumerate()
        .min_by(|a, b| {
            let distance = |value: f32| {
                let delta = (angle - value).rem_euclid(TAU);
                delta.min(TAU - delta)
            };
            distance(*a.1).total_cmp(&distance(*b.1))
        })
        .map(|(axis, _)| axis)
}

fn anchor(at: Point, center: Point) -> (Point, TextAlign) {
    let align = if (at.x - center.x).abs() < 4. {
        TextAlign::Center
    } else if at.x > center.x {
        TextAlign::Left
    } else {
        TextAlign::Right
    };
    let lift = if at.y < center.y { TEXT_SIZE } else { 0. };
    (at - point(0., lift), align)
}

type Label = (SharedString, Point, TextAlign);

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn paint_polar(
    bounds: Bounds,
    web: Signal<Web>,
    hidden: Signal<BTreeSet<usize>>,
    hover: Signal<Option<Hover>>,
    look: Look,
    painter: &mut Painter,
) {
    let (center, radius) = polar_frame(bounds);
    let colors = ChartTheme::colors();
    let focus = hover.get().map(|hover| hover.axis);
    let mut labels: Vec<Label> = Vec::new();
    web.with(|current| hidden.with(|hidden| {
        let count = current.axes.len();
        let shown = |row: &usize| !hidden.contains(&current.members[*row]);
        let outline = match look.shape {
            Shape::Spider(SpiderGrid::Polygon) => current.angles.clone(),
            _ => circle(),
        };
        for level in 1..=look.levels {
            let r = radius * level as f32 / look.levels as f32;
            stroke(&ring(center, r, &outline), true, 1., look.rule, painter);
            if look.shape == Shape::Pattern {
                let value =
                    (look.max - look.min).mul_add(level as f64 / look.levels as f64, look.min);
                labels.push((
                    look.format(value),
                    radial(center, r, 0.) + point(4., 0.),
                    TextAlign::Left,
                ));
            }
        }
        let spokes: Vec<(f32, Option<SharedString>)> = match look.shape {
            Shape::Pattern => (0..12)
                .map(|step| {
                    (
                        TAU * step as f32 / 12.,
                        Some(format!("{}°", step * 30).into()),
                    )
                })
                .collect(),
            Shape::Rose => (0..count)
                .map(|axis| (TAU * axis as f32 / count as f32, None))
                .collect(),
            _ => current.angles.iter().map(|angle| (*angle, None)).collect(),
        };
        for (angle, label) in spokes {
            stroke(
                &[center, radial(center, radius, angle)],
                false,
                1.,
                look.rule,
                painter,
            );
            if let Some(label) = label {
                let (at, align) = anchor(radial(center, radius + 10., angle), center);
                labels.push((label, at, align));
            }
        }
        if look.shape != Shape::Pattern {
            for (name, angle) in current.axes.iter().zip(&current.angles) {
                let (at, align) = anchor(radial(center, radius + 10., *angle), center);
                labels.push((name.clone(), at, align));
            }
        }
        let rows = (0..current.names.len()).filter(shown);
        match look.shape {
            Shape::Rose => {
                let mut base = vec![0f64; count];
                let half = TAU / count.max(1) as f32 / 2.;
                for row in rows {
                    let color = colors.series(current.members[row]);
                    for (axis, angle) in current.angles.iter().enumerate() {
                        let value = current.value(row, axis);
                        if !(value.is_finite() && value > 0.) {
                            continue;
                        }
                        let scale = |total: f64| radius * look.fraction(total).sqrt();
                        let (inner, outer) = (scale(base[axis]), scale(base[axis] + value));
                        base[axis] += value;
                        let alpha = if focus.is_none_or(|focus| focus == axis) {
                            look.fill
                        } else {
                            look.fill * 0.5
                        };
                        shade(
                            &wedge(
                                center,
                                (inner, outer),
                                (angle - half + 0.01, angle + half - 0.01),
                            ),
                            color.opacity(alpha),
                            painter,
                        );
                    }
                }
            }
            _ => {
                for row in rows {
                    let color = colors.series(current.members[row]);
                    let corners: Vec<Point> = current
                        .angles
                        .iter()
                        .enumerate()
                        .map(|(axis, angle)| {
                            radial(
                                center,
                                radius * look.fraction(current.value(row, axis)),
                                *angle,
                            )
                        })
                        .collect();
                    let closed = look.shape != Shape::Pattern
                        || current.angles.iter().copied().fold(0f32, f32::max)
                            - current.angles.iter().copied().fold(TAU, f32::min)
                            > TAU * 0.97;
                    if look.fill > 0. && closed {
                        shade(&corners, color.opacity(look.fill), painter);
                    }
                    stroke(&corners, closed, look.line, color, painter);
                    for (axis, corner) in corners.iter().enumerate() {
                        let focused = focus == Some(axis);
                        if look.shape != Shape::Pattern || focused {
                            dot(*corner, if focused { 8. } else { 5. }, color, painter);
                        }
                    }
                }
            }
        }
    }));
    for (label, at, align) in &labels {
        text::draw(painter, label, *at, *align, TEXT_SIZE, look.text);
    }
}

/// The center and outer radius of a polar chart painted in `bounds`.
fn polar_frame(bounds: Bounds) -> (Point, f32) {
    (bounds.center(), (bounds.size.width.min(bounds.size.height) / 2. - MARGIN).max(10.))
}

/// The axis nearest the pointer's angle, near the web.
fn polar_hover(bounds: Bounds, web: Signal<Web>, at: Point) -> Option<Hover> {
    let (center, radius) = polar_frame(bounds);
    let (dx, dy) = (at.x - center.x, at.y - center.y);
    (dx.hypot(dy) <= radius + MARGIN / 2.)
        .then(|| web.with(|web| gap(dx.atan2(-dy).rem_euclid(TAU), &web.angles)))
        .flatten()
        .map(|axis| Hover { axis, row: None, at: at - bounds.origin, area: bounds.size })
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn paint_parallel(
    bounds: Bounds,
    web: Signal<Web>,
    hidden: Signal<BTreeSet<usize>>,
    (hover, camera): (Signal<Option<Hover>>, Signal<Camera>),
    look: Look,
    painter: &mut Painter,
) {
    let plot = parallel_plot(bounds);
    let colors = ChartTheme::colors();
    let focus = hover.get().and_then(|hover| hover.row);
    let view = camera.get().view;
    let mut labels: Vec<Label> = Vec::new();
    let column = |axis: usize, count: usize| column_x(plot, view, axis, count);
    web.with(|current| hidden.with(|hidden| {
        let count = current.axes.len();
        let (upper, lower) = (1. - view.y0, 1. - view.y0 - view.sy);
        for (axis, name) in current.axes.iter().enumerate() {
            let x = column(axis, count);
            if x < plot.left() - 0.5 || x > plot.right() + 0.5 {
                continue;
            }
            stroke(
                &[point(x, plot.top()), point(x, plot.bottom())],
                false,
                1.,
                look.rule,
                painter,
            );
            let (low, high) = current.extents.get(axis).copied().unwrap_or((0., 1.));
            let span = high - low;
            labels.push((
                crate::format::trimmed(f64::from(upper).mul_add(span, low)),
                point(x, plot.top() - (TEXT_SIZE + 4.)),
                TextAlign::Center,
            ));
            labels.push((
                crate::format::trimmed(f64::from(lower).mul_add(span, low)),
                point(x, plot.bottom() + 4.),
                TextAlign::Center,
            ));
            labels.push((
                name.clone(),
                point(x, plot.bottom() + (TEXT_SIZE + 10.)),
                TextAlign::Center,
            ));
        }
        let draw = |row: usize, alpha: f32, width: f32, painter: &mut Painter| {
            let color = colors.series(current.members[row]).opacity(alpha);
            for run in geometry(plot, view, current, row).split(Option::is_none) {
                let run: Vec<Point> = run.iter().flatten().copied().collect();
                stroke(&run, false, width, color, painter);
            }
        };
        let mask = if view.is_home() { bounds } else { plot };
        painter.with_clip(mask.into(), |painter| {
            for row in (0..current.names.len())
                .filter(|row| !hidden.contains(&current.members[*row]) && Some(*row) != focus)
            {
                draw(
                    row,
                    if focus.is_some() {
                        look.fill * 0.4
                    } else {
                        look.fill
                    },
                    look.line,
                    painter,
                );
            }
            if let Some(row) = focus.filter(|row| !hidden.contains(&current.members[*row])) {
                draw(row, 1., look.line + 1.5, painter);
            }
        });
    }));
    for (label, at, align) in &labels {
        text::draw(painter, label, *at, *align, TEXT_SIZE, look.text);
    }
    camera.get().paint_selection(plot, crate::ui::theme().foreground, painter);
}

/// The plot of a parallel-coordinates chart painted in `bounds`.
fn parallel_plot(bounds: Bounds) -> Bounds {
    let (left, right, top, bottom) = PAD;
    Bounds::from_corners(bounds.origin + point(left, top), bounds.bottom_right() - point(right, bottom))
}

fn column_x(plot: Bounds, view: scene::View, axis: usize, count: usize) -> f32 {
    let x = if count > 1 { axis as f32 / (count - 1) as f32 } else { 0.5 };
    plot.left() + plot.size.width * ((x - view.x0) / view.sx)
}

/// Where row `row` crosses each axis (`None` where it has no value).
fn geometry(plot: Bounds, view: scene::View, web: &Web, row: usize) -> Vec<Option<Point>> {
    let count = web.axes.len();
    (0..count)
        .map(|axis| {
            let value = web.value(row, axis);
            let (low, high) = web.extents.get(axis).copied().unwrap_or((0., 1.));
            value.is_finite().then(|| {
                let share = if high > low { ((value - low) / (high - low)) as f32 } else { 0.5 };
                point(column_x(plot, view, axis, count), plot.top() + plot.size.height * ((1. - share - view.y0) / view.sy))
            })
        })
        .collect()
}

/// The line nearest the pointer, between the axes around it.
fn parallel_hover(
    bounds: Bounds,
    web: Signal<Web>,
    hidden: Signal<BTreeSet<usize>>,
    camera: Signal<Camera>,
    at: Point,
) -> Option<Hover> {
    let camera = camera.get();
    if camera.dragging() {
        return None;
    }
    let (plot, view) = (parallel_plot(bounds), camera.view);
    web.with(|current| hidden.with(|hidden| {
                let count = current.axes.len();
                if count < 2 || plot.size.width <= 0. {
                    return None;
                }
                let content = view.x0 + ((at.x - plot.left()) / plot.size.width).clamp(0., 1.) * view.sx;
                let t = content * (count - 1) as f32;
                let axis = (t.floor() as usize).min(count - 2);
                let blend = t - axis as f32;
                (0..current.names.len())
                    .filter(|row| !hidden.contains(&current.members[*row]))
                    .filter_map(|row| {
                        let points = geometry(plot, view, current, row);
                        let (a, b) = (points[axis]?, points[axis + 1]?);
                        let y = (b.y - a.y) * blend + a.y;
                        let distance = (y - at.y).abs();
                        (distance <= REACH).then_some((row, distance))
                    })
                    .min_by(|a, b| a.1.total_cmp(&b.1))
                    .map(|(row, _)| Hover {
                        axis: t.round() as usize,
                        row: Some(row),
                        at: at - bounds.origin,
                        area: bounds.size,
                    })
    }))
}

fn tip(
    web: Signal<Web>,
    hidden: Signal<BTreeSet<usize>>,
    hovered: Hover,
    look: Look,
) -> Option<Tip> {
    web.with(|current| hidden.with(|hidden| tip_of(current, hidden, hovered, look)))
}

fn tip_of(current: &Web, hidden: &BTreeSet<usize>, hovered: Hover, look: Look) -> Option<Tip> {
    let colors = ChartTheme::colors();
    let (title, accent, rows) = if let Some(row) = hovered.row {
        (
            current.names.get(row)?.clone(),
            Some(colors.series(*current.members.get(row)?)),
            current
                .axes
                .iter()
                .enumerate()
                .map(|(axis, name)| (None, name.clone(), look.format(current.value(row, axis))))
                .collect(),
        )
    } else {
        (
            current.axes.get(hovered.axis)?.clone(),
            None,
            (0..current.names.len())
                .filter(|row| !hidden.contains(&current.members[*row]))
                .map(|row| {
                    (
                        Some(colors.series(current.members[row])),
                        current.names[row].clone(),
                        look.format(current.value(row, hovered.axis)),
                    )
                })
                .collect(),
        )
    };
    Some(Tip {
        at: hovered.at,
        area: hovered.area,
        title,
        accent,
        rows,
    })
}

#[allow(clippy::too_many_arguments, clippy::needless_pass_by_value)]
fn shell(
    web: Signal<Web>,
    hidden: Signal<BTreeSet<usize>>,
    hover: Signal<Option<Hover>>,
    look: Look,
    legend: Legend,
    tooltip: Tooltip,
    class: Option<SharedString>,
) -> AnyElement {
    let camera = use_signal(Camera::default());
    let painted = use_signal(Bounds::default());
    let items: Option<Vec<LegendItem>> = legend.visible().then(|| {
        let colors = ChartTheme::colors();
        web.with(|web| hidden.with(|hidden| {
            web.groups
                .iter()
                .enumerate()
                .map(|(index, name)| LegendItem {
                    index,
                    name: name.clone(),
                    color: colors.series(index),
                    disabled: hidden.contains(&index),
                    values: Default::default(),
                })
                .collect()
        }))
    });
    let count = web.with(|web| web.groups.len());
    let popup = hover
        .get()
        .filter(|_| tooltip.mode != TooltipMode::Hidden)
        .and_then(|hovered| tip(web, hidden, hovered, look))
        .map(card);
    let panel = items.map(|items| legend_panel(legend, items, hidden, count));
    let parallel = look.shape == Shape::Parallel;
    let surface = canvas(move |bounds, painter| {
        let bounds = Bounds::from(bounds);
        painted.update_untracked(|painted| *painted = bounds);
        if parallel {
            Camera::place(camera, bounds);
            paint_parallel(bounds, web, hidden, (hover, camera), look, painter);
        } else {
            paint_polar(bounds, web, hidden, hover, look, painter);
        }
    })
    .class("absolute inset-0 size-full")
    .on_mouse_move(move |event: &MouseMoveEvent, _, _| {
        let (bounds, at) = (painted.get(), Point::from(event.position));
        let next = if parallel {
            parallel_hover(bounds, web, hidden, camera, at)
        } else {
            polar_hover(bounds, web, at)
        };
        if hover.get() != next {
            hover.set(next);
        }
    })
    .on_hover(move |inside, _, _| {
        if !inside && hover.get().is_some() {
            hover.set(None);
        }
    });
    let surface = if parallel { camera_input(surface, camera, Zoom::WheelAndDrag, false) } else { surface };
    div()
        .class("flex size-full min-w-0 min-h-0")
        .map(|el| if legend.on_right() { el.flex_row() } else { el.flex_col() })
        .when_some(class, Div::class)
        .child(div().class("relative flex-1 min-w-0 min-h-0").child(surface).child(popup))
        .child(panel)
        .into_any_element()
}

#[component]
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub fn spider<
    T: 'static,
    D: Dataset<T>,
    N: Fn(&T) -> &str + 'static,
    V: Fn(&T, usize) -> f64 + 'static,
>(
    data: D,
    axes: Vec<SharedString>,
    name: N,
    value: V,
    max: Option<f64>,
    levels: Option<usize>,
    grid: Option<SpiderGrid>,
    fill_opacity: Option<f32>,
    line_width: Option<f32>,
    legend: Option<Legend>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
) -> AnyElement {
    let web = use_signal(Web::default());
    let hidden = use_signal(BTreeSet::<usize>::new());
    let hover = use_signal(None::<Hover>);
    let key = data_key(&data, &axes);
    refresh(
        web,
        key,
        || data.rows(|rows| rows_by(rows, axes, &name, None, &value)).uniform(0.));
    let look = Look::new(
        Shape::Spider(grid.unwrap_or_default()),
        levels.unwrap_or(5),
        (fill_opacity.unwrap_or(0.2), line_width.unwrap_or(2.)),
        (0., max.unwrap_or_else(|| peak(web, false))));
    shell(
        web,
        hidden,
        hover,
        look,
        legend.unwrap_or_default(),
        tooltip.unwrap_or_default(),
        class)
}

#[component]
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub fn nightingale_rose<
    T: 'static,
    D: Dataset<T>,
    N: Fn(&T) -> &str + 'static,
    V: Fn(&T, usize) -> f64 + 'static,
>(
    data: D,
    sectors: Vec<SharedString>,
    name: N,
    value: V,
    max: Option<f64>,
    levels: Option<usize>,
    fill_opacity: Option<f32>,
    legend: Option<Legend>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
) -> AnyElement {
    let web = use_signal(Web::default());
    let hidden = use_signal(BTreeSet::<usize>::new());
    let hover = use_signal(None::<Hover>);
    let key = data_key(&data, &sectors);
    refresh(
        web,
        key,
        || data.rows(|rows| rows_by(rows, sectors, &name, None, &value)).uniform(0.5));
    let look = Look::new(
        Shape::Rose,
        levels.unwrap_or(4),
        (fill_opacity.unwrap_or(0.85), 0.),
        (0., max.unwrap_or_else(|| peak(web, true))));
    shell(
        web,
        hidden,
        hover,
        look,
        legend.unwrap_or_default(),
        tooltip.unwrap_or_default(),
        class)
}

#[component]
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub fn radiation_pattern<T: 'static, D: Dataset<T>, A: Fn(&T) -> f64 + 'static>(
    data: D,
    angle: A,
    min: Option<f64>,
    max: Option<f64>,
    levels: Option<usize>,
    fill_opacity: Option<f32>,
    line_width: Option<f32>,
    legend: Option<Legend>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
    children: Children<TrendField<T>>,
) -> AnyElement {
    let web = use_signal(Web::default());
    let hidden = use_signal(BTreeSet::<usize>::new());
    let hover = use_signal(None::<Hover>);
    let fields = children;
    let key = data_key(&data, fields.len());
    refresh(
        web,
        key,
        || data.rows(|rows| {
            let degrees: Vec<f64> = rows.iter().map(&angle).collect();
            let names: Vec<SharedString> = fields
                .iter()
                .enumerate()
                .map(|(index, field)| {
                    field
                        .name
                        .clone()
                        .unwrap_or_else(|| format!("Series {}", index + 1).into())
                })
                .collect();
            let mut values = Vec::with_capacity(rows.len() * fields.len());
            let mut column = Vec::new();
            for field in &fields {
                field.y.fill(rows, &mut column);
                values.extend_from_slice(&column);
            }
            Web {
                axes: degrees
                    .iter()
                    .map(|value| format!("{}°", crate::format::trimmed(*value)).into())
                    .collect(),
                angles: degrees
                    .iter()
                    .map(|value| value.to_radians().rem_euclid(std::f64::consts::TAU) as f32)
                    .collect(),
                groups: names.clone(),
                members: (0..names.len()).collect(),
                names,
                values,
                ..Web::default()
            }
        }));
    let top = max.unwrap_or_else(|| {
        let high = web.with(|web| {
            web.values.iter().copied().filter(|value| value.is_finite()).fold(f64::NEG_INFINITY, f64::max)
        });
        if high.is_finite() {
            (high / 10.).ceil() * 10.
        } else {
            0.
        }
    });
    let look = Look::new(
        Shape::Pattern,
        levels.unwrap_or(4),
        (fill_opacity.unwrap_or(0.1), line_width.unwrap_or(2.)),
        (min.unwrap_or(top - 40.), top));
    shell(
        web,
        hidden,
        hover,
        look,
        legend.unwrap_or_default(),
        tooltip.unwrap_or_default(),
        class)
}

#[component]
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub fn parallel_coordinates<
    T: 'static,
    D: Dataset<T>,
    N: Fn(&T) -> &str + 'static,
    V: Fn(&T, usize) -> f64 + 'static,
    G: Fn(&T) -> &str + 'static,
>(
    data: D,
    axes: Vec<SharedString>,
    name: N,
    value: V,
    group: Option<G>,
    line_opacity: Option<f32>,
    line_width: Option<f32>,
    legend: Option<Legend>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
) -> AnyElement {
    let web = use_signal(Web::default());
    let hidden = use_signal(BTreeSet::<usize>::new());
    let hover = use_signal(None::<Hover>);
    let key = data_key(&data, (&axes, group.is_some()));
    refresh(
        web,
        key,
        || {
            data.rows(|rows| {
                rows_by(rows, axes, &name, group.as_ref().map(|group| group as &dyn Fn(&T) -> &str), &value)
            })
            .measured()
        });
    let look = Look::new(
        Shape::Parallel,
        1,
        (line_opacity.unwrap_or(0.6), line_width.unwrap_or(1.5)),
        (0., 1.));
    let legend = legend.unwrap_or_else(|| {
        if group.is_some() {
            Legend::default()
        } else {
            Legend::hidden()
        }
    });
    shell(
        web,
        hidden,
        hover,
        look,
        legend,
        tooltip.unwrap_or_default(),
        class)
}
