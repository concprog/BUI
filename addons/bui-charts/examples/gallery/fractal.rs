use blenderui::prelude::*;
use bui_charts::*;
use bui_panels::prelude::*;

use crate::registry::Example;
use crate::dashboard::mode;
use crate::dashboards::keys;
use crate::finance::noise;

const REGIONS: [&str; 4] = ["North", "South", "East", "West"];
const METRICS: [&str; 5] = ["CPU", "Memory", "Latency", "Errors", "Queue"];

struct Palette {
    base: u32,
    mantle: u32,
    crust: u32,
    text: u32,
    surface: [u32; 3],
    blue: u32,
}

const LATTE: Palette = Palette {
    base: 0xeff1f5,
    mantle: 0xe6e9ef,
    crust: 0xdce0e8,
    text: 0x4c4f69,
    surface: [0xccd0da, 0xbcc0cc, 0xacb0be],
    blue: 0x1e66f5,
};

const MOCHA: Palette = Palette {
    base: 0x1e1e2e,
    mantle: 0x181825,
    crust: 0x11111b,
    text: 0xcdd6f4,
    surface: [0x313244, 0x45475a, 0x585b70],
    blue: 0x89b4fa,
};

/// A kit theme in Catppuccin's colors (a theme file's text: see BUI's docs/theming.md).
fn theme_json(name: &str, mode: &str, p: &Palette) -> String {
    let hex = |c: u32| format!("#{c:06x}");
    format!(
        r#"{{ "name": "Catppuccin", "themes": [{{ "name": "{name}", "mode": "{mode}", "colors": {{
            "background": "{base}", "foreground": "{text}", "border": "{overlay}",
            "primary.background": "{blue}", "primary.foreground": "{base}",
            "secondary.background": "{surface}", "accent.background": "{surface}",
            "popover.background": "{mantle}", "tooltip.background": "{crust}", "tooltip.text": "{text}",
            "region.background": "{base}", "panel.background": "{mantle}", "selection.background": "{highlight}"
        }} }}] }}"#,
        base = hex(p.base),
        text = hex(p.text),
        overlay = hex(p.surface[1]),
        blue = hex(p.blue),
        surface = hex(p.surface[0]),
        mantle = hex(p.mantle),
        crust = hex(p.crust),
        highlight = hex(p.surface[2]),
    )
}

/// Switches the kit theme (and with it the charts and panels) to Catppuccin Mocha or Latte.
fn catppuccin(dark: bool, cx: &mut App) {
    let (name, mode, palette) = if dark { ("Catppuccin Mocha", "dark", &MOCHA) } else { ("Catppuccin Latte", "light", &LATTE) };
    if blenderui::load_theme(&theme_json(name, mode, palette)).is_ok() {
        blenderui::set_theme(name);
    }
    // charts take their colors while rendering: render every window again
    cx.notify_all();
}

fn gestures() -> GestureKeys {
    GestureKeys {
        scale: Vec::new(),
        step: vec!["Alt".into()],
        ..default_gesture_keys()
    }
}

#[derive(Clone, PartialEq)]
pub struct Reading {
    pub name: &'static str,
    pub time: f64,
    pub value: f64,
}

fn readings(name: &'static str, seed: u64, scale: f64) -> Vec<Reading> {
    let mut level = 0.5;
    (0..48u64)
        .map(|index| {
            level = (level + (noise(seed * 977 + index) - 0.5) * 0.12).clamp(0.05, 0.98);
            Reading {
                name,
                time: 1_729_000_000_000. + index as f64 * 300_000.,
                value: (level * scale).round(),
            }
        })
        .collect()
}

#[component]
fn tile(index: usize) {
    let metric = METRICS[index % METRICS.len()];
    let data = use_signal(readings(
        metric,
        index as u64 + 1,
        if metric == "Latency" { 400. } else { 100. },
    ));
    match index % 5 {
        0 => rsx! { <Stat data={data} value={|r: &Reading| r.value} x={|r: &Reading| r.time} decimals={0} class="size-full" /> }.into_any_element(),
        1 => rsx! { <Gauge data={data} value={|r: &Reading| r.value} name={|r: &Reading| r.name} class="size-full" /> }.into_any_element(),
        2 => rsx! {
            <TimeSeries data={data} x={|r: &Reading| r.time} fill_opacity={0.2} legend={Legend::hidden()} class="size-full p-1">
                <TrendSeries name={metric} y={|r: &Reading| r.value} />
            </TimeSeries>
        }
        .into_any_element(),
        3 => rsx! { <BarGauge data={data} value={|r: &Reading| r.value} class="size-full p-2" /> }.into_any_element(),
        _ => rsx! {
            <Histogram data={data} legend={Legend::hidden()} class="size-full p-1">
                <HistogramSeries name={metric} value={|r: &Reading| r.value} />
            </Histogram>
        }
        .into_any_element(),
    }
}

