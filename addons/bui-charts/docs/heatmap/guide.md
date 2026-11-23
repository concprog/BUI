# Heatmap: choosing options

How to pick between the options in the [API reference](api.md). Every link goes to a runnable example; the gallery has them all (`cargo run --example gallery`, then **Heatmap**).

## Heatmap or something else?

| You want to see | Use |
| --- | --- |
| how a distribution changes over time: latency, sizes, durations | `<Heatmap>` with `calculate` |
| a value for every pair of two categories: day × hour, region × service | `<Heatmap>` with `value` |
| one value changing over time | [`<Trend>`](../trend/guide.md) or [`<XYChart>`](../xy-chart/guide.md) |
| which state something was in, over time | [`<StateTimeline>`](../state-timeline/guide.md) |

## Getting data in

- **Raw samples** (one row per request, measurement or event): pass `x`, `y` and `calculate`. Start with `Buckets::Auto`; set a `Size` once you know which resolution reads well, such as 1 minute by 20 ms.
- **Pre-counted cells**: pass `x`, `y` and `value`, one row per cell.
- **Histogram buckets** from Prometheus or OpenTelemetry: pass a `Frame` with x first and one field per bucket, and `bucket_layout={BucketLayout::Le}`.

Use `AxisConfig::new().time()` on `x_axis` when x is a timestamp.

Examples: [Latency over time](examples.md#1-latency-over-time), [Cells with values](examples.md#4-cells-with-values), [Histogram buckets](examples.md#3-histogram-buckets).

## Long-tailed values

Latencies and sizes often bunch up near zero with a long tail. Use `y_scale: BucketScale::Log2` (or `Log10`) so the tail gets as much room as the bulk.

Example: [Log buckets](examples.md#2-log-buckets).

## Choosing colours

| Goal | Use |
| --- | --- |
| a clear "more is hotter" reading | single-hue `Oranges`, `Reds` or `Blues`, or `YlOrRd` |
| fine differences across the whole range | `Viridis`, `Magma` or `Cividis`: evenly bright steps, readable in greyscale and by colour-blind people |
| a value with a meaningful middle | a diverging scheme such as `RdBu` or `Spectral` |
| matching your app's look | `HeatmapColor::theme(Scheme::…)` or `HeatmapColor::opacity(brand)` |

Fewer `steps` (8–16) make bands easier to compare. Fix `color_min` / `color_max` when charts should be comparable with each other or over time.

Examples: [Opacity mode](examples.md#5-opacity-mode), [16 steps, no gap](examples.md#6-16-steps-no-gap), [Theme scheme](examples.md#7-theme-scheme), [Fixed colour range](examples.md#9-fixed-colour-range).

## Reducing noise

- `hide_le` hides near-empty buckets so the shape stands out; `hide_ge` hides outliers that would wash out the scale.
- Add `exemplars` to point at individual samples worth investigating, such as the slowest requests with trace IDs.

Examples: [Hidden cells](examples.md#10-hidden-cells), [Exemplars](examples.md#11-exemplars).

## Reading values

- `y_histogram` in the tooltip shows the whole column, which is useful for spotting bimodal distributions.
- `show_value={VisibilityMode::Always}` with a `cell_gap` and `cell_radius` suits small category grids that people read cell by cell.

Examples: [Tooltip histogram](examples.md#8-tooltip-histogram), [Cells with values](examples.md#4-cells-with-values).

## Zooming

The default `SelectionMode::X` zooms time; `XY` zooms into a box. Use `on_select` when the selection should drive the rest of your app, such as fetching finer data for that range.

Example: [Box zoom](examples.md#12-box-zoom).
