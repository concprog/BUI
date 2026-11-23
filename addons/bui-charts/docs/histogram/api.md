# Histogram: API reference

`<Histogram>` counts how many values fall into each bucket and draws the counts as bars: how request times, sizes or scores are spread. It is a port of Grafana's Histogram panel; see [Coming from Grafana](#coming-from-grafana).

```rust
use blenderui::prelude::*;
use bui_charts::*;
```

- [`<Histogram>`](#histogram)
- [`<HistogramSeries>`](#histogramseries)
- [Buckets](#buckets)
- [Pre-counted buckets](#pre-counted-buckets)
- [Coming from Grafana](#coming-from-grafana)

Not sure which option fits? See the [guide](guide.md).

---

## `<Histogram>`

```rust
<Histogram data={requests}>
    <HistogramSeries name="GET" value={|r: &Request| r.get_ms} />
    <HistogramSeries name="POST" value={|r: &Request| r.post_ms} />
</Histogram>
```

| Prop | Type | Default |
| --- | --- | --- |
| `data` | `Vec<T>`, `Signal<Vec<T>>`, `Frame<N>`, `Signal<Frame<N>>`, or your own [`Dataset`](../xy-chart/api.md#your-own-data-source) | **required** |
| children | `<HistogramSeries>` tags | for a `Frame`: every field except `xMin` / `xMax` |
| `bucket_count` | `usize`: roughly how many buckets to aim for | `30` |
| `bucket_size` | `f64`: the width of every bucket | picked from `bucket_count`, see [Buckets](#buckets) |
| `bucket_offset` | `f64`: shifts where buckets start | `0.` |
| `combine` | `bool`: add every series into one `Count` series | `false` |
| `x_min` / `x_max` | `Fn(&T) -> f64`: each row's bucket edges, for [pre-counted buckets](#pre-counted-buckets) | none |
| `fill_opacity` | `f32`, `0.0`–`1.0` | `0.8` |
| `gradient` | [`GradientMode`](../trend/api.md#enums): `None`, `Opacity`, `Hue`, `Scheme` | `None` |
| `line_width` | `f32` px | `1.` |
| `stacking` | [`Stacking`](../trend/api.md#stacking): `None`, `Stacking::NORMAL`, `Stacking::PERCENT` | `None` |
| `hide_from` | [`HideFrom`](../xy-chart/api.md#enums) | nothing hidden |
| `thresholds`, `scheme` | colours for `gradient={GradientMode::Scheme}` | your theme's |
| `x_axis` | [`AxisConfig`](../xy-chart/api.md#axisconfig) | decimals from the bucket size |
| `y_axis` | `AxisConfig` | starts at 0 |
| `legend` | [`Legend`](../xy-chart/api.md#legend) | `Legend::list()` at the bottom |
| `tooltip` | [`Tooltip`](../trend/api.md#tooltip-legend-and-keyboard) | `Tooltip::single()` |
| `class`, `key` | | |

## `<HistogramSeries>`

```rust
<HistogramSeries name="GET" value={|r: &Request| r.get_ms} color={rgb(0x1e66f5)} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `value` | `Fn(&T) -> f64`: the value to count, or the count itself for pre-counted buckets | **required** |
| `name` | text | `"Series 1"`, `"Series 2"`, … |
| `color` | any colour | your theme's palette |
| `fill_opacity`, `line_width`, `gradient`, `hide_from` | as on the chart | the chart's |

Missing values (`NaN` or infinite) are not counted.

## Buckets

- **Size.** With `bucket_size`, every bucket is that wide. Without it, the chart divides the range of all values by `bucket_count` and picks the next size up from a list of round sizes (1, 2, 2.5, 4 and 5 times a power of ten). It never picks a size smaller than the smallest gap between two values.
- **Edges.** A value `v` falls in the bucket starting at `floor((v - offset) / size) * size + offset`. Buckets include their start and exclude their end.
- **Series.** Every series is counted into the same buckets so their bars line up. Buckets with no values are left out.
- **Hover.** The tooltip shows the bucket, such as `120 - 125`, and the count of each series.

## Pre-counted buckets

If your data is already bucketed, give each row's edges with `x_min` and `x_max`; each series' `value` is then the count:

```rust
<Histogram data={buckets} x_min={|b: &Bucket| b.low} x_max={|b: &Bucket| b.high}>
    <HistogramSeries name="requests" value={|b: &Bucket| b.count} />
</Histogram>
```

Buckets may have different widths. If only `x_min` or only `x_max` is given, the other edge is guessed from the gap between the first two rows. A `Frame` with fields named `xMin` and `xMax` is read the same way, as in Grafana.

## Coming from Grafana

| Grafana option | Here |
| --- | --- |
| Bucket count, Bucket size, Bucket offset | `bucket_count`, `bucket_size`, `bucket_offset` |
| Combine series | `combine` |
| Fill opacity, Gradient mode, Line width | `fill_opacity` (0.0–1.0), `gradient`, `line_width` |
| Stack series | `stacking` |
| Legend, Tooltip | `legend`, `tooltip` |
| Hide in area | `hide_from` |
| `xMin` / `xMax` frames | `x_min` / `x_max`, or a `Frame` with those field names |

Not available here: Prometheus `le` histograms (use [`<Heatmap>`](../heatmap/api.md) with `BucketLayout::Le`), units and log-scale bucket axes.
