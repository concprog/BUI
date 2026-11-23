# Finance charts: examples

Every example here also runs in the gallery (`cargo run --example gallery`), under **Charts** in the sidebar.

The examples share these imports and this sample data:

```rust
use blenderui::prelude::*;
use bui_charts::*;

const DAY: f64 = 86_400_000.;
const T0: f64 = 1_729_000_000_000.;

pub fn noise(seed: u64) -> f64 {
    let mut z = seed.wrapping_add(0x9e37_79b9_7f4a_7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    (z ^ (z >> 31)) as f64 / u64::MAX as f64
}

#[derive(Clone, PartialEq)]
struct Bar {
    pub time: f64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

fn bars(count: u64, seed: u64, drift: f64) -> Vec<Bar> {
    let mut price = 100.;
    (0..count)
        .map(|index| {
            let n = seed * 10_000 + index * 5;
            let open = price;
            let close = open * (1. + (noise(n) - 0.5 + drift) * 0.05);
            let high = open.max(close) * (1. + noise(n + 1) * 0.015);
            let low = open.min(close) * (1. - noise(n + 2) * 0.015);
            price = close;
            Bar {
                time: T0 + index as f64 * DAY,
                open,
                high,
                low,
                close,
                volume: (3_000. + noise(n + 3) * 7_000.).round(),
            }
        })
        .collect()
}
```

## `<HeikinAshi>`

### Smoothed trend

A rising market: Heikin-Ashi candles stay one colour while the trend lasts.

```rust
#[component]
fn trend_days() {
    let data = use_signal(bars(80, 1, 0.08));
    rsx! {
        <HeikinAshi data={data} x={|b: &Bar| b.time} open={|b: &Bar| b.open} high={|b: &Bar| b.high} low={|b: &Bar| b.low} close={|b: &Bar| b.close} volume={|b: &Bar| b.volume} class="h-72" />
    }
}
```

### Sideways market

No trend: colours alternate and small bodies with long wicks appear.

```rust
#[component]
fn choppy() {
    let data = use_signal(bars(80, 2, 0.));
    rsx! {
        <HeikinAshi data={data} x={|b: &Bar| b.time} open={|b: &Bar| b.open} high={|b: &Bar| b.high} low={|b: &Bar| b.low} close={|b: &Bar| b.close} class="h-72" />
    }
}
```

### OHLC bars

OHLC bars with custom colours, no volume.

```rust
#[component]
fn ohlc() {
    let data = use_signal(bars(60, 3, -0.06));
    rsx! {
        <HeikinAshi data={data} x={|b: &Bar| b.time} open={|b: &Bar| b.open} high={|b: &Bar| b.high} low={|b: &Bar| b.low} close={|b: &Bar| b.close} candle_style={CandleStyle::OhlcBars} up={rgb(0x1e66f5)} down={rgb(0xfe640b)} class="h-72" />
    }
}
```

### Falling market

A steady decline stays red with volume underneath.

```rust
#[component]
fn selloff() {
    let data = use_signal(bars(80, 14, -0.1));
    rsx! {
        <HeikinAshi data={data} x={|b: &Bar| b.time} open={|b: &Bar| b.open} high={|b: &Bar| b.high} low={|b: &Bar| b.low} close={|b: &Bar| b.close} volume={|b: &Bar| b.volume} class="h-72" />
    }
}
```

### Candles without volume

Volume is supplied but the mode hides it.

```rust
#[component]
fn candles_only() {
    let data = use_signal(bars(120, 15, 0.04));
    rsx! {
        <HeikinAshi data={data} x={|b: &Bar| b.time} open={|b: &Bar| b.open} high={|b: &Bar| b.high} low={|b: &Bar| b.low} close={|b: &Bar| b.close} volume={|b: &Bar| b.volume} mode={CandleMode::Candles} class="h-64" />
    }
}
```

### Volume only

Only the volume bars are drawn.

