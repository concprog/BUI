# XY Chart: examples

Every example here also runs in the gallery:

```bash
cargo run --example gallery
```

Choose **XY Chart** in the sidebar to see them all side by side. Not sure which example you need? Start from the [guide](guide.md).

The examples share these imports and this sample data:

```rust
use std::sync::LazyLock;

use blenderui::prelude::*;
use bui_charts::*;

#[derive(Clone, PartialEq)]
struct Reading {
    temperature: f64,
    load: f64,
    memory: f64,
    humidity: f64,
}

fn readings() -> Vec<Reading> {
    (0..200)
        .map(|i| {
            let t = f64::from(i);
            Reading {
                temperature: 15. + t % 25.,
                load: t % 90.,
                memory: t % 70.,
                humidity: 30. + t % 50.,
            }
        })
        .collect()
}

fn curve(f: fn(f64) -> f64, range: std::ops::RangeInclusive<i32>) -> Vec<(f64, f64)> {
    range.map(|i| (f64::from(i), f(f64::from(i)))).collect()
}
```

| # | Example | Shows |
| --- | --- | --- |
| 1 | [Basic scatter](#1-basic-scatter) | the smallest chart |
| 2 | [Lines and points](#2-lines-and-points) | `show`, `line_style`, several series |
| 3 | [Bubbles](#3-bubbles) | `size`, `size_range`, `fill_opacity` |
| 4 | [Colour by value](#4-colour-by-value) | gradients and thresholds |
| 5 | [Live data](#5-live-data) | updating a signal |
| 6 | [Auto mapping](#6-auto-mapping) | a `Frame` without `<Series>` |
| 7 | [Frame roles](#7-frame-roles) | size and colour fields |
| 8 | [Manual series over a frame](#8-manual-series-over-a-frame) | picking frame fields |
| 9 | [Log scale](#9-log-scale) | log axis, soft and hard limits |
| 10 | [Symlog on the right](#10-symlog-on-the-right) | symlog, centered zero, coloured ticks |
| 11 | [Table legend](#11-table-legend) | calcs, sorting, right placement |
| 12 | [Tooltip and hiding](#12-tooltip-and-hiding) | `Tooltip`, `HideFrom` |
| 13 | [Sparkline](#13-sparkline) | a minimal inline chart |
| 14 | [Formatting](#14-formatting) | `format`, `decimals`, `min`/`max` |
| 15 | [Series from a loop](#15-series-from-a-loop) | `for` inside `<XYChart>` |
| 16 | [Shared defaults](#16-shared-defaults) | chart defaults, series overrides |
| 17 | [Custom gradient](#17-custom-gradient) | your own colour list |

---

## 1. Basic scatter

One series of points. Everything else uses its defaults: 5px circles at half opacity, a legend at the bottom, and a tooltip on hover.

```rust
#[component]
fn basic_scatter() {
    let data = use_signal(readings());
    rsx! {
        <XYChart data={data} class="h-72">
            <Series name="Load" x={|r: &Reading| r.temperature} y={|r| r.load} />
        </XYChart>
    }
}
```

## 2. Lines and points

`show` is set once on the chart and overridden on one series. Lines join rows in the order you give them.

```rust
#[component]
fn lines_and_points() {
    let data = use_signal(curve(|x| x.sqrt() * 10., 0..=49));
    rsx! {
        <XYChart data={data} show={Show::PointsAndLines} point_size={4.} line_width={1.5} class="h-72">
            <Series name="√x" x={|p: &(f64, f64)| p.0} y={|p| p.1} />
            <Series name="√x · 0.5" x={|p: &(f64, f64)| p.0} y={|p| p.1 * 0.5} line_style={LineStyle::DASH} />
            <Series name="√x · 0.25" x={|p: &(f64, f64)| p.0} y={|p| p.1 * 0.25} show={Show::Lines} line_style={LineStyle::DOT} />
        </XYChart>
    }
}
```

## 3. Bubbles

`size` turns a value into point size, scaled by area, between the two ends of `size_range`. A lower `fill_opacity` keeps overlapping bubbles readable.

```rust
#[component]
fn bubbles() {
    let data = use_signal(readings());
    rsx! {
        <XYChart data={data} size_range={4.0..=32.0} fill_opacity={0.35} class="h-72">
            <Series name="Load" x={|r: &Reading| r.temperature} y={|r| r.load} size={|r| r.humidity} />
        </XYChart>
    }
}
```

## 4. Colour by value

A gradient for a continuous value, and thresholds for fixed limits. The threshold list is built once and reused, see [static colour lists](guide.md#static-colour-lists).

```rust
static THRESHOLDS: LazyLock<[(f64, Hsla); 3]> = LazyLock::new(|| {
    [
        (0., rgb(0x40a02b)),
        (50., rgb(0xdf8e1d)),
        (80., rgb(0xd20f39)),
    ]
});

#[component]
fn color_by_value() {
    let data = use_signal(readings());
    rsx! {
        <XYChart data={data} class="h-72">
            <Series name="Gradient" x={|r: &Reading| r.temperature} y={|r| r.load}
                    color={Color::by(|r: &Reading| r.memory, Scheme::BlueYellowRed)} />
            <Series name="Thresholds" x={|r: &Reading| r.temperature} y={|r| r.load + 100.}
                    color={Color::by(|r: &Reading| r.load, Scheme::thresholds(&THRESHOLDS[..]))}
                    point_shape={PointShape::Square} />
        </XYChart>
    }
}
```

## 5. Live data

Pass the signal itself. Updating it in place redraws the chart.

```rust
#[component]
fn live_signal() {
    let data = use_signal(readings());
    rsx! {
        <div class="flex flex-col gap-2 h-72">
            <div class="cursor-pointer text-sm underline" onClick={move || data.update(|rows| rows.iter_mut().for_each(|r| r.load = (r.load * 1.7) % 90.))}>
                "Shuffle"
            </div>
            <XYChart data={data} class="flex-1">
                <Series name="Load" x={|r: &Reading| r.temperature} y={|r| r.load} />
            </XYChart>
        </div>
    }
}
```

## 6. Auto mapping

With no `<Series>`, a `Frame` becomes the series: its first field is x and the rest are y. The shared word `cpu` is trimmed from the series names and becomes the y-axis label.

```rust
fn waves() -> Frame<3> {
    let rows: Vec<[f64; 3]> = (0..60)
        .map(|t| {
            let t = f64::from(t);
            [
                t,
                (t / 5.).sin().mul_add(40., 50.),
                (t / 5.).cos().mul_add(40., 50.),
            ]
        })
        .collect();
    Frame::new(["Time", "cpu host-a", "cpu host-b"]).with_rows(rows)
}

#[component]
fn auto_mapping() {
    let frame = use_signal(waves());
    rsx! { <XYChart data={frame} show={Show::PointsAndLines} class="h-72" /> }
}
```

## 7. Frame roles

`size` and `color` pick fields that drive every series. Those fields aren't drawn as series of their own. Use `from_columns` when your data comes as columns.

```rust
fn people() -> Frame<3> {
    Frame::from_columns([
        ("Height cm", [150., 160., 170., 180., 190.]),
        ("Weight kg", [50., 58., 70., 82., 95.]),
        ("Age", [18., 25., 33., 47., 62.]),
    ])
    .size("Age")
    .color("Age", Scheme::Purples)
}

#[component]
fn frame_roles() {
    let frame = use_signal(people());
    rsx! { <XYChart data={frame} size_range={8.0..=28.0} class="h-72" /> }
}
```

## 8. Manual series over a frame

Pick fields yourself by position. `frame.get("Requests")` works too, by name.

```rust
fn traffic() -> Frame<3> {
    Frame::new(["Time", "Requests", "Errors"]).with_rows([
        [0., 120., 2.],
        [1., 340., 9.],
        [2., 290., 4.],
        [3., 410., 12.],
    ])
}

#[component]
fn frame_manual() {
    let frame = use_signal(traffic());
    rsx! {
        <XYChart data={frame} class="h-72">
            <Series name="Errors vs requests" x={|r: &[f64; 3]| r[1]} y={|r| r[2]} size={|r| r[0]} size_range={6.0..=18.0} />
        </XYChart>
    }
}
```

## 9. Log scale

Log ranges snap to whole powers. `soft_min`/`soft_max` widen the range only when the data doesn't reach them, and `min` pins it.

```rust
#[component]
fn log_scale() {
    let data = use_signal(curve(|day| 1.5f64.powf((day / 2.).floor()), 1..=40));
    rsx! {
        <XYChart
            data={data}
            show={Show::PointsAndLines}
            x_axis={AxisConfig::new().label("Day").soft_min(0.).soft_max(45.)}
            y_axis={AxisConfig::new().label("Requests").scale(ScaleKind::Log { base: 10. }).min(1.)}
            class="h-72"
        >
            <Series name="Growth" x={|p: &(f64, f64)| p.0} y={|p| p.1} />
        </XYChart>
    }
}
```

## 10. Symlog on the right

Symlog handles large magnitudes on both sides of zero. The ticks always include 0 and stay symmetric.

```rust
#[component]
fn symlog_right_axis() {
    let data = use_signal(curve(|x| x.powi(3), -30..=30));
    rsx! {
        <XYChart
            data={data}
            y_axis={AxisConfig::new()
                .scale(ScaleKind::Symlog { base: 10., linear_threshold: 10. })
                .placement(AxisPlacement::Right)
                .centered_zero(true)
                .color_mode(AxisColorMode::Series)
                .border(true)}
            legend={Legend::hidden()}
            class="h-72"
        >
            <Series name="x³" x={|p: &(f64, f64)| p.0} y={|p| p.1} color={rgb(0x8839ef)} />
        </XYChart>
    }
}
```

## 11. Table legend

A table legend on the right, sorted by mean. Click a row to show only that series, and Ctrl-click (Cmd-click on macOS) to show or hide one.

```rust
#[component]
fn table_legend() {
    let data = use_signal(readings());
    rsx! {
        <XYChart
            data={data}
            legend={Legend::table()
                .right()
                .width(px(220.))
                .calcs([Calc::Min, Calc::Mean, Calc::Max])
                .sort_by(LegendSort::Calc(Calc::Mean))
                .sort_desc(true)}
            class="h-72"
        >
            <Series name="Load" x={|r: &Reading| r.temperature} y={|r| r.load} />
            <Series name="Memory" x={|r: &Reading| r.temperature} y={|r| r.memory} />
            <Series name="Humidity" x={|r: &Reading| r.temperature} y={|r| r.humidity} />
        </XYChart>
    }
}
```

## 12. Tooltip and hiding

A baseline that's drawn but kept out of the legend and the tooltip.

```rust
#[component]
fn tooltip_and_hiding() {
    let data = use_signal(readings());
    rsx! {
        <XYChart data={data} tooltip={Tooltip::single().max_width(px(180.))} class="h-72">
            <Series name="Load" x={|r: &Reading| r.temperature} y={|r| r.load} />
            <Series name="Baseline" x={|r: &Reading| r.temperature} y={|_| 45.} show={Show::Lines}
                    hide_from={HideFrom { tooltip: true, legend: true, viz: false }} />
        </XYChart>
    }
}
```

## 13. Sparkline

Hide the axes, grid, legend and tooltip for a small inline chart.

```rust
#[component]
fn sparkline() {
    let data = use_signal(curve(|x| (x / 3.).sin(), 0..=29));
    rsx! {
        <XYChart
            data={data}
            show={Show::Lines}
            x_axis={AxisConfig::new().placement(AxisPlacement::Hidden).grid(false)}
            y_axis={AxisConfig::new().placement(AxisPlacement::Hidden).grid(false)}
            legend={Legend::hidden()}
            tooltip={Tooltip::hidden()}
            class="h-12 w-40"
        >
            <Series x={|p: &(f64, f64)| p.0} y={|p| p.1} />
        </XYChart>
    }
}
```

## 14. Formatting

`format` sets the tick and tooltip text, `decimals` fixes the precision, and `min`/`max` pin the range.

```rust
#[component]
fn formatting() {
    let data = use_signal(readings());
    rsx! {
        <XYChart
            data={data}
            x_axis={AxisConfig::new().label("Temperature").format(|v| format!("{v:.0} °C").into())}
            y_axis={AxisConfig::new().label("Load").decimals(1).min(0.).max(100.)}
            class="h-72"
        >
            <Series name="Load" x={|r: &Reading| r.temperature} y={|r| r.load} />
        </XYChart>
    }
}
```

## 15. Series from a loop

Loops work inside `<XYChart>`. Give each `<Series>` a `key`.

```rust
const HOSTS: [&str; 3] = ["host-a", "host-b", "host-c"];

#[derive(Clone, Copy, PartialEq)]
struct Row {
    time: f64,
    values: [f64; 3],
}

fn host_rows() -> Vec<Row> {
    (0..40)
        .map(|t| {
            let t = f64::from(t);
            Row {
                time: t,
                values: [0., 2., 4.].map(|phase| ((t + phase) / 4.).sin().mul_add(30., 50.)),
            }
        })
        .collect()
}

#[component]
fn series_from_a_loop() {
    let rows = use_signal(host_rows());
    rsx! {
        <XYChart data={rows} show={Show::Lines} class="h-72">
            {for (column, host) in HOSTS.into_iter().enumerate() {
                <Series key={column} name={host} x={|r: &Row| r.time} y={move |r: &Row| r.values[column]} />
            }}
        </XYChart>
    }
}
```

## 16. Shared defaults

Style props on the chart apply to every series, and any series can override them.

```rust
#[component]
fn shared_defaults() {
    let data = use_signal(readings());
    rsx! {
        <XYChart
            data={data}
            point_shape={PointShape::Square}
            point_size={6.}
            point_stroke_width={0.}
            fill_opacity={0.8}
            class="h-72"
        >
            <Series name="Squares" x={|r: &Reading| r.temperature} y={|r| r.load} />
            <Series name="Circles" x={|r: &Reading| r.temperature} y={|r| r.memory}
                    point_shape={PointShape::Circle} point_stroke_width={1.} fill_opacity={0.2} />
        </XYChart>
    }
}
```

## 17. Custom gradient

Your own gradient, from a colour list built once. To share a gradient across your app, add it to your [theme](../theming.md) and use `Scheme::Named`.

```rust
static HEAT: LazyLock<[Hsla; 3]> = LazyLock::new(|| {
    [
        rgb(0x04a5e5),
        rgb(0xdf8e1d),
        rgb(0xd20f39),
    ]
});

#[component]
fn custom_gradient() {
    let data = use_signal(readings());
    rsx! {
        <XYChart data={data} class="h-72">
            <Series name="Load" x={|r: &Reading| r.temperature} y={|r| r.load}
                    color={Color::by(|r: &Reading| r.memory, Scheme::gradient(&HEAT[..]))} />
        </XYChart>
    }
}
```

