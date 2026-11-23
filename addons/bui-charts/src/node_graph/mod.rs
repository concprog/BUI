mod data;
mod layout;
mod model;
mod paint;

use std::hash::{DefaultHasher, Hash, Hasher};

use blenderui::element::{AnyElement, Div, FluentBuilder, IntoElement, ParentElement};
use blenderui::hooks::use_signal;
use blenderui::html::div;
use blenderui::{App, MouseButton, SharedString, Styled, Window, canvas, rgb};

pub use data::{ArcSection, GraphEdge, GraphNode, NodeLayout, StatFormats, ZoomMode};

use crate::geom::{Bounds, Point, Size};
use crate::icon::{IconName, icon};
use crate::plot::corner;
use crate::widgets::{button, icon_button, ui as widget_ui};
use crate::{Dataset, component};
use model::{Graph, Shape};
use paint::{Clicks, Picked, Target, Ui, View, paint, stat};

const NODE_LIMIT: usize = 200;
const EDGE: u32 = 0x999999;
const HIGHLIGHT: u32 = 0xaa0000;
const STEP: f32 = 1.25;

type Detail = (Point, Size, SharedString, SharedString, Vec<(SharedString, SharedString)>);

#[component]
#[allow(clippy::too_many_lines)]
pub fn node_graph<
    N: Dataset<GraphNode>,
    E: Dataset<GraphEdge>,
    F: Fn(&GraphNode, &mut Window, &mut App) + 'static,
    G: Fn(&GraphEdge, &mut Window, &mut App) + 'static,
>(
    nodes: N,
    edges: E,
    layout: Option<NodeLayout>,
    zoom_mode: Option<ZoomMode>,
    arcs: Option<Vec<ArcSection>>,
    node_limit: Option<usize>,
    formats: Option<StatFormats>,
    legend: Option<bool>,
    on_node_click: Option<F>,
    on_edge_click: Option<G>,
    class: Option<SharedString>,
) -> AnyElement {
    let graph = use_signal(Graph::default());
    let view_state = use_signal(View::default());
    let hover = use_signal(None::<Target>);
    let press = use_signal(None);
    let picked = use_signal(None::<Picked>);
    let root = use_signal(None::<usize>);
    let clicks = use_signal(Clicks::default());
    let chosen = use_signal(None::<NodeLayout>);
    let sort = use_signal(None::<(usize, bool)>);
    let bounds = use_signal(Bounds::default());
    clicks.update_untracked(|slot| {
        slot.node = on_node_click.map(|f| Box::new(f) as Box<_>);
        slot.edge = on_edge_click.map(|f| Box::new(f) as Box<_>);
    });

    let key = nodes.revision().zip(edges.revision()).map(|pair| {
        let mut state = DefaultHasher::new();
        pair.hash(&mut state);
        state.finish()
    });
    if !graph.with(|graph| graph.fresh(key)) {
        nodes.rows(|node_rows| {
            edges.rows(|edge_rows| graph.update_untracked(|graph| graph.load(key, node_rows, edge_rows)));
        });
        root.update_untracked(|root| *root = None);
        picked.update_untracked(|picked| *picked = None);
    }
    let current = chosen.get().unwrap_or_else(|| layout.unwrap_or_default());
    let limit = node_limit.unwrap_or(NODE_LIMIT).max(1);
    let arcs = arcs.unwrap_or_default();
    let formats = formats.unwrap_or_default();
    let theme = crate::ui::theme();
    let ui = Ui {
        text: theme.foreground,
        muted: theme.muted_foreground,
        background: theme.background,
        border: theme.border,
        accent: theme.primary,
        popover: theme.popover,
        popover_text: theme.popover_foreground,
        edge: rgb(EDGE),
        highlight: rgb(HIGHLIGHT),
        ring: theme.muted_foreground,
    };
    let widgets = widget_ui();
    let zoom_mode = zoom_mode.unwrap_or_default();
    let state = paint::State { bounds, graph, view: view_state, hover, press, picked, root, clicks };
    let empty = graph.with(|graph| graph.nodes.is_empty());
    let sorted = sort.get();
    let rooted = root.get();
    let shape = move |width: f32| Shape {
        layout: current,
        width: if current == NodeLayout::Grid { width.max(0.) as u32 } else { 0 },
        sort: sorted,
        limit,
        root: rooted,
    };
    let detail = picked.get().and_then(|(target, at, area)| graph.with(|graph| detail(graph, target, at, area, &arcs, formats)));
    // muted text on the popover's own background (the theme's muted text is for the page)
    let card_muted = theme.popover_muted;
    let card = detail.map(|(at, area, title, sub, rows)| {
        corner(crate::trend::card(), at, area)
            .id("node-graph-detail")
            .class("min-w-40 max-w-72")
            .style("pointer-events: auto")
            .bg(ui.popover)
            .text_color(ui.popover_text)
            .border_color(ui.border)
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .child(
                div()
                    .class("flex items-center justify-between gap-2")
                    .child(div().class("font-semibold").child(title))
                    .child(
                        div()
                            .id("close-detail")
                            .class("cursor-pointer")
                            .on_click(move |_, _, _| picked.set(None))
                            .child(icon(IconName::Close, card_muted)),
                    ),
            )
            .child((!sub.is_empty()).then(|| div().text_color(card_muted).child(sub)))
            .children(rows.into_iter().map(|(name, value)| {
                div()
                    .class("flex justify-between gap-4")
                    .child(div().text_color(card_muted).child(name))
                    .child(div().child(value))
            }))
    });
    // zooming leaves the fitted view (a fitted one is fitted again on every paint)
    let zoom_by = move |factor: f32| {
        view_state.update(|view| {
            view.scale = (view.scale * factor).clamp(0.1, 10.);
            view.fit = false;
        });
    };
    let pick = move |layout: NodeLayout| {
        chosen.set(Some(layout));
        view_state.set(View::default());
    };
    let controls = div()
        .class("absolute top-2 right-2 flex items-center gap-1")
        .child(icon_button("zoom-in", IconName::Plus, false, widgets, move |_, _, _| zoom_by(STEP)))
        .child(icon_button("zoom-out", IconName::Minus, false, widgets, move |_, _, _| zoom_by(1. / STEP)))
        .child(icon_button("reset-view", IconName::Maximize, false, widgets, move |_, _, _| {
            view_state.set(View::default());
            root.set(None);
        }))
        .child(button("layered", "Layered", current == NodeLayout::Layered, widgets, move |_, _, _| {
            pick(NodeLayout::Layered);
        }))
        .child(button("force", "Force", current == NodeLayout::Force, widgets, move |_, _, _| pick(NodeLayout::Force)))
        .child(button("grid", "Grid", current == NodeLayout::Grid, widgets, move |_, _, _| pick(NodeLayout::Grid)));
    let legend_view = (legend.unwrap_or(true) && !arcs.is_empty()).then(|| {
        div()
            .class("flex flex-wrap items-center gap-x-4 gap-y-1 px-2 py-1 text-xs")
            .text_color(ui.muted)
            .children(arcs.iter().enumerate().map(|(index, section)| {
                div()
                    .id(format!("arc-{index}"))
                    .class("flex items-center gap-2 cursor-pointer")
                    .on_click(move |_, _, _| {
                        chosen.set(Some(NodeLayout::Grid));
                        sort.set(Some(match sorted {
                            Some((arc, ascending)) if arc == index => (arc, !ascending),
                            _ => (index, false),
                        }));
                    })
                    .child(div().class("w-3 h-3 rounded-full").bg(section.color))
                    .child(section.name.clone())
                    .child(sorted.filter(|(arc, _)| *arc == index).map(|(_, ascending)| {
                        icon(if ascending { IconName::ArrowUp } else { IconName::ArrowDown }, ui.muted)
                    }))
            }))
    });
    let hint = empty.then(|| {
        div().class("absolute inset-0 flex items-center justify-center text-sm").text_color(ui.muted).child("No nodes")
    });
    let surface = canvas(move |area, painter| {
        let area = Bounds::from(area);
        graph.update_untracked(|graph| graph.ensure(shape(area.size.width)));
        paint(area, &arcs, formats, state, ui, painter);
    })
    .class("absolute inset-0 size-full")
    .on_mouse_move(move |event, _, _| paint::on_move(state, event))
    .on_hover(move |inside, _, _| {
        if !inside {
            paint::on_leave(state);
        }
    })
    .on_mouse_down(MouseButton::Left, move |event, _, _| paint::on_down(state, event))
    .on_mouse_up(MouseButton::Left, move |event, window, cx| paint::on_up(state, event, window, cx))
    .on_scroll_wheel(move |event, _, cx| paint::on_wheel(state, zoom_mode, event, cx));

    div()
        .id("node-graph")
        .class("flex flex-col size-full min-w-0 min-h-0")
        .when_some(class, Div::class)
        .child(
            div()
                .class("relative flex-1 min-w-0 min-h-0")
                .child(surface)
                .child(controls)
                .child(hint)
                .child(card),
        )
        .child(legend_view)
        .into_any_element()
}

