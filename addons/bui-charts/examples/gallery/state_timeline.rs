use std::sync::LazyLock;

use blenderui::prelude::*;
use bui_charts::*;


const T0: f64 = 1_729_000_000_000.;
const MINUTE: f64 = 60_000.;

fn noise(seed: u64) -> f64 {
    let mixed = seed
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    (mixed >> 11) as f64 / (1u64 << 53) as f64
}

#[derive(Clone, PartialEq)]
struct Check {
    time: f64,
    api: &'static str,
    web: &'static str,
    db: &'static str,
    queue: &'static str,
}

fn status(seed: u64) -> &'static str {
    match noise(seed) {
        roll if roll < 0.75 => "OK",
        roll if roll < 0.92 => "WARN",
        _ => "DOWN",
    }
}

fn checks() -> Vec<Check> {
    (0..48u64)
        .map(|step| {
            let slow = step / 4;
            Check {
                time: T0 + step as f64 * 5. * MINUTE,
                api: status(slow * 11),
                web: status(slow * 13 + 1),
                db: status(slow * 17 + 2),
                queue: status(slow * 19 + 3),
            }
        })
        .collect()
}

#[derive(Clone, PartialEq)]
struct Reading {
    time: f64,
    cpu: f64,
    memory: f64,
    disk: f64,
    online: bool,
}

fn readings() -> Vec<Reading> {
    (0..60u64)
        .map(|step| {
            let t = step as f64;
            Reading {
                time: T0 + t * 2. * MINUTE,
                cpu: ((40. + 35. * (t / 8.).sin()) / 10.).round() * 10.,
                memory: 50. + (t / 3.).floor() * 2.,
                disk: if (20..26).contains(&step) {
                    f64::NAN
                } else {
                    30. + (t / 10.).floor() * 15.
                },
                online: !(30..38).contains(&step),
            }
        })
        .collect()
}

fn fleet() -> Frame<4> {
    let rows: Vec<[f64; 4]> = (0..40u64)
        .map(|step| {
            let level = |seed: u64| (noise(seed + step / 5) * 3.).floor();
            [T0 + step as f64 * MINUTE, level(1), level(50), level(100)]
        })
        .collect();
    Frame::new(["time", "eu-west", "us-east", "ap-south"]).with_rows(rows)
}

static STATES: LazyLock<ValueMappings> = LazyLock::new(|| {
    ValueMappings::new()
        .value("OK", "OK", rgb(0x40a02b))
        .value("WARN", "Degraded", rgb(0xdf8e1d))
        .value("DOWN", "Down", rgb(0xd20f39))
});

static ONLINE: LazyLock<ValueMappings> = LazyLock::new(|| {
    ValueMappings::new()
        .special(Special::True, "Online", rgb(0x1e66f5))
        .special(Special::False, "Offline", rgb(0x9ca0b0))
});

static LIMITS: LazyLock<[(f64, Hsla); 3]> = LazyLock::new(|| {
    [
        (f64::NEG_INFINITY, rgb(0x40a02b)),
        (50., rgb(0xdf8e1d)),
        (70., rgb(0xd20f39)),
    ]
});

fn time() -> AxisConfig {
    AxisConfig::new().time()
}

#[component]
fn service_status() {
    let data = use_signal(checks());
    rsx! {
        <StateTimeline data={data} x={|c: &Check| c.time} mappings={&*STATES} x_axis={time()} class="h-64">
            <StateRow name="api" value={|c: &Check| c.api} />
            <StateRow name="web" value={|c: &Check| c.web} />
            <StateRow name="db" value={|c: &Check| c.db} />
            <StateRow name="queue" value={|c: &Check| c.queue} />
        </StateTimeline>
    }
}

