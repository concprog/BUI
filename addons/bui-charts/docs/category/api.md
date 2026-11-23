# Category charts: API reference

Charts with one bar, dot or line per named item: `<Lollipop>`, `<Dumbbell>`, `<Slope>`, `<Bump>`, `<BackToBack>`, `<Waterfall>`, `<Bullet>`, `<Funnel>` and `<Forest>`. All but `<Forest>` are built on [`<Trend>`](../trend/api.md), so hover, tooltips, the legend and theming work the same way.

```rust
use blenderui::prelude::*;
use bui_charts::*;
```

- [Shared props](#shared-props)
- [`<Lollipop>`](#lollipop)
- [`<Dumbbell>`](#dumbbell)
- [`<Slope>`](#slope)
- [`<Bump>`](#bump)
- [`<BackToBack>`](#backtoback)
- [`<Waterfall>`](#waterfall)
- [`<Bullet>`](#bullet)
- [`<Funnel>`](#funnel)
- [`<Forest>`](#forest)

Not sure which one fits? See the [guide](guide.md). `<Pareto>` is with the [quality charts](../quality/api.md).

---

## Shared props

| Prop | Type | Default |
| --- | --- | --- |
| `data` | `Vec<T>`, `Signal<Vec<T>>`, or your own [`Dataset`](../xy-chart/api.md#your-own-data-source) | **required** |
| `name` | `Fn(&T) -> &str`: the item's label on the category axis | **required** |
| `horizontal` | `bool`: categories down the side, values along the bottom | per chart |
| `sort` | [`SortOrder`](../trend/api.md#tooltip-legend-and-keyboard): `None` keeps your order | `SortOrder::None` |
| `value_axis` | [`AxisConfig`](../xy-chart/api.md#axisconfig) for the value axis | `AxisConfig::new()` |
| `legend` | [`Legend`](../xy-chart/api.md#legend) | as on `<Trend>`, where the chart has more than one series |
| `tooltip` | [`Tooltip`](../trend/api.md#tooltip-legend-and-keyboard) | as on `<Trend>` |
| `class` | Tailwind classes (`"h-64"`) | |

The category axis is made for you: one tick per item, labelled with `name`. Labels that would overlap are skipped.

## `<Lollipop>`

```rust
<Lollipop data={sales} name={|s: &Sale| s.region} value={|s: &Sale| s.total} sort={SortOrder::Descending} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `value` | `Fn(&T) -> f64` | **required** |
| `color` | colour | the first series colour |
| `point_size` | `f32`, pixels | `12.` |
| `show_values` | `bool` | `false` |
| `horizontal` | | `true` |

A thin stem from zero to a dot at the value.

## `<Dumbbell>`

```rust
<Dumbbell data={scores} name={|s: &Score| s.team} start={|s: &Score| s.q1} end={|s: &Score| s.q4} start_label="Q1" end_label="Q4" />
```

| Prop | Type | Default |
| --- | --- | --- |
| `start`, `end` | `Fn(&T) -> f64` | **required** |
| `start_label`, `end_label` | text, shown in the legend and tooltip | `"Start"`, `"End"` |
| `start_color`, `end_color` | colours | the first and second series colours |
| `point_size` | `f32` | `12.` |
| `horizontal` | | `true` |

Two dots per item joined by a bar. `sort` orders by `end`.

## `<Slope>`

```rust
<Slope data={countries} name={|c: &Country| c.name} before={|c: &Country| c.before} after={|c: &Country| c.after} before_label="2010" after_label="2020" />
```

| Prop | Type | Default |
| --- | --- | --- |
| `before`, `after` | `Fn(&T) -> f64` | **required** |
| `before_label`, `after_label` | text under the two columns | `"Before"`, `"After"` |
| `color_by_change` | `bool`: rises in the good colour, falls in the bad colour | `true` |
| `show_values` | `bool` | `true` |
| `y_axis` | `AxisConfig` | `AxisConfig::new()` |

One line per item between two columns. With `color_by_change={false}` each item gets its own series colour.

## `<Bump>`

```rust
<Bump data={standings} x={|r: &Rank| r.year} name={|r: &Rank| r.team} value={|r: &Rank| r.points} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `x` | `Fn(&T) -> f64`: the round, year or step | **required** |
| `name` | `Fn(&T) -> &str`: which line the row belongs to | **required** |
| `value` | `Fn(&T) -> f64`: the score that is ranked | **required** |
| `highest_first` | `bool`: rank 1 is the highest value | `true` |
| `smooth` | `bool` | `true` |
| `x_axis` | `AxisConfig` | whole numbers |

Rows are long format: one row per item per step. At each step the items are ranked by `value`; the y axis shows rank 1 at the top.

## `<BackToBack>`

```rust
<BackToBack data={ages} name={|a: &Age| a.band} left={|a: &Age| a.men} right={|a: &Age| a.women} left_label="Men" right_label="Women" />
```

| Prop | Type | Default |
| --- | --- | --- |
| `left`, `right` | `Fn(&T) -> f64`, positive numbers | **required** |
| `left_label`, `right_label` | text | `"Left"`, `"Right"` |
| `left_color`, `right_color` | colours | the first and second series colours |

Horizontal bars that grow left and right from a shared centre: population pyramids, A/B comparisons. Axis labels show magnitudes on both sides.

## `<Waterfall>`

```rust
<Waterfall data={bridge} name={|s: &Step| s.name} value={|s: &Step| s.change} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `value` | `Fn(&T) -> f64`: the change at this step, + or − | **required** |
| `total` | `bool`: add a final bar with the running total | `true` |
| `total_label` | text | `"Total"` |
| `increase`, `decrease`, `total_color` | colours | good, bad and the first series colour |
| `show_values` | `bool` | `true` |
| `horizontal` | | `false` |

Each bar floats from the previous running total to the new one.

## `<Bullet>`

```rust
<Bullet data={kpis} name={|k: &Kpi| k.name} value={|k: &Kpi| k.value} target={|k: &Kpi| k.target} ranges={|k: &Kpi| k.bands} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `value` | `Fn(&T) -> f64` | **required** |
| `target` | `Fn(&T) -> f64`: drawn as a short cross line | none |
| `ranges` | `Fn(&T) -> [f64; 3]`: the upper ends of the poor, satisfactory and good bands, smallest first | none |
| `color` | colour of the value bar | the first series colour |
| `target_color` | colour | the bad colour |
| `value_axis` | `AxisConfig` | starts at 0 |
| `horizontal` | | `true` |

The bands are drawn as three shades of grey behind a narrower value bar.

## `<Funnel>`

```rust
<Funnel data={stages} name={|s: &Stage| s.name} value={|s: &Stage| s.count} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `value` | `Fn(&T) -> f64` | **required** |
| `color` | one colour for every stage | a series colour per stage |
| `show_values` | `bool` | `true` |
| `horizontal` | | `true` |

Bars centred on one line, in your order, so each stage's drop-off is visible.

## `<Forest>`

```rust
<Forest data={trials} name={|s: &Study| s.name} estimate={|s: &Study| s.effect}
    low={|s: &Study| s.low} high={|s: &Study| s.high} weight={|s: &Study| s.patients} null={1.} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `estimate` | `Fn(&T) -> f64`: the point estimate | **required** |
| `low`, `high` | `Fn(&T) -> f64`: the confidence interval | **required** |
| `weight` | `Fn(&T) -> f64`: sizes each square, such as the study size | equal squares |
| `null` | `f64`: the dashed line of no effect | `0.` |
| `color` | colour | the first series colour |
| `value_axis` | `AxisConfig`; pass `.scale(ScaleKind::Log { base: 10. })` for ratios | `AxisConfig::new()` |
| `tooltip`, `class` | as on [`<XYChart>`](../xy-chart/api.md) | |

One row per study, in your order from the top. Each row is a line from `low` to `high` with a square at `estimate`. Built on `<XYChart>`, so the squares can be hovered.
