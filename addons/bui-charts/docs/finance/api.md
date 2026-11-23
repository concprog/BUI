# Finance charts: API reference

Price charts derived from your bars: `<HeikinAshi>`, `<Renko>`, `<Kagi>` and `<PointFigure>`. They compute their own rows from your data once per change, then draw through [`<Candlestick>`](../candlestick/api.md) or [`<Trend>`](../trend/api.md).

```rust
use blenderui::prelude::*;
use bui_charts::*;
```

- [`<HeikinAshi>`](#heikinashi)
- [`<Renko>`](#renko)
- [`<Kagi>`](#kagi)
- [`<PointFigure>`](#pointfigure)

Not sure which one fits? See the [guide](guide.md).

---

## `<HeikinAshi>`

```rust
<HeikinAshi data={bars} x={|b: &Bar| b.time}
    open={|b: &Bar| b.open} high={|b: &Bar| b.high} low={|b: &Bar| b.low}
    close={|b: &Bar| b.close} volume={|b: &Bar| b.volume} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `data` | `Vec<T>`, `Signal<Vec<T>>`, or your own [`Dataset`](../xy-chart/api.md#your-own-data-source) | **required** |
| `x` | `Fn(&T) -> f64`: the start of each period, in milliseconds since the epoch | **required** |
| `open`, `high`, `low`, `close` | `Fn(&T) -> f64` | **required** |
| `volume` | `Fn(&T) -> f64` | none |
| `mode` | [`CandleMode`](../candlestick/api.md#enums) | `CandlesVolume` |
| `candle_style` | [`CandleStyle`](../candlestick/api.md#enums) | `Candles` |
| `up`, `down` | colours | your theme's good and bad colours |
| `x_axis` | [`AxisConfig`](../xy-chart/api.md#axisconfig) | a time axis |
| `y_axis` | `AxisConfig` | `AxisConfig::new()` |
| `legend`, `tooltip`, `class` | as on `<Candlestick>` | |

Each candle is averaged with the one before it:

| Part | Formula |
| --- | --- |
| close | (open + high + low + close) / 4 |
| open | (previous open + previous close) / 2; the first candle uses (open + close) / 2 |
| high | the largest of high, open and close |
| low | the smallest of low, open and close |

## `<Renko>`

```rust
<Renko data={bars} close={|b: &Bar| b.close} box_size={2.} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `data` | as above | **required** |
| `close` | `Fn(&T) -> f64` | **required** |
| `box_size` | `f64`: the price move per brick | the price range / 25, rounded to 1, 2 or 5 × a power of ten |
| `up`, `down` | colours | good and bad |
| `y_axis`, `tooltip`, `class` | | |

A new brick is added each time the close moves a whole `box_size` past the last brick. Turning around takes two boxes. Bricks are evenly spaced: the x axis counts bricks, not time.

## `<Kagi>`

```rust
<Kagi data={bars} close={|b: &Bar| b.close} reversal={3.} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `data` | as above | **required** |
| `close` | `Fn(&T) -> f64` | **required** |
| `reversal` | `f64`: how far the price must turn back to start a new line | 2% of the mean close, rounded |
| `yang`, `yin` | colours of the thick (rising) and thin (falling) lines | good and bad |
| `y_axis`, `tooltip`, `class` | | |

The line keeps going while the price moves in the same direction and turns when it reverses by `reversal`. It turns thick (yang) when it rises above the last shoulder and thin (yin) when it falls below the last waist.

## `<PointFigure>`

```rust
<PointFigure data={bars} close={|b: &Bar| b.close} box_size={1.} reversal={3} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `data` | as above | **required** |
| `close` | `Fn(&T) -> f64` | **required** |
| `box_size` | `f64`: the price step of one box | the same default as `<Renko>` |
| `reversal` | `usize`: boxes needed to start a new column | `3` |
| `up`, `down` | colours of the X and O columns | good and bad |
| `y_axis`, `class` | | |

A column of X boxes grows while the price rises one whole box at a time. When it falls `reversal` boxes below the top, a new column of O boxes starts one box lower, and the reverse. Like Renko and Kagi it ignores time: the x axis counts columns.

The boxes stay square at every zoom: both axes zoom together. Ctrl and scroll zooms about the pointer, dragging a rectangle zooms into it, and double-click resets.

