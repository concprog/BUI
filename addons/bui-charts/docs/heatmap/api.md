# Heatmap: API reference

`<Heatmap>` shows how values are spread over two dimensions, usually how many samples fell into each time × value bucket. It's a port of Grafana's Heatmap panel; see [Coming from Grafana](#coming-from-grafana).

```rust
use blenderui::prelude::*;
use bui_charts::*;
```

- [`<Heatmap>`](#heatmap)
- [Where the cells come from](#where-the-cells-come-from)
- [Colour](#colour)
- [Cells](#cells)
- [Axes and time](#axes-and-time)
- [Tooltip and legend](#tooltip-and-legend)
- [Zooming and selection](#zooming-and-selection)
- [Enums and types](#enums-and-types)
- [Coming from Grafana](#coming-from-grafana)

Not sure which option fits? See the [guide](guide.md).

---

## `<Heatmap>`

| Prop | Type | Default |
| --- | --- | --- |
| `data` | `Vec<T>`, `Signal<Vec<T>>`, `Frame<N>`, `Signal<Frame<N>>`, or your own [`Dataset`](../xy-chart/api.md#your-own-data-source) | **required** |
| `x`, `y`, `value`, `x_end`, `y_end` | `Fn(&T) -> f64` | see [below](#where-the-cells-come-from) |
| `calculate` | [`Calculation`](#where-the-cells-come-from) | none |
| `x_field` | field name, for a `Frame` | the first field |
| `bucket_layout` | [`BucketLayout`](#enums-and-types) | `Auto` |
| `bucket_scale` | [`ScaleKind`](../xy-chart/api.md#enums) | none: bucket rows are evenly spaced |
| `color` | [`HeatmapColor`](#colour) | the `Oranges` scheme, 64 steps |
| `color_min` / `color_max` | `f64` | the smallest / largest shown value |
| `cell_gap` | `f32` px, `0`–`25` | `1.` |
| `cell_radius` | `f32` px | `0.` |
| `show_value` | [`VisibilityMode`](#enums-and-types) | `Auto` |
| `cell_format` | `Fn(f64) -> SharedString` | none |
| `cell_decimals` | `usize` | up to 3, trailing zeros dropped |
| `hide_le` / `hide_ge` | `f64` | `1e-9` / none: empty buckets are hidden |
| `x_axis` / `y_axis` | [`AxisConfig`](../xy-chart/api.md#axisconfig) | numeric |
| `tooltip` | [`HeatmapTooltip`](#tooltip-and-legend) | `HeatmapTooltip::single()` |
| `legend` | [`HeatmapLegend`](#tooltip-and-legend) | a colour bar at the bottom |
| `exemplars` | `Vec<(f64, f64)>` | none |
| `exemplar_color` | `Hsla` / `Rgba` | translucent magenta |
| `selection` | [`SelectionMode`](#zooming-and-selection) | `X` |
| `on_select` | `Fn(SelectedRange, &mut Window, &mut App)` | none |
| `class`, `key` | | |

## Where the cells come from

| You have | Pass | Cells |
| --- | --- | --- |
| raw samples, such as one row per request | `x`, `y` and `calculate` | the samples are counted into buckets |
| one row per cell, already counted | `x`, `y` and `value` (plus `x_end` / `y_end` if cells have different sizes) | one cell per row, starting at `x` / `y` |
| bucket fields, such as a Prometheus histogram | a `Frame`: x first, then one field per bucket, named by its bound | one row of cells per field |

**`Calculation`** follows Grafana's "Calculate from data":

```rust
Calculation { x_buckets: Buckets::Size(60_000.), y_buckets: Buckets::Count(20), y_scale: BucketScale::Linear }
```

| Field | Values |
| --- | --- |
| `x_buckets` / `y_buckets` | `Buckets::Auto` (about 20 columns and 10 rows), `Buckets::Count(n)` (about n, rounded to a tidy size), `Buckets::Size(s)` (exactly s wide) |
| `y_scale` | `BucketScale::Linear`, `Log2` or `Log10`. With a log scale, `Buckets::Size(n)` splits each power into n buckets |

With a time x axis, column sizes are rounded to whole seconds, minutes, hours and so on.

**Bucket fields.** By default each field becomes one evenly spaced row, labelled with the field name:
- `BucketLayout::Le`: names are upper bounds, as Prometheus `le` buckets are; the label sits on the row's top edge.
- `Ge`: names are lower bounds.
- `Auto`: labels sit in the middle of each row.

Set `bucket_scale` to place the rows by the numbers in their names instead, on that scale. `+Inf` is drawn as one more bucket.

## Colour

| `HeatmapColor` | Cells are coloured |
| --- | --- |
| `HeatmapColor::scheme(HeatmapScheme::…)` | along one of Grafana's 38 colour schemes |
| `HeatmapColor::theme(Scheme::…)` | along a scheme from your [theme](../theming.md) |
| `HeatmapColor::opacity(color)` | in one colour, more opaque for higher values |

| Method | Effect | Default |
| --- | --- | --- |
| `.steps(n)` | how many distinct colours, `2`–`128` | `64` |
| `.reverse(bool)` | flips a scheme | `false` |
| `.exponent(e)` | opacity mode: `0.1`–`2.0`; below 1 makes low values stand out more | linear |

The scheme colours are exactly Grafana's (d3-scale-chromatic). As in Grafana, the diverging and cyclical schemes are flipped. Single-hue schemes (Blues, Oranges, …) are flipped in dark mode, so high values are bright on a dark background. Multi-hue schemes (Viridis, Magma, Turbo, …) are flipped in light mode.

`color_min` / `color_max` fix the ends of the colour range. Values beyond them use the end colours.

## Cells

- `cell_gap` leaves space between cells; it's ignored on cells too small to fit it.
- `show_value`: `Auto` writes the value in cells wide enough for it, `Always` writes it everywhere, `Never` writes none.
- `cell_format` or `cell_decimals` set the value text, used in cells, the tooltip and the legend.
- `hide_le` hides cells at or below a value; the default `1e-9` hides empty buckets. Pass `f64::NEG_INFINITY` to show every cell. `hide_ge` hides cells at or above a value.
- `exemplars` draws individual points over the cells, such as single slow requests.

## Axes and time

Both axes take an [`AxisConfig`](../xy-chart/api.md#axisconfig). Three methods matter most here:

| Method | Effect |
| --- | --- |
| `AxisConfig::new().time()` | values are milliseconds since the Unix epoch. Ticks fall on round times and read `HH:mm:ss`, `HH:mm`, `MM/DD HH:mm`, `MM/DD`, `YYYY-MM` or `YYYY` depending on the span. Shown in UTC |
| `.utc_offset(minutes)` | shows times in another time zone, such as `120` for UTC+2 |
| `.reverse(true)` | turns the axis upside down |

The x axis spans exactly the first to the last bucket.

## Tooltip and legend

```rust
HeatmapTooltip::single().y_histogram(true).color_scale(true)
```

| Method | Shows |
| --- | --- |
| `HeatmapTooltip::single()` | the bucket's x range, y range and value |
| `HeatmapTooltip::hidden()` | no tooltip |
| `.y_histogram(true)` | the whole column as a small bar chart, the hovered bucket highlighted |
| `.color_scale(true)` | the colour scale with the value marked on it |
| `.max_width(px)` / `.max_height(px)` | size limits |

| `HeatmapLegend` | |
| --- | --- |
| `HeatmapLegend::shown()` / `hidden()` | the colour bar with the lowest and highest value, marking the hovered value |
| `.right()` | beside the chart instead of below |
| `.width(px)` | its width when on the right |

## Zooming and selection

Drag across the cells to zoom in, and double-click to zoom back out.

| `SelectionMode` | Dragging zooms |
| --- | --- |
| `X` | the x axis |
| `Y` | the y axis |
| `XY` | both, using the box you draw |

`on_select` receives the range too: a `SelectedRange { x: (from, to), y: Option<(from, to)> }`. Use it to move your app's time range to the selection.

## Enums and types

| Type | Values |
| --- | --- |
| `HeatmapScheme` | Diverging: `BrBG`, `PiYG`, `PRGn`, `PuOr`, `RdBu`, `RdGy`, `RdYlBu`, `RdYlGn`, `Spectral`. Single hue: `Blues`, `Greens`, `Greys`, `Oranges` (default), `Purples`, `Reds`. Multi-hue: `Turbo`, `Cividis`, `Viridis`, `Magma`, `Inferno`, `Plasma`, `Warm`, `Cool`, `Cubehelix`, `BuGn`, `BuPu`, `GnBu`, `OrRd`, `PuBuGn`, `PuBu`, `PuRd`, `RdPu`, `YlGnBu`, `YlGn`, `YlOrBr`, `YlOrRd`. Cyclical: `Rainbow`, `Sinebow`. `HeatmapScheme::ALL` lists them |
| `Buckets` | `Auto`, `Count(usize)`, `Size(f64)` |
| `BucketScale` | `Linear`, `Log2`, `Log10` |
| `BucketLayout` | `Auto`, `Le`, `Ge` |
| `VisibilityMode` | `Auto`, `Always`, `Never` |
| `SelectionMode` | `X`, `Y`, `XY` |

## Coming from Grafana

| Grafana option | Here |
| --- | --- |
| Calculate from data, X / Y buckets, Y bucket scale | `calculate` |
| Heatmap rows: value field, layout, Y bucket scale | a `Frame`, `bucket_layout`, `bucket_scale` |
| Colour mode, scheme, fill, scale, exponent, steps, reverse, start / end colour from value | `color`, `color_min`, `color_max` |
| Cell gap, Show values, Hide cells with values ≤ / ≥, Cell value unit and decimals | `cell_gap`, `show_value`, `hide_le` / `hide_ge`, `cell_format` / `cell_decimals` |
| Y axis placement, label, width, min, max, reverse, unit, decimals | `y_axis` |
| Tooltip mode, Show histogram (Y axis), Show colour scale | `tooltip` |
| Legend | `legend` |
| Exemplars colour | `exemplars`, `exemplar_color` |
| Selection mode | `selection`, `on_select` |

Not available here: units from Grafana's unit registry (use `cell_format` and `AxisConfig::format`), data links, and annotations.
