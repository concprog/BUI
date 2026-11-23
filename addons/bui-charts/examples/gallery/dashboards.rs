use blenderui::prelude::*;
use bui_charts::*;
use bui_panels::prelude::*;

use crate::registry::Example;
use crate::dashboard::mode;
use crate::category::{Step, steps};
use crate::finance::{Bar, bars, noise};
use crate::flow::{Flow, energy};
use crate::quality::{Defect, Sample, defects, samples};

#[component]
fn dash_header(title: &'static str, note: &'static str) {
    let (_, _, muted) = crate::page();
    rsx! {
        <div class="flex items-center gap-3 px-4 py-2 text-sm">
            <div class="font-semibold">{title.to_owned()}</div>
            <div class="text-xs" text_color={muted}>{note.to_owned()}</div>
        </div>
    }
}

#[derive(Clone, PartialEq)]
struct Exposure {
    book: &'static str,
    values: [f64; 6],
}

#[derive(Clone, PartialEq)]
struct Kpi {
    name: &'static str,
    time: f64,
    value: f64,
}

fn pnl() -> Vec<Kpi> {
    let mut total = 0.;
    (0..60u64)
        .map(|index| {
            total += (noise(index + 40) - 0.45) * 30.;
            Kpi {
                name: "P&L (k$)",
                time: 1_729_000_000_000. + index as f64 * 600_000.,
                value: total,
            }
        })
        .collect()
}

