# Gauge: API reference

`<Gauge>` shows a value as a bar along an arc or a ring, between a minimum and a maximum, with threshold markers, labels and a sparkline. It is a port of Grafana 12's Gauge panel; see [Coming from Grafana](#coming-from-grafana).

```rust
use blenderui::prelude::*;
use bui_charts::*;
```

- [`<Gauge>`](#gauge)
- [Shape and bar](#shape-and-bar)
- [Thresholds and colour](#thresholds-and-colour)
- [Effects](#effects)
- [Text and sparkline](#text-and-sparkline)
- [Several gauges](#several-gauges)
- [Enums and types](#enums-and-types)
- [Coming from Grafana](#coming-from-grafana)

Not sure which option fits? See the [guide](guide.md).

---

## `<Gauge>`

```rust
<Gauge data={samples} value={|s: &Sample| s.cpu} x={|s: &Sample| s.time} format={percent} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `data` | `Vec<T>`, `Signal<Vec<T>>`, `Frame<N>`, `Signal<Frame<N>>`, or your own [`Dataset`](../xy-chart/api.md#your-own-data-source) | **required** |
| `value`, `name`, `reduce`, `fields`, `x`, `x_field` | as on [`<Stat>`](../stat/api.md#which-values-are-shown) | the last value of every field |
| `shape` | [`GaugeShape`](#enums-and-types) | `Gauge`: a 220° arc |
| `bar_shape` | [`BarShape`](#enums-and-types) | `Flat` |
| `bar_width_factor` | `f32`: bar thickness relative to the radius | `0.5` |
| `segment_count` | `usize`: split the bar into this many segments | `1` |
| `segment_spacing` | `f32`, `0.0`–`1.0`: the gap between segments | `0.3` |
| `effects` | [`GaugeEffects`](#effects) | none |
| `endpoint_marker` | [`EndpointMarker`](#enums-and-types) | `Point` |
| `show_threshold_markers` | `bool`: a thin ring of threshold colours | `true` |
| `show_threshold_labels` | `bool`: threshold values around the arc | `false` |
| `neutral` | `f64`: the value the bar grows from | `min` |
| `sizing` | [`Sizing`](../bar-gauge/api.md#enums-and-types): `Auto` or `Manual { min_width, min_height, .. }` | `Auto` |
| `sparkline` | `bool` | `true` |
| `text_mode` | [`TextMode`](../bar-gauge/api.md#enums-and-types) | `Auto` |
| `orientation` | [`BarOrientation`](../bar-gauge/api.md#enums-and-types) | `Auto` |
| `title_size`, `value_size` | `Pixels` | fitted to the gauge |
| `min`, `max` | `f64` | `0.` and `100.` |
| `thresholds` | [`Thresholds`](../bar-gauge/api.md#thresholds) | your theme's good colour, then its bad colour from 80 |
| `color` | [`GaugeColor`](../bar-gauge/api.md#gaugecolor) | `Thresholds` |
| `mappings` | `&'static` [`ValueMappings`](../state-timeline/api.md#value-mappings) | none |
| `format` | `Fn(f64) -> SharedString` | none |
| `decimals` | `usize` | up to 3, trailing zeros dropped |
| `class`, `key` | | |

## Shape and bar

- **`GaugeShape::Gauge`** draws a 220° arc open at the bottom; **`Circle`** a full ring starting at the top. Both start from `min` and run clockwise.
- The bar fills the arc from `min` to the value; the rest is a grey track. With `neutral`, the bar runs from the neutral value to the value in either direction, for balances, deltas and temperatures around zero.
- `bar_width_factor` sets the thickness: the bar is `bar_width_factor × radius ÷ 3` wide.
- `BarShape::Rounded` gives the bar round ends. The `Point` endpoint marker, a small dot at each end, only shows on rounded bars.
- `segment_count` above 1 splits the arc into separate segments. Each lit segment takes the colour its own value would have, so thresholds show as bands. Segments are never drawn thinner than 3 px; the count is reduced to fit.

## Thresholds and colour

- The bar's colour comes from a matching mapping, otherwise from `color`, as in [Bar Gauge](../bar-gauge/api.md#gaugecolor).
- `show_threshold_markers` draws a thin ring outside the bar in each threshold step's colour, so you can see how close the value is to the next step. It is hidden when `min` equals `max`.
- `show_threshold_labels` writes `min`, each threshold and `max` around the arc, formatted with `format` / `decimals`. Percentage thresholds are written as percentages. With `neutral`, its value is labelled too.

## Effects

`GaugeEffects` switches on Grafana's three visual effects:

```rust
effects={GaugeEffects::new().gradient().bar_glow()}
effects={GaugeEffects::all()}
```

| Effect | Look |
| --- | --- |
| `gradient` | the bar is shaded along the arc: through the threshold colours, along the scheme, or from a darker to a lighter tone of the value's colour |
| `bar_glow` | a soft halo around the bar |
| `center_glow` | a faint radial glow of the value's colour inside the gauge |

`EndpointMarker::Glow` adds a bright spot at the end of the bar, in dark themes.

## Text and sparkline

- The value is centred in the gauge and sized to fit. `text_mode` `Auto` adds the name when there are several gauges or a `name` accessor; `Name`, `Value`, `ValueAndName` and `None` choose directly. `value_size` and `title_size` fix the sizes.
- With `sparkline` and several rows, a small chart of the history sits under the value, in the value's colour.

## Several gauges

Every value gets its own gauge. `BarOrientation::Auto` packs them into the grid that keeps them closest to square; `Horizontal` stacks them as rows; `Vertical` places them side by side. `Sizing::Manual { min_width, min_height, .. }` stops them from shrinking below that size. With several gauges, all values use the size of the longest text, so they line up.

## Enums and types

| Type | Values |
| --- | --- |
| `GaugeShape` | `Gauge` (default), `Circle` |
| `BarShape` | `Flat` (default), `Rounded` |
| `EndpointMarker` | `Point` (default), `Glow`, `None` |
| `GaugeEffects` | `{ bar_glow, center_glow, gradient }`, built with `new()`, `all()`, `.bar_glow()`, `.center_glow()`, `.gradient()` |

## Coming from Grafana

| Grafana option | Here |
| --- | --- |
| Calculation, Fields, Show, Limit | `reduce`, `fields` |
| Style: shape, bar shape, bar width | `shape`, `bar_shape`, `bar_width_factor` |
| Segments, Segment spacing | `segment_count`, `segment_spacing` |
| Effects: gradient, bar glow, center glow | `effects` |
| Endpoint marker | `endpoint_marker` |
| Show threshold markers, Show threshold labels | `show_threshold_markers`, `show_threshold_labels` |
| Neutral | `neutral` |
| Gauge size, Min width, Min height | `sizing` |
| Sparkline, Text mode, Orientation | `sparkline`, `text_mode`, `orientation` |
| Text size | `title_size`, `value_size` |
| Min, Max, Thresholds, Color scheme, Value mappings | `min`, `max`, `thresholds`, `color`, `mappings` |
| Unit, Decimals | `format`, `decimals` |

Not available here: units from Grafana's unit registry and data links. Threshold labels are written horizontally rather than along the arc.
