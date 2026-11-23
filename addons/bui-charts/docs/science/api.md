# Science and engineering charts: API reference

Charts that compute what they plot: `<Oscilloscope>`, `<EfficientFrontier>`, `<PhasePortrait>`, `<PoincareMap>`, `<Bifurcation>`, `<HrDiagram>`, `<Ramachandran>`, `<Harker>`, `<RootLocus>`, `<RadiationPattern>`, `<Quiver>`, `<Hodograph>`, `<SmithChart>` and `<EyeDiagram>`. The oscilloscope draws through [`<Trend>`](../trend/api.md) and the radiation pattern through the polar painter shared with [`<Spider>`](../flow/api.md#spider); the rest draw through [`<XYChart>`](../xy-chart/api.md). The work runs once per change of data or props, not on every frame.

```rust
use blenderui::prelude::*;
use bui_charts::*;
```

- [`<Oscilloscope>`](#oscilloscope)
- [`<EfficientFrontier>`](#efficientfrontier)
- [`<PhasePortrait>`](#phaseportrait)
- [`<PoincareMap>`](#poincaremap)
- [`<Bifurcation>`](#bifurcation)
- [`<HrDiagram>`](#hrdiagram)
- [`<Ramachandran>`](#ramachandran)
- [`<Harker>`](#harker)
- [`<RootLocus>`](#rootlocus)
- [`<RadiationPattern>`](#radiationpattern)
- [`<Quiver>`](#quiver)
- [`<Hodograph>`](#hodograph)
- [`<SmithChart>`](#smithchart)
- [`<EyeDiagram>`](#eyediagram)

Not sure which one fits? See the [guide](guide.md). `data` everywhere below takes `Vec<T>`, `Signal<Vec<T>>`, or your own [`Dataset`](../xy-chart/api.md#your-own-data-source).

---

## `<Oscilloscope>`

```rust
<Oscilloscope data={samples} x={|s: &Sample| s.time} trigger={0.5} span={0.02}>
    <TrendSeries name="CH1" y={|s: &Sample| s.ch1} />
    <TrendSeries name="CH2" y={|s: &Sample| s.ch2} />
</Oscilloscope>
```

| Prop | Type | Default |
| --- | --- | --- |
| `data` | | **required** |
| `x` | `Fn(&T) -> f64`: time, in seconds | **required** |
| children | [`<TrendSeries>`](../trend/api.md#trendseries): one channel each | **required** |
| `trigger` | `f64`: the level the first channel must rise through | none: free running |
| `span` | `f64`: how much time to show, from the trigger | all of it |
| `y_axis` | [`AxisConfig`](../xy-chart/api.md#axisconfig) | labelled "Volts" |
| `legend`, `tooltip`, `class` | as on `<Trend>` | |

With a `trigger`, the view starts at the first rising crossing of that level, found between samples, so a repeating wave stands still as data updates. The level is drawn as a dashed line.

## `<EfficientFrontier>`

```rust
<EfficientFrontier returns={vec![0.08, 0.12, 0.05]} covariance={covariance} risk_free={0.03} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `returns` | `Vec<f64>`: expected return per asset | **required** |
| `covariance` | `Vec<Vec<f64>>`: the assets' covariance matrix | **required** |
| `risk_free` | `f64`: the risk-free rate | `0.` |
| `portfolios` | `usize`: random long-only portfolios to scatter | `2000` |
| `scheme` | [`Scheme`](../xy-chart/api.md#color-and-scheme): colours the portfolios by Sharpe ratio | `BlueYellowRed` |
| `legend`, `tooltip`, `class` | | |

Draws the random portfolios, the frontier itself (solved exactly, short sales allowed), the capital market line from `risk_free`, and the maximum-Sharpe (tangency) portfolio. Returns and risk are shown as percentages.

## `<PhasePortrait>`

```rust
<PhasePortrait field={|x: f64, y: f64| (y, -x - 0.3 * y)} starts={vec![(2., 0.), (0., 2.)]} x_range={(-3., 3.)} y_range={(-3., 3.)} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `field` | `Fn(f64, f64) -> (f64, f64)`: (dx/dt, dy/dt) at (x, y) | **required** |
| `starts` | `Vec<(f64, f64)>`: one trajectory from each | **required** |
| `x_range`, `y_range` | `(f64, f64)` | **required** |
| `duration` | `f64`: time to follow each trajectory | `10.` |
| `step` | `f64`: the integration step | `0.01` |
| `arrows` | `usize`: direction arrows per side of the grid | `15` |
| `x_label`, `y_label` | text | `"x"`, `"y"` |
| `color` | colour of the direction arrows | the first series colour |
| `legend`, `tooltip`, `class` | | |

Trajectories are solved with fourth-order Runge–Kutta and stop once they are well outside the ranges.

## `<PoincareMap>`

```rust
<PoincareMap data={beats} value={|b: &Beat| b.interval} label="RR" />
```

| Prop | Type | Default |
| --- | --- | --- |
| `data` | | **required** |
| `value` | `Fn(&T) -> f64` | **required** |
| `lag` | `usize` | `1` |
| `identity` | `bool`: draw the line y = x | `true` |
| `label` | text: the axes read label<sub>n</sub> and label<sub>n+lag</sub> | `"x"` |
| `color` | colour | the first series colour |
| `point_size` | `f32` | `4.` |
| `tooltip`, `class` | | |

Plots each value against the one `lag` steps later. Steady signals cluster on the identity line; periodic ones form loops; chaos forms structured clouds.

## `<Bifurcation>`

```rust
<Bifurcation map={|r: f64, x: f64| r * x * (1. - x)} parameter={(2.5, 4.)} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `map` | `Fn(f64, f64) -> f64`: the next x from the parameter and the current x | **required** |
| `parameter` | `(f64, f64)`: the parameter range | **required** |
| `steps` | `usize`: parameter values across the range | `300` |
| `transient` | `usize`: iterations thrown away first | `300` |
| `keep` | `usize`: iterations plotted per parameter value | `60` |
| `start` | `f64`: the starting x | `0.5` |
| `x_label`, `y_label` | text | `"r"`, `"x"` |
| `color`, `point_size` | | first series colour, `1.5` |
| `class` | Tailwind classes (`"h-64"`) | |

Values that leave the finite range are skipped.

## `<HrDiagram>`

```rust
<HrDiagram data={stars} temperature={|s: &Star| s.kelvin} luminosity={|s: &Star| s.luminosity} radius={|s: &Star| s.radius} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `data` | | **required** |
| `temperature` | `Fn(&T) -> f64`: surface temperature in kelvin | **required** |
| `luminosity` | `Fn(&T) -> f64`: in solar luminosities | **required** |
| `radius` | `Fn(&T) -> f64`: sizes each star | one size |
| `point_size` | `f32` | `6.` |
| `tooltip`, `class` | | |

Both axes are logarithmic and temperature runs hot to cool, left to right, as astronomers draw it. Each star is coloured by its temperature, from red through white to blue.

## `<Ramachandran>`

```rust
<Ramachandran data={residues} phi={|r: &Residue| r.phi} psi={|r: &Residue| r.psi} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `data` | | **required** |
| `phi`, `psi` | `Fn(&T) -> f64`: backbone dihedral angles in degrees | **required** |
| `color`, `point_size` | | first series colour, `4.` |
| `tooltip`, `class` | | |

Both axes run from −180° to 180°.

## `<Harker>`

```rust
<Harker data={rocks} silica={|r: &Rock| r.silica}>
    <Oxide name="MgO" value={|r: &Rock| r.magnesia} />
    <Oxide name="CaO" value={|r: &Rock| r.lime} />
</Harker>
```

| Prop | Type | Default |
| --- | --- | --- |
| `data` | | **required** |
| `silica` | `Fn(&T) -> f64`: SiO₂ in weight % | **required** |
| children | `<Oxide name value>`: one small plot each | **required** |
| `columns` | `usize`: plots per row | `3` |
| `color`, `point_size` | | first series colour, `5.` |
| `class` | Tailwind classes (`"h-64"`) | |

Every plot shares the silica axis, so trends line up down a column.

## `<RootLocus>`

```rust
<RootLocus numerator={vec![1., 2.]} denominator={vec![1., 3., 2., 0.]} gain={(0., 50.)} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `numerator`, `denominator` | `Vec<f64>`: the open-loop transfer function's coefficients, highest power first | **required** |
| `gain` | `(f64, f64)`: the range of K | `(0., 100.)` |
| `steps` | `usize`: gains sampled across the range | `400` |
| `legend`, `tooltip`, `class` | | |

For each K the closed-loop poles are the roots of denominator + K × numerator, found with the Durand–Kerner method. Each branch is tracked from one K to the next so its line stays continuous. Open-loop poles are marked with squares and zeros with open circles.

## `<RadiationPattern>`

```rust
<RadiationPattern data={lobes} angle={|l: &Lobe| l.angle}>
    <TrendSeries name="E-plane" y={|l: &Lobe| l.e_plane} />
    <TrendSeries name="H-plane" y={|l: &Lobe| l.h_plane} />
</RadiationPattern>
```

| Prop | Type | Default |
| --- | --- | --- |
| `data` | | **required** |
| `angle` | `Fn(&T) -> f64`: degrees, clockwise from the top | **required** |
| children | [`<TrendSeries>`](../trend/api.md#trendseries): gain in dB, one pattern each | **required** |
| `max` | `f64`: dB at the rim | the highest gain, rounded up to 10 |
| `min` | `f64`: dB at the centre | `max − 40` |
| `levels` | `usize`: dB rings | `4` |
| `fill_opacity`, `line_width` | | `0.1`, `2.` |
| `legend`, `tooltip`, `class` | as on [`<Spider>`](../flow/api.md#spider) | |

Spokes every 30°, rings labelled in dB. A pattern that covers the full circle is closed and filled. Hover to read every pattern's gain at the nearest angle.

## `<Quiver>`

```rust
<Quiver data={field} x={|a: &Arrow| a.x} y={|a: &Arrow| a.y} u={|a: &Arrow| a.u} v={|a: &Arrow| a.v} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `data` | | **required** |
| `x`, `y` | `Fn(&T) -> f64`: where the arrow starts | **required** |
| `u`, `v` | `Fn(&T) -> f64`: the vector | **required** |
| `scale` | `f64`: arrow length per unit of vector | the longest arrow fills its share of the area |
| `equal_aspect` | `bool`: one unit is as long on x as on y | `true` |
| `color` | colour | the first series colour |
| `x_axis`, `y_axis`, `legend`, `tooltip`, `class` | as on `<XYChart>` | |

Hover an arrow's tail to read where it starts.

## `<Hodograph>`

```rust
<Hodograph data={sounding} u={|l: &Level| l.u} v={|l: &Level| l.v} height={|l: &Level| l.height} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `data` | one row per level, lowest first | **required** |
| `u`, `v` | `Fn(&T) -> f64`: the east and north wind components | **required** |
| `height` | `Fn(&T) -> f64`: colours the points | the row's position |
| `ring_step` | `f64`: speed between rings | a round number near a quarter of the top speed |
| `unit` | text | `"kt"` |
| `scheme` | [`Scheme`](../xy-chart/api.md#color-and-scheme) for the points | `BlueYellowRed` |
| `color` | colour of the line | the first series colour |
| `legend`, `tooltip`, `class` | | |

The wind vector at each level is joined from the ground up, over speed rings centred on calm. Both axes always share one scale, so rings stay round.

## `<SmithChart>`

```rust
<SmithChart data={sweep} resistance={|s: &Sweep| s.r} reactance={|s: &Sweep| s.x} impedance={50.} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `data` | | **required** |
| `resistance`, `reactance` | `Fn(&T) -> f64`, ohms | **required** |
| `impedance` | `f64`: the reference impedance Z₀, ohms | `50.` |
| `grid` | `Vec<f64>`: normalised resistance circles and reactance arcs | `[0.2, 0.5, 1, 2, 5]` |
| `overlay` | `SmithGrid::Impedance`, `Admittance` or `Both`: which grid to draw; the admittance grid is dashed | `Impedance` |
| `color`, `point_size` | | first series colour, `5.` |
| `legend`, `tooltip`, `class` | | |

Each row becomes the reflection coefficient Γ = (z − 1) / (z + 1), with z = (R + jX) / Z₀, and the points are joined in order. The centre is a perfect match and the rim is total reflection. Hover a point to read Γ.

## `<EyeDiagram>`

```rust
<EyeDiagram data={samples} x={|s: &Sample| s.time} value={|s: &Sample| s.volts} period={1.} offset={0.5} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `data` | samples in time order | **required** |
| `x` | `Fn(&T) -> f64`: time | **required** |
| `value` | `Fn(&T) -> f64` | **required** |
| `period` | `f64`: one unit interval, in the units of `x` | **required** |
| `offset` | `f64`: time that lands on the left edge | `0.` |
| `spans` | `usize`: unit intervals shown | `2` |
| `threshold` | `f64`: the level crossings are counted at | halfway between the lowest and highest value |
| `jitter` | `bool`: show the crossing histogram below | `true` |
| `bins` | `usize`: histogram bins across the window | `60` |
| `trace_opacity` | `f32` | `0.25` |
| `color` | colour of the traces | the first series colour |
| `x_axis`, `y_axis` | [`AxisConfig`](../xy-chart/api.md#axisconfig); x is always the window | |
| `class` | Tailwind classes (`"h-64"`) | |

The signal is cut every `spans × period` and the pieces are drawn on top of each other, joined across each cut so traces stay continuous. Faint traces build up into a density picture. The strip below counts where the signal crosses `threshold` inside the window: its width is the timing jitter. Built on `<XYChart>` and `<Trend>`, so Ctrl+scroll zooms both.
