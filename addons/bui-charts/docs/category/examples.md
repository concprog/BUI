# Category charts: examples

Every example here also runs in the gallery (`cargo run --example gallery`), under **Charts** in the sidebar.

The examples share these imports and this sample data:

```rust
use blenderui::prelude::*;
use bui_charts::*;

use crate::finance::noise;

#[derive(Clone, PartialEq)]
struct Country {
    pub name: &'static str,
    pub before: f64,
    pub after: f64,
}

fn countries() -> Vec<Country> {
    [
        ("Norway", 81.6, 83.2),
        ("Japan", 84.1, 84.5),
        ("Brazil", 74.2, 72.8),
        ("India", 68.9, 70.8),
        ("Kenya", 61.2, 66.1),
        ("Chile", 79.4, 80.2),
        ("Poland", 77.6, 76.4),
        ("Canada", 81.9, 82.6),
    ]
    .map(|(name, before, after)| Country {
        name,
        before,
        after,
    })
    .into()
}

#[derive(Clone, PartialEq)]
struct Rank {
    pub year: f64,
    pub team: &'static str,
    pub points: f64,
}

fn standings() -> Vec<Rank> {
    ranking(
        &["Falcons", "Rovers", "United", "Athletic", "City"],
        2019..=2025,
    )
}

fn ranking(teams: &[&'static str], years: std::ops::RangeInclusive<u32>) -> Vec<Rank> {
    years
        .flat_map(|year| {
            teams.iter().enumerate().map(move |(index, team)| Rank {
                year: f64::from(year),
                team,
                points: 40. + noise(u64::from(year) * 10 + index as u64) * 50.,
            })
        })
        .collect()
}

#[derive(Clone, PartialEq)]
struct Age {
    pub band: &'static str,
    pub men: f64,
    pub women: f64,
}

fn pyramid() -> Vec<Age> {
    [
        "85+", "75–84", "65–74", "55–64", "45–54", "35–44", "25–34", "15–24", "5–14", "0–4",
    ]
    .iter()
    .enumerate()
    .map(|(index, band)| {
        let base = 2.5 + index as f64 * 0.6;
        Age {
            band,
            men: base * (0.9 + noise(index as u64) * 0.2),
            women: base * (0.95 + noise(index as u64 + 40) * 0.2)
                + if index < 3 { 0.6 } else { 0. },
        }
    })
    .collect()
}

#[derive(Clone, PartialEq)]
struct Step {
    pub name: &'static str,
    pub value: f64,
}

fn steps(items: &[(&'static str, f64)]) -> Vec<Step> {
    items
        .iter()
        .map(|(name, value)| Step {
            name,
            value: *value,
        })
        .collect()
}

const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

fn monthly(base: f64, spread: f64, seed: u64) -> Vec<Step> {
    MONTHS
        .iter()
        .enumerate()
        .map(|(index, name)| Step {
            name,
            value: base + noise(seed + index as u64) * spread,
        })
        .collect()
}

fn regions() -> Vec<Step> {
    steps(&[
        ("Europe", 742.),
        ("Asia", 1_480.),
        ("Africa", 560.),
        ("North America", 620.),
        ("South America", 410.),
        ("Oceania", 95.),
        ("Middle East", 330.),
        ("Central Asia", 120.),
        ("Caribbean", 64.),
        ("Nordics", 88.),
    ])
}

fn languages() -> Vec<Step> {
    steps(&[
        ("JavaScript", 17.4),
        ("Python", 15.7),
        ("Java", 9.8),
        ("C#", 7.2),
        ("C++", 6.1),
        ("Go", 3.3),
        ("Rust", 2.2),
        ("Swift", 1.4),
    ])
}
```

## `<Lollipop>`

### Sorted

Life expectancy, sorted from highest.

```rust
#[component]
fn life() {
    let data = use_signal(countries());
    rsx! { <Lollipop data={data} name={|c: &Country| c.name} value={|c: &Country| c.after} sort={SortOrder::Descending} value_axis={AxisConfig::new().min(60.)} class="h-72" /> }
}
```

### Vertical, change

The change between two years, as columns, with values.

```rust
#[component]
fn vertical() {
    let data = use_signal(countries());
    rsx! { <Lollipop data={data} name={|c: &Country| c.name} value={|c: &Country| c.after - c.before} horizontal={false} color={rgb(0x8839ef)} show_values={true} class="h-72" /> }
}
```

### Few categories

Smaller dots for a short list.

```rust
#[component]
fn small() {
    let data = use_signal(steps(&[
        ("Mon", 12.),
        ("Tue", 18.),
        ("Wed", 9.),
        ("Thu", 22.),
        ("Fri", 15.),
    ]));
    rsx! { <Lollipop data={data} name={|s: &Step| s.name} value={|s: &Step| s.value} point_size={8.} class="h-56" /> }
}
```

### Ascending, with values

Population by region, smallest first, with each value printed.

```rust
#[component]
fn ascending() {
    let data = use_signal(regions());
    rsx! { <Lollipop data={data} name={|s: &Step| s.name} value={|s: &Step| s.value} sort={SortOrder::Ascending} show_values={true} class="h-80" /> }
}
```

### Large dots

Developers per language with oversized orange dots.

```rust
#[component]
fn large() {
    let data = use_signal(languages());
    rsx! { <Lollipop data={data} name={|s: &Step| s.name} value={|s: &Step| s.value} sort={SortOrder::Descending} point_size={16.} color={rgb(0xfe640b)} class="h-72" /> }
}
```

### No tooltip

Regional population in teal with the hover tooltip switched off.

