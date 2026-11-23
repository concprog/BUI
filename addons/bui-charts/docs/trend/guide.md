# Trend: choosing options

How to pick between the options in the [API reference](api.md). Every link goes to a runnable example; the gallery has them all (`cargo run --example gallery`, then **Trend**).

## Trend, Time Series or XY Chart?

| Your x | Use |
| --- | --- |
| always goes up: speed, distance, load, depth, size | `<Trend>`: lines join in x order, and the tooltip can show every series at one x |
| any order, or several y values per x: measurements, samples | [`<XYChart>`](../xy-chart/guide.md) |
| time | [`<TimeSeries>`](../time-series/guide.md): the same chart with a time axis, timezones and time compare |

Examples: [Basic trend](examples.md#1-basic-trend), [Unsorted x](examples.md#18-unsorted-x).

## Getting data in

- **Your own rows:** pass `x` on `<Trend>` and one `<TrendSeries y=… />` per line.
- **A `Frame`:** pass nothing else. The first field is x and every other field becomes a series; use `x_field` if x isn't the first field. Add `<TrendSeries>` children only when you want to style or pick series.
- Keep the data in a signal (`use_signal`), so the chart only does its work again when the data changes.

Examples: [From a frame](examples.md#2-from-a-frame), [Live data](examples.md#17-live-data).

## Lines, bars or points

| You want to show | Use |
| --- | --- |
| a continuous measurement | `Line` with `Linear`, or `Smooth` for presentation |
| a value that holds until it changes: a setting, a gear, a tier | `Line` with `StepAfter` |
| amounts per bucket | `Bars`, with `bar_width_factor` near 0.8 |
| raw samples without implying anything in between | `Points` |

Examples: [Smooth with fill](examples.md#3-smooth-with-fill), [Steps](examples.md#4-steps), [Bars](examples.md#5-bars), [Points](examples.md#6-points).

## Parts of a whole

- **Totals and how they're made up:** `Stacking::NORMAL` with a fill.
- **Shares only:** `Stacking::PERCENT`; the totals are hidden and the axis reads 0–100%.

Stack series with the same unit only. Use different `Stacking::Normal("group")` names to keep two stacks apart in one chart.

Examples: [Stacked](examples.md#7-stacked), [Percent stacking](examples.md#8-percent-stacking).

## Different units

When series have different units, such as Nm and hp, give the second series its own `y_axis` with a label. It goes on the right automatically. `AxisColorMode::Series` ties each axis to its line by colour.

Example: [Two axes](examples.md#9-two-axes).

## Limits and colour by value

| You want | Use |
| --- | --- |
| reference lines at limits | `thresholds` with `ThresholdsStyle::Line` or `Dashed` |
| shaded good / warning / bad zones | `ThresholdsStyle::Area` or `DashedAndArea` |
| the line itself to change colour as it crosses limits | `gradient={GradientMode::Scheme}` with `thresholds` |
| a smooth colour ramp by value | `gradient={GradientMode::Scheme}` with a `scheme` |
| a softer, decorative fill | `GradientMode::Opacity` or `Hue` |

Examples: [Thresholds](examples.md#10-thresholds), [Colour by thresholds](examples.md#11-colour-by-thresholds), [Hue gradient](examples.md#12-hue-gradient).

## Ranges and bands

To show a range around a line, such as min/max or a confidence band:
1. Add the upper series with `fill_below_to` naming the lower one, and `line_width={0.}`.
2. Add the lower series with `line_width={0.}`.
3. Draw the main line on top.

Example: [Min / max band](examples.md#14-min--max-band).

## Missing data

| Your data | Use |
| --- | --- |
| missing values are real outages | the default: the line breaks |
| missing values are just skipped samples | `span_nulls={NullSpan::Always}` |
| short gaps are noise but long ones matter | `NullSpan::Within(gap)` |
| rows are simply absent during a gap | `insert_nulls={gap}` |

Example: [Missing values](examples.md#13-missing-values).

## Reading values

- Use `Tooltip::multi()` when series are compared at the same x, and `sort` to put the biggest first.
- Use a table legend with calcs (min, mean, max…) for a summary that's always visible.
- `show_values` labels every point; keep it for sparse data.

Examples: [Legend table and values](examples.md#16-legend-table-and-values), [Negative Y and constant](examples.md#15-negative-y-and-constant).
