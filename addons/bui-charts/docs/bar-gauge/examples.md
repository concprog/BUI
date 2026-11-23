# Bar Gauge: examples

Every example here also runs in the gallery:

```bash
cargo run --example gallery
```

Choose **Bar Gauge** in the sidebar. Not sure which example you need? Start from the [guide](guide.md).

The examples share these imports and this sample data:

```rust
use std::sync::LazyLock;

use blenderui::prelude::*;
use bui_charts::*;

#[derive(Clone, PartialEq)]
struct Disk {
    mount: &'static str,
    used: f64,
}

fn disks() -> Vec<Disk> {
    [
        ("/", 42.),
        ("/home", 67.),
        ("/var", 88.),
        ("/tmp", 12.),
        ("/data", 95.),
    ]
    .map(|(mount, used)| Disk { mount, used })
    .into()
}

fn services() -> Frame<4> {
    Frame::new(["api", "web", "worker", "cron"]).with_rows([
        [120., 340., 80., 12.],
        [180., 310., 95., 20.],
        [240., 290., 70., 15.],
        [210., 360., 110., 18.],
    ])
}

static STEPS: LazyLock<[(f64, Hsla); 3]> = LazyLock::new(|| {
    [
        (f64::NEG_INFINITY, rgb(0x40a02b)),
        (60., rgb(0xdf8e1d)),
        (85., rgb(0xd20f39)),
    ]
});
```

