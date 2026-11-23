use blenderui::hooks::Signal;
use blenderui::style::TextAlign;
use blenderui::{
    App, Hsla, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Painter, ScrollWheelEvent, SharedString,
    Window,
};

use crate::geom::{Bounds, Point, Size, point, size};
use crate::path::Path;
use super::data::{ArcSection, GraphEdge, GraphNode, ZoomMode};
use super::layout::Position;
use super::model::Graph;
use crate::format::trimmed;
use crate::mappings::StateValue;
use crate::text;

pub(crate) const NODE_RADIUS: f32 = 40.;
const FONT: f32 = 10.;
const EDGE_FONT: f32 = 8.;
const LINE: f32 = 13.;
const MIN_FONT: f32 = 4.;
const STATS_WIDTH: f32 = 70.;
const TITLE_WIDTH: f32 = 140.;
const HOVER_WIDTH: f32 = 200.;
const EDGE_REACH: f32 = 6.;
const CLICK_SLOP: f32 = 3.;
const MIN_SCALE: f32 = 0.1;
const MAX_SCALE: f32 = 10.;
const ZOOM_SPEED: f32 = 0.0015;
const MARKER_LIMIT: usize = 100;
const DIM: f32 = 0.5;

pub(crate) type Format = fn(f64) -> SharedString;
pub(crate) use super::data::StatFormats as Formats;