```rust
#[component]
fn quiet() {
    let data = use_signal(regions());
    rsx! { <Lollipop data={data} name={|s: &Step| s.name} value={|s: &Step| s.value} color={rgb(0x179299)} tooltip={Tooltip::hidden()} class="h-80" /> }
}
```

### Axis title

A titled value axis on a short ranking.

```rust
#[component]
fn labelled() {
    let data = use_signal(languages());
    rsx! { <Lollipop data={data} name={|s: &Step| s.name} value={|s: &Step| s.value} value_axis={AxisConfig::new().label("Developers (millions)")} class="h-72" /> }
}
```

### Below zero

Monthly temperature anomaly with stems running below the baseline.

```rust
#[component]
fn anomaly() {
    let data = use_signal(monthly(-2., 4., 11));
    rsx! { <Lollipop data={data} name={|s: &Step| s.name} value={|s: &Step| s.value} horizontal={false} color={rgb(0xd20f39)} class="h-72" /> }
}
```

### Twelve months

Monthly sales as columns, with values on top.

```rust
#[component]
fn monthly_sales() {
    let data = use_signal(monthly(30., 40., 7));
    rsx! { <Lollipop data={data} name={|s: &Step| s.name} value={|s: &Step| s.value} horizontal={false} show_values={true} class="h-72" /> }
}
```

### Thin dots

Small dots keep a dense monthly list light.

```rust
#[component]
fn thin() {
    let data = use_signal(monthly(10., 90., 21));
    rsx! { <Lollipop data={data} name={|s: &Step| s.name} value={|s: &Step| s.value} sort={SortOrder::Ascending} point_size={5.} class="h-80" /> }
}
```

## `<Dumbbell>`

### Before and after

Each country's life expectancy in 2010 and 2020.

```rust
#[component]
fn change() {
    let data = use_signal(countries());
    rsx! { <Dumbbell data={data} name={|c: &Country| c.name} start={|c: &Country| c.before} end={|c: &Country| c.after} start_label="2010" end_label="2020" class="h-72" /> }
}
```

### Sorted, custom colours

Sorted by the end value.

```rust
#[component]
fn sorted() {
    let data = use_signal(countries());
    rsx! { <Dumbbell data={data} name={|c: &Country| c.name} start={|c: &Country| c.before} end={|c: &Country| c.after} sort={SortOrder::Descending} start_color={rgb(0x9ca0b0)} end_color={rgb(0x1e66f5)} class="h-72" /> }
}
```

### Vertical

Plan against actual per quarter.

```rust
#[component]
fn columns() {
    let data = use_signal(steps(&[("Q1", 40.), ("Q2", 55.), ("Q3", 48.), ("Q4", 70.)]));
    rsx! { <Dumbbell data={data} name={|s: &Step| s.name} start={|s: &Step| s.value * 0.7} end={|s: &Step| s.value} start_label="Plan" end_label="Actual" horizontal={false} class="h-72" /> }
}
```

### Legend on the right

Regional population in 1990 and 2020 with the legend at the side.

```rust
#[component]
fn legend_right() {
    let data = use_signal(regions());
    rsx! { <Dumbbell data={data} name={|s: &Step| s.name} start={|s: &Step| s.value * 0.6} end={|s: &Step| s.value} start_label="1990" end_label="2020" legend={Legend::list().right()} class="h-80" /> }
}
```

### No legend

Developer counts with the legend removed.

```rust
#[component]
fn no_legend() {
    let data = use_signal(languages());
    rsx! { <Dumbbell data={data} name={|s: &Step| s.name} start={|s: &Step| s.value * 1.25} end={|s: &Step| s.value} legend={Legend::hidden()} class="h-72" /> }
}
```

### Large points

Oversized amber and green ends make the gap obvious.

```rust
#[component]
fn large_points() {
    let data = use_signal(countries());
    rsx! { <Dumbbell data={data} name={|c: &Country| c.name} start={|c: &Country| c.before} end={|c: &Country| c.after} point_size={14.} start_color={rgb(0xdf8e1d)} end_color={rgb(0x40a02b)} class="h-80" /> }
}
```

### No tooltip

Life expectancy change with the hover tooltip off.

```rust
#[component]
fn quiet() {
    let data = use_signal(countries());
    rsx! { <Dumbbell data={data} name={|c: &Country| c.name} start={|c: &Country| c.before} end={|c: &Country| c.after} tooltip={Tooltip::hidden()} class="h-72" /> }
}
```

### Monthly budget

Budget against spend for twelve months, as columns.

```rust
#[component]
fn monthly_plan() {
    let data = use_signal(monthly(40., 40., 5));
    rsx! { <Dumbbell data={data} name={|s: &Step| s.name} start={|s: &Step| s.value * 0.8} end={|s: &Step| s.value} start_label="Budget" end_label="Spend" horizontal={false} point_size={7.} class="h-72" /> }
}
```

### Ascending, titled axis

Regions from smallest to largest with a value axis title.

```rust
#[component]
fn ascending() {
    let data = use_signal(regions());
    rsx! { <Dumbbell data={data} name={|s: &Step| s.name} start={|s: &Step| s.value * 0.85} end={|s: &Step| s.value} start_label="Q1" end_label="Q4" sort={SortOrder::Ascending} value_axis={AxisConfig::new().label("Orders (thousands)")} class="h-80" /> }
}
```

### Price range

Low and high price per product, sorted by the high end.

