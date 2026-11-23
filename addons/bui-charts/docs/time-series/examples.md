# Time Series: examples

Every example here also runs in the gallery:

```bash
cargo run --example gallery
```

Choose **Time Series** in the sidebar. Not sure which example you need? Start from the [guide](guide.md).

Each example is a component; render it anywhere with `rsx! { <Basic /> }`.

The examples share these imports and this sample data:

```rust
use std::sync::LazyLock;

use blenderui::prelude::*;
use bui_charts::*;

const T0: f64 = 1_729_000_000_000.;
const MINUTE: f64 = 60_000.;
const HOUR: f64 = 60. * MINUTE;

fn noise(seed: u64) -> f64 {
    let mut z = seed.wrapping_add(0x9e37_79b9_7f4a_7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    (z ^ (z >> 31)) as f64 / u64::MAX as f64
}

#[derive(Clone, PartialEq)]
struct Metric {
    time: f64,
    cpu: f64,
    memory: f64,
    p50: f64,
    p99: f64,
    requests: f64,
}

fn metrics(count: u64, step: f64, seed: u64) -> Vec<Metric> {
    (0..count)
        .map(|index| {
            let t = index as f64;
            let daily = (t * step / (24. * HOUR) * std::f64::consts::TAU).sin();
            let jitter = |salt: u64| noise(seed * 10_000 + index * 7 + salt);
            Metric {
                time: T0 + t * step,
                cpu: 45. + 20. * daily + 10. * jitter(1),
                memory: 55. + t * 0.05 + 5. * jitter(2),
                p50: 80. + 15. * daily + 10. * jitter(3),
                p99: 240. + 60. * daily + 80. * jitter(4),
                requests: (900. + 400. * daily + 120. * jitter(5)).round(),
            }
        })
        .collect()
}

static LOAD: LazyLock<[(f64, Hsla); 3]> = LazyLock::new(|| {
    [
        (f64::NEG_INFINITY, rgb(0x40a02b)),
        (60., rgb(0xdf8e1d)),
        (75., rgb(0xd20f39)),
    ]
});
```

## 1. Basic

Two series on a UTC time axis.

```rust
#[component]
fn basic() {
    let data = use_signal(metrics(120, MINUTE, 1));
    rsx! {
        <TimeSeries data={data} x={|m: &Metric| m.time} class="h-64">
            <TrendSeries name="CPU %" y={|m: &Metric| m.cpu} />
            <TrendSeries name="Memory %" y={|m: &Metric| m.memory} />
        </TimeSeries>
    }
}
```

## 2. Annotations

Deploy markers and an incident region; hover the markers at the bottom.

```rust
#[component]
fn deploys() {
    let data = use_signal(metrics(120, MINUTE, 2));
    let notes = vec![
        Annotation::at(T0 + 20. * MINUTE, "Deploy v1.4.2").text("Rolled out to 100% of pods"),
        Annotation::region(T0 + 55. * MINUTE, T0 + 75. * MINUTE, "Incident #218")
            .text("Elevated latency on checkout")
            .color(rgb(0xd20f39)),
        Annotation::at(T0 + 95. * MINUTE, "Deploy v1.4.3"),
    ];
    rsx! {
        <TimeSeries data={data} x={|m: &Metric| m.time} annotations={notes} class="h-64">
            <TrendSeries name="p99 ms" y={|m: &Metric| m.p99} />
        </TimeSeries>
    }
}
```

## 3. Exemplars

Individual slow requests as diamonds over the percentiles.

```rust
#[component]
fn exemplars() {
    let data = use_signal(metrics(120, MINUTE, 3));
    let slow: Vec<(f64, f64)> = (0..12u64)
        .map(|index| {
            let at = (noise(index + 50) * 119.).floor();
            (T0 + at * MINUTE, 300. + noise(index + 90) * 200.)
        })
        .collect();
    rsx! {
        <TimeSeries data={data} x={|m: &Metric| m.time} exemplars={slow} class="h-64">
            <TrendSeries name="p50 ms" y={|m: &Metric| m.p50} />
            <TrendSeries name="p99 ms" y={|m: &Metric| m.p99} />
        </TimeSeries>
    }
}
```

## 4. Drag to zoom

Drag across the time axis; double-click to zoom out.

```rust
#[component]
fn zoom() {
    let data = use_signal(metrics(240, MINUTE, 4));
    let picked = use_signal(SharedString::Borrowed(
        "Drag across the chart to zoom. Double-click to zoom out.",
    ));
    rsx! {
        <div class="flex flex-col gap-2 h-64">
            <div class="text-xs">{picked.get()}</div>
            <TimeSeries
                data={data}
                x={|m: &Metric| m.time}
                on_select={move |range: SelectedRange, _: &mut Window, _: &mut App| {
                    let minutes = (range.x.1 - range.x.0) / MINUTE;
                    picked.set(SharedString::from(format!("Zoomed to {minutes:.0} minutes")));
                }}
                class="flex-1"
            >
                <TrendSeries name="requests/min" y={|m: &Metric| m.requests} />
            </TimeSeries>
        </div>
    }
}
```

## 5. Time compare

Each series again, shifted by one day, dashed.

```rust
#[component]
fn compare() {
    let data = use_signal(metrics(96, HOUR, 5));
    rsx! {
        <TimeSeries data={data} x={|m: &Metric| m.time} time_compare={24. * HOUR} class="h-64">
            <TrendSeries name="requests/min" y={|m: &Metric| m.requests} />
        </TimeSeries>
    }
}
```

## 6. Several timezones