#[component]
fn trading() {
    let prices = use_signal(bars(90, 21, 0.03));
    let profit = use_signal(pnl());
    let attribution = use_signal(steps(&[
        ("Equities", 120.),
        ("Rates", -45.),
        ("FX", 30.),
        ("Credit", -20.),
        ("Fees", -15.),
    ]));
    let books = use_signal(vec![
        Exposure {
            book: "Desk A",
            values: [70., 40., 55., 30., 65., 50.],
        },
        Exposure {
            book: "Desk B",
            values: [35., 80., 45., 70., 30., 60.],
        },
    ]);
    let axes: Vec<SharedString> = [
        "Equity",
        "Rates",
        "FX",
        "Credit",
        "Commodities",
        "Volatility",
    ]
    .map(SharedString::Borrowed)
    .to_vec();
    let risk = use_signal(vec![Kpi {
        name: "VaR used",
        time: 0.,
        value: 72.,
    }]);
    rsx! {
        <div class="flex flex-col size-full">
            <dash_header title="Trading desk" note="Prices, P&L, attribution and risk. Space steps into a panel; Ctrl+scroll or pinch zooms a chart." />
            <div class="relative flex-1 min-h-0">
                <Workspace theme={mode()} no_floating={false} keep_pushed={false} keymap={keys()} gesture_keys={gestures()}>
                    <Row sizes={[3.0, 2.0]}>
                        <Column sizes={[1.6, 1.0]}>
                            <Panel id="prices">
                                <View id="candles" title="ACME daily" icon={IconName::Image} gestures={GestureOwner::Exclusive}>
                                    <Candlestick data={prices} x={|b: &Bar| b.time} open={|b: &Bar| b.open} high={|b: &Bar| b.high} low={|b: &Bar| b.low} close={|b: &Bar| b.close} volume={|b: &Bar| b.volume} class="size-full p-2" />
                                </View>
                                <View id="heikin" title="Heikin-Ashi" icon={IconName::Image} gestures={GestureOwner::Exclusive}>
                                    <HeikinAshi data={prices} x={|b: &Bar| b.time} open={|b: &Bar| b.open} high={|b: &Bar| b.high} low={|b: &Bar| b.low} close={|b: &Bar| b.close} class="size-full p-2" />
                                </View>
                                <View id="renko" title="Renko" icon={IconName::Image} gestures={GestureOwner::Exclusive}>
                                    <Renko data={prices} close={|b: &Bar| b.close} class="size-full p-2" />
                                </View>
                            </Panel>
                            <Panel id="pnl">
                                <View id="pnl-chart" title="Intraday P&L" icon={IconName::Image} gestures={GestureOwner::Exclusive}>
                                    <TimeSeries data={profit} x={|k: &Kpi| k.time} fill_opacity={0.2} gradient={GradientMode::Opacity} legend={Legend::hidden()} class="size-full p-2">
                                        <TrendSeries name="P&L (k$)" y={|k: &Kpi| k.value} />
                                    </TimeSeries>
                                </View>
                            </Panel>
                        </Column>
                        <Column sizes={[0.6, 1.0, 1.2]}>
                            <Panel id="stats">
                                <View id="pnl-stat" title="P&L" icon={IconName::Image} gestures={GestureOwner::Exclusive}>
                                    <Stat data={profit} value={|k: &Kpi| k.value} x={|k: &Kpi| k.time} show_percent_change={true} decimals={1} class="size-full" />
                                </View>
                                <View id="var" title="VaR" icon={IconName::Info} gestures={GestureOwner::Exclusive}>
                                    <Gauge data={risk} value={|k: &Kpi| k.value} name={|k: &Kpi| k.name} class="size-full" />
                                </View>
                            </Panel>
                            <Panel id="attribution">
                                <View id="waterfall" title="P&L attribution" icon={IconName::Grid} gestures={GestureOwner::Exclusive}>
                                    <Waterfall data={attribution} name={|s: &Step| s.name} value={|s: &Step| s.value} total_label="Net" class="size-full p-2" />
                                </View>
                            </Panel>
                            <Panel id="exposure">
                                <View id="spider" title="Exposure" icon={IconName::Outliner} gestures={GestureOwner::Exclusive}>
                                    <Spider data={books} axes={axes} name={|e: &Exposure| e.book} value={|e: &Exposure, axis: usize| e.values[axis]} max={100.} class="size-full p-2" />
                                </View>
                                <View id="frontier" title="Allocation" icon={IconName::Image} gestures={GestureOwner::Exclusive}>
                                    <EfficientFrontier
                                        returns={vec![0.05, 0.08, 0.11, 0.14]}
                                        covariance={vec![
                                            vec![0.002, 0.001, 0.0005, 0.0004],
                                            vec![0.001, 0.008, 0.002, 0.0015],
                                            vec![0.0005, 0.002, 0.018, 0.005],
                                            vec![0.0004, 0.0015, 0.005, 0.035],
                                        ]}
                                        risk_free={0.03}
                                        portfolios={800}
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

#[derive(Clone, PartialEq)]
struct Machine {
    time: f64,
    press: &'static str,
    lathe: &'static str,
    oven: &'static str,
    furnace: f64,
    coolant: f64,
}

fn state(seed: u64) -> &'static str {
    match noise(seed) {
        roll if roll < 0.75 => "Running",
        roll if roll < 0.9 => "Idle",
        _ => "Down",
    }
}

fn shift() -> Vec<Machine> {
    (0..48u64)
        .map(|index| {
            let t = index as f64;
            Machine {
                time: 1_729_000_000_000. + t * 900_000.,
                press: state(index / 3 * 11),
                lathe: state(index / 3 * 13 + 1),
                oven: state(index / 3 * 17 + 2),
                furnace: 850. + 25. * (t / 7.).sin() + 6. * (noise(index + 60) - 0.5),
                coolant: 22. + 3. * (t / 11.).cos() + (noise(index + 90) - 0.5),
            }
        })
        .collect()
}

#[derive(Clone, PartialEq)]
struct Vibration {
    time: f64,
    value: f64,
}

fn vibration() -> Vec<Vibration> {
    (0..1500u64)
        .map(|index| {
            let t = index as f64 / 10_000.;
            Vibration {
                time: t,
                value: (std::f64::consts::TAU * 180. * t).sin()
                    + 0.4 * (std::f64::consts::TAU * 540. * t).sin()
                    + 0.1 * (noise(index) - 0.5),
            }
        })
        .collect()
}

#[derive(Clone, PartialEq)]
struct Output {
    name: &'static str,
    value: f64,
}

static STATES: std::sync::LazyLock<ValueMappings> = std::sync::LazyLock::new(|| {
    ValueMappings::new()
        .value("Running", "Running", rgb(0x40a02b))
        .value("Idle", "Idle", rgb(0xdf8e1d))
        .value("Down", "Down", rgb(0xd20f39))
});

#[component]
fn factory() {
    let machines = use_signal(shift());
    let parts = use_signal(samples(60, 7, 0.3));
    let causes = use_signal(defects());
    let flows = use_signal(energy());
    let scope = use_signal(vibration());
    let output = use_signal(vec![
        Output {
            name: "Good",
            value: 86.,
        },
        Output {
            name: "Rework",
            value: 9.,
        },
        Output {
            name: "Scrap",
            value: 5.,
        },
    ]);
    let oee = use_signal(vec![Output {
        name: "OEE",
        value: 78.,
    }]);
    let notes = vec![
        Annotation::at(1_729_000_000_000. + 12. * 900_000., "Shift change"),
        Annotation::region(
            1_729_000_000_000. + 30. * 900_000.,
            1_729_000_000_000. + 34. * 900_000.,
            "Maintenance",
        )
        .color(rgb(0xdf8e1d)),
    ];
    rsx! {
        <div class="flex flex-col size-full">
            <dash_header title="Factory floor" note="Machine states, process control, quality and energy. Space steps into a panel; Ctrl+scroll or pinch zooms a chart." />
            <div class="relative flex-1 min-h-0">
                <Workspace theme={mode()} no_floating={false} keep_pushed={false} keymap={keys()} gesture_keys={gestures()}>
                    <Row sizes={[3.0, 2.0]}>
                        <Column sizes={[1.0, 1.0, 1.0]}>
                            <Panel id="machines">
                                <View id="states" title="Machine states" icon={IconName::Console} gestures={GestureOwner::Exclusive}>
                                    <StatusHistory data={machines} x={|m: &Machine| m.time} mappings={&*STATES} show_value={VisibilityMode::Never} x_axis={AxisConfig::new().time()} class="size-full p-2">
                                        <StatusRow name="Press" value={|m: &Machine| m.press} />
                                        <StatusRow name="Lathe" value={|m: &Machine| m.lathe} />
                                        <StatusRow name="Oven" value={|m: &Machine| m.oven} />
                                    </StatusHistory>
                                </View>
                            </Panel>
                            <Panel id="temperatures">
                                <View id="furnace" title="Furnace" icon={IconName::Warning} gestures={GestureOwner::Exclusive}>
                                    <TimeSeries data={machines} x={|m: &Machine| m.time} annotations={notes} class="size-full p-2">
                                        <TrendSeries name="Furnace °C" y={|m: &Machine| m.furnace} />
                                        <TrendSeries name="Coolant °C" y={|m: &Machine| m.coolant} y_axis={AxisConfig::new().label("Coolant").placement(AxisPlacement::Right)} />
                                    </TimeSeries>
                                </View>
                            </Panel>
                            <Panel id="control">
                                <View id="spc" title="Shaft diameter" icon={IconName::Text} gestures={GestureOwner::Exclusive}>
                                    <ControlChart data={parts} x={|s: &Sample| s.time} value={|s: &Sample| s.diameter} class="size-full p-2" />
                                </View>
                                <View id="scope" title="Vibration" icon={IconName::Console} gestures={GestureOwner::Exclusive}>
                                    <Oscilloscope data={scope} x={|v: &Vibration| v.time} trigger={0.5} span={0.03} class="size-full p-2">
                                        <TrendSeries name="Spindle" y={|v: &Vibration| v.value} />
                                    </Oscilloscope>
                                </View>
                            </Panel>
                        </Column>
                        <Column sizes={[0.8, 1.0, 1.2]}>
                            <Panel id="kpis">
                                <View id="oee" title="OEE" icon={IconName::Info} gestures={GestureOwner::Exclusive}>
                                    <Gauge data={oee} value={|o: &Output| o.value} name={|o: &Output| o.name} class="size-full" />
                                </View>
                                <View id="yield" title="Yield" icon={IconName::Grid} gestures={GestureOwner::Exclusive}>
                                    <Waffle data={output} name={|o: &Output| o.name} value={|o: &Output| o.value} class="size-full p-2" />
                                </View>
                            </Panel>
                            <Panel id="defects">
                                <View id="pareto" title="Defects" icon={IconName::Grid} gestures={GestureOwner::Exclusive}>
                                    <Pareto data={causes} name={|d: &Defect| d.cause} value={|d: &Defect| d.count} class="size-full p-2" />
                                </View>
                            </Panel>
                            <Panel id="energy">
                                <View id="sankey" title="Energy flow" icon={IconName::Info} gestures={GestureOwner::Exclusive}>
                                    <Sankey data={flows} source={|f: &Flow| f.from} target={|f: &Flow| f.to} value={|f: &Flow| f.amount} class="size-full p-2" />
                                </View>
                            </Panel>
                        </Column>
                    </Row>
                </Workspace>
            </div>
        </div>
    }
}

pub const TRADING: &[Example] = &[Example {
    title: "Trading desk",
    description: "Prices, P&L and risk.",
    render: || rsx! { <trading /> }.into_any_element(),
}];

pub const FACTORY: &[Example] = &[Example {
    title: "Factory floor",
    description: "Machines, quality and energy.",
    render: || rsx! { <factory /> }.into_any_element(),
}];

pub fn keys() -> Keymap {
    Keymap::from([(Command::FrameToggle, Some("Space".into()))])
}

pub fn gestures() -> GestureKeys {
    GestureKeys {
        scale: Vec::new(),
        ..default_gesture_keys()
    }
}
