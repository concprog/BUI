# Category charts: choosing a chart

How to pick between the charts in the [API reference](api.md). Every link goes to runnable examples; the gallery has them all (`cargo run --example gallery`, then **Charts**).

## Which chart

| Your question | Use |
| --- | --- |
| How big is each item? Many items, and bars feel heavy | [`<Lollipop>`](examples.md#lollipop) |
| How far apart are two values per item: before / after, min / max, men / women? | [`<Dumbbell>`](examples.md#dumbbell) |
| Which items went up and which went down between two moments? | [`<Slope>`](examples.md#slope) |
| How did the ranking change over several rounds? | [`<Bump>`](examples.md#bump) |
| Two groups side by side per category: a population pyramid, A vs B | [`<BackToBack>`](examples.md#backtoback) |
| How did a start value become an end value, step by step? | [`<Waterfall>`](examples.md#waterfall) |
| Is each KPI on target, and how good is that? | [`<Bullet>`](examples.md#bullet) |
| How many are left after each stage? | [`<Funnel>`](examples.md#funnel) |
| What did each study find, how sure is it, and do they agree? | [`<Forest>`](examples.md#forest) |
| Which few causes make up most of the problem? | [`<Pareto>`](../quality/guide.md) |

## Data shape

- Every chart except `<Bump>` takes one row per item and draws them in your order. `<Lollipop>` and `<Dumbbell>` can sort for you with `sort`.
- `<Bump>` takes long rows: one per item per round, with the raw score in `value`. It does the ranking itself.
- `<Waterfall>` takes the changes, not the running totals. Leave `total` on to finish with the end value.
- `<Bullet>` bands are upper ends, smallest first: `[poor, ok, good]`.

## Orientation

Long names read better down the side, so `<Lollipop>`, `<Dumbbell>`, `<Bullet>` and `<Funnel>` default to `horizontal={true}`. `<Waterfall>` reads left to right as a story, so it defaults to vertical. Flip either with `horizontal`.

## Colour

- `<Slope>` and `<Waterfall>` colour by direction: good for up, bad for down. Turn that off on a slope with `color_by_change={false}` when up is not "good".
- `<Dumbbell>` and `<BackToBack>` take one colour per side, and show them in the legend with your labels.
- `<Funnel>` gives each stage its own colour; pass `color` for one colour throughout.

## Forest plots

Pass the interval and the estimate in the same units. For ratios (odds, hazard or risk ratios) set `null={1.}` and a log `value_axis`, so that halving and doubling sit the same distance from the line. Use `weight` (study size or inverse variance) so the squares show which studies count most.
