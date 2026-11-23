# Diagram charts: choosing a chart

How to pick between the charts in the [API reference](api.md). Every link goes to runnable examples; the gallery has them all (`cargo run --example gallery`, then **Charts**).

## Which chart

| Your question | Use |
| --- | --- |
| How big is each market, and who owns what share of it? | [`<Mekko>`](examples.md#mekko) |
| Which of many hosts or services need attention right now? | [`<HoneyComb>`](examples.md#honeycomb) |
| How does a total split into parts, and parts of parts? | [`<Sunburst>`](examples.md#sunburst) |
| How much flows between every pair of groups? | [`<ChordDiagram>`](examples.md#chorddiagram) |
| Which items are linked, in a fixed order? | [`<ArcDiagram>`](examples.md#arcdiagram) |
| What could be causing a problem? | [`<Ishikawa>`](examples.md#ishikawa) |
| How do two or three sets overlap? | [`<Venn>`](examples.md#venn) |
| How do many sets overlap? | [`<UpSet>`](examples.md#upset) |
| Where do links run between positions on many segments, alongside tracks of values? | [`<Circos>`](examples.md#circos) |
| What share does each part take, shown as countable cells in a ring? | [`<WarpedWaffle>`](examples.md#warpedwaffle) |
| Which letters are conserved at each position of an alignment? | [`<SequenceLogo>`](examples.md#sequencelogo) |
| Which words or terms come up most? | [`<WordCloud>`](examples.md#wordcloud) |

## Mekko or stacked bars?

A Mekko adds one dimension to a 100% stacked bar: column width. Use it when the size of each column matters as much as its mix, for example market size and market share. When the columns are similar in size, a 100% stacked bar is easier to read.

## HoneyComb thresholds

Thresholds work as on `<BarGauge>`: absolute values, or percentages of `min`–`max`. Leave out `columns` and the grid picks the arrangement with the largest cells for the space. Values are shown inside each cell when it is large enough; otherwise hover.

## Sunburst or Icicle?

Both show the same tree. A sunburst is compact and good for a few levels. Use [`<Icicle>`](../flow/guide.md) when names are long or the tree is deep, since rectangles fit labels better than arcs. Click a slice to zoom into it.

## Chord, Sankey or Arc?

- `<ChordDiagram>`: flows between groups in both directions, all to all.
- [`<Sankey>`](../flow/guide.md): flows that go one way through stages.
- `<ArcDiagram>`: links when the node order means something, such as time or position. It also suits sparse graphs.

## Venn or UpSet?

Both take the same rows: one per exact combination (`"A&B"`), or one per item with the counts left out. Use `<Venn>` for two or three sets, where the shapes are exact. For four or more, or when you need exact combination sizes side by side, use `<UpSet>`.

## Ishikawa

One row per cause with its category. Classic category sets are the 6 Ms (Machine, Method, Material, Man, Measurement, Environment) for manufacturing and the 8 Ps for services. Keep causes short; long ones are shortened, and the full list is in the tooltip.

## Circos

Use it for genomes and for anything with long segments and links between positions on them, such as code modules or time ranges. Keep links under a few hundred and lower `link_opacity` when they overlap. Each track scales to its own maximum, so compare shapes within a track rather than heights between tracks.

## Sequence logos

Align the sequences first; the logo reads column by column. Bits show how conserved a position is (2 bits is fully conserved DNA). Probability shows the letter mix even where nothing is conserved. With fewer than about 20 sequences, keep `correction` on so small samples do not look more conserved than they are.

## Word clouds

Word clouds are good for a first impression and poor for comparison: font size and word length both affect how big a word looks. Use a bar chart when the numbers matter. Pass already-counted terms; remove stop words before counting.
