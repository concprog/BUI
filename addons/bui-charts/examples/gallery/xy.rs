use std::sync::LazyLock;

use blenderui::prelude::*;
use bui_charts::*;


#[derive(Clone, PartialEq)]
pub struct Reading {
    pub temperature: f64,
    pub load: f64,
    pub memory: f64,
    pub humidity: f64,
}

pub fn readings() -> Vec<Reading> {
    (0..200)
        .map(|i| {
            let t = f64::from(i);
            Reading {
                temperature: 15. + t % 25.,
                load: t % 90.,
                memory: t % 70.,
                humidity: 30. + t % 50.,
            }
        })
        .collect()
}

fn curve(f: fn(f64) -> f64, range: std::ops::RangeInclusive<i32>) -> Vec<(f64, f64)> {
    range.map(|i| (f64::from(i), f(f64::from(i)))).collect()
}

static THRESHOLDS: LazyLock<[(f64, Hsla); 3]> = LazyLock::new(|| {
    [
        (0., rgb(0x40a02b)),
        (50., rgb(0xdf8e1d)),
        (80., rgb(0xd20f39)),
    ]
});

static HEAT: LazyLock<[Hsla; 3]> = LazyLock::new(|| {
    [
        rgb(0x04a5e5),
        rgb(0xdf8e1d),
        rgb(0xd20f39),
    ]
});

#[component]
fn basic_scatter() {
    let data = use_signal(readings());
    rsx! {
        <XYChart data={data} class="h-72">
            <Series name="Load" x={|r: &Reading| r.temperature} y={|r| r.load} />
        </XYChart>
    }
}

#[component]
fn lines_and_points() {
    let data = use_signal(curve(|x| x.sqrt() * 10., 0..=49));
    rsx! {
        <XYChart data={data} show={Show::PointsAndLines} point_size={4.} line_width={1.5} class="h-72">
            <Series name="√x" x={|p: &(f64, f64)| p.0} y={|p| p.1} />
            <Series name="√x · 0.5" x={|p: &(f64, f64)| p.0} y={|p| p.1 * 0.5} line_style={LineStyle::DASH} />
            <Series name="√x · 0.25" x={|p: &(f64, f64)| p.0} y={|p| p.1 * 0.25} show={Show::Lines} line_style={LineStyle::DOT} />
        </XYChart>
    }
}

#[component]
fn bubbles() {
    let data = use_signal(readings());
    rsx! {
        <XYChart data={data} size_range={4.0..=32.0} fill_opacity={0.35} class="h-72">
            <Series name="Load" x={|r: &Reading| r.temperature} y={|r| r.load} size={|r| r.humidity} />
        </XYChart>
    }
}

#[component]
fn color_by_value() {
    let data = use_signal(readings());
    rsx! {
        <XYChart data={data} class="h-72">
            <Series name="Gradient" x={|r: &Reading| r.temperature} y={|r| r.load}
                    color={Color::by(|r: &Reading| r.memory, Scheme::BlueYellowRed)} />
            <Series name="Thresholds" x={|r: &Reading| r.temperature} y={|r| r.load + 100.}
                    color={Color::by(|r: &Reading| r.load, Scheme::thresholds(&THRESHOLDS[..]))}
                    point_shape={PointShape::Square} />
        </XYChart>
    }
}

#[component]
fn live_signal() {
    let data = use_signal(readings());
    rsx! {
        <div class="flex flex-col gap-2 h-72">
            <div class="cursor-pointer text-sm underline" onClick={move || data.update(|rows| rows.iter_mut().for_each(|r| r.load = (r.load * 1.7) % 90.))}>
                "Shuffle"
            </div>
            <XYChart data={data} class="flex-1">
                <Series name="Load" x={|r: &Reading| r.temperature} y={|r| r.load} />
            </XYChart>
        </div>
    }
}

fn waves() -> Frame<3> {
    let rows: Vec<[f64; 3]> = (0..60)
        .map(|t| {
            let t = f64::from(t);
            [
                t,
                (t / 5.).sin().mul_add(40., 50.),
                (t / 5.).cos().mul_add(40., 50.),
            ]
        })
        .collect();
    Frame::new(["Time", "cpu host-a", "cpu host-b"]).with_rows(rows)
}

