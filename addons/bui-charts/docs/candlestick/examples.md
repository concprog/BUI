# Candlestick: examples

Every example here also runs in the gallery:

```bash
cargo run --example gallery
```

Choose **Candlestick** in the sidebar. Not sure which example you need? Start from the [guide](guide.md).

Each example is a component; render it anywhere with `rsx! { <WithVolume /> }`.

The examples share these imports and this sample data:

```rust
use blenderui::prelude::*;
use bui_charts::*;

const T0: f64 = 1_729_000_000_000.;
const DAY: f64 = 86_400_000.;

fn noise(seed: u64) -> f64 {
    let mut z = seed.wrapping_add(0x9e37_79b9_7f4a_7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    (z ^ (z >> 31)) as f64 / u64::MAX as f64
}

#[derive(Clone, PartialEq)]
struct Bar {
    time: f64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
    average: f64,
}

fn bars(count: u64, seed: u64) -> Vec<Bar> {
    let mut price = 100.;
    let mut out: Vec<Bar> = Vec::with_capacity(count as usize);
    for index in 0..count {
        let n = seed * 10_000 + index * 5;
        let open = price;
        let close = open * (1. + (noise(n) - 0.48) * 0.06);
        let high = open.max(close) * (1. + noise(n + 1) * 0.02);
        let low = open.min(close) * (1. - noise(n + 2) * 0.02);
        let volume =
            (2_000. + noise(n + 3) * 6_000. * (1. + (close - open).abs() / open * 20.)).round();
        let window = out.iter().rev().take(9).map(|bar| bar.close).sum::<f64>() + close;
        let average = window / (out.len().min(9) + 1) as f64;
        out.push(Bar {
            time: T0 + index as f64 * DAY,
            open,
            high,
            low,
            close,
            volume,
            average,
        });
        price = close;
    }
    out
}

fn frame(seed: u64) -> Frame<6> {
    let rows: Vec<[f64; 6]> = bars(40, seed)
        .into_iter()
        .map(|bar| [bar.time, bar.open, bar.high, bar.low, bar.close, bar.volume])
        .collect();
    Frame::new(["time", "open", "high", "low", "close", "volume"]).with_rows(rows)
}

fn wide_frame(seed: u64) -> Frame<7> {
    let rows: Vec<[f64; 7]> = bars(40, seed)
        .into_iter()
        .map(|bar| {
            [
                bar.time,
                bar.open,
                bar.high,
                bar.low,
                bar.close,
                bar.volume,
                bar.average,
            ]
        })
        .collect();
    Frame::new(["time", "o", "h", "l", "c", "v", "sma"]).with_rows(rows)
}
```

## 1. Candles and volume

Daily candles with volume bars along the bottom.

```rust
#[component]
fn with_volume() {
    let data = use_signal(bars(60, 1));
    rsx! {
        <Candlestick
            data={data}
            x={|b: &Bar| b.time}
            open={|b: &Bar| b.open}
            high={|b: &Bar| b.high}
            low={|b: &Bar| b.low}
            close={|b: &Bar| b.close}
            volume={|b: &Bar| b.volume}
            class="h-72"
        />
    }
}
```

## 2. Candles

mode Candles: no volume.

```rust
#[component]
fn candles_only() {
    let data = use_signal(bars(60, 2));
    rsx! {
        <Candlestick
            data={data}
            x={|b: &Bar| b.time}
            open={|b: &Bar| b.open}
            high={|b: &Bar| b.high}
            low={|b: &Bar| b.low}
            close={|b: &Bar| b.close}
            volume={|b: &Bar| b.volume}
            mode={CandleMode::Candles}
            class="h-72"
        />
    }
}
```

## 3. Volume

mode Volume: only the volume bars, coloured by direction.

```rust
#[component]
fn volume_only() {
    let data = use_signal(bars(60, 3));
    rsx! {
        <Candlestick
            data={data}
            x={|b: &Bar| b.time}
            open={|b: &Bar| b.open}
            close={|b: &Bar| b.close}
            volume={|b: &Bar| b.volume}
            mode={CandleMode::Volume}
            class="h-72"
        />
    }
}
```

## 4. OHLC bars

candle_style OhlcBars: ticks for open and close.

```rust
#[component]
fn ohlc_bars() {
    let data = use_signal(bars(40, 4));
    rsx! {
        <Candlestick
            data={data}
            x={|b: &Bar| b.time}
            open={|b: &Bar| b.open}
            high={|b: &Bar| b.high}
            low={|b: &Bar| b.low}
            close={|b: &Bar| b.close}
            candle_style={CandleStyle::OhlcBars}
            mode={CandleMode::Candles}
            class="h-72"
        />
    }
}
```

