# Matrix charts: API reference

Grids of cells: `<Spectrogram>` and `<AttentionMap>` draw through [`<Heatmap>`](../heatmap/api.md); `<Waffle>` draws through [`<StateTimeline>`](../state-timeline/api.md).

```rust
use blenderui::prelude::*;
use bui_charts::*;
```

- [`<Spectrogram>`](#spectrogram)
- [`<AttentionMap>`](#attentionmap)
- [`<Waffle>`](#waffle)

Not sure which one fits? See the [guide](guide.md).

---

## `<Spectrogram>`

```rust
<Spectrogram data={samples} x={|s: &Sample| s.time} value={|s: &Sample| s.value} window_size={512} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `data` | `Vec<T>`, `Signal<Vec<T>>`, or your own [`Dataset`](../xy-chart/api.md#your-own-data-source) | **required** |
| `x` | `Fn(&T) -> f64`: time, in seconds, evenly spaced | **required** |
| `value` | `Fn(&T) -> f64`: the signal | **required** |
| `window_size` | `usize`: samples per FFT, rounded up to a power of two | `256` |
| `overlap` | `f64`: the share of each window that the next one repeats, 0 to 0.95 | `0.5` |
| `scheme` | [`HeatmapScheme`](../heatmap/api.md#enums-and-types) | `Magma` |
| `class` | Tailwind classes (`"h-64"`) | |

The signal is cut into windows, each window gets a Hann taper and a fast Fourier transform, and each cell is the power at that time and frequency in dB. The sample rate is taken from the median gap between `x` values; the y axis runs from 0 Hz to half the sample rate.

## `<AttentionMap>`

```rust
<AttentionMap tokens={tokens} weights={weights} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `tokens` | `Vec<SharedString>`: the keys, along the bottom | **required** |
| `queries` | `Vec<SharedString>`: the rows, down the side | the same as `tokens` |
| `weights` | `Vec<f64>`, row by row: `queries.len() × tokens.len()` values | **required** |
| `scheme` | `HeatmapScheme` | `Blues` |
| `show_values` | `bool`: write each weight in its cell | `true` up to 12 columns |
| `class` | Tailwind classes (`"h-64"`) | |

The first query is at the top, as in a matrix. Missing weights are left empty.

## `<Waffle>`

```rust
<Waffle data={mix} name={|s: &Share| s.name} value={|s: &Share| s.value} />
```

| Prop | Type | Default |
| --- | --- | --- |
| `data` | as above | **required** |
| `name` | `Fn(&T) -> &str` | **required** |
| `value` | `Fn(&T) -> f64` | **required** |
| `rows`, `columns` | `usize` | `10`, `10` |
| `gap` | `f32`: the space between squares, as a share of a square | `0.12` |
| `legend` | [`Legend`](../xy-chart/api.md#legend) | shown |
| `tooltip`, `class` | | |

Each item gets a share of the `rows × columns` squares, filled in order, row by row. Shares are rounded by the largest-remainder method, so the squares always add up to the full grid.
