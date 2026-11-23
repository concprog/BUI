# Status History: examples

Every example here also runs in the gallery:

```bash
cargo run --example gallery
```

Choose **Status History** in the sidebar. Not sure which example you need? Start from the [guide](guide.md).

Each example is a component; render it anywhere with `rsx! { <ServiceChecks /> }`.

The examples share these imports and this sample data:

```rust
use std::sync::LazyLock;

use blenderui::prelude::*;
use bui_charts::*;

const T0: f64 = 1_729_000_000_000.;
const HOUR: f64 = 3_600_000.;

fn noise(seed: u64) -> f64 {
    let mixed = seed
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    (mixed >> 11) as f64 / (1u64 << 53) as f64
}

fn status(seed: u64) -> &'static str {
    match noise(seed) {
        roll if roll < 0.7 => "OK",
        roll if roll < 0.9 => "WARN",
        _ => "DOWN",
    }
}

#[derive(Clone, PartialEq)]
struct Probe {
    time: f64,
    api: &'static str,
    web: &'static str,
    db: &'static str,
    cache: &'static str,
}

fn probes(count: u64) -> Vec<Probe> {
    (0..count)
        .map(|step| Probe {
            time: T0 + step as f64 * HOUR,
            api: status(step * 7 + 1),
            web: status(step * 11 + 2),
            db: status(step * 13 + 3),
            cache: status(step * 17 + 4),
        })
        .collect()
}

#[derive(Clone, PartialEq)]
struct Host {
    time: f64,
    cpu: f64,
    memory: f64,
    up: bool,
}

fn hosts() -> Vec<Host> {
    (0..16u64)
        .map(|step| {
            let t = step as f64;
            Host {
                time: T0 + t * HOUR,
                cpu: (55. + 40. * (t / 2.5).sin()).round(),
                memory: (40. + t * 3.5 + noise(step) * 10.).round(),
                up: noise(step * 31) > 0.2,
            }
        })
        .collect()
}

fn regions() -> Frame<5> {
    let rows: Vec<[f64; 5]> = (0..12u64)
        .map(|step| {
            let level = |seed: u64| (noise(seed * 97 + step) * 100.).round();
            [T0 + step as f64 * HOUR, level(1), level(2), level(3), level(4)]
        })
        .collect();
    Frame::new(["time", "eu-west", "us-east", "ap-south", "sa-east"]).with_rows(rows)
}

static STATES: LazyLock<ValueMappings> = LazyLock::new(|| {
    ValueMappings::new()
        .value("OK", "OK", rgb(0x40a02b))
        .value("WARN", "Slow", rgb(0xdf8e1d))
        .value("DOWN", "Down", rgb(0xd20f39))
});

static UP: LazyLock<ValueMappings> = LazyLock::new(|| {
    ValueMappings::new()
        .special(Special::True, "Up", rgb(0x1e66f5))
        .special(Special::False, "Down", rgb(0x9ca0b0))
});

static LOAD: LazyLock<[(f64, Hsla); 3]> = LazyLock::new(|| {
    [
        (f64::NEG_INFINITY, rgb(0x40a02b)),
        (60., rgb(0xdf8e1d)),
        (85., rgb(0xd20f39)),
    ]
});

fn time() -> AxisConfig {
    AxisConfig::new().time()
}
```

## 1. Service checks

Hourly checks, one cell per sample, renamed and coloured by mappings.

```rust
#[component]
fn service_checks() {
    let data = use_signal(probes(16));
    rsx! {
        <StatusHistory data={data} x={|p: &Probe| p.time} mappings={&*STATES} x_axis={time()} class="h-64">
            <StatusRow name="api" value={|p: &Probe| p.api} />
            <StatusRow name="web" value={|p: &Probe| p.web} />
            <StatusRow name="db" value={|p: &Probe| p.db} />
            <StatusRow name="cache" value={|p: &Probe| p.cache} />
        </StatusHistory>
    }
}
```

## 2. Threshold colours

Numbers coloured by the threshold step they reach.

```rust
#[component]
fn threshold_colors() {
    let data = use_signal(hosts());
    rsx! {
        <StatusHistory data={data} x={|h: &Host| h.time} color={GaugeColor::Thresholds} thresholds={Thresholds::absolute(&LOAD[..])} x_axis={time()} class="h-48">
            <StatusRow name="CPU %" value={|h: &Host| h.cpu} />
            <StatusRow name="Memory %" value={|h: &Host| h.memory} />
        </StatusHistory>
    }
}
```

## 3. Colour by value

The default green–yellow–red scheme over 0–100.

```rust
#[component]
fn by_value() {
    let data = use_signal(hosts());
    rsx! {
        <StatusHistory data={data} x={|h: &Host| h.time} min={0.} max={100.} x_axis={time()} class="h-48">
            <StatusRow name="CPU %" value={|h: &Host| h.cpu} />
            <StatusRow name="Memory %" value={|h: &Host| h.memory} />
        </StatusHistory>
    }
}
```

## 4. Booleans

true / false mapped to Up / Down.

```rust
#[component]
fn booleans() {
    let data = use_signal(hosts());
    rsx! {
        <StatusHistory data={data} x={|h: &Host| h.time} mappings={&*UP} x_axis={time()} class="h-32">
            <StatusRow name="node-3" value={|h: &Host| h.up} />
        </StatusHistory>
    }
}
```