#[component]
fn auto_mapping() {
    let frame = use_signal(waves());
    rsx! { <XYChart data={frame} show={Show::PointsAndLines} class="h-72" /> }
}

fn people() -> Frame<3> {
    Frame::from_columns([
        ("Height cm", [150., 160., 170., 180., 190.]),
        ("Weight kg", [50., 58., 70., 82., 95.]),
        ("Age", [18., 25., 33., 47., 62.]),
    ])
    .size("Age")
    .color("Age", Scheme::Purples)
}

#[component]
fn frame_roles() {
    let frame = use_signal(people());
    rsx! { <XYChart data={frame} size_range={8.0..=28.0} class="h-72" /> }
}

fn traffic() -> Frame<3> {
    Frame::new(["Time", "Requests", "Errors"]).with_rows([
        [0., 120., 2.],
        [1., 340., 9.],
        [2., 290., 4.],
        [3., 410., 12.],
    ])
}

#[component]
fn frame_manual() {
    let frame = use_signal(traffic());
    rsx! {
        <XYChart data={frame} class="h-72">
            <Series name="Errors vs requests" x={|r: &[f64; 3]| r[1]} y={|r| r[2]} size={|r| r[0]} size_range={6.0..=18.0} />
        </XYChart>
    }
}

#[component]
fn log_scale() {
    let data = use_signal(curve(|day| 1.5f64.powf((day / 2.).floor()), 1..=40));
    rsx! {
        <XYChart
            data={data}
            show={Show::PointsAndLines}
            x_axis={AxisConfig::new().label("Day").soft_min(0.).soft_max(45.)}
            y_axis={AxisConfig::new().label("Requests").scale(ScaleKind::Log { base: 10. }).min(1.)}
            class="h-72"
        >
            <Series name="Growth" x={|p: &(f64, f64)| p.0} y={|p| p.1} />
        </XYChart>
    }
}

#[component]
fn symlog_right_axis() {
    let data = use_signal(curve(|x| x.powi(3), -30..=30));
    rsx! {
        <XYChart
            data={data}
            y_axis={AxisConfig::new()
                .scale(ScaleKind::Symlog { base: 10., linear_threshold: 10. })
                .placement(AxisPlacement::Right)
                .centered_zero(true)
                .color_mode(AxisColorMode::Series)
                .border(true)}
            legend={Legend::hidden()}
            class="h-72"
        >
            <Series name="x³" x={|p: &(f64, f64)| p.0} y={|p| p.1} color={rgb(0x8839ef)} />
        </XYChart>
    }
}

#[component]
fn table_legend() {
    let data = use_signal(readings());
    rsx! {
        <XYChart
            data={data}
            legend={Legend::table()
                .right()
                .width(px(220.))
                .calcs([Calc::Min, Calc::Mean, Calc::Max])
                .sort_by(LegendSort::Calc(Calc::Mean))
                .sort_desc(true)}
            class="h-72"
        >
            <Series name="Load" x={|r: &Reading| r.temperature} y={|r| r.load} />
            <Series name="Memory" x={|r: &Reading| r.temperature} y={|r| r.memory} />
            <Series name="Humidity" x={|r: &Reading| r.temperature} y={|r| r.humidity} />
        </XYChart>
    }
}

#[component]
fn tooltip_and_hiding() {
    let data = use_signal(readings());
    rsx! {
        <XYChart data={data} tooltip={Tooltip::single().max_width(px(180.))} class="h-72">
            <Series name="Load" x={|r: &Reading| r.temperature} y={|r| r.load} />
            <Series name="Baseline" x={|r: &Reading| r.temperature} y={|_| 45.} show={Show::Lines}
                    hide_from={HideFrom { tooltip: true, legend: true, viz: false }} />
        </XYChart>
    }
}

