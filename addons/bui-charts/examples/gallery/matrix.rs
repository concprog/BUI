use blenderui::prelude::*;
use bui_charts::*;

use crate::distribution::normal;

#[derive(Clone, PartialEq)]
pub struct Sample {
    pub time: f64,
    pub value: f64,
}

pub fn chirp(seconds: f64, rate: f64) -> Vec<Sample> {
    let count = (seconds * rate) as u64;
    (0..count)
        .map(|index| {
            let t = index as f64 / rate;
            let sweep = 50. + 350. * t / seconds;
            Sample {
                time: t,
                value: (std::f64::consts::TAU * sweep * t * 0.5).sin()
                    + 0.5 * (std::f64::consts::TAU * 120. * t).sin()
                    + 0.2 * normal(index),
            }
        })
        .collect()
}

pub mod spectrogram {
    use super::*;

    #[component]
    fn sweep() {
        let data = use_signal(chirp(2., 1000.));
        rsx! { <Spectrogram data={data} x={|s: &Sample| s.time} value={|s: &Sample| s.value} class="h-72" /> }
    }

    #[component]
    fn fine() {
        let data = use_signal(chirp(2., 1000.));
        rsx! { <Spectrogram data={data} x={|s: &Sample| s.time} value={|s: &Sample| s.value} window_size={512} overlap={0.75} scheme={HeatmapScheme::Viridis} class="h-72" /> }
    }

    #[component]
    fn coarse() {
        let data = use_signal(chirp(2., 1000.));
        rsx! { <Spectrogram data={data} x={|s: &Sample| s.time} value={|s: &Sample| s.value} window_size={64} overlap={0.} scheme={HeatmapScheme::Inferno} class="h-72" /> }
    }

    #[component]
    fn turbo() {
        let data = use_signal(chirp(2., 1000.));
        rsx! { <Spectrogram data={data} x={|s: &Sample| s.time} value={|s: &Sample| s.value} scheme={HeatmapScheme::Turbo} class="h-72" /> }
    }

    #[component]
    fn burst() {
        let data = use_signal(chirp(0.5, 1000.));
        rsx! { <Spectrogram data={data} x={|s: &Sample| s.time} value={|s: &Sample| s.value} window_size={128} scheme={HeatmapScheme::Cividis} class="h-72" /> }
    }

    #[component]
    fn audio_rate() {
        let data = use_signal(chirp(1., 4000.));
        rsx! { <Spectrogram data={data} x={|s: &Sample| s.time} value={|s: &Sample| s.value} window_size={1024} overlap={0.9} scheme={HeatmapScheme::Magma} class="h-72" /> }
    }

    #[component]
    fn recording() {
        let data = use_signal(chirp(6., 500.));
        rsx! { <Spectrogram data={data} x={|s: &Sample| s.time} value={|s: &Sample| s.value} scheme={HeatmapScheme::Greys} class="h-72" /> }
    }

    #[component]
    fn blocks() {
        let data = use_signal(chirp(2., 1000.));
        rsx! { <Spectrogram data={data} x={|s: &Sample| s.time} value={|s: &Sample| s.value} window_size={128} overlap={0.} scheme={HeatmapScheme::Blues} class="h-72" /> }
    }

    #[component]
    fn smooth() {
        let data = use_signal(chirp(2., 1000.));
        rsx! { <Spectrogram data={data} x={|s: &Sample| s.time} value={|s: &Sample| s.value} overlap={0.95} scheme={HeatmapScheme::Purples} class="h-72" /> }
    }

    #[component]
    fn resolved() {
        let data = use_signal(chirp(4., 2000.));
        rsx! { <Spectrogram data={data} x={|s: &Sample| s.time} value={|s: &Sample| s.value} window_size={2048} scheme={HeatmapScheme::Plasma} class="h-72" /> }
    }

    examples! {
        sweep: "Rising tone" => "A chirp from 50 to 400 Hz over a constant 120 Hz hum, in dB.",
        fine: "Longer windows" => "512-sample windows: sharper frequencies, blurrier time.",
        coarse: "Short windows" => "64-sample windows: sharper time, blurrier frequencies.",
        turbo: "Turbo scheme" => "The default window with a rainbow scheme that separates power levels.",
        burst: "Half a second" => "A short capture with 128-sample windows.",
        audio_rate: "4 kHz sampling" => "A higher sample rate with 1024-sample windows and 90% overlap.",
        recording: "Long recording" => "Six seconds at 500 Hz in greyscale.",
        blocks: "No overlap" => "Windows that do not repeat, so each time step shows as a block.",
        smooth: "95% overlap" => "Nearly every sample starts a new window for a smooth picture.",
        resolved: "Very long windows" => "2048-sample windows resolve the hum and the sweep as separate lines.",
    }
}

pub mod attention_map {
    use super::*;

    fn sentence() -> Vec<SharedString> {
        ["The", "cat", "sat", "on", "the", "mat", "."]
            .map(SharedString::Borrowed)
            .to_vec()
    }