```rust
#[component]
fn volume_only() {
    let data = use_signal(bars(60, 16, 0.02));
    rsx! {
        <HeikinAshi data={data} x={|b: &Bar| b.time} open={|b: &Bar| b.open} high={|b: &Bar| b.high} low={|b: &Bar| b.low} close={|b: &Bar| b.close} volume={|b: &Bar| b.volume} mode={CandleMode::Volume} class="h-48" />
    }
}
```

### No tooltip

Sideways candles with the tooltip hidden.

```rust
#[component]
fn quiet() {
    let data = use_signal(bars(100, 17, 0.));
    rsx! {
        <HeikinAshi data={data} x={|b: &Bar| b.time} open={|b: &Bar| b.open} high={|b: &Bar| b.high} low={|b: &Bar| b.low} close={|b: &Bar| b.close} tooltip={Tooltip::hidden()} class="h-72" />
    }
}
```

### Labelled price axis

A rising market with a price label on the y axis.

```rust
#[component]
fn price_axis() {
    let data = use_signal(bars(90, 18, 0.05));
    rsx! {
        <HeikinAshi data={data} x={|b: &Bar| b.time} open={|b: &Bar| b.open} high={|b: &Bar| b.high} low={|b: &Bar| b.low} close={|b: &Bar| b.close} y_axis={AxisConfig::new().label("Price ($)")} class="h-72" />
    }
}
```

### Custom colours

Teal for up candles, purple for down, with volume.

```rust
#[component]
fn purple_teal() {
    let data = use_signal(bars(150, 19, 0.01));
    rsx! {
        <HeikinAshi data={data} x={|b: &Bar| b.time} open={|b: &Bar| b.open} high={|b: &Bar| b.high} low={|b: &Bar| b.low} close={|b: &Bar| b.close} volume={|b: &Bar| b.volume} up={rgb(0x04a5e5)} down={rgb(0x8839ef)} class="h-80" />
    }
}
```

### Long history

Two hundred and fifty days drawn as OHLC bars.

```rust
#[component]
fn long_run() {
    let data = use_signal(bars(250, 20, 0.02));
    rsx! {
        <HeikinAshi data={data} x={|b: &Bar| b.time} open={|b: &Bar| b.open} high={|b: &Bar| b.high} low={|b: &Bar| b.low} close={|b: &Bar| b.close} candle_style={CandleStyle::OhlcBars} class="h-96" />
    }
}
```

## `<Renko>`

### Automatic box size

Bricks of 1/25 of the price range; time is ignored.

```rust
#[component]
fn bricks() {
    let data = use_signal(bars(200, 4, 0.03));
    rsx! { <Renko data={data} close={|b: &Bar| b.close} class="h-72" /> }
}
```

### Fixed box size

Each brick is $2; a reversal needs two boxes.

```rust
#[component]
fn fixed_box() {
    let data = use_signal(bars(200, 5, -0.02));
    rsx! { <Renko data={data} close={|b: &Bar| b.close} box_size={2.} class="h-72" /> }
}
```

### Custom colours

Small $1 bricks in custom colours.

```rust
#[component]
fn colours() {
    let data = use_signal(bars(200, 6, 0.));
    rsx! { <Renko data={data} close={|b: &Bar| b.close} box_size={1.} up={rgb(0x04a5e5)} down={rgb(0x8839ef)} class="h-72" /> }
}
```

### Half-dollar bricks

A $0.50 box shows every small move.

```rust
#[component]
fn fine() {
    let data = use_signal(bars(200, 21, 0.01));
    rsx! { <Renko data={data} close={|b: &Bar| b.close} box_size={0.5} class="h-72" /> }
}
```

### Five-dollar bricks

A $5 box over 300 days keeps only big moves.

```rust
#[component]
fn chunky() {
    let data = use_signal(bars(300, 22, 0.02));
    rsx! { <Renko data={data} close={|b: &Bar| b.close} box_size={5.} class="h-56" /> }
}
```

### No tooltip

A falling market with the tooltip hidden.

