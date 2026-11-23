# Trend: API reference

`<Trend>` draws one or more series against a numeric x that only goes up: engine speed, distance, load, depth. It's Grafana's Time series graph with a numeric x instead of time, and takes all of its styling options. See [Coming from Grafana](#coming-from-grafana).

```rust
use blenderui::prelude::*;
use bui_charts::*;
```

- [`<Trend>`](#trend)
- [`<TrendSeries>`](#trendseries)
- [Style props](#style-props)
- [Axes](#axes)
- [Stacking](#stacking)
- [Colour, gradients and thresholds](#colour-gradients-and-thresholds)
- [Missing values](#missing-values)
- [Annotations, exemplars and zoom](#annotations-exemplars-and-zoom)
- [Tooltip, legend and keyboard](#tooltip-legend-and-keyboard)
- [Enums](#enums)
- [Coming from Grafana](#coming-from-grafana)

Not sure which option fits? See the [guide](guide.md).

---

## `<Trend>`

| Prop | Type | Default |
| --- | --- | --- |
| `data` | `Vec<T>`, `Signal<Vec<T>>`, `Frame<N>`, `Signal<Frame<N>>`, or your own [`Dataset`](../xy-chart/api.md#your-own-data-source) | **required** |
| `x` | `Fn(&T) -> f64` | for a `Frame`: the `x_field`, or the first field |
| `x_field` | field name, for a `Frame` | the first field |
| children | `<TrendSeries>` tags | for a `Frame`: one series per field other than x |
| `x_axis` | [`AxisConfig`](../xy-chart/api.md#axisconfig) | `AxisConfig::new()` |
| `y_axis` | [`AxisConfig`](../xy-chart/api.md#axisconfig) | `AxisConfig::new()` |
| `legend` | [`Legend`](../xy-chart/api.md#legend) | `Legend::list()` at the bottom |
| `tooltip` | [`Tooltip`](#tooltip-legend-and-keyboard) | `Tooltip::single()` |
| `thresholds` | [`Thresholds`](../bar-gauge/api.md#thresholds) | your theme's good colour, then its bad colour from 80 |
| `scheme` | [`Scheme`](../xy-chart/api.md#color-and-scheme) | none |
| `annotations` | [`Annotations`](#annotations-exemplars-and-zoom): a `Vec<Annotation>`, `Signal<Vec<Annotation>>` or `&'static [Annotation]` | none |
| `exemplars` | `Vec<(f64, f64)>`: (x, y) points drawn as diamonds | none |
| `exemplar_color` | colour | magenta |
| `orientation` | [`VizOrientation`](#enums): `Auto`, `Horizontal`, `Vertical` | `Auto` (horizontal) |
| `on_select` | `Fn(SelectedRange, &mut Window, &mut App)`, after a drag-zoom | none |
| `keyboard` | `bool`: arrow-key stepping | `true` |
| `class` | Tailwind classes | fills its parent |
| `key` | anything `Hash` (taken by `rsx!`: hook state follows it) | – |
| [style props](#style-props) | | defaults for every series |

**x must be in ascending order.** Rows with a missing x (`f64::NAN`) are skipped. If x goes down anywhere, the chart shows "Values must be in ascending order" instead of a graph, as Grafana does.

With no data, or a `Vec<T>` with no `<TrendSeries>`, the chart shows "No data".

## `<TrendSeries>`

| Prop | Type | Default |
| --- | --- | --- |
| `y` | `Fn(&T) -> f64` | **required** |
| `name` | text | `"Series 1"`, `"Series 2"`, … |
| `color` | `Hsla` / `Rgba` | the next colour from the theme palette |
| `y_axis` | [`AxisConfig`](#axes) | the chart's `y_axis` |
| `thresholds` | [`Thresholds`](../bar-gauge/api.md#thresholds) | the chart's `thresholds` |
| `scheme` | [`Scheme`](../xy-chart/api.md#color-and-scheme) | the chart's `scheme` |
| [style props](#style-props) | | inherited from `<Trend>` |

Return `f64::NAN` from `y` for a missing value.

```rust
<Trend data={pulls} x={|p: &Pull| p.rpm}>
    <TrendSeries name="Torque" y={|p: &Pull| p.torque} />
    <TrendSeries name="Power" y={|p: &Pull| p.power} y_axis={AxisConfig::new().label("hp")} />
</Trend>
```

## Style props

`<Trend>` and `<TrendSeries>` both take these. A value on the series wins over the chart's value, and the chart's value wins over the default.

| Prop | Type | Default | Notes |
| --- | --- | --- | --- |
| `draw_style` | [`DrawStyle`](#enums) | `Line` | lines, bars, or points only |
| `interpolation` | [`Interpolation`](#enums) | `Linear` | lines only |
| `line_width` | `f32` px | `1.` | `0.` draws no line |
| `line_style` | [`LineStyle`](../xy-chart/api.md#enums) | `Solid` | lines only |
| `fill_opacity` | `f32`, `0.0`–`1.0` | `0.` | the area under a line, or the inside of a bar |
| `gradient` | [`GradientMode`](#colour-gradients-and-thresholds) | `None` | |
| `fill_below_to` | another series' name | none | fills the band between this series and that one |
| `show_points` | [`PointsMode`](#enums) | `Auto` | |
| `point_size` | `f32` px | `5.` | never smaller than `line_width` |
| `show_values` | `bool` | `false` | the value above each shown point or bar |
| `span_nulls` | [`NullSpan`](#missing-values) | `Never` | |
| `insert_nulls` | `f64`, in x units | none | break the line where x jumps further than this |
| `bar_alignment` | [`BarAlignment`](#enums) | `Center` | |
| `bar_width_factor` | `f32`, `0.1`–`1.0` | `0.6` | share of the space between two x values |
| `bar_max_width` | `f32` px | `200.` | |
| `stacking` | [`Stacking`](#stacking) | `None` | |
| `transform` | [`ValueTransform`](#enums) | `None` | |
| `thresholds_style` | [`ThresholdsStyle`](#colour-gradients-and-thresholds) | `Off` | |
| `hide_from` | [`HideFrom`](../xy-chart/api.md#enums) | nothing hidden | hide a series from the legend, the tooltip, or the chart |

**Points with `Auto`** are shown when there's room, about one point every two point-widths, and hidden on dense data. Points that stand alone between missing values are always shown, so they don't vanish. Bars show no points with `Auto`.

## Axes

The x axis spans exactly the smallest to the largest x. The y axes round out to tidy tick values, like the [XY Chart's](../xy-chart/api.md#axisconfig). Every `AxisConfig` method works here: label, placement, soft and hard limits, log and symlog scales, grid, border, centred zero, colour mode, decimals and format.

**More than one y axis.** Give a series its own `y_axis` to put it on a second scale.
- Series whose `y_axis` has the same `label` share one axis.
- With `AxisPlacement::Auto`, the first axis goes on the left and the next ones on the right.
- `AxisColorMode::Series` colours an axis's ticks like its first series.
- The grid follows the first axis.

## Stacking

| `Stacking` | Effect |
| --- | --- |
| `Stacking::None` | each series on its own |
| `Stacking::Normal(group)`, or `Stacking::NORMAL` for group `"A"` | series in the same group are added on top of each other |
| `Stacking::Percent(group)`, or `Stacking::PERCENT` | each series is shown as its share of the group's total; the axis reads 0% to 100% |

Positive and negative values stack separately, so a negative series grows down from zero. A stacked series with a `fill_opacity` fills down to the series below it. The tooltip and legend show the real values, not the stacked totals. Series hidden from the legend drop out of the stack.

## Colour, gradients and thresholds

| `GradientMode` | Line | Fill |
| --- | --- | --- |
| `None` | the series colour | the series colour at `fill_opacity` |
| `Opacity` | the series colour | fades from `fill_opacity` at the top of the chart to transparent at the bottom |
| `Hue` | shifts in hue from top to bottom | the same, at `fill_opacity` |
| `Scheme` | changes colour with the value | the same, at `fill_opacity` |

`GradientMode::Scheme` follows the series' `scheme` if it has one, blending between its colours from the lowest to the highest value. Otherwise it follows its `thresholds`, switching colour at each step.

| `ThresholdsStyle` | What's drawn |
| --- | --- |
| `Off` | nothing |
| `Line` / `Dashed` | a line at each threshold step |
| `Area` | each band between steps shaded in its colour |
| `LineAndArea` / `DashedAndArea` | both |
| `Series` | the series itself coloured by threshold, like `GradientMode::Scheme` with thresholds |

Threshold lines and areas sit behind the series, once per y axis. Percentage thresholds are a share of the axis's `min`–`max`, or of the data when those aren't set.

## Missing values

A missing value is `f64::NAN` in `y`.

| `NullSpan` | The line at a missing value |
| --- | --- |
| `Never` | breaks |
| `Always` | joins the values either side |
| `Within(gap)` | joins them when they're no more than `gap` apart in x, otherwise breaks |

`insert_nulls` breaks the line wherever two values are further apart in x than its value, even if neither is missing. Use it for data that has gaps without explicit missing rows.

## Annotations, exemplars and zoom

```rust
let notes = vec![
    Annotation::at(deployed, "Deploy v1.4.2").text("Rolled out to all pods"),
    Annotation::region(start, end, "Incident #218").color(rgb(0xd20f39)),
];
<Trend data={rows} x={|r: &Row| r.x} annotations={notes} exemplars={slow} on_select={zoomed}>
```

- **Annotations** mark a point (a dashed line) or a region (a shaded band) with a marker along the bottom edge. Hover the marker for its title, time and text. `color` overrides the default cyan.
- **Exemplars** are individual samples, such as one slow request among the percentiles, drawn as diamonds on the first y axis. Hover one for its x and y.
- **Zoom.** Drag across the chart to zoom the x axis; double-click to zoom out. `on_select` receives the selected range.
- **Orientation.** `VizOrientation::Vertical` turns the chart so x runs down the left and values run left to right. Everything else (fills, gradients, thresholds, tooltips, zoom) works the same.

## Tooltip, legend and keyboard

```rust
Tooltip::multi().sort(SortOrder::Descending).hide_zeros(true).max_width(px(240.))
```

| Method | Effect |
| --- | --- |
| `Tooltip::single()` | the series nearest the pointer at the hovered x |
| `Tooltip::multi()` | every series at the hovered x |
| `Tooltip::hidden()` | no tooltip; the hover line still shows |
| `sort` | [`SortOrder`](#enums) of the rows in a multi tooltip |
| `hide_zeros` | leave out series whose value is 0 |
| `max_width` / `max_height` | `Pixels` |

The tooltip title is the x value. A vertical line marks the hovered x and highlights each series' point there.

The [legend](../xy-chart/api.md#legend) works like the XY Chart's: click a series to show only it, Ctrl-click to toggle it. Calcs use the real values.

**Keyboard.** Click the chart, then press ← / → (or ↑ / ↓) to move the hover by one x value, Shift + arrow to move by ten, and Esc to clear it. `keyboard={false}` turns this off.

## Enums

| Enum | Variants |
| --- | --- |
| `DrawStyle` | `Line` (default), `Bars`, `Points` |
| `Interpolation` | `Linear` (default), `Smooth`, `StepBefore`, `StepAfter` |
| `GradientMode` | `None` (default), `Opacity`, `Hue`, `Scheme` |
| `PointsMode` | `Auto` (default), `Always`, `Never` |
| `NullSpan` | `Never` (default), `Always`, `Within(f64)` |
| `BarAlignment` | `Before`, `Center` (default), `After` |
| `Stacking` | `None` (default), `Normal(&str)`, `Percent(&str)`, plus `NORMAL` and `PERCENT` for group `"A"` |
| `ValueTransform` | `None` (default), `Constant` (a flat line at the first value), `NegativeY` (drawn below zero) |
| `ThresholdsStyle` | `Off` (default), `Line`, `Dashed`, `Area`, `LineAndArea`, `DashedAndArea`, `Series` |
| `TooltipMode` | `Single` (default), `Multi`, `Hidden` |
| `SortOrder` | `None` (default), `Ascending`, `Descending` |
| `VizOrientation` | `Auto` (default, horizontal), `Horizontal`, `Vertical` |

`Smooth` curves pass through every point without overshooting it. `StepAfter` holds each value until the next x; `StepBefore` jumps to the next value straight away.

## Coming from Grafana

| Grafana option | Here |
| --- | --- |
| X field | `x`, or `x_field` for a `Frame` |
| Style, Line interpolation, Line width, Line style | `draw_style`, `interpolation`, `line_width`, `line_style` |
| Fill opacity (0–100) | `fill_opacity` (0.0–1.0) |
| Gradient mode | `gradient` |
| Fill below to | `fill_below_to` |
| Show points, Point size, Show values | `show_points`, `point_size`, `show_values` |
| Connect null values, Disconnect values | `span_nulls`, `insert_nulls` |
| Bar alignment, Bar width factor, Bar max width | `bar_alignment`, `bar_width_factor`, `bar_max_width` |
| Stack series | `stacking` |
| Transform | `transform` |
| Axis placement, label, width, soft min/max, scale, centred zero, colour, border, grid | `y_axis` on the chart or a series |
| Min, Max, Decimals, Unit | `AxisConfig::min`, `max`, `decimals`, `format` |
| Thresholds, Show thresholds | `thresholds`, `thresholds_style` |
| Color scheme | `color`, or `gradient` with `scheme` / `thresholds` |
| Hide in area | `hide_from` |
| Tooltip mode, sort, hide zeros, max width/height | `tooltip` |
| Legend | `legend` |
| Annotations, Exemplars | `annotations`, `exemplars` |
| Orientation | `orientation` |
| Disable keyboard events | `keyboard={false}` |

Not available here: units from Grafana's unit registry (use `AxisConfig::format`), value mappings, data links and overrides by field regex (set the prop on each `<TrendSeries>` instead).
