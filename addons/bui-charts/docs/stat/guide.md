# Stat: choosing options

How to pick between the options in the [API reference](api.md). Every link goes to a runnable example; the gallery has them all (`cargo run --example gallery`, then **Stat**).

## Stat, Gauge or Bar Gauge?

| You want | Use |
| --- | --- |
| the number itself, as large as possible, maybe with its history | `<Stat>` |
| the number against a range, with thresholds marked | [`<Gauge>`](../gauge/guide.md) |
| several numbers compared against the same range | [`<BarGauge>`](../bar-gauge/guide.md) |

## One number or many

- One headline number: pass `value` and `x`, and pick the `reduce` calc: `Last` for "now", `Mean` or `Max` for "over this period", `Sum` for totals.
- One tile per field of a `Frame`: name the time field with `x_field` so it isn't shown as a value.
- One tile per row, such as one per service: `name`, `value` and `reduce={Reduce::All { limit: None }}`.

Examples: [Single value](examples.md#1-single-value), [Many values](examples.md#5-many-values), [Value mappings](examples.md#12-value-mappings).

## Drawing attention

| Goal | Use |
| --- | --- |
| a calm dashboard | `StatColorMode::Value`, colour only on the number |
| status at a glance across the room | `Background` or `BackgroundSolid` |
| no colour, just the number | `StatColorMode::None` |

Pair colour with `thresholds` so the tile changes as the number crosses limits, or with `mappings` for codes such as HTTP statuses.

Examples: [Background](examples.md#2-background), [Solid background](examples.md#3-solid-background).

## History and trend

- The sparkline needs several rows; `GraphMode::Line` keeps it light, `None` removes it.
- `show_percent_change` adds the change since the first row. For numbers where up is bad, use `PercentChangeColor::Inverted`.

Examples: [Line graph](examples.md#4-line-graph), [Percent change](examples.md#7-percent-change), [Inverted change](examples.md#8-inverted-change).

## Layout

- Tiles pick their own layout from their size. Give a single stat a wide box for "name left, value right", or a tall one to stack them.
- `wide_layout={false}` keeps names above values. `justify_mode={JustifyMode::Center}` centres everything.
- `BarOrientation::Horizontal` lists tiles as rows: good for a sidebar.
- Fix `title_size` and `value_size` when tiles in different panels should match.

Examples: [Horizontal list](examples.md#6-horizontal-list), [Centred](examples.md#10-centred), [Stacked layout](examples.md#11-stacked-layout), [Fixed text sizes](examples.md#13-fixed-text-sizes).
