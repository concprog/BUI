# bui-charts

Charts for BUI's Rust binding, [blenderui](https://github.com/ppmpreetham/BUI). Each chart is a component: write it in `rsx!` like any other tag, style it with `class`, and feed it from your own state.

| Chart | Use it for | Docs |
| --- | --- | --- |
| `<XYChart>` | numeric x against numeric y: scatter plots, bubbles, paths | [Guide](xy-chart/guide.md) · [API](xy-chart/api.md) · [Examples](xy-chart/examples.md) |
| `<BarGauge>` | how full each value is between a minimum and a maximum | [Guide](bar-gauge/guide.md) · [API](bar-gauge/api.md) · [Examples](bar-gauge/examples.md) |
| `<FlameGraph>` | CPU, memory and time profiles, and before/after comparisons | [Guide](flame-graph/guide.md) · [API](flame-graph/api.md) · [Examples](flame-graph/examples.md) |
| `<Trend>` | series against a numeric x that only goes up: speed, distance, load | [Guide](trend/guide.md) · [API](trend/api.md) · [Examples](trend/examples.md) |
| `<Traces>` | one distributed trace: spans across services on a timeline | [Guide](traces/guide.md) · [API](traces/api.md) · [Examples](traces/examples.md) |
| `<Heatmap>` | how values are spread over time and value: latency, sizes, day × hour grids | [Guide](heatmap/guide.md) · [API](heatmap/api.md) · [Examples](heatmap/examples.md) |
| `<StateTimeline>` | which state each thing was in over time: up/down, modes, levels | [Guide](state-timeline/guide.md) · [API](state-timeline/api.md) · [Examples](state-timeline/examples.md) |
| `<StatusHistory>` | a grid of samples: checks, jobs or totals, one cell each | [Guide](status-history/guide.md) · [API](status-history/api.md) · [Examples](status-history/examples.md) |
| `<Histogram>` | how values are spread: counts per bucket | [Guide](histogram/guide.md) · [API](histogram/api.md) · [Examples](histogram/examples.md) |
| `<Stat>` | one big number per value, with its history and change | [Guide](stat/guide.md) · [API](stat/api.md) · [Examples](stat/examples.md) |
| `<Gauge>` | a value on an arc or ring against its range and thresholds | [Guide](gauge/guide.md) · [API](gauge/api.md) · [Examples](gauge/examples.md) |
| `<TimeSeries>` | values over time, with annotations, exemplars and time compare | [Guide](time-series/guide.md) · [API](time-series/api.md) · [Examples](time-series/examples.md) |
| `<Candlestick>` | open, high, low and close per period, with volume | [Guide](candlestick/guide.md) · [API](candlestick/api.md) · [Examples](candlestick/examples.md) |
| `<NodeGraph>` | services and the calls between them, with health arcs | [Guide](node-graph/guide.md) · [API](node-graph/api.md) · [Examples](node-graph/examples.md) |

### Derived charts

Built from the charts above, so they share their hover, tooltips, legends and theming.

| Charts | Use them for | Docs |
| --- | --- | --- |
| `<Lollipop>` `<Dumbbell>` `<Slope>` `<Bump>` `<BackToBack>` `<Waterfall>` `<Bullet>` `<Funnel>` `<Forest>` | one bar, dot or line per named item | [Guide](category/guide.md) · [API](category/api.md) · [Examples](category/examples.md) |
| `<HeikinAshi>` `<Renko>` `<Kagi>` `<PointFigure>` | price trends without the noise | [Guide](finance/guide.md) · [API](finance/api.md) · [Examples](finance/examples.md) |
| `<Pareto>` `<ControlChart>` `<BlandAltman>` | causes, process stability, method agreement | [Guide](quality/guide.md) · [API](quality/api.md) · [Examples](quality/examples.md) |
| `<Barcode>` `<Beeswarm>` `<Ridgeline>` `<Horizon>` | every value, distribution shapes, dense series | [Guide](distribution/guide.md) · [API](distribution/api.md) · [Examples](distribution/examples.md) |
| `<Spectrogram>` `<AttentionMap>` `<Waffle>` | frequencies over time, weight matrices, shares of a whole | [Guide](matrix/guide.md) · [API](matrix/api.md) · [Examples](matrix/examples.md) |
| `<Gantt>` `<Sankey>` `<Alluvial>` `<Icicle>` `<Dendrogram>` `<Spider>` `<NightingaleRose>` `<ParallelCoordinates>` `<SankeyFunnel>` | plans, flows, funnels, trees, clusters and multi-measure profiles | [Guide](flow/guide.md) · [API](flow/api.md) · [Examples](flow/examples.md) |
| `<Oscilloscope>` `<EfficientFrontier>` `<PhasePortrait>` `<PoincareMap>` `<Bifurcation>` `<HrDiagram>` `<Ramachandran>` `<Harker>` `<RootLocus>` `<RadiationPattern>` `<Quiver>` `<Hodograph>` `<SmithChart>` `<EyeDiagram>` | electronics, finance, dynamics, astronomy, biochemistry, geology, control, antennas, fields, weather, RF, high-speed links | [Guide](science/guide.md) · [API](science/api.md) · [Examples](science/examples.md) |
| `<Mekko>` `<HoneyComb>` `<Sunburst>` `<ChordDiagram>` `<ArcDiagram>` `<Ishikawa>` `<Venn>` `<UpSet>` `<Circos>` `<WarpedWaffle>` `<SequenceLogo>` `<WordCloud>` | market maps, status grids, hierarchies, flows, links, root causes, set overlaps, genomes, alignments, text | [Guide](diagram/guide.md) · [API](diagram/api.md) · [Examples](diagram/examples.md) |
| `<SankeyMatrix>` `<TreemapForce>` | flows between two sets with exact values, and grouped networks sized by value | [Guide](hybrid/guide.md) · [API](hybrid/api.md) · [Examples](hybrid/examples.md) |
| `<Ternary>` `<TernaryContour>` `<Piper>` `<Stuve>` `<Tephigram>` `<Choropleth>` | three-part mixtures, water chemistry, atmospheric soundings, maps | [Guide](earth/guide.md) · [API](earth/api.md) · [Examples](earth/examples.md) |

