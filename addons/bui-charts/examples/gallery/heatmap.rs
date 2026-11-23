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
struct Request {
    time: f64,
    latency: f64,
}

fn requests() -> Vec<Request> {
    (0..4_000u64)
        .map(|index| {
            let minute = (index % 120) as f64;
            let wave = 40. + 25. * (minute / 12.).sin();
            let spike = if (50..60).contains(&(index % 120)) {
                120.
            } else {
                0.
            };
            let jitter = noise(index) * 60. + noise(index * 7 + 1).powi(4) * 400.;
            Request {
                time: T0 + minute * MINUTE + noise(index * 3) * MINUTE,
                latency: wave + spike + jitter,
            }
        })
        .collect()
}

fn histogram() -> Frame<7> {
    let rows: Vec<[f64; 7]> = (0..40u64)
        .map(|step| {
            let t = step as f64;
            let load = 30. + 20. * (t / 6.).sin();
            [
                T0 + t * MINUTE,
                load * 0.2,
                load * 0.9,
                load,
                load * 0.6,
                load * 0.25,
                2. + noise(step) * 3.,
            ]
        })
        .collect();
    Frame::new(["time", "0.005", "0.01", "0.025", "0.05", "0.1", "+Inf"]).with_rows(rows)
}

#[derive(Clone, PartialEq)]
struct Slot {
    day: f64,
    hour: f64,
    deploys: f64,
}

const DAYS: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

fn deploys() -> Vec<Slot> {
    (0..7u64)
        .flat_map(|day| {
            (8..20u64).map(move |hour| {
                let busy = if day < 5 { 6. } else { 1. };
                Slot {
                    day: day as f64,
                    hour: hour as f64,
                    deploys: (busy * noise(day * 31 + hour)).round(),
                }
            })
        })
        .collect()
}

fn calc() -> Calculation {
    Calculation {
        x_buckets: Buckets::Size(4. * MINUTE),
        y_buckets: Buckets::Size(20.),
        y_scale: BucketScale::Linear,
    }
}

fn time() -> AxisConfig {
    AxisConfig::new().time()
}

#[component]
fn latency() {
    let data = use_signal(requests());
    rsx! {
        <Heatmap data={data} x={|r: &Request| r.time} y={|r| r.latency} calculate={calc()} x_axis={time()} y_axis={AxisConfig::new().label("ms")} class="h-72" />
    }
}

#[component]
fn log_buckets() {
    let data = use_signal(requests());
    rsx! {
        <Heatmap
            data={data}
            x={|r: &Request| r.time}
            y={|r| r.latency}
            calculate={Calculation { x_buckets: Buckets::Count(30), y_buckets: Buckets::Size(4.), y_scale: BucketScale::Log2 }}
            color={HeatmapColor::scheme(HeatmapScheme::Viridis)}
            x_axis={time()}
            class="h-72"
        />
    }
}

#[component]
fn prometheus() {
    let data = use_signal(histogram());
    rsx! {
        <Heatmap data={data} bucket_layout={BucketLayout::Le} color={HeatmapColor::scheme(HeatmapScheme::Spectral)} x_axis={time()} class="h-72" />
    }
}

#[component]
fn week() {
    let data = use_signal(deploys());
    rsx! {
        <Heatmap
            data={data}
            x={|s: &Slot| s.hour}
            y={|s| s.day}
            value={|s| s.deploys}
            show_value={VisibilityMode::Always}
            hide_le={f64::NEG_INFINITY}
            cell_gap={3.}
            cell_radius={4.}
            color={HeatmapColor::scheme(HeatmapScheme::Greens)}
            y_axis={AxisConfig::new().reverse(true).format(|day| DAYS.get(day as usize).copied().unwrap_or("").into())}
            x_axis={AxisConfig::new().format(|hour| format!("{hour:02}:00").into())}
            class="h-72"
        />
    }
}

#[component]
fn opacity() {
    let data = use_signal(requests());
    rsx! {
        <Heatmap data={data} x={|r: &Request| r.time} y={|r| r.latency} calculate={calc()} color={HeatmapColor::opacity(rgb(0x1e66f5)).exponent(0.5)} x_axis={time()} class="h-72" />
    }
}

#[component]
fn turbo() {
    let data = use_signal(requests());
    rsx! {
        <Heatmap data={data} x={|r: &Request| r.time} y={|r| r.latency} calculate={calc()} color={HeatmapColor::scheme(HeatmapScheme::Turbo).steps(16)} cell_gap={0.} x_axis={time()} class="h-72" />
    }
}

#[component]
fn theme_scheme() {
    let data = use_signal(requests());
    rsx! {
        <Heatmap data={data} x={|r: &Request| r.time} y={|r| r.latency} calculate={calc()} color={HeatmapColor::theme(Scheme::GreenYellowRed)} x_axis={time()} class="h-72" />
    }
}