#[component]
fn sparkline() {
    let data = use_signal(curve(|x| (x / 3.).sin(), 0..=29));
    rsx! {
        <XYChart
            data={data}
            show={Show::Lines}
            x_axis={AxisConfig::new().placement(AxisPlacement::Hidden).grid(false)}
            y_axis={AxisConfig::new().placement(AxisPlacement::Hidden).grid(false)}
            legend={Legend::hidden()}
            tooltip={Tooltip::hidden()}
            class="h-12 w-40"
        >
            <Series x={|p: &(f64, f64)| p.0} y={|p| p.1} />
        </XYChart>
    }
}

#[component]
fn formatting() {
    let data = use_signal(readings());
    rsx! {
        <XYChart
            data={data}
            x_axis={AxisConfig::new().label("Temperature").format(|v| format!("{v:.0} °C").into())}
            y_axis={AxisConfig::new().label("Load").decimals(1).min(0.).max(100.)}
            class="h-72"
        >
            <Series name="Load" x={|r: &Reading| r.temperature} y={|r| r.load} />
        </XYChart>
    }
}

const HOSTS: [&str; 3] = ["host-a", "host-b", "host-c"];

#[derive(Clone, Copy, PartialEq)]
struct Row {
    time: f64,
    values: [f64; 3],
}

fn host_rows() -> Vec<Row> {
    (0..40)
        .map(|t| {
            let t = f64::from(t);
            Row {
                time: t,
                values: [0., 2., 4.].map(|phase| ((t + phase) / 4.).sin().mul_add(30., 50.)),
            }
        })
        .collect()
}

#[component]
fn series_from_a_loop() {
    let rows = use_signal(host_rows());
    rsx! {
        <XYChart data={rows} show={Show::Lines} class="h-72">
            {for (column, host) in HOSTS.into_iter().enumerate() {
                <Series key={column} name={host} x={|r: &Row| r.time} y={move |r: &Row| r.values[column]} />
            }}
        </XYChart>
    }
}

#[component]
fn shared_defaults() {
    let data = use_signal(readings());
    rsx! {
        <XYChart
            data={data}
            point_shape={PointShape::Square}
            point_size={6.}
            point_stroke_width={0.}
            fill_opacity={0.8}
            class="h-72"
        >
            <Series name="Squares" x={|r: &Reading| r.temperature} y={|r| r.load} />
            <Series name="Circles" x={|r: &Reading| r.temperature} y={|r| r.memory}
                    point_shape={PointShape::Circle} point_stroke_width={1.} fill_opacity={0.2} />
        </XYChart>
    }
}

#[component]
fn custom_gradient() {
    let data = use_signal(readings());
    rsx! {
        <XYChart data={data} class="h-72">
            <Series name="Load" x={|r: &Reading| r.temperature} y={|r| r.load}
                    color={Color::by(|r: &Reading| r.memory, Scheme::gradient(&HEAT[..]))} />
        </XYChart>
    }
}

examples! {
    basic_scatter: "Basic scatter" => "One series of points with every default.",
    lines_and_points: "Lines and points" => "Show modes and line styles, set on the chart and overridden per series.",
    bubbles: "Bubbles" => "Point size driven by a value, scaled by area.",
    color_by_value: "Colour by value" => "A gradient scheme and a threshold scheme.",
    live_signal: "Live data" => "A signal as data: the chart redraws when it is written.",
    auto_mapping: "Auto mapping" => "A frame with no series: first field is x, the rest are y.",
    frame_roles: "Frame roles" => "Size and colour fields that drive every auto series.",
    frame_manual: "Manual series over a frame" => "Pick frame fields yourself.",
    log_scale: "Log scale" => "Log axis with soft and hard limits.",
    symlog_right_axis: "Symlog on the right" => "Negative and positive magnitudes on one axis.",
    table_legend: "Table legend" => "Calcs, sorting and right placement. Click to isolate, Ctrl-click to toggle.",
    tooltip_and_hiding: "Tooltip and hiding" => "A reference line kept out of the legend and tooltip.",
    sparkline: "Sparkline" => "Everything but the line turned off.",
    formatting: "Formatting" => "Custom tick text, fixed decimals, pinned range.",
    series_from_a_loop: "Series from a loop" => "One series per column, generated in view!.",
    shared_defaults: "Shared defaults" => "Chart-wide styling with per-series overrides.",
    custom_gradient: "Custom gradient" => "An inline gradient from a static colour list.",
}
