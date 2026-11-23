# Flame Graph: choosing options

How to pick between the options in the [API reference](api.md). Every link goes to a runnable example; the gallery has them all (`cargo run --example gallery`, then **Flame Graph**).

## Getting a profile in

| You have | Use |
| --- | --- |
| output from `perf`, `cargo flamegraph`, `inferno`, `py-spy`, `async-profiler`, or any `a;b;c 12` lines | `Profile::from_folded` |
| Grafana / Pyroscope data with level, label, value and self | `Profile::from_nested_set` |
| a before and an after profile | `Profile::diff(&before, &after)` |

Build the profile once and keep it in a signal: `let profile = use_signal(Profile::from_folded(text));`. A `Profile` passed directly is processed again on every render of your component.

Examples: [Flame graph and top table](examples.md#1-flame-graph-and-top-table), [Nested set rows](examples.md#15-nested-set-rows), [Diff](examples.md#8-diff).

## Which panes

| Question | Pane |
| --- | --- |
| "Where is the time going overall?" | Flame graph |
| "Which functions are the most expensive?" | Top table, sorted by Self |
| "What is the path to this expensive call?" | Call tree, or Sandwich view in the flame graph |
| "Who calls this function?" | Sandwich view, or the call tree's up-arrow (callers) button |

- The default, top table beside the flame graph, suits a full-width panel.
- Use `PaneLayout::Single(..)` with `header={false}` to embed one view in a dashboard.
- Use `vertical={true}` for tall, narrow spaces.

Examples: [Embedded](examples.md#2-embedded), [Call tree](examples.md#10-call-tree), [Top table and call tree](examples.md#11-top-table-and-call-tree), [Stacked panes](examples.md#12-stacked-panes).

## Colours

| You want | Use |
| --- | --- |
| related code to look related | `FlameColors::ByPackage` (the default) |
| hot spots to stand out | `FlameColors::ByValue` |
| a before/after comparison | a diff profile: `Diff`, or `DiffColorBlind` for colour-blind users |

Examples: [Colour by value](examples.md#3-colour-by-value), [Diff, colour-blind](examples.md#9-diff-colour-blind).

## Labels and grouping

- Long names, such as Java or Go functions with package paths: `text_align={LabelAlign::Right}` keeps the end of the name, which is usually the function itself.
- Deep stacks made of single calls are grouped onto one row by default. Turn this off with `collapsing={false}` when every frame matters, for example when teaching or debugging the profiler itself.

Examples: [Right-aligned labels](examples.md#4-right-aligned-labels), [No collapsing](examples.md#13-no-collapsing).

## Units

Set `unit` to match what the profile counts: `Nanoseconds` for wall or CPU time, `Bytes` for allocations, `Count` for samples. Use `format` when you need something else, such as milliseconds, percentages or a currency.

Examples: [Time](examples.md#5-time), [Custom value text](examples.md#6-custom-value-text), [Memory](examples.md#7-memory).

## Live profiles

Keep the profile in a signal and replace it when new data arrives. By default, focus and sandwich reset on new data. Set `keep_focus_on_data_change={true}` when the new data is the same program, refreshed, so the view stays where the user left it.

Example: [Live profile](examples.md#16-live-profile).

## Hooking into your app

Add entries to the frame menu with `actions`, for example to open the source file, link to a trace, or file a ticket. Each action is a plain function in a `static` list, so adding actions costs nothing at runtime.

Example: [Custom actions](examples.md#14-custom-actions).
