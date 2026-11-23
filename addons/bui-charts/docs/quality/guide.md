# Quality charts: choosing a chart

How to pick between the charts in the [API reference](api.md). Every link goes to runnable examples; the gallery has them all (`cargo run --example gallery`, then **Charts**).

## Which chart

| Your question | Use |
| --- | --- |
| Which few causes make up most of the problem? | [`<Pareto>`](examples.md#pareto) |
| Is the process stable, or did something change? | [`<ControlChart>`](examples.md#controlchart) |
| Do two ways of measuring the same thing agree? | [`<BlandAltman>`](examples.md#blandaltman) |

## Pareto

Pass the raw counts in any order; the chart sorts them. The cumulative line crosses the dashed `threshold` (80% by default) at the "vital few": fix the bars left of that point first.

## Control chart

- Let the chart estimate the limits from the data while you are learning a process. Once the process is known, pass `center` and `sigma` so new data is judged against the old limits rather than against itself.
- `zones={true}` adds ±1σ and ±2σ lines, for the Western Electric run rules.
- Points outside the limits are drawn in your theme's bad colour.

Examples: [In control](examples.md#in-control), [Process shift](examples.md#process-shift), [Known centre and σ](examples.md#known-centre-and-σ).

## Bland–Altman

Don't use a scatter plot of method A against method B to judge agreement: two methods can correlate well and still disagree. Bland–Altman plots the difference against the mean:

- the **bias** line shows how far apart the methods are on average;
- the **limits of agreement** show how far apart they can be for one sample (95% by default; `agreement={2.58}` for 99%);
- a slope in the points means the disagreement grows with the size of the value.
