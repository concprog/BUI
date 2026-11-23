# Flame Graph: examples

Every example here also runs in the gallery:

```bash
cargo run --example gallery
```

Choose **Flame Graph** in the sidebar. Not sure which example you need? Start from the [guide](guide.md).

The examples share these imports and two folded-stack profiles, a "before" and an "after":

```rust
use blenderui::prelude::*;
use bui_charts::*;

const CPU: &str = "\
main;runtime.main;main.main;net/http.(*Server).Serve;net/http.(*conn).serve;net/http.serverHandler.ServeHTTP;main.handler;encoding/json.Marshal;encoding/json.(*encodeState).marshal;encoding/json.(*encodeState).reflectValue 180
main;runtime.main;main.main;net/http.(*Server).Serve;net/http.(*conn).serve;net/http.serverHandler.ServeHTTP;main.handler;database/sql.(*DB).Query;database/sql.(*DB).query;database/sql.(*driverConn).query 260
main;runtime.main;main.main;net/http.(*Server).Serve;net/http.(*conn).serve;net/http.serverHandler.ServeHTTP;main.handler;main.render;html/template.(*Template).Execute 140
main;runtime.main;main.main;net/http.(*Server).Serve;net/http.(*conn).serve;net/http.(*conn).readRequest;net/http.readRequest;net/textproto.(*Reader).ReadLine 90
main;runtime.main;main.main;net/http.(*Server).Serve;net/http.(*conn).serve;bufio.(*Writer).Flush;net.(*conn).Write;syscall.write 70
main;runtime.main;main.main;main.loadConfig;os.ReadFile;syscall.read 15
main;runtime.mcall;runtime.park_m;runtime.schedule;runtime.findRunnable;runtime.netpoll 110
main;runtime.gcBgMarkWorker;runtime.gcDrain;runtime.scanobject;runtime.findObject 95
main;runtime.gcBgMarkWorker;runtime.gcDrain;runtime.markroot;runtime.scanstack 40
main;runtime.mallocgc;runtime.(*mcache).refill;runtime.(*mcentral).cacheSpan 55
main;main.fib;main.fib;main.fib;main.fib 30
";

const CPU_AFTER: &str = "\
main;runtime.main;main.main;net/http.(*Server).Serve;net/http.(*conn).serve;net/http.serverHandler.ServeHTTP;main.handler;encoding/json.Marshal;encoding/json.(*encodeState).marshal;encoding/json.(*encodeState).reflectValue 90
main;runtime.main;main.main;net/http.(*Server).Serve;net/http.(*conn).serve;net/http.serverHandler.ServeHTTP;main.handler;database/sql.(*DB).Query;database/sql.(*DB).query;database/sql.(*driverConn).query 410
main;runtime.main;main.main;net/http.(*Server).Serve;net/http.(*conn).serve;net/http.serverHandler.ServeHTTP;main.handler;main.render;html/template.(*Template).Execute 150
main;runtime.main;main.main;net/http.(*Server).Serve;net/http.(*conn).serve;net/http.(*conn).readRequest;net/http.readRequest;net/textproto.(*Reader).ReadLine 85
main;runtime.main;main.main;net/http.(*Server).Serve;net/http.(*conn).serve;bufio.(*Writer).Flush;net.(*conn).Write;syscall.write 60
main;runtime.mcall;runtime.park_m;runtime.schedule;runtime.findRunnable;runtime.netpoll 105
main;runtime.gcBgMarkWorker;runtime.gcDrain;runtime.scanobject;runtime.findObject 160
main;runtime.mallocgc;runtime.(*mcache).refill;runtime.(*mcentral).cacheSpan 120
main;main.cache;sync.(*Map).Load 45
";

fn cpu() -> Profile {
    Profile::from_folded(CPU)
}
```