## 5. Close to close

Coloured against the previous close; rising candles are hollow.

```rust
#[component]
fn close_close() {
    let data = use_signal(bars(40, 5));
    rsx! {
        <Candlestick
            data={data}
            x={|b: &Bar| b.time}
            open={|b: &Bar| b.open}
            high={|b: &Bar| b.high}
            low={|b: &Bar| b.low}
            close={|b: &Bar| b.close}
            volume={|b: &Bar| b.volume}
            color_strategy={ColorStrategy::CloseClose}
            class="h-72"
        />
    }
}
```

## 6. Custom colours

Blue for up, orange for down.

```rust
#[component]
fn custom_colors() {
    let data = use_signal(bars(60, 6));
    rsx! {
        <Candlestick
            data={data}
            x={|b: &Bar| b.time}
            open={|b: &Bar| b.open}
            high={|b: &Bar| b.high}
            low={|b: &Bar| b.low}
            close={|b: &Bar| b.close}
            volume={|b: &Bar| b.volume}
            up={rgb(0x1e66f5)}
            down={rgb(0xfe640b)}
            class="h-72"
        />
    }
}
```

## 7. Moving average

A TrendSeries overlay on the candles.

```rust
#[component]
fn moving_average() {
    let data = use_signal(bars(60, 7));
    rsx! {
        <Candlestick
            data={data}
            x={|b: &Bar| b.time}
            open={|b: &Bar| b.open}
            high={|b: &Bar| b.high}
            low={|b: &Bar| b.low}
            close={|b: &Bar| b.close}
            mode={CandleMode::Candles}
            class="h-72"
        >
            <TrendSeries name="SMA 10" y={|b: &Bar| b.average} color={rgb(0x8839ef)} line_width={2.} interpolation={Interpolation::Smooth} />
        </Candlestick>
    }
}
```

## 8. From a frame

Fields named open, high, low, close and volume are found by name.

```rust
#[component]
fn from_frame() {
    let data = use_signal(frame(8));
    rsx! { <Candlestick data={data} class="h-72" /> }
}
```

## 9. All fields

include_all_fields draws the extra sma field as a line.

```rust
#[component]
fn all_fields() {
    let data = use_signal(wide_frame(9));
    rsx! { <Candlestick data={data} include_all_fields={true} class="h-72" /> }
}
```

## 10. Annotations

An earnings marker and a highlighted period.

```rust
#[component]
fn earnings() {
    let data = use_signal(bars(60, 10));
    let notes = vec![
        Annotation::at(T0 + 20. * DAY, "Q3 earnings").text("Beat estimates by 4%"),
        Annotation::region(T0 + 38. * DAY, T0 + 44. * DAY, "Lock-up expiry").color(rgb(0xdf8e1d)),
    ];
    rsx! {
        <Candlestick
            data={data}
            x={|b: &Bar| b.time}
            open={|b: &Bar| b.open}
            high={|b: &Bar| b.high}
            low={|b: &Bar| b.low}
            close={|b: &Bar| b.close}
            volume={|b: &Bar| b.volume}
            annotations={notes}
            class="h-72"
        />
    }
}
```

## 11. Close only

Without open, each candle opens at the previous close.

```rust
#[component]
fn close_only() {
    let data = use_signal(bars(60, 11));
    rsx! {
        <Candlestick
            data={data}
            x={|b: &Bar| b.time}
            close={|b: &Bar| b.close}
            mode={CandleMode::Candles}
            class="h-72"
        />
    }
}
```

## 12. Drag to zoom

Drag across the chart; on_select reports the range.

```rust
#[component]
fn zoom() {
    let data = use_signal(bars(120, 12));
    let picked = use_signal(SharedString::Borrowed(
        "Drag across the chart to zoom. Double-click to zoom out.",
    ));
    rsx! {
        <div class="flex flex-col gap-2 h-72">
            <div class="text-xs">{picked.get()}</div>
            <Candlestick
                data={data}
                x={|b: &Bar| b.time}
                open={|b: &Bar| b.open}
                high={|b: &Bar| b.high}
                low={|b: &Bar| b.low}
                close={|b: &Bar| b.close}
                volume={|b: &Bar| b.volume}
                on_select={move |range: SelectedRange, _: &mut Window, _: &mut App| {
                    let days = (range.x.1 - range.x.0) / DAY;
                    picked.set(SharedString::from(format!("Zoomed to {days:.0} days")));
                }}
                class="flex-1"
            />
        </div>
    }
}
```

