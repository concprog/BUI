# Time Series: choosing options

How to pick between the options in the [API reference](api.md). Every link goes to a runnable example; the gallery has them all (`cargo run --example gallery`, then **Time Series**).

## Time Series or something else?

| You want to see | Use |
| --- | --- |
| values changing over time | `<TimeSeries>` |
| values against a numeric x that isn't time: speed, distance | [`<Trend>`](../trend/guide.md) |
| prices with open, high, low and close | [`<Candlestick>`](../candlestick/guide.md) |
| a whole distribution over time | [`<Heatmap>`](../heatmap/guide.md) |
| states over time | [`<StateTimeline>`](../state-timeline/guide.md) |

## Drawing the data

| Data | Use |
| --- | --- |
| a few continuous metrics | lines, the default |
| parts of a total | `stacking={Stacking::NORMAL}` with a `fill_opacity` |
| counts per interval | `draw_style={DrawStyle::Bars}` |
| two units on one chart | a second `y_axis` with `placement(AxisPlacement::Right)` |

Examples: [Basic](examples.md#1-basic), [Stacked areas](examples.md#8-stacked-areas), [Bars](examples.md#9-bars), [Two axes](examples.md#12-two-axes).

## Adding context

- **Annotations** mark deploys, incidents and maintenance windows right on the chart, so a spike can be read against what happened.
- **Exemplars** link a percentile line to the individual slow requests behind it.
- **Thresholds** show where a metric becomes a problem: `ThresholdsStyle::DashedAndArea` adds lines and shaded bands.

Examples: [Annotations](examples.md#2-annotations), [Exemplars](examples.md#3-exemplars), [Thresholds](examples.md#10-thresholds).

## Comparing

- `time_compare` overlays the previous day or week, dashed, to spot what is unusual today.
- `timezones` helps teams in several regions read the same chart.

Examples: [Time compare](examples.md#5-time-compare), [Several timezones](examples.md#6-several-timezones).

## Interacting

Drag to zoom and double-click to reset. Use `on_select` to drive the rest of your app, such as loading finer data for the selected range. `Tooltip::multi()` and a table legend with calcs help read exact values.

Examples: [Drag to zoom](examples.md#4-drag-to-zoom), [Live data](examples.md#14-live-data).

## Layout

`orientation={VizOrientation::Vertical}` runs time down the page: useful in tall, narrow panels or next to a log view.

Example: [Vertical](examples.md#7-vertical).
