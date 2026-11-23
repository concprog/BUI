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

pub fn cpu() -> Profile {
    Profile::from_folded(CPU)
}

#[component]
fn default_view() {
    let profile = use_signal(cpu());
    rsx! { <FlameGraph data={profile} class="h-[520px]" /> }
}

#[component]
fn flame_only() {
    let profile = use_signal(cpu());
    rsx! { <FlameGraph data={profile} header={false} layout={PaneLayout::Single(Pane::FlameGraph)} class="h-[360px]" /> }
}

#[component]
fn by_value() {
    let profile = use_signal(cpu());
    rsx! {
        <FlameGraph data={profile} color_scheme={FlameColors::ByValue} layout={PaneLayout::Single(Pane::FlameGraph)} class="h-[420px]" />
    }
}

#[component]
fn right_aligned() {
    let profile = use_signal(cpu());
    rsx! {
        <FlameGraph data={profile} text_align={LabelAlign::Right} layout={PaneLayout::Single(Pane::FlameGraph)} class="h-[420px]" />
    }
}

#[component]
fn time_unit() {
    let profile = use_signal(cpu());
    rsx! { <FlameGraph data={profile} unit={SampleUnit::Nanoseconds} layout={PaneLayout::Single(Pane::TopTable)} class="h-[360px]" /> }
}

#[component]
fn custom_format() {
    let profile = use_signal(cpu());
    rsx! {
        <FlameGraph data={profile} format={|v| format!("{v:.0} samples").into()} layout={PaneLayout::Single(Pane::FlameGraph)} class="h-[420px]" />
    }
}

#[component]
fn memory() {
    let profile = use_signal(cpu());
    rsx! { <FlameGraph data={profile} unit={SampleUnit::Bytes} class="h-[420px]" /> }
}

#[component]
fn diff_profile() {
    let profile = use_signal(Profile::diff(
        &Profile::from_folded(CPU),
        &Profile::from_folded(CPU_AFTER),
    ));
    rsx! { <FlameGraph data={profile} class="h-[520px]" /> }
}

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

#[component]
fn call_tree() {
    let profile = use_signal(cpu());
    rsx! { <FlameGraph data={profile} layout={PaneLayout::Single(Pane::CallTree)} class="h-[420px]" /> }
}

#[component]
fn table_and_tree() {
    let profile = use_signal(cpu());
    rsx! { <FlameGraph data={profile} layout={PaneLayout::Split(Pane::TopTable, Pane::CallTree)} class="h-[420px]" /> }
}

#[component]
fn stacked_panes() {
    let profile = use_signal(cpu());
    rsx! { <FlameGraph data={profile} vertical={true} class="h-[640px]" /> }
}

#[component]
fn no_collapsing() {
    let profile = use_signal(cpu());
    rsx! {
        <FlameGraph data={profile} collapsing={false} layout={PaneLayout::Single(Pane::FlameGraph)} class="h-[420px]" />
    }
}

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

examples! {
    default_view: "Flame graph and top table" => "Folded stacks with the default split view. Click a frame for its menu, search above.",
    flame_only: "Embedded" => "Just the flame graph: no header, one pane.",
    by_value: "Colour by value" => "Hotter frames are redder, relative to the current view.",
    right_aligned: "Right-aligned labels" => "Function names stay readable on narrow frames.",
    time_unit: "Time" => "Nanosecond samples, shown in the top table as ns / µs / ms.",
    custom_format: "Custom value text" => "Your own text for every value.",
    memory: "Memory" => "Byte samples, shown as KiB / MiB.",
    diff_profile: "Diff" => "Two profiles compared: red grew, green shrank.",
    diff_color_blind: "Diff, colour-blind" => "The same diff in blue and red.",
    call_tree: "Call tree" => "Expandable tree with self and total, plus its own search.",
    table_and_tree: "Top table and call tree" => "Any two panes side by side.",
    stacked_panes: "Stacked panes" => "Panes stacked vertically for tall layouts.",
    no_collapsing: "No collapsing" => "Every frame on its own row, no grouped chains.",
    custom_actions: "Custom actions" => "Your own entry in the frame menu.",
    nested_set: "Nested set rows" => "Grafana's level / label / value / self format.",
    live_profile: "Live profile" => "A signal as data: the graph redraws when it changes.",}
