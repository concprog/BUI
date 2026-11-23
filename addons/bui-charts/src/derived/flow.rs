use std::collections::HashMap;

use blenderui::element::{AnyElement, Div, FluentBuilder, IntoElement, ParentElement};
use blenderui::hooks::{Signal, use_signal};
use blenderui::html::div;
use blenderui::style::TextAlign;
use blenderui::{App, Hsla, MouseMoveEvent, Painter, SharedString, Window, canvas, rsx};

use super::sankey::{Sankey as Layout, SankeyAlign, SankeyGraph, SankeyLink, sankey_link_path};
use super::{Tip, card};
use crate::derive::data_key;
use crate::flame_graph::{FlameGraph, Pane, PaneLayout, ProfileData};
use crate::geom::{Bounds, Point, Size, point};
use crate::mappings::StateValue;
use crate::plot::{AxisConfig, SelectedRange, Tooltip, TooltipMode, VisibilityMode};
use crate::state_timeline::{Cells, Reader, StateLane, Style, Timeline, ValueAlign, timeline};
use crate::{ChartTheme, Dataset, Legend, component, text};

#[component]
pub fn gantt<
    T: 'static,
    D: Dataset<T>,
    N: Fn(&T) -> &str + 'static,
    S: Fn(&T) -> f64 + 'static,
    E: Fn(&T) -> f64 + 'static,
    G: Fn(&T) -> &str + 'static,
>(
    data: D,
    name: N,
    start: S,
    end: E,
    group: Option<G>,
    row_height: Option<f32>,
    show_value: Option<VisibilityMode>,
    x_axis: Option<AxisConfig>,
    legend: Option<Legend>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
) -> AnyElement {
    let (times, lanes) = data.rows(|items| {
    let mut times: Vec<f64> = items
        .iter()
        .flat_map(|item| [start(item), end(item)])
        .filter(|value| value.is_finite())
        .collect();
    times.sort_by(f64::total_cmp);
    times.dedup();
    let lanes: Vec<StateLane<f64>> = items
        .iter()
        .map(|item| {
            let (from, to) = (start(item), end(item));
            let label = SharedString::from(
                group
                    .as_ref()
                    .map_or_else(|| name(item), |group| group(item))
                    .to_owned(),
            );
            StateLane {
                name: Some(SharedString::from(name(item).to_owned())),
                value: Reader::Row(Box::new(move |at: &f64| {
                    if *at >= from && *at < to {
                        StateValue::Text(label.clone())
                    } else {
                        StateValue::Null
                    }
                })),
                mappings: None,
                color: None,
                thresholds: None,
                style: Style::default(),
            }
        })
        .collect();
    (times, lanes)
    });
    let count = lanes.len();
    let end_at = times.last().copied();
    let mut axis = x_axis.unwrap_or_else(|| AxisConfig::new().time());
    axis.max = axis.max.or(end_at);
    timeline(
        Timeline {
            data: times,
            x: Some(|at: &f64| *at),
            x_field: None,
            cells: Cells::Merged {
                merge: true,
                align: ValueAlign::Left,
            },
            show_value,
            row_height: Some(row_height.unwrap_or(0.7)),
            per_page: Some(count.max(1)),
            mappings: None,
            color: None,
            thresholds: None,
            min: None,
            max: None,
            style: Style {
                fill_opacity: Some(0.85),
                line_width: Some(0.),
                ..Style::default()
            },
            x_axis: Some(axis),
            y_axis: None,
            legend,
            tooltip,
            class,
            lanes,
        },
        None::<fn(SelectedRange, &mut Window, &mut App)>)
}

#[component]
pub fn icicle<D: ProfileData>(data: D, header: Option<bool>, class: Option<SharedString>) -> AnyElement {
    let layout = PaneLayout::Single(Pane::FlameGraph);
    let header = header.unwrap_or(false);
    let graph = rsx! { <FlameGraph data={data} layout={layout} header={header} /> };
    div().class("size-full").when_some(class, Div::class).child(graph).into_any_element()
}

