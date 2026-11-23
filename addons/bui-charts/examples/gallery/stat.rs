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
struct Sample {
    time: f64,
    cpu: f64,
    requests: f64,
}

fn samples(seed: u64) -> Vec<Sample> {
    (0..60u64)
        .map(|step| {
            let t = step as f64;
            Sample {
                time: T0 + t * MINUTE,
                cpu: 45. + 20. * (t / 9.).sin() + noise(seed * 100 + step) * 8. + t * 0.3,
                requests: (1_200. + 400. * (t / 14.).cos() + noise(seed * 300 + step) * 150.)
                    .round(),
            }
        })
        .collect()
}

fn regions() -> Frame<7> {
    let rows: Vec<[f64; 7]> = (0..30u64)
        .map(|step| {
            let t = step as f64;
            let wave = |seed: u64, base: f64| {
                base + 25. * ((t + seed as f64 * 3.) / 6.).sin() + noise(seed * 50 + step) * 10.
            };
            [
                T0 + t * MINUTE,
                wave(1, 40.),
                wave(2, 55.),
                wave(3, 70.),
                wave(4, 35.),
                wave(5, 85.),
                wave(6, 60.),
            ]
        })
        .collect();
    Frame::new([
        "time", "eu-west", "eu-north", "us-east", "us-west", "ap-south", "sa-east",
    ])
    .with_rows(rows)
}

#[derive(Clone, PartialEq)]
struct Check {
    name: &'static str,
    code: f64,
}

static CODES: LazyLock<ValueMappings> = LazyLock::new(|| {
    ValueMappings::new()
        .value("200", "Healthy", rgb(0x40a02b))
        .range(400., 499., "Client error", rgb(0xdf8e1d))
        .range(500., 599., "Server error", rgb(0xd20f39))
});

static LOAD: LazyLock<[(f64, Hsla); 3]> = LazyLock::new(|| {
    [
        (f64::NEG_INFINITY, rgb(0x40a02b)),
        (50., rgb(0xdf8e1d)),
        (70., rgb(0xd20f39)),
    ]
});

fn percent(value: f64) -> SharedString {
    format!("{value:.1}%").into()
}

fn rate(value: f64) -> SharedString {
    format!("{:.2}k req/s", value / 1_000.).into()
}

#[component]
fn single() {
    let data = use_signal(samples(1));
    rsx! {
        <Stat data={data} value={|s: &Sample| s.cpu} x={|s: &Sample| s.time} reduce={Reduce::Calc(Calc::Last)} thresholds={Thresholds::absolute(&LOAD[..])} format={percent} class="h-40" />
    }
}

#[component]
fn background() {
    let data = use_signal(samples(2));
    rsx! {
        <Stat data={data} value={|s: &Sample| s.requests} x={|s: &Sample| s.time} color={GaugeColor::Fixed(rgb(0x1e66f5))} color_mode={StatColorMode::Background} format={rate} class="h-40" />
    }
}

#[component]
fn solid() {
    let data = use_signal(samples(3));
    rsx! {
        <Stat data={data} value={|s: &Sample| s.cpu} x={|s: &Sample| s.time} color_mode={StatColorMode::BackgroundSolid} thresholds={Thresholds::absolute(&LOAD[..])} graph_mode={GraphMode::None} format={percent} class="h-40" />
    }
}

#[component]
fn line() {
    let data = use_signal(samples(4));
    rsx! {
        <Stat data={data} value={|s: &Sample| s.requests} x={|s: &Sample| s.time} graph_mode={GraphMode::Line} color={GaugeColor::Palette} format={rate} class="h-40" />
    }
}

#[component]
fn grid() {
    let data = use_signal(regions());
    rsx! {
        <Stat data={data} x_field="time" color={GaugeColor::Thresholds} thresholds={Thresholds::absolute(&LOAD[..])} decimals={1} class="h-72" />
    }
}

#[component]
fn list() {
    let data = use_signal(regions());
    rsx! {
        <Stat
            data={data}
            x_field="time"
            fields={&["eu-west", "us-east", "ap-south"]}
            orientation={BarOrientation::Horizontal}
            color_mode={StatColorMode::Background}
            color={GaugeColor::Scheme(Scheme::GreenYellowRed)}
            min={0.}
            max={100.}
            decimals={0}
            class="h-72"
        />
    }
}

