# Flow, tree and profile charts: choosing a chart

How to pick between the charts in the [API reference](api.md). Every link goes to runnable examples; the gallery has them all (`cargo run --example gallery`, then **Charts**).

## Which chart

| Your data | Use |
| --- | --- |
| tasks with a start and an end | [`<Gantt>`](examples.md#gantt) |
| amounts moving from one thing to another: energy, money, users | [`<Sankey>`](examples.md#sankey) |
| items that pass through the same stages, by category: class → outcome | [`<Alluvial>`](examples.md#alluvial) |
| a funnel fed by several channels, with drop-off at each step | [`<SankeyFunnel>`](examples.md#sankeyfunnel) |
| rows to group by similarity | [`<Dendrogram>`](examples.md#dendrogram) |
| a tree with sizes: a profile, a disk, a budget | [`<Icicle>`](examples.md#icicle) |
| the same few measures for several things | [`<Spider>`](examples.md#spider) |
| a measure per month or direction, stacked by cause | [`<NightingaleRose>`](examples.md#nightingalerose) |
| many rows across several measures, to spot patterns and outliers | [`<ParallelCoordinates>`](examples.md#parallelcoordinates) |
| states over time, many per row | [`<StateTimeline>`](../state-timeline/guide.md) |
| services and calls between them | [`<NodeGraph>`](../node-graph/guide.md) |

## Gantt

One task per row, in your order. Use `group` to colour by team or phase; the legend then lists the groups. Overlapping tasks are fine, since each has its own row.

## Sankey

- Each row is one link. Nodes are created from the names, so there is no separate node list.
- Links must flow one way: a loop has no left-to-right order.
- Raise `node_padding` when labels collide; lower `link_opacity` when many bands cross.

## Icicle or Flame Graph?

`<Icicle>` is the flame graph pane on its own with the root at the top: the shape people expect for disk usage or budgets. Use [`<FlameGraph>`](../flame-graph/guide.md) when you need the table, the sandwich view or a diff.

## Spider

- Keep to 3–10 spokes and no more than about 5 shapes; beyond that, use a grouped bar chart.
- Every spoke shares one scale. Put measures on the same scale (such as 0–100 scores) first, and set `max` so shapes are comparable between charts.
- Click a legend entry to hide a shape or isolate one.

## Sankey or Alluvial?

Use `<Sankey>` when you already have links (from → to, amount) and nodes can link anywhere. Use `<Alluvial>` when you have one row per item with a category at each stage: it builds the links between neighbouring stages for you and keeps each band in its starting colour, so you can follow a group across.

## Dendrogram

- `features` returns the numbers to compare. Put them on similar scales first (for example 0–10 or z-scores), or the largest one decides everything.
- `Average` linkage suits most data. `Ward` gives compact, even clusters. `Complete` keeps clusters tight. `Single` follows chains and finds elongated groups.
- `clusters` colours the tree as if cut into that many groups. Links above the cut stay neutral.
- The work grows with the cube of the row count. A few hundred rows is fine.

## Nightingale Rose

Read wedges by area, not radius: a wedge with twice the value is about 1.4 times as long. Stacking puts the most important series first (innermost). Use it for cyclic categories such as months, hours or compass directions. For non-cyclic categories, a bar chart is easier to read.

## Parallel Coordinates

- Each axis has its own range, from its minimum to its maximum, so compare shapes rather than heights between axes.
- The order of `axes` matters: patterns show best between neighbouring axes, so put related measures next to each other.
- With many rows, lower `line_opacity` and use `group` to colour by class. Hover a line to read it.
