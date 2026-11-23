# Status History: API reference

`<StatusHistory>` shows a grid of samples: one row per thing and one cell per sample, each cell coloured by its value. Use it when every sample matters: periodic health checks, hourly SLO results, a value per day. It is a port of Grafana's Status history panel; see [Coming from Grafana](#coming-from-grafana).

```rust
use blenderui::prelude::*;
use bui_charts::*;
```

- [`<StatusHistory>`](#statushistory)
- [`<StatusRow>`](#statusrow)
- [How cells are drawn](#how-cells-are-drawn)
- [Coming from Grafana](#coming-from-grafana)

Status History shares its colours, value mappings, tooltip, legend and pages with [State Timeline](../state-timeline/api.md). Not sure which of the two you need? See the [guide](guide.md).

---

## `<StatusHistory>`

```rust
<StatusHistory data={probes} x={|p: &Probe| p.time} mappings={&*STATES} x_axis={AxisConfig::new().time()}>
    <StatusRow name="api" value={|p: &Probe| p.api} />
    <StatusRow name="db" value={|p: &Probe| p.db} />
</StatusHistory>
```

| Prop | Type | Default |
| --- | --- | --- |
| `data` | `Vec<T>`, `Signal<Vec<T>>`, `Frame<N>`, `Signal<Frame<N>>`, or your own [`Dataset`](../xy-chart/api.md#your-own-data-source) | **required** |
| `x` | `Fn(&T) -> f64`: the time of each sample | for a `Frame`: the `x_field`, or the first field |
| `x_field` | field name, for a `Frame` | the first field |
| children | `<StatusRow>` tags | for a `Frame`: one row per field other than x |
| `show_value` | [`VisibilityMode`](../state-timeline/api.md#enums-and-types) | `Auto` |
| `row_height` | `f32`, share of each row, `0.0`–`1.0` | `0.9` |
| `col_width` | `f32`, share of each column, `0.0`–`1.0` | `0.9` |
| `per_page` | `usize` | `20` |
| `mappings` | `&'static` [`ValueMappings`](../state-timeline/api.md#value-mappings) | none |
| `color` | [`GaugeColor`](../state-timeline/api.md#colour) | green–yellow–red scheme |
| `thresholds` | [`Thresholds`](../bar-gauge/api.md#thresholds) | your theme's good colour, then its bad colour from 80 |
| `min` / `max` | `f64` | each row's smallest / largest number |
| `fill_opacity` | `f32`, `0.0`–`1.0` | `0.7` |
| `line_width` | `f32` px | `1.` |
| `hide_from` | [`HideFrom`](../xy-chart/api.md#enums) | nothing hidden |
| `x_axis` | [`AxisConfig`](../xy-chart/api.md#axisconfig) | numeric; use `.time()` for timestamps |
| `y_axis` | `AxisConfig`: `placement` (`Hidden` hides row names, `Right` moves them) and `width` | names on the left |
| `legend` | [`Legend`](../xy-chart/api.md#legend): `list()` / `hidden()`, `bottom()` / `right()` | `Legend::list()` at the bottom |
| `tooltip` | [`Tooltip`](../trend/api.md#tooltip-legend-and-keyboard) | `Tooltip::single()` |
| `on_select` | `Fn(SelectedRange, &mut Window, &mut App)` | none |
| `class`, `key` | | |

Zoom: hold Ctrl (Cmd on macOS) and scroll, or pinch, to zoom the time axis about the pointer. Drag to select a range, and double-click to reset.

## `<StatusRow>`

```rust
<StatusRow name="api" value={|p: &Probe| p.api} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `value` | `Fn(&T) -> V`, where `V` is a number, text, `bool`, `Option` of those, or a `StateValue` | **required** |
| `name` | text | `"Series 1"`, `"Series 2"`, … |
| `mappings`, `color`, `thresholds`, `fill_opacity`, `line_width`, `hide_from` | as on the chart | the chart's |

`None`, `f64::NAN` and `StateValue::Null` are missing values: their cells stay empty unless a mapping such as `Special::Null` names them.

## How cells are drawn

- **One cell per sample.** Each sample gets a column as wide as the typical gap between samples. The cell is centred on the sample's time, and `col_width` is how much of the column it fills. Equal neighbours never merge; use [State Timeline](../state-timeline/api.md) for that.
- **Rows.** Each row gets an equal share of the height; `row_height` is how much of it the cells fill.
- **Fill and outline.** Cells are filled at `fill_opacity`, with a `line_width` outline in the full colour.
- **Value text** is centred in the cell. `Auto` writes it when at least two characters fit, `Always` when one fits, `Never` hides it.
- **Tooltip.** Hovering shows the sample's time and value. `Tooltip::multi()` lists every row at that sample.
- **Zooming.** Drag across the cells to zoom the time axis; double-click to zoom out. `on_select` receives the range.

## Coming from Grafana

| Grafana option | Here |
| --- | --- |
| Show values, Row height, Column width | `show_value`, `row_height`, `col_width` |
| Page size | `per_page` |
| Line width, Fill opacity | `line_width`, `fill_opacity` (0.0–1.0) |
| Value mappings | `mappings` |
| Colour scheme, Thresholds, Min, Max | `color`, `thresholds`, `min`, `max` |
| Axis placement and width | `y_axis` |
| Legend, Tooltip | `legend`, `tooltip` |
| Hide in area | `hide_from` |

Not available here: units from Grafana's unit registry, data links and annotations.
