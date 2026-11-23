# Hybrid charts: examples

Every example here also runs in the gallery (`cargo run --example gallery`), under **Charts** in the sidebar.

The examples share these imports and this sample data:

```rust
use blenderui::prelude::*;
use bui_charts::*;

use crate::finance::noise;
```

## `<SankeyMatrix>`

This section also uses:

```rust
#[derive(Clone, PartialEq)]
pub struct Flow {
    from: &'static str,
    to: &'static str,
    value: f64,
}

fn flows(entries: &[(&'static str, &'static str, f64)]) -> Vec<Flow> {
    entries
        .iter()
        .map(|(from, to, value)| Flow {
            from,
            to,
            value: *value,
        })
        .collect()
}

fn migration() -> Vec<Flow> {
    flows(&[
        ("Europe", "Europe", 41.),
        ("Europe", "N. America", 18.),
        ("Europe", "Asia", 9.),
        ("Asia", "Europe", 14.),
        ("Asia", "N. America", 27.),
        ("Asia", "Asia", 66.),
        ("Asia", "Oceania", 8.),
        ("Africa", "Europe", 22.),
        ("Africa", "N. America", 7.),
        ("Africa", "Asia", 5.),
        ("Africa", "Africa", 31.),
        ("S. America", "N. America", 25.),
        ("S. America", "Europe", 11.),
        ("S. America", "S. America", 28.),
    ])
}

fn spend() -> Vec<Flow> {
    flows(&[
        ("Engineering", "Salaries", 520.),
        ("Engineering", "Cloud", 190.),
        ("Engineering", "Tools", 45.),
        ("Sales", "Salaries", 310.),
        ("Sales", "Travel", 120.),
        ("Sales", "Tools", 38.),
        ("Support", "Salaries", 240.),
        ("Support", "Tools", 52.),
        ("Marketing", "Salaries", 150.),
        ("Marketing", "Ads", 260.),
        ("Marketing", "Travel", 40.),
        ("Operations", "Salaries", 180.),
        ("Operations", "Cloud", 70.),
        ("Operations", "Travel", 25.),
    ])
}

fn traffic() -> Vec<Flow> {
    flows(&[
        ("Search", "Home", 1200.),
        ("Search", "Pricing", 640.),
        ("Search", "Blog", 2100.),
        ("Social", "Home", 480.),
        ("Social", "Blog", 1500.),
        ("Direct", "Home", 2400.),
        ("Direct", "Pricing", 380.),
        ("Direct", "Docs", 900.),
        ("Email", "Pricing", 520.),
        ("Email", "Docs", 260.),
        ("Referral", "Docs", 700.),
        ("Referral", "Blog", 330.),
    ])
}

fn grades() -> Vec<Flow> {
    let courses = ["Maths", "Physics", "Biology", "History", "Art"];
    let marks = ["A", "B", "C", "D", "F"];
    let mut rows = Vec::new();
    for (course, name) in courses.iter().enumerate() {
        for (mark, label) in marks.iter().enumerate() {
            let bias = (mark as f64 - course as f64 * 0.5).abs();
            rows.push(Flow {
                from: name,
                to: label,
                value: (3. + noise((course * 7 + mark) as u64) * 30. - bias * 3.)
                    .round()
                    .max(1.),
            });
        }
    }
    rows
}

fn energy() -> Vec<Flow> {
    flows(&[
        ("Coal", "Industry", 120.),
        ("Coal", "Power", 340.),
        ("Gas", "Homes", 210.),
        ("Gas", "Industry", 150.),
        ("Gas", "Power", 260.),
        ("Oil", "Transport", 520.),
        ("Oil", "Industry", 90.),
        ("Wind", "Power", 130.),
        ("Solar", "Power", 85.),
        ("Solar", "Homes", 30.),
        ("Nuclear", "Power", 160.),
    ])
}

fn wide() -> Vec<Flow> {
    let to = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let from = ["Alpha", "Beta", "Gamma", "Delta"];
    let mut rows = Vec::new();
    for (row, name) in from.iter().enumerate() {
        for (column, month) in to.iter().enumerate() {
            rows.push(Flow {
                from: name,
                to: month,
                value: (10. + noise((row * 31 + column) as u64) * 90.).round(),
            });
        }
    }
    rows
}

fn sparse() -> Vec<Flow> {
    flows(&[
        ("A", "X", 40.),
        ("B", "Y", 25.),
        ("C", "Z", 60.),
        ("A", "Z", 10.),
        ("C", "X", 5.),
    ])
}

fn pair() -> Vec<Flow> {
    flows(&[
        ("Free", "Stay", 820.),
        ("Free", "Upgrade", 140.),
        ("Free", "Churn", 310.),
        ("Paid", "Stay", 1900.),
        ("Paid", "Churn", 120.),
    ])
}
```

