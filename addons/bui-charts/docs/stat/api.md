# Stat: API reference

`<Stat>` shows one big number per value: the current CPU, today's revenue, the error count. It can add a sparkline of the history, colour the number or the whole tile, and show the change since the start. It is a port of Grafana's Stat panel; see [Coming from Grafana](#coming-from-grafana).

```rust
use blenderui::prelude::*;
use bui_charts::*;
```

- [`<Stat>`](#stat)
- [Which values are shown](#which-values-are-shown)
- [Layout](#layout)
- [Colour](#colour)
- [Percent change](#percent-change)
- [Enums](#enums)
- [Coming from Grafana](#coming-from-grafana)

Not sure which option fits? See the [guide](guide.md).

---

## `<Stat>`

```rust
<Stat data={samples} value={|s: &Sample| s.cpu} x={|s: &Sample| s.time} format={percent} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `data` | `Vec<T>`, `Signal<Vec<T>>`, `Frame<N>`, `Signal<Frame<N>>`, or your own [`Dataset`](../xy-chart/api.md#your-own-data-source) | **required** |
| `value` | `Fn(&T) -> f64` | every field of a `Frame` |
| `name` | `Fn(&T) -> &str`: the name of each row, with `Reduce::All` | field names for a `Frame` |
| `reduce` | [`Reduce`](../bar-gauge/api.md#reduce) | `Reduce::Calc(Calc::Last)` |
| `fields` | `&'static [&'static str]`: which fields of a `Frame` | every field except `x_field` |
| `x` | `Fn(&T) -> f64`: the time of each row, for the sparkline | the row number |
| `x_field` | field name: a `Frame`'s time field, used for the sparkline and not shown as a value | none |
| `color_mode` | [`StatColorMode`](#enums) | `Value` |
| `graph_mode` | [`GraphMode`](#enums) | `Area` |
| `justify_mode` | [`JustifyMode`](#enums) | `Auto` |
| `text_mode` | [`TextMode`](../bar-gauge/api.md#enums-and-types) | `Auto` |
| `orientation` | [`BarOrientation`](../bar-gauge/api.md#enums-and-types) | `Auto` |
| `wide_layout` | `bool` | `true` |
| `show_percent_change` | `bool` | `false` |
| `percent_change_color` | [`PercentChangeColor`](#enums) | `Standard` |
| `title_size`, `value_size`, `percent_size` | `Pixels` | fitted to the tile |
| `min`, `max` | `f64` | the smallest and largest number in the data |
| `thresholds` | [`Thresholds`](../bar-gauge/api.md#thresholds) | your theme's good colour, then its bad colour from 80 |
| `color` | [`GaugeColor`](../bar-gauge/api.md#gaugecolor) | `Thresholds` |
| `mappings` | `&'static` [`ValueMappings`](../state-timeline/api.md#value-mappings) | none |
| `format` | `Fn(f64) -> SharedString` | none |
| `decimals` | `usize` | up to 3, trailing zeros dropped |
| `class`, `key` | | |

## Which values are shown

`reduce` decides how many tiles there are:

| Your data | `reduce` | Tiles |
| --- | --- | --- |
| rows with `value` | `Reduce::Calc(calc)` (default `Last`) | one tile: `calc` over every row, with a sparkline |
| rows with `value` and `name` | `Reduce::All { limit }` | one tile per row, no sparkline |
| a `Frame` | `Reduce::Calc(calc)` | one tile per field, each with its own sparkline |
| a `Frame` | `Reduce::All { limit }` | one tile per cell |

## Layout

Each tile picks one of Grafana's four layouts from its size:

| Tile | Layout |
| --- | --- |
| more than 2.5 times wider than tall, with a sparkline and taller than 50 px | name on the left, value on the right, sparkline in the bottom half |
| more than 2.5 times wider than tall, otherwise | name and value side by side, centred vertically |
| taller than 100 px, with a sparkline | name above value, sparkline filling the rest |
| otherwise | name above value, centred vertically |

- `wide_layout={false}` always uses the stacked layouts.
- `justify_mode={JustifyMode::Center}` centres the text. Tiles without a name are always centred.
- Text is sized to fit. With several tiles, every tile uses the size that fits the longest name and value, so they line up. `title_size`, `value_size` and `percent_size` fix the sizes instead.
- `text_mode`: `Auto` shows only the value for a single tile and the name and value for several. `Name` shows the name in place of the value; `None` shows neither and leaves the colour and sparkline.
- **Several tiles.** `BarOrientation::Auto` packs tiles into the grid that keeps them closest to square. `Horizontal` stacks them as rows and `Vertical` places them side by side.

## Colour

The value's colour comes from a mapping if one matches, otherwise from `color`: the threshold step it reaches, its position on a scheme between `min` and `max`, a palette colour per tile, or one fixed colour.

| `color_mode` | Effect |
| --- | --- |
| `Value` | the number in the value's colour |
| `Background` | the tile filled with a gradient of the value's colour, text in black or white for contrast |
| `BackgroundSolid` | the tile filled with the value's colour |
| `None` | the number in your theme's text colour |

The sparkline uses the value's colour, or white on a background. `GraphMode::Area` fills under the line; `Line` draws only the line; `None` hides it.

## Percent change

With `show_percent_change`, each tile reduced from several rows shows the change from the first to the last number, as Grafana computes it: `(last - first) / first × 100`. An arrow shows the direction.

| `percent_change_color` | Colour |
| --- | --- |
| `Standard` | green when it rises, red when it falls or stays the same |
| `Inverted` | red when it rises, green when it falls: for errors, latency, cost |
| `SameAsValue` | the value's colour |

No change is shown when the first number is 0.

## Enums

| Type | Values |
| --- | --- |
| `StatColorMode` | `Value` (default), `Background`, `BackgroundSolid`, `None` |
| `GraphMode` | `Area` (default), `Line`, `None` |
| `JustifyMode` | `Auto` (default), `Center` |
| `PercentChangeColor` | `Standard` (default), `Inverted`, `SameAsValue` |
| `TextMode` | `Auto` (default), `ValueAndName`, `Value`, `Name`, `None` |
| `BarOrientation` | `Auto` (default), `Horizontal`, `Vertical` |

## Coming from Grafana

| Grafana option | Here |
| --- | --- |
| Calculation, Fields, Show (calculate / all values), Limit | `reduce`, `fields` |
| Orientation | `orientation` |
| Text mode, Wide layout | `text_mode`, `wide_layout` |
| Color mode, Graph mode, Text alignment | `color_mode`, `graph_mode`, `justify_mode` |
| Show percent change, Percent change color mode | `show_percent_change`, `percent_change_color` |
| Text size: title, value, percent | `title_size`, `value_size`, `percent_size` |
| Min, Max, Thresholds, Color scheme, Value mappings | `min`, `max`, `thresholds`, `color`, `mappings` |
| Unit, Decimals | `format`, `decimals` |

Not available here: units from Grafana's unit registry, data links and the panel's time range for the sparkline (it spans the data instead).
