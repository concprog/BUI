use blenderui::prelude::*;
use bui_charts::*;

use crate::distribution::normal;
use crate::finance::noise;

pub fn corners(names: [&'static str; 3]) -> Vec<SharedString> {
    names.map(SharedString::Borrowed).to_vec()
}

#[derive(Clone, PartialEq)]
pub struct Mix {
    pub label: SharedString,
    pub group: &'static str,
    pub parts: [f64; 3],
}

pub fn cluster(
    group: &'static str,
    centre: [f64; 3],
    spread: f64,
    count: u64,
    seed: u64,
) -> Vec<Mix> {
    (0..count)
        .map(|index| {
            let mut slot = 0;
            let parts = centre.map(|part| {
                slot += 1;
                normal(seed * 1000 + index * 3 + slot)
                    .mul_add(spread, part)
                    .max(0.01)
            });
            Mix {
                label: format!("{group} {}", index + 1).into(),
                group,
                parts,
            }
        })
        .collect()
}

pub mod ternary {
    use super::*;

    fn soils() -> Vec<Mix> {
        let mut all = cluster("Sand", [80., 12., 8.], 5., 15, 1);
        all.extend(cluster("Loam", [40., 40., 20.], 6., 15, 2));
        all.extend(cluster("Clay", [20., 25., 55.], 6., 15, 3));
        all
    }

    #[component]
    fn soil() {
        let data = use_signal(soils());
        rsx! { <Ternary data={data} a={|m: &Mix| m.parts[0]} b={|m: &Mix| m.parts[1]} c={|m: &Mix| m.parts[2]} name={|m: &Mix| m.label.as_ref()} group={|m: &Mix| m.group} corners={corners(["Sand", "Silt", "Clay"])} class="h-96" /> }
    }

    #[component]
    fn plain() {
        let data = use_signal(cluster("Sample", [33., 33., 34.], 12., 40, 4));
        rsx! { <Ternary data={data} a={|m: &Mix| m.parts[0]} b={|m: &Mix| m.parts[1]} c={|m: &Mix| m.parts[2]} class="h-96" /> }
    }

    #[component]
    fn coarse_grid() {
        let data = use_signal(soils());
        rsx! { <Ternary data={data} a={|m: &Mix| m.parts[0]} b={|m: &Mix| m.parts[1]} c={|m: &Mix| m.parts[2]} group={|m: &Mix| m.group} divisions={4} corners={corners(["Sand", "Silt", "Clay"])} class="h-96" /> }
    }

    #[component]
    fn big_points() {
        let data = use_signal(cluster("Run", [50., 30., 20.], 8., 12, 5));
        rsx! { <Ternary data={data} a={|m: &Mix| m.parts[0]} b={|m: &Mix| m.parts[1]} c={|m: &Mix| m.parts[2]} name={|m: &Mix| m.label.as_ref()} point_size={9.} class="h-96" /> }
    }

    #[component]
    fn election() {
        let mut all = cluster("North", [48., 32., 20.], 4., 12, 6);
        all.extend(cluster("South", [30., 52., 18.], 4., 12, 7));
        all.extend(cluster("Coast", [35., 30., 35.], 4., 12, 8));
        let data = use_signal(all);
        rsx! { <Ternary data={data} a={|m: &Mix| m.parts[0]} b={|m: &Mix| m.parts[1]} c={|m: &Mix| m.parts[2]} name={|m: &Mix| m.label.as_ref()} group={|m: &Mix| m.group} corners={corners(["Red", "Blue", "Green"])} class="h-96" /> }
    }

    #[component]
    fn alloys() {
        let mut all = cluster("Bronze", [88., 12., 0.5], 2., 10, 9);
        all.extend(cluster("Brass", [65., 1., 34.], 3., 10, 10));
        all.extend(cluster("Nickel silver", [60., 18., 22.], 3., 10, 11));
        let data = use_signal(all);
        rsx! { <Ternary data={data} a={|m: &Mix| m.parts[0]} b={|m: &Mix| m.parts[1]} c={|m: &Mix| m.parts[2]} group={|m: &Mix| m.group} corners={corners(["Cu", "Sn", "Zn"])} class="h-96" /> }
    }

    #[component]
    fn legend_right() {
        let data = use_signal(soils());
        rsx! { <Ternary data={data} a={|m: &Mix| m.parts[0]} b={|m: &Mix| m.parts[1]} c={|m: &Mix| m.parts[2]} group={|m: &Mix| m.group} legend={Legend::list().right()} class="h-96" /> }
    }

    #[component]
    fn diet() {
        let data = use_signal(cluster("Meal", [50., 25., 25.], 9., 30, 12));
        rsx! { <Ternary data={data} a={|m: &Mix| m.parts[0]} b={|m: &Mix| m.parts[1]} c={|m: &Mix| m.parts[2]} corners={corners(["Carbs", "Protein", "Fat"])} class="h-96" /> }
    }

    #[component]
    fn raw_counts() {
        let data = use_signal(cluster("Batch", [400., 150., 50.], 40., 20, 13));
        rsx! { <Ternary data={data} a={|m: &Mix| m.parts[0]} b={|m: &Mix| m.parts[1]} c={|m: &Mix| m.parts[2]} name={|m: &Mix| m.label.as_ref()} corners={corners(["Pass", "Rework", "Scrap"])} class="h-96" /> }
    }

    #[component]
    fn compact() {
        let data = use_signal(soils());
        rsx! { <Ternary data={data} a={|m: &Mix| m.parts[0]} b={|m: &Mix| m.parts[1]} c={|m: &Mix| m.parts[2]} group={|m: &Mix| m.group} point_size={3.} legend={Legend::hidden()} class="h-64" /> }
    }

    examples! {
        soil: "Soil texture" => "Sand, silt and clay shares of 45 samples; the grid reads in tens of percent.",
        plain: "Defaults" => "Corners A, B and C and a 10% grid.",
        coarse_grid: "Four divisions" => "A 25% grid.",
        big_points: "Larger points" => "point_size 9 and named points in the tooltip.",
        election: "Three-party vote" => "Vote shares by district, grouped by region.",
        alloys: "Copper alloys" => "Cu, Sn and Zn fractions.",
        legend_right: "Legend on the right" => "Click a group to hide it.",
        diet: "Meal macros" => "Carbohydrate, protein and fat.",
        raw_counts: "Raw counts" => "Values need not add up to 100; each row is normalised.",
        compact: "Compact" => "Small points and no legend.",
    }
}

pub mod ternary_contour {
    use super::*;

    #[component]
    fn peak() {
        rsx! { <TernaryContour field={|a: f64, b: f64, c: f64| (-((a - 0.5).powi(2) + (b - 0.3).powi(2) + (c - 0.2).powi(2)) * 12.).exp()} class="h-96" /> }
    }

    #[component]
    fn mixing_energy() {
        rsx! { <TernaryContour field={|a: f64, b: f64, c: f64| [a, b, c].iter().map(|x| if *x > 0. { x * x.ln() } else { 0. }).sum::<f64>() + 1.5 * a * b} corners={corners(["Fe", "Ni", "Cr"])} scheme={HeatmapScheme::Plasma} class="h-96" /> }
    }

    #[component]
    fn mesh_lines() {
        rsx! { <TernaryContour field={|a: f64, b: f64, _: f64| a * b} resolution={12} mesh={true} class="h-96" /> }
    }

    #[component]
    fn fine() {
        rsx! { <TernaryContour field={|a: f64, b: f64, c: f64| (a * 9.).sin() + (b * 7.).cos() + c} resolution={80} levels={12} scheme={HeatmapScheme::Turbo} class="h-96" /> }
    }

    #[component]
    fn no_lines() {
        rsx! { <TernaryContour field={|a: f64, _: f64, c: f64| a - c} levels={0} scheme={HeatmapScheme::RdBu} class="h-96" /> }
    }

    #[component]
    fn strength() {
        rsx! { <TernaryContour field={|a: f64, b: f64, c: f64| 30. * a + 45. * b + 20. * c + 60. * a * b - 25. * b * c} corners={corners(["Cement", "Slag", "Fly ash"])} scheme={HeatmapScheme::Greens} class="h-96" /> }
    }

    #[component]
    fn ridge() {
        rsx! { <TernaryContour field={|a: f64, b: f64, _: f64| (-(a - b).powi(2) * 30.).exp()} scheme={HeatmapScheme::Inferno} levels={5} class="h-96" /> }
    }

    #[component]
    fn two_peaks() {
        rsx! { <TernaryContour field={|a: f64, b: f64, c: f64| (-((a - 0.7).powi(2) + (c - 0.1).powi(2)) * 25.).exp() + (-((b - 0.6).powi(2) + (c - 0.3).powi(2)) * 25.).exp()} scheme={HeatmapScheme::Cividis} class="h-96" /> }
    }

    #[component]
    fn coarse() {
        rsx! { <TernaryContour field={|a: f64, b: f64, c: f64| a * b * c} resolution={8} mesh={true} levels={3} scheme={HeatmapScheme::Oranges} class="h-96" /> }
    }

    #[component]
    fn melting() {
        rsx! { <TernaryContour field={|a: f64, b: f64, c: f64| 1400. * a + 1100. * b + 900. * c - 600. * a * b - 400. * b * c - 300. * a * c} corners={corners(["Fo", "An", "Di"])} scheme={HeatmapScheme::YlOrRd} class="h-96" /> }
    }

    examples! {
        peak: "A single peak" => "A field over the triangle with eight contour levels; hover a cell for its value.",
        mixing_energy: "Mixing energy" => "An entropy-like surface over a three-metal alloy.",
        mesh_lines: "Visible mesh" => "A 12-step mesh with its triangle edges drawn.",
        fine: "Fine mesh" => "80 steps and 12 levels.",
        no_lines: "No contours" => "levels 0 keeps only the colour mesh.",
        strength: "Concrete strength" => "A response surface for three binders.",
        ridge: "A ridge" => "Highest where A equals B.",
        two_peaks: "Two peaks" => "Two optima.",
        coarse: "Coarse" => "Eight steps and three levels.",
        melting: "Melting temperature" => "A liquidus-like surface.",
    }
}

#[derive(Clone, PartialEq)]
pub struct Water {
    pub site: SharedString,
    pub aquifer: &'static str,
    pub ions: [f64; 6],
}

pub fn waters(aquifer: &'static str, centre: [f64; 6], count: u64, seed: u64) -> Vec<Water> {
    (0..count)
        .map(|index| Water {
            site: format!("{aquifer} {}", index + 1).into(),
            aquifer,
            ions: centre.map(|ion| {
                ion * (1. + normal(seed * 977 + index * 7 + (ion * 3.) as u64) * 0.18).max(0.05)
            }),
        })
        .collect()
}

pub mod piper {
    use super::*;

    fn basin() -> Vec<Water> {
        let mut all = waters("Limestone", [4.5, 1.2, 0.6, 5.2, 0.6, 0.5], 14, 1);
        all.extend(waters("Coastal", [1.2, 1.4, 9.5, 1.5, 1.6, 9.0], 14, 2));
        all.extend(waters("Gypsum", [6.0, 1.8, 0.8, 1.8, 6.5, 0.6], 14, 3));
        all
    }

    #[component]
    fn aquifers() {
        let data = use_signal(basin());
        rsx! { <Piper data={data} calcium={|w: &Water| w.ions[0]} magnesium={|w: &Water| w.ions[1]} sodium={|w: &Water| w.ions[2]} bicarbonate={|w: &Water| w.ions[3]} sulfate={|w: &Water| w.ions[4]} chloride={|w: &Water| w.ions[5]} name={|w: &Water| w.site.as_ref()} group={|w: &Water| w.aquifer} class="h-[480px]" /> }
    }

    #[component]
    fn single() {
        let data = use_signal(waters("Well", [3., 1., 2., 4., 1., 1.], 20, 4));
        rsx! { <Piper data={data} calcium={|w: &Water| w.ions[0]} magnesium={|w: &Water| w.ions[1]} sodium={|w: &Water| w.ions[2]} bicarbonate={|w: &Water| w.ions[3]} sulfate={|w: &Water| w.ions[4]} chloride={|w: &Water| w.ions[5]} class="h-[480px]" /> }
    }

    #[component]
    fn seawater_mixing() {
        let data = use_signal(
            (0..=10u64)
                .map(|step| {
                    let t = step as f64 / 10.;
                    let fresh: [f64; 6] = [4.5, 1.2, 0.6, 5.2, 0.6, 0.5];
                    let sea: [f64; 6] = [10., 53., 470., 2.3, 56., 545.];
                    Water {
                        site: format!("{}% sea", step * 10).into(),
                        aquifer: "Mix",
                        ions: [0, 1, 2, 3, 4, 5]
                            .map(|ion| (sea[ion] - fresh[ion]).mul_add(t * t * 0.1, fresh[ion])),
                    }
                })
                .collect::<Vec<_>>(),
        );
        rsx! { <Piper data={data} calcium={|w: &Water| w.ions[0]} magnesium={|w: &Water| w.ions[1]} sodium={|w: &Water| w.ions[2]} bicarbonate={|w: &Water| w.ions[3]} sulfate={|w: &Water| w.ions[4]} chloride={|w: &Water| w.ions[5]} name={|w: &Water| w.site.as_ref()} point_size={7.} class="h-[480px]" /> }
    }

    #[component]
    fn legend_right() {
        let data = use_signal(basin());
        rsx! { <Piper data={data} calcium={|w: &Water| w.ions[0]} magnesium={|w: &Water| w.ions[1]} sodium={|w: &Water| w.ions[2]} bicarbonate={|w: &Water| w.ions[3]} sulfate={|w: &Water| w.ions[4]} chloride={|w: &Water| w.ions[5]} group={|w: &Water| w.aquifer} legend={Legend::list().right()} class="h-[480px]" /> }
    }

    #[component]
    fn small_points() {
        let data = use_signal(basin());
        rsx! { <Piper data={data} calcium={|w: &Water| w.ions[0]} magnesium={|w: &Water| w.ions[1]} sodium={|w: &Water| w.ions[2]} bicarbonate={|w: &Water| w.ions[3]} sulfate={|w: &Water| w.ions[4]} chloride={|w: &Water| w.ions[5]} group={|w: &Water| w.aquifer} point_size={3.} class="h-[480px]" /> }
    }

    #[component]
    fn magnesium_rich() {
        let data = use_signal(waters("Dolomite", [2.2, 3.6, 0.5, 5.5, 0.5, 0.4], 18, 5));
        rsx! { <Piper data={data} calcium={|w: &Water| w.ions[0]} magnesium={|w: &Water| w.ions[1]} sodium={|w: &Water| w.ions[2]} bicarbonate={|w: &Water| w.ions[3]} sulfate={|w: &Water| w.ions[4]} chloride={|w: &Water| w.ions[5]} class="h-[480px]" /> }
    }

    #[component]
    fn brines() {
        let data = use_signal(waters("Brine", [20., 8., 120., 1., 10., 150.], 16, 6));
        rsx! { <Piper data={data} calcium={|w: &Water| w.ions[0]} magnesium={|w: &Water| w.ions[1]} sodium={|w: &Water| w.ions[2]} bicarbonate={|w: &Water| w.ions[3]} sulfate={|w: &Water| w.ions[4]} chloride={|w: &Water| w.ions[5]} class="h-[480px]" /> }
    }

    #[component]
    fn seasons() {
        let mut all = waters("Wet season", [3.5, 1.0, 1.0, 4.0, 0.8, 0.7], 12, 7);
        all.extend(waters("Dry season", [4.5, 1.6, 2.4, 4.8, 1.8, 2.2], 12, 8));
        let data = use_signal(all);
        rsx! { <Piper data={data} calcium={|w: &Water| w.ions[0]} magnesium={|w: &Water| w.ions[1]} sodium={|w: &Water| w.ions[2]} bicarbonate={|w: &Water| w.ions[3]} sulfate={|w: &Water| w.ions[4]} chloride={|w: &Water| w.ions[5]} group={|w: &Water| w.aquifer} class="h-[480px]" /> }
    }

    #[component]
    fn few() {
        let data = use_signal(vec![
            Water {
                site: "Spring A".into(),
                aquifer: "A",
                ions: [5., 1., 0.5, 5.5, 0.6, 0.4],
            },
            Water {
                site: "Well B".into(),
                aquifer: "B",
                ions: [1., 0.8, 8., 2., 1., 7.],
            },
            Water {
                site: "Mine C".into(),
                aquifer: "C",
                ions: [7., 2., 1., 1.5, 8., 0.5],
            },
        ]);
        rsx! { <Piper data={data} calcium={|w: &Water| w.ions[0]} magnesium={|w: &Water| w.ions[1]} sodium={|w: &Water| w.ions[2]} bicarbonate={|w: &Water| w.ions[3]} sulfate={|w: &Water| w.ions[4]} chloride={|w: &Water| w.ions[5]} name={|w: &Water| w.site.as_ref()} group={|w: &Water| w.aquifer} point_size={8.} class="h-[480px]" /> }
    }

    #[component]
    fn compact() {
        let data = use_signal(basin());
        rsx! { <Piper data={data} calcium={|w: &Water| w.ions[0]} magnesium={|w: &Water| w.ions[1]} sodium={|w: &Water| w.ions[2]} bicarbonate={|w: &Water| w.ions[3]} sulfate={|w: &Water| w.ions[4]} chloride={|w: &Water| w.ions[5]} group={|w: &Water| w.aquifer} legend={Legend::hidden()} class="h-80" /> }
    }

    examples! {
        aquifers: "Three aquifers" => "Cations bottom left, anions bottom right and both projected into the diamond; hover a point for its ion shares.",
        single: "One well field" => "A single group.",
        seawater_mixing: "Seawater intrusion" => "Fresh water turning into seawater moves across every panel.",
        legend_right: "Legend on the right" => "Click an aquifer to hide it.",
        small_points: "Small points" => "point_size 3.",
        magnesium_rich: "Dolomite water" => "Magnesium-rich bicarbonate water.",
        brines: "Brines" => "Sodium chloride waters at the right of the diamond.",
        seasons: "Wet and dry seasons" => "The same wells in two seasons.",
        few: "Three samples" => "One of each water type.",
        compact: "Compact" => "A smaller diagram without a legend.",
    }
}

#[derive(Clone, PartialEq)]
pub struct Level {
    pub pressure: f64,
    pub temperature: f64,
    pub dewpoint: f64,
}

pub fn sounding(
    surface: f64,
    lapse: f64,
    dryness: f64,
    inversion: Option<(f64, f64)>,
    seed: u64,
) -> Vec<Level> {
    (0..=36u64)
        .map(|step| {
            let pressure = 1000. - step as f64 * 25.;
            let height = 44.33 * (1. - (pressure / 1013.25).powf(0.19));
            let mut temperature = surface - lapse * height.min(11.);
            if let Some((top, warmth)) = inversion
                && pressure < 1000. - 1.
                && pressure > top
            {
                temperature += warmth * (1000. - pressure) / (1000. - top);
            }
            let depression = dryness * (0.5 + noise(seed * 37 + step)) + height * 1.2;
            Level {
                pressure,
                temperature: temperature + normal(seed * 91 + step) * 0.3,
                dewpoint: temperature - depression,
            }
        })
        .filter(|level| level.pressure >= 100.)
        .collect()
}

macro_rules! thermo_examples {
    ($tag:ident) => {
        #[component]
        fn midlatitude() {
            let data = use_signal(sounding(18., 6.5, 3., None, 1));
            rsx! { <$tag data={data} pressure={|l: &Level| l.pressure} temperature={|l: &Level| l.temperature} dewpoint={|l: &Level| l.dewpoint} class="h-[480px]" /> }
        }

        #[component]
        fn tropical() {
            let data = use_signal(sounding(30., 5.8, 1., None, 2));
            rsx! { <$tag data={data} pressure={|l: &Level| l.pressure} temperature={|l: &Level| l.temperature} dewpoint={|l: &Level| l.dewpoint} class="h-[480px]" /> }
        }

        #[component]
        fn inversion() {
            let data = use_signal(sounding(8., 6.5, 4., Some((850., 9.)), 3));
            rsx! { <$tag data={data} pressure={|l: &Level| l.pressure} temperature={|l: &Level| l.temperature} dewpoint={|l: &Level| l.dewpoint} class="h-[480px]" /> }
        }

        #[component]
        fn desert() {
            let data = use_signal(sounding(38., 9., 25., None, 4));
            rsx! { <$tag data={data} pressure={|l: &Level| l.pressure} temperature={|l: &Level| l.temperature} dewpoint={|l: &Level| l.dewpoint} temperature_range={(-50., 50.)} class="h-[480px]" /> }
        }

        #[component]
        fn temperature_only() {
            let data = use_signal(sounding(15., 6.5, 3., None, 5));
            rsx! { <$tag data={data} pressure={|l: &Level| l.pressure} temperature={|l: &Level| l.temperature} class="h-[480px]" /> }
        }

        #[component]
        fn lower_air() {
            let data = use_signal(sounding(20., 7.5, 2., None, 6));
            rsx! { <$tag data={data} pressure={|l: &Level| l.pressure} temperature={|l: &Level| l.temperature} dewpoint={|l: &Level| l.dewpoint} pressure_range={(1050., 500.)} temperature_range={(-20., 35.)} class="h-[480px]" /> }
        }

        #[component]
        fn bare() {
            let data = use_signal(sounding(18., 6.5, 3., None, 7));
            rsx! { <$tag data={data} pressure={|l: &Level| l.pressure} temperature={|l: &Level| l.temperature} dewpoint={|l: &Level| l.dewpoint} adiabats={false} moist={false} mixing={false} class="h-[480px]" /> }
        }

        #[component]
        fn dry_lines_only() {
            let data = use_signal(sounding(25., 9.8, 15., None, 8));
            rsx! { <$tag data={data} pressure={|l: &Level| l.pressure} temperature={|l: &Level| l.temperature} dewpoint={|l: &Level| l.dewpoint} moist={false} mixing={false} class="h-[480px]" /> }
        }

        #[component]
        fn winter() {
            let data = use_signal(sounding(-12., 5., 2., Some((900., 6.)), 9));
            rsx! { <$tag data={data} pressure={|l: &Level| l.pressure} temperature={|l: &Level| l.temperature} dewpoint={|l: &Level| l.dewpoint} temperature_range={(-60., 20.)} class="h-[480px]" /> }
        }

        #[component]
        fn saturated() {
            let data = use_signal(sounding(22., 5.5, 0.3, None, 10));
            rsx! { <$tag data={data} pressure={|l: &Level| l.pressure} temperature={|l: &Level| l.temperature} dewpoint={|l: &Level| l.dewpoint} class="h-96" /> }
        }

        examples! {
            midlatitude: "Mid-latitude" => "Temperature in red and dew point in green over isobars, isotherms, dry and moist adiabats and mixing-ratio lines. Hover a level.",
            tropical: "Tropical" => "A warm, humid column.",
            inversion: "Low inversion" => "Warming up to 850 hPa traps the boundary layer.",
            desert: "Desert" => "A hot surface with a dry, steep lapse rate.",
            temperature_only: "No dew point" => "Only the temperature line.",
            lower_air: "Lower troposphere" => "pressure_range 1050–500 hPa.",
            bare: "Grid only" => "Adiabats and mixing-ratio lines turned off.",
            dry_lines_only: "Dry adiabats only" => "Moist adiabats and mixing ratio off.",
            winter: "Winter" => "A cold column with an inversion.",
            saturated: "Saturated" => "Dew point close to temperature all the way up.",
        }
    };
}

pub mod stuve {
    use super::*;
    thermo_examples!(Stuve);
}

pub mod tephigram {
    use super::*;
    thermo_examples!(Tephigram);
}

#[derive(Clone, PartialEq)]
pub struct Stat {
    pub region: SharedString,
    pub value: f64,
}

const STATES: [(&str, i32, i32); 50] = [
    ("AK", 0, 0),
    ("ME", 11, 0),
    ("VT", 10, 1),
    ("NH", 11, 1),
    ("WA", 1, 2),
    ("ID", 2, 2),
    ("MT", 3, 2),
    ("ND", 4, 2),
    ("MN", 5, 2),
    ("IL", 6, 2),
    ("WI", 7, 2),
    ("MI", 8, 2),
    ("NY", 9, 2),
    ("RI", 10, 2),
    ("MA", 11, 2),
    ("OR", 1, 3),
    ("NV", 2, 3),
    ("WY", 3, 3),
    ("SD", 4, 3),
    ("IA", 5, 3),
    ("IN", 6, 3),
    ("OH", 7, 3),
    ("PA", 8, 3),
    ("NJ", 9, 3),
    ("CT", 10, 3),
    ("CA", 1, 4),
    ("UT", 2, 4),
    ("CO", 3, 4),
    ("NE", 4, 4),
    ("MO", 5, 4),
    ("KY", 6, 4),
    ("WV", 7, 4),
    ("VA", 8, 4),
    ("MD", 9, 4),
    ("DE", 10, 4),
    ("AZ", 2, 5),
    ("NM", 3, 5),
    ("KS", 4, 5),
    ("AR", 5, 5),
    ("TN", 6, 5),
    ("NC", 7, 5),
    ("SC", 8, 5),
    ("OK", 4, 6),
    ("LA", 5, 6),
    ("MS", 6, 6),
    ("AL", 7, 6),
    ("GA", 8, 6),
    ("HI", 0, 7),
    ("TX", 4, 7),
    ("FL", 9, 7),
];

pub fn tiles() -> Vec<Region> {
    STATES
        .iter()
        .map(|(name, column, row)| {
            let (x, y) = (f64::from(*column), -f64::from(*row));
            Region::new(
                *name,
                vec![vec![
                    (x, y),
                    (x + 0.92, y),
                    (x + 0.92, y - 0.92),
                    (x, y - 0.92),
                ]],
            )
        })
        .collect()
}

pub fn state_values(seed: u64, scale: f64) -> Vec<Stat> {
    STATES
        .iter()
        .enumerate()
        .map(|(index, (name, _, _))| Stat {
            region: SharedString::Borrowed(name),
            value: (noise(seed * 100 + index as u64) * scale).round(),
        })
        .collect()
}

pub fn hexes(columns: i32, rows: i32) -> Vec<Region> {
    (0..rows)
        .flat_map(|row| (0..columns).map(move |column| (column, row)))
        .map(|(column, row)| {
            let x =
                f64::from(column) * 3f64.sqrt() + if row % 2 == 1 { 3f64.sqrt() / 2. } else { 0. };
            let y = -f64::from(row) * 1.5;
            let corners = (0..6).map(|corner| {
                let angle =
                    std::f64::consts::PI / 3. * f64::from(corner) + std::f64::consts::PI / 6.;
                (x + angle.cos() * 0.97, y + angle.sin() * 0.97)
            });
            Region::new(
                format!("D{:02}", row * columns + column + 1),
                vec![corners.collect()],
            )
        })
        .collect()
}

pub fn island() -> Vec<Region> {
    let sectors = 9;
    let edge = |angle: f64, scale: f64| -> (f64, f64) {
        let wobble = 1. + 0.18 * (angle * 3.).sin() + 0.1 * (angle * 7. + 1.).cos();
        (
            angle.cos() * wobble * scale * 1.4,
            angle.sin() * wobble * scale,
        )
    };
    let mut regions = Vec::new();
    for sector in 0..sectors {
        let (from, to) = (
            std::f64::consts::TAU * f64::from(sector) / f64::from(sectors),
            std::f64::consts::TAU * f64::from(sector + 1) / f64::from(sectors),
        );
        let arc = |scale: f64| {
            (0..=12).map(move |step| edge((to - from).mul_add(f64::from(step) / 12., from), scale))
        };
        let mut outer: Vec<(f64, f64)> = arc(1.).collect();
        outer.extend(arc(0.45).collect::<Vec<_>>().into_iter().rev());
        regions.push(Region::new(format!("Coast {}", sector + 1), vec![outer]));
        let mut inner: Vec<(f64, f64)> = arc(0.45).collect();
        inner.push((0., 0.));
        regions.push(Region::new(format!("Upland {}", sector + 1), vec![inner]));
    }
    regions
}

pub fn continents() -> Vec<Region> {
    let block = |name: &'static str, west: f64, east: f64, south: f64, north: f64| {
        Region::new(
            name,
            vec![vec![
                (west, south),
                (east, south),
                (east, north),
                (west, north),
            ]],
        )
    };
    vec![
        block("North America", -165., -55., 15., 72.),
        block("South America", -82., -35., -55., 12.),
        block("Europe", -10., 40., 36., 71.),
        block("Africa", -18., 51., -35., 35.),
        block("Asia", 40., 150., 5., 75.),
        block("Oceania", 112., 178., -47., -10.),
        block("Antarctica", -180., 180., -85., -65.),
    ]
}

