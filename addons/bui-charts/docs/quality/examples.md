# Quality charts: examples

Every example here also runs in the gallery (`cargo run --example gallery`), under **Charts** in the sidebar.

The examples share these imports and this sample data:

```rust
use blenderui::prelude::*;
use bui_charts::*;

use crate::finance::noise;

#[derive(Clone, PartialEq)]
struct Defect {
    pub cause: &'static str,
    pub count: f64,
}

fn defects() -> Vec<Defect> {
    [
        ("Scratches", 142.),
        ("Misalignment", 87.),
        ("Wrong label", 51.),
        ("Dents", 33.),
        ("Discolouration", 19.),
        ("Missing screw", 12.),
        ("Other", 7.),
    ]
    .map(|(cause, count)| Defect { cause, count })
    .into()
}

#[derive(Clone, PartialEq)]
struct Sample {
    pub time: f64,
    pub diameter: f64,
}

fn samples(count: u64, seed: u64, shift: f64) -> Vec<Sample> {
    (0..count)
        .map(|index| {
            let drift = if index > count * 2 / 3 { shift } else { 0. };
            Sample {
                time: 1_729_000_000_000. + index as f64 * 600_000.,
                diameter: 25. + (noise(seed * 1000 + index) - 0.5) * 0.4 + drift,
            }
        })
        .collect()
}

#[derive(Clone, PartialEq)]
struct Reading {
    pub cuff: f64,
    pub arterial: f64,
}

fn readings(seed: u64) -> Vec<Reading> {
    (0..80u64)
        .map(|index| {
            let truth = 90. + noise(seed * 100 + index) * 70.;
            Reading {
                cuff: truth + (noise(seed * 300 + index) - 0.5) * 12. + 3.,
                arterial: truth + (noise(seed * 500 + index) - 0.5) * 8.,
            }
        })
        .collect()
}
```

## `<Pareto>`

### Defect causes

Bars sorted largest first and the cumulative share; the dashed line marks 80%.

```rust
#[component]
fn causes() {
    let data = use_signal(defects());
    rsx! { <Pareto data={data} name={|d: &Defect| d.cause} value={|d: &Defect| d.count} class="h-72" /> }
}
```

### Labels and 90% line

Counts written on the bars and a 90% threshold.

```rust
#[component]
fn values() {
    let data = use_signal(defects());
    rsx! { <Pareto data={data} name={|d: &Defect| d.cause} value={|d: &Defect| d.count} show_values={true} threshold={90.} color={rgb(0x8839ef)} class="h-72" /> }
}
```

### Support tickets

Which areas cause most tickets.

```rust
#[component]
fn tickets() {
    let data = use_signal(
        [
            ("Login", 412.),
            ("Billing", 198.),
            ("Export", 120.),
            ("Search", 64.),
            ("Mobile", 41.),
            ("API", 22.),
        ]
        .map(|(cause, count)| Defect { cause, count })
        .to_vec(),
    );
    rsx! { <Pareto data={data} name={|d: &Defect| d.cause} value={|d: &Defect| d.count} line_color={rgb(0xdf8e1d)} class="h-72" /> }
}
```

### No legend or tooltip

The defect chart with only bars and lines.

```rust
#[component]
fn bare() {
    let data = use_signal(defects());
    rsx! { <Pareto data={data} name={|d: &Defect| d.cause} value={|d: &Defect| d.count} legend={Legend::hidden()} tooltip={Tooltip::hidden()} class="h-72" /> }
}
```

### Legend on the right

Order returns with the legend beside the plot.

```rust
#[component]
fn side_legend() {
    let data = use_signal(returns());
    rsx! { <Pareto data={data} name={|d: &Defect| d.cause} value={|d: &Defect| d.count} legend={Legend::list().right()} class="h-72" /> }
}
```

### Returns at 70%

Custom colours, labels and a 70% threshold.

```rust
#[component]
fn returned() {
    let data = use_signal(returns());
    rsx! { <Pareto data={data} name={|d: &Defect| d.cause} value={|d: &Defect| d.count} show_values={true} threshold={70.} color={rgb(0x04a5e5)} line_color={rgb(0xd20f39)} class="h-72" /> }
}
```

### Machine downtime

Ten machines, minutes lost, with labels and a 75% line.

```rust
#[component]
fn downtime() {
    let data = use_signal(tally(&[
        ("Press 4", 540.),
        ("Lathe 2", 310.),
        ("Welder", 205.),
        ("Conveyor", 120.),
        ("Packer", 85.),
        ("Mixer", 40.),
        ("Labeller", 22.),
        ("Oven", 18.),
        ("Robot arm", 11.),
        ("Other", 30.),
    ]));
    rsx! { <Pareto data={data} name={|d: &Defect| d.cause} value={|d: &Defect| d.count} show_values={true} threshold={75.} color={rgb(0xfe640b)} class="h-80" /> }
}
```