```rust
#[component]
fn price_range() {
    let data = use_signal(languages());
    rsx! { <Dumbbell data={data} name={|s: &Step| s.name} start={|s: &Step| s.value * 2.} end={|s: &Step| s.value * 3.5} start_label="Low" end_label="High" sort={SortOrder::Descending} start_color={rgb(0x04a5e5)} end_color={rgb(0xea76cb)} value_axis={AxisConfig::new().min(0.)} class="h-72" /> }
}
```

## `<Slope>`

### Change by country

Rising lines in green, falling in red.

```rust
#[component]
fn countries_slope() {
    let data = use_signal(countries());
    rsx! { <Slope data={data} name={|c: &Country| c.name} before={|c: &Country| c.before} after={|c: &Country| c.after} before_label="2010" after_label="2020" class="h-80" /> }
}
```

### Palette colours

One colour per item, no value labels.

```rust
#[component]
fn palette() {
    let data = use_signal(countries());
    rsx! { <Slope data={data} name={|c: &Country| c.name} before={|c: &Country| c.before} after={|c: &Country| c.after} color_by_change={false} show_values={false} class="h-80" /> }
}
```

### Channels

Revenue share by channel, last year and this year.

```rust
#[component]
fn few() {
    let data = use_signal(steps(&[("Web", 42.), ("Mobile", 31.), ("Partners", 18.)]));
    rsx! { <Slope data={data} name={|s: &Step| s.name} before={|s: &Step| s.value} after={|s: &Step| s.value * (0.6 + noise(s.value as u64) * 0.9)} before_label="Last year" after_label="This year" class="h-80" /> }
}
```

### No legend

The slopes without the legend at the right.

```rust
#[component]
fn no_legend() {
    let data = use_signal(countries());
    rsx! { <Slope data={data} name={|c: &Country| c.name} before={|c: &Country| c.before} after={|c: &Country| c.after} legend={Legend::hidden()} class="h-80" /> }
}
```

### Legend below

Palette colours with the legend moved under the plot.

```rust
#[component]
fn legend_bottom() {
    let data = use_signal(countries());
    rsx! { <Slope data={data} name={|c: &Country| c.name} before={|c: &Country| c.before} after={|c: &Country| c.after} color_by_change={false} legend={Legend::list().bottom()} class="h-80" /> }
}
```

### Titled axis

A value axis starting at 60 with a title.

```rust
#[component]
fn titled() {
    let data = use_signal(countries());
    rsx! { <Slope data={data} name={|c: &Country| c.name} before={|c: &Country| c.before} after={|c: &Country| c.after} before_label="2010" after_label="2020" y_axis={AxisConfig::new().min(60.).label("Years")} class="h-80" /> }
}
```

### No tooltip, no labels

A bare slope chart with hover and value labels removed.

```rust
#[component]
fn quiet() {
    let data = use_signal(countries());
    rsx! { <Slope data={data} name={|c: &Country| c.name} before={|c: &Country| c.before} after={|c: &Country| c.after} show_values={false} tooltip={Tooltip::hidden()} class="h-80" /> }
}
```

### Ten regions

Population by region in 2000 and 2020.

```rust
#[component]
fn regional() {
    let data = use_signal(regions());
    rsx! { <Slope data={data} name={|s: &Step| s.name} before={|s: &Step| s.value} after={|s: &Step| s.value * (0.7 + noise(s.value as u64 + 3) * 0.7)} before_label="2000" after_label="2020" class="h-96" /> }
}
```

### Popularity shift

Language share ten years apart, with the legend hidden.

```rust
#[component]
fn languages_slope() {
    let data = use_signal(languages());
    rsx! { <Slope data={data} name={|s: &Step| s.name} before={|s: &Step| s.value} after={|s: &Step| s.value * (0.5 + noise(s.value as u64) * 1.2)} before_label="2015" after_label="2025" legend={Legend::hidden()} class="h-80" /> }
}
```

### Month by month

Twelve months compared across two years, in palette colours.

```rust
#[component]
fn monthly_pair() {
    let data = use_signal(monthly(20., 60., 9));
    rsx! { <Slope data={data} name={|s: &Step| s.name} before={|s: &Step| s.value} after={|s: &Step| s.value + 15. - noise(s.value as u64) * 30.} before_label="2024" after_label="2025" color_by_change={false} show_values={false} class="h-96" /> }
}
```

## `<Bump>`

### League table

Each team's rank per season, rank 1 at the top.

```rust
#[component]
fn league() {
    let data = use_signal(standings());
    rsx! { <Bump data={data} x={|r: &Rank| r.year} name={|r: &Rank| r.team} value={|r: &Rank| r.points} class="h-72" /> }
}
```

### Straight lines

Linear segments and the legend below.

```rust
#[component]
fn straight() {
    let data = use_signal(standings());
    rsx! { <Bump data={data} x={|r: &Rank| r.year} name={|r: &Rank| r.team} value={|r: &Rank| r.points} smooth={false} legend={Legend::list()} class="h-72" /> }
}
```

### Lowest first

Rank 1 is the lowest value, as for lap times.

```rust
#[component]
fn lowest() {
    let data = use_signal(standings());
    rsx! { <Bump data={data} x={|r: &Rank| r.year} name={|r: &Rank| r.team} value={|r: &Rank| r.points} highest_first={false} class="h-72" /> }
}
```

### No legend

The league table without a legend.

```rust
#[component]
fn no_legend() {
    let data = use_signal(standings());
    rsx! { <Bump data={data} x={|r: &Rank| r.year} name={|r: &Rank| r.team} value={|r: &Rank| r.points} legend={Legend::hidden()} class="h-72" /> }
}
```

### Legend below

The legend placed under the plot.