    fn weights(size: usize, causal: bool) -> Vec<f64> {
        let mut out = Vec::with_capacity(size * size);
        for line in 0..size {
            let raw: Vec<f64> = (0..size)
                .map(|column| {
                    if causal && column > line {
                        return 0.;
                    }
                    let distance = (line as f64 - column as f64).abs();
                    (-distance * 0.6).exp() + if column == 1 && line >= 2 { 0.8 } else { 0. }
                })
                .collect();
            let total: f64 = raw.iter().sum();
            out.extend(raw.iter().map(|value| value / total));
        }
        out
    }

    #[component]
    fn self_attention() {
        rsx! { <AttentionMap tokens={sentence()} weights={weights(7, false)} class="h-80" /> }
    }

    #[component]
    fn causal() {
        rsx! { <AttentionMap tokens={sentence()} weights={weights(7, true)} scheme={HeatmapScheme::Purples} class="h-80" /> }
    }

    #[component]
    fn cross() {
        let queries = ["Le", "chat", "est", "assis"]
            .map(SharedString::Borrowed)
            .to_vec();
        let keys = ["The", "cat", "is", "sitting"]
            .map(SharedString::Borrowed)
            .to_vec();
        let matrix = vec![
            0.85, 0.05, 0.05, 0.05, 0.05, 0.88, 0.04, 0.03, 0.05, 0.05, 0.8, 0.1, 0.04, 0.06, 0.15,
            0.75,
        ];
        rsx! { <AttentionMap tokens={keys} queries={queries} weights={matrix} scheme={HeatmapScheme::Oranges} class="h-72" /> }
    }

    fn words(items: &[&'static str]) -> Vec<SharedString> {
        items
            .iter()
            .copied()
            .map(SharedString::Borrowed)
            .collect()
    }

    #[component]
    fn warm() {
        rsx! { <AttentionMap tokens={sentence()} weights={weights(7, false)} scheme={HeatmapScheme::Reds} class="h-80" /> }
    }

    #[component]
    fn quiet() {
        rsx! { <AttentionMap tokens={sentence()} weights={weights(7, false)} scheme={HeatmapScheme::Viridis} show_values={false} class="h-80" /> }
    }

    #[component]
    fn short() {
        rsx! { <AttentionMap tokens={words(&["Yes", "we", "can", "!"])} weights={weights(4, true)} scheme={HeatmapScheme::Greens} class="h-64" /> }
    }

    #[component]
    fn long() {
        let tokens = words(&[
            "Attention",
            "is",
            "all",
            "you",
            "need",
            "to",
            "model",
            "long",
            "range",
            "links",
            "in",
            "a",
            "sequence",
        ]);
        rsx! { <AttentionMap tokens={tokens} weights={weights(13, true)} scheme={HeatmapScheme::Cividis} class="h-96" /> }
    }

    #[component]
    fn labelled() {
        let tokens = words(&["[CLS]", "good", "movie", "[SEP]", "bad", "plot", "[SEP]"]);
        rsx! { <AttentionMap tokens={tokens} weights={weights(7, false)} scheme={HeatmapScheme::Magma} class="h-80" /> }
    }

    #[component]
    fn summary_queries() {
        let queries = words(&["[CLS]", "[SUM]"]);
        let keys = words(&["Rain", "fell", "all", "night", "and", "rivers", "rose"]);
        let matrix = vec![
            0.05, 0.1, 0.05, 0.2, 0.05, 0.25, 0.3, 0.3, 0.1, 0.05, 0.1, 0.05, 0.1, 0.3,
        ];
        rsx! { <AttentionMap tokens={keys} queries={queries} weights={matrix} scheme={HeatmapScheme::Purples} class="h-48" /> }
    }

    #[component]
    fn alignment() {
        let queries = words(&["Ich", "habe", "Hunger"]);
        let keys = words(&["I", "am", "hungry", "."]);
        let matrix = vec![
            0.9, 0.04, 0.03, 0.03, 0.1, 0.6, 0.25, 0.05, 0.02, 0.08, 0.86, 0.04,
        ];
        rsx! { <AttentionMap tokens={keys} queries={queries} weights={matrix} scheme={HeatmapScheme::Turbo} class="h-64" /> }
    }

    examples! {
        self_attention: "Self-attention" => "Each token's attention over the sentence; values shown in the cells.",
        causal: "Causal mask" => "Tokens only attend to earlier tokens.",
        cross: "Cross-attention" => "French queries attending to English keys.",
        warm: "Red scheme" => "The same self-attention in a warm sequential scheme.",
        quiet: "Without values" => "show_values off for a cleaner picture.",
        short: "Four tokens" => "A tiny causal map with large cells.",
        long: "Thirteen tokens" => "Past 12 columns the values hide by themselves.",
        labelled: "Special tokens" => "A sentence pair with classification and separator tokens.",
        summary_queries: "Two queries" => "A rectangular map: two summary queries over seven keys.",
        alignment: "German to English" => "Word alignment with a Turbo scheme.",
    }
}

pub mod waffle {
    use super::*;

