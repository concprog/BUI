//! Charts that lay out their own shapes: a layout function turns the chart's model into a [`Scene`]
//! (polygons, polylines, labels, tooltip notes) for the canvas' size, and [`scene_view`] paints it
//! with hover, legend toggling, click focus and an optional zoomable camera. As in zopra-charts but
//! without `window` / `cx`:
//!
//! - `let stage = stage::<M>();` (a hook: the chart's state) then
//!   `stage.hold(key: Option<u64>, || model)` (built again only when `key` changes) and
//!   `scene_view(stage, layout: fn(&M, &Frame) -> Scene, Show { groups, legend, tooltip, clickable,
//!   zoom, class: Option<SharedString> }) -> AnyElement`.
//! - `Frame { width, height, ink: Ink { colors, text, muted, rule, background }, hidden, focus }`.
//! - `Scene::{shapes, labels, notes}`, `scene.label(text, at, TextAlign, size, color)`,
//!   `scene.note(Note) -> usize`, `scene.lines(points, area, color, width, tip)`,
//!   `scene.scale_bar(|t| color, (low, high), area, &ink)` (the sampler replaces zopra's
//!   `HeatmapScheme` argument: `|t| scheme.sample(t)`).
//! - `Shape::{filled, line}(points, color, [width,] tip).loose(slop).outlined(color, width)`.
//! - Geometry in local `f32` pixels (`At` = `crate::geom::Point`): `polar`, `wedge`, `ellipse`, `rect`,
//!   `hexagon`, `curve`, `inside`, `text_width`, `Area`, `clip`.
//! - A camera for any canvas (`profile` uses it too): `Camera` in a signal, `Camera::place(camera,
//!   bounds)` from the paint, `camera.paint_selection(bounds, color, painter)`, and
//!   `camera_input(element, camera, Zoom, uniform) -> Div` adds its wheel and drag listeners.

use std::collections::BTreeSet;
use std::f32::consts::TAU;
use std::hash::{DefaultHasher, Hash, Hasher};

use blenderui::element::{AnyElement, Div, FluentBuilder, IntoElement, ParentElement};
use blenderui::hooks::{Signal, use_signal};
use blenderui::html::div;
use blenderui::style::TextAlign;
use blenderui::{App, Hsla, MouseButton, MouseDownEvent, MouseMoveEvent, Painter, ScrollWheelEvent, SharedString, Styled, canvas};

use super::{Tip, card};
use crate::geom::{Bounds, Point, Size, point};
use crate::legend::{LegendItem, legend_panel};
use crate::path::Path;
use crate::plot::{Tooltip, TooltipMode, fraction, zoom_held};
use crate::{ChartColors, ChartTheme, Legend, text};

pub(crate) type At = Point;

pub(crate) struct Shape {
    pub points: Vec<At>,
    pub fill: Option<Hsla>,
    pub stroke: Option<(Hsla, f32)>,
    pub tip: Option<usize>,
    slop: f32,
}

impl Shape {
    pub(crate) const fn filled(points: Vec<At>, color: Hsla, tip: Option<usize>) -> Self {
        Self { points, fill: Some(color), stroke: None, tip, slop: 0. }
    }

    pub(crate) const fn line(points: Vec<At>, color: Hsla, width: f32, tip: Option<usize>) -> Self {
        Self { points, fill: None, stroke: Some((color, width)), tip, slop: 0. }
    }

    pub(crate) const fn loose(mut self, slop: f32) -> Self {
        self.slop = slop;
        self
    }

    pub(crate) const fn outlined(mut self, color: Hsla, width: f32) -> Self {
        self.stroke = Some((color, width));
        self
    }

    fn contains(&self, at: At) -> bool {
        let near = |reach: f32| self.points.windows(2).any(|pair| distance(pair[0], pair[1], at) <= reach);
        if self.fill.is_some() {
            return inside(&self.points, at)
                || (self.slop > 0.
                    && (near(self.slop)
                        || self
                            .points
                            .first()
                            .zip(self.points.last())
                            .is_some_and(|(first, last)| distance(*last, *first, at) <= self.slop)));
        }
        near(self.stroke.map_or(0., |(_, width)| width / 2.) + 3.)
    }
}