const LABEL_SPACE: f32 = 120.;
const MIN_RIBBON: f32 = 1.;
const HEADER: f32 = 22.;

#[derive(Default)]
struct Flows {
    key: Option<u64>,
    names: Vec<SharedString>,
    stages: Vec<SharedString>,
    node_stage: Vec<usize>,
    node_tints: Vec<usize>,
    links: Vec<SankeyLink>,
    link_tints: Vec<usize>,
    values: Vec<f64>,
}

#[derive(Default)]
struct Builder {
    flows: Flows,
    nodes: HashMap<(usize, String), usize>,
    tints: HashMap<String, usize>,
    pairs: HashMap<(usize, usize, usize), usize>,
}

impl Builder {
    fn tint(&mut self, label: &str) -> usize {
        let next = self.tints.len();
        *self.tints.entry(label.to_owned()).or_insert(next)
    }

    fn node(&mut self, stage: usize, label: &str, tint: Option<usize>) -> usize {
        let flows = &mut self.flows;
        *self
            .nodes
            .entry((stage, label.to_owned()))
            .or_insert_with(|| {
                let index = flows.names.len();
                flows.names.push(SharedString::from(label.to_owned()));
                flows.node_stage.push(stage);
                flows.node_tints.push(tint.unwrap_or(index));
                index
            })
    }

    fn link(&mut self, from: usize, to: usize, tint: usize, amount: f64) {
        if from == to || !amount.is_finite() || amount <= 0. {
            return;
        }
        let flows = &mut self.flows;
        let slot = *self.pairs.entry((from, to, tint)).or_insert_with(|| {
            flows.links.push(SankeyLink::new(from, to, 0.));
            flows.link_tints.push(tint);
            flows.values.push(0.);
            flows.values.len() - 1
        });
        flows.values[slot] += amount;
    }

    fn finish(mut self) -> Flows {
        for (link, value) in self.flows.links.iter_mut().zip(&self.flows.values) {
            *link = SankeyLink::new(link.source, link.target, *value);
        }
        self.flows
    }
}

