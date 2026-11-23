# Flame Graph: API reference

`<FlameGraph>` shows a CPU, memory or time profile three ways: as a flame graph, as a top table of the heaviest functions, and as an expandable call tree. It can compare two profiles too. It's a port of Grafana's Flame Graph; see [Coming from Grafana](#coming-from-grafana).

```rust
use blenderui::prelude::*;
use bui_charts::*;
```

The buttons' icons are line icons bui-charts draws itself: nothing to register.

- [`<FlameGraph>`](#flamegraph)
- [`Profile`](#profile)
- [Panes and layout](#panes-and-layout)
- [Interacting with the graph](#interacting-with-the-graph)
- [Colours](#colours)
- [Values and units](#values-and-units)
- [Custom menu actions](#custom-menu-actions)
- [Enums and types](#enums-and-types)
- [Coming from Grafana](#coming-from-grafana)

Not sure which option fits? See the [guide](guide.md).

---

## `<FlameGraph>`

| Prop | Type | Default |
| --- | --- | --- |
| `data` | `Profile` or `Signal<Profile>` | **required** |
| `layout` | [`PaneLayout`](#panes-and-layout) | top table and flame graph side by side |
| `header` | `bool` | `true` |
| `unit` | [`SampleUnit`](#values-and-units) | `Count` |
| `format` | `Fn(f64) -> SharedString` | none |
| `decimals` | `usize` | chosen by the unit |
| `color_scheme` | [`FlameColors`](#colours) | `ByPackage`, or `Diff` for a diff profile |
| `text_align` | [`LabelAlign`](#enums-and-types) | `Left` |
| `collapsing` | `bool` | `true` |
| `vertical` | `bool` | `false` |
| `keep_focus_on_data_change` | `bool` | `false` |
| `actions` | `&'static [FrameAction]` | none |
| `class` | Tailwind classes | fills its parent |
| `key` | anything `Hash` (taken by `rsx!`: hook state follows it) | – |

`layout`, `color_scheme` and `text_align` set the starting state. After that, the header and toolbar controls change them.

## `Profile`

A profile is a call tree: each frame (a function in a call stack) has a total value, which includes everything it called, and a self value, which is only its own work.

| Constructor | Use it for |
| --- | --- |
| `Profile::from_folded(text)` | **Folded stacks**: one line per stack, frames separated by `;`, then a count, e.g. `main;parse;lex 42`. This is what `perf` + `stackcollapse`, `cargo flamegraph` / `inferno`, `py-spy --format raw` and `async-profiler -o collapsed` produce. A `total` root frame is added on top. |
| `Profile::from_nested_set(rows)` | `(level, label, value, self)` rows in depth-first order, Grafana's profile format. Level 0 is the root. |
| `Profile::diff(&baseline, &comparison)` | Two profiles merged into a diff: matching stacks line up, and each frame keeps both values. |
| `Profile::diff_from_nested_set(rows)` | `(level, label, value, self, value_right, self_right)` rows that are already a diff. |

| Method | Returns |
| --- | --- |
| `is_diff()` | whether the profile compares two profiles |
| `len()` / `is_empty()` | the number of frames |

Labels are stored once per distinct name, however many frames use them.

## Panes and layout

| Pane | Shows |
| --- | --- |
| `Pane::FlameGraph` | the flame graph |
| `Pane::TopTable` | one row per function: Self and Total (or Baseline, Comparison and Diff), sortable by any column |
| `Pane::CallTree` | the call tree as an expandable table, with its own search |

| `PaneLayout` | Meaning |
| --- | --- |
| `PaneLayout::Single(pane)` | one pane |
| `PaneLayout::Split(left, right)` | two panes side by side, half and half (zopra's divider could be dragged; blenderui has no resizable split); stacked top and bottom when `vertical` is set |

A split layout shows a single pane when the chart is narrower than 800px (unless `vertical` is set). With `header={false}` the search box and pane pickers are hidden, so whatever `layout` you pass is fixed.

## Interacting with the graph

| Action | Result |
| --- | --- |
| hover a frame | tooltip with the total, self and sample count, as percentages too; Baseline / Comparison / Diff for a diff profile |
| click a frame | menu: **Focus block**, **Copy function name**, **Sandwich view**, your [actions](#custom-menu-actions), and group expand / collapse |
| Focus block | zooms to that frame; frames above it are drawn lighter. A pill shows its share and clears the focus. |
| Sandwich view | shows that function's callers above and its callees below, merged across every place it was called |
| search box | dims every frame that doesn't match and filters the top table. Separate several terms with commas. Each term is tried as a regular expression first, then as a fuzzy match. |
| top table search / sandwich buttons | search for exactly that function / open its sandwich view |
| call tree up / down arrow buttons | show that function's callers / callees as the tree |
| call tree search | finds functions (case-insensitive; regular expressions allowed), shows "1 of N", and the arrows step through matches, expanding and scrolling to each |
| Reset | clears search, focus and sandwich |

**Collapsing.** When a frame is the only child of its parent and holds more than 99% of the parent's value, the two are grouped onto one row and labelled `(N) name`. A strip on the left marks groups. Expand or collapse them from the frame menu, or all at once with the toolbar buttons. Set `collapsing={false}` to give every frame its own row.

**Changing data.** When `data` changes, focus, sandwich and grouping reset, unless you set `keep_focus_on_data_change`.

## Colours

| `FlameColors` | Colour of each frame |
| --- | --- |
| `ByPackage` | by package or module name, so related functions share a colour |
| `ByValue` | yellow for small frames to red for large ones, relative to the current zoom |
| `Diff` | for diff profiles: your theme's good colour where a frame shrank, its neutral colour where it stayed the same, and its bad colour where it grew |
| `DiffColorBlind` | the same, in blue, grey and red |

A diff profile shows a legend from −100% (removed) to +100% (added). Frames too narrow for a label are drawn grey. Set the good, neutral and bad colours in your [theme](../theming.md).

## Values and units

| `SampleUnit` | Values shown as | Tooltip title |
| --- | --- | --- |
| `Count` | 1.2 K, 3.4 Mil | Count |
| `Nanoseconds` | ns, µs, ms, s, min | Time |
| `Bytes` | B, KiB, MiB, GiB | RAM |

`format` replaces the unit's formatting everywhere, and `decimals` fixes the number of decimals. Sample counts in tooltips are always shown in full, with thousands separators.

## Custom menu actions

```rust
fn open_source(frame: &Selected<'_>, window: &mut Window, cx: &mut App) {
    println!("open {}", frame.label);
}

const ACTIONS: &[FrameAction] = &[FrameAction { label: "Open source", run: open_source }];

<FlameGraph data={profile} actions={ACTIONS} />
```

`Selected` gives the clicked frame's `label`, `total`, `self_value` and `level`.

## Enums and types

| Type | Values |
| --- | --- |
| `PaneLayout` | `Single(Pane)`, `Split(Pane, Pane)` |
| `Pane` | `TopTable`, `FlameGraph`, `CallTree` |
| `FlameColors` | `ByPackage`, `ByValue`, `Diff`, `DiffColorBlind` |
| `LabelAlign` | `Left` (default), `Right`: with `Right`, labels that don't fit keep the end of the function name |
| `SampleUnit` | `Count` (default), `Nanoseconds`, `Bytes` |
| `FrameAction` | `{ label: &'static str, run: fn(&Selected, &mut Window, &mut App) }` |
| `Selected` | `{ label, total, self_value, level }` |

## Coming from Grafana

| Grafana | Here |
| --- | --- |
| data frame with `level`, `label`, `value`, `self` | `Profile::from_nested_set` |
| `valueRight`, `selfRight` (diff) | `Profile::diff_from_nested_set`, or `Profile::diff` |
| Show flame graph only | `header={false}` with `layout={PaneLayout::Single(Pane::FlameGraph)}` |
| view mode, pane selectors | `layout` |
| color scheme | `color_scheme` |
| text align | `text_align` |
| disable collapsing | `collapsing={false}` |
| vertical | `vertical` |
| keep focus on data change | `keep_focus_on_data_change` |
| extra context menu buttons | `actions` |
| Unit, Decimals | `unit`, `format`, `decimals` |

Not available here: the "Analyze with Assistant" button and Grafana's usage analytics.