pub(crate) struct Label {
    pub text: SharedString,
    pub at: At,
    pub align: TextAlign,
    pub size: f32,
    pub color: Hsla,
}

pub(crate) struct Note {
    pub title: SharedString,
    pub accent: Option<Hsla>,
    pub rows: Vec<(Option<Hsla>, SharedString, SharedString)>,
}

#[derive(Default)]
pub(crate) struct Scene {
    key: Option<u64>,
    pub shapes: Vec<Shape>,
    pub labels: Vec<Label>,
    pub notes: Vec<Note>,
}

impl Scene {
    pub(crate) fn label(&mut self, text: impl Into<SharedString>, at: At, align: TextAlign, size: f32, color: Hsla) {
        self.labels.push(Label { text: text.into(), at, align, size, color });
    }

    pub(crate) fn note(&mut self, note: Note) -> usize {
        self.notes.push(note);
        self.notes.len() - 1
    }
}

pub(crate) struct Ink<'a> {
    pub colors: &'a ChartColors,
    pub text: Hsla,
    pub muted: Hsla,
    pub rule: Hsla,
    pub background: Hsla,
}

pub(crate) struct Frame<'a> {
    pub width: f32,
    pub height: f32,
    pub ink: Ink<'a>,
    pub hidden: &'a BTreeSet<usize>,
    pub focus: Option<usize>,
}

pub(crate) type Arrange<M> = fn(&M, &Frame) -> Scene;

#[derive(Default)]
pub(crate) struct Held<M> {
    pub key: Option<u64>,
    pub model: M,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Hover {
    tip: usize,
    at: Point,
    area: Size,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) enum Zoom {
    #[default]
    Off,
    Wheel,
    WheelAndDrag,
}

const MAX_ZOOM: f32 = 12.;
const MIN_DRAG: f32 = 4.;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct View {
    pub x0: f32,
    pub y0: f32,
    pub sx: f32,
    pub sy: f32,
}

#[derive(Clone, Copy)]
pub(crate) struct Pose {
    kx: f32,
    ky: f32,
    ox: f32,
    oy: f32,
}

impl Pose {
    pub(crate) const fn map(self, at: At) -> At {
        point(at.x.mul_add(self.kx, self.ox), at.y.mul_add(self.ky, self.oy))
    }

    fn unmap(self, at: At) -> At {
        point((at.x - self.ox) / self.kx, (at.y - self.oy) / self.ky)
    }
}

impl View {
    pub(crate) const HOME: Self = Self { x0: 0., y0: 0., sx: 1., sy: 1. };

    pub(crate) fn is_home(self) -> bool {
        self == Self::HOME
    }

    pub(crate) fn pose(self, width: f32, height: f32) -> Pose {
        Pose { kx: 1. / self.sx, ky: 1. / self.sy, ox: -self.x0 * width / self.sx, oy: -self.y0 * height / self.sy }
    }

    fn fitted(self) -> Self {
        let least = 1. / MAX_ZOOM;
        let (sx, sy) = (self.sx.clamp(least, 1.), self.sy.clamp(least, 1.));
        Self { x0: self.x0.clamp(0., 1. - sx), y0: self.y0.clamp(0., 1. - sy), sx, sy }
    }

    fn zoomed(self, (fx, fy): (f32, f32), factor: f32) -> Self {
        let least = 1. / MAX_ZOOM;
        let sx = (self.sx * factor).clamp(least, 1.);
        let sy = (self.sy * factor).clamp(least, 1.);
        Self { x0: fx.mul_add(self.sx - sx, self.x0), y0: fy.mul_add(self.sy - sy, self.y0), sx, sy }.fitted()
    }

    fn panned(self, dx: f32, dy: f32) -> Self {
        Self { x0: dx.mul_add(-self.sx, self.x0), y0: dy.mul_add(-self.sy, self.y0), ..self }.fitted()
    }

