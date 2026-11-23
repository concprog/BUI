use blenderui::prelude::*;
use bui_charts::*;
use bui_panels::prelude::*;

use crate::registry::Example;
use crate::bar_gauge::{Disk, disks, services};
use crate::dashboards::{gestures, keys};
use crate::flame_graph::cpu;
use crate::xy::{Reading, readings};

fn jitter(seed: u64) -> f64 {
    let mixed = seed
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    (mixed >> 40) as f64 / f64::from(1u32 << 24) - 0.5
}

#[component]
fn note_line(text: &'static str) {
    let (_, _, muted) = crate::page();
    rsx! { <div class="text-xs" text_color={muted}>{text.to_owned()}</div> }
}

/// The panels' look follows the kit theme.
pub fn mode() -> ThemeMode {
    if blenderui::theme_name() == "dark" { ThemeMode::Dark } else { ThemeMode::Light }
}

#[component]
fn dashboard() {
    let latency = use_signal(readings());
    let usage = use_signal(disks());
    let traffic = use_signal(services());
    let profile = use_signal(cpu());
    let tick = use_signal(0u64);
    let theme = mode();
    let refresh = move || {
        let seed = tick.get() + 1;
        tick.set(seed);
        latency.update(|rows| {
            for (index, row) in rows.iter_mut().enumerate() {
                row.load = (row.load + jitter(seed ^ index as u64) * 6.).clamp(0., 90.);
            }
        });
        usage.update(|rows| {
            for (index, row) in rows.iter_mut().enumerate() {
                row.used = (row.used + jitter(seed ^ (index as u64) << 8) * 10.).clamp(0., 100.);
            }
        });
        let Some(last) = traffic.rows(|rows| rows.last().copied()) else {
            return;
        };
        let next: [f64; 4] = std::array::from_fn(|field| {
            (last[field] * (1. + jitter(seed ^ field as u64) * 0.4)).max(0.)
        });
        traffic.update(|frame| frame.push(next));
    };

    rsx! {
        <div class="flex flex-col size-full">
            <div class="flex items-center gap-3 px-4 py-2 text-sm">
                <div class="font-semibold">"Checkout service"</div>
                <note_line text="Drag tabs, resize dividers or float panels. Space steps into a panel; Ctrl+scroll or pinch zooms a chart." />
                <div class="flex-1" />
                <div
                    id="refresh"
                    class="flex items-center gap-1 px-2 h-7 rounded-md border text-xs cursor-pointer"
                    onClick={move || refresh()}
                >
                    "↻ Refresh"
                </div>
            </div>
            <div class="relative flex-1 min-h-0">
                <Workspace theme={theme} no_floating={false} keep_pushed={false} keymap={keys()} gesture_keys={gestures()}>
                    <Row sizes={[3.0, 2.0]}>
                        <Column sizes={[1.0, 1.2]}>
                            <Panel id="latency">
                                <View id="latency" title="Latency vs load" icon={IconName::Grid} gestures={GestureOwner::Exclusive}>
                                    <XYChart data={latency} class="size-full p-2" legend={Legend::list()}>
                                        <Series name="p50" x={|r: &Reading| r.load} y={|r| r.temperature} />
                                        <Series name="p99" x={|r: &Reading| r.load} y={|r| r.humidity} />
                                    </XYChart>
                                </View>
                            </Panel>
                            <Panel id="profiles">
                                <View id="flame" title="CPU profile" icon={IconName::Console} gestures={GestureOwner::Exclusive}>
                                    <FlameGraph data={profile} layout={PaneLayout::Single(Pane::FlameGraph)} header={false} />
                                </View>
                                <View id="calls" title="Call tree" icon={IconName::Outliner} gestures={GestureOwner::Exclusive}>
                                    <FlameGraph data={profile} layout={PaneLayout::Single(Pane::CallTree)} header={false} />
                                </View>
                                <View id="top" title="Top functions" icon={IconName::File} gestures={GestureOwner::Exclusive}>
                                    <FlameGraph data={profile} layout={PaneLayout::Single(Pane::TopTable)} header={false} />
                                </View>
                            </Panel>
                        </Column>
                        <Column sizes={[1.0, 1.0]}>
                            <Panel id="disks">
                                <View id="disks" title="Disk usage" icon={IconName::Folder} gestures={GestureOwner::Exclusive}>
                                    <BarGauge data={usage} name={|d: &Disk| d.mount} value={|d| d.used} decimals={0} orientation={BarOrientation::Horizontal} class="size-full p-2" />
                                </View>
                                <View id="disks-lcd" title="Disk usage (LCD)" icon={IconName::Folder} gestures={GestureOwner::Exclusive}>
                                    <BarGauge data={usage} name={|d: &Disk| d.mount} value={|d| d.used} decimals={0} display={BarDisplay::Lcd} orientation={BarOrientation::Horizontal} class="size-full p-2" />
                                </View>
                            </Panel>
                            <Panel id="traffic">
                                <View id="traffic" title="Requests / s" icon={IconName::Image} gestures={GestureOwner::Exclusive}>
                                    <BarGauge
                                        data={traffic}
                                        reduce={Reduce::Calc(Calc::Last)}
                                        max={500.}
                                        decimals={0}
                                        orientation={BarOrientation::Vertical}
                                        color={GaugeColor::Palette}
                                        class="size-full p-2"
                                    />
                                </View>
                            </Panel>
                        </Column>
                    </Row>
                </Workspace>
            </div>
        </div>
    }
}

pub const EXAMPLES: &[Example] = &[Example {
    title: "Service dashboard",
    description: "Every chart in movable, tabbed bui-panels panels.",
    render: || rsx! { <dashboard /> }.into_any_element(),
}];
