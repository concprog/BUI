# State Timeline: choosing options

How to pick between the options in the [API reference](api.md). Every link goes to a runnable example; the gallery has them all (`cargo run --example gallery`, then **State Timeline**).

## State Timeline or something else?

| You want to see | Use |
| --- | --- |
| which state each thing was in, and when it changed | `<StateTimeline>` |
| one result per sample, every sample visible | [`<StatusHistory>`](../status-history/guide.md) |
| how a number moved over time | [`<Trend>`](../trend/guide.md) |
| how a whole distribution moved over time | [`<Heatmap>`](../heatmap/guide.md) |

## Text states

Most status data is text or codes: `"OK"`, `"DEGRADED"`, `200`, `503`. Give every state a name and a colour with [value mappings](api.md#value-mappings), and share one `static` mapping across every lane so the legend lists each state once.

Examples: [Service status](examples.md#1-service-status), [Booleans](examples.md#4-booleans).

## Numbers as states

| You want | Use |
| --- | --- |
| levels such as normal / high / critical | `color={GaugeColor::Thresholds}` with `thresholds`; neighbouring samples in the same level merge |
| to see the actual number, shaded by size | the default scheme, with `min` / `max` |
| one colour per lane, values as labels | `GaugeColor::Palette` |

Examples: [Threshold states](examples.md#2-threshold-states), [Colour by value](examples.md#3-colour-by-value), [From a frame](examples.md#10-from-a-frame).

## Dense or sparse

- Many short states: `show_value={VisibilityMode::Never}` and a lower `row_height` keep it readable; hover for details.
- Sampled data where every sample matters: `merge_values={false}`.
- Few long states: `align_value={ValueAlign::Center}` or `Right`.

Examples: [Thin rows, no labels](examples.md#5-thin-rows-no-labels), [Every sample](examples.md#8-every-sample), [Right-aligned values](examples.md#6-right-aligned-values).

## Missing data

| Your data | Use |
| --- | --- |
| a missing value means "unknown" | the default: a gap |
| a missing value means "no change" | `span_nulls={NullSpan::Always}` |
| rows simply stop when a source goes silent | `insert_nulls={timeout}` |
| "no data" is a state worth showing | a `Special::Null` mapping |

Example: [Missing values](examples.md#7-missing-values).

## Many lanes

`per_page` splits lanes into pages; keep it at a number that fits your panel's height. Use `Tooltip::multi()` to compare all lanes at one moment.

Examples: [Pages](examples.md#9-pages), [Multi tooltip](examples.md#11-multi-tooltip).

## Linking to the rest of your app

Dragging zooms the chart. Use `on_select` to move the rest of your dashboard to the same time range.

Example: [Drag to zoom](examples.md#12-drag-to-zoom).