```rust
#[component]
fn legend_bottom() {
    let data = use_signal(standings());
    rsx! { <Bump data={data} x={|r: &Rank| r.year} name={|r: &Rank| r.team} value={|r: &Rank| r.points} legend={Legend::list().bottom()} class="h-72" /> }
}
```

### No tooltip

Rank lines with the hover tooltip off.

```rust
#[component]
fn quiet() {
    let data = use_signal(standings());
    rsx! { <Bump data={data} x={|r: &Rank| r.year} name={|r: &Rank| r.team} value={|r: &Rank| r.points} tooltip={Tooltip::hidden()} class="h-72" /> }
}
```

### Titled axis

An x axis titled Season.

```rust
#[component]
fn titled() {
    let data = use_signal(standings());
    rsx! { <Bump data={data} x={|r: &Rank| r.year} name={|r: &Rank| r.team} value={|r: &Rank| r.points} x_axis={AxisConfig::new().label("Season")} class="h-72" /> }
}
```

### Eight teams, twelve seasons

A busier table where the lines cross often.

```rust
#[component]
fn big_league() {
    let data = use_signal(ranking(
        &[
            "Lions", "Tigers", "Bears", "Wolves", "Eagles", "Hawks", "Sharks", "Giants",
        ],
        2014..=2025,
    ));
    rsx! { <Bump data={data} x={|r: &Rank| r.year} name={|r: &Rank| r.team} value={|r: &Rank| r.points} class="h-96" /> }
}
```

### Three contenders

A short run of straight segments for three teams.

```rust
#[component]
fn short() {
    let data = use_signal(ranking(&["Mercury", "Venus", "Terra"], 2022..=2025));
    rsx! { <Bump data={data} x={|r: &Rank| r.year} name={|r: &Rank| r.team} value={|r: &Rank| r.points} smooth={false} class="h-72" /> }
}
```

### Race results

Six drivers over twelve races with the lowest value ranked first.

```rust
#[component]
fn lap_times() {
    let data = use_signal(ranking(
        &[
            "Alonso",
            "Hamilton",
            "Verstappen",
            "Leclerc",
            "Norris",
            "Russell",
        ],
        2010..=2021,
    ));
    rsx! { <Bump data={data} x={|r: &Rank| r.year} name={|r: &Rank| r.team} value={|r: &Rank| r.points} highest_first={false} legend={Legend::list().bottom()} x_axis={AxisConfig::new().label("Race")} class="h-80" /> }
}
```

## `<BackToBack>`

### Population pyramid

Men to the left, women to the right, by age band.

```rust
#[component]
fn population() {
    let data = use_signal(pyramid());
    rsx! { <BackToBack data={data} name={|a: &Age| a.band} left={|a: &Age| a.men} right={|a: &Age| a.women} left_label="Men (%)" right_label="Women (%)" class="h-80" /> }
}
```

### Custom colours

The same pyramid in custom colours.

```rust
#[component]
fn colours() {
    let data = use_signal(pyramid());
    rsx! { <BackToBack data={data} name={|a: &Age| a.band} left={|a: &Age| a.men} right={|a: &Age| a.women} left_color={rgb(0x179299)} right_color={rgb(0xea76cb)} class="h-80" /> }
}
```

### Referendum

Yes against no by region.

```rust
#[component]
fn votes() {
    let data = use_signal(steps(&[
        ("North", 52.),
        ("South", 47.),
        ("East", 61.),
        ("West", 39.),
    ]));
    rsx! { <BackToBack data={data} name={|s: &Step| s.name} left={|s: &Step| s.value} right={|s: &Step| 100. - s.value} left_label="Yes" right_label="No" class="h-56" /> }
}
```

### Legend on the right

The pyramid with the legend at the side.

```rust
#[component]
fn legend_right() {
    let data = use_signal(pyramid());
    rsx! { <BackToBack data={data} name={|a: &Age| a.band} left={|a: &Age| a.men} right={|a: &Age| a.women} left_label="Men" right_label="Women" legend={Legend::list().right()} class="h-80" /> }
}
```

### No legend

The pyramid with the legend removed.

```rust
#[component]
fn no_legend() {
    let data = use_signal(pyramid());
    rsx! { <BackToBack data={data} name={|a: &Age| a.band} left={|a: &Age| a.men} right={|a: &Age| a.women} legend={Legend::hidden()} class="h-80" /> }
}
```

### No tooltip

An orange left side with the hover tooltip off.

```rust
#[component]
fn quiet() {
    let data = use_signal(pyramid());
    rsx! { <BackToBack data={data} name={|a: &Age| a.band} left={|a: &Age| a.men} right={|a: &Age| a.women} left_color={rgb(0xfe640b)} tooltip={Tooltip::hidden()} class="h-80" /> }
}
```

### Exports and imports

Trade volume per region, exports left and imports right.

```rust
#[component]
fn trade() {
    let data = use_signal(regions());
    rsx! { <BackToBack data={data} name={|s: &Step| s.name} left={|s: &Step| s.value} right={|s: &Step| s.value * 0.7} left_label="Exports" right_label="Imports" left_color={rgb(0x1e66f5)} right_color={rgb(0xdf8e1d)} class="h-80" /> }
}
```

### Cash flow

Twelve months of inflow against outflow.

```rust
#[component]
fn monthly_flow() {
    let data = use_signal(monthly(20., 50., 13));
    rsx! { <BackToBack data={data} name={|s: &Step| s.name} left={|s: &Step| s.value} right={|s: &Step| 80. - s.value * 0.6} left_label="Inflow" right_label="Outflow" class="h-96" /> }
}
```

### Survey agreement

Disagree against agree per topic with the legend at the side.

