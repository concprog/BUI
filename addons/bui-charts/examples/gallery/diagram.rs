use blenderui::prelude::*;
use bui_charts::*;

use crate::finance::noise;

pub fn names(items: &[&'static str]) -> Vec<SharedString> {
    items
        .iter()
        .copied()
        .map(SharedString::Borrowed)
        .collect()
}

#[derive(Clone, PartialEq)]
pub struct Column {
    pub name: &'static str,
    pub parts: Vec<f64>,
}

pub fn columns(items: &[(&'static str, &[f64])]) -> Vec<Column> {
    items
        .iter()
        .map(|(name, parts)| Column {
            name,
            parts: parts.to_vec(),
        })
        .collect()
}

#[derive(Clone, PartialEq)]
pub struct Item {
    pub label: SharedString,
    pub value: f64,
}

pub fn items(prefix: &str, count: u64, seed: u64, scale: f64) -> Vec<Item> {
    (0..count)
        .map(|index| Item {
            label: format!("{prefix}-{:02}", index + 1).into(),
            value: (noise(seed * 1000 + index) * scale).round(),
        })
        .collect()
}

pub fn listed(entries: &[(&'static str, f64)]) -> Vec<Item> {
    entries
        .iter()
        .map(|(label, value)| Item {
            label: SharedString::Borrowed(label),
            value: *value,
        })
        .collect()
}

pub mod mekko {
    use super::*;

    fn phones() -> Vec<Column> {
        columns(&[
            ("North America", &[48., 22., 18., 12.]),
            ("Europe", &[34., 28., 21., 17.]),
            ("Asia", &[18., 24., 38., 20.]),
            ("Latin America", &[22., 31., 29., 18.]),
            ("Africa", &[12., 26., 44., 18.]),
        ])
    }

    fn brands() -> Vec<SharedString> {
        names(&["Apple", "Samsung", "Xiaomi", "Other"])
    }

    fn scaled(factor: &[f64]) -> Vec<Column> {
        phones()
            .into_iter()
            .zip(factor)
            .map(|(column, factor)| Column {
                parts: column.parts.iter().map(|part| part * factor).collect(),
                ..column
            })
            .collect()
    }

    #[component]
    fn market_share() {
        let data = use_signal(scaled(&[3.4, 4.1, 9.8, 1.6, 1.1]));
        rsx! { <Mekko data={data} segments={brands()} name={|c: &Column| c.name} value={|c: &Column, i: usize| c.parts[i]} class="h-80" /> }
    }

    #[component]
    fn plain() {
        let data = use_signal(scaled(&[3.4, 4.1, 9.8, 1.6, 1.1]));
        rsx! { <Mekko data={data} segments={brands()} name={|c: &Column| c.name} value={|c: &Column, i: usize| c.parts[i]} show_values={false} class="h-80" /> }
    }

    #[component]
    fn tight() {
        let data = use_signal(scaled(&[3.4, 4.1, 9.8, 1.6, 1.1]));
        rsx! { <Mekko data={data} segments={brands()} name={|c: &Column| c.name} value={|c: &Column, i: usize| c.parts[i]} gap={0.} class="h-80" /> }
    }

    #[component]
    fn legend_right() {
        let data = use_signal(scaled(&[3.4, 4.1, 9.8, 1.6, 1.1]));
        rsx! { <Mekko data={data} segments={brands()} name={|c: &Column| c.name} value={|c: &Column, i: usize| c.parts[i]} legend={Legend::list().right()} class="h-80" /> }
    }

    #[component]
    fn equal_columns() {
        let data = use_signal(phones());
        rsx! { <Mekko data={data} segments={brands()} name={|c: &Column| c.name} value={|c: &Column, i: usize| c.parts[i]} class="h-80" /> }
    }

    #[component]
    fn channels() {
        let data = use_signal(columns(&[
            ("Laptops", &[420., 310., 90.]),
            ("Phones", &[880., 240., 160.]),
            ("Tablets", &[160., 120., 40.]),
            ("Audio", &[210., 60., 130.]),
        ]));
        rsx! { <Mekko data={data} segments={names(&["Online", "Retail", "Partners"])} name={|c: &Column| c.name} value={|c: &Column, i: usize| c.parts[i]} class="h-80" /> }
    }

    #[component]
    fn budget() {
        let data = use_signal(columns(&[
            ("Engineering", &[5.2, 1.1, 0.8, 0.6]),
            ("Sales", &[2.4, 0.4, 1.6, 0.9]),
            ("Marketing", &[1.1, 0.3, 2.2, 0.4]),
            ("Support", &[1.6, 0.6, 0.2, 0.2]),
            ("G&A", &[0.9, 0.7, 0.3, 0.5]),
        ]));
        rsx! { <Mekko data={data} segments={names(&["Salaries", "Tools", "Campaigns", "Travel"])} name={|c: &Column| c.name} value={|c: &Column, i: usize| c.parts[i]} class="h-80" /> }
    }

    #[component]
    fn two_parts() {
        let data = use_signal(columns(&[
            ("Q1", &[620., 380.]),
            ("Q2", &[710., 340.]),
            ("Q3", &[540., 460.]),
            ("Q4", &[930., 270.]),
        ]));
        rsx! { <Mekko data={data} segments={names(&["Renewals", "New"])} name={|c: &Column| c.name} value={|c: &Column, i: usize| c.parts[i]} class="h-72" /> }
    }

    #[component]
    fn many() {
        let data = use_signal(
            (0..12u64)
                .map(|month| Column {
                    name: [
                        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct",
                        "Nov", "Dec",
                    ][month as usize],
                    parts: (0..3)
                        .map(|part| 20. + noise(month * 10 + part) * 80.)
                        .collect(),
                })
                .collect::<Vec<_>>(),
        );
        rsx! { <Mekko data={data} segments={names(&["Web", "App", "Store"])} name={|c: &Column| c.name} value={|c: &Column, i: usize| c.parts[i]} show_values={false} gap={1.} class="h-80" /> }
    }

    #[component]
    fn no_legend() {
        let data = use_signal(scaled(&[3.4, 4.1, 9.8, 1.6, 1.1]));
        rsx! { <Mekko data={data} segments={brands()} name={|c: &Column| c.name} value={|c: &Column, i: usize| c.parts[i]} legend={Legend::hidden()} class="h-72" /> }
    }

    examples! {
        market_share: "Phone market" => "Column width is each region's sales; heights split it by brand. Hover a block for its shares.",
        plain: "Without labels" => "show_values off leaves only the blocks.",
        tight: "No gaps" => "gap 0 joins the columns into one block.",
        legend_right: "Legend on the right" => "Click a brand to hide it; the columns re-scale.",
        equal_columns: "Similar sizes" => "Smaller totals give narrower columns.",
        channels: "Sales channels" => "Products by channel.",
        budget: "Department budgets" => "Spend type within each department.",
        two_parts: "Renewals and new" => "Two segments per quarter.",
        many: "Twelve months" => "Many thin columns with a 1 px gap.",
        no_legend: "No legend" => "Hover the blocks to read them.",
    }
}

pub mod honey_comb {
    use super::*;

    fn steps() -> Thresholds {
        Thresholds::absolute(vec![
            (f64::NEG_INFINITY, rgb(0x40a02b)),
            (60., rgb(0xdf8e1d)),
            (85., rgb(0xd20f39)),
        ])
    }

    #[component]
    fn servers() {
        let data = use_signal(items("web", 24, 1, 100.));
        rsx! { <HoneyComb data={data} name={|i: &Item| i.label.as_ref()} value={|i: &Item| i.value} thresholds={steps()} unit="%" class="h-80" /> }
    }

    #[component]
    fn default_thresholds() {
        let data = use_signal(items("db", 12, 2, 100.));
        rsx! { <HoneyComb data={data} name={|i: &Item| i.label.as_ref()} value={|i: &Item| i.value} class="h-72" /> }
    }

    #[component]
    fn percentage() {
        let data = use_signal(items("disk", 16, 3, 2000.));
        rsx! { <HoneyComb data={data} name={|i: &Item| i.label.as_ref()} value={|i: &Item| i.value} thresholds={Thresholds::percentage(vec![(f64::NEG_INFINITY, rgb(0x1e66f5)), (50., rgb(0x8839ef)), (90., rgb(0xd20f39))])} max={2000.} unit=" GB" class="h-72" /> }
    }

    #[component]
    fn decimals() {
        let data = use_signal(
            items("node", 10, 4, 400.)
                .into_iter()
                .map(|item| Item {
                    value: item.value / 100.,
                    ..item
                })
                .collect::<Vec<_>>(),
        );
        rsx! { <HoneyComb data={data} name={|i: &Item| i.label.as_ref()} value={|i: &Item| i.value} decimals={2} thresholds={Thresholds::absolute(vec![(f64::NEG_INFINITY, rgb(0x40a02b)), (2., rgb(0xd20f39))])} class="h-72" /> }
    }

    #[component]
    fn fixed_columns() {
        let data = use_signal(items("pod", 18, 5, 100.));
        rsx! { <HoneyComb data={data} name={|i: &Item| i.label.as_ref()} value={|i: &Item| i.value} columns={9} thresholds={steps()} unit="%" class="h-56" /> }
    }

    #[component]
    fn crowd() {
        let data = use_signal(items("svc", 80, 6, 100.));
        rsx! { <HoneyComb data={data} name={|i: &Item| i.label.as_ref()} value={|i: &Item| i.value} thresholds={steps()} class="h-80" /> }
    }

    #[component]
    fn few() {
        let data = use_signal(listed(&[("api", 42.), ("web", 77.), ("worker", 93.)]));
        rsx! { <HoneyComb data={data} name={|i: &Item| i.label.as_ref()} value={|i: &Item| i.value} thresholds={steps()} unit="%" class="h-64" /> }
    }

    #[component]
    fn status() {
        let data = use_signal(items("check", 20, 7, 1.).into_iter().collect::<Vec<_>>());
        rsx! { <HoneyComb data={data} name={|i: &Item| i.label.as_ref()} value={|i: &Item| i.value} thresholds={Thresholds::absolute(vec![(f64::NEG_INFINITY, rgb(0xd20f39)), (1., rgb(0x40a02b))])} class="h-72" /> }
    }

    #[component]
    fn latency() {
        let data = use_signal(listed(&[
            ("checkout", 120.),
            ("search", 45.),
            ("login", 310.),
            ("cart", 80.),
            ("profile", 60.),
            ("payments", 540.),
            ("images", 30.),
        ]));
        rsx! { <HoneyComb data={data} name={|i: &Item| i.label.as_ref()} value={|i: &Item| i.value} thresholds={Thresholds::absolute(vec![(f64::NEG_INFINITY, rgb(0x40a02b)), (100., rgb(0xdf8e1d)), (300., rgb(0xd20f39))])} unit=" ms" class="h-72" /> }
    }

    #[component]
    fn missing() {
        let data = use_signal(
            items("host", 14, 8, 100.)
                .into_iter()
                .enumerate()
                .map(|(index, item)| Item {
                    value: if index % 5 == 2 { f64::NAN } else { item.value },
                    ..item
                })
                .collect::<Vec<_>>(),
        );
        rsx! { <HoneyComb data={data} name={|i: &Item| i.label.as_ref()} value={|i: &Item| i.value} thresholds={steps()} unit="%" class="h-72" /> }
    }

    examples! {
        servers: "Server load" => "One hexagon per server, coloured by threshold; the grid fills the space.",
        default_thresholds: "Default thresholds" => "Without thresholds, good below 80 and bad above.",
        percentage: "Percentage thresholds" => "Steps at 50% and 90% of a 2 000 GB range.",
        decimals: "Two decimals" => "decimals fixes the value format.",
        fixed_columns: "Nine columns" => "columns sets the grid width.",
        crowd: "Eighty services" => "Labels hide when cells get small; hover to read them.",
        few: "Three cells" => "A few large cells.",
        status: "Up or down" => "0 and 1 as red and green.",
        latency: "Endpoint latency" => "Millisecond steps at 100 and 300.",
        missing: "Missing values" => "Cells with no value are greyed out.",
    }
}

#[derive(Clone, PartialEq)]
pub struct Leaf {
    pub path: &'static str,
    pub size: f64,
}

pub fn leaves(entries: &[(&'static str, f64)]) -> Vec<Leaf> {
    entries
        .iter()
        .map(|(path, size)| Leaf { path, size: *size })
        .collect()
}

pub fn disk() -> Vec<Leaf> {
    leaves(&[
        ("src/charts/trend", 420.),
        ("src/charts/xy", 260.),
        ("src/charts/heatmap", 310.),
        ("src/plot/axis", 120.),
        ("src/plot/scale", 90.),
        ("src/plot/labels", 70.),
        ("docs/guides", 180.),
        ("docs/api", 220.),
        ("assets/fonts", 640.),
        ("assets/icons", 150.),
        ("target/debug", 1800.),
        ("target/release", 900.),
    ])
}

pub mod sunburst {
    use super::*;

    #[component]
    fn files() {
        let data = use_signal(disk());
        rsx! { <Sunburst data={data} path={|l: &Leaf| l.path} value={|l: &Leaf| l.size} root="repo" class="h-96" /> }
    }

    #[component]
    fn two_levels() {
        let data = use_signal(disk());
        rsx! { <Sunburst data={data} path={|l: &Leaf| l.path} value={|l: &Leaf| l.size} levels={2} class="h-96" /> }
    }

    #[component]
    fn budget() {
        let data = use_signal(leaves(&[
            ("Housing/Rent", 1600.),
            ("Housing/Utilities", 180.),
            ("Food/Groceries", 520.),
            ("Food/Restaurants", 240.),
            ("Transport/Car", 310.),
            ("Transport/Transit", 90.),
            ("Savings/Retirement", 700.),
            ("Savings/Emergency", 300.),
            ("Fun/Travel", 250.),
            ("Fun/Hobbies", 120.),
        ]));
        rsx! { <Sunburst data={data} path={|l: &Leaf| l.path} value={|l: &Leaf| l.size} root="Budget" class="h-96" /> }
    }

    #[component]
    fn population() {
        let data = use_signal(leaves(&[
            ("Asia.India", 1428.),
            ("Asia.China", 1425.),
            ("Asia.Indonesia", 277.),
            ("Asia.Pakistan", 240.),
            ("Africa.Nigeria", 224.),
            ("Africa.Ethiopia", 127.),
            ("Africa.Egypt", 113.),
            ("Americas.USA", 340.),
            ("Americas.Brazil", 216.),
            ("Americas.Mexico", 129.),
            ("Europe.Russia", 144.),
            ("Europe.Germany", 84.),
        ]));
        rsx! { <Sunburst data={data} path={|l: &Leaf| l.path} value={|l: &Leaf| l.size} separator="." root="World (M)" class="h-96" /> }
    }

    #[component]
    fn org() {
        let data = use_signal(leaves(&[
            ("Eng/Platform/Infra", 14.),
            ("Eng/Platform/Data", 9.),
            ("Eng/Product/Web", 18.),
            ("Eng/Product/Mobile", 11.),
            ("Eng/Product/Design", 6.),
            ("Sales/EMEA", 12.),
            ("Sales/Americas", 16.),
            ("Sales/APAC", 8.),
            ("Ops/Finance", 5.),
            ("Ops/People", 4.),
        ]));
        rsx! { <Sunburst data={data} path={|l: &Leaf| l.path} value={|l: &Leaf| l.size} root="Headcount" class="h-96" /> }
    }

    #[component]
    fn single_level() {
        let data = use_signal(leaves(&[
            ("Chrome", 65.),
            ("Safari", 18.),
            ("Edge", 5.),
            ("Firefox", 3.),
            ("Other", 9.),
        ]));
        rsx! { <Sunburst data={data} path={|l: &Leaf| l.path} value={|l: &Leaf| l.size} root="Browsers" class="h-80" /> }
    }

    #[component]
    fn deep() {
        let data = use_signal(
            (0..40u64)
                .map(|index| Leaf {
                    path: [
                        "a/b/c/d", "a/b/c/e", "a/b/f", "a/g/h/i", "a/g/h/j", "a/g/k", "l/m/n",
                        "l/m/o/p", "l/q", "r/s/t/u",
                    ][(index % 10) as usize],
                    size: 1. + noise(index) * 10.,
                })
                .collect::<Vec<_>>(),
        );
        rsx! { <Sunburst data={data} path={|l: &Leaf| l.path} value={|l: &Leaf| l.size} class="h-96" /> }
    }

    #[component]
    fn sales() {
        let data = use_signal(leaves(&[
            ("Europe/Germany/Berlin", 32.),
            ("Europe/Germany/Munich", 21.),
            ("Europe/France/Paris", 40.),
            ("Europe/France/Lyon", 12.),
            ("Americas/USA/New York", 55.),
            ("Americas/USA/Austin", 18.),
            ("Americas/Canada/Toronto", 22.),
            ("Asia/Japan/Tokyo", 47.),
            ("Asia/India/Bengaluru", 29.),
        ]));
        rsx! { <Sunburst data={data} path={|l: &Leaf| l.path} value={|l: &Leaf| l.size} root="Revenue" class="h-96" /> }
    }

    #[component]
    fn no_tooltip() {
        let data = use_signal(disk());
        rsx! { <Sunburst data={data} path={|l: &Leaf| l.path} value={|l: &Leaf| l.size} tooltip={Tooltip::hidden()} class="h-80" /> }
    }

    #[component]
    fn small() {
        let data = use_signal(disk());
        rsx! { <Sunburst data={data} path={|l: &Leaf| l.path} value={|l: &Leaf| l.size} class="h-56" /> }
    }

    examples! {
        files: "Repository size" => "Rings go from folders to subfolders; click a slice to zoom in and the centre to zoom out.",
        two_levels: "Two rings" => "levels caps how many rings are drawn.",
        budget: "Monthly budget" => "Categories and items.",
        population: "World population" => "A dot separator and a custom root label.",
        org: "Headcount" => "Departments, groups and teams.",
        single_level: "One level" => "A flat list makes a donut.",
        deep: "Deep tree" => "Up to four levels with repeated paths summed.",
        sales: "Revenue by city" => "Region, country and city.",
        no_tooltip: "No tooltip" => "Labels only.",
        small: "Small" => "Labels drop out as slices shrink.",
    }
}

pub mod chord_diagram {
    use super::*;

    fn regions() -> Vec<SharedString> {
        names(&["Africa", "Americas", "Asia", "Europe", "Oceania"])
    }

    fn migration() -> Vec<f64> {
        vec![
            0., 3.2, 1.1, 9.4, 0.3, 2.1, 0., 4.5, 3.3, 0.6, 1.4, 7.8, 0., 6.2, 2.9, 2.2, 3.1, 2.4,
            0., 1.5, 0.1, 0.4, 0.9, 0.7, 0.,
        ]
    }

    fn random(count: usize, seed: u64) -> Vec<f64> {
        (0..count * count)
            .map(|cell| {
                if cell % (count + 1) == 0 {
                    0.
                } else {
                    (noise(seed * 100 + cell as u64) * 10.).round()
                }
            })
            .collect()
    }

    #[component]
    fn migrations() {
        rsx! { <ChordDiagram names={regions()} matrix={migration()} class="h-96" /> }
    }

    #[component]
    fn wide_gaps() {
        rsx! { <ChordDiagram names={regions()} matrix={migration()} pad_angle={8.} class="h-96" /> }
    }

    #[component]
    fn faint() {
        rsx! { <ChordDiagram names={regions()} matrix={migration()} ribbon_opacity={0.3} class="h-96" /> }
    }

    #[component]
    fn with_legend() {
        rsx! { <ChordDiagram names={regions()} matrix={migration()} legend={Legend::list().right()} class="h-96" /> }
    }

    #[component]
    fn trade() {
        rsx! { <ChordDiagram names={names(&["USA", "China", "EU", "Japan", "Mexico", "Canada"])} matrix={random(6, 1)} class="h-96" /> }
    }

    #[component]
    fn three() {
        rsx! { <ChordDiagram names={names(&["Red", "Green", "Blue"])} matrix={vec![0., 4., 2., 3., 0., 5., 1., 6., 0.]} class="h-80" /> }
    }

    #[component]
    fn symmetric() {
        let size = 5;
        let base = random(size, 2);
        let matrix = (0..size * size)
            .map(|cell| {
                let (row, column) = (cell / size, cell % size);
                base[row.min(column) * size + row.max(column)]
            })
            .collect();
        rsx! { <ChordDiagram names={names(&["Rust", "Go", "Python", "TypeScript", "Java"])} matrix={matrix} class="h-96" /> }
    }

    #[component]
    fn many() {
        rsx! { <ChordDiagram names={names(&["A", "B", "C", "D", "E", "F", "G", "H", "I", "J"])} matrix={random(10, 3)} pad_angle={1.} class="h-96" /> }
    }

    #[component]
    fn self_links() {
        rsx! { <ChordDiagram names={names(&["Team A", "Team B", "Team C"])} matrix={vec![5., 2., 1., 1., 6., 2., 3., 1., 4.]} class="h-80" /> }
    }

    #[component]
    fn one_way() {
        rsx! { <ChordDiagram names={names(&["Source", "Hub", "Sink", "Edge"])} matrix={vec![0., 8., 0., 2., 0., 0., 7., 1., 0., 0., 0., 0., 0., 3., 1., 0.]} class="h-80" /> }
    }

    examples! {
        migrations: "Migration between regions" => "Each arc is a region's outflow; ribbons join both directions. Hover to compare.",
        wide_gaps: "Wider gaps" => "pad_angle 8° separates the groups.",
        faint: "Faint ribbons" => "ribbon_opacity 0.3.",
        with_legend: "With a legend" => "Click a region to remove it.",
        trade: "Trade flows" => "Six partners.",
        three: "Three groups" => "A small matrix.",
        symmetric: "Co-usage" => "A symmetric matrix: equal both ways.",
        many: "Ten groups" => "Thin arcs with a 1° gap.",
        self_links: "Self links" => "The diagonal draws ribbons that return to their own group.",
        one_way: "One-way flows" => "Flows in only one direction.",
    }
}

#[derive(Clone, PartialEq)]
pub struct Edge {
    pub from: &'static str,
    pub to: &'static str,
    pub weight: f64,
}

pub fn edges(entries: &[(&'static str, &'static str, f64)]) -> Vec<Edge> {
    entries
        .iter()
        .map(|(from, to, weight)| Edge {
            from,
            to,
            weight: *weight,
        })
        .collect()
}

pub mod arc_diagram {
    use super::*;

    fn scenes() -> Vec<Edge> {
        edges(&[
            ("Valjean", "Javert", 17.),
            ("Valjean", "Cosette", 31.),
            ("Valjean", "Marius", 19.),
            ("Valjean", "Fantine", 9.),
            ("Cosette", "Marius", 21.),
            ("Marius", "Enjolras", 7.),
            ("Enjolras", "Gavroche", 7.),
            ("Gavroche", "Thenardier", 3.),
            ("Thenardier", "Cosette", 4.),
            ("Fantine", "Javert", 5.),
            ("Javert", "Enjolras", 6.),
            ("Myriel", "Valjean", 5.),
        ])
    }

    #[component]
    fn characters() {
        let data = use_signal(scenes());
        rsx! { <ArcDiagram data={data} source={|e: &Edge| e.from} target={|e: &Edge| e.to} value={|e: &Edge| e.weight} class="h-80" /> }
    }

    #[component]
    fn unweighted() {
        let data = use_signal(scenes());
        rsx! { <ArcDiagram data={data} source={|e: &Edge| e.from} target={|e: &Edge| e.to} class="h-80" /> }
    }

    #[component]
    fn modules() {
        let data = use_signal(edges(&[
            ("app", "ui", 8.),
            ("app", "store", 6.),
            ("ui", "theme", 4.),
            ("ui", "icons", 2.),
            ("store", "api", 5.),
            ("api", "http", 7.),
            ("api", "auth", 3.),
            ("auth", "http", 2.),
            ("store", "cache", 3.),
        ]));
        rsx! { <ArcDiagram data={data} source={|e: &Edge| e.from} target={|e: &Edge| e.to} value={|e: &Edge| e.weight} class="h-72" /> }
    }

    #[component]
    fn flights() {
        let data = use_signal(edges(&[
            ("LHR", "JFK", 42.),
            ("LHR", "DXB", 30.),
            ("JFK", "LAX", 55.),
            ("DXB", "SIN", 25.),
            ("SIN", "SYD", 18.),
            ("LAX", "SYD", 12.),
            ("CDG", "JFK", 28.),
            ("CDG", "DXB", 16.),
            ("HND", "SIN", 21.),
            ("HND", "LAX", 26.),
        ]));
        rsx! { <ArcDiagram data={data} source={|e: &Edge| e.from} target={|e: &Edge| e.to} value={|e: &Edge| e.weight} class="h-80" /> }
    }

    #[component]
    fn chain() {
        let data = use_signal(edges(&[
            ("A", "B", 1.),
            ("B", "C", 1.),
            ("C", "D", 1.),
            ("D", "E", 1.),
            ("E", "F", 1.),
        ]));
        rsx! { <ArcDiagram data={data} source={|e: &Edge| e.from} target={|e: &Edge| e.to} value={|e: &Edge| e.weight} class="h-48" /> }
    }

    #[component]
    fn star() {
        let data = use_signal(edges(&[
            ("hub", "a", 5.),
            ("hub", "b", 3.),
            ("hub", "c", 8.),
            ("hub", "d", 2.),
            ("hub", "e", 6.),
            ("hub", "f", 4.),
        ]));
        rsx! { <ArcDiagram data={data} source={|e: &Edge| e.from} target={|e: &Edge| e.to} value={|e: &Edge| e.weight} class="h-72" /> }
    }

    #[component]
    fn crowded() {
        let labels = [
            "n01", "n02", "n03", "n04", "n05", "n06", "n07", "n08", "n09", "n10", "n11", "n12",
            "n13", "n14", "n15", "n16", "n17", "n18", "n19", "n20",
        ];
        let data = use_signal(
            (0..40u64)
                .map(|index| Edge {
                    from: labels[(noise(index) * 20.) as usize % 20],
                    to: labels[(noise(index + 500) * 20.) as usize % 20],
                    weight: 1. + noise(index + 900) * 4.,
                })
                .collect::<Vec<_>>(),
        );
        rsx! { <ArcDiagram data={data} source={|e: &Edge| e.from} target={|e: &Edge| e.to} value={|e: &Edge| e.weight} class="h-80" /> }
    }

    #[component]
    fn citations() {
        let data = use_signal(edges(&[
            ("2024", "2019", 3.),
            ("2024", "2021", 5.),
            ("2023", "2019", 2.),
            ("2023", "2015", 1.),
            ("2021", "2015", 4.),
            ("2019", "2015", 6.),
            ("2022", "2021", 2.),
            ("2022", "2019", 3.),
        ]));
        rsx! { <ArcDiagram data={data} source={|e: &Edge| e.from} target={|e: &Edge| e.to} value={|e: &Edge| e.weight} class="h-72" /> }
    }

    #[component]
    fn pair() {
        let data = use_signal(edges(&[("Alice", "Bob", 4.)]));
        rsx! { <ArcDiagram data={data} source={|e: &Edge| e.from} target={|e: &Edge| e.to} value={|e: &Edge| e.weight} class="h-48" /> }
    }

    #[component]
    fn quiet() {
        let data = use_signal(scenes());
        rsx! { <ArcDiagram data={data} source={|e: &Edge| e.from} target={|e: &Edge| e.to} value={|e: &Edge| e.weight} tooltip={Tooltip::hidden()} class="h-72" /> }
    }

    examples! {
        characters: "Shared scenes" => "Nodes on a line, arcs sized by weight and coloured by source. Hover an arc or a node.",
        unweighted: "Unweighted" => "Without value every link counts as 1.",
        modules: "Module imports" => "A dependency graph read left to right.",
        flights: "Flight routes" => "Airports linked by weekly flights.",
        chain: "A chain" => "Neighbours only.",
        star: "A hub" => "One node links to all others.",
        crowded: "Twenty nodes" => "Labels alternate rows when crowded.",
        citations: "Citations" => "Newer papers citing older ones.",
        pair: "One link" => "The smallest case.",
        quiet: "No tooltip" => "Shapes only.",
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Cause {
    pub group: &'static str,
    pub reason: &'static str,
}

pub fn causes(entries: &[(&'static str, &'static str)]) -> Vec<Cause> {
    entries
        .iter()
        .map(|(group, reason)| Cause { group, reason })
        .collect()
}

pub mod ishikawa {
    use super::*;

    fn factory() -> Vec<Cause> {
        causes(&[
            ("Machine", "Worn spindle"),
            ("Machine", "Loose fixture"),
            ("Method", "No warm-up"),
            ("Method", "Wrong feed rate"),
            ("Material", "Hard batch"),
            ("Material", "Burrs"),
            ("Man", "New operator"),
            ("Man", "Fatigue"),
            ("Measurement", "Gauge drift"),
            ("Measurement", "Wrong datum"),
            ("Environment", "Heat"),
            ("Environment", "Vibration"),
        ])
    }

    #[component]
    fn six_m() {
        let data = use_signal(factory());
        rsx! { <Ishikawa data={data} effect="Oversized bores" category={|c: &Cause| c.group} cause={|c: &Cause| c.reason} class="h-96" /> }
    }

    #[component]
    fn outage() {
        let data = use_signal(causes(&[
            ("Code", "Unchecked null"),
            ("Code", "Retry storm"),
            ("Infra", "Disk full"),
            ("Infra", "DNS TTL"),
            ("Process", "No canary"),
            ("Process", "Friday deploy"),
            ("Monitoring", "Alert muted"),
            ("Monitoring", "No SLO"),
        ]));
        rsx! { <Ishikawa data={data} effect="Checkout outage" category={|c: &Cause| c.group} cause={|c: &Cause| c.reason} class="h-80" /> }
    }

    #[component]
    fn late_delivery() {
        let data = use_signal(causes(&[
            ("Suppliers", "Late parts"),
            ("Suppliers", "Wrong spec"),
            ("Planning", "Optimistic dates"),
            ("Planning", "No buffer"),
            ("Logistics", "Customs"),
            ("Logistics", "Carrier delay"),
        ]));
        rsx! { <Ishikawa data={data} effect="Late delivery" category={|c: &Cause| c.group} cause={|c: &Cause| c.reason} class="h-80" /> }
    }

    #[component]
    fn churn() {
        let data = use_signal(causes(&[
            ("Product", "Missing feature"),
            ("Product", "Slow app"),
            ("Price", "Too expensive"),
            ("Support", "Slow replies"),
            ("Support", "No phone line"),
            ("Onboarding", "Confusing setup"),
            ("Competition", "Cheaper rival"),
        ]));
        rsx! { <Ishikawa data={data} effect="Customer churn" category={|c: &Cause| c.group} cause={|c: &Cause| c.reason} class="h-80" /> }
    }

    #[component]
    fn two_bones() {
        let data = use_signal(causes(&[
            ("People", "Training"),
            ("People", "Staffing"),
            ("Tools", "Old laptops"),
            ("Tools", "Slow CI"),
        ]));
        rsx! { <Ishikawa data={data} effect="Slow releases" category={|c: &Cause| c.group} cause={|c: &Cause| c.reason} class="h-72" /> }
    }

    #[component]
    fn single() {
        let data = use_signal(causes(&[
            ("Weather", "Rain"),
            ("Weather", "Wind"),
            ("Weather", "Snow"),
        ]));
        rsx! { <Ishikawa data={data} effect="Cancelled match" category={|c: &Cause| c.group} cause={|c: &Cause| c.reason} class="h-64" /> }
    }

    #[component]
    fn many_causes() {
        let data = use_signal(causes(&[
            ("Design", "Tolerance"),
            ("Design", "Material choice"),
            ("Design", "Thin wall"),
            ("Design", "Sharp corner"),
            ("Design", "No draft"),
            ("Process", "Mould temp"),
            ("Process", "Cycle time"),
            ("Process", "Pressure"),
            ("Process", "Cooling"),
        ]));
        rsx! { <Ishikawa data={data} effect="Warped parts" category={|c: &Cause| c.group} cause={|c: &Cause| c.reason} class="h-80" /> }
    }

    #[component]
    fn eight() {
        let data = use_signal(causes(&[
            ("Product", "Recipe"),
            ("Price", "Discounts"),
            ("Place", "Location"),
            ("Promotion", "Ads"),
            ("People", "Staff"),
            ("Process", "Wait time"),
            ("Physical", "Seating"),
            ("Productivity", "Turnover"),
        ]));
        rsx! { <Ishikawa data={data} effect="Fewer diners" category={|c: &Cause| c.group} cause={|c: &Cause| c.reason} class="h-96" /> }
    }

    #[component]
    fn long_effect() {
        let data = use_signal(factory());
        rsx! { <Ishikawa data={data} effect="Customer complaints about finish" category={|c: &Cause| c.group} cause={|c: &Cause| c.reason} class="h-96" /> }
    }

    #[component]
    fn compact() {
        let data = use_signal(factory());
        rsx! { <Ishikawa data={data} effect="Defects" category={|c: &Cause| c.group} cause={|c: &Cause| c.reason} class="h-64" /> }
    }

    examples! {
        six_m: "Six Ms" => "Categories alternate above and below the spine; hover a bone to list its causes.",
        outage: "Incident review" => "Code, infra, process and monitoring.",
        late_delivery: "Late delivery" => "Three categories.",
        churn: "Customer churn" => "Categories with one or two causes each.",
        two_bones: "Two categories" => "One above and one below.",
        single: "One category" => "Only the top side.",
        many_causes: "Many causes" => "Five causes on one bone.",
        eight: "Eight Ps" => "The service-industry eight.",
        long_effect: "Long effect" => "The head grows to fit.",
        compact: "Compact" => "Labels shorten to fit small charts.",
    }
}

#[derive(Clone, PartialEq)]
pub struct Member {
    pub sets: &'static str,
    pub count: f64,
}

pub fn members(entries: &[(&'static str, f64)]) -> Vec<Member> {
    entries
        .iter()
        .map(|(sets, count)| Member {
            sets,
            count: *count,
        })
        .collect()
}

pub mod venn {
    use super::*;

    #[component]
    fn two() {
        let data = use_signal(members(&[
            ("Python", 420.),
            ("Rust", 180.),
            ("Python&Rust", 90.),
        ]));
        rsx! { <Venn data={data} sets={|m: &Member| m.sets} value={|m: &Member| m.count} class="h-80" /> }
    }

    #[component]
    fn three() {
        let data = use_signal(members(&[
            ("Email", 300.),
            ("Push", 200.),
            ("SMS", 120.),
            ("Email&Push", 80.),
            ("Email&SMS", 40.),
            ("Push&SMS", 30.),
            ("Email&Push&SMS", 15.),
        ]));
        rsx! { <Venn data={data} sets={|m: &Member| m.sets} value={|m: &Member| m.count} class="h-80" /> }
    }

    #[component]
    fn disjoint() {
        let data = use_signal(members(&[("Cats", 120.), ("Dogs", 160.)]));
        rsx! { <Venn data={data} sets={|m: &Member| m.sets} value={|m: &Member| m.count} class="h-72" /> }
    }

    #[component]
    fn subset() {
        let data = use_signal(members(&[("Mammals", 400.), ("Mammals&Primates", 60.)]));
        rsx! { <Venn data={data} sets={|m: &Member| m.sets} value={|m: &Member| m.count} class="h-72" /> }
    }

    #[component]
    fn items() {
        let data = use_signal(members(&[
            ("A", 1.),
            ("A", 1.),
            ("A&B", 1.),
            ("B", 1.),
            ("B&C", 1.),
            ("C", 1.),
            ("C", 1.),
            ("A&B&C", 1.),
        ]));
        rsx! { <Venn data={data} sets={|m: &Member| m.sets} class="h-72" /> }
    }

    #[component]
    fn commas() {
        let data = use_signal(members(&[
            ("iOS", 50.),
            ("Android", 70.),
            ("iOS, Android", 25.),
        ]));
        rsx! { <Venn data={data} sets={|m: &Member| m.sets} value={|m: &Member| m.count} separator="," class="h-72" /> }
    }

    #[component]
    fn solid() {
        let data = use_signal(members(&[
            ("Read", 500.),
            ("Write", 300.),
            ("Read&Write", 220.),
        ]));
        rsx! { <Venn data={data} sets={|m: &Member| m.sets} value={|m: &Member| m.count} fill_opacity={0.55} class="h-72" /> }
    }

    #[component]
    fn with_legend() {
        let data = use_signal(members(&[
            ("Frontend", 40.),
            ("Backend", 55.),
            ("DevOps", 20.),
            ("Frontend&Backend", 25.),
            ("Backend&DevOps", 15.),
        ]));
        rsx! { <Venn data={data} sets={|m: &Member| m.sets} value={|m: &Member| m.count} legend={Legend::list()} class="h-80" /> }
    }

    #[component]
    fn four() {
        let data = use_signal(members(&[
            ("A", 50.),
            ("B", 40.),
            ("C", 30.),
            ("D", 25.),
            ("A&B", 12.),
            ("B&C", 10.),
            ("C&D", 8.),
            ("A&D", 6.),
        ]));
        rsx! { <Venn data={data} sets={|m: &Member| m.sets} value={|m: &Member| m.count} class="h-80" /> }
    }

    #[component]
    fn equal() {
        let data = use_signal(members(&[
            ("Left", 100.),
            ("Right", 100.),
            ("Left&Right", 50.),
        ]));
        rsx! { <Venn data={data} sets={|m: &Member| m.sets} value={|m: &Member| m.count} class="h-72" /> }
    }

    examples! {
        two: "Two sets" => "Circle areas follow set sizes and the overlap follows the shared count.",
        three: "Three channels" => "Every region shows its exclusive count.",
        disjoint: "No overlap" => "Sets with nothing in common sit apart.",
        subset: "A subset" => "One set inside another.",
        items: "From items" => "Without value each row counts once.",
        commas: "Comma separator" => "separator \",\".",
        solid: "Stronger fill" => "fill_opacity 0.55.",
        with_legend: "With a legend" => "Click a set to hide it.",
        four: "Four sets" => "Placed as close to the overlaps as circles allow.",
        equal: "Equal sets" => "Two equal circles.",
    }
}

pub mod up_set {
    use super::*;

    fn genres() -> Vec<Member> {
        members(&[
            ("Drama", 520.),
            ("Comedy", 410.),
            ("Action", 260.),
            ("Drama&Romance", 180.),
            ("Comedy&Romance", 150.),
            ("Action&Thriller", 140.),
            ("Drama&Thriller", 90.),
            ("Action&Comedy", 70.),
            ("Horror", 120.),
            ("Horror&Thriller", 60.),
            ("Drama&Comedy&Romance", 40.),
            ("Action&Drama&Thriller", 30.),
        ])
    }

    #[component]
    fn movies() {
        let data = use_signal(genres());
        rsx! { <UpSet data={data} sets={|m: &Member| m.sets} value={|m: &Member| m.count} class="h-96" /> }
    }

    #[component]
    fn top_five() {
        let data = use_signal(genres());
        rsx! { <UpSet data={data} sets={|m: &Member| m.sets} value={|m: &Member| m.count} max_intersections={5} class="h-80" /> }
    }

    #[component]
    fn minimum() {
        let data = use_signal(genres());
        rsx! { <UpSet data={data} sets={|m: &Member| m.sets} value={|m: &Member| m.count} min_size={100.} class="h-80" /> }
    }

    #[component]
    fn from_items() {
        let data = use_signal(
            (0..300u64)
                .map(|index| Member {
                    sets: [
                        "Web",
                        "Web&Mobile",
                        "Mobile",
                        "Web&Desktop",
                        "Desktop",
                        "Web&Mobile&Desktop",
                        "Mobile&Desktop",
                    ][((noise(index) * noise(index + 7)) * 7.) as usize % 7],
                    count: 1.,
                })
                .collect::<Vec<_>>(),
        );
        rsx! { <UpSet data={data} sets={|m: &Member| m.sets} class="h-80" /> }
    }

    #[component]
    fn two_sets() {
        let data = use_signal(members(&[
            ("Read", 500.),
            ("Write", 300.),
            ("Read&Write", 220.),
        ]));
        rsx! { <UpSet data={data} sets={|m: &Member| m.sets} value={|m: &Member| m.count} class="h-64" /> }
    }

    #[component]
    fn mutations() {
        let data = use_signal(members(&[
            ("TP53", 210.),
            ("KRAS", 160.),
            ("TP53&KRAS", 75.),
            ("EGFR", 90.),
            ("TP53&EGFR", 40.),
            ("PIK3CA", 70.),
            ("KRAS&PIK3CA", 30.),
            ("TP53&KRAS&PIK3CA", 12.),
            ("BRAF", 35.),
        ]));
        rsx! { <UpSet data={data} sets={|m: &Member| m.sets} value={|m: &Member| m.count} class="h-96" /> }
    }

    #[component]
    fn commas() {
        let data = use_signal(members(&[
            ("vim", 40.),
            ("emacs", 25.),
            ("vim, vscode", 30.),
            ("vscode", 90.),
            ("emacs, vscode", 12.),
        ]));
        rsx! { <UpSet data={data} sets={|m: &Member| m.sets} value={|m: &Member| m.count} separator="," class="h-72" /> }
    }

    #[component]
    fn skills() {
        let data = use_signal(members(&[
            ("SQL", 60.),
            ("SQL&Python", 45.),
            ("Python", 30.),
            ("Python&ML", 25.),
            ("SQL&Python&ML", 18.),
            ("Excel", 50.),
            ("Excel&SQL", 22.),
        ]));
        rsx! { <UpSet data={data} sets={|m: &Member| m.sets} value={|m: &Member| m.count} class="h-80" /> }
    }

    #[component]
    fn many_sets() {
        let labels = ["A", "B", "C", "D", "E", "F", "G", "H"];
        let combos: Vec<&'static str> = vec![
            "A", "B", "C", "D", "E", "F", "G", "H", "A&B", "C&D", "E&F", "G&H", "A&C&E", "B&D&F",
            "A&H",
        ];
        let data = use_signal(
            combos
                .iter()
                .enumerate()
                .map(|(index, sets)| Member {
                    sets,
                    count: (5. + noise(index as u64) * 60.).round(),
                })
                .collect::<Vec<_>>(),
        );
        let _ = labels;
        rsx! { <UpSet data={data} sets={|m: &Member| m.sets} value={|m: &Member| m.count} class="h-96" /> }
    }

    #[component]
    fn compact() {
        let data = use_signal(genres());
        rsx! { <UpSet data={data} sets={|m: &Member| m.sets} value={|m: &Member| m.count} max_intersections={8} class="h-64" /> }
    }

    examples! {
        movies: "Movie genres" => "Bars are exact combinations, the matrix shows which sets, and the left bars are set sizes.",
        top_five: "Top five" => "max_intersections 5.",
        minimum: "At least 100" => "min_size drops small combinations.",
        from_items: "From items" => "Each row is one item; combinations are counted.",
        two_sets: "Two sets" => "Where a Venn diagram would also work.",
        mutations: "Co-mutations" => "Genes mutated together.",
        commas: "Comma separator" => "Editors people use together.",
        skills: "Skill overlap" => "Seven combinations.",
        many_sets: "Eight sets" => "UpSet scales where Venn diagrams cannot.",
        compact: "Compact" => "A short chart.",
    }
}

pub mod circos {
    use super::*;

    fn chromosomes(count: usize) -> Vec<(SharedString, f64)> {
        (0..count)
            .map(|index| {
                (
                    SharedString::from(format!("chr{}", index + 1)),
                    (250. - index as f64 * 14.).max(50.),
                )
            })
            .collect()
    }

    fn links(count: usize, segments: usize, seed: u64, width: f64) -> Vec<CircosLink> {
        (0..count as u64)
            .map(|index| {
                let source = (noise(seed + index) * segments as f64) as usize % segments;
                let target = (noise(seed + 500 + index) * segments as f64) as usize % segments;
                let from = noise(seed + 900 + index) * 40.;
                let to = noise(seed + 1300 + index) * 40.;
                CircosLink {
                    source,
                    from: (from, from + width),
                    target,
                    to: (to, to + width),
                    weight: 1. + noise(seed + 1700 + index) * 9.,
                }
            })
            .collect()
    }

    fn track(segments: &[(SharedString, f64)], bins: usize, seed: u64) -> Vec<CircosBin> {
        segments
            .iter()
            .enumerate()
            .flat_map(|(segment, (_, length))| {
                let step = length / bins as f64;
                (0..bins).map(move |bin| CircosBin {
                    segment,
                    start: bin as f64 * step,
                    end: (bin + 1) as f64 * step,
                    value: noise(seed + (segment * 100 + bin) as u64) * 10.,
                })
            })
            .collect()
    }

    #[component]
    fn genome() {
        let segments = chromosomes(12);
        let tracks = vec![track(&segments, 20, 1), track(&segments, 20, 2)];
        rsx! { <Circos segments={segments} links={links(30, 12, 3, 6.)} tracks={tracks} ticks={50.} class="h-[480px]" /> }
    }

    #[component]
    fn ideogram() {
        rsx! { <Circos segments={chromosomes(8)} ticks={25.} class="h-96" /> }
    }

    #[component]
    fn thin_links() {
        rsx! { <Circos segments={chromosomes(10)} links={links(60, 10, 4, 0.)} class="h-96" /> }
    }

    #[component]
    fn ribbons() {
        rsx! { <Circos segments={chromosomes(6)} links={links(12, 6, 5, 18.)} link_opacity={0.4} class="h-96" /> }
    }

    #[component]
    fn tracks_only() {
        let segments = chromosomes(10);
        let tracks = vec![
            track(&segments, 24, 6),
            track(&segments, 24, 7),
            track(&segments, 24, 8),
        ];
        rsx! { <Circos segments={segments} tracks={tracks} class="h-96" /> }
    }

    #[component]
    fn wide_gaps() {
        rsx! { <Circos segments={chromosomes(5)} links={links(15, 5, 9, 10.)} gap={10.} class="h-96" /> }
    }

    #[component]
    fn departments() {
        let segments: Vec<(SharedString, f64)> = vec![
            ("Sales".into(), 120.),
            ("Support".into(), 80.),
            ("Product".into(), 100.),
            ("Ops".into(), 60.),
        ];
        let messages = vec![
            CircosLink {
                source: 0,
                from: (0., 40.),
                target: 1,
                to: (0., 30.),
                weight: 40.,
            },
            CircosLink {
                source: 0,
                from: (40., 90.),
                target: 2,
                to: (0., 50.),
                weight: 50.,
            },
            CircosLink {
                source: 1,
                from: (30., 70.),
                target: 2,
                to: (50., 90.),
                weight: 40.,
            },
            CircosLink {
                source: 3,
                from: (0., 50.),
                target: 0,
                to: (90., 120.),
                weight: 30.,
            },
        ];
        rsx! { <Circos segments={segments} links={messages} class="h-96" /> }
    }

    #[component]
    fn single_track() {
        let segments = chromosomes(16);
        let tracks = vec![track(&segments, 12, 10)];
        rsx! { <Circos segments={segments} links={links(20, 16, 11, 0.)} tracks={tracks} gap={1.} class="h-[480px]" /> }
    }

    #[component]
    fn dense() {
        rsx! { <Circos segments={chromosomes(22)} links={links(120, 22, 12, 0.)} link_opacity={0.25} gap={0.8} class="h-[480px]" /> }
    }

    #[component]
    fn small() {
        rsx! { <Circos segments={chromosomes(6)} links={links(10, 6, 13, 4.)} class="h-64" /> }
    }

    examples! {
        genome: "Genome view" => "Twelve chromosomes, two value tracks and links between regions; hover anything.",
        ideogram: "Segments only" => "The outer ring with ticks every 25 units.",
        thin_links: "Point links" => "Zero-width links become curves.",
        ribbons: "Wide ribbons" => "Links that cover ranges become ribbons.",
        tracks_only: "Three tracks" => "Bar tracks without links.",
        wide_gaps: "Wide gaps" => "gap 10°.",
        departments: "Messages between teams" => "Any segments, not only genomes.",
        single_track: "One track" => "Sixteen segments with a 1° gap.",
        dense: "Dense links" => "120 faint links.",
        small: "Small" => "Labels and rings shrink to fit.",
    }
}

pub mod warped_waffle {
    use super::*;

    fn energy() -> Vec<Item> {
        listed(&[
            ("Solar", 31.),
            ("Wind", 24.),
            ("Hydro", 17.),
            ("Gas", 19.),
            ("Coal", 9.),
        ])
    }

    #[component]
    fn ring() {
        let data = use_signal(energy());
        rsx! { <WarpedWaffle data={data} name={|i: &Item| i.label.as_ref()} value={|i: &Item| i.value} class="h-80" /> }
    }

    #[component]
    fn solid() {
        let data = use_signal(energy());
        rsx! { <WarpedWaffle data={data} name={|i: &Item| i.label.as_ref()} value={|i: &Item| i.value} hole={0.} class="h-80" /> }
    }

    #[component]
    fn thin() {
        let data = use_signal(energy());
        rsx! { <WarpedWaffle data={data} name={|i: &Item| i.label.as_ref()} value={|i: &Item| i.value} rings={2} sectors={50} hole={0.6} class="h-80" /> }
    }

    #[component]
    fn no_gaps() {
        let data = use_signal(energy());
        rsx! { <WarpedWaffle data={data} name={|i: &Item| i.label.as_ref()} value={|i: &Item| i.value} gap={0.} class="h-80" /> }
    }

    #[component]
    fn survey() {
        let data = use_signal(listed(&[("Yes", 62.), ("No", 31.), ("Undecided", 7.)]));
        rsx! { <WarpedWaffle data={data} name={|i: &Item| i.label.as_ref()} value={|i: &Item| i.value} class="h-80" /> }
    }

    #[component]
    fn coarse() {
        let data = use_signal(energy());
        rsx! { <WarpedWaffle data={data} name={|i: &Item| i.label.as_ref()} value={|i: &Item| i.value} rings={3} sectors={8} class="h-80" /> }
    }

    #[component]
    fn fine() {
        let data = use_signal(energy());
        rsx! { <WarpedWaffle data={data} name={|i: &Item| i.label.as_ref()} value={|i: &Item| i.value} rings={10} sectors={40} gap={0.2} class="h-80" /> }
    }

    #[component]
    fn legend_right() {
        let data = use_signal(listed(&[
            ("Done", 46.),
            ("In progress", 21.),
            ("Blocked", 4.),
            ("To do", 29.),
        ]));
        rsx! { <WarpedWaffle data={data} name={|i: &Item| i.label.as_ref()} value={|i: &Item| i.value} legend={Legend::list().right()} class="h-80" /> }
    }

    #[component]
    fn many() {
        let data = use_signal(items("Team", 8, 3, 50.));
        rsx! { <WarpedWaffle data={data} name={|i: &Item| i.label.as_ref()} value={|i: &Item| i.value} class="h-80" /> }
    }

    #[component]
    fn single() {
        let data = use_signal(listed(&[("Used", 73.), ("Free", 27.)]));
        rsx! { <WarpedWaffle data={data} name={|i: &Item| i.label.as_ref()} value={|i: &Item| i.value} rings={4} sectors={25} class="h-72" /> }
    }

    examples! {
        ring: "Energy mix" => "A waffle bent into a ring: 100 cells over 5 rings and 20 sectors, filled around.",
        solid: "No hole" => "hole 0 fills to the centre.",
        thin: "Thin ring" => "Two rings of 50 sectors.",
        no_gaps: "No gaps" => "gap 0.",
        survey: "Survey" => "Three answers.",
        coarse: "Coarse cells" => "24 large cells.",
        fine: "Fine cells" => "400 small cells.",
        legend_right: "Legend on the right" => "Click a status to hide it; the rest re-share the cells.",
        many: "Eight teams" => "More categories.",
        single: "Used and free" => "Two parts.",
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Read {
    pub sequence: String,
}

pub fn reads(entries: &[&'static str]) -> Vec<Read> {
    entries
        .iter()
        .map(|sequence| Read {
            sequence: (*sequence).to_owned(),
        })
        .collect()
}

pub fn motif(consensus: &str, count: u64, mutation: f64, seed: u64) -> Vec<Read> {
    let letters = ['A', 'C', 'G', 'T'];
    (0..count)
        .map(|index| {
            let text: String = consensus
                .chars()
                .enumerate()
                .map(|(position, letter)| {
                    if noise(seed * 10_000 + index * 101 + position as u64) < mutation {
                        letters
                            [(noise(seed * 777 + index * 13 + position as u64) * 4.) as usize % 4]
                    } else {
                        letter
                    }
                })
                .collect();
            Read { sequence: text }
        })
        .collect()
}

pub mod sequence_logo {
    use super::*;

    #[component]
    fn tata_box() {
        let data = use_signal(motif("TATAAAAGGC", 60, 0.2, 1));
        rsx! { <SequenceLogo data={data} sequence={|r: &Read| r.sequence.as_str()} class="h-64" /> }
    }

    #[component]
    fn probability() {
        let data = use_signal(motif("TATAAAAGGC", 60, 0.2, 1));
        rsx! { <SequenceLogo data={data} sequence={|r: &Read| r.sequence.as_str()} units={LogoUnits::Probability} class="h-64" /> }
    }

    #[component]
    fn conserved() {
        let data = use_signal(motif("GATTACA", 40, 0.03, 2));
        rsx! { <SequenceLogo data={data} sequence={|r: &Read| r.sequence.as_str()} class="h-64" /> }
    }

    #[component]
    fn variable() {
        let data = use_signal(motif("ACGTACGTACGT", 50, 0.55, 3));
        rsx! { <SequenceLogo data={data} sequence={|r: &Read| r.sequence.as_str()} class="h-64" /> }
    }

    #[component]
    fn rna() {
        let data = use_signal(reads(&[
            "AUGGCU", "AUGGCC", "AUGGCA", "AUGAAU", "AUGGCU", "AUGCCU",
        ]));
        rsx! { <SequenceLogo data={data} sequence={|r: &Read| r.sequence.as_str()} class="h-64" /> }
    }

    #[component]
    fn protein() {
        let data = use_signal(reads(&[
            "MKVLAAGIVG",
            "MKVLSAGIVG",
            "MRVLAAGLVG",
            "MKILAAGIVA",
            "MKVLTAGIVG",
            "MKVLAEGIVG",
        ]));
        rsx! { <SequenceLogo data={data} sequence={|r: &Read| r.sequence.as_str()} class="h-64" /> }
    }

    #[component]
    fn numbered() {
        let data = use_signal(motif("CCAAT", 30, 0.15, 4));
        rsx! { <SequenceLogo data={data} sequence={|r: &Read| r.sequence.as_str()} start={-80} class="h-56" /> }
    }

    #[component]
    fn uncorrected() {
        let data = use_signal(motif("GGGCGG", 5, 0.1, 5));
        rsx! { <SequenceLogo data={data} sequence={|r: &Read| r.sequence.as_str()} correction={false} class="h-56" /> }
    }

    #[component]
    fn long() {
        let data = use_signal(motif("ATGCGTACGTTAGCCGATCGATTACGGCATGCA", 80, 0.25, 6));
        rsx! { <SequenceLogo data={data} sequence={|r: &Read| r.sequence.as_str()} class="h-64" /> }
    }

    #[component]
    fn gaps() {
        let data = use_signal(reads(&[
            "AC-GT", "ACTGT", "AC-GA", "ACTGT", "AC-GT", "GCTGT",
        ]));
        rsx! { <SequenceLogo data={data} sequence={|r: &Read| r.sequence.as_str()} alphabet={LogoAlphabet::Dna} class="h-56" /> }
    }

    examples! {
        tata_box: "TATA box" => "Letter height is frequency × information; a column carries up to 2 bits. Hover a letter.",
        probability: "Probabilities" => "units Probability stacks raw frequencies to 1.",
        conserved: "Conserved motif" => "Almost every column is near 2 bits.",
        variable: "Variable sites" => "Heavy mutation leaves little information.",
        rna: "RNA" => "U replaces T; the start codon stands out.",
        protein: "Protein" => "Amino acids coloured by chemistry, up to 4.3 bits.",
        numbered: "Custom numbering" => "start −80 numbers positions from upstream.",
        uncorrected: "No small-sample correction" => "correction false with five reads.",
        long: "Long alignment" => "Position labels thin out to fit.",
        gaps: "Gaps" => "Gap characters are skipped in the counts.",
    }
}

#[derive(Clone, PartialEq)]
pub struct Term {
    pub word: &'static str,
    pub weight: f64,
    pub topic: &'static str,
}

pub fn terms(entries: &[(&'static str, f64, &'static str)]) -> Vec<Term> {
    entries
        .iter()
        .map(|(word, weight, topic)| Term {
            word,
            weight: *weight,
            topic,
        })
        .collect()
}

pub fn rust_terms() -> Vec<Term> {
    terms(&[
        ("rust", 120., "Language"),
        ("ownership", 80., "Language"),
        ("borrow", 74., "Language"),
        ("lifetime", 60., "Language"),
        ("trait", 66., "Language"),
        ("async", 58., "Runtime"),
        ("tokio", 44., "Runtime"),
        ("future", 40., "Runtime"),
        ("cargo", 70., "Tooling"),
        ("clippy", 38., "Tooling"),
        ("rustfmt", 30., "Tooling"),
        ("crate", 52., "Tooling"),
        ("unsafe", 34., "Language"),
        ("macro", 46., "Language"),
        ("generic", 42., "Language"),
        ("closure", 36., "Language"),
        ("iterator", 48., "Language"),
        ("enum", 39., "Language"),
        ("match", 33., "Language"),
        ("serde", 41., "Ecosystem"),
        ("gpui", 37., "Ecosystem"),
        ("wasm", 29., "Ecosystem"),
        ("embedded", 25., "Ecosystem"),
        ("rayon", 22., "Runtime"),
        ("channel", 21., "Runtime"),
        ("mutex", 20., "Runtime"),
        ("arc", 18., "Runtime"),
        ("test", 26., "Tooling"),
        ("bench", 15., "Tooling"),
        ("doc", 17., "Tooling"),
    ])
}

pub mod word_cloud {
    use super::*;

    #[component]
    fn topics() {
        let data = use_signal(rust_terms());
        rsx! { <WordCloud data={data} word={|t: &Term| t.word} weight={|t: &Term| t.weight} group={|t: &Term| t.topic} class="h-80" /> }
    }

    #[component]
    fn ungrouped() {
        let data = use_signal(rust_terms());
        rsx! { <WordCloud data={data} word={|t: &Term| t.word} weight={|t: &Term| t.weight} class="h-80" /> }
    }

    #[component]
    fn top_ten() {
        let data = use_signal(rust_terms());
        rsx! { <WordCloud data={data} word={|t: &Term| t.word} weight={|t: &Term| t.weight} max_words={10} class="h-72" /> }
    }

    #[component]
    fn flat_sizes() {
        let data = use_signal(rust_terms());
        rsx! { <WordCloud data={data} word={|t: &Term| t.word} weight={|t: &Term| t.weight} font_range={(14., 22.)} class="h-72" /> }
    }

    #[component]
    fn dramatic() {
        let data = use_signal(rust_terms());
        rsx! { <WordCloud data={data} word={|t: &Term| t.word} weight={|t: &Term| t.weight} font_range={(9., 80.)} class="h-96" /> }
    }

    #[component]
    fn legend_right() {
        let data = use_signal(rust_terms());
        rsx! { <WordCloud data={data} word={|t: &Term| t.word} weight={|t: &Term| t.weight} group={|t: &Term| t.topic} legend={Legend::list().right()} class="h-80" /> }
    }

    #[component]
    fn feedback() {
        let data = use_signal(terms(&[
            ("fast", 90., "Positive"),
            ("easy", 70., "Positive"),
            ("reliable", 55., "Positive"),
            ("pricey", 60., "Negative"),
            ("slow support", 40., "Negative"),
            ("confusing", 35., "Negative"),
            ("beautiful", 50., "Positive"),
            ("buggy", 25., "Negative"),
            ("powerful", 45., "Positive"),
            ("missing export", 20., "Negative"),
        ]));
        rsx! { <WordCloud data={data} word={|t: &Term| t.word} weight={|t: &Term| t.weight} group={|t: &Term| t.topic} class="h-72" /> }
    }

    #[component]
    fn many() {
        let words = [
            "alpha", "beta", "gamma", "delta", "epsilon", "zeta", "eta", "theta", "iota", "kappa",
            "lambda", "mu", "nu", "xi", "omicron", "pi", "rho", "sigma", "tau", "upsilon", "phi",
            "chi", "psi", "omega",
        ];
        let data = use_signal(
            (0..96u64)
                .map(|index| Term {
                    word: words[(index % 24) as usize],
                    weight: 1. + noise(index) * 99.,
                    topic: ["A", "B", "C"][(index % 3) as usize],
                })
                .collect::<Vec<_>>(),
        );
        rsx! { <WordCloud data={data} word={|t: &Term| t.word} weight={|t: &Term| t.weight} class="h-96" /> }
    }

    #[component]
    fn wide() {
        let data = use_signal(rust_terms());
        rsx! { <WordCloud data={data} word={|t: &Term| t.word} weight={|t: &Term| t.weight} group={|t: &Term| t.topic} class="h-48" /> }
    }

    #[component]
    fn tall() {
        let data = use_signal(rust_terms());
        rsx! { <div class="w-80 h-96"><WordCloud data={data} word={|t: &Term| t.word} weight={|t: &Term| t.weight} class="size-full" /></div> }
    }

    examples! {
        topics: "Rust topics" => "Size grows with weight and colour shows the topic; hover a word.",
        ungrouped: "Ungrouped" => "Without a group, words cycle through the palette.",
        top_ten: "Top ten" => "max_words 10.",
        flat_sizes: "Narrow size range" => "font_range 14–22.",
        dramatic: "Wide size range" => "font_range 9–80.",
        legend_right: "Legend on the right" => "Click a topic to hide its words.",
        feedback: "Review feedback" => "Positive and negative phrases.",
        many: "Repeated words" => "Each row is placed separately.",
        wide: "Wide and short" => "The spiral stretches to the shape.",
        tall: "Narrow" => "A narrow container.",
    }
}