    fn framed(self, (left, top, right, bottom): (f32, f32, f32, f32), uniform: bool) -> Self {
        let (mut sx, mut sy) = ((right - left) * self.sx, (bottom - top) * self.sy);
        if uniform {
            (sx, sy) = (sx.max(sy), sx.max(sy));
        }
        let (sx, sy) = (sx.clamp(1. / MAX_ZOOM, 1.), sy.clamp(1. / MAX_ZOOM, 1.));
        Self {
            x0: f32::midpoint(left, right).mul_add(self.sx, self.x0) - sx / 2.,
            y0: f32::midpoint(top, bottom).mul_add(self.sy, self.y0) - sy / 2.,
            sx,
            sy,
        }
        .fitted()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Drag {
    from: Point,
    to: Point,
    pan: bool,
}

/// A zoomable view of a canvas: the view, a drag in progress, and the bounds the canvas last
/// painted in (where the listeners measure the pointer).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Camera {
    pub view: View,
    drag: Option<Drag>,
    bounds: Bounds,
}

impl Default for Camera {
    fn default() -> Self {
        Self { view: View::HOME, drag: None, bounds: Bounds::default() }
    }
}

impl Camera {
    pub(crate) const fn dragging(self) -> bool {
        self.drag.is_some()
    }

    /// The canvas painted in `bounds` (called from its paint).
    pub(crate) fn place(camera: Signal<Self>, bounds: Bounds) {
        camera.update_untracked(|camera| camera.bounds = bounds);
    }

    pub(crate) fn paint_selection(self, bounds: Bounds, color: Hsla, painter: &mut Painter) {
        if let Some(drag) = self.drag.filter(|drag| !drag.pan) {
            painter.fill(Bounds::from_corners(drag.from, drag.to).intersect(&bounds).into(), color.opacity(0.2));
        }
    }

    fn fractions(self, at: Point) -> (f32, f32) {
        let bounds = self.bounds;
        (fraction(at.x, bounds.left(), bounds.size.width), fraction(at.y, bounds.top(), bounds.size.height))
    }

    fn wheel(camera: Signal<Self>, event: &ScrollWheelEvent, cx: &mut App) {
        let at = Point::from(event.position);
        let current = camera.get();
        if !zoom_held(event.modifiers) || !current.bounds.contains(at) {
            return;
        }
        let rise = event.pixels.unwrap_or(event.delta * 16.);
        let factor = (-rise * 0.004).clamp(-0.5, 0.5).exp();
        if (factor - 1.).abs() > f32::EPSILON {
            camera.update(|camera| camera.view = camera.view.zoomed(current.fractions(at), factor));
        }
        cx.stop_propagation();
        cx.prevent_default();
    }

    fn down(camera: Signal<Self>, zoom: Zoom, event: &MouseDownEvent) {
        let at = Point::from(event.position);
        let current = camera.get();
        if !current.bounds.contains(at) {
            return;
        }
        if event.click_count >= 2 {
            camera.set(Self { bounds: current.bounds, ..Self::default() });
            return;
        }
        let pan = zoom == Zoom::Wheel || event.modifiers.shift;
        if pan && current.view.is_home() {
            return;
        }
        camera.update(|camera| camera.drag = Some(Drag { from: at, to: at, pan }));
    }

    fn moved(camera: Signal<Self>, event: &MouseMoveEvent) {
        let Some(drag) = camera.get().drag else { return };
        let at = Point::from(event.position);
        camera.update(|camera| {
            let (width, height) = (camera.bounds.size.width.max(1.), camera.bounds.size.height.max(1.));
            if event.pressed_button != Some(MouseButton::Left) {
                camera.drag = None;
            } else if drag.pan {
                let step = at - drag.to;
                camera.view = camera.view.panned(step.x / width, step.y / height);
                camera.drag = Some(Drag { from: at, to: at, pan: true });
            } else {
                camera.drag = Some(Drag { to: at, ..drag });
            }
        });
    }