```rust
#[component]
fn survey() {
    let data = use_signal(steps(&[
        ("Pricing", 64.),
        ("Support", 82.),
        ("Reliability", 71.),
        ("Onboarding", 55.),
        ("Docs", 48.),
        ("Features", 77.),
    ]));
    rsx! { <BackToBack data={data} name={|s: &Step| s.name} left={|s: &Step| 100. - s.value} right={|s: &Step| s.value} left_label="Disagree" right_label="Agree" left_color={rgb(0xd20f39)} right_color={rgb(0x40a02b)} legend={Legend::list().right()} class="h-72" /> }
}
```

### Professional and learning

Developers per language split by experience.

```rust
#[component]
fn languages_split() {
    let data = use_signal(languages());
    rsx! { <BackToBack data={data} name={|s: &Step| s.name} left={|s: &Step| s.value} right={|s: &Step| s.value * (0.4 + noise(s.value as u64) * 0.8)} left_label="Professional" right_label="Learning" class="h-72" /> }
}
```

## `<Waterfall>`

### Profit bridge

Increases in green, decreases in red, the total in blue.

```rust
#[component]
fn profit() {
    let data = use_signal(steps(&[
        ("Revenue", 420.),
        ("Cost of sales", -180.),
        ("Marketing", -65.),
        ("Salaries", -110.),
        ("Other income", 25.),
        ("Tax", -22.),
    ]));
    rsx! { <Waterfall data={data} name={|s: &Step| s.name} value={|s: &Step| s.value} total_label="Profit" class="h-72" /> }
}
```

### Below zero

The running total dips below zero and recovers.

```rust
#[component]
fn negative() {
    let data = use_signal(steps(&[
        ("Start", 50.),
        ("Q1", -80.),
        ("Q2", -20.),
        ("Q3", 30.),
        ("Q4", 45.),
    ]));
    rsx! { <Waterfall data={data} name={|s: &Step| s.name} value={|s: &Step| s.value} total_label="End" class="h-72" /> }
}
```

### Horizontal

A budget burn-down as horizontal bars.

```rust
#[component]
fn horizontal() {
    let data = use_signal(steps(&[
        ("Budget", 100.),
        ("Design", -18.),
        ("Build", -42.),
        ("Test", -15.),
        ("Grant", 20.),
    ]));
    rsx! { <Waterfall data={data} name={|s: &Step| s.name} value={|s: &Step| s.value} horizontal={true} total_label="Left" class="h-72" /> }
}
```

### Monthly cash flow

Twelve monthly changes ending in a year-end total.

```rust
#[component]
fn yearly() {
    let data = use_signal(cash_flow());
    rsx! { <Waterfall data={data} name={|s: &Step| s.name} value={|s: &Step| s.value} total_label="Year end" class="h-80" /> }
}
```

### No total

The same monthly changes without a closing bar.

```rust
#[component]
fn open_ended() {
    let data = use_signal(cash_flow());
    rsx! { <Waterfall data={data} name={|s: &Step| s.name} value={|s: &Step| s.value} total={false} class="h-80" /> }
}
```

### Custom colours

Subscriber movement with teal gains, orange losses and a purple total.

```rust
#[component]
fn colours() {
    let data = use_signal(steps(&[
        ("Opening", 300.),
        ("New", 120.),
        ("Expansion", 70.),
        ("Churn", -90.),
        ("Downgrade", -35.),
    ]));
    rsx! { <Waterfall data={data} name={|s: &Step| s.name} value={|s: &Step| s.value} total_label="Closing" increase={rgb(0x179299)} decrease={rgb(0xfe640b)} total_color={rgb(0x8839ef)} class="h-72" /> }
}
```

### Value labels

Monthly changes with each value printed.

```rust
#[component]
fn labelled() {
    let data = use_signal(cash_flow());
    rsx! { <Waterfall data={data} name={|s: &Step| s.name} value={|s: &Step| s.value} total_label="Net" show_values={true} class="h-80" /> }
}
```

### Titled axis

Stock movement with a units title on the value axis.

```rust
#[component]
fn titled() {
    let data = use_signal(steps(&[
        ("Stock", 5_200.),
        ("Deliveries", 1_800.),
        ("Sales", -3_400.),
        ("Returns", 260.),
        ("Spoilage", -310.),
    ]));
    rsx! { <Waterfall data={data} name={|s: &Step| s.name} value={|s: &Step| s.value} total_label="Stock now" value_axis={AxisConfig::new().label("Units")} class="h-72" /> }
}
```

### No tooltip or legend

Weight change with hover and legend removed.

```rust
#[component]
fn quiet() {
    let data = use_signal(steps(&[
        ("Weight", 82.),
        ("Diet", -4.),
        ("Holiday", 3.),
        ("Running", -5.),
        ("Gym", -2.),
    ]));
    rsx! { <Waterfall data={data} name={|s: &Step| s.name} value={|s: &Step| s.value} total_label="Now" tooltip={Tooltip::hidden()} legend={Legend::hidden()} class="h-72" /> }
}
```

### Emissions target

A horizontal bridge where cuts are green and rises are red.

```rust
#[component]
fn emissions() {
    let data = use_signal(steps(&[
        ("2020", 48.),
        ("Transport", -6.),
        ("Energy", -11.),
        ("Industry", 2.),
        ("Farming", -3.),
        ("Offsets", -4.),
    ]));
    rsx! { <Waterfall data={data} name={|s: &Step| s.name} value={|s: &Step| s.value} horizontal={true} total_label="2030" show_values={true} decrease={rgb(0x40a02b)} increase={rgb(0xd20f39)} class="h-80" /> }
}
```

