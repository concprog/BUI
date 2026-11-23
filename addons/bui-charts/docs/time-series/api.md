# Time Series: API reference

`<TimeSeries>` plots values over time: metrics, latency percentiles, request rates. It is a port of Grafana's Time series panel. It shares everything with [`<Trend>`](../trend/api.md), which in Grafana is built on Time series too, and adds the time-specific options: a time axis by default, several timezones and time compare.

```rust
use blenderui::prelude::*;
use bui_charts::*;
```

- [`<TimeSeries>`](#timeseries)
- [Time axis and timezones](#time-axis-and-timezones)
- [Time compare](#time-compare)
- [Coming from Grafana](#coming-from-grafana)

Not sure which option fits? See the [guide](guide.md).

---

## `<TimeSeries>`

```rust
<TimeSeries data={metrics} x={|m: &Metric| m.time} annotations={deploys}>
    <TrendSeries name="p99" y={|m: &Metric| m.p99} />
    <TrendSeries name="p50" y={|m: &Metric| m.p50} line_style={LineStyle::DASH} />
</TimeSeries>
```

Every prop of [`<Trend>`](../trend/api.md#trend) works the same here. Series are [`<TrendSeries>`](../trend/api.md#trendseries) tags, with the same [style props](../trend/api.md#style-props), [axes](../trend/api.md#axes), [stacking](../trend/api.md#stacking), [colours and thresholds](../trend/api.md#colour-gradients-and-thresholds), [missing values](../trend/api.md#missing-values), [annotations, exemplars and zoom](../trend/api.md#annotations-exemplars-and-zoom) and [tooltip, legend and keyboard](../trend/api.md#tooltip-legend-and-keyboard).

These props differ or are new:

| Prop | Type | Default |
| --- | --- | --- |
| `x` | `Fn(&T) -> f64`: milliseconds since the Unix epoch | for a `Frame`: the `x_field`, or the first field |
| `x_axis` | [`AxisConfig`](../xy-chart/api.md#axisconfig) | `AxisConfig::new().time()`, in UTC |
| `timezones` | `&'static [i32]`: UTC offsets in minutes | `[0]` |
| `time_compare` | `f64`: a shift in milliseconds | none |

## Time axis and timezones

The x axis is a time axis even when you pass your own `x_axis`. Tick labels pick a round step (seconds up to years) and show only what changes, as in Grafana.

`timezones` adds one row of tick labels per timezone, under the first, all at the same ticks. The first entry is the main axis and is also used in the tooltip:

```rust
timezones={&[0, 330, -300]}
```

shows UTC, UTC+5:30 and UTC−5. With `orientation={VizOrientation::Vertical}`, each timezone gets a column of labels instead.

## Time compare

`time_compare={24. * HOUR}` draws every series a second time, shifted forward by that amount, so yesterday's values sit under today's. The copies:

- are dashed, unfilled and never stacked, in the colour of their series;
- are named `"<name> (1d ago)"`, and show in the legend and tooltip like any series;
- take each value from the row exactly one shift earlier, within half the smallest gap between rows. Rows without a match are left as gaps.

Your data must cover the earlier period too: to compare the last 24 hours with the 24 before, pass 48 hours of rows.

## Coming from Grafana

Everything in [Trend's table](../trend/api.md#coming-from-grafana), plus:

| Grafana option | Here |
| --- | --- |
| Time zone, additional time zones | `timezones` |
| Time shift / compare | `time_compare` |
| Orientation | `orientation` |
| Annotations, Exemplars | `annotations`, `exemplars` |
| Disable keyboard events | `keyboard={false}` |

Not available here: the legend's faceted label filter (series here have no labels), dashboard time pickers and annotation editing.
