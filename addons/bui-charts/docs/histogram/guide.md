# Histogram: choosing options

How to pick between the options in the [API reference](api.md). Every link goes to a runnable example; the gallery has them all (`cargo run --example gallery`, then **Histogram**).

## Histogram or something else?

| You want to see | Use |
| --- | --- |
| how values are spread, all at once | `<Histogram>` |
| how that spread changes over time | [`<Heatmap>`](../heatmap/guide.md) with `calculate` |
| each value against another | [`<XYChart>`](../xy-chart/guide.md) |

## Getting data in

- **Raw values** (one row per request, file or score): one `<HistogramSeries>` per column you want to count.
- **Already counted**: `x_min`, `x_max` and the count as `value`.
- **A `Frame`**: every field is counted, or read as buckets when it has `xMin` and `xMax` fields.

Examples: [Latency](examples.md#1-latency), [Pre-counted buckets](examples.md#8-pre-counted-buckets), [xMin / xMax frame](examples.md#9-xmin--xmax-frame).

## Choosing buckets

| Goal | Use |
| --- | --- |
| a quick look | the default: about 30 round-sized buckets |
| coarser or finer | `bucket_count` |
| buckets that mean something, such as 25 ms or 10 points | `bucket_size` |
| buckets centred on round numbers | `bucket_offset` of half the size |

Examples: [Bucket size](examples.md#5-bucket-size), [Bucket offset](examples.md#6-bucket-offset), [Bucket count](examples.md#7-bucket-count).

## Comparing series

| Goal | Use |
| --- | --- |
| compare shapes | overlay them with a lower `fill_opacity` |
| see the total and each part | `stacking={Stacking::NORMAL}` |
| only the total | `combine={true}` |

Examples: [Two series](examples.md#2-two-series), [Stacked](examples.md#3-stacked), [Combined](examples.md#4-combined).

## Styling

`gradient={GradientMode::Opacity}` fades bars towards the axis; `Hue` shifts their colour. Set a series' `color` to match your app.

Examples: [Opacity gradient](examples.md#10-opacity-gradient), [Hue gradient](examples.md#11-hue-gradient).
