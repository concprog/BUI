# Matrix charts: examples

Every example here also runs in the gallery (`cargo run --example gallery`), under **Charts** in the sidebar.

The examples share these imports and this sample data:

```rust
use blenderui::prelude::*;
use bui_charts::*;

use crate::distribution::normal;

#[derive(Clone, PartialEq)]
struct Sample {
    pub time: f64,
    pub value: f64,
}

fn chirp(seconds: f64, rate: f64) -> Vec<Sample> {
    let count = (seconds * rate) as u64;
    (0..count)
        .map(|index| {
            let t = index as f64 / rate;
            let sweep = 50. + 350. * t / seconds;
            Sample {
                time: t,
                value: (std::f64::consts::TAU * sweep * t * 0.5).sin()
                    + 0.5 * (std::f64::consts::TAU * 120. * t).sin()
                    + 0.2 * normal(index),
            }
        })
        .collect()
}
```

## `<Spectrogram>`

### Rising tone

A chirp from 50 to 400 Hz over a constant 120 Hz hum, in dB.

```rust
#[component]
fn sweep() {
    let data = use_signal(chirp(2., 1000.));
    rsx! { <Spectrogram data={data} x={|s: &Sample| s.time} value={|s: &Sample| s.value} class="h-72" /> }
}
```

### Longer windows

512-sample windows: sharper frequencies, blurrier time.

```rust
#[component]
fn fine() {
    let data = use_signal(chirp(2., 1000.));
    rsx! { <Spectrogram data={data} x={|s: &Sample| s.time} value={|s: &Sample| s.value} window_size={512} overlap={0.75} scheme={HeatmapScheme::Viridis} class="h-72" /> }
}
```

### Short windows

64-sample windows: sharper time, blurrier frequencies.

```rust
#[component]
fn coarse() {
    let data = use_signal(chirp(2., 1000.));
    rsx! { <Spectrogram data={data} x={|s: &Sample| s.time} value={|s: &Sample| s.value} window_size={64} overlap={0.} scheme={HeatmapScheme::Inferno} class="h-72" /> }
}
```

### Turbo scheme

The default window with a rainbow scheme that separates power levels.

```rust
#[component]
fn turbo() {
    let data = use_signal(chirp(2., 1000.));
    rsx! { <Spectrogram data={data} x={|s: &Sample| s.time} value={|s: &Sample| s.value} scheme={HeatmapScheme::Turbo} class="h-72" /> }
}
```

### Half a second

A short capture with 128-sample windows.

```rust
#[component]
fn burst() {
    let data = use_signal(chirp(0.5, 1000.));
    rsx! { <Spectrogram data={data} x={|s: &Sample| s.time} value={|s: &Sample| s.value} window_size={128} scheme={HeatmapScheme::Cividis} class="h-72" /> }
}
```

### 4 kHz sampling

A higher sample rate with 1024-sample windows and 90% overlap.

```rust
#[component]
fn audio() {
    let data = use_signal(chirp(1., 4000.));
    rsx! { <Spectrogram data={data} x={|s: &Sample| s.time} value={|s: &Sample| s.value} window_size={1024} overlap={0.9} scheme={HeatmapScheme::Magma} class="h-72" /> }
}
```

### Long recording

Six seconds at 500 Hz in greyscale.

```rust
#[component]
fn recording() {
    let data = use_signal(chirp(6., 500.));
    rsx! { <Spectrogram data={data} x={|s: &Sample| s.time} value={|s: &Sample| s.value} scheme={HeatmapScheme::Greys} class="h-72" /> }
}
```

### No overlap

Windows that do not repeat, so each time step shows as a block.

```rust
#[component]
fn blocks() {
    let data = use_signal(chirp(2., 1000.));
    rsx! { <Spectrogram data={data} x={|s: &Sample| s.time} value={|s: &Sample| s.value} window_size={128} overlap={0.} scheme={HeatmapScheme::Blues} class="h-72" /> }
}
```

### 95% overlap

Nearly every sample starts a new window for a smooth picture.

```rust
#[component]
fn smooth() {
    let data = use_signal(chirp(2., 1000.));
    rsx! { <Spectrogram data={data} x={|s: &Sample| s.time} value={|s: &Sample| s.value} overlap={0.95} scheme={HeatmapScheme::Purples} class="h-72" /> }
}
```

### Very long windows

2048-sample windows resolve the hum and the sweep as separate lines.

```rust
#[component]
fn resolved() {
    let data = use_signal(chirp(4., 2000.));
    rsx! { <Spectrogram data={data} x={|s: &Sample| s.time} value={|s: &Sample| s.value} window_size={2048} scheme={HeatmapScheme::Plasma} class="h-72" /> }
}
```

## `<AttentionMap>`

This section also uses:

