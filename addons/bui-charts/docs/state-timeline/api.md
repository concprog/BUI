# State Timeline: API reference

`<StateTimeline>` shows which state each thing was in over time, one lane per thing: services up or down, a machine's mode, a threshold level. It's a port of Grafana's State timeline panel; see [Coming from Grafana](#coming-from-grafana).

```rust
use blenderui::prelude::*;
use bui_charts::*;
```

- [`<StateTimeline>`](#statetimeline)
- [`<StateRow>`](#staterow)
- [Value mappings](#value-mappings)
- [Colour](#colour)
- [How states are drawn](#how-states-are-drawn)
- [Tooltip, legend and pages](#tooltip-legend-and-pages)
- [Enums and types](#enums-and-types)
- [Coming from Grafana](#coming-from-grafana)

Not sure which option fits? See the [guide](guide.md).

---

## `<StateTimeline>`

| Prop | Type | Default |
| --- | --- | --- |
| `data` | `Vec<T>`, `Signal<Vec<T>>`, `Frame<N>`, `Signal<Frame<N>>`, or your own [`Dataset`](../xy-chart/api.md#your-own-data-source) | **required** |
| `x` | `Fn(&T) -> f64`: the time of each row | for a `Frame`: the `x_field`, or the first field |
| `x_field` | field name, for a `Frame` | the first field |
| children | `<StateRow>` tags | for a `Frame`: one lane per field other than x |
| `merge_values` | `bool` | `true` |
| `show_value` | [`VisibilityMode`](#enums-and-types) | `Auto` |
| `align_value` | [`ValueAlign`](#enums-and-types) | `Left` |
| `row_height` | `f32`, share of each lane, `0.0`–`1.0` | `0.9` |
| `per_page` | `usize` | `20` |
| `mappings` | `&'static` [`ValueMappings`](#value-mappings) | none |
| `color` | [`GaugeColor`](#colour) | green–yellow–red scheme |
| `thresholds` | [`Thresholds`](../bar-gauge/api.md#thresholds) | your theme's good colour, then its bad colour from 80 |
| `min` / `max` | `f64` | each lane's smallest / largest number |
| `fill_opacity` | `f32`, `0.0`–`1.0` | `0.7` |
| `line_width` | `f32` px | `0.` |
| `span_nulls` | [`NullSpan`](../trend/api.md#missing-values) | `Never` |
| `insert_nulls` | `f64`, in x units | none |
| `hide_from` | [`HideFrom`](../xy-chart/api.md#enums) | nothing hidden |
| `x_axis` | [`AxisConfig`](../xy-chart/api.md#axisconfig) | numeric; use `.time()` for timestamps |
| `y_axis` | `AxisConfig`: `placement` (`Hidden` hides lane names, `Right` moves them) and `width` | names on the left |
| `legend` | [`Legend`](../xy-chart/api.md#legend): `list()` / `hidden()`, `bottom()` / `right()` | `Legend::list()` at the bottom |
| `tooltip` | [`Tooltip`](../trend/api.md#tooltip-legend-and-keyboard) | `Tooltip::single()` |
| `on_select` | `Fn(SelectedRange, &mut Window, &mut App)` | none |
| `class`, `key` | | |

## `<StateRow>`

```rust
<StateRow name="api" value={|c: &Check| c.api} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `value` | `Fn(&T) -> V`, where `V` is a number, text, `bool`, `Option` of those, or a `StateValue` | **required** |
| `name` | text | `"Series 1"`, `"Series 2"`, … |
| `mappings`, `color`, `thresholds`, `fill_opacity`, `line_width`, `span_nulls`, `insert_nulls`, `hide_from` | as on the chart | the chart's |

`None`, `f64::NAN` and `StateValue::Null` are missing values.

## Value mappings

Mappings turn raw values into state names and colours. Build them once, in a `static`:

```rust
static STATES: LazyLock<ValueMappings> = LazyLock::new(|| {
    ValueMappings::new()
        .value("200", "OK", rgb(0x40a02b))
        .range(500., 599., "Server error", rgb(0xd20f39))
        .regex("^timeout", "Timeout", rgb(0xdf8e1d))
        .special(Special::Null, "No data", rgb(0x9ca0b0))
});

<StateTimeline data={checks} x={|c: &Check| c.time} mappings={&*STATES}>
```

| Method | Matches |
| --- | --- |
| `value(text, name, color)` | the value's text exactly: `"OK"`, `"1"`, `"true"` |
| `range(from, to, name, color)` | numbers from `from` to `to`, both included |
| `regex(pattern, name, color)` | text that matches the regular expression |
| `special(Special::…, name, color)` | `Null`, `NaN`, `NullOrNaN`, `True`, `False` or `Empty` (empty text) |

The first matching mapping wins. An empty `name` keeps the original value as the label. Mapped missing values are drawn as states; unmapped missing values are gaps.

## Colour

Values that no mapping matches are coloured by `color`:

| `GaugeColor` | Numbers | Text and `bool` |
| --- | --- | --- |
| `Scheme(scheme)` (default: green–yellow–red) | their position between `min` and `max` on the scheme | one palette colour per distinct value |
| `Thresholds` | the threshold step they reach | one palette colour per distinct value |
| `Palette` | the lane's palette colour | one palette colour per distinct value |
| `Fixed(color)` | that colour | that colour |

With `Thresholds` and `merge_values`, neighbouring numbers in the same step merge into one block labelled `< 50`, `50+`, `70+`, as in Grafana.

## How states are drawn

- **Merging.** Each state lasts until the next row with a different state. With `merge_values={false}`, every row starts a new block.
- **The last state** lasts until `x_axis.max` if you set it, otherwise for one typical step past the last row.
- **Missing values** end the current state (a gap) unless `span_nulls` bridges them. `insert_nulls` ends a state when the next row is further away than its value.
- **Rows.** Each lane gets an equal share of the height; `row_height` is how much of it the blocks fill.
- **Fill and outline.** Blocks are filled at `fill_opacity`. `line_width` adds an outline in the full colour. Colours that already have their own transparency keep it.
- **Value text.** With `show_value`:
  - `Auto` writes the label when at least two characters fit, shortening it with "…".
  - `Always` writes it even when only one character fits.
  - `Never` writes none.
  - `align_value` places it at the start, centre or end of the visible part of the block.

**Zooming.** Drag across the lanes to zoom the time axis; double-click to zoom out. `on_select` receives the range as a `SelectedRange`.

## Tooltip, legend and pages

- **Tooltip.**
  - `Tooltip::single()` shows the hovered lane's state, with its start time and how long it lasted.
  - `Tooltip::multi()` shows every lane's state at the hovered time, and draws a line there.
  - `sort` orders the rows by state name.
- **Legend.** It lists the mapped states; otherwise the threshold steps, the distinct text values, or the colour scheme as a gradient with its range.
- **Pages.** With more lanes than `per_page`, page buttons appear below the chart.

## Enums and types

| Type | Values |
| --- | --- |
| `StateValue` | `Null`, `Number(f64)`, `Text(SharedString)`, `Bool(bool)`; made from numbers, text, `bool` and `Option` |
| `Special` | `Null`, `NaN`, `NullOrNaN`, `True`, `False`, `Empty` |
| `ValueAlign` | `Left` (default), `Center`, `Right` |
| `VisibilityMode` | `Auto` (default), `Always`, `Never` |
| `SelectedRange` | `{ x: (f64, f64), y: Option<(f64, f64)> }` |

## Coming from Grafana

| Grafana option | Here |
| --- | --- |
| Merge equal consecutive values | `merge_values` |
| Show values, Align values, Row height | `show_value`, `align_value`, `row_height` |
| Page size | `per_page` |
| Line width, Fill opacity | `line_width`, `fill_opacity` (0.0–1.0) |
| Connect null values, Disconnect values | `span_nulls`, `insert_nulls` |
| Value mappings | `mappings` |
| Colour scheme, Thresholds, Min, Max | `color`, `thresholds`, `min`, `max` |
| Axis placement and width | `y_axis` |
| Legend, Tooltip | `legend`, `tooltip` |
| Hide in area | `hide_from` |

Not available here: units from Grafana's unit registry, data links and annotations. Text values without a mapping get one palette colour per value, where Grafana gives them all the field's colour.