pub mod choropleth {
    use super::*;

    fn named(entries: &[(&'static str, f64)]) -> Vec<Stat> {
        entries
            .iter()
            .map(|(region, value)| Stat {
                region: SharedString::Borrowed(region),
                value: *value,
            })
            .collect()
    }

    fn districts(seed: u64) -> Vec<Stat> {
        (0..48u64)
            .map(|index| Stat {
                region: format!("D{:02}", index + 1).into(),
                value: 20. + noise(seed + index) * 60.,
            })
            .collect()
    }

    fn provinces(seed: u64) -> Vec<Stat> {
        (1..=9u64)
            .flat_map(|sector| {
                [
                    Stat {
                        region: format!("Coast {sector}").into(),
                        value: 100. + noise(seed + sector) * 400.,
                    },
                    Stat {
                        region: format!("Upland {sector}").into(),
                        value: 20. + noise(seed + 50 + sector) * 120.,
                    },
                ]
            })
            .collect()
    }

    #[component]
    fn states() {
        let (regions, data) = (use_signal(tiles()), use_signal(state_values(1, 100.)));
        rsx! { <Choropleth data={data} regions={regions} name={|s: &Stat| s.region.as_ref()} value={|s: &Stat| s.value} projection={Projection::Plane} class="h-80" /> }
    }

    #[component]
    fn missing() {
        let regions = use_signal(tiles());
        let data = use_signal(
            state_values(2, 100.)
                .into_iter()
                .step_by(2)
                .collect::<Vec<_>>(),
        );
        rsx! { <Choropleth data={data} regions={regions} name={|s: &Stat| s.region.as_ref()} value={|s: &Stat| s.value} projection={Projection::Plane} scheme={HeatmapScheme::Purples} class="h-80" /> }
    }

    #[component]
    fn diverging() {
        let regions = use_signal(tiles());
        let data = use_signal(
            state_values(3, 20.)
                .into_iter()
                .map(|stat| Stat {
                    value: stat.value - 10.,
                    ..stat
                })
                .collect::<Vec<_>>(),
        );
        rsx! { <Choropleth data={data} regions={regions} name={|s: &Stat| s.region.as_ref()} value={|s: &Stat| s.value} projection={Projection::Plane} scheme={HeatmapScheme::RdBu} range={(-10., 10.)} class="h-80" /> }
    }

    #[component]
    fn hex_districts() {
        let (regions, data) = (use_signal(hexes(8, 6)), use_signal(districts(4)));
        rsx! { <Choropleth data={data} regions={regions} name={|s: &Stat| s.region.as_ref()} value={|s: &Stat| s.value} projection={Projection::Plane} scheme={HeatmapScheme::Viridis} decimals={1} class="h-80" /> }
    }

    #[component]
    fn island_map() {
        let (regions, data) = (use_signal(island()), use_signal(provinces(5)));
        rsx! { <Choropleth data={data} regions={regions} name={|s: &Stat| s.region.as_ref()} value={|s: &Stat| s.value} projection={Projection::Plane} scheme={HeatmapScheme::YlGnBu} class="h-80" /> }
    }

    #[component]
    fn world_equirectangular() {
        let regions = use_signal(continents());
        let data = use_signal(named(&[
            ("North America", 600.),
            ("South America", 430.),
            ("Europe", 745.),
            ("Africa", 1400.),
            ("Asia", 4700.),
            ("Oceania", 45.),
        ]));
        rsx! { <Choropleth data={data} regions={regions} name={|s: &Stat| s.region.as_ref()} value={|s: &Stat| s.value} scheme={HeatmapScheme::Oranges} class="h-80" /> }
    }

    #[component]
    fn world_mercator() {
        let regions = use_signal(continents());
        let data = use_signal(named(&[
            ("North America", 600.),
            ("South America", 430.),
            ("Europe", 745.),
            ("Africa", 1400.),
            ("Asia", 4700.),
            ("Oceania", 45.),
        ]));
        rsx! { <Choropleth data={data} regions={regions} name={|s: &Stat| s.region.as_ref()} value={|s: &Stat| s.value} projection={Projection::Mercator} scheme={HeatmapScheme::Oranges} class="h-80" /> }
    }

    #[component]
    fn no_bar() {
        let (regions, data) = (use_signal(tiles()), use_signal(state_values(6, 100.)));
        rsx! { <Choropleth data={data} regions={regions} name={|s: &Stat| s.region.as_ref()} value={|s: &Stat| s.value} projection={Projection::Plane} scale_bar={false} scheme={HeatmapScheme::Greens} class="h-72" /> }
    }

    #[component]
    fn fixed_range() {
        let (regions, data) = (use_signal(hexes(10, 5)), use_signal(districts(7)));
        rsx! { <Choropleth data={data} regions={regions} name={|s: &Stat| s.region.as_ref()} value={|s: &Stat| s.value} projection={Projection::Plane} range={(0., 100.)} scheme={HeatmapScheme::Magma} class="h-80" /> }
    }

    #[component]
    fn turnout() {
        let (regions, data) = (
            use_signal(island()),
            use_signal(
                provinces(8)
                    .into_iter()
                    .map(|stat| Stat {
                        value: 40. + stat.value % 50.,
                        ..stat
                    })
                    .collect::<Vec<_>>(),
            ),
        );
        rsx! { <Choropleth data={data} regions={regions} name={|s: &Stat| s.region.as_ref()} value={|s: &Stat| s.value} projection={Projection::Plane} scheme={HeatmapScheme::Cividis} decimals={0} class="h-80" /> }
    }

    examples! {
        states: "US tile map" => "One square per state coloured by value; hover a state.",
        missing: "Missing data" => "Regions without a value are grey.",
        diverging: "Diverging scale" => "A fixed −10 to 10 range on a red–blue scheme.",
        hex_districts: "Hexagon districts" => "Any polygons work; these are hexagons.",
        island_map: "Island provinces" => "Irregular polygons for coast and upland provinces.",
        world_equirectangular: "Continents" => "Longitude and latitude, plotted as they are.",
        world_mercator: "Mercator" => "The same blocks; high latitudes stretch.",
        no_bar: "No scale bar" => "scale_bar false.",
        fixed_range: "Fixed range" => "range 0–100 keeps colours comparable between maps.",
        turnout: "Turnout" => "Whole-number percentages.",
    }
}