#[component]
fn toggle() {
    let dark = blenderui::theme_name() == "dark";
    let (_, foreground, _) = crate::page();
    let label = if dark { "☾ Mocha" } else { "☀ Latte" };
    rsx! {
        <div
            id="theme-toggle"
            class="px-3 py-1 rounded-md border text-xs cursor-pointer"
            border_color={Rgba { a: 0.2, ..foreground }}
            bg={Rgba { a: 0.08, ..foreground }}
            onClick={move |_, _, cx: &mut App| catppuccin(!dark, cx)}
        >
            {label}
        </div>
    }
}

#[component]
fn fractal() {
    let (_, _, muted) = crate::page();
    rsx! {
        <div class="flex flex-col size-full">
            <div class="flex items-center gap-3 px-4 py-2 text-sm">
                <div class="font-semibold">"Operations"</div>
                <div class="text-xs flex-1" text_color={muted}>"About 90 nested panels in 4 regions. Alt+scroll steps in and out. On a touchpad: two-finger swipe pans, pinch or Ctrl+swipe zooms, Space frames a panel."</div>
                <toggle />
            </div>
            <div class="relative flex-1 min-h-0">
                <Workspace theme={mode()} navigation={Navigation::Free} no_floating={false} keep_pushed={false} keymap={keys()} gesture_keys={gestures()}>
                    {regions()}
                </Workspace>
            </div>
        </div>
    }
}

pub const EXAMPLES: &[Example] = &[Example {
    title: "Operations",
    description: "About 100 panels with tabs, Alt+scroll navigation and a Catppuccin toggle.",
    render: || rsx! { <fractal /> }.into_any_element(),
}];

#[component]
fn feature(region: usize, kind: usize) {
    let data = use_signal(readings("Requests", 900 + region as u64, 5000.));
    match kind {
        0 => rsx! {
            <TimeSeries data={data} x={|r: &Reading| r.time} fill_opacity={0.25} legend={Legend::hidden()} class="size-full p-2">
                <TrendSeries name="Requests" y={|r: &Reading| r.value} />
            </TimeSeries>
        }
        .into_any_element(),
        1 => rsx! {
            <Histogram data={data} legend={Legend::hidden()} class="size-full p-2">
                <HistogramSeries name="Requests" value={|r: &Reading| r.value} />
            </Histogram>
        }
        .into_any_element(),
        _ => rsx! { <Stat data={data} value={|r: &Reading| r.value} x={|r: &Reading| r.time} show_percent_change={true} class="size-full" /> }.into_any_element(),
    }
}

const DEPTHS: [u32; 4] = [4, 3, 4, 3];

fn leaf(region: usize, path: &str, index: usize) -> AnyElement {
    let name = REGIONS[region];
    rsx! {
        <Panel id={format!("{path}-panel")}>
            <View
                id={format!("{path}-view")}
                title={format!("{name} {} {}", METRICS[index % METRICS.len()], index % 97 + 1)}
                icon={IconName::Image}
            >
                <tile index={index} />
            </View>
        </Panel>
    }
    .into_any_element()
}

fn overview(region: usize) -> AnyElement {
    let name = REGIONS[region];
    rsx! {
        <Panel id={format!("{name}-overview")}>
            {for (kind, title) in ["Traffic", "Distribution", "Summary"].into_iter().enumerate() {
                <View id={format!("{name}-{title}")} title={format!("{name} {title}")} icon={IconName::Grid}>
                    <feature region={region} kind={kind} />
                </View>
            }}
        </Panel>
    }
    .into_any_element()
}

fn split(
    region: usize,
    path: &str,
    depth: u32,
    row: bool,
    index: usize,
) -> AnyElement {
    if depth == 0 {
        return leaf(region, path, index);
    }
    let first = if path.len() == 1 {
        overview(region)
    } else {
        leaf(region, &format!("{path}0"), index)
    };
    let second = split(
        region,
        &format!("{path}1"),
        depth - 1,
        !row,
        index * 3 + 1,
    );
    let third = split(
        region,
        &format!("{path}2"),
        depth - 1,
        !row,
        index * 3 + 2,
    );
    let seed = index as u64 * 7 + 3;
    let sizes = vec![
        1.2 + noise(seed) * 1.8,
        0.6 + noise(seed + 1),
        0.5 + noise(seed + 2) * 1.2,
    ];
    if row {
        rsx! { <Row id={format!("{path}-row")} sizes={sizes}>{first}{second}{third}</Row> }.into_any_element()
    } else {
        rsx! { <Column id={format!("{path}-column")} sizes={sizes}>{first}{second}{third}</Column> }.into_any_element()
    }
}

fn regions() -> AnyElement {
    let [north, south, east, west] = [0, 1, 2, 3].map(|region| {
        split(
            region,
            &REGIONS[region][..1],
            DEPTHS[region],
            false,
            region + 1,
        )
    });
    rsx! { <Row id="regions" sizes={vec![1.4, 1.1, 1.3, 1.1]}>{north}{south}{east}{west}</Row> }.into_any_element()
}