### Three causes

Complaints where one cause is already 70%.

```rust
#[component]
fn few() {
    let data = use_signal(tally(&[("Late", 70.), ("Wrong", 20.), ("Rude", 10.)]));
    rsx! { <Pareto data={data} name={|d: &Defect| d.cause} value={|d: &Defect| d.count} show_values={true} threshold={90.} class="h-72" /> }
}
```

### Twelve categories

A long tail of tenant complaints against a 60% line.

```rust
#[component]
fn complaints() {
    let data = use_signal(tally(&[
        ("Noise", 18.),
        ("Parking", 15.),
        ("Waste", 14.),
        ("Rent", 12.),
        ("Heating", 11.),
        ("Lifts", 9.),
        ("Pests", 7.),
        ("Smell", 5.),
        ("Lighting", 4.),
        ("Security", 3.),
        ("Post", 2.),
        ("Pets", 1.),
    ]));
    rsx! { <Pareto data={data} name={|d: &Defect| d.cause} value={|d: &Defect| d.count} threshold={60.} color={rgb(0x40a02b)} class="h-72" /> }
}
```

### Small batch

Only seven defects in a batch, with a 95% threshold.

```rust
#[component]
fn small_batch() {
    let data = use_signal(tally(&[
        ("Burr", 3.),
        ("Crack", 2.),
        ("Porosity", 1.),
        ("Warp", 1.),
    ]));
    rsx! { <Pareto data={data} name={|d: &Defect| d.cause} value={|d: &Defect| d.count} show_values={true} threshold={95.} line_color={rgb(0x7287fd)} class="h-72" /> }
}
```

## `<ControlChart>`

### In control

Centre line and ±3σ limits from the average moving range.

```rust
#[component]
fn in_control() {
    let data = use_signal(samples(60, 1, 0.));
    rsx! { <ControlChart data={data} x={|s: &Sample| s.time} value={|s: &Sample| s.diameter} class="h-72" /> }
}
```

### Process shift

A late shift crosses the upper limit; points out of control are marked. Zones at ±1σ and ±2σ.

```rust
#[component]
fn shifted() {
    let data = use_signal(samples(60, 2, 0.35));
    rsx! { <ControlChart data={data} x={|s: &Sample| s.time} value={|s: &Sample| s.diameter} zones={true} class="h-72" /> }
}
```

### Known centre and σ

Limits from a target diameter and a known σ.

```rust
#[component]
fn specified() {
    let data = use_signal(samples(60, 3, 0.2));
    rsx! { <ControlChart data={data} x={|s: &Sample| s.time} value={|s: &Sample| s.diameter} center={25.} sigma={0.1} limit={3.} class="h-72" /> }
}
```

### Zones, no legend

A stable process with ±1σ and ±2σ zones and the legend hidden.

```rust
#[component]
fn zoned() {
    let data = use_signal(samples(60, 4, 0.));
    rsx! { <ControlChart data={data} x={|s: &Sample| s.time} value={|s: &Sample| s.diameter} zones={true} legend={Legend::hidden()} class="h-72" /> }
}
```

### 2σ limits

A tighter limit flags more points as out of control.

```rust
#[component]
fn warning() {
    let data = use_signal(samples(60, 5, 0.15));
    rsx! { <ControlChart data={data} x={|s: &Sample| s.time} value={|s: &Sample| s.diameter} limit={2.} class="h-72" /> }
}
```

### 240 samples

A long run with a late shift and no tooltip.

```rust
#[component]
fn long_run() {
    let data = use_signal(samples(240, 6, 0.25));
    rsx! { <ControlChart data={data} x={|s: &Sample| s.time} value={|s: &Sample| s.diameter} tooltip={Tooltip::hidden()} class="h-72" /> }
}
```

### 25 samples

A short start-up run in purple.

```rust
#[component]
fn startup() {
    let data = use_signal(samples(25, 7, 0.));
    rsx! { <ControlChart data={data} x={|s: &Sample| s.time} value={|s: &Sample| s.diameter} color={rgb(0x8839ef)} class="h-72" /> }
}
```

### Downward shift

The diameter drops late in the run, with zones and the legend on the right.

```rust
#[component]
fn falling() {
    let data = use_signal(samples(60, 8, -0.4));
    rsx! { <ControlChart data={data} x={|s: &Sample| s.time} value={|s: &Sample| s.diameter} zones={true} legend={Legend::list().right()} class="h-72" /> }
}
```

### Tight known σ

A known σ of 0.05 is tighter than the process, so natural noise breaks the limits.

