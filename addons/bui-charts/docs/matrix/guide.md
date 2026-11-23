# Matrix charts: choosing a chart

How to pick between the charts in the [API reference](api.md). Every link goes to runnable examples; the gallery has them all (`cargo run --example gallery`, then **Charts**).

## Which chart

| Your data | Use |
| --- | --- |
| a signal sampled at a steady rate, and you want its frequencies over time | [`<Spectrogram>`](examples.md#spectrogram) |
| a square or rectangular matrix of weights between two lists of labels | [`<AttentionMap>`](examples.md#attentionmap) |
| shares of a whole, to be counted at a glance | [`<Waffle>`](examples.md#waffle) |
| values over time and value buckets | [`<Heatmap>`](../heatmap/guide.md) |

## Spectrogram window

`window_size` trades time detail for frequency detail. Long windows separate close frequencies but smear changes over time; short windows catch quick changes but blur the frequencies. `overlap` gives more columns without changing that trade. The sample rate comes from your `x` values, so pass time in seconds to read the y axis in Hz.

## Attention maps

Pass `weights` row by row: one row per query, one value per key. For self-attention, leave out `queries`. Values are written in the cells up to 12 tokens; beyond that the cells get too small, so they are off unless you pass `show_values={true}`.

## Waffle

10 × 10 makes each square one percent. For other grids, the chart rounds the shares so that the squares still add up exactly.