One time axis per timezone: UTC, UTC+5:30 and UTC−5.

```rust
#[component]
fn timezones() {
    let data = use_signal(metrics(48, 30. * MINUTE, 6));
    rsx! {
        <TimeSeries data={data} x={|m: &Metric| m.time} timezones={&[0, 330, -300]} class="h-64">
            <TrendSeries name="CPU %" y={|m: &Metric| m.cpu} />
        </TimeSeries>
    }
}
```

## 7. Vertical

orientation Vertical: time runs down the chart.

```rust
#[component]
fn vertical() {
    let data = use_signal(metrics(60, MINUTE, 7));
    rsx! {
        <TimeSeries data={data} x={|m: &Metric| m.time} orientation={VizOrientation::Vertical} fill_opacity={0.15} class="h-96">
            <TrendSeries name="p50 ms" y={|m: &Metric| m.p50} />
            <TrendSeries name="p99 ms" y={|m: &Metric| m.p99} />
        </TimeSeries>
    }
}
```

## 8. Stacked areas

Two series stacked with a filled area.

```rust
#[component]
fn stacked() {
    let data = use_signal(metrics(120, MINUTE, 8));
    rsx! {
        <TimeSeries data={data} x={|m: &Metric| m.time} stacking={Stacking::NORMAL} fill_opacity={0.6} line_width={1.} class="h-64">
            <TrendSeries name="CPU %" y={|m: &Metric| m.cpu} />
            <TrendSeries name="Memory %" y={|m: &Metric| m.memory} />
        </TimeSeries>
    }
}
```

## 9. Bars

draw_style Bars on half-hour buckets.

```rust
#[component]
fn bars() {
    let data = use_signal(metrics(48, 30. * MINUTE, 9));
    rsx! {
        <TimeSeries data={data} x={|m: &Metric| m.time} draw_style={DrawStyle::Bars} fill_opacity={0.8} class="h-64">
            <TrendSeries name="requests/min" y={|m: &Metric| m.requests} />
        </TimeSeries>
    }
}
```

## 10. Thresholds

Dashed threshold lines and shaded threshold areas.

```rust
#[component]
fn threshold_lines() {
    let data = use_signal(metrics(120, MINUTE, 10));
    rsx! {
        <TimeSeries
            data={data}
            x={|m: &Metric| m.time}
            thresholds={Thresholds::absolute(&LOAD[..])}
            thresholds_style={ThresholdsStyle::DashedAndArea}
            class="h-64"
        >
            <TrendSeries name="CPU %" y={|m: &Metric| m.cpu} />
        </TimeSeries>
    }
}
```

## 11. Scheme gradient

A smooth line shaded on a colour scheme.

```rust
#[component]
fn gradient() {
    let data = use_signal(metrics(120, MINUTE, 11));
    rsx! {
        <TimeSeries data={data} x={|m: &Metric| m.time} gradient={GradientMode::Scheme} scheme={Scheme::GreenYellowRed} fill_opacity={0.4} interpolation={Interpolation::Smooth} line_width={2.} class="h-64">
            <TrendSeries name="CPU %" y={|m: &Metric| m.cpu} />
        </TimeSeries>
    }
}
```

## 12. Two axes

Latency on the left, request bars on the right.

```rust
#[component]
fn two_axes() {
    let data = use_signal(metrics(120, MINUTE, 12));
    rsx! {
        <TimeSeries data={data} x={|m: &Metric| m.time} class="h-64">
            <TrendSeries name="p99 ms" y={|m: &Metric| m.p99} y_axis={AxisConfig::new().label("ms")} />
            <TrendSeries name="requests/min" y={|m: &Metric| m.requests} y_axis={AxisConfig::new().label("req").placement(AxisPlacement::Right)} draw_style={DrawStyle::Bars} fill_opacity={0.3} />
        </TimeSeries>
    }
}
```

## 13. Missing values

A gap, and the same gap spanned.

```rust
#[component]
fn nulls() {
    let data = use_signal(
        metrics(120, MINUTE, 13)
            .into_iter()
            .map(|mut metric| {
                if (40.0..60.).contains(&((metric.time - T0) / MINUTE)) {
                    metric.cpu = f64::NAN;
                }
                metric
            })
            .collect::<Vec<_>>(),
    );
    rsx! {
        <TimeSeries data={data} x={|m: &Metric| m.time} class="h-64">
            <TrendSeries name="gap" y={|m: &Metric| m.cpu} />
            <TrendSeries name="spanned" y={|m: &Metric| m.cpu + 30.} span_nulls={NullSpan::Always} line_style={LineStyle::DASH} />
        </TimeSeries>
    }
}
```

## 14. Live data

A signal as data, a multi tooltip and a table legend.

```rust
#[component]
fn live() {
    let round = use_signal(1u64);
    let data = use_signal(metrics(120, MINUTE, 1));
    rsx! {
        <div class="flex flex-col gap-2 h-64">
            <div
                id="next"
                class="cursor-pointer text-sm underline"
                onClick={move || {
                    let next = round.get() + 1;
                    round.set(next);
                    data.set(metrics(120, MINUTE, next));
                }}
            >
                "Load new data"
            </div>
            <TimeSeries data={data} x={|m: &Metric| m.time} tooltip={Tooltip::multi()} legend={Legend::table().calcs([Calc::Mean, Calc::Max])} class="flex-1">
                <TrendSeries name="p50 ms" y={|m: &Metric| m.p50} />
                <TrendSeries name="p99 ms" y={|m: &Metric| m.p99} />
            </TimeSeries>
        </div>
    }
}
```

