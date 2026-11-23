# Trend: examples

Every example here also runs in the gallery:

```bash
cargo run --example gallery
```

Choose **Trend** in the sidebar. Not sure which example you need? Start from the [guide](guide.md).

Each example is a component; render it anywhere with `rsx! { <Basic /> }`.

The examples share these imports and this sample data:

```rust
use std::sync::LazyLock;

use blenderui::prelude::*;
use bui_charts::*;

#[derive(Clone, PartialEq)]
struct Pull {
    rpm: f64,
    torque: f64,
    power: f64,
    boost: f64,
}

fn dyno() -> Vec<Pull> {
    (0..=24)
        .map(|step| {
            let rpm = f64::from(step).mul_add(250., 1000.);
            let t = (rpm - 1000.) / 6000.;
            let torque = 220. + 180. * (std::f64::consts::PI * t.powf(0.8)).sin();
            Pull {
                rpm,
                torque,
                power: torque * rpm / 7127.,
                boost: (t * 2.4).min(1.35),
            }
        })
        .collect()
}

fn mix() -> Frame<4> {
    Frame::from_columns([
        (
            "Distance",
            [0., 5., 10., 15., 20., 25., 30., 35., 40.].as_slice(),
        ),
        (
            "Cars",
            [12., 18., 25., 31., 27., 22., 30., 36., 41.].as_slice(),
        ),
        ("Bikes", [4., 6., 9., 7., 11., 14., 12., 9., 8.].as_slice()),
        ("Walkers", [2., 3., 5., 6., 5., 4., 6., 7., 5.].as_slice()),
    ])
}

fn latency() -> Frame<4> {
    Frame::from_columns([
        (
            "Load",
            [10., 20., 30., 40., 50., 60., 70., 80., 90., 100.].as_slice(),
        ),
        (
            "p50",
            [11., 12., 12., 14., 15., 18., 21., 25., 33., 41.].as_slice(),
        ),
        (
            "min",
            [6., 7., 7., 8., 9., 10., 11., 12., 14., 15.].as_slice(),
        ),
        (
            "max",
            [19., 24., 22., 29., 31., 38., 47., 58., 71., 96.].as_slice(),
        ),
    ])
}

fn gappy() -> Frame<3> {
    let nan = f64::NAN;
    Frame::from_columns([
        (
            "x",
            [0., 1., 2., 3., 4., 5., 6., 7., 8., 12., 13., 14.].as_slice(),
        ),
        (
            "Gaps",
            [3., 4., nan, 6., 5., nan, nan, 7., 8., 6., 7., 9.].as_slice(),
        ),
        (
            "Spanned",
            [1., 2., nan, 3., 2., nan, nan, 4., 3., 2., 3., 4.].as_slice(),
        ),
    ])
}

static LIMITS: LazyLock<[(f64, Hsla); 3]> = LazyLock::new(|| {
    [
        (f64::NEG_INFINITY, rgb(0x40a02b)),
        (300., rgb(0xdf8e1d)),
        (380., rgb(0xd20f39)),
    ]
});
```

## 1. Basic trend

Torque against engine speed: x must only go up.

```rust
#[component]
fn basic() {
    let data = use_signal(dyno());
    rsx! {
        <Trend data={data} x={|p: &Pull| p.rpm} class="h-72">
            <TrendSeries name="Torque" y={|p: &Pull| p.torque} />
        </Trend>
    }
}
```

## 2. From a frame

No series: the first field is x, the rest are lines.

```rust
#[component]
fn auto_fields() {
    let data = use_signal(mix());
    rsx! { <Trend data={data} class="h-72" /> }
}
```

## 3. Smooth with fill

Smooth interpolation and an opacity gradient fill.

```rust
#[component]
fn smooth_fill() {
    let data = use_signal(dyno());
    rsx! {
        <Trend data={data} x={|p: &Pull| p.rpm} interpolation={Interpolation::Smooth} line_width={2.} fill_opacity={0.4} gradient={GradientMode::Opacity} class="h-72">
            <TrendSeries name="Power" y={|p: &Pull| p.power} />
        </Trend>
    }
}
```

## 4. Steps

Step after and step before.

```rust
#[component]
fn steps() {
    let data = use_signal(dyno());
    rsx! {
        <Trend data={data} x={|p: &Pull| p.rpm} interpolation={Interpolation::StepAfter} line_width={2.} class="h-72">
            <TrendSeries name="Boost" y={|p: &Pull| p.boost} />
            <TrendSeries name="Boost (before)" y={|p: &Pull| p.boost - 0.2} interpolation={Interpolation::StepBefore} line_style={LineStyle::DASH} />
        </Trend>
    }
}
```

## 5. Bars

Draw style bars, 80% of the column width.

```rust
#[component]
fn bars() {
    let data = use_signal(mix());
    rsx! { <Trend data={data} draw_style={DrawStyle::Bars} fill_opacity={0.7} bar_width_factor={0.8} class="h-72" /> }
}
```

## 6. Points

Draw style points.

```rust
#[component]
fn points() {
    let data = use_signal(dyno());
    rsx! {
        <Trend data={data} x={|p: &Pull| p.rpm} draw_style={DrawStyle::Points} point_size={8.} class="h-72">
            <TrendSeries name="Torque" y={|p: &Pull| p.torque} />
        </Trend>
    }
}
```

## 7. Stacked

Normal stacking with filled areas.

```rust
#[component]
fn stacked() {
    let data = use_signal(mix());
    rsx! { <Trend data={data} stacking={Stacking::NORMAL} fill_opacity={0.5} line_width={1.} class="h-72" /> }
}
```

## 8. Percent stacking

Each series as a share of the total, with a multi tooltip.

