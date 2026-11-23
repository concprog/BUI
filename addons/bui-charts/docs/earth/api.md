# Earth and lab science charts: API reference

`<Ternary>`, `<TernaryContour>`, `<Piper>`, `<Stuve>`, `<Tephigram>` and `<Choropleth>`. They are drawn with the same shape painter as the [diagram charts](../diagram/api.md): hover fades the other shapes and shows a tooltip, legends hide groups, and layouts are cached until the data, size or theme changes.

```rust
use blenderui::prelude::*;
use bui_charts::*;
```

- [`<Ternary>`](#ternary)
- [`<TernaryContour>`](#ternarycontour)
- [`<Piper>`](#piper)
- [`<Stuve>` and `<Tephigram>`](#stuve-and-tephigram)
- [`<Choropleth>`](#choropleth)

Not sure which one fits? See the [guide](guide.md). `data` takes `Vec<T>`, `Signal<Vec<T>>`, or your own [`Dataset`](../xy-chart/api.md#your-own-data-source). Every chart also takes `tooltip` and `class`.

---

## `<Ternary>`

```rust
<Ternary data={soils} a={|s: &Soil| s.sand} b={|s: &Soil| s.silt} c={|s: &Soil| s.clay} corners={vec!["Sand".into(), "Silt".into(), "Clay".into()]} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `a`, `b`, `c` | `Fn(&T) -> f64`: the three parts; each row is normalised to 100% | **required** |
| `name` | `Fn(&T) -> &str`: the tooltip title | "Point n" |
| `group` | `Fn(&T) -> &str`: colour and legend entry | one colour |
| `corners` | `Vec<SharedString>`: names of A (bottom left), B (bottom right) and C (top) | `A`, `B`, `C` |
| `divisions` | `usize`: grid lines per side | `10` |
| `point_size` | `f32`: radius in pixels | `5.` |
| `legend` | [`Legend`](../xy-chart/api.md#legend) | shown with a `group` |

## `<TernaryContour>`

```rust
<TernaryContour field={|a: f64, b: f64, c: f64| a * b + c} corners={names} levels={8} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `field` | `Fn(f64, f64, f64) -> f64`: the value at each composition; the parts add up to 1 | **required** |
| `corners` | as on `<Ternary>` | `A`, `B`, `C` |
| `resolution` | `usize`: mesh steps per side (2–200) | `30` |
| `levels` | `usize`: contour lines | `8` |
| `scheme` | [`HeatmapScheme`](../heatmap/api.md#enums-and-types) | `Viridis` |
| `mesh` | `bool`: draw the mesh triangle edges | `false` |

The triangle is split into `resolution²` small triangles, each coloured by the mean of its corners. Contours are traced through the mesh with marching triangles. A colour scale sits on the right. The field is evaluated once per change of these props.

## `<Piper>`

```rust
<Piper data={samples} calcium={|w: &Water| w.ca} magnesium={|w: &Water| w.mg} sodium={|w: &Water| w.na_k}
    bicarbonate={|w: &Water| w.hco3} sulfate={|w: &Water| w.so4} chloride={|w: &Water| w.cl} group={|w: &Water| w.aquifer} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `calcium`, `magnesium`, `sodium` | `Fn(&T) -> f64`: cations in meq/L; `sodium` is Na⁺ + K⁺ | **required** |
| `bicarbonate`, `sulfate`, `chloride` | `Fn(&T) -> f64`: anions in meq/L; `bicarbonate` is HCO₃⁻ + CO₃²⁻ | **required** |
| `name`, `group`, `point_size`, `legend` | as on `<Ternary>` | |

Each sample is drawn three times: in the cation triangle, in the anion triangle, and in the diamond, at its Na⁺+K⁺ share against its SO₄²⁻+Cl⁻ share. The tooltip lists all six ion shares.

## `<Stuve>` and `<Tephigram>`

```rust
<Tephigram data={levels} pressure={|l: &Level| l.hpa} temperature={|l: &Level| l.t} dewpoint={|l: &Level| l.td} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `pressure` | `Fn(&T) -> f64`: hPa | **required** |
| `temperature` | `Fn(&T) -> f64`: °C | **required** |
| `dewpoint` | `Fn(&T) -> f64`: °C | none |
| `pressure_range` | `(f64, f64)`: bottom and top, hPa | `(1050., 100.)` |
| `temperature_range` | `(f64, f64)`: °C across the bottom | Stüve `(-80., 40.)`, Tephigram `(-40., 40.)` |
| `adiabats`, `moist`, `mixing` | `bool`: dry adiabats, moist adiabats, mixing-ratio lines | all `true` |

Both draw isobars (labelled on the left), isotherms (labelled at the bottom), dry adiabats every 10 K, saturated adiabats integrated from 1000 hPa, and mixing-ratio lines in g/kg. The temperature line is red and the dew point green. Hover a level for its values, mixing ratio and potential temperature θ.

- **Stüve**: temperature across, pressure^0.286 up. Dry adiabats are straight lines.
- **Tephigram**: temperature against ln θ, turned 45° so isobars run nearly level. Equal areas mean equal energy.

Zoom: hold Ctrl (Cmd on macOS) and scroll, or pinch on a trackpad, to zoom about the pointer. Drag to pan once zoomed in, and double-click to reset.

## `<Choropleth>`

```rust
<Choropleth data={values} regions={shapes} name={|v: &Value| v.region.as_ref()} value={|v: &Value| v.amount} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `data` | one row per region with a value | **required** |
| `regions` | `Vec<Region>` or `Signal<Vec<Region>>`: the shapes | **required** |
| `name` | `Fn(&T) -> &str`: matched against `Region::name` | **required** |
| `value` | `Fn(&T) -> f64` | **required** |
| `projection` | `Projection::Equirectangular`, `Mercator` or `Plane` (x, y as given) | `Equirectangular` |
| `scheme` | `HeatmapScheme` | `Blues` |
| `range` | `(f64, f64)` | the data's minimum and maximum |
| `decimals` | `usize` | as needed |
| `scale_bar` | `bool` | `true` |

`Region::new(name, rings)` takes polygon rings of `(longitude, latitude)` (or `(x, y)` with `Plane`). Every ring is filled, so islands work; holes are not cut out. Regions without a value are grey. Keep `regions` in a signal: the map is recomputed only when it or the data changes.