## `<Bullet>`

This section also uses:

```rust
#[derive(Clone, PartialEq)]
pub struct Kpi {
    pub name: &'static str,
    pub value: f64,
    pub target: f64,
    pub bands: [f64; 3],
}

fn kpis() -> Vec<Kpi> {
    vec![
        Kpi {
            name: "Revenue",
            value: 270.,
            target: 250.,
            bands: [150., 225., 300.],
        },
        Kpi {
            name: "Profit",
            value: 22.5,
            target: 27.,
            bands: [20., 25., 30.],
        },
        Kpi {
            name: "Orders",
            value: 1650.,
            target: 2100.,
            bands: [1000., 1800., 2500.],
        },
        Kpi {
            name: "Satisfaction",
            value: 4.4,
            target: 4.5,
            bands: [3.5, 4.25, 5.],
        },
    ]
}
```

### Revenue

Value bar over poor / fair / good bands, with the target tick.

```rust
#[component]
fn single() {
    let data = use_signal(kpis().into_iter().take(1).collect::<Vec<_>>());
    rsx! { <Bullet data={data} name={|k: &Kpi| k.name} value={|k: &Kpi| k.value} target={|k: &Kpi| k.target} ranges={|k: &Kpi| k.bands} class="h-24" /> }
}
```

### KPIs as % of range

Several KPIs on one shared percentage scale.

```rust
#[component]
fn scaled() {
    let data = use_signal(
        kpis()
            .into_iter()
            .map(|kpi| {
                let top = kpi.bands[2];
                Kpi {
                    value: kpi.value / top * 100.,
                    target: kpi.target / top * 100.,
                    bands: kpi.bands.map(|band| band / top * 100.),
                    ..kpi
                }
            })
            .collect::<Vec<_>>(),
    );
    rsx! { <Bullet data={data} name={|k: &Kpi| k.name} value={|k: &Kpi| k.value} target={|k: &Kpi| k.target} ranges={|k: &Kpi| k.bands} class="h-72" /> }
}
```

### Vertical

Two KPIs as columns.

```rust
#[component]
fn columns() {
    let data = use_signal(kpis().into_iter().take(2).collect::<Vec<_>>());
    rsx! { <Bullet data={data} name={|k: &Kpi| k.name} value={|k: &Kpi| k.value / k.bands[2] * 100.} target={|k: &Kpi| k.target / k.bands[2] * 100.} ranges={|k: &Kpi| k.bands.map(|band| band / k.bands[2] * 100.)} horizontal={false} color={rgb(0x179299)} class="h-72" /> }
}
```

### Server utilisation

Five resources against a shared 0 to 100 percent scale.

```rust
#[component]
fn utilisation() {
    let data = use_signal(servers());
    rsx! { <Bullet data={data} name={|k: &Kpi| k.name} value={|k: &Kpi| k.value} target={|k: &Kpi| k.target} ranges={|k: &Kpi| k.bands} class="h-72" /> }
}
```

### Upright, purple

The same resources as purple columns.

```rust
#[component]
fn upright() {
    let data = use_signal(servers());
    rsx! { <Bullet data={data} name={|k: &Kpi| k.name} value={|k: &Kpi| k.value} target={|k: &Kpi| k.target} ranges={|k: &Kpi| k.bands} horizontal={false} color={rgb(0x8839ef)} class="h-72" /> }
}
```

### Target colour

A blue bar with a red target tick.

```rust
#[component]
fn target_colour() {
    let data = use_signal(servers());
    rsx! { <Bullet data={data} name={|k: &Kpi| k.name} value={|k: &Kpi| k.value} target={|k: &Kpi| k.target} ranges={|k: &Kpi| k.bands} color={rgb(0x1e66f5)} target_color={rgb(0xd20f39)} class="h-72" /> }
}
```

### No tooltip

Server utilisation with the hover tooltip off.

```rust
#[component]
fn quiet() {
    let data = use_signal(servers());
    rsx! { <Bullet data={data} name={|k: &Kpi| k.name} value={|k: &Kpi| k.value} target={|k: &Kpi| k.target} ranges={|k: &Kpi| k.bands} tooltip={Tooltip::hidden()} class="h-72" /> }
}
```

### Titled axis

Three resources with a titled value axis.

```rust
#[component]
fn titled() {
    let data = use_signal(servers().into_iter().take(3).collect::<Vec<_>>());
    rsx! { <Bullet data={data} name={|k: &Kpi| k.name} value={|k: &Kpi| k.value} target={|k: &Kpi| k.target} ranges={|k: &Kpi| k.bands} value_axis={AxisConfig::new().label("Utilisation (%)")} class="h-56" /> }
}
```

### Profit

One KPI in green that misses its target.

```rust
#[component]
fn profit() {
    let data = use_signal(kpis().into_iter().skip(1).take(1).collect::<Vec<_>>());
    rsx! { <Bullet data={data} name={|k: &Kpi| k.name} value={|k: &Kpi| k.value} target={|k: &Kpi| k.target} ranges={|k: &Kpi| k.bands} color={rgb(0x40a02b)} class="h-24" /> }
}
```

### Rating from 3

A score axis starting at 3 with an orange target.

```rust
#[component]
fn satisfaction() {
    let data = use_signal(kpis().into_iter().skip(3).collect::<Vec<_>>());
    rsx! { <Bullet data={data} name={|k: &Kpi| k.name} value={|k: &Kpi| k.value} target={|k: &Kpi| k.target} ranges={|k: &Kpi| k.bands} value_axis={AxisConfig::new().min(3.)} target_color={rgb(0xfe640b)} class="h-24" /> }
}
```

