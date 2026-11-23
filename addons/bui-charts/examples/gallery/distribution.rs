use blenderui::prelude::*;
use bui_charts::*;

use crate::finance::noise;

pub fn normal(seed: u64) -> f64 {
    let (a, b) = (noise(seed * 2 + 1).max(1e-9), noise(seed * 2 + 2));
    (-2. * a.ln()).sqrt() * (std::f64::consts::TAU * b).cos()
}

#[derive(Clone, PartialEq)]
pub struct Measure {
    pub group: &'static str,
    pub value: f64,
}

pub fn measures(groups: &[(&'static str, f64, f64)], each: u64) -> Vec<Measure> {
    groups
        .iter()
        .enumerate()
        .flat_map(|(slot, (group, center, spread))| {
            (0..each).map(move |index| Measure {
                group,
                value: center + spread * normal(slot as u64 * 10_000 + index),
            })
        })
        .collect()
}

pub mod barcode {
    use super::*;

    #[component]
    fn single() {
        let data = use_signal(measures(&[("", 50., 12.)], 120));
        rsx! { <Barcode data={data} value={|m: &Measure| m.value} class="h-24" /> }
    }

    #[component]
    fn groups() {
        let data = use_signal(measures(
            &[
                ("Plant A", 40., 6.),
                ("Plant B", 52., 10.),
                ("Plant C", 47., 3.),
            ],
            80,
        ));
        rsx! { <Barcode data={data} value={|m: &Measure| m.value} group={|m: &Measure| m.group} class="h-56" /> }
    }

    #[component]
    fn thick() {
        let data = use_signal(measures(&[("Before", 12., 2.), ("After", 9., 1.5)], 60));
        rsx! { <Barcode data={data} value={|m: &Measure| m.value} group={|m: &Measure| m.group} tick_width={3.} color={rgb(0x8839ef)} class="h-40" /> }
    }

    #[component]
    fn weekdays() {
        let data = use_signal(measures(
            &[("Weekday", 35., 8.), ("Weekend", 50., 15.)],
            200,
        ));
        rsx! { <Barcode data={data} value={|m: &Measure| m.value} group={|m: &Measure| m.group} tick_width={1.} class="h-40" /> }
    }

    #[component]
    fn latency() {
        let data = use_signal(measures(&[("", 180., 40.)], 300));
        rsx! { <Barcode data={data} value={|m: &Measure| m.value} color={rgb(0xfe640b)} x_axis={AxisConfig::new().label("Latency (ms)")} class="h-28" /> }
    }

    #[component]
    fn regions() {
        let data = use_signal(measures(
            &[
                ("North", 62., 9.),
                ("South", 71., 6.),
                ("East", 55., 12.),
                ("West", 68., 8.),
                ("Central", 60., 4.),
            ],
            50,
        ));
        rsx! { <Barcode data={data} value={|m: &Measure| m.value} group={|m: &Measure| m.group} class="h-64" /> }
    }

    #[component]
    fn quiet() {
        let data = use_signal(measures(&[("Day", 22., 3.), ("Night", 17., 4.)], 90));
        rsx! { <Barcode data={data} value={|m: &Measure| m.value} group={|m: &Measure| m.group} tooltip={Tooltip::hidden()} class="h-40" /> }
    }

    #[component]
    fn sparse() {
        let data = use_signal(measures(&[("Pilot", 74., 10.)], 15));
        rsx! { <Barcode data={data} value={|m: &Measure| m.value} tick_width={6.} color={rgb(0xdf8e1d)} class="h-24" /> }
    }

    #[component]
    fn scores() {
        let data = use_signal(measures(
            &[
                ("Maths", 64., 14.),
                ("Physics", 58., 12.),
                ("Art", 75., 9.),
                ("History", 69., 11.),
            ],
            70,
        ));
        rsx! { <Barcode data={data} value={|m: &Measure| m.value} group={|m: &Measure| m.group} x_axis={AxisConfig::new().label("Score")} class="h-56" /> }
    }

    #[component]
    fn dense() {
        let data = use_signal(measures(&[("", 0., 1.)], 600));
        rsx! { <Barcode data={data} value={|m: &Measure| m.value} tick_width={1.} color={rgb(0x40a02b)} class="h-32" /> }
    }

    examples! {
        single: "One distribution" => "One tick per value.",
        groups: "Per plant" => "One row of ticks per group.",
        thick: "Thicker ticks" => "Wider ticks in one colour.",
        weekdays: "Hairline ticks" => "Two hundred values per group drawn with 1px ticks.",
        latency: "Labelled axis" => "Request latencies in orange with an axis label.",
        regions: "Five regions" => "Five rows of ticks, one per sales region.",
        quiet: "No tooltip" => "Day and night readings with the tooltip hidden.",
        sparse: "Few values" => "Fifteen wide ticks show a small sample.",
        scores: "Exam scores" => "Four subjects compared on one labelled axis.",
        dense: "Six hundred values" => "A dense standard-normal sample in green.",
    }
}

pub mod beeswarm {
    use super::*;

    #[component]
    fn groups() {
        let data = use_signal(measures(
            &[
                ("Control", 50., 8.),
                ("Dose 1", 56., 9.),
                ("Dose 2", 63., 7.),
            ],
            70,
        ));
        rsx! { <Beeswarm data={data} value={|m: &Measure| m.value} group={|m: &Measure| m.group} class="h-72" /> }
    }

    #[component]
    fn single() {
        let data = use_signal(measures(&[("", 120., 25.)], 150));
        rsx! { <Beeswarm data={data} value={|m: &Measure| m.value} point_size={4.} x_axis={AxisConfig::new().label("Response time (ms)")} class="h-56" /> }
    }

    #[component]
    fn platforms() {
        let data = use_signal(measures(&[("iOS", 4.2, 0.4), ("Android", 3.9, 0.6)], 60));
        rsx! { <Beeswarm data={data} value={|m: &Measure| m.value} group={|m: &Measure| m.group} legend={Legend::list()} class="h-56" /> }
    }

    fn salaries() -> Vec<Measure> {
        measures(
            &[
                ("Design", 82., 14.),
                ("Engineering", 105., 20.),
                ("Sales", 76., 18.),
                ("Support", 58., 9.),
            ],
            45,
        )
    }

    #[component]
    fn teams() {
        let data = use_signal(salaries());
        rsx! { <Beeswarm data={data} value={|m: &Measure| m.value} group={|m: &Measure| m.group} legend={Legend::list().right()} class="h-72" /> }
    }

    #[component]
    fn no_legend() {
        let data = use_signal(salaries());
        rsx! { <Beeswarm data={data} value={|m: &Measure| m.value} group={|m: &Measure| m.group} legend={Legend::hidden()} x_axis={AxisConfig::new().label("Salary (k$)")} class="h-72" /> }
    }

    #[component]
    fn big_dots() {
        let data = use_signal(measures(
            &[("Cohort 1", 30., 6.), ("Cohort 2", 38., 5.)],
            25,
        ));
        rsx! { <Beeswarm data={data} value={|m: &Measure| m.value} group={|m: &Measure| m.group} point_size={7.} class="h-56" /> }
    }

    #[component]
    fn fine_dots() {
        let data = use_signal(measures(&[("", 500., 90.)], 400));
        rsx! { <Beeswarm data={data} value={|m: &Measure| m.value} point_size={2.} class="h-48" /> }
    }

    #[component]
    fn quiet() {
        let data = use_signal(measures(&[("Before", 8.2, 1.1), ("After", 6.9, 0.9)], 80));
        rsx! { <Beeswarm data={data} value={|m: &Measure| m.value} group={|m: &Measure| m.group} tooltip={Tooltip::hidden()} class="h-56" /> }
    }

    #[component]
    fn heights() {
        let data = use_signal(measures(&[("Women", 165., 7.), ("Men", 178., 8.)], 100));
        rsx! { <Beeswarm data={data} value={|m: &Measure| m.value} group={|m: &Measure| m.group} x_axis={AxisConfig::new().label("Height (cm)")} legend={Legend::list().right()} class="h-64" /> }
    }

    #[component]
    fn seasons() {
        let data = use_signal(measures(
            &[
                ("Winter", 4., 3.),
                ("Spring", 13., 4.),
                ("Summer", 24., 3.),
                ("Autumn", 14., 4.),
                ("Monsoon", 21., 2.),
            ],
            40,
        ));
        rsx! { <Beeswarm data={data} value={|m: &Measure| m.value} group={|m: &Measure| m.group} point_size={3.} class="h-96" /> }
    }

    examples! {
        groups: "Treatment groups" => "Every value as a dot, pushed apart so none overlap.",
        single: "One group" => "Response times without groups.",
        platforms: "With legend" => "App ratings per platform.",
        teams: "Legend on the right" => "Salaries by team with a list legend beside the plot.",
        no_legend: "Hidden legend" => "The same salaries without a legend and with an axis label.",
        big_dots: "Large dots" => "Twenty-five large dots per cohort.",
        fine_dots: "Tiny dots" => "Four hundred 2px dots in one swarm.",
        quiet: "No tooltip" => "Before and after readings with the tooltip hidden.",
        heights: "Two populations" => "Heights in centimetres with a right-hand legend.",
        seasons: "Five seasons" => "Daily temperatures in five seasons, drawn with small dots.",
    }
}

pub mod ridgeline {
    use super::*;

    fn months() -> Vec<Measure> {
        let names = [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ];
        let groups: Vec<(&'static str, f64, f64)> = names
            .iter()
            .enumerate()
            .map(|(index, name)| {
                let season = (index as f64 / 12. * std::f64::consts::TAU - 0.5).cos();
                (*name, 12. - 10. * season, 3. + index as f64 % 3.)
            })
            .collect();
        measures(&groups, 120)
    }

    #[component]
    fn temperatures() {
        let data = use_signal(months());
        rsx! { <Ridgeline data={data} value={|m: &Measure| m.value} group={|m: &Measure| m.group} x_axis={AxisConfig::new().label("°C")} class="h-96" /> }
    }

    #[component]
    fn tight() {
        let data = use_signal(months());
        rsx! { <Ridgeline data={data} value={|m: &Measure| m.value} group={|m: &Measure| m.group} overlap={0.9} fill_opacity={0.9} class="h-96" /> }
    }

    #[component]
    fn smooth() {
        let data = use_signal(measures(
            &[("Small", 20., 4.), ("Medium", 35., 6.), ("Large", 55., 10.)],
            150,
        ));
        rsx! { <Ridgeline data={data} value={|m: &Measure| m.value} group={|m: &Measure| m.group} bandwidth={5.} class="h-72" /> }
    }

    #[component]
    fn faint() {
        let data = use_signal(months());
        rsx! { <Ridgeline data={data} value={|m: &Measure| m.value} group={|m: &Measure| m.group} fill_opacity={0.4} class="h-96" /> }
    }

    #[component]
    fn spaced() {
        let data = use_signal(months());
        rsx! { <Ridgeline data={data} value={|m: &Measure| m.value} group={|m: &Measure| m.group} overlap={0.2} class="h-96" /> }
    }

    #[component]
    fn exams() {
        let data = use_signal(measures(
            &[
                ("Class A", 62., 12.),
                ("Class B", 70., 9.),
                ("Class C", 55., 15.),
                ("Class D", 78., 7.),
            ],
            100,
        ));
        rsx! { <Ridgeline data={data} value={|m: &Measure| m.value} group={|m: &Measure| m.group} bandwidth={3.} x_axis={AxisConfig::new().label("Score")} class="h-72" /> }
    }

    #[component]
    fn quiet() {
        let data = use_signal(measures(&[("Before", 40., 8.), ("After", 33., 5.)], 200));
        rsx! { <Ridgeline data={data} value={|m: &Measure| m.value} group={|m: &Measure| m.group} tooltip={Tooltip::hidden()} class="h-48" /> }
    }

    #[component]
    fn narrow() {
        let data = use_signal(measures(&[("Morning", 8., 1.), ("Evening", 18., 2.)], 250));
        rsx! { <Ridgeline data={data} value={|m: &Measure| m.value} group={|m: &Measure| m.group} bandwidth={0.8} class="h-56" /> }
    }

    #[component]
    fn ages() {
        let data = use_signal(measures(
            &[
                ("Free", 24., 6.),
                ("Basic", 32., 8.),
                ("Pro", 38., 7.),
                ("Team", 41., 9.),
                ("Enterprise", 47., 8.),
            ],
            130,
        ));
        rsx! { <Ridgeline data={data} value={|m: &Measure| m.value} group={|m: &Measure| m.group} overlap={0.6} x_axis={AxisConfig::new().label("Age")} class="h-80" /> }
    }

    #[component]
    fn stacked() {
        let data = use_signal(measures(
            &[("Mon", 30., 6.), ("Tue", 34., 6.), ("Wed", 40., 7.)],
            180,
        ));
        rsx! { <Ridgeline data={data} value={|m: &Measure| m.value} group={|m: &Measure| m.group} overlap={1.2} fill_opacity={0.7} class="h-64" /> }
    }

    examples! {
        temperatures: "Temperatures by month" => "One density curve per month, overlapping.",
        tight: "Less overlap" => "overlap 0.9 and opaque fills.",
        smooth: "Wide bandwidth" => "A fixed 5-unit bandwidth smooths the curves.",
        faint: "Translucent fills" => "Fill opacity 0.4 lets overlapping curves show through.",
        spaced: "Separated rows" => "overlap 0.2 keeps each month in its own band.",
        exams: "Exam scores" => "Four classes with a 3-point bandwidth and a labelled axis.",
        quiet: "No tooltip" => "Before and after curves with the tooltip hidden.",
        narrow: "Narrow bandwidth" => "A 0.8 bandwidth keeps the sharp peaks of two commute hours.",
        ages: "Customer ages" => "Five plans with overlap 0.6.",
        stacked: "Heavy overlap" => "overlap 1.2 piles the curves on top of each other.",
    }
}

pub mod horizon {
    use super::*;

    #[derive(Clone, PartialEq)]
    pub struct Tick {
        pub time: f64,
        pub cpu: f64,
        pub memory: f64,
        pub latency: f64,
    }

    fn ticks() -> Vec<Tick> {
        (0..300u64)
            .map(|index| {
                let t = index as f64;
                Tick {
                    time: 1_729_000_000_000. + t * 60_000.,
                    cpu: 30. * (t / 20.).sin() + 15. * normal(index),
                    memory: 20. * (t / 45.).cos() + 8. * normal(index + 500),
                    latency: 40. * ((t / 70.).sin().powi(3)) + 6. * normal(index + 900),
                }
            })
            .collect()
    }

    #[component]
    fn servers() {
        let data = use_signal(ticks());
        rsx! {
            <Horizon data={data} x={|t: &Tick| t.time} x_axis={AxisConfig::new().time()} class="h-56">
                <TrendSeries name="CPU Δ" y={|t: &Tick| t.cpu} />
                <TrendSeries name="Memory Δ" y={|t: &Tick| t.memory} />
                <TrendSeries name="Latency Δ" y={|t: &Tick| t.latency} />
            </Horizon>
        }
    }

    #[component]
    fn more_bands() {
        let data = use_signal(ticks());
        rsx! {
            <Horizon data={data} x={|t: &Tick| t.time} bands={5} x_axis={AxisConfig::new().time()} class="h-40">
                <TrendSeries name="CPU Δ" y={|t: &Tick| t.cpu} />
                <TrendSeries name="Latency Δ" y={|t: &Tick| t.latency} />
            </Horizon>
        }
    }

    #[component]
    fn colours() {
        let data = use_signal(ticks());
        rsx! {
            <Horizon data={data} x={|t: &Tick| t.time} positive={rgb(0x1e66f5)} negative={rgb(0xfe640b)} x_axis={AxisConfig::new().time()} class="h-32">
                <TrendSeries name="Memory Δ" y={|t: &Tick| t.memory} />
            </Horizon>
        }
    }

    #[component]
    fn single() {
        let data = use_signal(ticks());
        rsx! {
            <Horizon data={data} x={|t: &Tick| t.time} x_axis={AxisConfig::new().time()} class="h-24">
                <TrendSeries name="CPU Δ" y={|t: &Tick| t.cpu} />
            </Horizon>
        }
    }

    #[component]
    fn two_bands() {
        let data = use_signal(ticks());
        rsx! {
            <Horizon data={data} x={|t: &Tick| t.time} bands={2} x_axis={AxisConfig::new().time()} class="h-40">
                <TrendSeries name="Memory Δ" y={|t: &Tick| t.memory} />
                <TrendSeries name="Latency Δ" y={|t: &Tick| t.latency} />
            </Horizon>
        }
    }

    #[component]
    fn green_purple() {
        let data = use_signal(ticks());
        rsx! {
            <Horizon data={data} x={|t: &Tick| t.time} positive={rgb(0x40a02b)} negative={rgb(0x8839ef)} x_axis={AxisConfig::new().time()} class="h-40">
                <TrendSeries name="CPU Δ" y={|t: &Tick| t.cpu} />
                <TrendSeries name="Memory Δ" y={|t: &Tick| t.memory} />
            </Horizon>
        }
    }

    #[component]
    fn fine_bands() {
        let data = use_signal(ticks());
        rsx! {
            <Horizon data={data} x={|t: &Tick| t.time} bands={8} x_axis={AxisConfig::new().time()} class="h-24">
                <TrendSeries name="Latency Δ" y={|t: &Tick| t.latency} />
            </Horizon>
        }
    }

    #[component]
    fn recent() {
        let data = use_signal(ticks().into_iter().skip(220).collect::<Vec<_>>());
        rsx! {
            <Horizon data={data} x={|t: &Tick| t.time} bands={4} x_axis={AxisConfig::new().time()} class="h-56">
                <TrendSeries name="CPU Δ" y={|t: &Tick| t.cpu} />
                <TrendSeries name="Memory Δ" y={|t: &Tick| t.memory} />
                <TrendSeries name="Latency Δ" y={|t: &Tick| t.latency} />
            </Horizon>
        }
    }

    #[component]
    fn teal_red() {
        let data = use_signal(ticks());
        rsx! {
            <Horizon data={data} x={|t: &Tick| t.time} bands={4} positive={rgb(0x179299)} negative={rgb(0xd20f39)} x_axis={AxisConfig::new().time()} class="h-32">
                <TrendSeries name="Latency Δ" y={|t: &Tick| t.latency} />
                <TrendSeries name="CPU Δ" y={|t: &Tick| t.cpu} />
            </Horizon>
        }
    }

    #[component]
    fn tall() {
        let data = use_signal(ticks());
        rsx! {
            <Horizon data={data} x={|t: &Tick| t.time} bands={6} x_axis={AxisConfig::new().time()} class="h-80">
                <TrendSeries name="CPU Δ" y={|t: &Tick| t.cpu} />
                <TrendSeries name="Memory Δ" y={|t: &Tick| t.memory} />
                <TrendSeries name="Latency Δ" y={|t: &Tick| t.latency} />
            </Horizon>
        }
    }

    examples! {
        servers: "Three metrics" => "Each row folds its values into 3 bands; darker means further from zero, red is below zero.",
        more_bands: "Five bands" => "More bands fit a wider range in the same height.",
        colours: "Custom colours" => "Blue above zero, orange below.",
        single: "One metric" => "A single compact row for CPU.",
        two_bands: "Two bands" => "Fewer bands give a lighter, simpler fold.",
        green_purple: "Green and purple" => "Green above zero, purple below, for two rows.",
        fine_bands: "Eight bands" => "Eight bands squeeze a wide range into a thin row.",
        recent: "Recent window" => "Only the last 80 minutes of the three metrics, in four bands.",
        teal_red: "Teal and red" => "Four bands with teal above zero and red below.",
        tall: "Tall rows" => "Six bands in taller rows for three metrics.",
    }
}