### Migration

Rows are origins, columns destinations. The band leaving each row thins as its flow is spent.

```rust
#[component]
fn migration_flows() {
    let data = use_signal(migration());
    rsx! { <SankeyMatrix data={data} source={|f: &Flow| f.from} target={|f: &Flow| f.to} value={|f: &Flow| f.value} class="h-96" /> }
}
```

### Department spend

Square area is the amount; hover for the shares of row, column and total.

```rust
#[component]
fn department_spend() {
    let data = use_signal(spend());
    rsx! { <SankeyMatrix data={data} source={|f: &Flow| f.from} target={|f: &Flow| f.to} value={|f: &Flow| f.value} class="h-96" /> }
}
```

### Site traffic

Visits by source and landing page. Bars above the columns are page totals.

```rust
#[component]
fn site_traffic() {
    let data = use_signal(traffic());
    rsx! { <SankeyMatrix data={data} source={|f: &Flow| f.from} target={|f: &Flow| f.to} value={|f: &Flow| f.value} class="h-80" /> }
}
```

### Course grades

Students per mark in each course.

```rust
#[component]
fn course_grades() {
    let data = use_signal(grades());
    rsx! { <SankeyMatrix data={data} source={|f: &Flow| f.from} target={|f: &Flow| f.to} value={|f: &Flow| f.value} class="h-80" /> }
}
```

### Energy use

Fuel into sector, in terawatt-hours.

```rust
#[component]
fn energy_use() {
    let data = use_signal(energy());
    rsx! { <SankeyMatrix data={data} source={|f: &Flow| f.from} target={|f: &Flow| f.to} value={|f: &Flow| f.value} class="h-80" /> }
}
```

### Without values

show_values off leaves only the squares.

```rust
#[component]
fn plain() {
    let data = use_signal(migration());
    rsx! { <SankeyMatrix data={data} source={|f: &Flow| f.from} target={|f: &Flow| f.to} value={|f: &Flow| f.value} show_values={false} class="h-96" /> }
}
```

### Legend on the right

Click a row in the legend to hide it; the matrix re-scales.

```rust
#[component]
fn legend_right() {
    let data = use_signal(spend());
    rsx! { <SankeyMatrix data={data} source={|f: &Flow| f.from} target={|f: &Flow| f.to} value={|f: &Flow| f.value} legend={Legend::list().right()} class="h-96" /> }
}
```

### No legend

Row names are already on the left.

```rust
#[component]
fn no_legend() {
    let data = use_signal(traffic());
    rsx! { <SankeyMatrix data={data} source={|f: &Flow| f.from} target={|f: &Flow| f.to} value={|f: &Flow| f.value} legend={Legend::hidden()} class="h-72" /> }
}
```

### Twelve columns

Wide matrices keep their squares readable.

```rust
#[component]
fn twelve_columns() {
    let data = use_signal(wide());
    rsx! { <SankeyMatrix data={data} source={|f: &Flow| f.from} target={|f: &Flow| f.to} value={|f: &Flow| f.value} show_values={false} class="h-72" /> }
}
```

### Sparse

Empty cells stay empty.

```rust
#[component]
fn sparse_matrix() {
    let data = use_signal(sparse());
    rsx! { <SankeyMatrix data={data} source={|f: &Flow| f.from} target={|f: &Flow| f.to} value={|f: &Flow| f.value} class="h-72" /> }
}
```

### Two rows

Even a pair of rows reads as a Sankey.