| # | Example | Shows |
| --- | --- | --- |
| 1 | [Disk usage](#1-disk-usage) | rows with `name` and `value` |
| 2 | [Retro LCD](#2-retro-lcd) | `BarDisplay::Lcd` |
| 3 | [Basic](#3-basic) | `BarDisplay::Basic` |
| 4 | [Columns](#4-columns) | `BarOrientation::Vertical` |
| 5 | [Reduce a frame](#5-reduce-a-frame) | one bar per field |
| 6 | [Pick fields](#6-pick-fields) | `fields`, `Calc::Mean` |
| 7 | [Percentage thresholds](#7-percentage-thresholds) | `Thresholds::percentage` |
| 8 | [Colour scheme](#8-colour-scheme) | `GaugeColor::Scheme` |
| 9 | [Palette colours](#9-palette-colours) | `GaugeColor::Palette` |
| 10 | [Names on the left](#10-names-on-the-left) | `NamePlacement::Left`, `ValueMode::Text` |
| 11 | [Manual sizing](#11-manual-sizing) | `Sizing::Manual` |
| 12 | [Units and range](#12-units-and-range) | `min`, `max`, `format`, `show_unfilled` |
| 13 | [Live data](#13-live-data) | updating a signal |
| 14 | [With a legend](#14-with-a-legend) | `legend`, `TextMode::Value` |

---

## 1. Disk usage

One bar per row. The gradient passes through the default thresholds: your theme's good colour, then its bad colour from 80.

```rust
#[component]
fn disk_usage() {
    let data = use_signal(disks());
    rsx! {
        <BarGauge data={data} name={|d: &Disk| d.mount} value={|d| d.used} class="h-64" />
    }
}
```

## 2. Retro LCD

Cells light up to the value, and each cell takes the threshold colour for its own position.

```rust
#[component]
fn retro_lcd() {
    let data = use_signal(disks());
    rsx! {
        <BarGauge data={data} name={|d: &Disk| d.mount} value={|d| d.used} display={BarDisplay::Lcd} class="h-64" />
    }
}
```

## 3. Basic

A flat, translucent bar with a solid edge at the value.

```rust
#[component]
fn basic_bars() {
    let data = use_signal(disks());
    rsx! {
        <BarGauge data={data} name={|d: &Disk| d.mount} value={|d| d.used} display={BarDisplay::Basic} class="h-64" />
    }
}
```

## 4. Columns

Vertical bars side by side, with names underneath.

```rust
#[component]
fn columns() {
    let data = use_signal(disks());
    rsx! {
        <BarGauge
            data={data}
            name={|d: &Disk| d.mount}
            value={|d| d.used}
            orientation={BarOrientation::Vertical}
            display={BarDisplay::Lcd}
            class="h-64"
        />
    }
}
```

## 5. Reduce a frame

With a `Frame`, each field becomes a bar showing its last value.

```rust
#[component]
fn reduce_a_frame() {
    let frame = use_signal(services());
    rsx! {
        <BarGauge data={frame} reduce={Reduce::Calc(Calc::Last)} max={400.} class="h-64" />
    }
}
```

## 6. Pick fields

Only the listed fields, each summarised by its mean.

```rust
#[component]
fn pick_fields() {
    let frame = use_signal(services());
    rsx! {
        <BarGauge
            data={frame}
            reduce={Reduce::Calc(Calc::Mean)}
            fields={&["api", "worker"]}
            max={300.}
            display={BarDisplay::Basic}
            class="h-64"
        />
    }
}
```

## 7. Percentage thresholds

Steps given as percentages of the range. The list is built once, in a static.

```rust
#[component]
fn custom_thresholds() {
    let data = use_signal(disks());
    rsx! {
        <BarGauge
            data={data}
            name={|d: &Disk| d.mount}
            value={|d| d.used}
            thresholds={Thresholds::percentage(&STEPS[..])}
            class="h-64"
        />
    }
}
```

## 8. Colour scheme

A continuous scheme instead of thresholds: each bar is coloured by where its value falls, and the gradient shows the whole scheme.

```rust
#[component]
fn scheme_gradient() {
    let data = use_signal(disks());
    rsx! {
        <BarGauge
            data={data}
            name={|d: &Disk| d.mount}
            value={|d| d.used}
            color={GaugeColor::Scheme(Scheme::GreenYellowRed)}
            class="h-64"
        />
    }
}
```

## 9. Palette colours

Each bar takes the next palette colour, for telling bars apart rather than judging them.

```rust
#[component]
fn palette_colors() {
    let frame = use_signal(services());
    rsx! {
        <BarGauge
            data={frame}
            color={GaugeColor::Palette}
            orientation={BarOrientation::Vertical}
            max={400.}
            class="h-64"
        />
    }
}
```

## 10. Names on the left

Names in their own column, values in the normal text colour.

```rust
#[component]
fn text_modes() {
    let data = use_signal(disks());
    rsx! {
        <BarGauge
            data={data}
            name={|d: &Disk| d.mount}
            value={|d| d.used}
            value_mode={ValueMode::Text}
            name_placement={NamePlacement::Left}
            class="h-64"
        />
    }
}
```

## 11. Manual sizing

Rows capped at 24px tall, however much room there is.

```rust
#[component]
fn manual_sizing() {
    let data = use_signal(disks());
    rsx! {
        <BarGauge
            data={data}
            name={|d: &Disk| d.mount}
            value={|d| d.used}
            sizing={Sizing::Manual { min_width: px(8.), min_height: px(16.), max_height: px(24.) }}
            name_placement={NamePlacement::Left}
            display={BarDisplay::Lcd}
            class="h-64"
        />
    }
}
```

## 12. Units and range

A custom range and value text, a fixed colour, and no unfilled track.

```rust
#[component]
fn units_and_range() {
    let data = use_signal(disks());
    rsx! {
        <BarGauge
            data={data}
            name={|d: &Disk| d.mount}
            value={|d| d.used * 10.24}
            min={0.}
            max={1024.}
            format={|v| format!("{v:.0} GiB").into()}
            show_unfilled={false}
            color={GaugeColor::Fixed(rgb(0x1e66f5))}
            class="h-64"
        />
    }
}
```

## 13. Live data

Pass the signal itself. Updating it in place moves the bars.

```rust
#[component]
fn live_value() {
    let data = use_signal(disks());
    rsx! {
        <div class="flex flex-col gap-2 h-64">
            <div class="cursor-pointer text-sm underline" onClick={move || data.update(|rows| rows.iter_mut().for_each(|d| d.used = (d.used * 1.37) % 100.))}>
                "Shuffle"
            </div>
            <BarGauge data={data} name={|d: &Disk| d.mount} value={|d| d.used} class="flex-1" />
        </div>
    }
}
```

## 14. With a legend

A legend on the right; click an entry to show only that bar, Ctrl-click to toggle one.

```rust
#[component]
fn with_legend() {
    let frame = use_signal(services());
    rsx! {
        <BarGauge
            data={frame}
            color={GaugeColor::Palette}
            max={400.}
            legend={Legend::list().right()}
            text_mode={TextMode::Value}
            class="h-64"
        />
    }
}
```

