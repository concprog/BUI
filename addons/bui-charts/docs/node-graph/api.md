# Node Graph: API reference

`<NodeGraph>` draws a directed graph: services and the calls between them, pipeline stages, dependencies. Nodes are circles with two stats inside, a title below and coloured arc sections around them; edges are arrows that can carry stats of their own. It is a port of Grafana's Node graph panel; see [Coming from Grafana](#coming-from-grafana).

```rust
use blenderui::prelude::*;
use bui_charts::*;
```

- [`<NodeGraph>`](#nodegraph)
- [`GraphNode`](#graphnode)
- [`GraphEdge`](#graphedge)
- [Arcs and colours](#arcs-and-colours)
- [Layouts](#layouts)
- [Interaction](#interaction)
- [Enums and types](#enums-and-types)
- [Coming from Grafana](#coming-from-grafana)

Not sure which option fits? See the [guide](guide.md).

---

## `<NodeGraph>`

```rust
let nodes = vec![
    GraphNode::new("api").sub_title("gateway").main_stat("120 req/s").arcs([0.98, 0.02]),
    GraphNode::new("db").sub_title("postgres").main_stat("40 ms"),
];
let edges = vec![GraphEdge::new("api", "db").main_stat("12 ms")];
let health = vec![
    ArcSection::new("success", rgb(0x40a02b)),
    ArcSection::new("errors", rgb(0xd20f39)),
];

<NodeGraph nodes={nodes} edges={edges} arcs={health} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `nodes` | `Vec<GraphNode>`, `Signal<Vec<GraphNode>>` or your own `Dataset<GraphNode>` | **required** |
| `edges` | `Vec<GraphEdge>`, `Signal<Vec<GraphEdge>>` or your own `Dataset<GraphEdge>` | **required** |
| `layout` | [`NodeLayout`](#layouts) | `Layered` |
| `zoom_mode` | [`ZoomMode`](#interaction) | `Cooperative` |
| `arcs` | `Vec<ArcSection>`: the name and colour of each arc section | none |
| `node_limit` | `usize`: nodes shown at once | `200` |
| `formats` | [`StatFormats`](#enums-and-types): how numeric stats are written | numbers as they are |
| `legend` | `bool`: the arc legend below the graph | `true` |
| `on_node_click` | `Fn(&GraphNode, &mut Window, &mut App)` | a details popover |
| `on_edge_click` | `Fn(&GraphEdge, &mut Window, &mut App)` | a details popover |
| `class`, `key` | | |

Pass `Signal`s when the graph changes over time: layouts are computed once per change and cached.

## `GraphNode`

| Method | Effect | Default |
| --- | --- | --- |
| `GraphNode::new(id)` | a node with this unique id | title = id |
| `title(text)`, `sub_title(text)` | two lines under the circle | |
| `main_stat(value)`, `secondary_stat(value)` | two lines inside the circle: numbers, text, `bool` or a `StateValue` | empty |
| `arcs([..])` | one fraction from 0 to 1 per [`ArcSection`](#arcs-and-colours), in the same order | none |
| `color(colour)` | the outline colour when there are no arcs | grey |
| `radius(px)` | the circle's radius | `40.` |
| `highlighted(bool)` | fills the circle in red | `false` |
| `detail(name, value)` | an extra row in the details popover; call it once per row | none |

## `GraphEdge`

| Method | Effect | Default |
| --- | --- | --- |
| `GraphEdge::new(source, target)` | an arrow from the node with id `source` to the node with id `target` | |
| `id(text)` | a unique id | `"source→target"` |
| `main_stat(value)`, `secondary_stat(value)` | a label shown when you hover the edge or one of its nodes | empty |
| `thickness(px)` | line width | `1.` |
| `color(colour)` | line colour | grey |
| `dashed(bool)` | a dashed line | `false` |
| `highlighted(bool)` | red and one pixel thicker | `false` |
| `detail(name, value)` | an extra row in the details popover | none |

Edges whose source or target id is not among the nodes are left out.

## Arcs and colours

Each node's outline is drawn from its `arcs` values, in the order of the chart's `arcs` sections, clockwise from the top:

- A value of 1 or more paints the whole outline in that section's colour.
- Otherwise each non-zero value takes that share of the circle. Shares past 100% are cut off.
- A node without non-zero arcs gets its `color`, or grey.

The legend lists the sections. Clicking a section switches to the grid layout sorted by it; click again to reverse the order.

## Layouts

| `NodeLayout` | Placement |
| --- | --- |
| `Layered` (default) | left to right in layers, so every edge points right where it can; crossings are reduced; loops are allowed |
| `Force` | a physical simulation that pulls connected nodes together and pushes others apart (Grafana's d3-force settings) |
| `Grid` | rows of nodes in data order, or sorted by an arc from the legend |

The buttons in the top-right corner switch layouts at any time.

**Node limit.** With more nodes than `node_limit`, the graph shows the nodes nearest the roots (nodes with no incoming edges) and collapses each unexplored branch into a "N nodes" marker. Click a marker to centre the view on that branch. In the grid layout, the limit shows a window of the sorted nodes.

## Interaction

- **Pan** by dragging the background. **Zoom** with the + and − buttons, or the mouse wheel: with `ZoomMode::Cooperative`, hold Ctrl (⌘ on macOS) so the page can still scroll; with `Greedy`, the wheel always zooms. The ⛶ button resets the view and fits the graph.
- **Hover** a node to highlight it and its neighbours, dim the rest, show its full title and the stats of its edges. Hover an edge to show its stats.
- **Click** a node or edge for a popover with its title, stats, arc shares and `detail` rows, or handle the click yourself with `on_node_click` / `on_edge_click`.

The graph starts fitted to the panel, never zoomed in past 100%.

## Enums and types

| Type | Values |
| --- | --- |
| `NodeLayout` | `Layered` (default), `Force`, `Grid` |
| `ZoomMode` | `Cooperative` (default), `Greedy` |
| `ArcSection` | `ArcSection::new(name, colour)` |
| `StatFormats` | `StatFormats::new()` with `.node_main(f)`, `.node_secondary(f)`, `.edge_main(f)`, `.edge_secondary(f)`, each `fn(f64) -> SharedString` for numeric stats |

## Coming from Grafana

| Grafana | Here |
| --- | --- |
| Nodes frame: id, title, subtitle, mainstat, secondarystat | `GraphNode::new(id)`, `title`, `sub_title`, `main_stat`, `secondary_stat` |
| `arc__*` fields, arc colours | `arcs([..])` and the chart's `arcs` |
| color, nodeRadius, highlighted, `detail__*` | `color`, `radius`, `highlighted`, `detail` |
| Edges frame: id, source, target, mainstat, secondarystat | `GraphEdge::new(source, target)`, `id`, `main_stat`, `secondary_stat` |
| thickness, color, strokeDasharray, highlighted, `detail__*` | `thickness`, `color`, `dashed`, `highlighted`, `detail` |
| Layout algorithm | `layout` |
| Zoom mode | `zoom_mode` |
| Nodes / edges main and secondary stat unit | `formats` |
| Node limit | `node_limit` |
| Data links in the context menu | `on_node_click`, `on_edge_click` |

Differences:
- Grafana lays out every node and then applies the limit; here only the shown nodes are laid out, so a limited graph stays compact.
- The graph starts fitted to the panel instead of at 100%.
- Node icons and fixed node positions are not available.