    #[derive(Clone, PartialEq)]
    pub struct Share {
        pub name: &'static str,
        pub value: f64,
    }

    fn mix(items: &[(&'static str, f64)]) -> Vec<Share> {
        items
            .iter()
            .map(|(name, value)| Share {
                name,
                value: *value,
            })
            .collect()
    }

    #[component]
    fn energy() {
        let data = use_signal(mix(&[
            ("Solar", 31.),
            ("Wind", 24.),
            ("Hydro", 17.),
            ("Gas", 19.),
            ("Coal", 9.),
        ]));
        rsx! { <Waffle data={data} name={|s: &Share| s.name} value={|s: &Share| s.value} class="h-72" /> }
    }

    #[component]
    fn wide() {
        let data = use_signal(mix(&[("Yes", 62.), ("No", 31.), ("Undecided", 7.)]));
        rsx! { <Waffle data={data} name={|s: &Share| s.name} value={|s: &Share| s.value} rows={5} columns={20} class="h-40" /> }
    }

    #[component]
    fn tight() {
        let data = use_signal(mix(&[
            ("Done", 46.),
            ("In progress", 21.),
            ("Blocked", 4.),
            ("To do", 29.),
        ]));
        rsx! { <Waffle data={data} name={|s: &Share| s.name} value={|s: &Share| s.value} gap={0.04} class="h-72" /> }
    }

    #[component]
    fn budget() {
        let data = use_signal(mix(&[
            ("Housing", 34.),
            ("Food", 14.),
            ("Transport", 12.),
            ("Savings", 15.),
            ("Leisure", 9.),
            ("Health", 7.),
            ("Other", 9.),
        ]));
        rsx! { <Waffle data={data} name={|s: &Share| s.name} value={|s: &Share| s.value} legend={Legend::list().right()} class="h-72" /> }
    }

    #[component]
    fn quiet() {
        let data = use_signal(mix(&[("Mobile", 58.), ("Desktop", 37.), ("Tablet", 5.)]));
        rsx! { <Waffle data={data} name={|s: &Share| s.name} value={|s: &Share| s.value} legend={Legend::hidden()} tooltip={Tooltip::hidden()} class="h-72" /> }
    }

    #[component]
    fn parliament() {
        let data = use_signal(mix(&[
            ("Left", 41.),
            ("Green", 18.),
            ("Centre", 27.),
            ("Right", 49.),
            ("Other", 5.),
        ]));
        rsx! { <Waffle data={data} name={|s: &Share| s.name} value={|s: &Share| s.value} rows={7} columns={20} class="h-56" /> }
    }

    #[component]
    fn day() {
        let data = use_signal(mix(&[
            ("Sleep", 8.),
            ("Work", 9.),
            ("Commute", 1.5),
            ("Meals", 2.),
            ("Free time", 3.5),
        ]));
        rsx! { <Waffle data={data} name={|s: &Share| s.name} value={|s: &Share| s.value} rows={4} columns={6} gap={0.2} class="h-64" /> }
    }

    #[component]
    fn strip() {
        let data = use_signal(mix(&[("Complete", 13.), ("Remaining", 7.)]));
        rsx! { <Waffle data={data} name={|s: &Share| s.name} value={|s: &Share| s.value} rows={1} columns={20} gap={0.25} legend={Legend::hidden()} class="h-16" /> }
    }

    #[component]
    fn dense() {
        let data = use_signal(mix(&[
            ("Vaccinated", 71.),
            ("Partly", 9.),
            ("Unvaccinated", 20.),
        ]));
        rsx! { <Waffle data={data} name={|s: &Share| s.name} value={|s: &Share| s.value} rows={20} columns={20} gap={0.02} class="h-96" /> }
    }

    #[component]
    fn airy() {
        let data = use_signal(mix(&[("Pass", 83.), ("Fail", 11.), ("Skipped", 6.)]));
        rsx! { <Waffle data={data} name={|s: &Share| s.name} value={|s: &Share| s.value} gap={0.4} class="h-72" /> }
    }

    examples! {
        energy: "Energy mix" => "100 squares, one per percent, filled in order.",
        wide: "5 × 20" => "A wide grid for a survey result.",
        tight: "Small gaps" => "Task status with almost no gap.",
        budget: "Household budget" => "Seven categories with the legend on the right.",
        quiet: "No legend or tooltip" => "Device split drawn as bare squares.",
        parliament: "Seats" => "A 7 × 20 grid with 140 squares, one per seat.",
        day: "A day in hours" => "A 4 × 6 grid where each square is one hour.",
        strip: "Progress strip" => "A single row of twenty squares for 13 of 20 tests passed.",
        dense: "20 × 20" => "400 squares with a hair-thin gap for vaccination coverage.",
        airy: "Wide gaps" => "Test results with gaps of 40% of a square.",
    }
}