```rust
#[component]
fn percent() {
    let data = use_signal(mix());
    rsx! {
        <Trend data={data} stacking={Stacking::PERCENT} fill_opacity={0.8} interpolation={Interpolation::Smooth} tooltip={Tooltip::multi()} class="h-72" />
    }
}
```

## 9. Two axes

A second y axis on the right, coloured like its series.

```rust
#[component]
fn two_axes() {
    let data = use_signal(dyno());
    rsx! {
        <Trend data={data} x={|p: &Pull| p.rpm} y_axis={AxisConfig::new().label("Nm")} x_axis={AxisConfig::new().label("RPM")} line_width={2.} tooltip={Tooltip::multi()} class="h-72">
            <TrendSeries name="Torque" y={|p: &Pull| p.torque} />
            <TrendSeries name="Power" y={|p: &Pull| p.power} y_axis={AxisConfig::new().label("hp").color_mode(AxisColorMode::Series)} />
        </Trend>
    }
}
```

## 10. Thresholds

Dashed threshold lines over shaded threshold areas.

```rust
#[component]
fn threshold_lines() {
    let data = use_signal(dyno());
    rsx! {
        <Trend data={data} x={|p: &Pull| p.rpm} thresholds={Thresholds::absolute(&LIMITS[..])} thresholds_style={ThresholdsStyle::DashedAndArea} line_width={2.} class="h-72">
            <TrendSeries name="Torque" y={|p: &Pull| p.torque} />
        </Trend>
    }
}
```

## 11. Colour by thresholds

Gradient mode scheme: the line and fill change colour with the value.

```rust
#[component]
fn scheme_gradient() {
    let data = use_signal(dyno());
    rsx! {
        <Trend data={data} x={|p: &Pull| p.rpm} thresholds={Thresholds::absolute(&LIMITS[..])} gradient={GradientMode::Scheme} fill_opacity={0.3} line_width={3.} class="h-72">
            <TrendSeries name="Torque" y={|p: &Pull| p.torque} />
        </Trend>
    }
}
```

## 12. Hue gradient

Gradient mode hue.

```rust
#[component]
fn hue_gradient() {
    let data = use_signal(dyno());
    rsx! {
        <Trend data={data} x={|p: &Pull| p.rpm} gradient={GradientMode::Hue} fill_opacity={0.6} line_width={2.} interpolation={Interpolation::Smooth} class="h-72">
            <TrendSeries name="Power" y={|p: &Pull| p.power} color={rgb(0x8839ef)} />
        </Trend>
    }
}
```

## 13. Missing values

Gaps at missing values or wide x jumps, or spanned.

```rust
#[component]
fn nulls() {
    let data = use_signal(gappy());
    rsx! {
        <Trend data={data} line_width={2.} insert_nulls={3.} class="h-72">
            <TrendSeries name="Gaps" y={|r: &[f64; 3]| r[1]} />
            <TrendSeries name="Spanned" y={|r: &[f64; 3]| r[2]} span_nulls={NullSpan::Always} />
        </Trend>
    }
}
```

## 14. Min / max band

Fill below to: a band between two series.

```rust
#[component]
fn band() {
    let data = use_signal(latency());
    rsx! {
        <Trend data={data} tooltip={Tooltip::multi().sort(SortOrder::Descending)} class="h-72">
            <TrendSeries name="max" y={|r: &[f64; 4]| r[3]} line_width={0.} fill_below_to="min" />
            <TrendSeries name="p50" y={|r: &[f64; 4]| r[1]} line_width={2.} />
            <TrendSeries name="min" y={|r: &[f64; 4]| r[2]} line_width={0.} />
        </Trend>
    }
}
```

## 15. Negative Y and constant

One series flipped below zero, one drawn as a constant.

```rust
#[component]
fn mirrored() {
    let data = use_signal(mix());
    rsx! {
        <Trend data={data} fill_opacity={0.3} class="h-72">
            <TrendSeries name="Cars" y={|r: &[f64; 4]| r[1]} />
            <TrendSeries name="Bikes (down)" y={|r: &[f64; 4]| r[2]} transform={ValueTransform::NegativeY} />
            <TrendSeries name="Average" y={|_: &[f64; 4]| 24.} transform={ValueTransform::Constant} line_style={LineStyle::DOT} line_width={2.} fill_opacity={0.} />
        </Trend>
    }
}
```

## 16. Legend table and values

Calcs in a table legend, values above every point.

```rust
#[component]
fn legend_table() {
    let data = use_signal(mix());
    rsx! {
        <Trend
            data={data}
            show_points={PointsMode::Always}
            show_values={true}
            legend={Legend::table().right().calcs([Calc::Min, Calc::Mean, Calc::Max])}
            tooltip={Tooltip::multi().hide_zeros(true)}
            class="h-72"
        />
    }
}
```

## 17. Live data

A signal as data: the chart redraws when it changes.

```rust
#[component]
fn live() {
    let data = use_signal(dyno());
    rsx! {
        <div class="flex flex-col gap-2 h-72">
            <div id="tune" class="cursor-pointer text-sm underline" onClick={move || data.update(|rows| rows.iter_mut().for_each(|p| p.torque *= 1.05))}>
                "Add 5% torque"
            </div>
            <Trend data={data} x={|p: &Pull| p.rpm} line_width={2.} class="flex-1">
                <TrendSeries name="Torque" y={|p: &Pull| p.torque} />
            </Trend>
        </div>
    }
}
```

## 18. Unsorted x

What you see when x is not in ascending order.

```rust
#[component]
fn unsorted() {
    let data = use_signal(vec![(3., 1.), (1., 2.), (2., 3.)]);
    rsx! {
        <Trend data={data} x={|p: &(f64, f64)| p.0} class="h-40">
            <TrendSeries y={|p: &(f64, f64)| p.1} />
        </Trend>
    }
}
```

