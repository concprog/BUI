# Finance charts: choosing a chart

How to pick between the charts in the [API reference](api.md). Every link goes to runnable examples; the gallery has them all (`cargo run --example gallery`, then **Charts**).

## Which chart

| Your question | Use |
| --- | --- |
| Exactly what happened in each period | [`<Candlestick>`](../candlestick/guide.md) |
| Which way is the trend, without the noise? | [`<HeikinAshi>`](examples.md#heikinashi) |
| Only moves of a certain size matter, not time | [`<Renko>`](examples.md#renko) |
| Where does the price turn, and when does supply or demand take over? | [`<Kagi>`](examples.md#kagi) |
| Where are support and resistance, with every move rounded to a box? | [`<PointFigure>`](examples.md#pointfigure) |

## Heikin-Ashi

Each candle is averaged with the one before it, so runs of one colour show a trend and small bodies with long wicks show indecision. The prices on the axis are averages, not real trades: use `<Candlestick>` when the exact open or close matters. It takes the same `mode` and `candle_style` as `<Candlestick>`, so `volume` and OHLC bars work too.

## Renko, Kagi and Point & Figure

Renko, Kagi and Point & Figure all ignore time: the x axis counts bricks, turns or columns.

- Pick `box_size` (Renko, Point & Figure) or `reversal` (Kagi) in price units. Point & Figure also takes `reversal` as a number of boxes, 3 by default. The defaults are round numbers: 1/25 of the price range for Renko, 2% of the average price for Kagi.
- Smaller sizes react sooner and show more noise; larger sizes show only big moves.
- Both take only `close`: feed them closes from any bar size, or individual trades.

Examples: [Renko](examples.md#renko), [Kagi](examples.md#kagi).

## On a dashboard

The **Trading desk** dashboard in the gallery puts candles, Heikin-Ashi and Renko next to each other on the same prices, with P&L, exposure and an efficient frontier.
