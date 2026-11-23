# Gauge: examples

Every example here also runs in the gallery:

```bash
cargo run --example gallery
```

Choose **Gauge** in the sidebar. Not sure which example you need? Start from the [guide](guide.md).

Each example is a component; render it anywhere with `rsx! { <Basic /> }`.

The examples share these imports and this sample data:

```rust
use std::sync::LazyLock;

use blenderui::prelude::*;
use bui_charts::*;

const T0: f64 = 1_729_000_000_000.;
const MINUTE: f64 = 60_000.;

fn noise(seed: u64) -> f64 {
    let mixed = seed
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    (mixed >> 11) as f64 / (1u64 << 53) as f64
}

#[derive(Clone, PartialEq)]
struct Sample {
    time: f64,
    cpu: f64,
    temperature: f64,
    balance: f64,
}

fn samples(seed: u64) -> Vec<Sample> {
    (0..40u64)
        .map(|step| {
            let t = step as f64;
            Sample {
                time: T0 + t * MINUTE,
                cpu: (50. + 25. * (t / 7.).sin() + noise(seed * 100 + step) * 10.).clamp(0., 100.),
                temperature: 45. + 20. * (t / 11.).cos() + noise(seed * 200 + step) * 4.,
                balance: 30. * (t / 6.).sin() + noise(seed * 300 + step) * 10. - 5.,
            }
        })
        .collect()
}

fn nodes() -> Frame<5> {
    let rows: Vec<[f64; 5]> = (0..30u64)
        .map(|step| {
            let t = step as f64;
            let load = |seed: u64, base: f64| (base + 15. * ((t + seed as f64) / 5.).sin() + noise(seed * 40 + step) * 8.).clamp(0., 100.);
            [T0 + t * MINUTE, load(1, 35.), load(2, 62.), load(3, 81.), load(4, 48.)]
        })
        .collect();
    Frame::new(["time", "node-1", "node-2", "node-3", "node-4"]).with_rows(rows)
}

static LOAD: LazyLock<[(f64, Hsla); 3]> = LazyLock::new(|| {
    [
        (f64::NEG_INFINITY, rgb(0x40a02b)),
        (60., rgb(0xdf8e1d)),
        (85., rgb(0xd20f39)),
    ]
});

static HEAT: LazyLock<[(f64, Hsla); 4]> = LazyLock::new(|| {
    [
        (f64::NEG_INFINITY, rgb(0x1e66f5)),
        (40., rgb(0x40a02b)),
        (60., rgb(0xdf8e1d)),
        (75., rgb(0xd20f39)),
    ]
});

fn percent(value: f64) -> SharedString {
    format!("{value:.0}%").into()
}

fn celsius(value: f64) -> SharedString {
    format!("{value:.1} °C").into()
}
```

## 1. Gauge

The last CPU reading on a 220° arc with threshold markers and a sparkline.

```rust
#[component]
fn basic() {
    let data = use_signal(samples(1));
    rsx! {
        <Gauge data={data} value={|s: &Sample| s.cpu} x={|s: &Sample| s.time} thresholds={Thresholds::absolute(&LOAD[..])} format={percent} class="h-56" />
    }
}
```

## 2. Circle

shape Circle: a full ring.

```rust
#[component]
fn circle() {
    let data = use_signal(samples(2));
    rsx! {
        <Gauge data={data} value={|s: &Sample| s.cpu} x={|s: &Sample| s.time} shape={GaugeShape::Circle} thresholds={Thresholds::absolute(&LOAD[..])} format={percent} class="h-56" />
    }
}
```

## 3. Rounded bar

Rounded ends, a wider bar and point markers.

```rust
#[component]
fn rounded() {
    let data = use_signal(samples(3));
    rsx! {
        <Gauge data={data} value={|s: &Sample| s.cpu} x={|s: &Sample| s.time} bar_shape={BarShape::Rounded} bar_width_factor={0.8} sparkline={false} thresholds={Thresholds::absolute(&LOAD[..])} format={percent} class="h-56" />
    }
}
```

## 4. Segments

20 segments, each coloured by the threshold it represents.

```rust
#[component]
fn segments() {
    let data = use_signal(samples(4));
    rsx! {
        <Gauge data={data} value={|s: &Sample| s.cpu} x={|s: &Sample| s.time} segment_count={20} segment_spacing={0.4} sparkline={false} thresholds={Thresholds::absolute(&LOAD[..])} format={percent} class="h-56" />
    }
}
```

## 5. Effects

Gradient, bar glow, centre glow and a glowing endpoint.