    fn up(camera: Signal<Self>, uniform: bool) {
        let Some(drag) = camera.get().drag else { return };
        camera.update(|camera| {
            camera.drag = None;
            if drag.pan {
                return;
            }
            let (width, height) = (camera.bounds.size.width.max(1.), camera.bounds.size.height.max(1.));
            let (a, b) = (camera.fractions(drag.from), camera.fractions(drag.to));
            let (left, right) = (a.0.min(b.0), a.0.max(b.0));
            let (top, bottom) = (a.1.min(b.1), a.1.max(b.1));
            let (wide, tall) = ((right - left) * width, (bottom - top) * height);
            let usable = if uniform { wide.max(tall) >= MIN_DRAG } else { wide >= MIN_DRAG && tall >= MIN_DRAG };
            if usable {
                camera.view = camera.view.framed((left, top, right, bottom), uniform);
            }
        });
    }
}

/// The camera's listeners on `element` (the canvas it views): the zoom modifier with the wheel
/// zooms around the pointer, a drag frames an area (or pans: `Zoom::Wheel`, or with Shift), a
/// double click goes home.
pub(crate) fn camera_input(element: Div, camera: Signal<Camera>, zoom: Zoom, uniform: bool) -> Div {
    if zoom == Zoom::Off {
        return element;
    }
    element
        .on_scroll_wheel(move |event, _, cx| Camera::wheel(camera, event, cx))
        .on_mouse_down(MouseButton::Left, move |event, _, _| Camera::down(camera, zoom, event))
        .on_mouse_move(move |event, _, _| Camera::moved(camera, event))
        .on_mouse_up(MouseButton::Left, move |_, _, _| Camera::up(camera, uniform))
}

pub(crate) struct Stage<M: 'static> {
    pub held: Signal<Held<M>>,
    scene: Signal<Scene>,
    hidden: Signal<BTreeSet<usize>>,
    hover: Signal<Option<Hover>>,
    camera: Signal<Camera>,
    pub focus: Signal<Option<usize>>,
}

impl<M: 'static> Clone for Stage<M> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<M: 'static> Copy for Stage<M> {}

/// The state of the calling scene chart (a hook).
#[track_caller]
pub(crate) fn stage<M: Default + 'static>() -> Stage<M> {
    Stage {
        held: use_signal(Held::default()),
        scene: use_signal(Scene::default()),
        hidden: use_signal(BTreeSet::new()),
        hover: use_signal(None),
        camera: use_signal(Camera::default()),
        focus: use_signal(None),
    }
}

impl<M: 'static> Stage<M> {
    pub(crate) fn hold(self, key: Option<u64>, build: impl FnOnce() -> M) {
        if key.is_some() && self.held.with(|held| held.key) == key {
            return;
        }
        let model = build();
        self.held.update_untracked(|held| *held = Held { key, model });
        self.scene.update_untracked(|scene| scene.key = None);
        self.focus.update_untracked(|focus| *focus = None);
        self.camera.update_untracked(|camera| *camera = Camera { bounds: camera.bounds, ..Camera::default() });
    }
}

pub(crate) struct Show {
    pub groups: Vec<SharedString>,
    pub legend: Option<Legend>,
    pub tooltip: Option<Tooltip>,
    pub clickable: bool,
    pub zoom: Zoom,
    pub class: Option<SharedString>,
}

/// Lays the model out for `size` unless the scene already shows it.
fn arrange<M>(stage: Stage<M>, layout: Arrange<M>, size: (f32, f32)) {
    let focus = stage.focus.get();
    let held_key = stage.held.with(|held| held.key);
    let key = stage.hidden.with(|hidden| {
        let mut state = DefaultHasher::new();
        (held_key, size.0.to_bits(), size.1.to_bits(), hidden, focus, ChartTheme::revision()).hash(&mut state);
        Some(state.finish())
    });
    if held_key.is_some() && stage.scene.with(|scene| scene.key) == key {
        return;
    }
    let theme = crate::ui::theme();
    let colors = ChartTheme::colors();
    let mut scene = stage.hidden.with(|hidden| {
        let frame = Frame {
            width: size.0,
            height: size.1,
            ink: Ink {
                colors: &colors,
                text: theme.foreground,
                muted: theme.muted_foreground,
                rule: theme.chart_grid,
                background: theme.background,
            },
            hidden,
            focus,
        };
        stage.held.with(|held| layout(&held.model, &frame))
    });
    scene.key = key;
    stage.scene.update_untracked(|current| *current = scene);
}

