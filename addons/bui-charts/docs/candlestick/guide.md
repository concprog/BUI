# Candlestick: choosing options

How to pick between the options in the [API reference](api.md). Every link goes to a runnable example; the gallery has them all (`cargo run --example gallery`, then **Candlestick**).

## Candlestick or Time Series?

| Your data | Use |
| --- | --- |
| open, high, low and close per period: prices, or min / max / first / last of any metric | `<Candlestick>` |
| one value per point in time | [`<TimeSeries>`](../time-series/guide.md) |

Candles work for more than prices: the min, max, first and last latency of each minute show the spread and the trend at once.

## Getting data in

- Your own rows: pass `x` and the part accessors you have. Only `close` is enough; candles then open at the previous close.
- A `Frame` with fields named open / high / low / close / volume (or o / h / l / c / v) needs no accessors at all.

Examples: [Candles and volume](examples.md#1-candles-and-volume), [From a frame](examples.md#8-from-a-frame), [Close only](examples.md#11-close-only).

## Display

| Goal | Use |
| --- | --- |
| price and activity together | the default `CandlesVolume` |
| price only, more room | `CandleMode::Candles` |
| activity only | `CandleMode::Volume` |
| a classic bar chart look | `CandleStyle::OhlcBars` |
| colours against the previous close, as many trading tools do | `ColorStrategy::CloseClose` |

Examples: [Candles](examples.md#2-candles), [Volume](examples.md#3-volume), [OHLC bars](examples.md#4-ohlc-bars), [Close to close](examples.md#5-close-to-close).

## Indicators and context

- Add moving averages, bands or targets as `<TrendSeries>` children; they get the full Trend styling.
- With a `Frame`, `include_all_fields` draws extra fields such as a precomputed average.
- Mark earnings, splits or outages with `annotations`.

Examples: [Moving average](examples.md#7-moving-average), [All fields](examples.md#9-all-fields), [Annotations](examples.md#10-annotations).

## Colours

Pick `up` and `down` colours that read for colour-blind viewers too, such as blue and orange.

Example: [Custom colours](examples.md#6-custom-colours).
