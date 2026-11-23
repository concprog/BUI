# Distribution charts: API reference

Charts that show how values are spread: `<Barcode>`, `<Beeswarm>`, `<Ridgeline>` and `<Horizon>`. Group them with an accessor that returns the group name; groups keep the order they first appear in.

```rust
use blenderui::prelude::*;
use bui_charts::*;
```

- [`<Barcode>`](#barcode)
- [`<Beeswarm>`](#beeswarm)
- [`<Ridgeline>`](#ridgeline)
- [`<Horizon>`](#horizon)

Not sure which one fits? See the [guide](guide.md). For counts per bucket, use [`<Histogram>`](../histogram/api.md).

---

## `<Barcode>`

```rust
<Barcode data={requests} value={|r: &Request| r.latency} group={|r: &Request| r.region} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `data` | `Vec<T>`, `Signal<Vec<T>>`, or your own [`Dataset`](../xy-chart/api.md#your-own-data-source) | **required** |
| `value` | `Fn(&T) -> f64` | **required** |
| `group` | `Fn(&T) -> &str`: one row of ticks per group | one row |
| `tick_width` | `f32`, pixels | `1.5` |
| `color` | one colour for every group | a series colour per group |
| `x_axis` | [`AxisConfig`](../xy-chart/api.md#axisconfig) | `AxisConfig::new()` |
| `tooltip`, `class` | as on [`<Trend>`](../trend/api.md) | |

A thin tick per value. Dense areas look darker.

## `<Beeswarm>`

```rust
<Beeswarm data={people} value={|p: &Person| p.salary} group={|p: &Person| p.team} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `data` | as above | **required** |
| `value` | `Fn(&T) -> f64` | **required** |
| `group` | `Fn(&T) -> &str`: one lane and colour per group | one lane |
| `point_size` | `f32`, pixels | `5.` |
| `x_axis` | `AxisConfig` | `AxisConfig::new()` |
| `legend`, `tooltip`, `class` | as on [`<XYChart>`](../xy-chart/api.md) | |

Every value is a dot at its exact x. Dots that would overlap are pushed up or down within their lane, so the lane's outline is the shape of the distribution.

## `<Ridgeline>`

```rust
<Ridgeline data={readings} value={|r: &Reading| r.temperature} group={|r: &Reading| r.month} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `data` | as above | **required** |
| `value` | `Fn(&T) -> f64` | **required** |
| `group` | `Fn(&T) -> &str`: one curve per group | **required** |
| `bandwidth` | `f64`: kernel width, in units of `value` | Silverman's rule, per group |
| `overlap` | `f64`: the tallest curve's height, in rows | `1.6` |
| `fill_opacity` | `f32` | `0.75` |
| `x_axis` | `AxisConfig` | `AxisConfig::new()` |
| `tooltip`, `class` | | |

Each curve is a Gaussian kernel density estimate. The first group is at the top. Lower `bandwidth` shows more detail; higher smooths more.

## `<Horizon>`

```rust
<Horizon data={ticks} x={|t: &Tick| t.time} x_axis={AxisConfig::new().time()}>
    <TrendSeries name="CPU Δ" y={|t: &Tick| t.cpu} />
    <TrendSeries name="Latency Δ" y={|t: &Tick| t.latency} />
</Horizon>
```

| Prop | Type | Default |
| --- | --- | --- |
| `data` | as above | **required** |
| `x` | `Fn(&T) -> f64` | **required** |
| children | [`<TrendSeries>`](../trend/api.md#trendseries): one horizon row each; `name` is the row label | **required** |
| `bands` | `usize`: shades per sign | `3` |
| `positive`, `negative` | colours | good and bad |
| `x_axis` | `AxisConfig` | `AxisConfig::new()` |
| `class` | Tailwind classes (`"h-64"`) | |

Each row is cut into `bands` equal slices of its largest absolute value. The slices are stacked on top of each other, darker for bigger values, and negative values are flipped up in the `negative` colour. A row takes a fraction of the height a normal line chart would need. The slice size is shown next to each row's name.
