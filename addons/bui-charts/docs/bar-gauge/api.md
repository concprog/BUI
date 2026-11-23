# Bar Gauge: API reference

`<BarGauge>` shows one or more values as bars filled between a minimum and a maximum, coloured by thresholds or a colour scheme. If you have used Grafana's Bar Gauge panel, see [Coming from Grafana](#coming-from-grafana).

```rust
use blenderui::prelude::*;
use bui_charts::*;
```

- [`<BarGauge>`](#bargauge)
- [Where the bars come from](#where-the-bars-come-from)
- [Display options](#display-options)
- [Range, colour and thresholds](#range-colour-and-thresholds)
- [Value text](#value-text)
- [Enums and types](#enums-and-types)
- [Coming from Grafana](#coming-from-grafana)

Not sure which option fits? See the [guide](guide.md).

---

## `<BarGauge>`

| Prop | Type | Default |
| --- | --- | --- |
| `data` | `Vec<T>`, `Signal<Vec<T>>`, `Frame<N>`, `Signal<Frame<N>>`, or your own [`Dataset`](../xy-chart/api.md#your-own-data-source) | **required** |
| `name` | `Fn(&T) -> &str` | field names for a `Frame` |
| `value` | `Fn(&T) -> f64` | field values for a `Frame` |
| `reduce` | [`Reduce`](#reduce) | one bar per row with `value`; the last value of each field for a `Frame` |
| `fields` | `&'static [&'static str]` | every field of a `Frame` |
| `display` | [`BarDisplay`](#enums-and-types) | `Gradient` |
| `value_mode` | [`ValueMode`](#enums-and-types) | `Color` |
| `name_placement` | [`NamePlacement`](#enums-and-types) | `Auto` |
| `text_mode` | [`TextMode`](#enums-and-types) | `Auto` |
| `orientation` | [`BarOrientation`](#enums-and-types) | `Auto` |
| `show_unfilled` | `bool` | `true` |
| `sizing` | [`Sizing`](#enums-and-types) | `Auto` |
| `title_size`, `value_size` | `Pixels` | fitted to the space |
| `min`, `max` | `f64` | `0.` and `100.` |
| `thresholds` | [`Thresholds`](#thresholds) | your theme's good colour, then its bad colour from 80 |
| `color` | [`GaugeColor`](#gaugecolor) | `Thresholds` |
| `format` | `Fn(f64) -> SharedString` | none |
| `decimals` | `usize` | up to 3, trailing zeros dropped |
| `legend` | [`Legend`](../xy-chart/api.md#legend) | `Legend::hidden()` |
| `class` | Tailwind classes | fills its parent |
| `key` | anything `Hash` (taken by `rsx!`: hook state follows it) | – |

## Where the bars come from

| Data | Props | Bars |
| --- | --- | --- |
| rows of your own type | `name` and `value` | one per row |
| rows of your own type | `value` and `reduce={Reduce::Calc(..)}` | one bar summarising every row |
| a `Frame` | none | one per field, showing its last value |
| a `Frame` | `reduce={Reduce::Calc(Calc::Mean)}` | one per field, summarised with the calc |
| a `Frame` | `reduce={Reduce::All { limit: Some(20) }}` | one per cell, row by row, up to the limit |
| a `Frame` | `fields={&["api", "web"]}` | only those fields |

`name` returns a borrowed string, so there's nothing to clone:

```rust
<BarGauge data={disks} name={|d: &Disk| d.mount.as_str()} value={|d| d.used} />
```

### `Reduce`

| Variant | Meaning |
| --- | --- |
| `Reduce::Calc(Calc)` | one value per field, using any [`Calc`](../xy-chart/api.md#calcs): `Last`, `Mean`, `Max`, … |
| `Reduce::All { limit }` | every value; `limit: None` shows them all |

## Display options

| Prop | Effect |
| --- | --- |
| `display` | `Gradient` fills the bar with a blend of the colours it passes through. `Lcd` lights up separate cells. `Basic` is a flat bar with a solid edge. |
| `orientation` | `Horizontal` stacks the bars as rows. `Vertical` lays them out as columns. `Auto` picks columns when the area is wider than it is tall. |
| `name_placement` | `Top` puts the name above the bar, `Left` beside it, `Hidden` hides it. `Auto` places it on top when the row is taller than 40px, otherwise on the left. Vertical bars always show the name underneath. |
| `text_mode` | `Value`, `Name`, `ValueAndName` or `None`. `Auto` shows the name when there's more than one bar or you provide a `name` accessor. |
| `value_mode` | `Color` draws the value in the bar's colour, `Text` in the normal text colour, `Hidden` hides it. |
| `show_unfilled` | Draws the empty part of the track. Ignored in `Lcd` mode. |
| `sizing` | `Auto` divides the space evenly, with rows between 16 and 300px tall and columns at least 8px wide. `Sizing::Manual { min_width, min_height, max_height }` sets those limits yourself. |
| `title_size`, `value_size` | Fixed text sizes instead of sizes fitted to the space. |

When the bars need more room than the area has, the ones that don't fit are cut off.

## Range, colour and thresholds

The bar's length is its value's position between `min` and `max`. Values above `max` fill the bar, and values below `min` or missing (`NaN`) show only a sliver.

### `GaugeColor`

| Variant | Colour of each bar |
| --- | --- |
| `GaugeColor::Thresholds` | the threshold step the value has reached |
| `GaugeColor::Palette` | the next colour from your theme's palette, one per bar |
| `GaugeColor::Scheme(Scheme)` | the value's position between `min` and `max` on a [colour scheme](../xy-chart/api.md#color-and-scheme) |
| `GaugeColor::Fixed(Hsla)` | one colour for every bar |

In `Gradient` mode, thresholds and schemes also blend along the bar: with thresholds the bar passes through each step it has reached, and with a scheme it shows the whole scheme along the track. In `Lcd` mode, each cell takes the colour for its own position.

### `Thresholds`

```rust
static STEPS: LazyLock<[(f64, Hsla); 3]> =
    LazyLock::new(|| [(f64::NEG_INFINITY, rgb(0x40a02b)), (60., rgb(0xdf8e1d)), (85., rgb(0xd20f39))]);

thresholds={Thresholds::absolute(&STEPS[..])}
```

| Constructor | Step values are |
| --- | --- |
| `Thresholds::absolute(steps)` | values in the same units as the data |
| `Thresholds::percentage(steps)` | percentages of the `min`–`max` range, from 0 to 100 |

The first step is the base colour, used for everything below the second step, so its value doesn't matter. Each later step applies from its value upwards. Steps take a `Vec` or a `&'static` slice; a static slice costs nothing to rebuild on every render.

## Value text

The value is shown using `format` if you set it, otherwise with `decimals`, otherwise with up to three decimals and the trailing zeros dropped.

```rust
format={|v| format!("{v:.0} GiB").into()}
```

## Enums and types

| Type | Values |
| --- | --- |
| `BarDisplay` | `Gradient` (default), `Lcd`, `Basic` |
| `ValueMode` | `Color` (default), `Text`, `Hidden` |
| `NamePlacement` | `Auto` (default), `Top`, `Left`, `Hidden` |
| `TextMode` | `Auto` (default), `ValueAndName`, `Value`, `Name`, `None` |
| `BarOrientation` | `Auto` (default), `Horizontal`, `Vertical` |
| `Sizing` | `Auto` (default), `Manual { min_width, min_height, max_height }` |
| `Reduce` | `Calc(Calc)`, `All { limit }` |
| `ThresholdsMode` | `Absolute`, `Percentage` |
| `GaugeColor` | `Thresholds` (default), `Palette`, `Scheme(Scheme)`, `Fixed(Hsla)` |

## Coming from Grafana

| Grafana option | Here |
| --- | --- |
| Display mode | `display` |
| Value display | `value_mode` |
| Name placement | `name_placement` |
| Text mode | `text_mode` |
| Show unfilled area | `show_unfilled` |
| Bar size, Min width, Min height, Max height | `sizing` |
| Orientation | `orientation` |
| Value options: Calculation / All values / Limit | `reduce` |
| Value options: Fields | `fields` |
| Text size: title / value | `title_size`, `value_size` |
| Min, Max | `min`, `max` |
| Thresholds (absolute / percentage) | `thresholds` |
| Color scheme | `color` |
| Unit, Decimals | `format`, `decimals` |
| Legend | `legend` |

Not available here: value mappings, data links, and the colour schemes that need Grafana's field-colour registry. Use `GaugeColor::Scheme` with any scheme from your theme instead.
