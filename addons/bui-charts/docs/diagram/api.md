# Diagram charts: API reference

Charts drawn from shapes rather than axes: `<Mekko>`, `<HoneyComb>`, `<Sunburst>`, `<ChordDiagram>`, `<ArcDiagram>`, `<Ishikawa>`, `<Venn>`, `<UpSet>`, `<Circos>`, `<WarpedWaffle>`, `<SequenceLogo>` and `<WordCloud>`. They share one painter, so they all work the same way:

- hovering a shape fades the others and shows a tooltip;
- clicking a legend entry hides it (where there is a legend);
- the layout is recomputed only when the data, the size, the hidden set or the theme changes.

```rust
use blenderui::prelude::*;
use bui_charts::*;
```

- [`<Mekko>`](#mekko)
- [`<HoneyComb>`](#honeycomb)
- [`<Sunburst>`](#sunburst)
- [`<ChordDiagram>`](#chorddiagram)
- [`<ArcDiagram>`](#arcdiagram)
- [`<Ishikawa>`](#ishikawa)
- [`<Venn>`](#venn)
- [`<UpSet>`](#upset)
- [`<Circos>`](#circos)
- [`<WarpedWaffle>`](#warpedwaffle)
- [`<SequenceLogo>`](#sequencelogo)
- [`<WordCloud>`](#wordcloud)

Not sure which one fits? See the [guide](guide.md). `data` below takes `Vec<T>`, `Signal<Vec<T>>`, or your own [`Dataset`](../xy-chart/api.md#your-own-data-source). Every chart also takes `tooltip` ([`Tooltip`](../xy-chart/api.md#tooltip)) and `class`.

---

## `<Mekko>`

```rust
<Mekko data={regions} segments={brands} name={|c: &Column| c.name} value={|c: &Column, i: usize| c.parts[i]} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `data` | one row per column | **required** |
| `segments` | `Vec<SharedString>`: the stacked parts, bottom first | **required** |
| `name` | `Fn(&T) -> &str` | **required** |
| `value` | `Fn(&T, usize) -> f64`: the row's value for segment `index` | **required** |
| `show_values` | `bool`: share of the column, written inside blocks that fit | `true` |
| `gap` | `f32`: pixels between columns | `2.` |
| `legend` | [`Legend`](../xy-chart/api.md#legend) | shown |

Each column's width is its share of the grand total; its blocks split it by segment. Below each column: its name and its share of the total. The tooltip shows the value, the share of the column and the share of the total.

## `<HoneyComb>`

```rust
<HoneyComb data={servers} name={|s: &Server| s.name.as_ref()} value={|s: &Server| s.cpu} thresholds={steps} unit="%" />
```

| Prop | Type | Default |
| --- | --- | --- |
| `data` | one row per cell | **required** |
| `name`, `value` | `Fn(&T) -> &str`, `Fn(&T) -> f64` | **required** |
| `thresholds` | [`Thresholds`](../bar-gauge/api.md) | good below 80, bad above |
| `min`, `max` | `f64`: the range for percentage thresholds | `0.`, `100.` |
| `decimals` | `usize` | as many as needed |
| `unit` | text after the value | none |
| `columns` | `usize`: cells per row | the count that gives the largest cells |

Hexagons in offset rows, coloured by threshold, with the value and name inside when they fit. Missing values are greyed out.

## `<Sunburst>`

```rust
<Sunburst data={files} path={|f: &File| f.path} value={|f: &File| f.bytes} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `data` | one row per leaf | **required** |
| `path` | `Fn(&T) -> &str`: `"a/b/c"` | **required** |
| `value` | `Fn(&T) -> f64` | **required** |
| `separator` | text between levels | `"/"` |
| `root` | label of the centre | `"Total"` |
| `levels` | `usize`: rings drawn at once | all |

Rows with the same path add up; parents are the sum of their children. Colours follow the top-level branch and get lighter with depth. Click a slice to make it the centre; click the centre to go back up. The tooltip shows the value, the share of the parent and the share of the total.

## `<ChordDiagram>`

```rust
<ChordDiagram names={regions} matrix={flows} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `names` | `Vec<SharedString>` | **required** |
| `matrix` | `Vec<f64>`, row by row: `matrix[i * n + j]` flows from `i` to `j` | **required** |
| `pad_angle` | `f32`: degrees between groups | `2.` |
| `ribbon_opacity` | `f32` | `0.65` |
| `legend` | `Legend` | hidden; names are written around the circle |

Each group's arc is its outflow. One ribbon joins each pair; its two ends are the flows each way, and it takes the colour of the larger side. The diagonal draws ribbons back into the same group.

Zoom: hold Ctrl (Cmd on macOS) and scroll, or pinch on a trackpad, to zoom about the pointer. Drag to pan once zoomed in, and double-click to reset.

## `<ArcDiagram>`

```rust
<ArcDiagram data={links} source={|l: &Link| l.from} target={|l: &Link| l.to} value={|l: &Link| l.weight} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `data` | one row per link | **required** |
| `source`, `target` | `Fn(&T) -> &str`: node names | **required** |
| `value` | `Fn(&T) -> f64`: arc width | `1.` per link |

Nodes sit on a line in the order they first appear, sized by their total weight. Arcs are half-circles above the line, flattened when they would not fit. Labels alternate between two rows when they would overlap.

Zoom: hold Ctrl (Cmd on macOS) and scroll, or pinch, to zoom about the pointer. Drag a rectangle to zoom into it, Shift and drag to pan, and double-click to reset.

## `<Ishikawa>`

```rust
<Ishikawa data={causes} effect="Oversized bores" category={|c: &Cause| c.group} cause={|c: &Cause| c.reason} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `data` | one row per cause | **required** |
| `effect` | text in the head | **required** |
| `category` | `Fn(&T) -> &str`: the bone | **required** |
| `cause` | `Fn(&T) -> &str` | **required** |

Bones alternate above and below the spine in the order categories first appear. Long causes are shortened with "…"; hover a bone to list all its causes.

## `<Venn>`

```rust
<Venn data={users} sets={|u: &Use| u.sets} value={|u: &Use| u.count} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `data` | one row per exact combination, such as `"A"` or `"A&B"`, or one row per item | **required** |
| `sets` | `Fn(&T) -> &str`: the sets the row belongs to | **required** |
| `value` | `Fn(&T) -> f64` | `1.` per row |
| `separator` | text between set names | `"&"` |
| `fill_opacity` | `f32` | `0.25` |
| `legend` | `Legend` | hidden; names are written next to the circles |

Rows with the same combination add up. Circle areas follow the set sizes, and each pair is placed so its overlap area matches the shared count. Up to three sets are exact where circles allow; more sets are placed as close as possible. Each region shows the count of items in exactly that combination.

## `<UpSet>`

```rust
<UpSet data={items} sets={|i: &Item| i.sets} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `data`, `sets`, `value`, `separator` | as on `<Venn>` | |
| `max_intersections` | `usize`: combination bars shown, largest first | `20` |
| `min_size` | `f64`: hide smaller combinations | `0.` |

Bars at the top are exact combinations, the dot matrix below shows which sets each one contains, and the bars on the left are set sizes. Use it when there are too many sets for a Venn diagram.

## `<Circos>`

```rust
<Circos segments={chromosomes} links={links} tracks={vec![coverage]} ticks={50.} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `segments` | `Vec<(SharedString, f64)>`: name and length, clockwise from the top | **required** |
| `links` | `Vec<CircosLink>`: `{ source, from: (start, end), target, to: (start, end), weight }` | none |
| `tracks` | `Vec<Vec<CircosBin>>`: one ring of bars per inner vector; `{ segment, start, end, value }` | none |
| `gap` | `f32`: degrees between segments | `2.` |
| `ticks` | `f64`: tick spacing in segment units | none |
| `link_opacity` | `f32` | `0.55` |

Segments form the outer ring. Each track is a ring of bars scaled to its own maximum, outermost first. Links with zero-width ranges draw as curves, sized by weight; links over ranges draw as ribbons. Everything has a tooltip.

Zoom: hold Ctrl (Cmd on macOS) and scroll, or pinch on a trackpad, to zoom about the pointer. Drag to pan once zoomed in, and double-click to reset.

## `<WarpedWaffle>`

```rust
<WarpedWaffle data={mix} name={|s: &Share| s.name} value={|s: &Share| s.value} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `data`, `name`, `value` | as on [`<Waffle>`](../matrix/api.md#waffle) | **required** |
| `rings`, `sectors` | `usize`: the grid is `rings × sectors` cells | `5`, `20` |
| `hole` | `f32`: inner radius as a share of the outer | `0.35` |
| `gap` | `f32`: space between cells, as a share of a cell | `0.12` |
| `legend` | `Legend` | shown |

A waffle bent into a ring. Cells fill sector by sector, so each category takes a slice. Shares use the same largest-remainder rounding as `<Waffle>`. Hidden categories give their cells to the rest. The centre shows the total.

## `<SequenceLogo>`

```rust
<SequenceLogo data={alignment} sequence={|r: &Read| r.sequence.as_str()} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `data` | one row per aligned sequence | **required** |
| `sequence` | `Fn(&T) -> &str` | **required** |
| `alphabet` | `LogoAlphabet::Auto`, `Dna`, `Rna` or `Protein` | `Auto` |
| `units` | `LogoUnits::Bits` or `Probability` | `Bits` |
| `start` | `i64`: the first position's number | `1` |
| `correction` | `bool`: small-sample correction | `true` |

Each column stacks its letters, largest on top. In bits, a column's height is its information content, log₂(alphabet size) − entropy − correction, and letters split it by frequency. DNA and RNA letters are vector glyphs stretched to their height. Amino acids are coloured by chemistry. Gaps and unknown letters are skipped.

## `<WordCloud>`

```rust
<WordCloud data={terms} word={|t: &Term| t.word} weight={|t: &Term| t.count} group={|t: &Term| t.topic} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `word` | `Fn(&T) -> &str` | **required** |
| `weight` | `Fn(&T) -> f64` | **required** |
| `group` | `Fn(&T) -> &str`: colour and legend | words cycle through the palette |
| `max_words` | `usize`: the heaviest are kept | `120` |
| `font_range` | `(f32, f32)`: smallest and largest font size | `(11., 52.)` |
| `legend` | `Legend` | shown with a `group` |

Font size grows with the square root of weight. Words are placed heaviest first along a spiral from the centre, stretched to the chart's shape, and skipped if they don't fit. Words are horizontal.

Zoom: hold Ctrl (Cmd on macOS) and scroll, or pinch on a trackpad, to zoom about the pointer. Drag to pan once zoomed in, and double-click to reset.

