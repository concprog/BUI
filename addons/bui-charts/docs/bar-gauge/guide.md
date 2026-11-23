# Bar Gauge: choosing options

How to pick between the options in the [API reference](api.md). Every link goes to a runnable example; the gallery has them all (`cargo run --example gallery`, then **Bar Gauge**).

## Is a bar gauge the right chart?

Use a bar gauge when each value has a **known range** and you want to see how full it is: disk usage, quotas, SLO budgets, queue depth against capacity. If the values have no natural maximum, or you're comparing their sizes rather than how full they are, use a bar chart. If you want to see a value change over time, use an XY chart.

## Gradient, LCD or basic

| You want | Use |
| --- | --- |
| a smooth bar that shows which thresholds it passed | `BarDisplay::Gradient` (the default) |
| a dashboard look where it's easy to count how far a bar is filled | `BarDisplay::Lcd` |
| a quiet bar that doesn't compete with other panels | `BarDisplay::Basic` |

Examples: [Disk usage](examples.md#1-disk-usage), [Retro LCD](examples.md#2-retro-lcd), [Basic](examples.md#3-basic).

## Rows or columns

`BarOrientation::Auto` gives columns in wide spaces and rows in tall ones. Set it yourself when:

- names are long: use `Horizontal` with `name_placement={NamePlacement::Left}` so names get their own column;
- there are many bars and little height: use `Vertical`;
- rows should stay thin however much space there is: use `Sizing::Manual { .. }` with a small `max_height`.

Examples: [Columns](examples.md#4-columns), [Names on the left](examples.md#10-names-on-the-left), [Manual sizing](examples.md#11-manual-sizing).

## Where values come from

| Your data | Use |
| --- | --- |
| a list of things, each with a current value | rows with `name` and `value` |
| a table of series over time where you want the latest of each | a `Frame` (one bar per field, last value) |
| the average or peak of each series | a `Frame` with `reduce={Reduce::Calc(Calc::Mean)}` |
| only some of the series | `fields={&[..]}` |

Examples: [Reduce a frame](examples.md#5-reduce-a-frame), [Pick fields](examples.md#6-pick-fields).

## Colours

| You want | Use |
| --- | --- |
| green, then red past a limit | the default thresholds (your theme's good colour, then its bad colour from 80) |
| your own limits | `thresholds={Thresholds::absolute(..)}` |
| limits that follow `min`/`max` when they change | `Thresholds::percentage(..)` |
| a continuous scale | `color={GaugeColor::Scheme(..)}` |
| bars told apart rather than judged | `GaugeColor::Palette` |
| one brand colour | `GaugeColor::Fixed(..)` |

Build threshold and colour lists once, in a `static` or `const`, rather than with `vec![..]` in your component: see [static colour lists](../xy-chart/guide.md#static-colour-lists).

Examples: [Percentage thresholds](examples.md#7-percentage-thresholds), [Colour scheme](examples.md#8-colour-scheme), [Palette colours](examples.md#9-palette-colours).

## Text

- `value_mode={ValueMode::Text}` keeps numbers readable when bar colours are pale.
- `text_mode={TextMode::Value}` hides names when a legend or the surrounding layout already says what each bar is.
- `format` adds units; `decimals` keeps numbers the same width.

## Performance

Pass data as a signal (`use_signal`) so the bars, names and value text are only rebuilt when the data changes. The `name` accessor returns a borrowed `&str`, so it never clones.