```rust
#[component]
fn effects() {
    let data = use_signal(samples(5));
    rsx! {
        <Gauge
            data={data}
            value={|s: &Sample| s.temperature}
            x={|s: &Sample| s.time}
            min={0.}
            max={100.}
            effects={GaugeEffects::all()}
            bar_shape={BarShape::Rounded}
            endpoint_marker={EndpointMarker::Glow}
            thresholds={Thresholds::absolute(&HEAT[..])}
            format={celsius}
            class="h-56"
        />
    }
}
```

## 6. Threshold labels

Threshold values written around the arc.

```rust
#[component]
fn labels() {
    let data = use_signal(samples(6));
    rsx! {
        <Gauge data={data} value={|s: &Sample| s.temperature} x={|s: &Sample| s.time} show_threshold_labels={true} sparkline={false} thresholds={Thresholds::absolute(&HEAT[..])} format={celsius} class="h-56" />
    }
}
```

## 7. Neutral point

The bar grows from 0 in either direction.

```rust
#[component]
fn neutral() {
    let data = use_signal(samples(7));
    rsx! {
        <Gauge
            data={data}
            value={|s: &Sample| s.balance}
            x={|s: &Sample| s.time}
            min={-50.}
            max={50.}
            neutral={0.}
            show_threshold_markers={false}
            show_threshold_labels={true}
            color={GaugeColor::Scheme(Scheme::RedYellowGreen)}
            decimals={1}
            class="h-56"
        />
    }
}
```

## 8. Many gauges

Every field of a frame, laid out in a grid.

```rust
#[component]
fn many() {
    let data = use_signal(nodes());
    rsx! {
        <Gauge data={data} x_field="time" thresholds={Thresholds::absolute(&LOAD[..])} format={percent} class="h-72" />
    }
}
```

## 9. Colour scheme

Blue–yellow–red scheme as a gradient on rings.

```rust
#[component]
fn scheme_rings() {
    let data = use_signal(nodes());
    rsx! {
        <Gauge
            data={data}
            x_field="time"
            fields={&["node-2", "node-3"]}
            shape={GaugeShape::Circle}
            color={GaugeColor::Scheme(Scheme::BlueYellowRed)}
            effects={GaugeEffects::new().gradient()}
            bar_shape={BarShape::Rounded}
            format={percent}
            class="h-56"
        />
    }
}
```

## 10. Manual sizing

A row of gauges at least 140 px wide.

```rust
#[component]
fn row() {
    let data = use_signal(nodes());
    rsx! {
        <Gauge
            data={data}
            x_field="time"
            orientation={BarOrientation::Vertical}
            sizing={Sizing::Manual { min_width: px(140.), min_height: px(75.), max_height: px(300.) }}
            text_mode={TextMode::ValueAndName}
            sparkline={false}
            reduce={Reduce::Calc(Calc::Mean)}
            thresholds={Thresholds::absolute(&LOAD[..])}
            format={percent}
            class="h-48"
        />
    }
}
```

## 11. Palette colours

One palette colour per gauge, with gradient and glow.

```rust
#[component]
fn palette() {
    let data = use_signal(nodes());
    rsx! {
        <Gauge
            data={data}
            x_field="time"
            fields={&["node-1", "node-4"]}
            color={GaugeColor::Palette}
            effects={GaugeEffects::new().gradient().bar_glow()}
            show_threshold_markers={false}
            bar_width_factor={0.3}
            format={percent}
            class="h-56"
        />
    }
}
```

## 12. No text

text_mode None, no markers, no sparkline.

```rust
#[component]
fn quiet() {
    let data = use_signal(samples(8));
    rsx! {
        <Gauge
            data={data}
            value={|s: &Sample| s.cpu}
            text_mode={TextMode::None}
            sparkline={false}
            show_threshold_markers={false}
            endpoint_marker={EndpointMarker::None}
            color={GaugeColor::Fixed(rgb(0x8839ef))}
            class="h-56"
        />
    }
}
```

## 13. Live data

A signal as data: the gauge redraws when it changes.

```rust
#[component]
fn live() {
    let round = use_signal(1u64);
    let data = use_signal(samples(1));
    rsx! {
        <div class="flex flex-col gap-2 h-64">
            <div
                id="next"
                class="cursor-pointer text-sm underline"
                onClick={move || {
                    let next = round.get() + 1;
                    round.set(next);
                    data.set(samples(next));
                }}
            >
                "Load new readings"
            </div>
            <Gauge data={data} value={|s: &Sample| s.cpu} x={|s: &Sample| s.time} bar_shape={BarShape::Rounded} effects={GaugeEffects::new().gradient()} thresholds={Thresholds::absolute(&LOAD[..])} format={percent} class="flex-1" />
        </div>
    }
}
```

