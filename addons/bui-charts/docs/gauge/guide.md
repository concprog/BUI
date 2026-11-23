# Gauge: choosing options

How to pick between the options in the [API reference](api.md). Every link goes to a runnable example; the gallery has them all (`cargo run --example gallery`, then **Gauge**).

## Gauge or something else?

| You want | Use |
| --- | --- |
| one value against a known range, with limits marked | `<Gauge>` |
| the number as large as possible | [`<Stat>`](../stat/guide.md) |
| many values compared side by side | [`<BarGauge>`](../bar-gauge/guide.md) |

## Setting the range

Gauges read best with a fixed range. Set `min` and `max` to the real limits: 0–100 for percentages, the rated maximum for temperatures and speeds. The default is 0–100.

For values that can go either way, such as a balance or a difference, set `neutral` so the bar grows from it.

Examples: [Gauge](examples.md#1-gauge), [Neutral point](examples.md#7-neutral-point).

## Showing limits

| Goal | Use |
| --- | --- |
| see which step the value is in | the default threshold markers ring |
| read the limits | `show_threshold_labels={true}` |
| see each step as a band on the bar | `segment_count` |
| a clean look | `show_threshold_markers={false}` |

Examples: [Threshold labels](examples.md#6-threshold-labels), [Segments](examples.md#4-segments), [No text](examples.md#12-no-text).

## Style

- **Arc or ring.** `GaugeShape::Gauge` suits single headline values; `Circle` packs better in grids.
- **Bar.** `BarShape::Rounded` with a larger `bar_width_factor` looks softer; thin flat bars look technical.
- **Effects.** `gradient` makes progress along the arc obvious; glows add depth on dark dashboards. Use them sparingly on busy pages.

Examples: [Circle](examples.md#2-circle), [Rounded bar](examples.md#3-rounded-bar), [Effects](examples.md#5-effects), [Colour scheme](examples.md#9-colour-scheme).

## Many gauges

Pass a `Frame`, name its time field with `x_field`, and every other field gets a gauge. Use `Sizing::Manual` to keep them readable when the panel is narrow.

Examples: [Many gauges](examples.md#8-many-gauges), [Manual sizing](examples.md#10-manual-sizing), [Palette colours](examples.md#11-palette-colours).