```rust
#[component]
fn two_plans() {
    let data = use_signal(pair());
    rsx! { <SankeyMatrix data={data} source={|f: &Flow| f.from} target={|f: &Flow| f.to} value={|f: &Flow| f.value} class="h-60" /> }
}
```

## `<TreemapForce>`

This section also uses:

```rust
#[derive(Clone, PartialEq)]
pub struct Node {
    name: SharedString,
    group: &'static str,
    value: f64,
}

fn nodes(groups: &[(&'static str, &[(&str, f64)])]) -> Vec<Node> {
    groups
        .iter()
        .flat_map(|(group, members)| {
            members.iter().map(move |(name, value)| Node {
                name: SharedString::from((*name).to_owned()),
                group,
                value: *value,
            })
        })
        .collect()
}

fn wired(count: usize, edges: u64, seed: u64) -> Vec<ForceLink> {
    (0..edges)
        .map(|edge| {
            let pick = |offset: u64| {
                ((noise(seed * 977 + edge * 2 + offset) * count as f64) as usize).min(count - 1)
            };
            ForceLink {
                source: pick(0),
                target: pick(1),
                weight: 1. + noise(seed * 31 + edge) * 9.,
            }
        })
        .collect()
}

fn services() -> Vec<Node> {
    nodes(&[
        (
            "Platform",
            &[
                ("auth", 8.),
                ("gateway", 12.),
                ("queue", 6.),
                ("config", 3.),
            ],
        ),
        (
            "Payments",
            &[
                ("billing", 14.),
                ("ledger", 9.),
                ("fraud", 7.),
                ("refunds", 4.),
            ],
        ),
        (
            "Catalog",
            &[
                ("search", 11.),
                ("items", 8.),
                ("pricing", 5.),
                ("images", 6.),
            ],
        ),
        ("Data", &[("etl", 7.), ("warehouse", 10.), ("reports", 4.)]),
    ])
}

fn service_links() -> Vec<ForceLink> {
    let pairs = [
        (1, 0),
        (4, 5),
        (4, 6),
        (5, 6),
        (7, 5),
        (8, 9),
        (8, 10),
        (9, 11),
        (12, 13),
        (13, 14),
        (4, 1),
        (8, 4),
        (9, 13),
        (3, 2),
        (1, 8),
        (12, 5),
        (1, 4),
        (0, 4),
    ];
    pairs
        .iter()
        .map(|(source, target)| ForceLink {
            source: *source,
            target: *target,
            weight: 3.,
        })
        .collect()
}

fn modules() -> Vec<Node> {
    nodes(&[
        (
            "core",
            &[("alloc", 30.), ("hash", 12.), ("iter", 22.), ("fmt", 18.)],
        ),
        ("net", &[("http", 28.), ("tls", 16.), ("dns", 8.)]),
        ("fs", &[("path", 9.), ("watch", 6.), ("walk", 11.)]),
        ("cli", &[("args", 7.), ("shell", 5.)]),
    ])
}

fn people() -> Vec<Node> {
    nodes(&[
        (
            "Design",
            &[("Ana", 5.), ("Bo", 3.), ("Chi", 4.), ("Dee", 2.)],
        ),
        (
            "Build",
            &[
                ("Eli", 6.),
                ("Fay", 5.),
                ("Gus", 4.),
                ("Hal", 5.),
                ("Ivy", 3.),
            ],
        ),
        ("Ship", &[("Jun", 4.), ("Kai", 3.), ("Lou", 2.)]),
    ])
}

fn many() -> Vec<Node> {
    let groups = ["A", "B", "C", "D", "E"];
    (0..70)
        .map(|index| Node {
            name: format!("n{index}").into(),
            group: groups[(noise(index * 3) * 5.) as usize % 5],
            value: 1. + noise(index * 5 + 1) * 9.,
        })
        .collect()
}

fn two() -> Vec<Node> {
    nodes(&[
        (
            "Frontend",
            &[("web", 10.), ("ios", 6.), ("android", 6.), ("ui kit", 4.)],
        ),
        ("Backend", &[("api", 12.), ("jobs", 5.), ("db", 8.)]),
    ])
}

macro_rules! treemap {
    ($name:ident, $nodes:expr, $links:expr, $class:literal $(, $prop:ident = $value:expr)*) => {
        #[component]
        fn $name() {
            let data = use_signal($nodes);
            rsx! { <TreemapForce data={data} name={|n: &Node| n.name.as_ref()} group={|n: &Node| n.group} value={|n: &Node| n.value} links={$links} class=$class $($prop={$value})* /> }
        }
    };
}

treemap!(microservices, services(), service_links(), "h-96");
treemap!(unlinked, services(), Vec::<ForceLink>::new(), "h-96");
treemap!(
    without_labels,
    services(),
    service_links(),
    "h-96",
    show_labels = false
);
treemap!(loose, services(), service_links(), "h-96", distance = 40.);
treemap!(tight, services(), service_links(), "h-96", distance = 2.);
treemap!(
    legend_right,
    services(),
    service_links(),
    "h-96",
    legend = Legend::list().right()
);
treemap!(
    no_legend,
    services(),
    service_links(),
    "h-96",
    legend = Legend::hidden()
);
treemap!(crates, modules(), wired(9, 14, 4), "h-80");
treemap!(teams, people(), wired(12, 22, 9), "h-80");
treemap!(crowd, many(), wired(70, 90, 2), "h-96", show_labels = false);
treemap!(quick, services(), service_links(), "h-96", iterations = 20);
treemap!(pair, two(), wired(7, 9, 6), "h-72");

examples! {
    Microservices: "Microservices" => "Each team is a treemap cell sized by its nodes' values; calls are pulled across cells.",
    Unlinked: "No links" => "Without links, nodes just pack into their cells.",
    WithoutLabels: "Without labels" => "show_labels off keeps only the circles.",
    Loose: "Loose" => "distance 40 holds linked nodes further apart.",
    Tight: "Tight" => "distance 2 pulls linked nodes together.",
    LegendRight: "Legend on the right" => "Click a team to hide it; the remaining cells grow.",
    NoLegend: "No legend" => "Cell titles already name the groups.",
    Crates: "Modules" => "Random weighted links between module nodes.",
    Teams: "Teams" => "Who works with whom across three teams.",
    Crowd: "Seventy nodes" => "Many small nodes still settle without overlapping.",
    Quick: "Few iterations" => "20 iterations leave the layout half-settled.",
    Pair: "Two groups" => "The smallest useful treemap.",
}
```