#[component]
fn tooltip_extras() {
    let data = use_signal(requests());
    rsx! {
        <Heatmap
            data={data}
            x={|r: &Request| r.time}
            y={|r| r.latency}
            calculate={calc()}
            tooltip={HeatmapTooltip::single().y_histogram(true).color_scale(true)}
            color={HeatmapColor::scheme(HeatmapScheme::Magma)}
            x_axis={time()}
            class="h-72"
        />
    }
}

#[component]
fn legend_right() {
    let data = use_signal(requests());
    rsx! {
        <Heatmap
            data={data}
            x={|r: &Request| r.time}
            y={|r| r.latency}
            calculate={calc()}
            legend={HeatmapLegend::shown().right().width(px(120.))}
            color_min={0.}
            color_max={40.}
            color={HeatmapColor::scheme(HeatmapScheme::YlOrRd)}
            x_axis={time()}
            class="h-72"
        />
    }
}

#[component]
fn filtered() {
    let data = use_signal(requests());
    rsx! {
        <Heatmap data={data} x={|r: &Request| r.time} y={|r| r.latency} calculate={calc()} hide_le={3.} hide_ge={30.} color={HeatmapColor::scheme(HeatmapScheme::Blues)} x_axis={time()} class="h-72" />
    }
}

#[component]
fn exemplars() {
    let data = use_signal(requests());
    let marks: Vec<(f64, f64)> = (0..6)
        .map(|step| {
            (
                T0 + f64::from(step) * 20. * MINUTE + 5. * MINUTE,
                380. - f64::from(step) * 30.,
            )
        })
        .collect();
    rsx! {
        <Heatmap data={data} x={|r: &Request| r.time} y={|r| r.latency} calculate={calc()} exemplars={marks} x_axis={time()} class="h-72" />
    }
}

#[component]
fn zoom() {
    let data = use_signal(requests());
    let picked = use_signal(SharedString::Borrowed(
        "Drag a box over the cells. Double-click to zoom out.",
    ));
    rsx! {
        <div class="flex flex-col gap-2 h-72">
            <div class="text-xs">{picked.get()}</div>
            <Heatmap
                data={data}
                x={|r: &Request| r.time}
                y={|r| r.latency}
                calculate={calc()}
                selection={SelectionMode::XY}
                on_select={move |range: SelectedRange, _: &mut Window, _: &mut App| {
                    let (low, high) = range.y.unwrap_or_default();
                    picked.set(SharedString::from(format!("Selected {low:.0}–{high:.0} ms")));
                }}
                x_axis={time()}
                class="flex-1"
            />
        </div>
    }
}

#[component]
fn right_axis() {
    let data = use_signal(requests());
    rsx! {
        <Heatmap
            data={data}
            x={|r: &Request| r.time}
            y={|r| r.latency}
            calculate={calc()}
            y_axis={AxisConfig::new().placement(AxisPlacement::Right).reverse(true).label("ms")}
            legend={HeatmapLegend::hidden()}
            color={HeatmapColor::scheme(HeatmapScheme::Cividis)}
            x_axis={time()}
            class="h-72"
        />
    }
}

#[component]
fn live() {
    let data = use_signal(requests());
    rsx! {
        <div class="flex flex-col gap-2 h-72">
            <div id="slower" class="cursor-pointer text-sm underline" onClick={move || data.update(|rows| rows.iter_mut().for_each(|r| r.latency *= 1.15))}>
                "Make it 15% slower"
            </div>
            <Heatmap data={data} x={|r: &Request| r.time} y={|r| r.latency} calculate={calc()} x_axis={time()} class="flex-1" />
        </div>
    }
}

examples! {
    latency: "Latency over time" => "Raw request samples, counted into 4-minute by 20 ms buckets.",
    log_buckets: "Log buckets" => "Log2 latency buckets, 30 columns, Viridis.",
    prometheus: "Histogram buckets" => "Prometheus-style bucket fields (le), one row per bucket.",
    week: "Cells with values" => "One row per cell, values shown in every cell, gaps and rounded corners.",
    opacity: "Opacity mode" => "One colour, stronger for higher counts.",
    turbo: "16 steps, no gap" => "Turbo quantised to 16 colours, cells touching.",
    theme_scheme: "Theme scheme" => "A colour scheme from your theme file.",
    tooltip_extras: "Tooltip histogram" => "The tooltip shows the whole column and where the value sits on the scale.",
    legend_right: "Fixed colour range" => "Colours pinned to 0–40, legend on the right.",
    filtered: "Hidden cells" => "Cells with 3 or fewer, or 30 or more, are left out.",
    exemplars: "Exemplars" => "Individual samples marked over the buckets.",
    zoom: "Box zoom" => "Drag to zoom into both axes; on_select reports the range.",
    right_axis: "Reversed y axis" => "Largest buckets on top of the axis turned upside down, on the right.",
    live: "Live data" => "A signal as data: the heatmap redraws when it changes.",
}
