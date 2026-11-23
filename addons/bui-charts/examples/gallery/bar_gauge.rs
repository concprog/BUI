use std::sync::LazyLock;

use blenderui::prelude::*;
use bui_charts::*;


#[derive(Clone, PartialEq)]
pub struct Disk {
    pub mount: &'static str,
    pub used: f64,
}

pub fn disks() -> Vec<Disk> {
    [
        ("/", 42.),
        ("/home", 67.),
        ("/var", 88.),
        ("/tmp", 12.),
        ("/data", 95.),
    ]
    .map(|(mount, used)| Disk { mount, used })
    .into()
}

pub fn services() -> Frame<4> {
    Frame::new(["api", "web", "worker", "cron"]).with_rows([
        [120., 340., 80., 12.],
        [180., 310., 95., 20.],
        [240., 290., 70., 15.],
        [210., 360., 110., 18.],
    ])
}

static STEPS: LazyLock<[(f64, Hsla); 3]> = LazyLock::new(|| {
    [
        (f64::NEG_INFINITY, rgb(0x40a02b)),
        (60., rgb(0xdf8e1d)),
        (85., rgb(0xd20f39)),
    ]
});

#[component]
fn disk_usage() {
    let data = use_signal(disks());
    rsx! {
        <BarGauge data={data} name={|d: &Disk| d.mount} value={|d| d.used} class="h-64" />
    }
}

#[component]
fn retro_lcd() {
    let data = use_signal(disks());
    rsx! {
        <BarGauge data={data} name={|d: &Disk| d.mount} value={|d| d.used} display={BarDisplay::Lcd} class="h-64" />
    }
}

#[component]
fn basic_bars() {
    let data = use_signal(disks());
    rsx! {
        <BarGauge data={data} name={|d: &Disk| d.mount} value={|d| d.used} display={BarDisplay::Basic} class="h-64" />
    }
}

#[component]
fn columns() {
    let data = use_signal(disks());
    rsx! {
        <BarGauge
            data={data}
            name={|d: &Disk| d.mount}
            value={|d| d.used}
            orientation={BarOrientation::Vertical}
            display={BarDisplay::Lcd}
            class="h-64"
        />
    }
}

#[component]
fn reduce_a_frame() {
    let frame = use_signal(services());
    rsx! {
        <BarGauge data={frame} reduce={Reduce::Calc(Calc::Last)} max={400.} class="h-64" />
    }
}

#[component]
fn pick_fields() {
    let frame = use_signal(services());
    rsx! {
        <BarGauge
            data={frame}
            reduce={Reduce::Calc(Calc::Mean)}
            fields={&["api", "worker"]}
            max={300.}
            display={BarDisplay::Basic}
            class="h-64"
        />
    }
}

#[component]
fn custom_thresholds() {
    let data = use_signal(disks());
    rsx! {
        <BarGauge
            data={data}
            name={|d: &Disk| d.mount}
            value={|d| d.used}
            thresholds={Thresholds::percentage(&STEPS[..])}
            class="h-64"
        />
    }
}

#[component]
fn scheme_gradient() {
    let data = use_signal(disks());
    rsx! {
        <BarGauge
            data={data}
            name={|d: &Disk| d.mount}
            value={|d| d.used}
            color={GaugeColor::Scheme(Scheme::GreenYellowRed)}
            class="h-64"
        />
    }
}

#[component]
fn palette_colors() {
    let frame = use_signal(services());
    rsx! {
        <BarGauge
            data={frame}
            color={GaugeColor::Palette}
            orientation={BarOrientation::Vertical}
            max={400.}
            class="h-64"
        />
    }
}

#[component]
fn text_modes() {
    let data = use_signal(disks());
    rsx! {
        <BarGauge
            data={data}
            name={|d: &Disk| d.mount}
            value={|d| d.used}
            value_mode={ValueMode::Text}
            name_placement={NamePlacement::Left}
            class="h-64"
        />
    }
}

#[component]
fn manual_sizing() {
    let data = use_signal(disks());
    rsx! {
        <BarGauge
            data={data}
            name={|d: &Disk| d.mount}
            value={|d| d.used}
            sizing={Sizing::Manual { min_width: px(8.), min_height: px(16.), max_height: px(24.) }}
            name_placement={NamePlacement::Left}
            display={BarDisplay::Lcd}
            class="h-64"
        />
    }
}

#[component]
fn units_and_range() {
    let data = use_signal(disks());
    rsx! {
        <BarGauge
            data={data}
            name={|d: &Disk| d.mount}
            value={|d| d.used * 10.24}
            min={0.}
            max={1024.}
            format={|v| format!("{v:.0} GiB").into()}
            show_unfilled={false}
            color={GaugeColor::Fixed(rgb(0x1e66f5))}
            class="h-64"
        />
    }
}

#[component]
fn live_value() {
    let data = use_signal(disks());
    rsx! {
        <div class="flex flex-col gap-2 h-64">
            <div class="cursor-pointer text-sm underline" onClick={move || data.update(|rows| rows.iter_mut().for_each(|d| d.used = (d.used * 1.37) % 100.))}>
                "Shuffle"
            </div>
            <BarGauge data={data} name={|d: &Disk| d.mount} value={|d| d.used} class="flex-1" />
        </div>
    }
}

#[component]
fn with_legend() {
    let frame = use_signal(services());
    rsx! {
        <BarGauge
            data={frame}
            color={GaugeColor::Palette}
            max={400.}
            legend={Legend::list().right()}
            text_mode={TextMode::Value}
            class="h-64"
        />
    }
}

examples! {
    disk_usage: "Disk usage" => "One bar per row, gradient through the default thresholds.",
    retro_lcd: "Retro LCD" => "Cells lit up to the value, each coloured by its own threshold.",
    basic_bars: "Basic" => "Flat bars with a solid edge.",
    columns: "Columns" => "Vertical bars side by side.",
    reduce_a_frame: "Reduce a frame" => "One bar per field, showing its last value.",
    pick_fields: "Pick fields" => "Only some fields, reduced to their mean.",
    custom_thresholds: "Percentage thresholds" => "Your own steps, as a percentage of the range.",
    scheme_gradient: "Colour scheme" => "A continuous gradient instead of thresholds.",
    palette_colors: "Palette colours" => "Each bar takes the next palette colour.",
    text_modes: "Names on the left" => "Name beside the bar, value in the text colour.",
    manual_sizing: "Manual sizing" => "Thin rows with a fixed maximum height.",
    units_and_range: "Units and range" => "Custom range and value text, no unfilled track.",
    live_value: "Live data" => "A signal as data: bars move when it changes.",
    with_legend: "With a legend" => "A legend you can click to hide bars.",}