fn mapped(points: &[At], origin: Point, pose: Pose) -> Vec<At> {
    points.iter().map(|at| origin + pose.map(*at)).collect()
}

fn paint_scene<M: 'static>(bounds: Bounds, stage: Stage<M>, layout: Arrange<M>, painter: &mut Painter) {
    let (width, height) = (bounds.size.width, bounds.size.height);
    Camera::place(stage.camera, bounds);
    arrange(stage, layout, (width, height));
    let origin = bounds.origin;
    let focus = stage.hover.get().map(|hover| hover.tip);
    let camera = stage.camera.get();
    let pose = camera.view.pose(width, height);
    let draw = |painter: &mut Painter| {
        stage.scene.with(|scene| {
            for shape in scene.shapes.iter().filter(|shape| shape.points.len() >= 2) {
                let dim = focus.is_some() && shape.tip.is_some() && shape.tip != focus;
                let fade = |color: Hsla| if dim { color.opacity(color.a * 0.35) } else { color };
                let path = if shape.fill.is_some() {
                    Path::polygon(&mapped(&shape.points, origin, pose))
                } else {
                    Path::polyline(mapped(&shape.points, origin, pose))
                };
                if let Some(color) = shape.fill {
                    path.fill(painter, fade(color));
                }
                if let Some((color, width)) = shape.stroke {
                    path.stroke(painter, fade(color), width);
                }
            }
            for label in &scene.labels {
                let at = pose.map(label.at);
                if (-100. ..=width + 100.).contains(&at.x) && (-100. ..=height + 100.).contains(&at.y) {
                    text::draw(painter, &label.text, origin + at, label.align, label.size, label.color);
                }
            }
        });
    };
    if camera.view.is_home() {
        draw(painter);
    } else {
        painter.with_clip(bounds.into(), draw);
    }
    camera.paint_selection(bounds, crate::ui::theme().foreground, painter);
}

/// What the pointer is over: the topmost shape with a note.
fn hover_at<M: 'static>(stage: Stage<M>, event: &MouseMoveEvent) {
    let camera = stage.camera.get();
    let bounds = camera.bounds;
    let local = Point::from(event.position) - bounds.origin;
    let at = camera.view.pose(bounds.size.width, bounds.size.height).unmap(local);
    let next = (!camera.dragging())
        .then(|| {
            stage.scene.with(|scene| {
                scene.shapes.iter().rev().find(|shape| shape.tip.is_some() && shape.contains(at)).and_then(|shape| shape.tip)
            })
        })
        .flatten()
        .map(|tip| Hover { tip, at: local, area: bounds.size });
    if stage.hover.get() != next {
        stage.hover.set(next);
    }
}