```rust
#[component]
fn quiet() {
    let data = use_signal(bars(200, 23, -0.03));
    rsx! { <Renko data={data} close={|b: &Bar| b.close} tooltip={Tooltip::hidden()} class="h-72" /> }
}
```

### Labelled price axis

$3 bricks with a price label on the y axis.

```rust
#[component]
fn price_axis() {
    let data = use_signal(bars(250, 24, 0.015));
    rsx! { <Renko data={data} close={|b: &Bar| b.close} box_size={3.} y_axis={AxisConfig::new().label("Price ($)")} class="h-72" /> }
}
```

### Long history

Five hundred days of a sideways market in $2 bricks.

```rust
#[component]
fn long_history() {
    let data = use_signal(bars(500, 25, 0.));
    rsx! { <Renko data={data} close={|b: &Bar| b.close} box_size={2.} class="h-96" /> }
}
```

### Amber and blue

Amber up bricks and blue down bricks.

```rust
#[component]
fn amber_blue() {
    let data = use_signal(bars(150, 26, 0.04));
    rsx! { <Renko data={data} close={|b: &Bar| b.close} up={rgb(0xdf8e1d)} down={rgb(0x1e66f5)} class="h-64" /> }
}
```

### Sharp decline

A steep fall with $4 bricks and deep red down bricks.

```rust
#[component]
fn crash() {
    let data = use_signal(bars(120, 27, -0.12));
    rsx! { <Renko data={data} close={|b: &Bar| b.close} box_size={4.} down={rgb(0xd20f39)} class="h-80" /> }
}
```

## `<Kagi>`

### Kagi

Thick yang lines above the last shoulder, thin yin lines below the last waist.

```rust
#[component]
fn lines() {
    let data = use_signal(bars(200, 7, 0.02));
    rsx! { <Kagi data={data} close={|b: &Bar| b.close} class="h-72" /> }
}
```

### Small reversal

A $1 reversal amount turns on every small swing.

```rust
#[component]
fn sensitive() {
    let data = use_signal(bars(200, 8, 0.));
    rsx! { <Kagi data={data} close={|b: &Bar| b.close} reversal={1.} class="h-72" /> }
}
```

### Large reversal

A $5 reversal keeps only the big moves.

```rust
#[component]
fn coarse() {
    let data = use_signal(bars(300, 9, -0.01));
    rsx! { <Kagi data={data} close={|b: &Bar| b.close} reversal={5.} yang={rgb(0x1e66f5)} yin={rgb(0x9ca0b0)} class="h-72" /> }
}
```

### Two-dollar reversal

A $2 reversal balances detail and noise.

```rust
#[component]
fn medium() {
    let data = use_signal(bars(250, 28, 0.01));
    rsx! { <Kagi data={data} close={|b: &Bar| b.close} reversal={2.} class="h-72" /> }
}
```

### Fifty-cent reversal

A $0.50 reversal over 150 days turns almost every day.

```rust
#[component]
fn hairline() {
    let data = use_signal(bars(150, 29, 0.));
    rsx! { <Kagi data={data} close={|b: &Bar| b.close} reversal={0.5} class="h-72" /> }
}
```

### Ten-dollar reversal

A $10 reversal over 500 days leaves only the main trend.

```rust
#[component]
fn major_moves() {
    let data = use_signal(bars(500, 30, 0.01));
    rsx! { <Kagi data={data} close={|b: &Bar| b.close} reversal={10.} class="h-72" /> }
}
```

### No tooltip

A falling market with the tooltip hidden.

```rust
#[component]
fn quiet() {
    let data = use_signal(bars(200, 31, -0.02));
    rsx! { <Kagi data={data} close={|b: &Bar| b.close} tooltip={Tooltip::hidden()} class="h-64" /> }
}
```

### Labelled price axis

A $3 reversal with a price label on the y axis.

```rust
#[component]
fn price_axis() {
    let data = use_signal(bars(300, 32, 0.02));
    rsx! { <Kagi data={data} close={|b: &Bar| b.close} reversal={3.} y_axis={AxisConfig::new().label("Price ($)")} class="h-72" /> }
}
```