## `<Funnel>`

### Sign-up funnel

Each stage centred, as wide as its count.

```rust
#[component]
fn signup() {
    let data = use_signal(steps(&[
        ("Visited", 12_400.),
        ("Signed up", 4_100.),
        ("Activated", 2_300.),
        ("Subscribed", 640.),
        ("Renewed", 410.),
    ]));
    rsx! { <Funnel data={data} name={|s: &Step| s.name} value={|s: &Step| s.value} class="h-72" /> }
}
```

### One colour

A sales pipeline in a single colour.

```rust
#[component]
fn single_colour() {
    let data = use_signal(steps(&[
        ("Leads", 900.),
        ("Qualified", 420.),
        ("Proposal", 160.),
        ("Won", 58.),
    ]));
    rsx! { <Funnel data={data} name={|s: &Step| s.name} value={|s: &Step| s.value} color={rgb(0x1e66f5)} class="h-72" /> }
}
```

### Vertical

Hiring stages as centred columns.

```rust
#[component]
fn columns() {
    let data = use_signal(steps(&[
        ("Applied", 320.),
        ("Screened", 140.),
        ("Interviewed", 48.),
        ("Offered", 9.),
    ]));
    rsx! { <Funnel data={data} name={|s: &Step| s.name} value={|s: &Step| s.value} horizontal={false} class="h-72" /> }
}
```

### Checkout, no labels

Shop conversion without the value labels.

```rust
#[component]
fn checkout() {
    let data = use_signal(steps(&[
        ("Product views", 48_000.),
        ("Add to cart", 9_600.),
        ("Checkout", 4_200.),
        ("Payment", 3_300.),
        ("Confirmed", 3_050.),
    ]));
    rsx! { <Funnel data={data} name={|s: &Step| s.name} value={|s: &Step| s.value} show_values={false} class="h-72" /> }
}
```

### No tooltip

An advertising funnel with the hover tooltip off.

```rust
#[component]
fn quiet() {
    let data = use_signal(steps(&[
        ("Impressions", 250_000.),
        ("Clicks", 12_500.),
        ("Trials", 1_900.),
        ("Paid", 310.),
    ]));
    rsx! { <Funnel data={data} name={|s: &Step| s.name} value={|s: &Step| s.value} tooltip={Tooltip::hidden()} class="h-72" /> }
}
```

### Pink columns

Webinar attendance as vertical pink stages.

```rust
#[component]
fn upright_colour() {
    let data = use_signal(steps(&[
        ("Registered", 1_800.),
        ("Attended", 760.),
        ("Stayed to end", 410.),
        ("Booked demo", 95.),
    ]));
    rsx! { <Funnel data={data} name={|s: &Step| s.name} value={|s: &Step| s.value} horizontal={false} color={rgb(0xea76cb)} class="h-72" /> }
}
```

### Eight stages

A long marketing funnel from awareness to advocacy.

```rust
#[component]
fn deep() {
    let data = use_signal(steps(&[
        ("Aware", 100_000.),
        ("Interested", 42_000.),
        ("Considering", 18_000.),
        ("Intent", 7_500.),
        ("Evaluated", 3_100.),
        ("Purchased", 1_200.),
        ("Repeat", 540.),
        ("Advocate", 150.),
    ]));
    rsx! { <Funnel data={data} name={|s: &Step| s.name} value={|s: &Step| s.value} class="h-96" /> }
}
```

### Three stages

A short email funnel in teal.

```rust
#[component]
fn short() {
    let data = use_signal(steps(&[
        ("Opened", 820.),
        ("Clicked", 190.),
        ("Bought", 41.),
    ]));
    rsx! { <Funnel data={data} name={|s: &Step| s.name} value={|s: &Step| s.value} color={rgb(0x179299)} class="h-56" /> }
}
```

### Support tickets

Orange columns for ticket handling without labels.

```rust
#[component]
fn tickets() {
    let data = use_signal(steps(&[
        ("Received", 1_240.),
        ("Triaged", 1_180.),
        ("Assigned", 960.),
        ("Resolved", 870.),
        ("Confirmed", 610.),
    ]));
    rsx! { <Funnel data={data} name={|s: &Step| s.name} value={|s: &Step| s.value} color={rgb(0xfe640b)} show_values={false} horizontal={false} class="h-72" /> }
}
```

### App retention

Installs to day 30 in purple with the tooltip off.

```rust
#[component]
fn app_install() {
    let data = use_signal(steps(&[
        ("Store views", 30_000.),
        ("Installs", 6_300.),
        ("Opened", 5_100.),
        ("Day 7", 1_400.),
        ("Day 30", 520.),
    ]));
    rsx! { <Funnel data={data} name={|s: &Step| s.name} value={|s: &Step| s.value} color={rgb(0x8839ef)} tooltip={Tooltip::hidden()} class="h-72" /> }
}
```

## `<Forest>`

This section also uses:

```rust
#[derive(Clone, PartialEq)]
pub struct Study {
    pub name: &'static str,
    pub effect: f64,
    pub low: f64,
    pub high: f64,
    pub patients: f64,
}

fn trials() -> Vec<Study> {
    [
        ("Adams 2014", 0.82, 0.61, 1.10, 420.),
        ("Baker 2015", 0.74, 0.58, 0.95, 860.),
        ("Chen 2017", 0.91, 0.70, 1.18, 510.),
        ("Diaz 2018", 0.65, 0.44, 0.96, 240.),
        ("Evans 2019", 0.88, 0.76, 1.02, 1620.),
        ("Fischer 2021", 0.71, 0.52, 0.97, 380.),
        ("Gupta 2023", 0.79, 0.66, 0.94, 1150.),
    ]
    .map(|(name, effect, low, high, patients)| Study {
        name,
        effect,
        low,
        high,
        patients,
    })
    .into()
}
```