pub(crate) fn scene_view<M: 'static>(stage: Stage<M>, layout: Arrange<M>, show: Show) -> AnyElement {
    let Show { groups, legend, tooltip, clickable, zoom, class } = show;
    let legend = legend.unwrap_or_else(|| if groups.is_empty() { Legend::hidden() } else { Legend::default() });
    let items: Option<Vec<LegendItem>> = (legend.visible() && !groups.is_empty()).then(|| {
        let colors = ChartTheme::colors();
        stage.hidden.with(|hidden| {
            groups
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
        })
    });
    let count = groups.len();
    let shown = tooltip.unwrap_or_default().mode != TooltipMode::Hidden;
    let popup = stage
        .hover
        .get()
        .filter(|_| shown)
        .and_then(|hover| {
            stage.scene.with(|scene| {
                let note = scene.notes.get(hover.tip)?;
                Some(Tip {
                    at: hover.at,
                    area: hover.area,
                    title: note.title.clone(),
                    accent: note.accent,
                    rows: note.rows.clone(),
                })
            })
        })
        .map(card);
    let panel = items.map(|items| legend_panel(legend, items, stage.hidden, count));
    let surface = canvas(move |bounds, painter| paint_scene(bounds.into(), stage, layout, painter))
        .class("absolute inset-0 size-full")
        .on_mouse_move(move |event, _, _| hover_at(stage, event))
        .on_hover(move |inside, _, _| {
            if !inside && stage.hover.get().is_some() {
                stage.hover.set(None);
            }
        })
        .when(clickable, |el| {
            el.on_mouse_down(MouseButton::Left, move |_, _, _| {
                let next = stage.hover.get().map(|hover| hover.tip);
                stage.focus.set(next);
            })
        });
    div()
        .class("flex size-full min-w-0 min-h-0")
        .map(|el| if legend.on_right() { el.flex_row() } else { el.flex_col() })
        .when_some(class, Div::class)
        .child(
            div()
                .class("relative flex-1 min-w-0 min-h-0")
                .child(camera_input(surface, stage.camera, zoom, true))
                .child(popup),
        )
        .child(panel)
        .into_any_element()
}

pub(crate) fn polar(center: At, radius: f32, angle: f32) -> At {
    point(
        radius.mul_add(angle.sin(), center.x),
        (-radius).mul_add(angle.cos(), center.y),
    )
}

pub(crate) fn wedge(center: At, (inner, outer): (f32, f32), (from, to): (f32, f32)) -> Vec<At> {
    let steps = ((to - from).abs() / TAU * 96.).ceil().max(2.) as usize;
    let arc = |radius: f32| {
        (0..=steps).map(move |step| {
            polar(
                center,
                radius,
                (to - from).mul_add(step as f32 / steps as f32, from),
            )
        })
    };
    let mut points: Vec<At> = arc(outer).collect();
    if inner > 0. {
        points.extend(arc(inner).collect::<Vec<_>>().into_iter().rev());
    } else {
        points.push(center);
    }
    points
}

pub(crate) fn ellipse(center: At, (rx, ry): (f32, f32), steps: usize) -> Vec<At> {
    (0..steps)
        .map(|step| {
            let angle = TAU * step as f32 / steps as f32;
            point(
                rx.mul_add(angle.cos(), center.x),
                ry.mul_add(angle.sin(), center.y),
            )
        })
        .collect()
}

pub(crate) fn rect(x: f32, y: f32, width: f32, height: f32) -> Vec<At> {
    vec![
        point(x, y),
        point(x + width, y),
        point(x + width, y + height),
        point(x, y + height),
    ]
}

pub(crate) fn hexagon(center: At, radius: f32) -> Vec<At> {
    (0..6)
        .map(|corner| polar(center, radius, TAU * corner as f32 / 6.))
        .collect()
}

pub(crate) fn curve(from: At, control: At, to: At, steps: usize) -> impl Iterator<Item = At> {
    (0..=steps).map(move |step| {
        let t = step as f32 / steps as f32;
        let u = 1. - t;
        point(
            (u * u).mul_add(from.x, (2. * u * t).mul_add(control.x, t * t * to.x)),
            (u * u).mul_add(from.y, (2. * u * t).mul_add(control.y, t * t * to.y)),
        )
    })
}

pub(crate) fn inside(points: &[At], at: At) -> bool {
    let mut odd = false;
    let mut previous = match points.last() {
        Some(last) => *last,
        None => return false,
    };
    for current in points {
        if (current.y > at.y) != (previous.y > at.y)
            && at.x
                < (previous.x - current.x) * (at.y - current.y) / (previous.y - current.y)
                    + current.x
        {
            odd = !odd;
        }
        previous = *current;
    }
    odd
}

fn distance(a: At, b: At, at: At) -> f32 {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let length = dx.mul_add(dx, dy * dy);
    let t = if length > 0. {
        ((at.x - a.x).mul_add(dx, (at.y - a.y) * dy) / length).clamp(0., 1.)
    } else {
        0.
    };
    (at.x - t.mul_add(dx, a.x)).hypot(at.y - t.mul_add(dy, a.y))
}

