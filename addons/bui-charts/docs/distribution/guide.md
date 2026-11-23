# Distribution charts: choosing a chart

How to pick between the charts in the [API reference](api.md). Every link goes to runnable examples; the gallery has them all (`cargo run --example gallery`, then **Charts**).

## Which chart

| Your data | Use |
| --- | --- |
| thousands of values; counts per bucket are enough | [`<Histogram>`](../histogram/guide.md) |
| a few hundred values; every one should be visible | [`<Barcode>`](examples.md#barcode) |
| up to a few hundred values; every one visible and the shape clear | [`<Beeswarm>`](examples.md#beeswarm) |
| many groups, compared by shape | [`<Ridgeline>`](examples.md#ridgeline) |
| many series over time, in little height | [`<Horizon>`](examples.md#horizon) |

## Groups

`<Barcode>`, `<Beeswarm>` and `<Ridgeline>` take a `group` accessor. Groups keep the order of their first row, so sort your rows to sort the groups.

## Ridgeline bandwidth

The default bandwidth (Silverman's rule) suits roughly bell-shaped data. For data with several peaks, pass a smaller `bandwidth` to keep them apart. `overlap` sets how far each curve reaches into the row above.

## Horizon

A horizon row folds a line chart into `bands` layers, so a row can be a fifth of the height of a line chart and still show the same detail. Read the darkest shade as "largest". Use it for deltas or values around a baseline, which have both signs. More bands means flatter rows that are harder to read; 3 is a good start.
