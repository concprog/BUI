use blenderui::prelude::*;
use bui_charts::*;


fn noise(seed: u64) -> f64 {
    let mut z = seed.wrapping_add(0x9e37_79b9_7f4a_7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    (z ^ (z >> 31)) as f64 / u64::MAX as f64
}

const SERVICES: [(&str, &str); 8] = [
    ("frontend", "web"),
    ("gateway", "edge"),
    ("auth", "service"),
    ("orders", "service"),
    ("payments", "service"),
    ("inventory", "service"),
    ("postgres", "database"),
    ("redis", "cache"),
];

const CALLS: [(usize, usize); 10] = [
    (0, 1),
    (1, 2),
    (1, 3),
    (3, 4),
    (3, 5),
    (2, 7),
    (3, 6),
    (4, 6),
    (5, 6),
    (5, 7),
];

fn services(seed: u64) -> Vec<GraphNode> {
    SERVICES
        .iter()
        .enumerate()
        .map(|(index, (name, kind))| {
            let errors = (noise(seed * 100 + index as u64) * 0.3).powi(2) * 3.;
            let rate = (40. + noise(seed * 200 + index as u64) * 400.).round();
            GraphNode::new(*name)
                .sub_title(*kind)
                .main_stat(format!("{rate} req/s"))
                .secondary_stat(format!(
                    "{:.0} ms",
                    5. + noise(seed * 300 + index as u64) * 90.
                ))
                .arcs([1. - errors, errors])
                .detail("Kind", *kind)
                .detail("Pods", format!("{}", 1 + index % 4))
        })
        .collect()
}

fn calls(seed: u64) -> Vec<GraphEdge> {
    CALLS
        .iter()
        .enumerate()
        .map(|(index, (from, to))| {
            GraphEdge::new(SERVICES[*from].0, SERVICES[*to].0)
                .main_stat(format!(
                    "{:.0} ms",
                    2. + noise(seed * 50 + index as u64) * 40.
                ))
                .secondary_stat(format!(
                    "{:.0} req/s",
                    10. + noise(seed * 70 + index as u64) * 200.
                ))
        })
        .collect()
}

fn health() -> Vec<ArcSection> {
    vec![
        ArcSection::new("success", rgb(0x40a02b)),
        ArcSection::new("errors", rgb(0xd20f39)),
    ]
}

fn tree(count: usize) -> (Vec<GraphNode>, Vec<GraphEdge>) {
    let nodes = (0..count)
        .map(|index| {
            GraphNode::new(format!("n{index}"))
                .main_stat(index as f64)
                .arcs([noise(index as u64 + 7)])
        })
        .collect();
    let edges = (1..count)
        .map(|index| GraphEdge::new(format!("n{}", (index - 1) / 3), format!("n{index}")))
        .collect();
    (nodes, edges)
}

fn pipelines(lanes: usize, length: usize) -> (Vec<GraphNode>, Vec<GraphEdge>) {
    let mut nodes = vec![GraphNode::new("ingest").main_stat("source")];
    let mut edges = Vec::new();
    for lane in 0..lanes {
        let mut previous = SharedString::Borrowed("ingest");
        for step in 0..length {
            let id: SharedString = format!("stage {lane}.{step}").into();
            nodes.push(
                GraphNode::new(id.clone())
                    .main_stat(format!("step {step}"))
                    .arcs([noise((lane * 100 + step) as u64)]),
            );
            edges.push(GraphEdge::new(previous, id.clone()));
            previous = id;
        }
    }
    (nodes, edges)
}

#[component]
fn service_map() {
    let nodes = use_signal(services(1));
    let edges = use_signal(calls(1));
    rsx! { <NodeGraph nodes={nodes} edges={edges} arcs={health()} class="h-96" /> }
}

#[component]
fn force() {
    let nodes = use_signal(services(2));
    let edges = use_signal(calls(2));
    rsx! { <NodeGraph nodes={nodes} edges={edges} arcs={health()} layout={NodeLayout::Force} class="h-96" /> }
}

#[component]
fn grid() {
    let nodes = use_signal(services(3));
    let edges = use_signal(calls(3));
    rsx! { <NodeGraph nodes={nodes} edges={edges} arcs={health()} layout={NodeLayout::Grid} class="h-96" /> }
}

#[component]
fn colors() {
    let palette = [
        0x1e66f5, 0x40a02b, 0xdf8e1d, 0x8839ef, 0xd20f39, 0x04a5e5, 0xea76cb, 0x179299,
    ];
    let nodes = use_signal(
        services(4)
            .into_iter()
            .zip(palette)
            .map(|(node, color)| node.arcs([]).color(rgb(color)))
            .collect::<Vec<_>>(),
    );
    let edges = use_signal(calls(4));
    rsx! { <NodeGraph nodes={nodes} edges={edges} class="h-96" /> }
}

#[component]
fn highlighted() {
    let nodes = use_signal(
        services(5)
            .into_iter()
            .map(|node| {
                let hot = node.id().as_ref() == "payments";
                node.highlighted(hot)
            })
            .collect::<Vec<_>>(),
    );
    let edges = use_signal(
        calls(5)
            .into_iter()
            .map(|edge| {
                let hot = edge.target().as_ref() == "payments";
                edge.highlighted(hot)
            })
            .collect::<Vec<_>>(),
    );
    rsx! { <NodeGraph nodes={nodes} edges={edges} arcs={health()} class="h-96" /> }
}

#[component]
fn edge_styles() {
    let nodes = use_signal(services(6));
    let edges = use_signal(
        calls(6)
            .into_iter()
            .enumerate()
            .map(|(index, edge)| {
                let edge = edge.thickness(1. + (index % 4) as f32);
                if index % 3 == 0 {
                    edge.dashed(true).color(rgb(0x8839ef))
                } else {
                    edge
                }
            })
            .collect::<Vec<_>>(),
    );
    rsx! { <NodeGraph nodes={nodes} edges={edges} arcs={health()} class="h-96" /> }
}

#[component]
fn sizes() {
    let nodes = use_signal(
        services(7)
            .into_iter()
            .enumerate()
            .map(|(index, node)| node.radius(25. + (index % 4) as f32 * 10.))
            .collect::<Vec<_>>(),
    );
    let edges = use_signal(calls(7));
    rsx! { <NodeGraph nodes={nodes} edges={edges} arcs={health()} class="h-96" /> }
}

#[component]
fn node_limit() {
    let (nodes, edges) = pipelines(6, 10);
    let nodes = use_signal(nodes);
    let edges = use_signal(edges);
    rsx! {
        <NodeGraph
            nodes={nodes}
            edges={edges}
            node_limit={19}
            arcs={vec![ArcSection::new("load", rgb(0x1e66f5))]}
            class="h-96"
        />
    }
}

#[component]
fn callbacks() {
    let nodes = use_signal(services(8));
    let edges = use_signal(calls(8));
    let picked = use_signal(SharedString::Borrowed("Click a node or an edge."));
    rsx! {
        <div class="flex flex-col gap-2 h-96">
            <div class="text-xs">{picked.get()}</div>
            <NodeGraph
                nodes={nodes}
                edges={edges}
                arcs={health()}
                on_node_click={move |node: &GraphNode, _: &mut Window, _: &mut App| {
                    picked.set(SharedString::from(format!("Node {}", node.id())));
                }}
                on_edge_click={move |edge: &GraphEdge, _: &mut Window, _: &mut App| {
                    picked.set(SharedString::from(format!("Edge {} → {}", edge.source(), edge.target())));
                }}
                class="flex-1"
            />
        </div>
    }
}

#[component]
fn numbers() {
    fn rate(value: f64) -> SharedString {
        format!("{value:.0} req/s").into()
    }
    fn latency(value: f64) -> SharedString {
        format!("{value:.1} ms").into()
    }
    let nodes = use_signal(
        services(9)
            .into_iter()
            .enumerate()
            .map(|(index, node)| {
                node.main_stat(50. + noise(index as u64) * 500.)
                    .secondary_stat(noise(index as u64 + 30) * 80.)
            })
            .collect::<Vec<_>>(),
    );
    let edges = use_signal(
        calls(9)
            .into_iter()
            .enumerate()
            .map(|(index, edge)| {
                edge.main_stat(noise(index as u64 + 60) * 20.)
                    .secondary_stat(SharedString::default())
            })
            .collect::<Vec<_>>(),
    );
    rsx! {
        <NodeGraph
            nodes={nodes}
            edges={edges}
            arcs={health()}
            formats={StatFormats::new().node_main(rate).node_secondary(latency).edge_main(latency)}
            class="h-96"
        />
    }
}

#[component]
fn cycles() {
    let nodes = use_signal(
        ["a", "b", "c", "d", "e"]
            .map(|id| GraphNode::new(id).main_stat(id.to_uppercase()))
            .to_vec(),
    );
    let edges = use_signal(vec![
        GraphEdge::new("a", "b"),
        GraphEdge::new("b", "c"),
        GraphEdge::new("c", "a"),
        GraphEdge::new("c", "d"),
        GraphEdge::new("d", "e"),
        GraphEdge::new("e", "c"),
    ]);
    rsx! { <NodeGraph nodes={nodes} edges={edges} class="h-96" /> }
}

#[component]
fn greedy() {
    let (nodes, edges) = tree(30);
    let nodes = use_signal(nodes);
    let edges = use_signal(edges);
    rsx! {
        <NodeGraph
            nodes={nodes}
            edges={edges}
            zoom_mode={ZoomMode::Greedy}
            legend={false}
            arcs={vec![ArcSection::new("load", rgb(0xdf8e1d))]}
            class="h-96"
        />
    }
}

#[component]
fn live() {
    let round = use_signal(1u64);
    let nodes = use_signal(services(1));
    let edges = use_signal(calls(1));
    rsx! {
        <div class="flex flex-col gap-2 h-96">
            <div
                id="refresh"
                class="cursor-pointer text-sm underline"
                onClick={move || {
                    let next = round.get() + 1;
                    round.set(next);
                    nodes.set(services(next));
                    edges.set(calls(next));
                }}
            >
                "Refresh the stats"
            </div>
            <NodeGraph nodes={nodes} edges={edges} arcs={health()} class="flex-1" />
        </div>
    }
}

examples! {
    service_map: "Service map" => "Services left to right in layers, with success and error arcs. Hover a node or an edge.",
    force: "Force layout" => "The same services placed by a force simulation.",
    grid: "Grid layout" => "Nodes in a grid. Click an arc in the legend to sort by it.",
    colors: "Node colours" => "No arcs: each node's outline in its own colour.",
    highlighted: "Highlighted" => "A highlighted node and the edges into it.",
    edge_styles: "Edge styles" => "Thicker, coloured and dashed edges.",
    sizes: "Node sizes" => "A radius per node.",
    node_limit: "Node limit" => "61 nodes shown 19 at a time; click a marker to explore further.",
    callbacks: "Click callbacks" => "on_node_click and on_edge_click instead of the details popover.",
    numbers: "Number formats" => "Numeric stats formatted with StatFormats.",
    cycles: "Cycles" => "The layered layout handles loops.",
    greedy: "Greedy zoom" => "The wheel zooms without holding Ctrl; no legend.",
    live: "Live data" => "Signals as data: the stats update in place.",}