pub(crate) fn text_width(text: &str, size: f32) -> f32 {
    text.chars().count() as f32 * size * 0.56
}

#[derive(Clone, Copy)]
pub(crate) struct Area {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

impl Area {
    pub(crate) const fn inset(
        width: f32,
        height: f32,
        (left, top, right, bottom): (f32, f32, f32, f32),
    ) -> Self {
        Self {
            left,
            top,
            right: width - right,
            bottom: height - bottom,
        }
    }

    pub(crate) fn width(self) -> f32 {
        (self.right - self.left).max(1.)
    }

    pub(crate) fn height(self) -> f32 {
        (self.bottom - self.top).max(1.)
    }

    pub(crate) fn holds(self, at: At) -> bool {
        at.x >= self.left - 0.5
            && at.x <= self.right + 0.5
            && at.y >= self.top - 0.5
            && at.y <= self.bottom + 0.5
    }
}

pub(crate) fn clip(points: impl IntoIterator<Item = At>, area: Area) -> Vec<Vec<At>> {
    let mut runs: Vec<Vec<At>> = Vec::new();
    let mut current: Vec<At> = Vec::new();
    for at in points {
        if area.holds(at) && at.x.is_finite() && at.y.is_finite() {
            current.push(at);
        } else if !current.is_empty() {
            runs.push(std::mem::take(&mut current));
        }
    }
    if current.len() > 1 {
        runs.push(current);
    }
    runs.retain(|run| run.len() > 1);
    runs
}

impl Scene {
    pub(crate) fn lines(
        &mut self,
        points: impl IntoIterator<Item = At>,
        area: Area,
        color: Hsla,
        width: f32,
        tip: Option<usize>,
    ) {
        for run in clip(points, area) {
            self.shapes.push(Shape::line(run, color, width, tip));
        }
    }

    pub(crate) fn scale_bar(
        &mut self,
        sample: impl Fn(f64) -> Hsla,
        (low, high): (f64, f64),
        area: Area,
        ink: &Ink,
    ) {
        let steps = 48;
        let height = area.height() / steps as f32;
        for step in 0..steps {
            let t = 1. - (f64::from(step) + 0.5) / f64::from(steps);
            self.shapes.push(Shape::filled(
                rect(
                    area.left,
                    (step as f32).mul_add(height, area.top),
                    area.width(),
                    height + 0.5,
                ),
                sample(t),
                None,
            ));
        }
        self.label(
            crate::format::trimmed(high),
            point(area.right + 4., area.top - TEXT / 2.),
            TextAlign::Left,
            TEXT,
            ink.muted,
        );
        self.label(
            crate::format::trimmed(low),
            point(area.right + 4., area.bottom - TEXT / 2.),
            TextAlign::Left,
            TEXT,
            ink.muted,
        );
    }
}

const TEXT: f32 = crate::plot::TEXT_SIZE;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zoom_keeps_the_cursor_anchored_and_stays_in_bounds() {
        let view = View::HOME.zoomed((0.25, 0.5), 0.5);
        let pose = view.pose(400., 200.);
        let back = pose.unmap(point(100., 100.));
        assert!((back.x - 100.).abs() < 1e-3 && (back.y - 100.).abs() < 1e-3);
        let deep = (0..40).fold(View::HOME, |view, _| view.zoomed((1., 1.), 0.5));
        assert!((deep.sx - 1. / MAX_ZOOM).abs() < 1e-6);
        assert!(deep.x0 + deep.sx <= 1. + 1e-6);
        assert!(deep.zoomed((0.5, 0.5), 100.).is_home());
    }

    #[test]
    fn framing_a_band_stays_uniform_and_pan_is_clamped() {
        let view = View::HOME.framed((0.4, 0.5, 0.6, 0.5), true);
        assert!((view.sx - view.sy).abs() < 1e-6);
        assert!((view.sx - 0.2).abs() < 1e-6);
        let moved = view.panned(-10., 10.);
        assert!(moved.x0 >= 0. && moved.x0 + moved.sx <= 1. + 1e-6);
    }
}
