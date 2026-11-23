# Candlestick: API reference

`<Candlestick>` draws price bars: each period's open, high, low and close as a candle or an OHLC bar, with trading volume along the bottom. It is a port of Grafana's Candlestick panel; see [Coming from Grafana](#coming-from-grafana).

```rust
use blenderui::prelude::*;
use bui_charts::*;
```

- [`<Candlestick>`](#candlestick)
- [Fields](#fields)
- [How candles are drawn](#how-candles-are-drawn)
- [Enums](#enums)
- [Coming from Grafana](#coming-from-grafana)

Not sure which option fits? See the [guide](guide.md).

---

## `<Candlestick>`

```rust
<Candlestick data={bars} x={|b: &Bar| b.time}
    open={|b: &Bar| b.open} high={|b: &Bar| b.high} low={|b: &Bar| b.low}
    close={|b: &Bar| b.close} volume={|b: &Bar| b.volume}>
    <TrendSeries name="SMA 20" y={|b: &Bar| b.sma} />
</Candlestick>
```

| Prop | Type | Default |
| --- | --- | --- |
| `data` | `Vec<T>`, `Signal<Vec<T>>`, `Frame<N>`, `Signal<Frame<N>>`, or your own [`Dataset`](../xy-chart/api.md#your-own-data-source) | **required** |
| `x` | `Fn(&T) -> f64`: the start of each period, in milliseconds since the epoch | for a `Frame`: the `x_field`, or the first field |
| `x_field` | field name, for a `Frame` | the first field |
| `open`, `high`, `low`, `close`, `volume` | `Fn(&T) -> f64` | for a `Frame`: found by name, see [Fields](#fields) |
| children | [`<TrendSeries>`](../trend/api.md#trendseries) overlays, such as moving averages | none |
| `mode` | [`CandleMode`](#enums) | `CandlesVolume` |
| `candle_style` | [`CandleStyle`](#enums) | `Candles` |
| `color_strategy` | [`ColorStrategy`](#enums) | `OpenClose` |
| `up`, `down`, `flat` | colours | your theme's good, bad and neutral colours |
| `include_all_fields` | `bool`: draw a `Frame`'s other fields as lines | `false` |
| `x_axis` | [`AxisConfig`](../xy-chart/api.md#axisconfig) | a time axis |
| `y_axis` | `AxisConfig` for the price axis | `AxisConfig::new()` |
| `legend` | [`Legend`](../xy-chart/api.md#legend) | lists the overlays |
| `tooltip` | [`Tooltip`](../trend/api.md#tooltip-legend-and-keyboard) | `Tooltip::multi()`: open, high, low, close and volume |
| `annotations` | [`Annotations`](../trend/api.md#annotations-exemplars-and-zoom) | none |
| `orientation`, `timezones`, `keyboard`, `on_select` | as on [`<TimeSeries>`](../time-series/api.md) | |
| `class`, `key` | | |

## Fields

With a `Frame` and no field accessors, fields are matched by name, ignoring case:

| Part | Field names |
| --- | --- |
| open | `open`, `o` |
| high | `high`, `h`, `max` |
| low | `low`, `l`, `min` |
| close | `close`, `c` |
| volume | `volume`, `v` |

Missing parts are filled in as Grafana does:

- **No open:** each candle opens at the previous close.
- **No close:** each candle closes at the next open.
- **No high or low:** the wick ends at the body.

`include_all_fields` draws every other field of the frame as a line on the price axis. In `Candles` mode an unused volume field becomes one of those lines; in `Volume` mode, high and low do.

## How candles are drawn

- **Width.** Bodies are 60% of the narrowest gap between periods. Wicks and OHLC ticks are 1 px wide when bodies are 12 px or narrower, otherwise 2 px.
- **Colour.** With `ColorStrategy::OpenClose`, a candle is up when it closes above its open. With `CloseClose`, it is up when it closes above the previous close; rising candles are then drawn hollow, outlined in their colour. Equal values count as up, except the first candle under `CloseClose`, which is flat.
- **Volume.** In `CandlesVolume` mode, volume bars sit in the bottom seventh of the chart on their own hidden scale (0 to 7 × the largest volume), coloured like their candle at half opacity. In `Volume` mode, they use the whole chart and the price axis.
- **Overlays.** `<TrendSeries>` children are drawn over the candles on the price axis and listed in the legend; the price parts themselves are not.

## Enums

| Enum | Variants |
| --- | --- |
| `CandleMode` | `Candles`, `CandlesVolume` (default), `Volume` |
| `CandleStyle` | `Candles` (default), `OhlcBars` |
| `ColorStrategy` | `OpenClose` (default), `CloseClose` |

## Coming from Grafana

| Grafana option | Here |
| --- | --- |
| Mode | `mode` |
| Candle style | `candle_style` |
| Color strategy | `color_strategy` |
| Up color, Down color, Flat color | `up`, `down`, `flat` |
| Open, High, Low, Close, Volume fields | `open`, `high`, `low`, `close`, `volume`, or field names |
| Include all fields | `include_all_fields`, or `<TrendSeries>` children |
| Legend, Tooltip, Annotations | `legend`, `tooltip`, `annotations` |

Not available here: the volume field's own fill opacity (volume is drawn at 50%) and tick labels on the volume axis.
