# Quality charts: API reference

Charts for finding causes and checking processes: `<Pareto>`, `<ControlChart>` (a Shewhart individuals chart) and `<BlandAltman>`. Each one computes its statistics from your rows, once per change.

```rust
use blenderui::prelude::*;
use bui_charts::*;
```

- [`<Pareto>`](#pareto)
- [`<ControlChart>`](#controlchart)
- [`<BlandAltman>`](#blandaltman)

Not sure which one fits? See the [guide](guide.md).

---

## `<Pareto>`

```rust
<Pareto data={defects} name={|d: &Defect| d.cause} value={|d: &Defect| d.count} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `data` | `Vec<T>`, `Signal<Vec<T>>`, or your own [`Dataset`](../xy-chart/api.md#your-own-data-source) | **required** |
| `name` | `Fn(&T) -> &str` | **required** |
| `value` | `Fn(&T) -> f64` | **required** |
| `threshold` | `f64`: the dashed cumulative line, in percent | `80.` |
| `color` | colour of the bars | the first series colour |
| `line_color` | colour of the cumulative line | the second series colour |
| `show_values` | `bool` | `false` |
| `y_axis` | [`AxisConfig`](../xy-chart/api.md#axisconfig) for the counts | `AxisConfig::new()` |
| `legend`, `tooltip`, `class` | as on [`<Trend>`](../trend/api.md) | |

Bars are sorted largest first. The cumulative share uses a second axis from 0% to 100% on the right.

## `<ControlChart>`

```rust
<ControlChart data={samples} x={|s: &Sample| s.time} value={|s: &Sample| s.diameter} zones={true} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `data` | as above | **required** |
| `x` | `Fn(&T) -> f64`: time, in milliseconds since the epoch | **required** |
| `value` | `Fn(&T) -> f64` | **required** |
| `center` | `f64`: a known process mean | the mean of the data |
| `sigma` | `f64`: a known standard deviation | the average moving range / 1.128 |
| `limit` | `f64`: the control limits, in σ | `3.` |
| `zones` | `bool`: also draw ±1σ and ±2σ | `false` |
| `color` | colour of the measurements | the first series colour |
| `x_axis` | `AxisConfig` | a time axis |
| `y_axis` | `AxisConfig` | `AxisConfig::new()` |
| `legend`, `tooltip`, `class` | | |

Points outside `center ± limit × sigma` are drawn in the bad colour. Estimating σ from the moving range follows the usual individuals (X-mR) chart, so a slow drift does not widen the limits.

## `<BlandAltman>`

```rust
<BlandAltman data={readings} a={|r: &Reading| r.cuff} b={|r: &Reading| r.arterial} a_label="Cuff" b_label="Arterial line" />
```

| Prop | Type | Default |
| --- | --- | --- |
| `data` | as above | **required** |
| `a`, `b` | `Fn(&T) -> f64`: the same quantity by two methods | **required** |
| `a_label`, `b_label` | text, used in the axis titles | `"A"`, `"B"` |
| `agreement` | `f64`: the width of the limits of agreement, in standard deviations | `1.96` (95%) |
| `color` | colour of the points | the first series colour |
| `legend`, `tooltip`, `class` | as on [`<XYChart>`](../xy-chart/api.md) | |

Each point is the mean of the two methods (x) against their difference, a − b (y). Lines mark the bias (the mean difference) and bias ± `agreement` × SD.

Zoom: drag a rectangle to zoom into it, Ctrl and scroll to zoom about the pointer, and double-click to reset.