```rust
fn sentence() -> Vec<SharedString> {
    ["The", "cat", "sat", "on", "the", "mat", "."]
        .map(SharedString::new_static)
        .to_vec()
}

fn weights(size: usize, causal: bool) -> Vec<f64> {
    let mut out = Vec::with_capacity(size * size);
    for line in 0..size {
        let raw: Vec<f64> = (0..size)
            .map(|column| {
                if causal && column > line {
                    return 0.;
                }
                let distance = (line as f64 - column as f64).abs();
                (-distance * 0.6).exp() + if column == 1 && line >= 2 { 0.8 } else { 0. }
            })
            .collect();
        let total: f64 = raw.iter().sum();
        out.extend(raw.iter().map(|value| value / total));
    }
    out
}
```

### Self-attention

Each token's attention over the sentence; values shown in the cells.

```rust
#[component]
fn self_attention() {
    rsx! { <AttentionMap tokens={sentence()} weights={weights(7, false)} class="h-80" /> }
}
```

### Causal mask

Tokens only attend to earlier tokens.

```rust
#[component]
fn causal() {
    rsx! { <AttentionMap tokens={sentence()} weights={weights(7, true)} scheme={HeatmapScheme::Purples} class="h-80" /> }
}
```

### Cross-attention

French queries attending to English keys.

```rust
#[component]
fn cross() {
    let queries = ["Le", "chat", "est", "assis"]
        .map(SharedString::new_static)
        .to_vec();
    let keys = ["The", "cat", "is", "sitting"]
        .map(SharedString::new_static)
        .to_vec();
    let matrix = vec![
        0.85, 0.05, 0.05, 0.05, 0.05, 0.88, 0.04, 0.03, 0.05, 0.05, 0.8, 0.1, 0.04, 0.06, 0.15,
        0.75,
    ];
    rsx! { <AttentionMap tokens={keys} queries={queries} weights={matrix} scheme={HeatmapScheme::Oranges} class="h-72" /> }
}
```

### Red scheme

The same self-attention in a warm sequential scheme.

```rust
#[component]
fn warm() {
    rsx! { <AttentionMap tokens={sentence()} weights={weights(7, false)} scheme={HeatmapScheme::Reds} class="h-80" /> }
}
```

### Without values

show_values off for a cleaner picture.

```rust
#[component]
fn quiet() {
    rsx! { <AttentionMap tokens={sentence()} weights={weights(7, false)} scheme={HeatmapScheme::Viridis} show_values={false} class="h-80" /> }
}
```

### Four tokens

A tiny causal map with large cells.

```rust
#[component]
fn short() {
    rsx! { <AttentionMap tokens={words(&["Yes", "we", "can", "!"])} weights={weights(4, true)} scheme={HeatmapScheme::Greens} class="h-64" /> }
}
```

### Thirteen tokens

Past 12 columns the values hide by themselves.

```rust
#[component]
fn long() {
    let tokens = words(&[
        "Attention",
        "is",
        "all",
        "you",
        "need",
        "to",
        "model",
        "long",
        "range",
        "links",
        "in",
        "a",
        "sequence",
    ]);
    rsx! { <AttentionMap tokens={tokens} weights={weights(13, true)} scheme={HeatmapScheme::Cividis} class="h-96" /> }
}
```

### Special tokens

A sentence pair with classification and separator tokens.

```rust
#[component]
fn labelled() {
    let tokens = words(&["[CLS]", "good", "movie", "[SEP]", "bad", "plot", "[SEP]"]);
    rsx! { <AttentionMap tokens={tokens} weights={weights(7, false)} scheme={HeatmapScheme::Magma} class="h-80" /> }
}
```

### Two queries

A rectangular map: two summary queries over seven keys.

```rust
#[component]
fn summary() {
    let queries = words(&["[CLS]", "[SUM]"]);
    let keys = words(&["Rain", "fell", "all", "night", "and", "rivers", "rose"]);
    let matrix = vec![
        0.05, 0.1, 0.05, 0.2, 0.05, 0.25, 0.3, 0.3, 0.1, 0.05, 0.1, 0.05, 0.1, 0.3,
    ];
    rsx! { <AttentionMap tokens={keys} queries={queries} weights={matrix} scheme={HeatmapScheme::Purples} class="h-48" /> }
}
```

### German to English

Word alignment with a Turbo scheme.

```rust
#[component]
fn alignment() {
    let queries = words(&["Ich", "habe", "Hunger"]);
    let keys = words(&["I", "am", "hungry", "."]);
    let matrix = vec![
        0.9, 0.04, 0.03, 0.03, 0.1, 0.6, 0.25, 0.05, 0.02, 0.08, 0.86, 0.04,
    ];
    rsx! { <AttentionMap tokens={keys} queries={queries} weights={matrix} scheme={HeatmapScheme::Turbo} class="h-64" /> }
}
```