#[component]
fn threshold_states() {
    let data = use_signal(readings());
    rsx! {
        <StateTimeline data={data} x={|r: &Reading| r.time} color={GaugeColor::Thresholds} thresholds={Thresholds::absolute(&LIMITS[..])} x_axis={time()} class="h-56">
            <StateRow name="CPU %" value={|r: &Reading| r.cpu} />
            <StateRow name="Memory %" value={|r: &Reading| r.memory} />
        </StateTimeline>
    }
}

#[component]
fn scheme() {
    let data = use_signal(readings());
    rsx! {
        <StateTimeline data={data} x={|r: &Reading| r.time} min={0.} max={100.} x_axis={time()} class="h-56">
            <StateRow name="CPU %" value={|r: &Reading| r.cpu} />
            <StateRow name="Memory %" value={|r: &Reading| r.memory} />
        </StateTimeline>
    }
}

#[component]
fn booleans() {
    let data = use_signal(readings());
    rsx! {
        <StateTimeline data={data} x={|r: &Reading| r.time} mappings={&*ONLINE} align_value={ValueAlign::Center} x_axis={time()} class="h-40">
            <StateRow name="node-7" value={|r: &Reading| r.online} />
        </StateTimeline>
    }
}

#[component]
fn thin_rows() {
    let data = use_signal(checks());
    rsx! {
        <StateTimeline
            data={data}
            x={|c: &Check| c.time}
            mappings={&*STATES}
            show_value={VisibilityMode::Never}
            row_height={0.5}
            line_width={1.}
            fill_opacity={0.35}
            x_axis={time()}
            class="h-56"
        >
            <StateRow name="api" value={|c: &Check| c.api} />
            <StateRow name="web" value={|c: &Check| c.web} />
            <StateRow name="db" value={|c: &Check| c.db} />
        </StateTimeline>
    }
}

#[component]
fn aligned() {
    let data = use_signal(checks());
    rsx! {
        <StateTimeline data={data} x={|c: &Check| c.time} mappings={&*STATES} align_value={ValueAlign::Right} x_axis={time()} class="h-40">
            <StateRow name="api" value={|c: &Check| c.api} />
            <StateRow name="db" value={|c: &Check| c.db} />
        </StateTimeline>
    }
}

#[component]
fn nulls() {
    let data = use_signal(readings());
    rsx! {
        <StateTimeline data={data} x={|r: &Reading| r.time} color={GaugeColor::Thresholds} thresholds={Thresholds::absolute(&LIMITS[..])} x_axis={time()} class="h-48">
            <StateRow name="gaps" value={|r: &Reading| r.disk} />
            <StateRow name="spanned" value={|r: &Reading| r.disk} span_nulls={NullSpan::Always} />
        </StateTimeline>
    }
}

#[component]
fn unmerged() {
    let data = use_signal(readings());
    rsx! {
        <StateTimeline data={data} x={|r: &Reading| r.time} merge_values={false} min={0.} max={100.} show_value={VisibilityMode::Never} x_axis={time()} class="h-40">
            <StateRow name="CPU %" value={|r: &Reading| r.cpu} />
        </StateTimeline>
    }
}

#[component]
fn pages() {
    let data = use_signal(checks());
    rsx! {
        <StateTimeline data={data} x={|c: &Check| c.time} mappings={&*STATES} per_page={2} x_axis={time()} class="h-48">
            <StateRow name="api" value={|c: &Check| c.api} />
            <StateRow name="web" value={|c: &Check| c.web} />
            <StateRow name="db" value={|c: &Check| c.db} />
            <StateRow name="queue" value={|c: &Check| c.queue} />
        </StateTimeline>
    }
}

#[component]
fn from_frame() {
    let data = use_signal(fleet());
    rsx! { <StateTimeline data={data} color={GaugeColor::Palette} x_axis={time()} class="h-48" /> }
}