### Hazard ratios

One row per trial on a log axis; square size follows trial size, and the dashed line marks no effect.

```rust
#[component]
fn ratios() {
    let data = use_signal(trials());
    rsx! { <Forest data={data} name={|s: &Study| s.name} estimate={|s: &Study| s.effect} low={|s: &Study| s.low} high={|s: &Study| s.high} weight={|s: &Study| s.patients} null={1.} value_axis={AxisConfig::new().scale(ScaleKind::Log { base: 10. }).label("Hazard ratio")} class="h-80" /> }
}
```

### Mean differences

Equal-sized markers around a null of 0.

```rust
#[component]
fn differences() {
    let data = use_signal(
        trials()
            .into_iter()
            .map(|study| Study {
                effect: (study.effect - 1.) * 10.,
                low: (study.low - 1.) * 10.,
                high: (study.high - 1.) * 10.,
                ..study
            })
            .collect::<Vec<_>>(),
    );
    rsx! { <Forest data={data} name={|s: &Study| s.name} estimate={|s: &Study| s.effect} low={|s: &Study| s.low} high={|s: &Study| s.high} value_axis={AxisConfig::new().label("Change in mmHg")} color={rgb(0x8839ef)} class="h-80" /> }
}
```

### Linear axis

The same trials on a linear axis with a titled risk ratio.

```rust
#[component]
fn linear() {
    let data = use_signal(trials());
    rsx! { <Forest data={data} name={|s: &Study| s.name} estimate={|s: &Study| s.effect} low={|s: &Study| s.low} high={|s: &Study| s.high} weight={|s: &Study| s.patients} null={1.} value_axis={AxisConfig::new().label("Risk ratio")} class="h-80" /> }
}
```

### Teal markers

Trials in a single custom colour.

```rust
#[component]
fn teal() {
    let data = use_signal(trials());
    rsx! { <Forest data={data} name={|s: &Study| s.name} estimate={|s: &Study| s.effect} low={|s: &Study| s.low} high={|s: &Study| s.high} weight={|s: &Study| s.patients} null={1.} color={rgb(0x179299)} class="h-80" /> }
}
```

### No tooltip

Trial estimates with the hover tooltip off.

```rust
#[component]
fn quiet() {
    let data = use_signal(trials());
    rsx! { <Forest data={data} name={|s: &Study| s.name} estimate={|s: &Study| s.effect} low={|s: &Study| s.low} high={|s: &Study| s.high} weight={|s: &Study| s.patients} null={1.} tooltip={Tooltip::hidden()} class="h-80" /> }
}
```

### Equal weights

Every trial given the same marker size.

```rust
#[component]
fn equal() {
    let data = use_signal(trials());
    rsx! { <Forest data={data} name={|s: &Study| s.name} estimate={|s: &Study| s.effect} low={|s: &Study| s.low} high={|s: &Study| s.high} weight={|_: &Study| 1.} null={1.} class="h-80" /> }
}
```

### Large trials only

Trials of more than 500 patients.

```rust
#[component]
fn large_trials() {
    let data = use_signal(
        trials()
            .into_iter()
            .filter(|study| study.patients > 500.)
            .collect::<Vec<_>>(),
    );
    rsx! { <Forest data={data} name={|s: &Study| s.name} estimate={|s: &Study| s.effect} low={|s: &Study| s.low} high={|s: &Study| s.high} weight={|s: &Study| s.patients} null={1.} value_axis={AxisConfig::new().label("Hazard ratio")} class="h-56" /> }
}
```

### Subgroup analysis

Effect by patient subgroup on a log axis, with the overall result last.

```rust
#[component]
fn subgroup() {
    let data = use_signal(subgroups());
    rsx! { <Forest data={data} name={|s: &Study| s.name} estimate={|s: &Study| s.effect} low={|s: &Study| s.low} high={|s: &Study| s.high} weight={|s: &Study| s.patients} null={1.} value_axis={AxisConfig::new().scale(ScaleKind::Log { base: 10. }).label("Hazard ratio")} class="h-96" /> }
}
```

### Subgroups, orange

Subgroup estimates in orange with no tooltip.

```rust
#[component]
fn subgroup_orange() {
    let data = use_signal(subgroups());
    rsx! { <Forest data={data} name={|s: &Study| s.name} estimate={|s: &Study| s.effect} low={|s: &Study| s.low} high={|s: &Study| s.high} weight={|s: &Study| s.patients} null={1.} color={rgb(0xfe640b)} tooltip={Tooltip::hidden()} class="h-96" /> }
}
```

### Percent change

Subgroup effects as a percentage change around zero.

```rust
#[component]
fn percent_change() {
    let data = use_signal(
        subgroups()
            .into_iter()
            .map(|study| Study {
                effect: (study.effect - 1.) * 100.,
                low: (study.low - 1.) * 100.,
                high: (study.high - 1.) * 100.,
                ..study
            })
            .collect::<Vec<_>>(),
    );
    rsx! { <Forest data={data} name={|s: &Study| s.name} estimate={|s: &Study| s.effect} low={|s: &Study| s.low} high={|s: &Study| s.high} weight={|s: &Study| s.patients} null={0.} value_axis={AxisConfig::new().label("Change in events (%)")} color={rgb(0xd20f39)} class="h-96" /> }
}
```