Also see [Theming](theming.md) for chart colours and light/dark mode.

The pictures in these pages were rendered by zopra-charts, the GPUI library bui-charts is ported from; bui-charts draws the same charts with BUI's kit (the gallery's `--screenshot` mode renders them).

## Install

```toml
[dependencies]
blenderui = { path = "../bindings/rust/blenderui" }  # inside a BUI checkout
bui-charts = { path = "../addons/bui-charts" }
```

Keep the dependency named `blenderui`: `rsx!` and the chart components expand to `::blenderui::..` paths.

## Quick start

```rust
use blenderui::prelude::*;
use bui_charts::*;

#[derive(Clone, PartialEq)]
struct Reading {
    temperature: f64,
    load: f64,
}

#[component]
fn dashboard() {
    let readings = use_signal(vec![
        Reading { temperature: 18., load: 32. },
        Reading { temperature: 24., load: 51. },
        Reading { temperature: 31., load: 77. },
    ]);
    rsx! {
        <div class="flex flex-col gap-4">
            <XYChart data={readings} class="h-72">
                <Series name="Load" x={|r: &Reading| r.temperature} y={|r| r.load} />
            </XYChart>
            <BarGauge data={readings} value={|r: &Reading| r.load} class="h-16" />
        </div>
    }
}
```

Things that hold for every chart:

1. **Keep data in a signal** (`use_signal`) and pass the signal. The chart then does its work only when the data changes, not on every render of your component.
2. **Accessors read from a row.** Write the row type on the first closure of a tag (`|r: &Reading|`). Accessors that return text return a borrowed `&str`, so nothing is cloned.
3. **Build colour lists once**, in a `static` or `const`, rather than with `vec![..]` in your component.
4. **Props on a chart are defaults; props on its children override them.**
5. **Zoom with Ctrl+scroll or a pinch** (BUI reports Cmd as Ctrl on macOS, and a pinch as Ctrl + wheel) on charts with axes: XY, Trend, Time Series, Candlestick, Heatmap and the charts built on them. On time-based charts and heatmaps you can also drag across a range. Double-click resets. In a bui-panels workspace, give the view `gestures={GestureOwner::Exclusive}` so the chart gets the gesture instead of the workspace. Inside a scrolling container the wheel also scrolls it (blenderui scrolls after its listeners run).
6. **Render the chart's tag in a component or a view's `render`.** Components take no `window` or `cx`; a chart reads its theme and data while rendering.

## See every example

```bash
cargo run --example gallery
```

Headless, one chart's page to a PPM image: `cargo run --example gallery -- --screenshot gauge.ppm --chart "Gauge" [--dark] [--size 1280x900] [--hover 400,300]`.

Headless, one chart's page to a PPM image: `cargo run --example gallery -- --screenshot gauge.ppm --chart "Gauge" [--dark] [--size 1280x900] [--hover 400,300]`.

Headless, one chart's page to a PPM image: `cargo run --example gallery -- --screenshot gauge.ppm --chart "Gauge" [--dark] [--size 1280x900] [--hover 400,300]`.

The gallery sidebar has three groups. **Dashboards** shows real-life dashboards (a checkout service, a trading desk and a factory floor) in movable, tabbed panels. **Base** lists the core charts the others are built on. **Charts** lists every other chart. Pick one to see all its examples running side by side.