#[derive(Default)]
struct Placed {
    key: Option<(u64, u32, u32)>,
    /// where the canvas last painted (the pointer handlers' frame)
    bounds: Bounds,
    graph: SankeyGraph,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Target {
    Node(usize),
    Link(usize),
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Hover {
    target: Target,
    at: Point,
    area: Size,
}

#[derive(Clone, Copy)]
struct Look {
    node_width: f32,
    align: SankeyAlign,
    node_padding: f32,
    opacity: f32,
    text: Hsla,
    muted: Hsla,
}

/// A flow chart's state: its flows, their layout for the canvas, and what the pointer is over.
#[derive(Clone, Copy)]
struct State {
    flows: Signal<Flows>,
    placed: Signal<Placed>,
    hover: Signal<Option<Hover>>,
}

#[track_caller]
fn use_state() -> State {
    State { flows: use_signal(Flows::default()), placed: use_signal(Placed::default()), hover: use_signal(None) }
}

fn contains(graph: &SankeyGraph, cursor: Point) -> Option<Target> {
    if let Some(node) = graph.nodes.iter().position(|node| {
        cursor.x >= node.x0 && cursor.x <= node.x1 && cursor.y >= node.y0 && cursor.y <= node.y1
    }) {
        return Some(Target::Node(node));
    }
    graph
        .links
        .iter()
        .position(|link| {
            let (source, target) = (&graph.nodes[link.source], &graph.nodes[link.target]);
            let span = target.x0 - source.x1;
            if span <= 0. || cursor.x < source.x1 || cursor.x > target.x0 {
                return false;
            }
            let t = (cursor.x - source.x1) / span;
            let ease = t * t * (3. - 2. * t);
            let middle = (link.y1 - link.y0).mul_add(ease, link.y0);
            let half = (link.target_width - link.source_width).mul_add(t, link.source_width).max(MIN_RIBBON) / 2.;
            (cursor.y - middle).abs() <= half
        })
        .map(Target::Link)
}

/// Lays the flows out for `bounds` unless the layout already fits them.
fn place(state: State, bounds: Bounds, look: Look) {
    let (width, height) = (bounds.size.width, bounds.size.height);
    let (top, key) = state.flows.with(|flows| {
        let top = if flows.stages.is_empty() { 4. } else { HEADER };
        (top, flows.key.map(|key| (key, width.to_bits(), height.to_bits())))
    });
    state.placed.update_untracked(|placed| placed.bounds = bounds);
    if key.is_some() && state.placed.with(|placed| placed.key) == key {
        return;
    }
    let graph = state.flows.with(|flows| {
        Layout::new()
            .node_width(look.node_width)
            .node_padding(look.node_padding)
            .node_align(look.align)
            .extent(LABEL_SPACE / 4., top, (width - LABEL_SPACE / 4.).max(1.), (height - 4.).max(top + 1.))
            .layout(flows.names.len(), &flows.links)
            .unwrap_or_default()
    });
    state.placed.update_untracked(|placed| {
        placed.key = key;
        placed.graph = graph;
    });
}

fn paint_flows(bounds: Bounds, state: State, look: Look, painter: &mut Painter) {
    place(state, bounds, look);
    let width = bounds.size.width;
    let colors = ChartTheme::colors();
    let hovered = state.hover.get();
    state.flows.with(|current| {
        state.placed.with(|placed| {
            let graph = &placed.graph;
            let active = |link: usize| match hovered.map(|hover| hover.target) {
                Some(Target::Link(index)) => index == link,
                Some(Target::Node(node)) => {
                    let link = &graph.links[link];
                    link.source == node || link.target == node
                }
                None => true,
            };
            for (index, link) in graph.links.iter().enumerate() {
                let (source, target) = (&graph.nodes[link.source], &graph.nodes[link.target]);
                let alpha = if active(index) { look.opacity } else { look.opacity * 0.35 };
                let tint = current.link_tints.get(index).copied().unwrap_or(link.source);
                sankey_link_path(source, target, link, MIN_RIBBON, bounds.origin)
                    .fill(painter, colors.series(tint).opacity(alpha));
            }
            for (index, node) in graph.nodes.iter().enumerate() {
                let area =
                    Bounds::from_corners(bounds.origin + point(node.x0, node.y0), bounds.origin + point(node.x1, node.y1));
                let tint = current.node_tints.get(index).copied().unwrap_or(index);
                painter.fill(area.into(), colors.series(tint));
                let middle = f32::midpoint(node.y0, node.y1) - crate::plot::TEXT_SIZE / 2.;
                let (x, align) =
                    if node.x0 < width / 2. { (node.x1 + 6., TextAlign::Left) } else { (node.x0 - 6., TextAlign::Right) };
                if let Some(name) = current.names.get(index) {
                    let at = bounds.origin + point(x, middle);
                    text::draw(painter, name, at, align, crate::plot::TEXT_SIZE, look.text);
                }
            }
            for (stage, name) in current.stages.iter().enumerate() {
                let column = graph
                    .nodes
                    .iter()
                    .enumerate()
                    .filter(|(index, _)| current.node_stage.get(*index) == Some(&stage))
                    .map(|(_, node)| f32::midpoint(node.x0, node.x1))
                    .fold(f32::NAN, f32::min);
                if column.is_finite() {
                    let at = bounds.origin + point(column, 2.);
                    text::draw(painter, name, at, TextAlign::Center, crate::plot::TEXT_SIZE, look.muted);
                }
            }
        });
    });
}

fn hover_flows(state: State, event: &MouseMoveEvent) {
    let next = state.placed.with(|placed| {
        let cursor = Point::from(event.position) - placed.bounds.origin;
        contains(&placed.graph, cursor).map(|target| Hover { target, at: cursor, area: placed.bounds.size })
    });
    if state.hover.get() != next {
        state.hover.set(next);
    }
}

fn flow_tip(flows: &Flows, graph: &SankeyGraph, hovered: Hover) -> Option<Tip> {
    let sum = |links: &[usize]| -> f64 {
        links
            .iter()
            .map(|link| flows.values.get(*link).copied().unwrap_or(0.))
            .sum()
    };
    let (title, amount) = match hovered.target {
        Target::Node(node) => {
            let laid = graph.nodes.get(node)?;
            (
                flows.names.get(node)?.clone(),
                sum(&laid.target_links).max(sum(&laid.source_links)),
            )
        }
        Target::Link(link) => {
            let laid = graph.links.get(link)?;
            let names = (flows.names.get(laid.source)?, flows.names.get(laid.target)?);
            (
                SharedString::from(format!("{} → {}", names.0, names.1)),
                flows.values.get(link).copied().unwrap_or(0.),
            )
        }
    };
    Some(Tip {
        at: hovered.at,
        area: hovered.area,
        title,
        accent: None,
        rows: vec![(
            None,
            SharedString::Borrowed("Value"),
            crate::format::trimmed(amount),
        )],
    })
}

struct FlowSpec {
    align: SankeyAlign,
    node_width: Option<f32>,
    node_padding: Option<f32>,
    link_opacity: Option<f32>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
}

fn flow_view(state: State, key: Option<u64>, build: impl FnOnce() -> Flows, spec: FlowSpec) -> AnyElement {
    if key.is_none() || state.flows.with(|flows| flows.key) != key {
        let mut next = build();
        next.key = key;
        state.flows.update_untracked(|flows| *flows = next);
        state.placed.update_untracked(|placed| placed.key = None);
    }
    let theme = crate::ui::theme();
    let look = Look {
        node_width: spec.node_width.unwrap_or(12.),
        align: spec.align,
        node_padding: spec.node_padding.unwrap_or(12.),
        opacity: spec.link_opacity.unwrap_or(0.4),
        text: theme.foreground,
        muted: theme.muted_foreground,
    };
    let shown = spec.tooltip.unwrap_or_default().mode != TooltipMode::Hidden;
    let popup = state
        .hover
        .get()
        .filter(|_| shown)
        .and_then(|hovered| state.flows.with(|flows| state.placed.with(|placed| flow_tip(flows, &placed.graph, hovered))))
        .map(card);
    div()
        .class("relative size-full min-w-0 min-h-0")
        .when_some(spec.class, Div::class)
        .child(
            canvas(move |bounds, painter| paint_flows(bounds.into(), state, look, painter))
                .class("absolute inset-0 size-full")
                .on_mouse_move(move |event, _, _| hover_flows(state, event))
                .on_hover(move |inside, _, _| {
                    if !inside && state.hover.get().is_some() {
                        state.hover.set(None);
                    }
                }),
        )
        .child(popup)
        .into_any_element()
}

#[component]
pub fn sankey<
    T: 'static,
    D: Dataset<T>,
    S: Fn(&T) -> &str + 'static,
    G: Fn(&T) -> &str + 'static,
    V: Fn(&T) -> f64 + 'static,
>(
    data: D,
    source: S,
    target: G,
    value: V,
    node_width: Option<f32>,
    node_padding: Option<f32>,
    link_opacity: Option<f32>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
) -> AnyElement {
    let state = use_state();
    let key = data_key(&data, ());
    flow_view(
        state,
        key,
        || {
            let mut builder = Builder::default();
            data.rows(|rows| {
                for item in rows {
                    let from = builder.node(0, source(item), None);
                    let to = builder.node(0, target(item), None);
                    builder.link(from, to, from, value(item));
                }
            });
            builder.finish()
        },
        FlowSpec {
            align: SankeyAlign::Justify,
            node_width,
            node_padding,
            link_opacity,
            tooltip,
            class,
        })
}

pub struct StageField<T> {
    name: SharedString,
    value: Box<dyn Fn(&T) -> &str>,
}

#[component]
pub fn stage<T: 'static, V: Fn(&T) -> &str + 'static>(
    name: SharedString,
    value: V,
) -> StageField<T> {
    StageField {
        name,
        value: Box::new(value),
    }
}

#[component]
pub fn alluvial<T: 'static, D: Dataset<T>, V: Fn(&T) -> f64 + 'static>(
    data: D,
    value: Option<V>,
    node_width: Option<f32>,
    node_padding: Option<f32>,
    link_opacity: Option<f32>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
    children: Children<StageField<T>>,
) -> AnyElement {
    let state = use_state();
    let stages = children;
    let names: Vec<SharedString> = stages.iter().map(|stage| stage.name.clone()).collect();
    let key = data_key(&data, &names);
    flow_view(
        state,
        key,
        || {
            let mut builder = Builder::default();
            data.rows(|rows| for item in rows {
                let Some(first) = stages.first() else {
                    break;
                };
                let amount = value.as_ref().map_or(1., |value| value(item));
                let origin = builder.tint((first.value)(item));
                let mut previous: Option<usize> = None;
                for (index, stage) in stages.iter().enumerate() {
                    let label = (stage.value)(item);
                    let tint = builder.tint(label);
                    let node = builder.node(index, label, Some(tint));
                    if let Some(from) = previous {
                        builder.link(from, node, origin, amount);
                    }
                    previous = Some(node);
                }
            });
            let mut flows = builder.finish();
            flows.stages = names;
            flows
        },
        FlowSpec {
            align: SankeyAlign::Justify,
            node_width,
            node_padding,
            link_opacity,
            tooltip,
            class,
        })
}

#[component]
pub fn sankey_funnel<
    T: 'static,
    D: Dataset<T>,
    N: Fn(&T) -> &str + 'static,
    V: Fn(&T, usize) -> f64 + 'static,
>(
    data: D,
    stages: Vec<SharedString>,
    name: N,
    value: V,
    source_label: Option<SharedString>,
    drop_label: Option<SharedString>,
    drops: Option<bool>,
    node_width: Option<f32>,
    node_padding: Option<f32>,
    link_opacity: Option<f32>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
) -> AnyElement {
    let state = use_state();
    let source = source_label.unwrap_or(SharedString::Borrowed("Source"));
    let dropped = drop_label.unwrap_or(SharedString::Borrowed("Dropped"));
    let leaks = drops.unwrap_or(true);
    let key = data_key(&data, (&stages, &source, &dropped, leaks));
    flow_view(
        state,
        key,
        || {
            let mut builder = Builder::default();
            data.rows(|rows| {
            let channels = rows.len();
            for (channel, item) in rows.iter().enumerate() {
                let entry = builder.node(0, name(item), Some(channel));
                let mut previous = (entry, f64::INFINITY);
                for (index, stage) in stages.iter().enumerate() {
                    let reached = value(item, index).min(previous.1).max(0.);
                    let node = builder.node(index + 1, stage, Some(channels + index));
                    builder.link(previous.0, node, channel, reached);
                    if leaks && index > 0 {
                        let lost = previous.1 - reached;
                        let label = format!("{dropped} after {}", stages[index - 1]);
                        let sink = builder.node(index + 1, &label, Some(channels + stages.len()));
                        builder.link(previous.0, sink, channel, lost);
                    }
                    previous = (node, reached);
                }
            }
            });
            let mut flows = builder.finish();
            flows.stages = std::iter::once(source)
                .chain(stages.iter().cloned())
                .collect();
            flows
        },
        FlowSpec {
            align: SankeyAlign::Left,
            node_width,
            node_padding,
            link_opacity,
            tooltip,
            class,
        })
}
