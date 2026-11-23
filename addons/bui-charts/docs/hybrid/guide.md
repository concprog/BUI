# Hybrid charts: choosing a chart

How to pick between the charts in the [API reference](api.md). Every link goes to runnable examples; the gallery has them all (`cargo run --example gallery`, then **Charts**).

## Which chart

| Your question | Use |
| --- | --- |
| How much goes from each of these to each of those, and how much of each row is left after every step? | [`<SankeyMatrix>`](examples.md#sankeymatrix) |
| Which nodes belong to which group, how big is each, and who is linked to whom? | [`<TreemapForce>`](examples.md#treemapforce) |

## Sankey matrix or Sankey?

A [`<Sankey>`](../flow/guide.md) draws the flows as ribbons and gets tangled when many nodes connect to many others. A matrix puts every pair in its own cell, so nothing crosses and every value can be read, but it loses the flowing picture. `<SankeyMatrix>` keeps both: read exact values from the squares, and see each row's total thin out along its band. Use it for 5 to 20 rows and columns. Beyond that, use a heatmap.

## Reading the matrix

- Square area is the value, so a square twice as wide is four times the value.
- The band behind a row starts as wide as the row's total. After each cell it is narrower by that cell's value, so a band that is still wide late in the row means most of its flow goes to the later columns.
- Order matters: rows and columns appear in first-seen order. Sort your data first if you want the biggest first.

## Treemap force or node graph?

The [node graph](../node-graph/api.md) places nodes where their links want them, which is best for exploring structure. `<TreemapForce>` forces the groups into separate cells whose area is meaningful, so it answers "how big is each group" and "who talks to whom" in one picture. Use it when nodes have an obvious group, such as team, module or region, and a value to size them by.

## Tuning the layout

- Too crowded: give the chart more room or lower `distance`.
- Clusters look stiff: the circles are clamped inside their cell, so a link that pulls two circles against a border will leave them touching it. That is expected.
- Long names: turn `show_labels` off and read names from the tooltip.
- Many nodes: a few hundred are fine. The layout is recomputed on resize, theme change and when a group is hidden, not on hover.
