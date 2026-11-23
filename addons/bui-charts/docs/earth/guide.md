# Earth and lab science charts: choosing a chart

How to pick between the charts in the [API reference](api.md). Every link goes to runnable examples; the gallery has them all (`cargo run --example gallery`, then **Charts**).

## Which chart

| Field | Your question | Use |
| --- | --- | --- |
| soils, alloys, mixtures, votes | How do three parts that add to a whole vary between samples? | [`<Ternary>`](examples.md#ternary) |
| materials, phase diagrams | How does a property change across every three-part mixture? | [`<TernaryContour>`](examples.md#ternarycontour) |
| hydrogeology | What type of water is each sample, and how do they mix? | [`<Piper>`](examples.md#piper) |
| meteorology | How stable is the atmosphere, and where will clouds form? | [`<Stuve>`](examples.md#stuve) or [`<Tephigram>`](examples.md#tephigram) |
| any field with regions | How does a value change across places? | [`<Choropleth>`](examples.md#choropleth) |

## Ternary diagrams

Pass raw amounts; each row is normalised, so counts, masses and percentages all work. Read a point's share of a corner from the grid lines parallel to the opposite side. Use `<TernaryContour>` when you have a formula or model rather than samples.

## Piper diagrams

Pass concentrations in meq/L, not mg/L: divide mg/L by the ion's equivalent weight (Ca 20.04, Mg 12.15, Na 22.99, K 39.10, HCO₃ 61.02, SO₄ 48.03, Cl 35.45). In the diamond, the bottom corner is Ca–HCO₃ water, the top is Na–Cl, the left is Ca–SO₄/Cl and the right is Na–HCO₃. Mixing between two waters shows as a straight line in every panel.

## Stüve or Tephigram?

Both take the same sounding.

- **Stüve**: easy to read and hand-draw, since dry adiabats are straight.
- **Tephigram**: preferred in the UK and Commonwealth. Areas are proportional to energy, so the area between the parcel path and the environment measures CAPE and CIN.

Pass levels in any order; they are sorted by pressure. Narrow `pressure_range` and `temperature_range` to study the boundary layer.

## Choropleths

Bring your own geometry: convert GeoJSON polygons to `Region`s once, keep them in a signal, and join values by name. Use `Mercator` for web-style maps, `Equirectangular` for world overviews, and `Plane` for tile maps, floor plans or schematic regions. Use a diverging scheme with a symmetric `range` for values around zero.
