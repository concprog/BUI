# XY Chart: API reference

`<XYChart>` plots numeric x/y pairs as points, lines, or both. A third value can drive point size, and another can drive point colour. If you have used Grafana's XY Chart panel, every option there has a counterpart here; see [Coming from Grafana](#coming-from-grafana).

```rust
use bui_charts::*;
```

- [`<XYChart>`](#xychart)
- [`<Series>`](#series)
- [Style props](#style-props)
- [`AxisConfig`](#axisconfig)
- [`Legend`](#legend)
- [`Tooltip`](#tooltip)
- [`Color` and `Scheme`](#color-and-scheme)
- [Data](#data)
- [Enums](#enums)
- [Coming from Grafana](#coming-from-grafana)

Not sure which option fits? See the [guide](guide.md).

---

## `<XYChart>`

| Prop | Type | Default |
| --- | --- | --- |
| `data` | `Vec<T>`, `Signal<Vec<T>>`, `Frame<N>`, `Signal<Frame<N>>`, or your own [`Dataset`](#your-own-data-source) | **required** |
| children | `<Series>` tags | none: series are created from `data` |
| `x_axis` | [`AxisConfig`](#axisconfig) | `AxisConfig::new()` |
| `y_axis` | [`AxisConfig`](#axisconfig) | `AxisConfig::new()` |
| `legend` | [`Legend`](#legend) | `Legend::list()` at the bottom |
| `tooltip` | [`Tooltip`](#tooltip) | `Tooltip::single()` |
| `class` | Tailwind classes | fills its parent |
| `key` | anything `Hash` (taken by `rsx!`: hook state follows it) | – |
| [style props](#style-props) | | defaults for every series |

**Where the series come from**

- **With `<Series>` children**, each child is one series.
- **Without children**, the series come from a [`Frame`](#framen): the first field is x and every other field is a y series. A `Vec<T>` has no field names, so a chart with no children and a `Vec<T>` shows **"No data"**.

The chart remembers which series you hid in the legend and which point is hovered. Give each chart a `key` when you render several of them in a loop, so that state stays with the right one.

## `<Series>`

| Prop | Type | Default |
| --- | --- | --- |
| `x` | `Fn(&T) -> f64` | **required** |
| `y` | `Fn(&T) -> f64` | **required** |
| `name` | text | `"Series 1"`, `"Series 2"`, … |
| `size` | `Fn(&T) -> f64` | every point uses `point_size` |
| `color` | [`Color<T>`](#color-and-scheme), or any `Hsla` / `Rgba` | the next colour from the theme palette |
| `key` | anything `Hash` (taken by `rsx!`: hook state follows it) | – |
| [style props](#style-props) | | inherited from `<XYChart>` |

Return `f64::NAN` from `x` or `y` to leave a row out.

Write the row type on the first closure of each tag. The other closures on that tag pick it up:

```rust
<Series x={|r: &Reading| r.temperature} y={|r| r.load} />
```

## Style props

`<XYChart>` and `<Series>` both take these props. A value on the series wins over the chart's value, and the chart's value wins over the default.

| Prop | Type | Default | Notes |
| --- | --- | --- | --- |
| `show` | [`Show`](#enums) | `Points` | points, lines, or both |
| `point_size` | `f32` px | `5.` | used when the series has no `size` |
| `size_range` | `RangeInclusive<f32>` px | `5.0..=100.0` | smallest and largest point, when the series has a `size` |
| `point_shape` | [`PointShape`](#enums) | `Circle` | |
| `point_stroke_width` | `f32` px | `1.` | outline around each point |
| `fill_opacity` | `f32`, `0.0`–`1.0` | `0.5` | colours with their own alpha keep it |
| `line_width` | `f32` px | `2.` | |
| `line_style` | [`LineStyle`](#enums) | `Solid` | |
| `hide_from` | [`HideFrom`](#enums) | nothing hidden | hide a series from the legend, the tooltip, or the chart |

Point size grows **by area**: a value twice as large gets a point with twice the area, not twice the diameter. The smallest and largest size values across all series map to the two ends of `size_range`.

## `AxisConfig`

```rust
AxisConfig::new()
    .label("Load")
    .placement(AxisPlacement::Right)
    .soft_min(0.)
    .scale(ScaleKind::Log { base: 10. })
```

| Method | Argument | Default |
| --- | --- | --- |
| `label` | text | worked out from field names (see below) |
| `placement` | [`AxisPlacement`](#enums) | x at the bottom, y on the left |
| `width` | `Pixels` | fits the widest tick label |
| `soft_min` / `soft_max` | `f64` | none |
| `min` / `max` | `f64` | none |
| `scale` | [`ScaleKind`](#enums) | `Linear` |
| `grid` | `bool` | `true` |
| `border` | `bool` | `false` |
| `centered_zero` | `bool` | `false` |
| `color_mode` | [`AxisColorMode`](#enums) | `Text` |
| `decimals` | `usize` | chosen from the tick spacing |
| `format` | `\|value\| -> SharedString` closure | none |
| `time` | – : values are milliseconds since the Unix epoch, shown as dates and times in UTC | off |
| `utc_offset` | `i32` minutes | `0` |
| `reverse` | `bool`: turns the axis around | `false` |

**The axis range** starts at the data's extent, rounded out to tidy tick values (whole powers for a log scale).
- `soft_min` / `soft_max` widen the range when the data doesn't reach them, but never narrow it.
- `centered_zero` makes the range symmetric around 0.
- `min` / `max` pin the range exactly, and data outside it is cut off.

A log scale ignores zero and negative values. With no data the range is `0` to `1`.

**The label**, when you don't set one, is the field name if every series shares it, or the words common to the start and end of the field names. Only `Frame` data has field names.

**Tick and tooltip text** uses `format` if you set it. Otherwise it uses `decimals`. Otherwise the decimals are chosen from the tick spacing. `format` takes a closure that doesn't capture anything; end it with `.into()`:

```rust
AxisConfig::new().format(|v| format!("{v:.0} °C").into())
```

## `Legend`

```rust
Legend::table().right().width(px(240.)).calcs([Calc::Mean, Calc::Max]).sort_by(LegendSort::Calc(Calc::Mean)).sort_desc(true)
```

| Method | Argument | Default |
| --- | --- | --- |
| `Legend::list()` / `table()` / `hidden()` | – | `list` |
| `bottom()` / `right()` / `placement` | [`LegendPlacement`](#enums) | `Bottom` |
| `width` | `Pixels` | fits its content (right placement only) |
| `calcs` | a list of [`Calc`](#calcs) | none |
| `sort_by` | `LegendSort::Name` or `LegendSort::Calc(..)` | series order |
| `sort_desc` | `bool` | `false` |
| `limit` | `usize` | no limit |
| `wrap` | `bool` | `false`: long names are cut off with an ellipsis |
| `mode` | [`LegendMode`](#enums) | `List` |

**Clicking** a legend item shows only that series; click it again to show them all. **Ctrl-click** (**Cmd-click** on macOS) shows or hides a single series. Hidden series stay in the legend, dimmed. The axes rescale to the series still shown, while colours and point sizes stay the same.

### Calcs

Calcs summarise each series' **y** values, skipping `NaN`. They always appear in this order, whatever order you list them in:

| `Calc` | Shown as | Meaning |
| --- | --- | --- |
| `First` / `Last` | First / Last | first / last value |
| `Min` / `Max` | Min / Max | smallest / largest |
| `Mean` | Mean | average |
| `Median` | Median | middle value |
| `Sum` | Total | sum |
| `Count` | Count | number of values |
| `Range` | Range | `Max - Min` |
| `Diff` | Difference | `Last - First` |
| `Variance` / `StdDev` | Variance / StdDev | spread of the values |

## `Tooltip`

| Method | Argument | Default |
| --- | --- | --- |
| `Tooltip::single()` / `hidden()` | – | `single` |
| `max_width` | `Pixels` | none |
| `max_height` | `Pixels` | none |

The same `Tooltip` type is shared with [`<Trend>`](../trend/api.md#tooltip-legend-and-keyboard), which also uses `Tooltip::multi()`, `sort` and `hide_zeros`. The XY Chart shows `multi()` like `single()`, as Grafana's XY Chart does.

Hovering a point highlights it and shows a tooltip next to it. The tooltip has:

- the series name and the point's colour;
- the x and y values, each labelled with its field name or axis label;
- the size and colour values, when the series has them.

Series with `hide_from.tooltip` are still highlighted on hover, but show no tooltip. Points from series drawn only as lines can't be hovered.

## `Color` and `Scheme`

```rust
color={rgb(0x89b4fa)}
color={Color::by(|r: &Reading| r.memory, Scheme::GreenYellowRed)}
color={Color::by(|r: &Reading| r.load, Scheme::thresholds(vec![(0., green), (80., red)]))}
```

| `Color<T>` | Meaning |
| --- | --- |
| `Color::fixed(color)`, or any `Hsla` / `Rgba` | one colour for the whole series |
| `Color::by(value, scheme)` | each point is coloured by its value, through `scheme` |

| `Scheme` | Meaning |
| --- | --- |
| `GreenYellowRed`, `RedYellowGreen`, `BlueYellowRed`, `YellowRed`, `BluePurple`, `YellowBlue`, `Blues`, `Reds`, `Greens`, `Purples`, `Oranges` | gradients from the [theme](../theming.md) |
| `Named("…")` | a gradient you added to your theme file |
| `Scheme::gradient(colors)` | your own gradient |
| `Scheme::thresholds([(value, color), …])` | fixed steps: a value takes the colour of the highest threshold at or below it, and values below the first threshold take the first colour |

`gradient` and `thresholds` take a `Vec` or a `&'static` slice. Prefer a static slice: it costs nothing to rebuild on every render. See [Static colour lists](guide.md#static-colour-lists).

A gradient spans the smallest to the largest colour value **across all series** that colour by value, so equal values look the same in every series. Points with a `NaN` colour value use the theme's fallback colour.

## Data

### Which data type

| `data` | Use it when | Recomputed |
| --- | --- | --- |
| `Vec<T>` | the data is fixed, or comes from props | on every render |
| `Signal<Vec<T>>` | the data lives in your component's state or changes over time | only when the signal is written |
| `Frame<N>` | the data is a table of named numeric fields | on every render |
| `Signal<Frame<N>>` | a table that lives in state or streams in | only when the signal is written |

See [Pass data as a signal](guide.md#pass-data-as-a-signal).

### `Frame<N>`

A table with `N` named numeric fields, one row per `[f64; N]`.

| Method | Meaning |
| --- | --- |
| `Frame::new(names)` | an empty frame with `N` field names |
| `Frame::from_columns([(name, values), …])` | a frame built from columns; short columns are padded with `NaN` |
| `.with_rows(rows)` | sets the rows, from a `Vec<[f64; N]>` or an array of them |
| `.push(row)` | appends a row |
| `.size(field)` | without `<Series>`, sizes every series' points by this field |
| `.color(field, scheme)` | without `<Series>`, colours every series' points by this field |
| `.index(field)` | the position of a field |
| `.get(field)` | a value accessor for `<Series>`, found by field name |

Without `<Series>`, the size and colour fields aren't drawn as series of their own. Words shared by every series name are trimmed: `cpu load host-a` and `cpu load host-b` become `host-a` and `host-b`, and the y axis is labelled `cpu load`.

With `<Series>`, read fields by position (`x={|r: &[f64; 3]| r[1]}`) or by name (`x={frame.get("Requests")}`).

### Your own data source

Implement `Dataset<T>` to chart data stored some other way:

```rust
pub trait Dataset<T>: 'static {
    // the rows (and the schema, for named fields), lent for the time of `f`
    fn read<R>(&self, f: impl FnOnce(&[T], Option<Schema<'_, T>>) -> R) -> R;
    #[track_caller]
    fn revision(&self) -> Option<Revision> { None }
    // provided: `rows(|rows| ..)` and `schema(|schema| ..)`, both through `read`
}

pub struct Schema<'a, T> {
    pub names: &'a [SharedString],
    pub read: fn(&T, usize) -> f64,
    pub size: Option<usize>,
    pub color: Option<(usize, &'a Scheme)>,
}
```

| Method | Implement it to |
| --- | --- |
| `read` | call `f` with the rows and, when the data names its fields, a schema. A signal's value lives in blenderui's signal store, so the rows are lent for the call rather than returned; don't read the same dataset again inside `f`. |
| `revision` | return a value that changes whenever the rows change. While it stays the same, the chart reuses its work; `None` derives again on every render. Charts call it while rendering. `Revision::of(signal)` builds one from a blenderui signal: a memo at the caller notices every write to it. |
| `schema` (in `read`) | describe named numeric fields, so charts can build series (XY Chart) or bars (Bar Gauge) without accessors. `read(row, field)` returns one value; `size` and `color` pick fields for the XY Chart's auto mapping. |

`Vec<T>`, `Signal<Vec<T>>`, `Frame<N>` and `Signal<Frame<N>>` implement it. A plain `Vec` or `Frame` has no revision, so keep chart data in a `use_signal`.

## Enums

| Enum | Variants |
| --- | --- |
| `Show` | `Points` (default), `Lines`, `PointsAndLines` |
| `PointShape` | `Circle` (default), `Square` |
| `LineStyle` | `Solid` (default), `Dash(&[on, off, …])`, `Dot(gap)`; ready-made `LineStyle::DASH` and `LineStyle::DOT` |
| `HideFrom` | `{ legend, tooltip, viz }`, all `bool` |
| `ScaleKind` | `Linear` (default), `Log { base }`, `Symlog { base, linear_threshold }` |
| `AxisPlacement` | `Auto` (default), `Top`, `Right`, `Bottom`, `Left`, `Hidden` |
| `AxisColorMode` | `Text` (default), `Series` (tick labels use the first series' colour) |
| `LegendMode` | `List` (default), `Table`, `Hidden` |
| `LegendPlacement` | `Bottom` (default), `Right` |
| `LegendSort` | `Name`, `Calc(Calc)` |
| `TooltipMode` | `Single` (default), `Multi` (shown like `Single`), `Hidden` |

`Dot` draws small square dots along the line.

## Coming from Grafana

| Grafana option | Here |
| --- | --- |
| Series mapping: Auto | no `<Series>` children, with a `Frame` |
| Series mapping: Manual | `<Series>` children |
| Series: X / Y / Size / Color field | `<Series x y size color>` |
| Series name | `<Series name>` |
| Legend | `Legend` |
| Tooltip | `Tooltip` |
| Field defaults (show, point size, line style, …) | style props on `<XYChart>` |
| Field overrides | style props on `<Series>` |
| Axis options, scale distribution | `AxisConfig` |
| Min, Max, Decimals | `AxisConfig::min`, `max`, `decimals` |
| Unit | `AxisConfig::format` |
| Color scheme | `Color` and `Scheme`, with the palette from your [theme](../theming.md) |

Not available here: several data frames in one chart (combine your rows first), separate y axes per unit, value mappings, data links, the tooltip's sort and hide-zeros options, and ordinal axes. `fill_opacity` goes from `0.0` to `1.0` rather than `0` to `100`.
