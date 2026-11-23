# Flow, tree and profile charts: API reference

`<Gantt>`, `<Sankey>`, `<Alluvial>`, `<Icicle>`, `<Dendrogram>`, `<Spider>`, `<NightingaleRose>`, `<ParallelCoordinates>` and `<SankeyFunnel>`. Gantt draws through [`<StateTimeline>`](../state-timeline/api.md) and Icicle through [`<FlameGraph>`](../flame-graph/api.md); Sankey, Alluvial and Sankey Funnel share one painter; Spider, Nightingale Rose and Parallel Coordinates share another; Dendrogram draws through `<XYChart>`. All of them have the same hover, tooltip and theme as every other chart.

```rust
use blenderui::prelude::*;
use bui_charts::*;
```

- [`<Gantt>`](#gantt)
- [`<Sankey>`](#sankey)
- [`<Icicle>`](#icicle)
- [`<Spider>`](#spider)
- [`<Alluvial>`](#alluvial)
- [`<Dendrogram>`](#dendrogram)
- [`<NightingaleRose>`](#nightingalerose)
- [`<ParallelCoordinates>`](#parallelcoordinates)
- [`<SankeyFunnel>`](#sankeyfunnel)

Not sure which one fits? See the [guide](guide.md).

---

## `<Gantt>`

```rust
<Gantt data={tasks} name={|t: &Task| t.name} start={|t: &Task| t.start} end={|t: &Task| t.end} group={|t: &Task| t.team} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `data` | `Vec<T>`, `Signal<Vec<T>>`, or your own [`Dataset`](../xy-chart/api.md#your-own-data-source) | **required** |
| `name` | `Fn(&T) -> &str`: the row label | **required** |
| `start`, `end` | `Fn(&T) -> f64`, milliseconds since the epoch | **required** |
| `group` | `Fn(&T) -> &str`: the colour and legend entry | the task's `name` |
| `row_height` | `f32`: the share of a row the bar fills | `0.7` |
| `show_value` | [`VisibilityMode`](../state-timeline/api.md#enums-and-types): the label in each bar | as on `<StateTimeline>` |
| `x_axis` | [`AxisConfig`](../xy-chart/api.md#axisconfig) | a time axis |
| `legend` | [`Legend`](../xy-chart/api.md#legend) | lists the groups |
| `tooltip`, `class` | as on `<StateTimeline>` | |

One row per task, in your order.

Zoom: hold Ctrl (Cmd on macOS) and scroll, or pinch, to zoom the time axis about the pointer. Drag to select a range, and double-click to reset.

## `<Sankey>`

```rust
<Sankey data={flows} source={|f: &Flow| f.from} target={|f: &Flow| f.to} value={|f: &Flow| f.amount} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `data` | as above: one row per link | **required** |
| `source`, `target` | `Fn(&T) -> &str`: node names; nodes are created from them | **required** |
| `value` | `Fn(&T) -> f64` | **required** |
| `node_width` | `f32`, pixels | `12.` |
| `node_padding` | `f32`: the vertical space between nodes, pixels | `12.` |
| `link_opacity` | `f32` | `0.4` |
| `tooltip` | [`Tooltip`](../xy-chart/api.md#tooltip) | shown |
| `class` | Tailwind classes (`"h-64"`) | |

Nodes are placed in columns by their distance from a source and sized by the larger of what flows in and out. Hover a node or a link to see its amount. Links must not form a loop.

## `<Icicle>`

```rust
<Icicle data={profile} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `data` | any [`ProfileData`](../flame-graph/api.md#profile): the same data `<FlameGraph>` takes | **required** |
| `header` | `bool`: show search and reset | `false` |
| `class` | Tailwind classes (`"h-64"`) | |

The root is at the top and each child sits below its parent, sized by its value. Click a block to zoom into it. For the table, the sandwich view and diffs, use `<FlameGraph>`.

## `<Spider>`

```rust
<Spider data={players} axes={axes} name={|p: &Player| p.name} value={|p: &Player, axis: usize| p.stats[axis]} max={100.} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `data` | as above: one row per shape | **required** |
| `axes` | `Vec<SharedString>`: one spoke each, clockwise from the top | **required** |
| `name` | `Fn(&T) -> &str` | **required** |
| `value` | `Fn(&T, usize) -> f64`: the row's value on spoke `axis` | **required** |
| `max` | `f64`: the value at the rim | the largest value |
| `levels` | `usize`: grid rings | `5` |
| `grid` | `SpiderGrid::Polygon` or `SpiderGrid::Circle` | `Polygon` |
| `fill_opacity` | `f32` | `0.2` |
| `line_width` | `f32`, pixels | `2.` |
| `legend` | [`Legend`](../xy-chart/api.md#legend): click an entry to hide or isolate a shape | a list |
| `tooltip` | [`Tooltip`](../xy-chart/api.md#tooltip): every shape's value on the hovered spoke | shown |
| `class` | Tailwind classes (`"h-64"`) | |

All spokes share one scale from 0 to `max`. Values above it reach past the rim.

## `<Alluvial>`

```rust
<Alluvial data={passengers} value={|p: &Passenger| p.count}>
    <Stage name="Class" value={|p: &Passenger| p.class} />
    <Stage name="Sex" value={|p: &Passenger| p.sex} />
    <Stage name="Survived" value={|p: &Passenger| p.survived} />
</Alluvial>
```

| Prop | Type | Default |
| --- | --- | --- |
| `data` | as above: one row per item or group of items | **required** |
| children | `<Stage name value>`: one column each; `value` is `Fn(&T) -> &str` | **required** |
| `value` | `Fn(&T) -> f64`: how many items the row stands for | `1.` per row |
| `node_width`, `node_padding`, `link_opacity`, `tooltip`, `class` | as on `<Sankey>` | |

Built on the same layout and painter as `<Sankey>`. Each stage is a column of categories with its name on top. Bands keep the colour of the first stage's category all the way across, so you can follow a group through every stage. The same category name gets the same colour in every column.

## `<Dendrogram>`

```rust
<Dendrogram data={animals} name={|a: &Animal| a.name} features={|a: &Animal| a.traits.as_slice()} clusters={3} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `data` | as above: one row per leaf | **required** |
| `name` | `Fn(&T) -> &str` | **required** |
| `features` | `Fn(&T) -> &[f64]`: the values compared between rows | **required** |
| `linkage` | `Linkage::Single`, `Complete`, `Average` or `Ward` | `Average` |
| `clusters` | `usize`: cut the tree into this many coloured clusters | `1` |
| `horizontal` | `bool`: leaves down the side | `false` |
| `color` | colour, when `clusters` is 1 | the first series colour |
| `value_axis` | `AxisConfig` for the merge distance | from 0, labelled "Distance" |
| `class` | Tailwind classes (`"h-64"`) | |

Rows are clustered by Euclidean distance, joining the two closest clusters again and again (the Lance–Williams method). The height of each join is the distance between the two clusters. Leaves are ordered so branches never cross. Links above the cut are drawn in the neutral colour.

## `<NightingaleRose>`

```rust
<NightingaleRose data={causes} sectors={months} name={|c: &Cause| c.name} value={|c: &Cause, month: usize| c.deaths[month]} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `data` | as above: one row per stacked series | **required** |
| `sectors` | `Vec<SharedString>`: one wedge each, clockwise from the top | **required** |
| `name` | `Fn(&T) -> &str` | **required** |
| `value` | `Fn(&T, usize) -> f64`: the row's value in sector `index` | **required** |
| `max` | `f64`: the stacked total at the rim | the largest stacked total |
| `levels` | `usize`: grid rings | `4` |
| `fill_opacity` | `f32` | `0.85` |
| `legend`, `tooltip`, `class` | as on `<Spider>` | |

Rows are stacked outwards in each sector. Each wedge's **area** follows its value: the radius grows with the square root, as in Nightingale's original. It shares the polar painter, legend and tooltip with `<Spider>`.

## `<ParallelCoordinates>`

```rust
<ParallelCoordinates data={flowers} axes={axes} name={|f: &Flower| f.id.as_ref()} value={|f: &Flower, axis: usize| f.sizes[axis]} group={|f: &Flower| f.species} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `data` | as above: one row per line | **required** |
| `axes` | `Vec<SharedString>`: one vertical axis each | **required** |
| `name` | `Fn(&T) -> &str`: the line's label in the tooltip | **required** |
| `value` | `Fn(&T, usize) -> f64` | **required** |
| `group` | `Fn(&T) -> &str`: colour and legend entry | one colour per row |
| `line_opacity` | `f32` | `0.6` |
| `line_width` | `f32` | `1.5` |
| `legend` | `Legend` | shown with a `group`, hidden without |
| `tooltip`, `class` | | |

Each axis is scaled from its own minimum to its maximum, both written at its ends. Hover near a line to highlight it and read all its values. A missing value breaks the line. It takes the same `axes` / `value` data as `<Spider>`.

Zoom: hold Ctrl (Cmd on macOS) and scroll, or pinch, to zoom about the pointer. Drag a rectangle to zoom into it, Shift and drag to pan, and double-click to reset.

## `<SankeyFunnel>`

```rust
<SankeyFunnel data={channels} stages={stages} name={|c: &Channel| c.name} value={|c: &Channel, i: usize| c.counts[i]} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `data` | one row per entry channel | **required** |
| `stages` | `Vec<SharedString>`: the funnel steps, in order | **required** |
| `name` | `Fn(&T) -> &str`: the channel | **required** |
| `value` | `Fn(&T, usize) -> f64`: how many from this channel reached stage `index` | **required** |
| `source_label` | text above the channel column | `"Source"` |
| `drop_label` | text before "after <stage>" on the drop-off nodes | `"Dropped"` |
| `drops` | `bool`: show drop-off flows | `true` |
| `node_width`, `node_padding`, `link_opacity`, `tooltip`, `class` | as on `<Sankey>` | |

A funnel with several entries, built on the `<Sankey>` layout. Each channel's band keeps its colour through every stage, so you can see which channel converts. After each stage, the people who did not continue flow into a drop-off node placed right next to it. Counts above the previous stage are capped to it.