## 5. Narrow cells

col_width 0.5 and row_height 0.6, values hidden.

```rust
#[component]
fn narrow() {
    let data = use_signal(probes(16));
    rsx! {
        <StatusHistory data={data} x={|p: &Probe| p.time} mappings={&*STATES} col_width={0.5} row_height={0.6} show_value={VisibilityMode::Never} x_axis={time()} class="h-48">
            <StatusRow name="api" value={|p: &Probe| p.api} />
            <StatusRow name="web" value={|p: &Probe| p.web} />
            <StatusRow name="db" value={|p: &Probe| p.db} />
        </StatusHistory>
    }
}
```

## 6. No gaps

col_width and row_height 1 with a solid fill.

```rust
#[component]
fn solid() {
    let data = use_signal(probes(24));
    rsx! {
        <StatusHistory data={data} x={|p: &Probe| p.time} mappings={&*STATES} col_width={1.} row_height={1.} fill_opacity={1.} line_width={0.} show_value={VisibilityMode::Never} legend={Legend::hidden()} x_axis={time()} class="h-40">
            <StatusRow name="api" value={|p: &Probe| p.api} />
            <StatusRow name="web" value={|p: &Probe| p.web} />
            <StatusRow name="db" value={|p: &Probe| p.db} />
        </StatusHistory>
    }
}
```

## 7. From a frame

Every field is a row, coloured on the Blues scheme.

```rust
#[component]
fn from_frame() {
    let data = use_signal(regions());
    rsx! { <StatusHistory data={data} color={GaugeColor::Scheme(Scheme::Blues)} min={0.} max={100.} x_axis={time()} class="h-56" /> }
}
```

## 8. Pages

Two rows per page with page buttons.

```rust
#[component]
fn pages() {
    let data = use_signal(probes(12));
    rsx! {
        <StatusHistory data={data} x={|p: &Probe| p.time} mappings={&*STATES} per_page={2} x_axis={time()} class="h-48">
            <StatusRow name="api" value={|p: &Probe| p.api} />
            <StatusRow name="web" value={|p: &Probe| p.web} />
            <StatusRow name="db" value={|p: &Probe| p.db} />
            <StatusRow name="cache" value={|p: &Probe| p.cache} />
        </StatusHistory>
    }
}
```

## 9. Multi tooltip

Every row at the hovered sample, legend on the right.

```rust
#[component]
fn multi_tooltip() {
    let data = use_signal(probes(16));
    rsx! {
        <StatusHistory data={data} x={|p: &Probe| p.time} mappings={&*STATES} tooltip={Tooltip::multi()} legend={Legend::list().right()} x_axis={time()} class="h-56">
            <StatusRow name="api" value={|p: &Probe| p.api} />
            <StatusRow name="web" value={|p: &Probe| p.web} />
            <StatusRow name="db" value={|p: &Probe| p.db} />
        </StatusHistory>
    }
}
```

## 10. Per-row colours

One row on thresholds, one on mappings.

```rust
#[component]
fn per_row() {
    let data = use_signal(hosts());
    rsx! {
        <StatusHistory data={data} x={|h: &Host| h.time} x_axis={time()} class="h-48">
            <StatusRow name="CPU %" value={|h: &Host| h.cpu} color={GaugeColor::Thresholds} thresholds={Thresholds::absolute(&LOAD[..])} />
            <StatusRow name="up" value={|h: &Host| h.up} mappings={&*UP} />
        </StatusHistory>
    }
}
```

## 11. Weekly grid

Numeric x with a custom label format: busy hours per day.

```rust
#[component]
fn week() {
    let data = use_signal(
        Frame::new(["day", "00–06", "06–12", "12–18", "18–24"]).with_rows(
            (0..7u64)
                .map(|day| {
                    let busy = |slot: u64, base: f64| (base + noise(day * 4 + slot) * 30.).round();
                    [day as f64, busy(0, 5.), busy(1, 45.), busy(2, 60.), busy(3, 30.)]
                })
                .collect::<Vec<_>>(),
        ),
    );
    rsx! {
        <StatusHistory
            data={data}
            color={GaugeColor::Scheme(Scheme::Oranges)}
            min={0.}
            max={90.}
            x_axis={AxisConfig::new().format(|day| ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"][(day as usize).min(6)].into())}
            class="h-56"
        />
    }
}
```

## 12. Drag to zoom

Drag across the cells; on_select reports the range.

```rust
#[component]
fn zoom() {
    let data = use_signal(probes(48));
    let picked = use_signal(SharedString::Borrowed(
        "Drag across the cells to zoom. Double-click to zoom out.",
    ));
    rsx! {
        <div class="flex flex-col gap-2 h-56">
            <div class="text-xs">{picked.get()}</div>
            <StatusHistory
                data={data}
                x={|p: &Probe| p.time}
                mappings={&*STATES}
                show_value={VisibilityMode::Never}
                on_select={move |range: SelectedRange, _: &mut Window, _: &mut App| {
                    let hours = (range.x.1 - range.x.0) / HOUR;
                    picked.set(SharedString::from(format!("Zoomed to {hours:.0} hours")));
                }}
                x_axis={time()}
                class="flex-1"
            >
                <StatusRow name="api" value={|p: &Probe| p.api} />
                <StatusRow name="db" value={|p: &Probe| p.db} />
            </StatusHistory>
        </div>
    }
}
```

