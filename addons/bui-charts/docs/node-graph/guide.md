# Node Graph: choosing options

How to pick between the options in the [API reference](api.md). Every link goes to a runnable example; the gallery has them all (`cargo run --example gallery`, then **Node Graph**).

## Node Graph or something else?

| You want to see | Use |
| --- | --- |
| what talks to what, and how healthy each part is | `<NodeGraph>` |
| one request's path through the services, in time | [`<Traces>`](../traces/guide.md) |
| where the time goes inside one service | [`<FlameGraph>`](../flame-graph/guide.md) |

## Showing health

- Give every node `arcs` with the share of successful and failed requests, and pass the matching `arcs` sections: red slices stand out immediately.
- Put the request rate in `main_stat` and latency in `secondary_stat`, and per-edge latency on the edges.
- Mark a known problem with `highlighted` on the node and on the edges into it.

Examples: [Service map](examples.md#1-service-map), [Highlighted](examples.md#5-highlighted), [Number formats](examples.md#10-number-formats).

## Choosing a layout

| Your graph | Use |
| --- | --- |
| a flow with a direction: requests, pipelines, dependencies | `Layered` |
| a web of peers without a clear direction | `Force` |
| many nodes to compare, where connections matter less | `Grid`, sorted by an arc from the legend |

Examples: [Service map](examples.md#1-service-map), [Force layout](examples.md#2-force-layout), [Grid layout](examples.md#3-grid-layout), [Cycles](examples.md#11-cycles).

## Encoding more

| Goal | Use |
| --- | --- |
| node type or team | `color` per node, without arcs |
| traffic volume | `radius` per node, `thickness` per edge |
| async or optional calls | `dashed` edges |

Examples: [Node colours](examples.md#4-node-colours), [Node sizes](examples.md#7-node-sizes), [Edge styles](examples.md#6-edge-styles).

## Big graphs

Keep `node_limit` at a size that stays readable (the default is 200). Unexplored branches collapse into markers you can click to move the view.

Example: [Node limit](examples.md#8-node-limit).

## Linking to your app

Use `on_node_click` to open a service's page, logs or traces; the default popover lists `detail` rows. In dashboards that scroll, keep `ZoomMode::Cooperative` so the wheel scrolls the page unless Ctrl is held.

Examples: [Click callbacks](examples.md#9-click-callbacks), [Greedy zoom](examples.md#12-greedy-zoom).