#[derive(Clone, Copy)]
pub(crate) struct Ui {
    pub text: Hsla,
    pub muted: Hsla,
    pub background: Hsla,
    pub border: Hsla,
    pub accent: Hsla,
    pub popover: Hsla,
    pub popover_text: Hsla,
    pub edge: Hsla,
    pub highlight: Hsla,
    pub ring: Hsla,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Target {
    Node(usize),
    Marker(usize),
    Edge(usize),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct View {
    pub scale: f32,
    pub pan: Position,
    pub fit: bool,
}

impl Default for View {
    fn default() -> Self {
        Self {
            scale: 1.,
            pan: [0.; 2],
            fit: true,
        }
    }
}

const FIT_MARGIN: f32 = 16.;
const CONTROLS: f32 = 40.;

fn fitted(graph: &Graph, bounds: Bounds) -> View {
    let placed = graph.placed();
    let (mut left, mut right, mut top, mut bottom) = (
        f32::INFINITY,
        f32::NEG_INFINITY,
        f32::INFINITY,
        f32::NEG_INFINITY,
    );
    for (node, [x, y]) in placed.iter().enumerate() {
        if !graph.view.shows(node) && !graph.view.marked(node) {
            continue;
        }
        let r = graph.nodes.get(node).map_or(NODE_RADIUS, radius);
        left = left.min(x - r.max(TITLE_WIDTH / 2.));
        right = right.max(x + r.max(TITLE_WIDTH / 2.));
        top = top.min(y - r);
        bottom = bottom.max(y + r + 2.0f32.mul_add(LINE, 5.));
    }
    if left > right {
        return View::default();
    }
    let width = (bounds.size.width - FIT_MARGIN * 2.).max(1.);
    let height = (bounds.size.height - FIT_MARGIN * 2. - CONTROLS).max(1.);
    let scale = (width / (right - left))
        .min(height / (bottom - top))
        .clamp(MIN_SCALE, 1.);
    View {
        scale,
        pan: [
            -f32::midpoint(left, right),
            CONTROLS / 2. / scale - f32::midpoint(top, bottom),
        ],
        fit: true,
    }
}

pub(crate) type Picked = (Target, Point, Size);

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Press {
    from: Point,
    pan: Position,
    target: Option<Target>,
    moved: bool,
}

type NodeClick = dyn Fn(&GraphNode, &mut Window, &mut App);
type EdgeClick = dyn Fn(&GraphEdge, &mut Window, &mut App);

#[derive(Default)]
pub(crate) struct Clicks {
    pub node: Option<Box<NodeClick>>,
    pub edge: Option<Box<EdgeClick>>,
}

#[derive(Clone, Copy)]
pub(crate) struct State {
    /// where the canvas last painted (the handlers' frame)
    pub bounds: Signal<Bounds>,
    pub graph: Signal<Graph>,
    pub view: Signal<View>,
    pub hover: Signal<Option<Target>>,
    pub press: Signal<Option<Press>>,
    pub picked: Signal<Option<Picked>>,
    pub root: Signal<Option<usize>>,
    pub clicks: Signal<Clicks>,
}

pub(crate) fn stat(value: &StateValue, format: Option<Format>) -> SharedString {
    match (value, format) {
        (StateValue::Null, _) => SharedString::default(),
        (StateValue::Number(number), Some(format)) => format(*number),
        (StateValue::Number(number), None) => trimmed(*number),
        (other, _) => other.text(),
    }
}

pub(crate) fn radius(node: &GraphNode) -> f32 {
    node.radius.unwrap_or(NODE_RADIUS)
}

fn ring_width(radius: f32) -> f32 {
    (radius * 0.075).ceil()
}

struct Frame {
    bounds: Bounds,
    view: View,
}

impl Frame {
    fn at(&self, position: Position) -> Point {
        let center = self.bounds.center();
        let [x, y] = position;
        point(
            center.x + ((x + self.view.pan[0]) * self.view.scale),
            center.y + ((y + self.view.pan[1]) * self.view.scale),
        )
    }

    fn world(&self, at: Point) -> Position {
        let center = self.bounds.center();
        [
            (at.x - center.x) / self.view.scale - self.view.pan[0],
            (at.y - center.y) / self.view.scale - self.view.pan[1],
        ]
    }
}

struct Segment {
    start: Point,
    end: Point,
    tip: Point,
    head: f32,
}

fn segment(graph: &Graph, frame: &Frame, edge: usize) -> Option<Segment> {
    let (source, target) = graph.links.get(edge).copied().flatten()?;
    let view = &graph.view;
    let drawable = |node: usize| view.shows(node) || view.marked(node);
    if !drawable(source) || !drawable(target) || source == target {
        return None;
    }
    let placed = graph.placed();
    let (from, to) = (
        frame.at(*placed.get(source)?),
        frame.at(*placed.get(target)?),
    );
    let (dx, dy) = ((to.x - from.x), (to.y - from.y));
    let length = dx.hypot(dy);
    if length <= f32::EPSILON {
        return None;
    }
    let (ux, uy) = (dx / length, dy / length);
    let scale = frame.view.scale;
    let size = |node: usize| {
        if view.shows(node) {
            radius(&graph.nodes[node])
        } else {
            NODE_RADIUS
        }
    };
    let (rs, rt) = (size(source), size(target));
    let thickness = graph.edges[edge].thickness;
    let head = (2.0f32.mul_add(thickness, 10.)) * scale;
    let start_off = (rs + ring_width(rs) / 2.) * scale;
    let tip_off = (rt + ring_width(rt) / 2.) * scale;
    let shift = |by: f32| point(ux * by, uy * by);
    Some(Segment {
        start: from + shift(start_off),
        end: to - shift(tip_off + head),
        tip: to - shift(tip_off),
        head,
    })
}

fn distance(at: Point, a: Point, b: Point) -> f32 {
    let (px_, py) = (at.x, at.y);
    let (ax, ay, bx, by) = (a.x, a.y, b.x, b.y);
    let (dx, dy) = (bx - ax, by - ay);
    let length = dx.mul_add(dx, dy * dy);
    let t = if length > 0. {
        (((px_ - ax) * dx + (py - ay) * dy) / length).clamp(0., 1.)
    } else {
        0.
    };
    (ax + t * dx - px_).hypot(ay + t * dy - py)
}

fn hit(graph: &Graph, frame: &Frame, at: Point) -> Option<Target> {
    let placed = graph.placed();
    let scale = frame.view.scale;
    for (node, position) in placed.iter().enumerate().rev() {
        let center = frame.at(*position);
        let (dx, dy) = ((at.x - center.x), (at.y - center.y));
        if graph.view.shows(node) {
            let r = radius(&graph.nodes[node]) * scale;
            let pad = 5. * scale;
            if dx >= -r - pad && dx <= r + pad && dy >= -r - pad && dy <= r + 45. * scale {
                return Some(Target::Node(node));
            }
        } else if graph.view.marked(node) && dx.hypot(dy) <= NODE_RADIUS * scale {
            return Some(Target::Marker(node));
        }
    }
    (0..graph.edges.len()).find_map(|edge| {
        let line = segment(graph, frame, edge)?;
        (distance(at, line.start, line.tip) <= EDGE_REACH).then_some(Target::Edge(edge))
    })
}

fn clip(label: &SharedString, width: f32, size: f32) -> SharedString {
    if label.is_empty() || text::measure(label, size) <= width {
        return label.clone();
    }
    let mut chars: Vec<char> = label.chars().collect();
    while !chars.is_empty() {
        chars.pop();
        let candidate: SharedString = format!("{}…", chars.iter().collect::<String>()).into();
        if text::measure(&candidate, size) <= width {
            return candidate;
        }
    }
    SharedString::Borrowed("…")
}

fn circle(painter: &mut Painter, center: Point, r: f32, background: Hsla, border: f32, color: Hsla) {
    let side = r * 2.;
    painter.quad(Bounds::centered_at(center, size(side, side)).into(), r, background, color, border);
}

fn arc(
    center: Point,
    r: f32,
    from: f32,
    length: f32,
    width: f32,
    color: Hsla,
    painter: &mut Painter,
) {
    let steps = ((length * 120.).ceil() as usize).max(2);
    let mut builder = Path::with_capacity(steps + 1);
    for step in 0..=steps {
        let fraction = from + length * step as f32 / steps as f32;
        let angle = std::f32::consts::TAU * fraction;
        let at = center + point(angle.sin() * r, -angle.cos() * r);
        if step == 0 {
            builder.move_to(at);
        } else {
            builder.line_to(at);
        }
    }
    builder.stroke(painter, color, width);
}

struct Label {
    text: SharedString,
    at: Point,
    size: f32,
    color: Hsla,
}

fn related(graph: &Graph, hover: Option<Target>, node: usize) -> bool {
    match hover {
        Some(Target::Node(hovered)) => {
            hovered == node
                || graph.around[hovered]
                    .iter()
                    .any(|edge| graph.other(*edge, hovered) == Some(node))
        }
        Some(Target::Edge(edge)) => graph
            .links
            .get(edge)
            .copied()
            .flatten()
            .is_some_and(|(source, target)| source == node || target == node),
        _ => true,
    }
}

#[allow(clippy::too_many_lines)]
pub(crate) fn paint(bounds: Bounds, arcs: &[ArcSection], formats: Formats, state: State, ui: Ui, painter: &mut Painter) {
    state.bounds.update_untracked(|slot| *slot = bounds);
    let mut current = state.view.get();
    if current.fit {
        current = state.graph.with(|graph| fitted(graph, bounds));
        state.view.update_untracked(|view| *view = current);
    }
    let frame = Frame { bounds, view: current };
    let hover = state.hover.get();
    let scale = frame.view.scale;
    let font = FONT * scale;
    let mut labels: Vec<Label> = Vec::new();
    state.graph.with(|graph| {
        let placed = graph.placed();
        for (index, edge) in graph.edges.iter().enumerate() {
            let Some(line) = segment(graph, &frame, index) else {
                continue;
            };
            let hovered = hover == Some(Target::Edge(index))
                || matches!(hover, Some(Target::Node(node)) if graph.around[node].contains(&index));
            let color = if edge.highlighted { edge.color.unwrap_or(ui.highlight) } else { edge.color.unwrap_or(ui.edge) };
            let width =
                (edge.thickness + f32::from(u8::from(hovered)) + f32::from(u8::from(edge.highlighted))) * scale;
            let builder = Path::polyline([line.start, line.end]);
            if edge.dashed {
                builder.stroke_dashed(painter, color, width.max(0.5), &[5. * scale, 5. * scale]);
            } else {
                builder.stroke(painter, color, width.max(0.5));
            }
            let (dx, dy) = (line.tip.x - line.end.x, line.tip.y - line.end.y);
            let length = dx.hypot(dy).max(f32::EPSILON);
            let (nx, ny) = (-dy / length * line.head / 2., dx / length * line.head / 2.);
            Path::polygon(&[line.tip, line.end + point(nx, ny), line.end - point(nx, ny)]).fill(painter, color);
            if hovered {
                let stats: Vec<SharedString> =
                    [stat(&edge.main_stat, formats.edge_main), stat(&edge.secondary_stat, formats.edge_secondary)]
                        .into_iter()
                        .filter(|line| !line.is_empty())
                        .collect();
                if !stats.is_empty() {
                    let middle = point(f32::midpoint(line.start.x, line.tip.x), f32::midpoint(line.start.y, line.tip.y));
                    let height = 15. * stats.len() as f32 * scale;
                    let card = Bounds::new(middle - point(40. * scale, height / 2.), size(80. * scale, height));
                    painter.quad(card.into(), 2., ui.popover, Hsla::TRANSPARENT, 0.);
                    for (row, text) in stats.into_iter().enumerate() {
                        labels.push(Label {
                            text,
                            at: point(middle.x, card.top() + (row as f32).mul_add(15., 3.5) * scale),
                            size: EDGE_FONT * scale,
                            color: ui.popover_text,
                        });
                    }
                }
            }
        }
        for (index, node) in graph.nodes.iter().enumerate() {
            let Some(position) = placed.get(index) else { continue };
            let center = frame.at(*position);
            if graph.view.marked(index) {
                circle(painter, center, NODE_RADIUS * scale, ui.background, 1., ui.border);
                let (_, count) =
                    graph.view.markers.iter().find(|(marker, _)| *marker == index).copied().unwrap_or_default();
                let text: SharedString = if count > MARKER_LIMIT {
                    SharedString::Borrowed(">100 nodes")
                } else {
                    format!("{count} nodes").into()
                };
                labels.push(Label { text, at: center - point(0., font / 2.), size: font, color: ui.text });
                continue;
            }
            if !graph.view.shows(index) {
                continue;
            }
            let alpha = if related(graph, hover, index) { 1. } else { DIM };
            let is_hovered = hover == Some(Target::Node(index));
            let r = radius(node);
            let ring = ring_width(r) * scale;
            let fill_color = if node.highlighted { ui.highlight } else { ui.background };
            circle(painter, center, r * scale, fill_color.opacity(alpha), 0., Hsla::TRANSPARENT);
            let full = node.arcs.iter().position(|value| *value >= 1.).and_then(|index| arcs.get(index));
            let any = node.arcs.iter().any(|value| *value != 0.);
            if let Some(section) = full {
                circle(painter, center, r * scale, Hsla::TRANSPARENT, ring, section.color.opacity(alpha));
            } else if any {
                let mut start = 0.;
                for (value, section) in node.arcs.iter().zip(arcs) {
                    if *value == 0. {
                        continue;
                    }
                    let length = (*value as f32).min(1. - start);
                    if length > 0. {
                        arc(center, r * scale, start, length, ring, section.color.opacity(alpha), painter);
                    }
                    start += *value as f32;
                }
            } else {
                circle(painter, center, r * scale, Hsla::TRANSPARENT, ring, node.color.unwrap_or(ui.ring).opacity(alpha));
            }
            if is_hovered {
                circle(painter, center, (r - 3.) * scale, Hsla::TRANSPARENT, ring, ui.accent.opacity(0.5));
            }
            if font < MIN_FONT {
                continue;
            }
            let stats_width = if is_hovered { HOVER_WIDTH } else { STATS_WIDTH } * scale;
            let title_width = if is_hovered { HOVER_WIDTH } else { TITLE_WIDTH } * scale;
            let text_color = ui.text.opacity(alpha);
            let lines = [
                (stat(&node.main_stat, formats.node_main), -15. + 1.),
                (stat(&node.secondary_stat, formats.node_secondary), -15. + 1. + LINE),
                (node.title.clone(), r + 5. + 1.),
                (node.sub_title.clone(), r + 5. + 1. + LINE),
            ];
            for (slot, (text, offset)) in lines.into_iter().enumerate() {
                if text.is_empty() {
                    continue;
                }
                let width = if slot < 2 { stats_width } else { title_width };
                labels.push(Label {
                    text: clip(&text, width, font),
                    at: center + point(0., offset * scale),
                    size: font,
                    color: text_color,
                });
            }
        }
    });
    for label in &labels {
        text::draw(painter, &label.text, label.at, TextAlign::Center, label.size, label.color);
    }
}

/// The pointer moved: a press pans once past the click slop, else the hovered node or edge.
pub(crate) fn on_move(state: State, event: &MouseMoveEvent) {
    let position = Point::from(event.position);
    if let Some(mut current) = state.press.get() {
        if event.pressed_button.is_none() {
            state.press.set(None);
            return;
        }
        let (dx, dy) = (position.x - current.from.x, position.y - current.from.y);
        if current.moved || dx.hypot(dy) > CLICK_SLOP {
            current.moved = true;
            let scale = state.view.get().scale;
            let pan = [current.pan[0] + dx / scale, current.pan[1] + dy / scale];
            state.view.update(|view| {
                view.pan = pan;
                view.fit = false;
            });
            state.press.set(Some(current));
        }
        return;
    }
    let frame = Frame { bounds: state.bounds.get(), view: state.view.get() };
    let target = state.graph.with(|graph| hit(graph, &frame, position));
    if state.hover.get() != target {
        state.hover.set(target);
    }
}

pub(crate) fn on_leave(state: State) {
    if state.hover.get().is_some() {
        state.hover.set(None);
    }
}

pub(crate) fn on_down(state: State, event: &MouseDownEvent) {
    if event.button != MouseButton::Left {
        return;
    }
    let position = Point::from(event.position);
    let frame = Frame { bounds: state.bounds.get(), view: state.view.get() };
    let target = state.graph.with(|graph| hit(graph, &frame, position));
    state.press.set(Some(Press { from: position, pan: frame.view.pan, target, moved: false }));
}

/// A press released without panning: a click on a node, an edge, a marker (rooted there) or
/// nothing (the detail card closes).
pub(crate) fn on_up(state: State, event: &MouseUpEvent, window: &mut Window, cx: &mut App) {
    let State { graph, view, press, picked, root, clicks, bounds, .. } = state;
    let Some(done) = press.get() else { return };
    press.set(None);
    if done.moved {
        return;
    }
    let bounds = bounds.get();
    let at = Point::from(event.position) - bounds.origin;
    match done.target {
        None => picked.set(None),
        Some(Target::Marker(node)) => {
            root.set(Some(node));
            view.set(View::default());
            picked.set(None);
        }
        Some(target @ Target::Node(node)) => {
            let item = graph.with(|graph| graph.nodes.get(node).cloned());
            let mut slot = Clicks::default();
            clicks.update_untracked(|current| std::mem::swap(current, &mut slot));
            match (&slot.node, item) {
                (Some(callback), Some(item)) => callback(&item, window, cx),
                _ => picked.set(Some((target, at, bounds.size))),
            }
            clicks.update_untracked(|current| std::mem::swap(current, &mut slot));
        }
        Some(target @ Target::Edge(edge)) => {
            let item = graph.with(|graph| graph.edges.get(edge).cloned());
            let mut slot = Clicks::default();
            clicks.update_untracked(|current| std::mem::swap(current, &mut slot));
            match (&slot.edge, item) {
                (Some(callback), Some(item)) => callback(&item, window, cx),
                _ => picked.set(Some((target, at, bounds.size))),
            }
            clicks.update_untracked(|current| std::mem::swap(current, &mut slot));
        }
    }
}

/// The wheel zooms around the pointer (cooperative: only with Ctrl, as a pinch reports itself).
pub(crate) fn on_wheel(state: State, zoom_mode: ZoomMode, event: &ScrollWheelEvent, cx: &mut App) {
    let wanted = match zoom_mode {
        ZoomMode::Greedy => true,
        ZoomMode::Cooperative => event.modifiers.control,
    };
    if !wanted {
        return;
    }
    let delta = event.pixels.unwrap_or(event.delta * 20.);
    let position = Point::from(event.position);
    let bounds = state.bounds.get();
    let current = state.view.get();
    let scale = (current.scale * (delta * ZOOM_SPEED).exp()).clamp(MIN_SCALE, MAX_SCALE);
    let anchor = Frame { bounds, view: current }.world(position);
    let center = bounds.center();
    let offset = [(position.x - center.x) / scale - anchor[0], (position.y - center.y) / scale - anchor[1]];
    state.view.set(View { scale, pan: offset, fit: false });
    cx.stop_propagation();
    cx.prevent_default();
}