| # | Example | Shows |
| --- | --- | --- |
| 1 | [Flame graph and top table](#1-flame-graph-and-top-table) | the default split view |
| 2 | [Embedded](#2-embedded) | `header={false}`, a single pane |
| 3 | [Colour by value](#3-colour-by-value) | `FlameColors::ByValue` |
| 4 | [Right-aligned labels](#4-right-aligned-labels) | `LabelAlign::Right` |
| 5 | [Time](#5-time) | `SampleUnit::Nanoseconds` |
| 6 | [Custom value text](#6-custom-value-text) | `format` |
| 7 | [Memory](#7-memory) | `SampleUnit::Bytes` |
| 8 | [Diff](#8-diff) | `Profile::diff` |
| 9 | [Diff, colour-blind](#9-diff-colour-blind) | `FlameColors::DiffColorBlind` |
| 10 | [Call tree](#10-call-tree) | `Pane::CallTree` |
| 11 | [Top table and call tree](#11-top-table-and-call-tree) | any two panes |
| 12 | [Stacked panes](#12-stacked-panes) | `vertical` |
| 13 | [No collapsing](#13-no-collapsing) | `collapsing={false}` |
| 14 | [Custom actions](#14-custom-actions) | `actions` |
| 15 | [Nested set rows](#15-nested-set-rows) | `Profile::from_nested_set` |
| 16 | [Live profile](#16-live-profile) | updating a signal |

---

## 1. Flame graph and top table

Folded stacks with the default layout. Click a frame for its menu, hover for details, and search from the header.

```rust
#[component]
fn default_view() {
    let profile = use_signal(cpu());
    rsx! { <FlameGraph data={profile} class="h-[520px]" /> }
}
```

## 2. Embedded

Just the flame graph, with no header: good for dashboards.

```rust
#[component]
fn flame_only() {
    let profile = use_signal(cpu());
    rsx! { <FlameGraph data={profile} header={false} layout={PaneLayout::Single(Pane::FlameGraph)} class="h-[360px]" /> }
}
```

## 3. Colour by value

Larger frames are redder, relative to the current zoom, so focusing a frame recolours its subtree.

```rust
#[component]
fn by_value() {
    let profile = use_signal(cpu());
    rsx! {
        <FlameGraph data={profile} color_scheme={FlameColors::ByValue} layout={PaneLayout::Single(Pane::FlameGraph)} class="h-[420px]" />
    }
}
```

## 4. Right-aligned labels

Labels that don't fit keep the end of the name, which is usually the function itself.

```rust
#[component]
fn right_aligned() {
    let profile = use_signal(cpu());
    rsx! {
        <FlameGraph data={profile} text_align={LabelAlign::Right} layout={PaneLayout::Single(Pane::FlameGraph)} class="h-[420px]" />
    }
}
```

## 5. Time

Samples in nanoseconds, shown as ns / µs / ms in the top table.

```rust
#[component]
fn time_unit() {
    let profile = use_signal(cpu());
    rsx! { <FlameGraph data={profile} unit={SampleUnit::Nanoseconds} layout={PaneLayout::Single(Pane::TopTable)} class="h-[360px]" /> }
}
```

## 6. Custom value text

Your own text for every value.

```rust
#[component]
fn custom_format() {
    let profile = use_signal(cpu());
    rsx! {
        <FlameGraph data={profile} format={|v| format!("{v:.0} samples").into()} layout={PaneLayout::Single(Pane::FlameGraph)} class="h-[420px]" />
    }
}
```

## 7. Memory

Byte samples, shown as KiB / MiB.

```rust
#[component]
fn memory() {
    let profile = use_signal(cpu());
    rsx! { <FlameGraph data={profile} unit={SampleUnit::Bytes} class="h-[420px]" /> }
}
```

## 8. Diff

Two profiles compared. Frames that grew use your theme's bad colour, frames that shrank its good colour; the top table shows baseline, comparison and the change.

```rust
#[component]
fn diff_profile() {
    let profile = use_signal(Profile::diff(
        &Profile::from_folded(CPU),
        &Profile::from_folded(CPU_AFTER),
    ));
    rsx! { <FlameGraph data={profile} class="h-[520px]" /> }
}
```

## 9. Diff, colour-blind

The same comparison in blue and red.

```rust
#[component]
fn diff_color_blind() {
    let profile = use_signal(Profile::diff(
        &Profile::from_folded(CPU),
        &Profile::from_folded(CPU_AFTER),
    ));
    rsx! {
        <FlameGraph data={profile} color_scheme={FlameColors::DiffColorBlind} layout={PaneLayout::Single(Pane::FlameGraph)} class="h-[420px]" />
    }
}
```

## 10. Call tree

The call tree on its own: expand rows, sort columns, search, and switch to callers or callees.

```rust
#[component]
fn call_tree() {
    let profile = use_signal(cpu());
    rsx! { <FlameGraph data={profile} layout={PaneLayout::Single(Pane::CallTree)} class="h-[420px]" /> }
}
```

## 11. Top table and call tree

Any two panes can share the space.

```rust
#[component]
fn table_and_tree() {
    let profile = use_signal(cpu());
    rsx! { <FlameGraph data={profile} layout={PaneLayout::Split(Pane::TopTable, Pane::CallTree)} class="h-[420px]" /> }
}
```

## 12. Stacked panes

Panes stacked top and bottom, for tall spaces.

```rust
#[component]
fn stacked_panes() {
    let profile = use_signal(cpu());
    rsx! { <FlameGraph data={profile} vertical={true} class="h-[640px]" /> }
}
```

## 13. No collapsing

Every frame on its own row, with no grouped chains.

```rust
#[component]
fn no_collapsing() {
    let profile = use_signal(cpu());
    rsx! {
        <FlameGraph data={profile} collapsing={false} layout={PaneLayout::Single(Pane::FlameGraph)} class="h-[420px]" />
    }
}
```

## 14. Custom actions

Your own entry in the frame menu. Actions are plain functions in a static list.

```rust
fn log_frame(selected: &Selected<'_>, _: &mut Window, _: &mut App) {
    println!(
        "{} took {} ({} self)",
        selected.label, selected.total, selected.self_value
    );
}

const ACTIONS: &[FrameAction] = &[FrameAction {
    label: "Log to console",
    run: log_frame,
}];

#[component]
fn custom_actions() {
    let profile = use_signal(cpu());
    rsx! {
        <FlameGraph data={profile} actions={ACTIONS} layout={PaneLayout::Single(Pane::FlameGraph)} class="h-[420px]" />
    }
}
```

## 15. Nested set rows

Grafana's level / label / value / self rows, in depth-first order.

```rust
#[component]
fn nested_set() {
    let profile = use_signal(Profile::from_nested_set([
        (0, "total", 100., 0.),
        (1, "main", 100., 5.),
        (2, "parse", 60., 10.),
        (3, "lex", 50., 50.),
        (2, "render", 35., 35.),
    ]));
    rsx! { <FlameGraph data={profile} layout={PaneLayout::Single(Pane::FlameGraph)} class="h-[300px]" /> }
}
```

## 16. Live profile

Replace the profile in its signal and the graph redraws. `keep_focus_on_data_change` keeps focus and sandwich across updates.

```rust
#[component]
fn live_profile() {
    let profile = use_signal(cpu());
    let round = use_signal(1u32);
    rsx! {
        <div class="flex flex-col gap-2 h-[460px]">
            <div class="cursor-pointer text-sm underline" onClick={move || {
                let next = if round.get().is_multiple_of(2) { CPU } else { CPU_AFTER };
                round.set(round.get() + 1);
                profile.update(|profile| *profile = Profile::from_folded(next));
            }}>
                "Swap profile"
            </div>
            <FlameGraph data={profile} layout={PaneLayout::Single(Pane::FlameGraph)} keep_focus_on_data_change={true} class="flex-1" />
        </div>
    }
}
```