### Microservices

Each team is a treemap cell sized by its nodes' values; calls are pulled across cells.

```rust
treemap!(microservices, services(), service_links(), "h-96");
```

### No links

Without links, nodes just pack into their cells.

```rust
treemap!(unlinked, services(), Vec::<ForceLink>::new(), "h-96");
```

### Without labels

show_labels off keeps only the circles.

```rust
treemap!(
    without_labels,
    services(),
    service_links(),
    "h-96",
    show_labels = false
);
```

### Loose

distance 40 holds linked nodes further apart.

```rust
treemap!(loose, services(), service_links(), "h-96", distance = 40.);
```

### Tight

distance 2 pulls linked nodes together.

```rust
treemap!(tight, services(), service_links(), "h-96", distance = 2.);
```

### Legend on the right

Click a team to hide it; the remaining cells grow.

```rust
treemap!(
    legend_right,
    services(),
    service_links(),
    "h-96",
    legend = Legend::list().right()
);
```

### No legend

Cell titles already name the groups.

```rust
treemap!(
    no_legend,
    services(),
    service_links(),
    "h-96",
    legend = Legend::hidden()
);
```

### Modules

Random weighted links between module nodes.

```rust
treemap!(crates, modules(), wired(9, 14, 4), "h-80");
```

### Teams

Who works with whom across three teams.

```rust
treemap!(teams, people(), wired(12, 22, 9), "h-80");
```

### Seventy nodes

Many small nodes still settle without overlapping.

```rust
treemap!(crowd, many(), wired(70, 90, 2), "h-96", show_labels = false);
```

### Few iterations

20 iterations leave the layout half-settled.

```rust
treemap!(quick, services(), service_links(), "h-96", iterations = 20);
```

### Two groups

The smallest useful treemap.

```rust
treemap!(pair, two(), wired(7, 9, 6), "h-72");
```
