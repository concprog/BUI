# Hybrid charts: API reference

Two charts that combine a familiar layout with a second one: `<SankeyMatrix>` (a matrix with Sankey bands) and `<TreemapForce>` (a treemap with a force-directed graph in each cell). They use the same painter as the [diagram charts](../diagram/api.md), so they behave the same way:

- hovering a shape fades the others and shows a tooltip;
- clicking a legend entry hides it;
- the layout is recomputed only when the data, the size, the hidden set or the theme changes.

```rust
use blenderui::prelude::*;
use bui_charts::*;
```

- [`<SankeyMatrix>`](#sankeymatrix)
- [`<TreemapForce>`](#treemapforce)

Not sure which one fits? See the [guide](guide.md). `data` below takes `Vec<T>`, `Signal<Vec<T>>`, or your own [`Dataset`](../xy-chart/api.md#your-own-data-source). Every chart also takes `tooltip` ([`Tooltip`](../xy-chart/api.md#tooltip)), `legend` ([`Legend`](../xy-chart/api.md#legend)) and `class`.

---

## `<SankeyMatrix>`

```rust
<SankeyMatrix data={flows} source={|f: &Flow| f.from} target={|f: &Flow| f.to} value={|f: &Flow| f.amount} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `data` | one row per flow | **required** |
| `source` | `Fn(&T) -> &str`: the row of the matrix | **required** |
| `target` | `Fn(&T) -> &str`: the column of the matrix | **required** |
| `value` | `Fn(&T) -> f64`: rows with the same pair add up; zero, negative and missing values are skipped | **required** |
| `show_values` | `bool`: the value written inside squares that are big enough | `true` |
| `legend` | `Legend`: one entry per row | shown |

Rows and columns appear in the order they first show up in the data. Each cell is a square whose area is its value. Behind each row, a band as thick as the row's total runs left to right and thins by every cell's value, so what is left of the band is what has not yet been spent. The bars above the columns are column totals. The tooltip shows the value and its share of the row, the column and the grand total. Hiding a row in the legend re-scales everything.

## `<TreemapForce>`

```rust
<TreemapForce data={nodes} name={|n: &Node| n.name.as_ref()} group={|n: &Node| n.team} value={|n: &Node| n.weight} links={links} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `data` | one row per node | **required** |
| `name` | `Fn(&T) -> &str` | **required** |
| `group` | `Fn(&T) -> &str`: the treemap cell the node lives in, also its colour | **required** |
| `value` | `Fn(&T) -> f64`: the node's size | **required** |
| `links` | `Vec<ForceLink>`: `{ source, target, weight }`, where `source` and `target` are indexes into `data` | none |
| `iterations` | `usize`: force steps | `240` |
| `distance` | `f32`: preferred gap between linked circles, in pixels | `14.` |
| `show_labels` | `bool`: names written inside circles that are big enough | `true` |
| `legend` | `Legend`: one entry per group | shown |

Groups are laid out as a squarified treemap, so each cell's area follows its total value. Inside a cell, nodes are circles sized by their share of it. A force simulation pulls linked nodes together, keeps circles from overlapping and keeps each one inside its own cell. Links between cells are drawn across the borders, and the circles are pulled toward the cell they link to. Links with a missing index, or from a node to itself, are ignored. Hiding a group in the legend gives its space to the rest.