#[component]
fn multi_tooltip() {
    let data = use_signal(checks());
    rsx! {
        <StateTimeline
            data={data}
            x={|c: &Check| c.time}
            mappings={&*STATES}
            tooltip={Tooltip::multi().sort(SortOrder::Ascending)}
            legend={Legend::list().right()}
            x_axis={time()}
            class="h-56"
        >
            <StateRow name="api" value={|c: &Check| c.api} />
            <StateRow name="web" value={|c: &Check| c.web} />
            <StateRow name="db" value={|c: &Check| c.db} />
        </StateTimeline>
    }
}

#[component]
fn zoom() {
    let data = use_signal(checks());
    let picked = use_signal(SharedString::Borrowed(
        "Drag across the lanes to zoom. Double-click to zoom out.",
    ));
    rsx! {
        <div class="flex flex-col gap-2 h-56">
            <div class="text-xs">{picked.get()}</div>
            <StateTimeline
                data={data}
                x={|c: &Check| c.time}
                mappings={&*STATES}
                on_select={move |range: SelectedRange, _: &mut Window, _: &mut App| {
                    let minutes = (range.x.1 - range.x.0) / MINUTE;
                    picked.set(SharedString::from(format!("Zoomed to {minutes:.0} minutes")));
                }}
                x_axis={time()}
                class="flex-1"
            >
                <StateRow name="api" value={|c: &Check| c.api} />
                <StateRow name="db" value={|c: &Check| c.db} />
            </StateTimeline>
        </div>
    }
}

#[component]
fn fixed() {
    let data = use_signal(readings());
    rsx! {
        <StateTimeline
            data={data}
            x={|r: &Reading| r.time}
            color={GaugeColor::Fixed(rgb(0x8839ef))}
            fill_opacity={1.}
            y_axis={AxisConfig::new().placement(AxisPlacement::Hidden)}
            legend={Legend::hidden()}
            x_axis={time()}
            class="h-32"
        >
            <StateRow name="memory" value={|r: &Reading| r.memory} />
        </StateTimeline>
    }
}

#[component]
fn live() {
    let data = use_signal(checks());
    let round = use_signal(0u64);
    rsx! {
        <div class="flex flex-col gap-2 h-56">
            <div
                id="reroll"
                class="cursor-pointer text-sm underline"
                onClick={move || {
                    let next = round.get() + 1;
                    round.set(next);
                    data.update(|rows| {
                        for (index, row) in rows.iter_mut().enumerate() {
                            row.api = status(next * 101 + index as u64 / 4);
                        }
                    });
                }}
            >
                "Re-roll the api states"
            </div>
            <StateTimeline data={data} x={|c: &Check| c.time} mappings={&*STATES} x_axis={time()} class="flex-1">
                <StateRow name="api" value={|c: &Check| c.api} />
                <StateRow name="web" value={|c: &Check| c.web} />
            </StateTimeline>
        </div>
    }
}

examples! {
    service_status: "Service status" => "Text states coloured and renamed by value mappings.",
    threshold_states: "Threshold states" => "Numbers grouped into threshold steps; equal steps merge into one block.",
    scheme: "Colour by value" => "The default green–yellow–red scheme over 0–100.",
    booleans: "Booleans" => "true / false mapped to Online / Offline, centred labels.",
    thin_rows: "Thin rows, no labels" => "Row height 0.5, an outline and a lighter fill.",
    aligned: "Right-aligned values" => "Labels sit at the end of each state.",
    nulls: "Missing values" => "Gaps at missing values, or bridged across them.",
    unmerged: "Every sample" => "merge_values off: one block per sample.",
    pages: "Pages" => "Two lanes per page with page buttons.",
    from_frame: "From a frame" => "Every field is a lane; numbers coloured from the palette.",
    multi_tooltip: "Multi tooltip" => "Every lane at the hovered time, legend on the right.",
    zoom: "Drag to zoom" => "Drag across the time axis; on_select reports the range.",
    fixed: "Fixed colour" => "One colour, solid fill, lane names hidden.",
    live: "Live data" => "A signal as data: the lanes redraw when it changes.",
}