### Green and red

Green yang lines and red yin lines.

```rust
#[component]
fn green_red() {
    let data = use_signal(bars(200, 33, 0.03));
    rsx! { <Kagi data={data} close={|b: &Bar| b.close} yang={rgb(0x40a02b)} yin={rgb(0xd20f39)} class="h-56" /> }
}
```

### Strong rally

A steep rise with amber yang lines and a $4 reversal.

```rust
#[component]
fn rally() {
    let data = use_signal(bars(120, 34, 0.1));
    rsx! { <Kagi data={data} close={|b: &Bar| b.close} reversal={4.} yang={rgb(0xdf8e1d)} class="h-80" /> }
}
```

## `<PointFigure>`

### Three-box reversal

X columns for rises and O columns for falls; a new column needs a three-box turn.

```rust
#[component]
fn classic() {
    let data = use_signal(bars(250, 11, 0.01));
    rsx! { <PointFigure data={data} close={|b: &Bar| b.close} class="h-72" /> }
}
```

### Small boxes

A $1 box and a two-box reversal show every swing.

```rust
#[component]
fn fine() {
    let data = use_signal(bars(250, 12, 0.));
    rsx! { <PointFigure data={data} close={|b: &Bar| b.close} box_size={1.} reversal={2} class="h-72" /> }
}
```

### Large boxes

A $4 box keeps only the major trend.

```rust
#[component]
fn coarse() {
    let data = use_signal(bars(400, 13, -0.005));
    rsx! { <PointFigure data={data} close={|b: &Bar| b.close} box_size={4.} up={rgb(0x1e66f5)} down={rgb(0xfe640b)} class="h-72" /> }
}
```

### Five-box reversal

A $2 box with a five-box reversal filters out noise.

```rust
#[component]
fn five_box() {
    let data = use_signal(bars(300, 35, 0.01));
    rsx! { <PointFigure data={data} close={|b: &Bar| b.close} box_size={2.} reversal={5} class="h-72" /> }
}
```

### One-box reversal

A single-box reversal flips columns on every move.

```rust
#[component]
fn one_box() {
    let data = use_signal(bars(200, 36, 0.));
    rsx! { <PointFigure data={data} close={|b: &Bar| b.close} box_size={2.} reversal={1} class="h-72" /> }
}
```

### Tiny boxes

A $0.50 box with a four-box reversal over a falling market.

```rust
#[component]
fn tight_boxes() {
    let data = use_signal(bars(250, 37, -0.01));
    rsx! { <PointFigure data={data} close={|b: &Bar| b.close} box_size={0.5} reversal={4} class="h-64" /> }
}
```

### Labelled price axis

$3 boxes with a price label on the y axis.

```rust
#[component]
fn price_axis() {
    let data = use_signal(bars(300, 38, 0.015));
    rsx! { <PointFigure data={data} close={|b: &Bar| b.close} box_size={3.} y_axis={AxisConfig::new().label("Price ($)")} class="h-72" /> }
}
```

### Long history

Six hundred days of a sideways market in $2 boxes.

```rust
#[component]
fn long_history() {
    let data = use_signal(bars(600, 39, 0.));
    rsx! { <PointFigure data={data} close={|b: &Bar| b.close} box_size={2.} reversal={3} class="h-96" /> }
}
```

### Green and red

Green X columns and red O columns.

```rust
#[component]
fn green_red() {
    let data = use_signal(bars(200, 40, 0.03));
    rsx! { <PointFigure data={data} close={|b: &Bar| b.close} up={rgb(0x40a02b)} down={rgb(0xd20f39)} class="h-56" /> }
}
```

### Breakout

A strong rise with $1.50 boxes and amber X columns.

```rust
#[component]
fn breakout() {
    let data = use_signal(bars(150, 41, 0.08));
    rsx! { <PointFigure data={data} close={|b: &Bar| b.close} box_size={1.5} reversal={4} up={rgb(0xdf8e1d)} class="h-80" /> }
}
```