```rust
#[component]
fn tight_spec() {
    let data = use_signal(samples(60, 9, 0.));
    rsx! { <ControlChart data={data} x={|s: &Sample| s.time} value={|s: &Sample| s.diameter} center={25.} sigma={0.05} zones={true} class="h-72" /> }
}
```

### 4σ limits

Wider limits around a known centre; only a large shift is flagged.

```rust
#[component]
fn wide_limits() {
    let data = use_signal(samples(120, 10, 0.3));
    rsx! { <ControlChart data={data} x={|s: &Sample| s.time} value={|s: &Sample| s.diameter} center={25.} sigma={0.1} limit={4.} color={rgb(0x179299)} class="h-72" /> }
}
```

## `<BlandAltman>`

### Two methods

Difference against mean, with the bias and 95% limits of agreement.

```rust
#[component]
fn pressure() {
    let data = use_signal(readings(1));
    rsx! { <BlandAltman data={data} a={|r: &Reading| r.cuff} b={|r: &Reading| r.arterial} a_label="Cuff" b_label="Arterial line" class="h-72" /> }
}
```

### 99% limits

agreement 2.58 widens the limits to 99%.

```rust
#[component]
fn ninety_five() {
    let data = use_signal(readings(2));
    rsx! { <BlandAltman data={data} a={|r: &Reading| r.cuff} b={|r: &Reading| r.arterial} agreement={2.58} color={rgb(0x179299)} class="h-72" /> }
}
```

### Proportional bias

One method reads 8% high: the differences grow with the mean.

```rust
#[component]
fn biased() {
    let data = use_signal(
        readings(3)
            .into_iter()
            .map(|reading| Reading {
                cuff: reading.cuff * 1.08,
                ..reading
            })
            .collect::<Vec<_>>(),
    );
    rsx! { <BlandAltman data={data} a={|r: &Reading| r.cuff} b={|r: &Reading| r.arterial} class="h-72" /> }
}
```

### No legend or tooltip

Only the points and the three lines.

```rust
#[component]
fn bare() {
    let data = use_signal(readings(4));
    rsx! { <BlandAltman data={data} a={|r: &Reading| r.cuff} b={|r: &Reading| r.arterial} legend={Legend::hidden()} tooltip={Tooltip::hidden()} class="h-72" /> }
}
```

### ±1 SD limits

agreement 1 narrows the limits to about 68%.

```rust
#[component]
fn one_sigma() {
    let data = use_signal(readings(5));
    rsx! { <BlandAltman data={data} a={|r: &Reading| r.cuff} b={|r: &Reading| r.arterial} agreement={1.} color={rgb(0x8839ef)} class="h-72" /> }
}
```

### Constant offset

The cuff reads 10 mmHg high everywhere, so the bias line sits above zero.

```rust
#[component]
fn constant_offset() {
    let data = use_signal(adjusted(6, 1., 10.));
    rsx! { <BlandAltman data={data} a={|r: &Reading| r.cuff} b={|r: &Reading| r.arterial} a_label="Cuff" b_label="Arterial line" class="h-72" /> }
}
```

### Under-reading

A wrist device reads 10% low, with the legend on the right.

```rust
#[component]
fn under_reading() {
    let data = use_signal(adjusted(7, 0.9, 0.));
    rsx! { <BlandAltman data={data} a={|r: &Reading| r.cuff} b={|r: &Reading| r.arterial} a_label="Wrist" b_label="Arterial line" color={rgb(0xdf8e1d)} legend={Legend::list().right()} class="h-72" /> }
}
```

### 3 SD limits

A very conservative band of agreement.

```rust
#[component]
fn wide_agreement() {
    let data = use_signal(adjusted(8, 1., -4.));
    rsx! { <BlandAltman data={data} a={|r: &Reading| r.cuff} b={|r: &Reading| r.arterial} agreement={3.} color={rgb(0x179299)} class="h-72" /> }
}
```

### Methods swapped

Swapping a and b flips the sign of the differences.

```rust
#[component]
fn swapped() {
    let data = use_signal(readings(9));
    rsx! { <BlandAltman data={data} a={|r: &Reading| r.arterial} b={|r: &Reading| r.cuff} a_label="Arterial line" b_label="Cuff" class="h-72" /> }
}
```

### One-sided 95%

A small bias and 1.645 SD limits for a one-sided check.

```rust
#[component]
fn well_matched() {
    let data = use_signal(adjusted(10, 1., -3.));
    rsx! { <BlandAltman data={data} a={|r: &Reading| r.cuff} b={|r: &Reading| r.arterial} a_label="Oscillometric" b_label="Invasive" agreement={1.645} color={rgb(0x40a02b)} class="h-72" /> }
}
```
