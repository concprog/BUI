# Science and engineering charts: choosing a chart

How to pick between the charts in the [API reference](api.md). Every link goes to runnable examples; the gallery has them all (`cargo run --example gallery`, then **Charts**).

## Which chart

| Field | Your question | Use |
| --- | --- | --- |
| electronics | What does the waveform look like, held still? | [`<Oscilloscope>`](examples.md#oscilloscope) |
| finance | Which mix of assets gives the best return for its risk? | [`<EfficientFrontier>`](examples.md#efficientfrontier) |
| dynamics | Where does a 2-D system go from each start? | [`<PhasePortrait>`](examples.md#phaseportrait) |
| dynamics, physiology | Is a sequence steady, periodic or chaotic? (heart-rate RR intervals) | [`<PoincareMap>`](examples.md#poincaremap) |
| dynamics | How does long-run behaviour change with a parameter? | [`<Bifurcation>`](examples.md#bifurcation) |
| astronomy | Where do stars sit by temperature and brightness? | [`<HrDiagram>`](examples.md#hrdiagram) |
| biochemistry | Which backbone angles do the residues take? | [`<Ramachandran>`](examples.md#ramachandran) |
| geology | How do oxides change with silica in a rock suite? | [`<Harker>`](examples.md#harker) |
| control | Where do the closed-loop poles move as gain rises? | [`<RootLocus>`](examples.md#rootlocus) |
| antennas | How does gain change with direction? | [`<RadiationPattern>`](examples.md#radiationpattern) |
| fluids, fields | Which way and how fast does it flow at each point? | [`<Quiver>`](examples.md#quiver) |
| meteorology | How does the wind change with height? | [`<Hodograph>`](examples.md#hodograph) |
| RF | How well is a load matched across frequency? | [`<SmithChart>`](examples.md#smithchart) |
| high-speed links | How open is the eye, and how much timing jitter is there? | [`<EyeDiagram>`](examples.md#eyediagram) |

## Computed charts and performance

`<EfficientFrontier>`, `<PhasePortrait>`, `<Bifurcation>` and `<RootLocus>` compute their points from props rather than rows. The result is kept until a prop changes, so redraws, hover and resizing cost nothing extra. The defaults are sized for instant results; raise `portfolios`, `steps`, `keep` or `duration` for print-quality detail.

Pass closures (`field`, `map`) that are pure: the chart calls them many thousands of times and only when its numeric props change.

## Oscilloscope

- Without `trigger` the trace scrolls with your data. With `trigger`, each update lines the wave up on the first rising crossing of that level, as a real scope does.
- `span` sets the time per screen. The first child is the trigger channel.

## Root locus

Coefficients are highest power first: s² + 3s + 2 is `vec![1., 3., 2.]`. Branches start at the open-loop poles (squares) and end at the zeros (circles) or run off to infinity.

## Equal aspect

`<Quiver>`, `<Hodograph>` and `<SmithChart>` keep one unit the same length on both axes, so circles stay round and angles stay true. The plot shrinks to fit and is centred. Turn it off for a quiver with `equal_aspect={false}` when x and y have different units.

## Radiation pattern

Pass gain in dB. Patterns are often normalised to 0 dB at their peak, which the default range of 0 down to −40 dB suits. Raise `min` to −20 for broad patterns so small differences show. Angles can be sampled at any spacing; hover snaps to the nearest sample.

## Smith chart

Pass resistance and reactance in ohms with your system's `impedance` (50 Ω by default). The closer the trace is to the centre, the better the match. Circles of constant resistance and arcs of constant reactance are drawn for the values in `grid`.

## Eye diagram

Set `period` to one unit interval and `offset` so the crossings sit where you want them (half a period puts one eye in the middle). The eye height is the noise margin and the histogram width is the jitter. For PAM4 or other multi-level signals, pass the middle `threshold` and read the three eyes.

Use `overlay={SmithGrid::Both}` on a Smith chart when designing matching networks with both series and shunt parts.
