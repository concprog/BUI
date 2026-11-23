# XY Chart: choosing options

How to pick between the options in the [API reference](api.md). Each choice links to a runnable example in the [examples](examples.md) and the gallery (`cargo run --example gallery`).

## Is an XY chart the right chart?

Use an XY chart when **both axes are numbers** and you want to see how one value relates to another: temperature against load, height against weight, requests against errors. If x is a category (team, region), use a bar chart. If x is time and the data is a single trend, a line chart is simpler.

## Points, lines or both

| You want to show | Use |
| --- | --- |
| how values cluster, outliers, correlation | `Show::Points` (the default) |
| a path or trend through ordered rows | `Show::Lines` |
| a trend where each row still matters | `Show::PointsAndLines` |

Lines connect rows **in the order you give them**, not sorted by x, so sort your rows first if you want a left-to-right line. Examples: [Lines and points](examples.md#2-lines-and-points), [Sparkline](examples.md#13-sparkline).

## Showing a third or fourth value

| You want | Use |
| --- | --- |
| a value read as "how much" | `size`. Area grows with the value, so a value twice as large looks about twice as big. Use `size_range` to keep small values visible and large ones from covering the plot. |
| a value read as "how hot / how good" | `color={Color::by(.., scheme)}` with a gradient |
| a value read as "ok / warning / critical" | `color={Color::by(.., Scheme::thresholds(..))}` |
| two extra values | both `size` and `color` on the same series |

Lower `fill_opacity` when points overlap a lot, so dense areas read darker. Examples: [Bubbles](examples.md#3-bubbles), [Colour by value](examples.md#4-colour-by-value).

### Picking a scheme

- **One-sided values** (0 = none, more = more): a single-hue scheme like `Blues` or `Purples`.
- **Good to bad**: `GreenYellowRed`, or `RedYellowGreen` when higher is better.
- **Values around a midpoint**: `BlueYellowRed`.
- **Fixed limits** (an SLO, a budget): `Scheme::thresholds`.

## `<Series>` or a `Frame`

| Your data | Use |
| --- | --- |
| your own structs (`Vec<Reading>`) | `<Series>` with closures: `x={\|r: &Reading\| r.temperature}` |
| a table of named numbers where every column after the first is a series | a `Frame` with no `<Series>`. You get series names and axis labels for free. |
| a table where you pick the columns | a `Frame` with `<Series>`: `x={\|r: &[f64; 3]\| r[1]}` |
| a series per item in a list | `<Series>` inside a `for` loop, with a `key` |

Examples: [Auto mapping](examples.md#6-auto-mapping), [Frame roles](examples.md#7-frame-roles), [Manual series over a frame](examples.md#8-manual-series-over-a-frame), [Series from a loop](examples.md#15-series-from-a-loop).

## Pass data as a signal

Keep chart data in a signal (`use_signal`) and pass the signal itself:

```rust
let data = use_signal(load_readings());
rsx! { <XYChart data={data}> .. </XYChart> }
```

With a signal, the chart does its per-row work once and then only again when you write to the signal. With a plain `Vec` or `Frame`, it redoes that work on **every** render of your component, including renders caused by unrelated state such as a hover elsewhere in the window. That's fine for a few hundred rows, but use a signal for anything larger or anything that updates.

To add data over time, change the signal in place instead of replacing it:

```rust
set_data(|rows| rows.push(next));
```

Example: [Live data](examples.md#5-live-data).

## Keep accessor closures simple

Write `x`, `y`, `size` and `color` closures that only read from the row: `|r| r.load`, not `|r| r.load * factor` where `factor` changes. With a signal as data, the chart reuses its results until the signal changes, so a value captured from elsewhere wouldn't update. If a value like that needs to change, put it in the data.

Closures that capture nothing (`|r| r.load`) are also the cheapest to create on every render.

## Static colour lists

Colour lists for `Scheme::gradient` and `Scheme::thresholds`, and dash patterns for `LineStyle::Dash`, can be built once and reused:

```rust
static STEPS: LazyLock<[(f64, Hsla); 3]> =
    LazyLock::new(|| [(0., rgb(0x40a02b)), (50., rgb(0xdf8e1d)), (80., rgb(0xd20f39))]);

color={Color::by(|r: &Reading| r.load, Scheme::thresholds(&STEPS[..]))}
```

Passing a `vec![..]` works too, but it builds a new list on every render.

## Axes

| Situation | Use |
| --- | --- |
| values span several orders of magnitude, all positive | `ScaleKind::Log { base: 10. }` |
| huge magnitudes on both sides of zero | `ScaleKind::Symlog { base: 10., linear_threshold }`; set the threshold near the smallest value you care about |
| the axis should always include a value (like 0) | `soft_min(0.)`: widens the range, never narrows it |
| the axis must be exactly a range, such as 0–100% | `min(0.).max(100.)`: data outside is cut off |
| positive and negative values compared by size | `centered_zero(true)` |
| units on the ticks | `format(\|v\| format!("{v:.0} ms").into())` |
| a fixed number of decimals | `decimals(n)` |
| several charts whose y axes should line up | the same `width` on each |

Examples: [Log scale](examples.md#9-log-scale), [Symlog on the right](examples.md#10-symlog-on-the-right), [Formatting](examples.md#14-formatting).

## Legend

| Situation | Use |
| --- | --- |
| a few series, just names | `Legend::list()` (the default) |
| comparing numbers across series | `Legend::table().calcs([..])` |
| many series, or a wide chart | `.right()` with a `width` |
| ranking series | `.sort_by(LegendSort::Calc(..)).sort_desc(true)`, plus `limit` for a top N |
| one series, or a tiny chart | `Legend::hidden()` |

Example: [Table legend](examples.md#11-table-legend).

## Tooltip and hiding

- `Tooltip::hidden()` for decorative charts like sparklines.
- `max_width` / `max_height` keep tooltips small on dense dashboards.
- `hide_from` keeps one series, such as a baseline or target line, out of the legend or the tooltip while still drawing it. `hide_from.viz` removes it from the chart entirely.

Example: [Tooltip and hiding](examples.md#12-tooltip-and-hiding).

## Big data sets

- Pass the data as a signal (see above).
- Prefer `Show::Points` over lines for tens of thousands of rows.
- Keep `point_stroke_width` at `0` and `fill_opacity` high when points are tiny.
- Several charts showing the same data can share one signal.