#[component]
fn percent_change() {
    let data = use_signal(samples(5));
    rsx! {
        <Stat data={data} value={|s: &Sample| s.requests} x={|s: &Sample| s.time} reduce={Reduce::Calc(Calc::Last)} show_percent_change={true} color={GaugeColor::Fixed(rgb(0x7287fd))} format={rate} class="h-40" />
    }
}

#[component]
fn inverted() {
    let data = use_signal(samples(6));
    rsx! {
        <Stat data={data} value={|s: &Sample| s.cpu} x={|s: &Sample| s.time} show_percent_change={true} percent_change_color={PercentChangeColor::Inverted} thresholds={Thresholds::absolute(&LOAD[..])} format={percent} class="h-40" />
    }
}

#[component]
fn names() {
    let data = use_signal(regions());
    rsx! {
        <Stat data={data} x_field="time" fields={&["eu-west", "us-west"]} text_mode={TextMode::Name} color_mode={StatColorMode::BackgroundSolid} thresholds={Thresholds::absolute(&LOAD[..])} graph_mode={GraphMode::None} class="h-40" />
    }
}

#[component]
fn centered() {
    let data = use_signal(regions());
    rsx! {
        <Stat data={data} x_field="time" fields={&["eu-north", "ap-south", "sa-east"]} justify_mode={JustifyMode::Center} reduce={Reduce::Calc(Calc::Mean)} decimals={1} class="h-40" />
    }
}

#[component]
fn wide_off() {
    let data = use_signal(regions());
    rsx! {
        <Stat data={data} x_field="time" fields={&["eu-west", "us-east"]} wide_layout={false} text_mode={TextMode::ValueAndName} reduce={Reduce::Calc(Calc::Max)} decimals={0} class="h-40" />
    }
}

#[component]
fn mappings() {
    let data = use_signal(vec![
        Check {
            name: "checkout",
            code: 200.,
        },
        Check {
            name: "search",
            code: 503.,
        },
        Check {
            name: "login",
            code: 429.,
        },
        Check {
            name: "billing",
            code: 200.,
        },
    ]);
    rsx! {
        <Stat data={data} name={|c: &Check| c.name} value={|c: &Check| c.code} reduce={Reduce::All { limit: None }} mappings={&*CODES} color_mode={StatColorMode::Background} class="h-48" />
    }
}

#[component]
fn text_sizes() {
    let data = use_signal(regions());
    rsx! {
        <Stat
            data={data}
            x_field="time"
            fields={&["us-east"]}
            reduce={Reduce::Calc(Calc::Max)}
            text_mode={TextMode::ValueAndName}
            title_size={px(14.)}
            value_size={px(40.)}
            color={GaugeColor::Fixed(rgb(0xd20f39))}
            decimals={0}
            class="h-40"
        />
    }
}

#[component]
fn live() {
    let round = use_signal(1u64);
    let data = use_signal(samples(1));
    rsx! {
        <div class="flex flex-col gap-2 h-48">
            <div
                id="next"
                class="cursor-pointer text-sm underline"
                onClick={move || {
                    let next = round.get() + 1;
                    round.set(next);
                    data.set(samples(next));
                }}
            >
                "Load the next hour"
            </div>
            <Stat data={data} value={|s: &Sample| s.cpu} x={|s: &Sample| s.time} show_percent_change={true} thresholds={Thresholds::absolute(&LOAD[..])} format={percent} class="flex-1" />
        </div>
    }
}

examples! {
    single: "Single value" => "The last CPU reading, coloured by thresholds, with a sparkline.",
    background: "Background" => "Gradient background in the value's colour.",
    solid: "Solid background" => "A flat background and no graph.",
    line: "Line graph" => "graph_mode Line: the sparkline without a fill.",
    grid: "Many values" => "Every field of a frame, laid out in a grid.",
    list: "Horizontal list" => "Three fields stacked as rows, coloured on a scheme.",
    percent_change: "Percent change" => "Change between the first and last value.",
    inverted: "Inverted change" => "Rising CPU is bad: increases are red.",
    names: "Names only" => "text_mode Name: the field names, coloured by value.",
    centered: "Centred" => "justify_mode Center with name and mean value.",
    wide_off: "Stacked layout" => "wide_layout off: name above value even when wide.",
    mappings: "Value mappings" => "Status codes mapped to names and colours.",
    text_sizes: "Fixed text sizes" => "title_size and value_size instead of fitted text, with the name shown.",
    live: "Live data" => "A signal as data: value, sparkline and change update.",}