/// The detail card's title, subtitle and rows for a clicked node or edge.
fn detail(
    graph: &Graph,
    target: Target,
    at: Point,
    area: Size,
    arcs: &[ArcSection],
    formats: StatFormats,
) -> Option<Detail> {
    let (title, sub, mut rows, details): (SharedString, SharedString, Vec<(SharedString, SharedString)>, _) =
        match target {
            Target::Node(node) => {
                let node = graph.nodes.get(node)?;
                let mut rows = vec![
                    (SharedString::Borrowed("Main stat"), stat(&node.main_stat, formats.node_main)),
                    (SharedString::Borrowed("Secondary stat"), stat(&node.secondary_stat, formats.node_secondary)),
                ];
                rows.extend(node.arcs.iter().zip(arcs).map(|(value, section)| {
                    (section.name.clone(), format!("{}%", crate::format::trimmed(value * 100.)).into())
                }));
                (node.title.clone(), node.sub_title.clone(), rows, &node.details)
            }
            Target::Edge(edge) => {
                let edge = graph.edges.get(edge)?;
                let rows = vec![
                    (SharedString::Borrowed("Main stat"), stat(&edge.main_stat, formats.edge_main)),
                    (SharedString::Borrowed("Secondary stat"), stat(&edge.secondary_stat, formats.edge_secondary)),
                ];
                (format!("{} → {}", edge.source, edge.target).into(), SharedString::default(), rows, &edge.details)
            }
            Target::Marker(_) => return None,
        };
    rows.retain(|(_, value)| !value.is_empty());
    rows.extend(details.iter().cloned());
    Some((at, area, title, sub, rows))
}