## `<Waffle>`

This section also uses:

```rust
#[derive(Clone, PartialEq)]
pub struct Share {
    pub name: &'static str,
    pub value: f64,
}

fn mix(items: &[(&'static str, f64)]) -> Vec<Share> {
    items
        .iter()
        .map(|(name, value)| Share {
            name,
            value: *value,
        })
        .collect()
}
```

### Energy mix

100 squares, one per percent, filled in order.

```rust
#[component]
fn energy() {
    let data = use_signal(mix(&[
        ("Solar", 31.),
        ("Wind", 24.),
        ("Hydro", 17.),
        ("Gas", 19.),
        ("Coal", 9.),
    ]));
    rsx! { <Waffle data={data} name={|s: &Share| s.name} value={|s: &Share| s.value} class="h-72" /> }
}
```

### 5 × 20

A wide grid for a survey result.

```rust
#[component]
fn wide() {
    let data = use_signal(mix(&[("Yes", 62.), ("No", 31.), ("Undecided", 7.)]));
    rsx! { <Waffle data={data} name={|s: &Share| s.name} value={|s: &Share| s.value} rows={5} columns={20} class="h-40" /> }
}
```

### Small gaps

Task status with almost no gap.

```rust
#[component]
fn tight() {
    let data = use_signal(mix(&[
        ("Done", 46.),
        ("In progress", 21.),
        ("Blocked", 4.),
        ("To do", 29.),
    ]));
    rsx! { <Waffle data={data} name={|s: &Share| s.name} value={|s: &Share| s.value} gap={0.04} class="h-72" /> }
}
```

### Household budget

Seven categories with the legend on the right.

```rust
#[component]
fn budget() {
    let data = use_signal(mix(&[
        ("Housing", 34.),
        ("Food", 14.),
        ("Transport", 12.),
        ("Savings", 15.),
        ("Leisure", 9.),
        ("Health", 7.),
        ("Other", 9.),
    ]));
    rsx! { <Waffle data={data} name={|s: &Share| s.name} value={|s: &Share| s.value} legend={Legend::list().right()} class="h-72" /> }
}
```

### No legend or tooltip

Device split drawn as bare squares.

```rust
#[component]
fn quiet() {
    let data = use_signal(mix(&[("Mobile", 58.), ("Desktop", 37.), ("Tablet", 5.)]));
    rsx! { <Waffle data={data} name={|s: &Share| s.name} value={|s: &Share| s.value} legend={Legend::hidden()} tooltip={Tooltip::hidden()} class="h-72" /> }
}
```

### Seats

A 7 × 20 grid with 140 squares, one per seat.

```rust
#[component]
fn parliament() {
    let data = use_signal(mix(&[
        ("Left", 41.),
        ("Green", 18.),
        ("Centre", 27.),
        ("Right", 49.),
        ("Other", 5.),
    ]));
    rsx! { <Waffle data={data} name={|s: &Share| s.name} value={|s: &Share| s.value} rows={7} columns={20} class="h-56" /> }
}
```

### A day in hours

A 4 × 6 grid where each square is one hour.

```rust
#[component]
fn day() {
    let data = use_signal(mix(&[
        ("Sleep", 8.),
        ("Work", 9.),
        ("Commute", 1.5),
        ("Meals", 2.),
        ("Free time", 3.5),
    ]));
    rsx! { <Waffle data={data} name={|s: &Share| s.name} value={|s: &Share| s.value} rows={4} columns={6} gap={0.2} class="h-64" /> }
}
```

### Progress strip

A single row of twenty squares for 13 of 20 tests passed.

```rust
#[component]
fn strip() {
    let data = use_signal(mix(&[("Complete", 13.), ("Remaining", 7.)]));
    rsx! { <Waffle data={data} name={|s: &Share| s.name} value={|s: &Share| s.value} rows={1} columns={20} gap={0.25} legend={Legend::hidden()} class="h-16" /> }
}
```

### 20 × 20

400 squares with a hair-thin gap for vaccination coverage.

```rust
#[component]
fn dense() {
    let data = use_signal(mix(&[
        ("Vaccinated", 71.),
        ("Partly", 9.),
        ("Unvaccinated", 20.),
    ]));
    rsx! { <Waffle data={data} name={|s: &Share| s.name} value={|s: &Share| s.value} rows={20} columns={20} gap={0.02} class="h-96" /> }
}
```

### Wide gaps

Test results with gaps of 40% of a square.

```rust
#[component]
fn airy() {
    let data = use_signal(mix(&[("Pass", 83.), ("Fail", 11.), ("Skipped", 6.)]));
    rsx! { <Waffle data={data} name={|s: &Share| s.name} value={|s: &Share| s.value} gap={0.4} class="h-72" /> }
}
```
