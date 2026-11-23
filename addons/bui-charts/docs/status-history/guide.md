# Status History: choosing options

How to pick between the options in the [API reference](api.md). Every link goes to a runnable example; the gallery has them all (`cargo run --example gallery`, then **Status History**).

## Status History or State Timeline?

| Your data | Use |
| --- | --- |
| periodic samples where each one is a result: checks, hourly jobs, daily totals | `<StatusHistory>`: one cell per sample |
| a state that lasts until it changes: up/down, a machine's mode | [`<StateTimeline>`](../state-timeline/guide.md): equal neighbours merge into one block |
| a number whose shape over time matters | [`<Trend>`](../trend/guide.md) |

## Text and codes

Give every state a name and a colour with [value mappings](../state-timeline/api.md#value-mappings), and share one `static` mapping across rows so the legend lists each state once.

Examples: [Service checks](examples.md#1-service-checks), [Booleans](examples.md#4-booleans).

## Numbers

| You want | Use |
| --- | --- |
| levels such as normal / high / critical | `color={GaugeColor::Thresholds}` with `thresholds` |
| the number itself, shaded by size | the default scheme with `min` / `max`, or any [`Scheme`](../xy-chart/api.md#color-and-scheme) |
| different rules per row | `color`, `thresholds` or `mappings` on a `<StatusRow>` |

Examples: [Threshold colours](examples.md#2-threshold-colours), [Colour by value](examples.md#3-colour-by-value), [Per-row colours](examples.md#10-per-row-colours).

## Density

- Many samples: hide the text with `show_value={VisibilityMode::Never}` and hover for details.
- A compact strip: `col_width={1.}` and `row_height={1.}` remove the gaps.
- Separated dots: a lower `col_width` and `row_height`.

Examples: [Narrow cells](examples.md#5-narrow-cells), [No gaps](examples.md#6-no-gaps).

## Not time

`x` can be any number: days, build numbers, test runs. Give `x_axis` a `format` to label them.

Example: [Weekly grid](examples.md#11-weekly-grid).

## Many rows

`per_page` splits rows into pages. `Tooltip::multi()` compares every row at one sample.

Examples: [Pages](examples.md#8-pages), [Multi tooltip](examples.md#9-multi-tooltip).
