# Flow, tree and profile charts: examples

Every example here also runs in the gallery (`cargo run --example gallery`), under **Charts** in the sidebar.

The examples share these imports and this sample data:

```rust
use blenderui::prelude::*;
use bui_charts::*;

const DAY: f64 = 86_400_000.;
const T0: f64 = 1_729_000_000_000.;

#[derive(Clone, PartialEq)]
struct Task {
    pub name: &'static str,
    pub team: &'static str,
    pub start: f64,
    pub end: f64,
}

fn tasks(entries: &[(&'static str, &'static str, f64, f64)]) -> Vec<Task> {
    entries
        .iter()
        .map(|&(name, team, start, end)| Task {
            name,
            team,
            start: T0 + start * DAY,
            end: T0 + end * DAY,
        })
        .collect()
}

fn plan() -> Vec<Task> {
    tasks(&[
        ("Research", "Product", 0., 6.),
        ("Design", "Design", 4., 12.),
        ("API", "Backend", 8., 22.),
        ("Web app", "Frontend", 12., 26.),
        ("Mobile", "Frontend", 16., 30.),
        ("Testing", "QA", 24., 33.),
        ("Launch", "Product", 33., 35.),
    ])
}

#[derive(Clone, PartialEq)]
struct Flow {
    pub from: &'static str,
    pub to: &'static str,
    pub amount: f64,
}

fn energy() -> Vec<Flow> {
    [
        ("Solar", "Electricity", 120.),
        ("Wind", "Electricity", 90.),
        ("Gas", "Electricity", 140.),
        ("Gas", "Heating", 160.),
        ("Oil", "Transport", 260.),
        ("Electricity", "Homes", 150.),
        ("Electricity", "Industry", 140.),
        ("Electricity", "Losses", 60.),
        ("Heating", "Homes", 110.),
        ("Heating", "Industry", 50.),
        ("Transport", "Cars", 170.),
        ("Transport", "Freight", 90.),
    ]
    .map(|(from, to, amount)| Flow { from, to, amount })
    .into()
}
```

## `<Gantt>`

### Project plan

One row per task, coloured by team.

```rust
#[component]
fn project() {
    let data = use_signal(plan());
    rsx! { <Gantt data={data} name={|t: &Task| t.name} start={|t: &Task| t.start} end={|t: &Task| t.end} group={|t: &Task| t.team} class="h-80" /> }
}
```

### Task names

Bars labelled by task, thinner rows, no legend.

```rust
#[component]
fn by_task() {
    let data = use_signal(plan());
    rsx! { <Gantt data={data} name={|t: &Task| t.name} start={|t: &Task| t.start} end={|t: &Task| t.end} row_height={0.5} legend={Legend::hidden()} class="h-72" /> }
}
```

### Sprint

Short tasks without labels.

```rust
#[component]
fn sprint() {
    let data = use_signal(
        plan()
            .into_iter()
            .take(4)
            .map(|task| Task {
                end: task.start + (task.end - task.start) * 0.5,
                ..task
            })
            .collect::<Vec<_>>(),
    );
    rsx! { <Gantt data={data} name={|t: &Task| t.name} start={|t: &Task| t.start} end={|t: &Task| t.end} group={|t: &Task| t.team} show_value={VisibilityMode::Never} class="h-56" /> }
}
```

### Construction

Nine trades overlapping across ten weeks.

```rust
#[component]
fn construction() {
    let data = use_signal(building());
    rsx! { <Gantt data={data} name={|t: &Task| t.name} start={|t: &Task| t.start} end={|t: &Task| t.end} group={|t: &Task| t.team} class="h-96" /> }
}
```

### Conference

Always-visible labels on tasks grouped by workstream.

```rust
#[component]
fn conference() {
    let data = use_signal(tasks(&[
        ("Venue", "Logistics", 0., 14.),
        ("Speakers", "Programme", 4., 30.),
        ("Sponsors", "Sales", 8., 40.),
        ("Tickets", "Sales", 20., 60.),
        ("Marketing", "Sales", 16., 58.),
        ("Schedule", "Programme", 28., 42.),
        ("Event day", "Logistics", 60., 62.),
    ]));
    rsx! { <Gantt data={data} name={|t: &Task| t.name} start={|t: &Task| t.start} end={|t: &Task| t.end} group={|t: &Task| t.team} show_value={VisibilityMode::Always} class="h-80" /> }
}
```

### Legend on the right

The team legend moved beside the chart.

```rust
#[component]
fn legend_right() {
    let data = use_signal(plan());
    rsx! { <Gantt data={data} name={|t: &Task| t.name} start={|t: &Task| t.start} end={|t: &Task| t.end} group={|t: &Task| t.team} legend={Legend::list().right()} class="h-80" /> }
}
```

### No tooltip

Hovering a bar shows nothing.

```rust
#[component]
fn no_tooltip() {
    let data = use_signal(plan());
    rsx! { <Gantt data={data} name={|t: &Task| t.name} start={|t: &Task| t.start} end={|t: &Task| t.end} group={|t: &Task| t.team} tooltip={Tooltip::hidden()} class="h-80" /> }
}
```

### Thick bars

Bars fill almost the whole row.

```rust
#[component]
fn thick_bars() {
    let data = use_signal(building());
    rsx! { <Gantt data={data} name={|t: &Task| t.name} start={|t: &Task| t.start} end={|t: &Task| t.end} group={|t: &Task| t.team} row_height={0.95} legend={Legend::hidden()} class="h-96" /> }
}
```

### Slim bars

Bars take under a third of each row.

```rust
#[component]
fn slim_bars() {
    let data = use_signal(plan());
    rsx! { <Gantt data={data} name={|t: &Task| t.name} start={|t: &Task| t.start} end={|t: &Task| t.end} group={|t: &Task| t.team} row_height={0.3} class="h-72" /> }
}
```

### Data migration

A single sequence of steps with no groups or legend.

```rust
#[component]
fn migration() {
    let data = use_signal(tasks(&[
        ("Audit", "Data", 0., 5.),
        ("Schema", "Data", 3., 10.),
        ("Backfill", "Data", 9., 21.),
        ("Dual write", "Data", 15., 28.),
        ("Cutover", "Data", 28., 30.),
        ("Cleanup", "Data", 30., 36.),
    ]));
    rsx! { <Gantt data={data} name={|t: &Task| t.name} start={|t: &Task| t.start} end={|t: &Task| t.end} legend={Legend::hidden()} class="h-72" /> }
}
```

## `<Sankey>`

### Energy flows

Sources to uses; hover a node or a band for its amount.

```rust
#[component]
fn energy_flow() {
    let data = use_signal(energy());
    rsx! { <Sankey data={data} source={|f: &Flow| f.from} target={|f: &Flow| f.to} value={|f: &Flow| f.amount} class="h-96" /> }
}
```

### Household budget

Income into one budget and out into categories.

```rust
#[component]
fn budget() {
    let data = use_signal(
        [
            ("Salary", "Budget", 4200.),
            ("Freelance", "Budget", 900.),
            ("Budget", "Rent", 1600.),
            ("Budget", "Food", 700.),
            ("Budget", "Savings", 1500.),
            ("Budget", "Transport", 300.),
            ("Budget", "Fun", 1000.),
        ]
        .map(|(from, to, amount)| Flow { from, to, amount })
        .to_vec(),
    );
    rsx! { <Sankey data={data} source={|f: &Flow| f.from} target={|f: &Flow| f.to} value={|f: &Flow| f.amount} node_width={18.} link_opacity={0.55} class="h-80" /> }
}
```

### Acquisition

Where visitors come from and where they go.

```rust
#[component]
fn funnel_flow() {
    let data = use_signal(
        [
            ("Ads", "Visit", 600.),
            ("Search", "Visit", 900.),
            ("Visit", "Sign-up", 500.),
            ("Visit", "Bounce", 1000.),
            ("Sign-up", "Paid", 120.),
            ("Sign-up", "Free", 380.),
        ]
        .map(|(from, to, amount)| Flow { from, to, amount })
        .to_vec(),
    );
    rsx! { <Sankey data={data} source={|f: &Flow| f.from} target={|f: &Flow| f.to} value={|f: &Flow| f.amount} node_padding={24.} class="h-80" /> }
}
```

### Thin links

Narrow nodes and faint bands.

```rust
#[component]
fn thin_links() {
    let data = use_signal(energy());
    rsx! { <Sankey data={data} source={|f: &Flow| f.from} target={|f: &Flow| f.to} value={|f: &Flow| f.amount} node_width={6.} link_opacity={0.2} class="h-96" /> }
}
```

### No tooltip

Tight node spacing and no hover tooltip.

```rust
#[component]
fn no_tooltip() {
    let data = use_signal(energy());
    rsx! { <Sankey data={data} source={|f: &Flow| f.from} target={|f: &Flow| f.to} value={|f: &Flow| f.amount} node_padding={4.} tooltip={Tooltip::hidden()} class="h-96" /> }
}
```

### Water supply

From rivers and wells to users and losses.

```rust
#[component]
fn water() {
    let data = use_signal(links(&[
        ("Rivers", "Treatment", 700.),
        ("Wells", "Treatment", 300.),
        ("Treatment", "Households", 520.),
        ("Treatment", "Farms", 280.),
        ("Treatment", "Industry", 140.),
        ("Treatment", "Leaks", 60.),
        ("Households", "Sewage", 400.),
        ("Industry", "Sewage", 90.),
    ]));
    rsx! { <Sankey data={data} source={|f: &Flow| f.from} target={|f: &Flow| f.to} value={|f: &Flow| f.amount} node_width={16.} class="h-96" /> }
}
```

### Supply chain

Grain through a mill to shops and pubs.

```rust
#[component]
fn supply_chain() {
    let data = use_signal(links(&[
        ("Farms", "Mill", 800.),
        ("Imports", "Mill", 200.),
        ("Mill", "Bakery", 650.),
        ("Mill", "Brewery", 250.),
        ("Mill", "Waste", 100.),
        ("Bakery", "Supermarkets", 420.),
        ("Bakery", "Cafes", 230.),
        ("Brewery", "Supermarkets", 150.),
        ("Brewery", "Pubs", 100.),
    ]));
    rsx! { <Sankey data={data} source={|f: &Flow| f.from} target={|f: &Flow| f.to} value={|f: &Flow| f.amount} node_padding={20.} link_opacity={0.5} class="h-80" /> }
}
```

### Income statement

Revenue down to net income, wider nodes.

```rust
#[component]
fn income() {
    let data = use_signal(links(&[
        ("Products", "Revenue", 620.),
        ("Services", "Revenue", 280.),
        ("Revenue", "Cost of sales", 340.),
        ("Revenue", "Gross profit", 560.),
        ("Gross profit", "Operating costs", 380.),
        ("Gross profit", "Operating profit", 180.),
        ("Operating profit", "Tax", 45.),
        ("Operating profit", "Net income", 135.),
    ]));
    rsx! { <Sankey data={data} source={|f: &Flow| f.from} target={|f: &Flow| f.to} value={|f: &Flow| f.amount} node_width={20.} class="h-96" /> }
}
```

### University admissions

Applicants narrowed to enrolled students by subject.

```rust
#[component]
fn admissions() {
    let data = use_signal(links(&[
        ("Applicants", "Admitted", 3200.),
        ("Applicants", "Rejected", 6800.),
        ("Admitted", "Enrolled", 1800.),
        ("Admitted", "Declined", 1400.),
        ("Enrolled", "Science", 800.),
        ("Enrolled", "Arts", 450.),
        ("Enrolled", "Business", 550.),
    ]));
    rsx! { <Sankey data={data} source={|f: &Flow| f.from} target={|f: &Flow| f.to} value={|f: &Flow| f.amount} class="h-80" /> }
}
```

### Emissions

Fuel to sector emissions with loose spacing.

```rust
#[component]
fn emissions() {
    let data = use_signal(links(&[
        ("Coal", "Power", 9.),
        ("Gas", "Power", 5.),
        ("Gas", "Buildings", 4.),
        ("Oil", "Transport", 11.),
        ("Oil", "Industry", 3.),
        ("Power", "Buildings", 6.),
        ("Power", "Industry", 8.),
    ]));
    rsx! { <Sankey data={data} source={|f: &Flow| f.from} target={|f: &Flow| f.to} value={|f: &Flow| f.amount} node_width={10.} node_padding={32.} link_opacity={0.65} class="h-80" /> }
}
```

## `<Icicle>`

### CPU profile

The root on top and each call below its caller; click to zoom.

```rust
#[component]
fn profile() {
    let data = use_signal(cpu());
    rsx! { <Icicle data={data} class="h-96" /> }
}
```

### With search

The header adds search and reset.

```rust
#[component]
fn with_header() {
    let data = use_signal(cpu());
    rsx! { <Icicle data={data} header={true} class="h-96" /> }
}
```

### Game frame

One frame split into update, render and present.

```rust
#[component]
fn game_frame() {
    let data = use_signal(frame());
    rsx! { <Icicle data={data} class="h-80" /> }
}
```

### Compiler

Where a Rust build spends its time, with search.

```rust
#[component]
fn compile() {
    let data = use_signal(compiler());
    rsx! { <Icicle data={data} header={true} class="h-80" /> }
}
```

### Database query

A query plan dominated by scans and joins.

```rust
#[component]
fn query() {
    let data = use_signal(database());
    rsx! { <Icicle data={data} class="h-72" /> }
}
```

### System boot

Kernel and service start-up, with search.

```rust
#[component]
fn boot() {
    let data = use_signal(startup());
    rsx! { <Icicle data={data} header={true} class="h-72" /> }
}
```

### Allocator

Small and large allocations against frees.

```rust
#[component]
fn memory() {
    let data = use_signal(allocator());
    rsx! { <Icicle data={data} class="h-64" /> }
}
```

### Build and test

A make run from compile to integration tests.

```rust
#[component]
fn make() {
    let data = use_signal(build());
    rsx! { <Icicle data={data} class="h-80" /> }
}
```

### Deep stacks

Narrow, deeply nested calls with the header.

```rust
#[component]
fn deep_stacks() {
    let data = use_signal(deep());
    rsx! { <Icicle data={data} header={true} class="h-96" /> }
}
```

### Thread pool

Many equal siblings next to idle time.

```rust
#[component]
fn thread_pool() {
    let data = use_signal(workers());
    rsx! { <Icicle data={data} class="h-56" /> }
}
```

## `<Spider>`

This section also uses:

```rust
#[derive(Clone, PartialEq)]
pub struct Player {
    pub name: &'static str,
    pub stats: [f64; 6],
}

fn players() -> Vec<Player> {
    vec![
        Player {
            name: "Ava",
            stats: [88., 72., 64., 91., 70., 80.],
        },
        Player {
            name: "Ben",
            stats: [65., 90., 85., 60., 78., 70.],
        },
        Player {
            name: "Cleo",
            stats: [75., 68., 92., 74., 88., 66.],
        },
    ]
}

fn axes() -> Vec<SharedString> {
    [
        "Pace",
        "Shooting",
        "Passing",
        "Dribbling",
        "Defending",
        "Physical",
    ]
    .map(SharedString::new_static)
    .to_vec()
}
```

### Three players

Six skills per player on a 0–100 web; hover a spoke to compare.

```rust
#[component]
fn compare() {
    let data = use_signal(players());
    rsx! { <Spider data={data} axes={axes()} name={|p: &Player| p.name} value={|p: &Player, axis: usize| p.stats[axis]} max={100.} class="h-96" /> }
}
```

### Circular grid

One player on a circular grid.

```rust
#[component]
fn circles() {
    let data = use_signal(players().into_iter().take(1).collect::<Vec<_>>());
    rsx! { <Spider data={data} axes={axes()} name={|p: &Player| p.name} value={|p: &Player, axis: usize| p.stats[axis]} max={100.} grid={SpiderGrid::Circle} fill_opacity={0.35} class="h-96" /> }
}
```

### Legend on the right

Click a legend entry to hide or isolate a player.

```rust
#[component]
fn legend_right() {
    let data = use_signal(players());
    rsx! { <Spider data={data} axes={axes()} name={|p: &Player| p.name} value={|p: &Player, axis: usize| p.stats[axis]} levels={4} legend={Legend::list().right()} class="h-96" /> }
}
```

### Phone review

Four phones rated 0–10 on five qualities.

```rust
#[component]
fn phone_review() {
    let data = use_signal(phones());
    rsx! { <Spider data={data} axes={labels(&["Camera", "Battery", "Display", "Speed", "Value"])} name={|r: &Rated| r.name} value={|r: &Rated, axis: usize| r.scores[axis]} max={10.} class="h-96" /> }
}
```

### Ten rings

A fine grid with bold outlines and light fill.

```rust
#[component]
fn phone_rings() {
    let data = use_signal(phones());
    rsx! { <Spider data={data} axes={labels(&["Camera", "Battery", "Display", "Speed", "Value"])} name={|r: &Rated| r.name} value={|r: &Rated, axis: usize| r.scores[axis]} max={10.} levels={10} line_width={4.} fill_opacity={0.08} class="h-96" /> }
}
```

### City life

Three cities on a circular grid with the legend on the right.

```rust
#[component]
fn city_life() {
    let data = use_signal(cities());
    rsx! { <Spider data={data} axes={labels(&["Safety", "Transit", "Cost", "Weather", "Culture"])} name={|r: &Rated| r.name} value={|r: &Rated, axis: usize| r.scores[axis]} max={10.} grid={SpiderGrid::Circle} legend={Legend::list().right()} class="h-96" /> }
}
```

### Automatic maximum

The rim follows the largest value, with three rings.

```rust
#[component]
fn auto_max() {
    let data = use_signal(cities());
    rsx! { <Spider data={data} axes={labels(&["Safety", "Transit", "Cost", "Weather", "Culture"])} name={|r: &Rated| r.name} value={|r: &Rated, axis: usize| r.scores[axis]} levels={3} class="h-80" /> }
}
```

### No tooltip

Hovering a spoke shows nothing.

```rust
#[component]
fn no_tooltip() {
    let data = use_signal(players());
    rsx! { <Spider data={data} axes={axes()} name={|p: &Player| p.name} value={|p: &Player, axis: usize| p.stats[axis]} max={100.} tooltip={Tooltip::hidden()} class="h-96" /> }
}
```

### Sparse grid

Two rings, hairline outlines and no legend.

```rust
#[component]
fn sparse() {
    let data = use_signal(players());
    rsx! { <Spider data={data} axes={axes()} name={|p: &Player| p.name} value={|p: &Player, axis: usize| p.stats[axis]} max={100.} levels={2} line_width={1.} fill_opacity={0.05} legend={Legend::hidden()} class="h-80" /> }
}
```

### Circles with legend

Eight circular rings and a right-hand legend.

```rust
#[component]
fn circles_right() {
    let data = use_signal(players());
    rsx! { <Spider data={data} axes={axes()} name={|p: &Player| p.name} value={|p: &Player, axis: usize| p.stats[axis]} max={100.} grid={SpiderGrid::Circle} levels={8} legend={Legend::list().right()} class="h-96" /> }
}
```

## `<Alluvial>`

This section also uses:

```rust
#[derive(Clone, PartialEq)]
pub struct Passenger {
    pub class: &'static str,
    pub sex: &'static str,
    pub age: &'static str,
    pub survived: &'static str,
    pub count: f64,
}

fn titanic() -> Vec<Passenger> {
    [
        ("1st", "Male", "Adult", "No", 118.),
        ("1st", "Male", "Adult", "Yes", 57.),
        ("1st", "Female", "Adult", "No", 4.),
        ("1st", "Female", "Adult", "Yes", 140.),
        ("1st", "Male", "Child", "Yes", 5.),
        ("1st", "Female", "Child", "Yes", 1.),
        ("2nd", "Male", "Adult", "No", 154.),
        ("2nd", "Male", "Adult", "Yes", 14.),
        ("2nd", "Female", "Adult", "No", 13.),
        ("2nd", "Female", "Adult", "Yes", 80.),
        ("2nd", "Male", "Child", "Yes", 11.),
        ("2nd", "Female", "Child", "Yes", 13.),
        ("3rd", "Male", "Adult", "No", 387.),
        ("3rd", "Male", "Adult", "Yes", 75.),
        ("3rd", "Female", "Adult", "No", 89.),
        ("3rd", "Female", "Adult", "Yes", 76.),
        ("3rd", "Male", "Child", "No", 35.),
        ("3rd", "Male", "Child", "Yes", 13.),
        ("3rd", "Female", "Child", "No", 17.),
        ("3rd", "Female", "Child", "Yes", 14.),
        ("Crew", "Male", "Adult", "No", 670.),
        ("Crew", "Male", "Adult", "Yes", 192.),
        ("Crew", "Female", "Adult", "No", 3.),
        ("Crew", "Female", "Adult", "Yes", 20.),
    ]
    .map(|(class, sex, age, survived, count)| Passenger {
        class,
        sex,
        age,
        survived,
        count,
    })
    .into()
}
```

### Titanic passengers

Each band follows one group of passengers through four stages, coloured by class.

```rust
#[component]
fn passengers() {
    let data = use_signal(titanic());
    rsx! {
        <Alluvial data={data} value={|p: &Passenger| p.count} class="h-96">
            <Stage name="Class" value={|p: &Passenger| p.class} />
            <Stage name="Sex" value={|p: &Passenger| p.sex} />
            <Stage name="Age" value={|p: &Passenger| p.age} />
            <Stage name="Survived" value={|p: &Passenger| p.survived} />
        </Alluvial>
    }
}
```

### Class and survival

Two stages, wider nodes and stronger bands.

```rust
#[component]
fn two_stages() {
    let data = use_signal(titanic());
    rsx! {
        <Alluvial data={data} value={|p: &Passenger| p.count} link_opacity={0.6} node_width={20.} class="h-80">
            <Stage name="Class" value={|p: &Passenger| p.class} />
            <Stage name="Survived" value={|p: &Passenger| p.survived} />
        </Alluvial>
    }
}
```

### Sex first

Three stages starting from sex, so bands take its colours.

```rust
#[component]
fn sex_first() {
    let data = use_signal(titanic());
    rsx! {
        <Alluvial data={data} value={|p: &Passenger| p.count} class="h-96">
            <Stage name="Sex" value={|p: &Passenger| p.sex} />
            <Stage name="Class" value={|p: &Passenger| p.class} />
            <Stage name="Survived" value={|p: &Passenger| p.survived} />
        </Alluvial>
    }
}
```

### Age and class

Faint bands from age through class to survival.

```rust
#[component]
fn age_class() {
    let data = use_signal(titanic());
    rsx! {
        <Alluvial data={data} value={|p: &Passenger| p.count} link_opacity={0.3} class="h-80">
            <Stage name="Age" value={|p: &Passenger| p.age} />
            <Stage name="Class" value={|p: &Passenger| p.class} />
            <Stage name="Survived" value={|p: &Passenger| p.survived} />
        </Alluvial>
    }
}
```

### Survival first

Two stages with the outcome as the first column and loose spacing.

```rust
#[component]
fn survival_first() {
    let data = use_signal(titanic());
    rsx! {
        <Alluvial data={data} value={|p: &Passenger| p.count} node_padding={30.} class="h-80">
            <Stage name="Survived" value={|p: &Passenger| p.survived} />
            <Stage name="Class" value={|p: &Passenger| p.class} />
        </Alluvial>
    }
}
```

### No tooltip

Four stages with no hover tooltip.

```rust
#[component]
fn no_tooltip() {
    let data = use_signal(titanic());
    rsx! {
        <Alluvial data={data} value={|p: &Passenger| p.count} tooltip={Tooltip::hidden()} class="h-96">
            <Stage name="Class" value={|p: &Passenger| p.class} />
            <Stage name="Sex" value={|p: &Passenger| p.sex} />
            <Stage name="Age" value={|p: &Passenger| p.age} />
            <Stage name="Survived" value={|p: &Passenger| p.survived} />
        </Alluvial>
    }
}
```

### Graduate careers

Schools, fields and sectors for a graduating class.

```rust
#[component]
fn careers() {
    let data = use_signal(graduates());
    rsx! {
        <Alluvial data={data} value={|g: &Graduate| g.count} class="h-96">
            <Stage name="School" value={|g: &Graduate| g.school} />
            <Stage name="Field" value={|g: &Graduate| g.field} />
            <Stage name="Sector" value={|g: &Graduate| g.job} />
        </Alluvial>
    }
}
```

### Field to sector

Narrow nodes and tight spacing between two stages.

```rust
#[component]
fn field_to_sector() {
    let data = use_signal(graduates());
    rsx! {
        <Alluvial data={data} value={|g: &Graduate| g.count} node_width={8.} node_padding={6.} link_opacity={0.5} class="h-80">
            <Stage name="Field" value={|g: &Graduate| g.field} />
            <Stage name="Sector" value={|g: &Graduate| g.job} />
        </Alluvial>
    }
}
```

### One per row

Without a value every row counts as one.

```rust
#[component]
fn row_counts() {
    let data = use_signal(graduates());
    rsx! {
        <Alluvial data={data} class="h-80">
            <Stage name="School" value={|g: &Graduate| g.school} />
            <Stage name="Sector" value={|g: &Graduate| g.job} />
        </Alluvial>
    }
}
```

### Sex and survival

Wide nodes and nearly opaque bands.

```rust
#[component]
fn sex_survival() {
    let data = use_signal(titanic());
    rsx! {
        <Alluvial data={data} value={|p: &Passenger| p.count} link_opacity={0.8} node_width={30.} class="h-72">
            <Stage name="Sex" value={|p: &Passenger| p.sex} />
            <Stage name="Survived" value={|p: &Passenger| p.survived} />
        </Alluvial>
    }
}
```

## `<NightingaleRose>`

This section also uses:

```rust
#[derive(Clone, PartialEq)]
pub struct Cause {
    pub name: &'static str,
    pub deaths: [f64; 12],
}

fn months() -> Vec<SharedString> {
    [
        "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec", "Jan", "Feb", "Mar",
    ]
    .map(SharedString::new_static)
    .to_vec()
}

fn crimea() -> Vec<Cause> {
    vec![
        Cause {
            name: "Disease",
            deaths: [
                1.4, 6.2, 4.7, 150., 328., 312., 197., 340., 631., 1023., 822., 480.,
            ],
        },
        Cause {
            name: "Wounds",
            deaths: [0., 0., 0., 0., 1., 16., 108., 115., 42., 32., 28., 48.],
        },
        Cause {
            name: "Other",
            deaths: [
                5., 9., 6., 23., 30., 70., 128., 106., 131., 324., 361., 172.,
            ],
        },
    ]
}
```

### Causes of death

Florence Nightingale's 1854–55 data: wedge area, not radius, grows with deaths; hover a month.

```rust
#[component]
fn crimean_war() {
    let data = use_signal(crimea());
    rsx! { <NightingaleRose data={data} sectors={months()} name={|c: &Cause| c.name} value={|c: &Cause, month: usize| c.deaths[month]} class="h-96" /> }
}
```

### One cause

Deaths from disease alone, lighter and without a legend.

```rust
#[component]
fn single() {
    let data = use_signal(crimea().into_iter().take(1).collect::<Vec<_>>());
    rsx! { <NightingaleRose data={data} sectors={months()} name={|c: &Cause| c.name} value={|c: &Cause, month: usize| c.deaths[month]} levels={3} fill_opacity={0.6} legend={Legend::hidden()} class="h-96" /> }
}
```

### Dense rings

Eight grid rings and solid wedges.

```rust
#[component]
fn dense_rings() {
    let data = use_signal(crimea());
    rsx! { <NightingaleRose data={data} sectors={months()} name={|c: &Cause| c.name} value={|c: &Cause, month: usize| c.deaths[month]} levels={8} fill_opacity={1.} class="h-96" /> }
}
```

### Monthly rainfall

Three climates stacked per month, with the legend on the right.

```rust
#[component]
fn rainfall() {
    let data = use_signal(series(&[
        (
            "Seattle",
            &[
                140., 95., 90., 60., 45., 35., 15., 20., 40., 80., 150., 160.,
            ],
        ),
        (
            "Mumbai",
            &[3., 3., 3., 12., 40., 520., 710., 440., 320., 90., 25., 5.],
        ),
        ("Cairo", &[5., 4., 3., 1., 0.5, 0., 0., 0., 0., 1., 3., 5.]),
    ]));
    rsx! { <NightingaleRose data={data} sectors={calendar()} name={|s: &Series| s.name} value={|s: &Series, month: usize| s.values[month]} legend={Legend::list().right()} class="h-96" /> }
}
```

### Home energy

Heating, cooling and lighting through the year.

```rust
#[component]
fn home_energy() {
    let data = use_signal(series(&[
        (
            "Heating",
            &[
                900., 820., 600., 380., 180., 40., 10., 10., 90., 330., 640., 880.,
            ],
        ),
        (
            "Cooling",
            &[0., 0., 10., 40., 160., 380., 520., 480., 230., 40., 0., 0.],
        ),
        (
            "Lighting",
            &[
                260., 220., 190., 150., 130., 110., 110., 130., 170., 210., 250., 280.,
            ],
        ),
    ]));
    rsx! { <NightingaleRose data={data} sectors={calendar()} name={|s: &Series| s.name} value={|s: &Series, month: usize| s.values[month]} levels={5} class="h-96" /> }
}
```

### Weekday visits

Seven wedges for web and app traffic.

```rust
#[component]
fn weekdays() {
    let data = use_signal(series(&[
        ("Web", &[820., 910., 880., 930., 760., 420., 380.]),
        ("App", &[540., 560., 590., 610., 700., 880., 840.]),
    ]));
    rsx! { <NightingaleRose data={data} sectors={labels(&["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"])} name={|s: &Series| s.name} value={|s: &Series, day: usize| s.values[day]} fill_opacity={0.7} class="h-80" /> }
}
```

### Seasonal sales

Four wedges, one per season.

```rust
#[component]
fn seasons() {
    let data = use_signal(series(&[
        ("Coats", &[20., 5., 60., 140.]),
        ("Shorts", &[40., 120., 30., 5.]),
        ("Umbrellas", &[70., 30., 80., 50.]),
    ]));
    rsx! { <NightingaleRose data={data} sectors={labels(&["Spring", "Summer", "Autumn", "Winter"])} name={|s: &Series| s.name} value={|s: &Series, season: usize| s.values[season]} levels={3} class="h-80" /> }
}
```

### Fixed maximum

The rim fixed at a stacked total of 500.

```rust
#[component]
fn capped() {
    let data = use_signal(crimea());
    rsx! { <NightingaleRose data={data} sectors={months()} name={|c: &Cause| c.name} value={|c: &Cause, month: usize| c.deaths[month]} max={500.} class="h-96" /> }
}
```

### No tooltip

Hovering a wedge shows nothing.

```rust
#[component]
fn no_tooltip() {
    let data = use_signal(crimea());
    rsx! { <NightingaleRose data={data} sectors={months()} name={|c: &Cause| c.name} value={|c: &Cause, month: usize| c.deaths[month]} tooltip={Tooltip::hidden()} class="h-96" /> }
}
```

### Emergency calls

Eight three-hour blocks, solid wedges and no legend.

```rust
#[component]
fn emergency_calls() {
    let data = use_signal(series(&[
        ("Calls", &[12., 5., 18., 64., 92., 70., 55., 31.]),
        ("Escalated", &[2., 1., 3., 9., 14., 10., 8., 5.]),
    ]));
    rsx! { <NightingaleRose data={data} sectors={labels(&["00–03", "03–06", "06–09", "09–12", "12–15", "15–18", "18–21", "21–24"])} name={|s: &Series| s.name} value={|s: &Series, block: usize| s.values[block]} levels={2} fill_opacity={1.} legend={Legend::hidden()} class="h-80" /> }
}
```

## `<ParallelCoordinates>`

This section also uses:

```rust
#[derive(Clone, PartialEq)]
pub struct Flower {
    pub id: SharedString,
    pub species: &'static str,
    pub sizes: [f64; 4],
}

fn axes() -> Vec<SharedString> {
    ["Sepal length", "Sepal width", "Petal length", "Petal width"]
        .map(SharedString::new_static)
        .to_vec()
}

fn flowers() -> Vec<Flower> {
    let species = [
        ("Setosa", [5.0, 3.4, 1.5, 0.25]),
        ("Versicolor", [5.9, 2.8, 4.3, 1.3]),
        ("Virginica", [6.6, 3.0, 5.6, 2.0]),
    ];
    (0..90u64)
        .map(|index| {
            let (name, means) = species[(index % 3) as usize];
            let mut sizes = means;
            for (axis, size) in sizes.iter_mut().enumerate() {
                *size += normal(index * 7 + axis as u64) * means[axis] * 0.08;
            }
            Flower {
                id: format!("{name} #{}", index / 3 + 1).into(),
                species: name,
                sizes,
            }
        })
        .collect()
}
```

### Three species

Each flower is one line across four measures, coloured by species; hover a line to read it.

```rust
#[component]
fn iris() {
    let data = use_signal(flowers());
    rsx! { <ParallelCoordinates data={data} axes={axes()} name={|f: &Flower| f.id.as_ref()} value={|f: &Flower, axis: usize| f.sizes[axis]} group={|f: &Flower| f.species} class="h-80" /> }
}
```

### Twelve flowers

Without a group every line has its own colour.

```rust
#[component]
fn ungrouped() {
    let data = use_signal(flowers().into_iter().take(12).collect::<Vec<_>>());
    rsx! { <ParallelCoordinates data={data} axes={axes()} name={|f: &Flower| f.id.as_ref()} value={|f: &Flower, axis: usize| f.sizes[axis]} line_width={2.} line_opacity={0.9} class="h-80" /> }
}
```

### Dense lines

Ninety flowers drawn thin and faint to show the density.

```rust
#[component]
fn dense() {
    let data = use_signal(flowers());
    rsx! { <ParallelCoordinates data={data} axes={axes()} name={|f: &Flower| f.id.as_ref()} value={|f: &Flower, axis: usize| f.sizes[axis]} group={|f: &Flower| f.species} line_width={1.} line_opacity={0.25} class="h-80" /> }
}
```

### Bold lines

Thirty flowers with thick, opaque lines.

```rust
#[component]
fn bold() {
    let data = use_signal(flowers().into_iter().take(30).collect::<Vec<_>>());
    rsx! { <ParallelCoordinates data={data} axes={axes()} name={|f: &Flower| f.id.as_ref()} value={|f: &Flower, axis: usize| f.sizes[axis]} group={|f: &Flower| f.species} line_width={3.} line_opacity={1.} class="h-80" /> }
}
```

### No legend

Species colours without a legend.

```rust
#[component]
fn no_legend() {
    let data = use_signal(flowers());
    rsx! { <ParallelCoordinates data={data} axes={axes()} name={|f: &Flower| f.id.as_ref()} value={|f: &Flower, axis: usize| f.sizes[axis]} group={|f: &Flower| f.species} legend={Legend::hidden()} class="h-80" /> }
}
```

### No tooltip

Hovering a line shows nothing.

```rust
#[component]
fn no_tooltip() {
    let data = use_signal(flowers());
    rsx! { <ParallelCoordinates data={data} axes={axes()} name={|f: &Flower| f.id.as_ref()} value={|f: &Flower, axis: usize| f.sizes[axis]} group={|f: &Flower| f.species} tooltip={Tooltip::hidden()} class="h-80" /> }
}
```

### Car classes

Twelve cars across mileage, power, weight, acceleration and price.

```rust
#[component]
fn car_classes() {
    let data = use_signal(cars());
    rsx! { <ParallelCoordinates data={data} axes={specs()} name={|c: &Car| c.name} value={|c: &Car, axis: usize| c.specs[axis]} group={|c: &Car| c.class} class="h-96" /> }
}
```

### Car models

Every car in its own colour with thicker lines.

```rust
#[component]
fn car_models() {
    let data = use_signal(cars());
    rsx! { <ParallelCoordinates data={data} axes={specs()} name={|c: &Car| c.name} value={|c: &Car, axis: usize| c.specs[axis]} line_width={2.5} class="h-96" /> }
}
```

### Legend on the right

Car classes with the legend beside the chart.

```rust
#[component]
fn cars_right() {
    let data = use_signal(cars());
    rsx! { <ParallelCoordinates data={data} axes={specs()} name={|c: &Car| c.name} value={|c: &Car, axis: usize| c.specs[axis]} group={|c: &Car| c.class} legend={Legend::list().right()} line_opacity={0.9} class="h-80" /> }
}
```

### One species

Only Setosa, so every line shares a colour.

```rust
#[component]
fn setosa() {
    let data = use_signal(
        flowers()
            .into_iter()
            .filter(|flower| flower.species == "Setosa")
            .collect::<Vec<_>>(),
    );
    rsx! { <ParallelCoordinates data={data} axes={axes()} name={|f: &Flower| f.id.as_ref()} value={|f: &Flower, axis: usize| f.sizes[axis]} group={|f: &Flower| f.species} class="h-72" /> }
}
```

## `<Dendrogram>`

This section also uses:

```rust
#[derive(Clone, PartialEq)]
pub struct Animal {
    pub name: &'static str,
    pub traits: [f64; 4],
}

fn animals() -> Vec<Animal> {
    [
        ("Lion", [8., 7., 2., 9.]),
        ("Tiger", [8., 8., 2., 9.]),
        ("Leopard", [8., 6., 3., 8.]),
        ("Wolf", [7., 5., 6., 7.]),
        ("Fox", [6., 2., 4., 5.]),
        ("Dog", [6., 4., 8., 5.]),
        ("Cow", [2., 9., 7., 1.]),
        ("Horse", [8., 9., 7., 1.]),
        ("Sheep", [3., 5., 8., 1.]),
        ("Goat", [4., 5., 7., 1.]),
        ("Eagle", [9., 2., 1., 8.]),
        ("Owl", [5., 1., 1., 7.]),
    ]
    .map(|(name, traits)| Animal { name, traits })
    .into()
}
```

### Three clusters

Average linkage on speed, size, sociability and diet; the tree is cut into three coloured clusters.

```rust
#[component]
fn animal_tree() {
    let data = use_signal(animals());
    rsx! { <Dendrogram data={data} name={|a: &Animal| a.name} features={|a: &Animal| a.traits.as_slice()} clusters={3} class="h-80" /> }
}
```

### Ward, horizontal

Ward linkage cut into four clusters, with the leaves down the side.

```rust
#[component]
fn sideways() {
    let data = use_signal(animals());
    rsx! { <Dendrogram data={data} name={|a: &Animal| a.name} features={|a: &Animal| a.traits.as_slice()} linkage={Linkage::Ward} clusters={4} horizontal={true} class="h-96" /> }
}
```

### Single linkage

Single linkage chains close neighbours together.

```rust
#[component]
fn single_link() {
    let data = use_signal(animals());
    rsx! { <Dendrogram data={data} name={|a: &Animal| a.name} features={|a: &Animal| a.traits.as_slice()} linkage={Linkage::Single} color={rgb(0x179299)} class="h-80" /> }
}
```

### Complete linkage

Complete linkage joins by the farthest members.

```rust
#[component]
fn complete_link() {
    let data = use_signal(animals());
    rsx! { <Dendrogram data={data} name={|a: &Animal| a.name} features={|a: &Animal| a.traits.as_slice()} linkage={Linkage::Complete} clusters={3} class="h-80" /> }
}
```

### Five clusters

Ward linkage cut into five small groups.

```rust
#[component]
fn ward_five() {
    let data = use_signal(animals());
    rsx! { <Dendrogram data={data} name={|a: &Animal| a.name} features={|a: &Animal| a.traits.as_slice()} linkage={Linkage::Ward} clusters={5} class="h-80" /> }
}
```

### Countries

Income, life expectancy and schooling grouped by Ward linkage.

```rust
#[component]
fn countries() {
    let data = use_signal(items(&[
        ("Norway", &[82., 83., 18.]),
        ("Germany", &[56., 81., 17.]),
        ("Japan", &[40., 84., 15.]),
        ("Brazil", &[9., 76., 8.]),
        ("Mexico", &[11., 75., 9.]),
        ("Turkey", &[10., 78., 8.]),
        ("India", &[2., 70., 7.]),
        ("Kenya", &[2., 67., 7.]),
        ("Nigeria", &[2., 55., 7.]),
        ("Canada", &[52., 82., 17.]),
    ]));
    rsx! { <Dendrogram data={data} name={|i: &Item| i.name} features={|i: &Item| i.values.as_slice()} linkage={Linkage::Ward} clusters={3} horizontal={true} class="h-96" /> }
}
```

### Foods

Twelve foods clustered by calories, protein, carbs, fat and fibre.

```rust
#[component]
fn foods() {
    let data = use_signal(items(&[
        ("Apple", &[52., 0.3, 14., 0.2, 2.4]),
        ("Banana", &[89., 1.1, 23., 0.3, 2.6]),
        ("Orange", &[47., 0.9, 12., 0.1, 2.4]),
        ("Bread", &[265., 9., 49., 3.2, 2.7]),
        ("Rice", &[130., 2.7, 28., 0.3, 0.4]),
        ("Pasta", &[158., 5.8, 31., 0.9, 1.8]),
        ("Chicken", &[165., 31., 0., 3.6, 0.]),
        ("Salmon", &[208., 20., 0., 13., 0.]),
        ("Beef", &[250., 26., 0., 15., 0.]),
        ("Almonds", &[579., 21., 22., 50., 12.]),
        ("Walnuts", &[654., 15., 14., 65., 6.7]),
        ("Lentils", &[116., 9., 20., 0.4, 7.9]),
    ]));
    rsx! { <Dendrogram data={data} name={|i: &Item| i.name} features={|i: &Item| i.values.as_slice()} clusters={4} class="h-80" /> }
}
```

### City climates

Temperature, rainfall and sunshine, leaves down the side.

```rust
#[component]
fn climates() {
    let data = use_signal(items(&[
        ("Oslo", &[6., 760., 1700.]),
        ("Moscow", &[6., 710., 1750.]),
        ("London", &[11., 600., 1500.]),
        ("Paris", &[12., 640., 1650.]),
        ("Rome", &[16., 800., 2500.]),
        ("Athens", &[18., 400., 2800.]),
        ("Cairo", &[22., 25., 3500.]),
        ("Dubai", &[27., 100., 3600.]),
        ("Singapore", &[27., 2300., 2000.]),
        ("Mumbai", &[27., 2400., 2600.]),
    ]));
    rsx! { <Dendrogram data={data} name={|i: &Item| i.name} features={|i: &Item| i.values.as_slice()} linkage={Linkage::Complete} clusters={3} horizontal={true} class="h-80" /> }
}
```

### One colour

No cut, so the whole tree takes the chosen colour.

```rust
#[component]
fn plain_tree() {
    let data = use_signal(animals());
    rsx! { <Dendrogram data={data} name={|a: &Animal| a.name} features={|a: &Animal| a.traits.as_slice()} color={rgb(0xd20f39)} class="h-72" /> }
}
```

### Six animals

A small horizontal tree cut into two clusters.

```rust
#[component]
fn few_leaves() {
    let data = use_signal(animals().into_iter().take(6).collect::<Vec<_>>());
    rsx! { <Dendrogram data={data} name={|a: &Animal| a.name} features={|a: &Animal| a.traits.as_slice()} linkage={Linkage::Single} clusters={2} horizontal={true} class="h-64" /> }
}
```

## `<SankeyFunnel>`

This section also uses:

```rust
#[derive(Clone, PartialEq)]
pub struct Channel {
    pub name: &'static str,
    pub counts: Vec<f64>,
}

fn channels(entries: &[(&'static str, &[f64])]) -> Vec<Channel> {
    entries
        .iter()
        .map(|(name, counts)| Channel {
            name,
            counts: counts.to_vec(),
        })
        .collect()
}

fn stages(names: &[&'static str]) -> Vec<SharedString> {
    names
        .iter()
        .copied()
        .map(SharedString::new_static)
        .collect()
}

fn signup() -> Vec<Channel> {
    channels(&[
        ("Search", &[5000., 2100., 900., 420.]),
        ("Ads", &[3200., 1100., 380., 150.]),
        ("Social", &[2400., 700., 260., 90.]),
        ("Referral", &[800., 520., 330., 210.]),
    ])
}

fn steps() -> Vec<SharedString> {
    stages(&["Visit", "Sign-up", "Trial", "Paid"])
}
```

### Acquisition funnel

Several entry channels flow through each stage; drop-offs peel off at every step. Hover a band.

```rust
#[component]
fn acquisition() {
    let data = use_signal(signup());
    rsx! { <SankeyFunnel data={data} stages={steps()} name={|c: &Channel| c.name} value={|c: &Channel, i: usize| c.counts[i]} class="h-96" /> }
}
```

### Without drop-offs

drops false shows only the people who continue.

```rust
#[component]
fn no_drops() {
    let data = use_signal(signup());
    rsx! { <SankeyFunnel data={data} stages={steps()} name={|c: &Channel| c.name} value={|c: &Channel, i: usize| c.counts[i]} drops={false} class="h-96" /> }
}
```

### Custom labels

source_label and drop_label.

```rust
#[component]
fn labels() {
    let data = use_signal(signup());
    rsx! { <SankeyFunnel data={data} stages={steps()} name={|c: &Channel| c.name} value={|c: &Channel, i: usize| c.counts[i]} source_label="Channel" drop_label="Lost" class="h-96" /> }
}
```

### Checkout by device

Mobile loses most at shipping.

```rust
#[component]
fn checkout() {
    let data = use_signal(channels(&[
        ("Desktop", &[9000., 4100., 2600., 2200.]),
        ("Mobile", &[14000., 4800., 2100., 1500.]),
        ("App", &[4000., 2600., 2000., 1850.]),
    ]));
    rsx! { <SankeyFunnel data={data} stages={stages(&["Cart", "Shipping", "Payment", "Order"])} name={|c: &Channel| c.name} value={|c: &Channel, i: usize| c.counts[i]} class="h-96" /> }
}
```

### Hiring pipeline

Five stages from three sources.

```rust
#[component]
fn hiring() {
    let data = use_signal(channels(&[
        ("Referrals", &[120., 70., 30., 12., 9.]),
        ("Job boards", &[900., 160., 45., 10., 6.]),
        ("Agencies", &[200., 80., 30., 9., 5.]),
    ]));
    rsx! { <SankeyFunnel data={data} stages={stages(&["Applied", "Screened", "Interviewed", "Offer", "Hired"])} name={|c: &Channel| c.name} value={|c: &Channel, i: usize| c.counts[i]} class="h-96" /> }
}
```

### One channel

A plain funnel with drop-offs.

```rust
#[component]
fn one_channel() {
    let data = use_signal(channels(&[("All traffic", &[10000., 3500., 1200., 400.])]));
    rsx! { <SankeyFunnel data={data} stages={steps()} name={|c: &Channel| c.name} value={|c: &Channel, i: usize| c.counts[i]} class="h-80" /> }
}
```

### Two stages

The shortest funnel.

```rust
#[component]
fn two_stages() {
    let data = use_signal(signup());
    rsx! { <SankeyFunnel data={data} stages={stages(&["Visit", "Sign-up"])} name={|c: &Channel| c.name} value={|c: &Channel, i: usize| c.counts[i]} class="h-80" /> }
}
```

### Strong bands

Wider nodes and stronger bands.

```rust
#[component]
fn strong_bands() {
    let data = use_signal(signup());
    rsx! { <SankeyFunnel data={data} stages={steps()} name={|c: &Channel| c.name} value={|c: &Channel, i: usize| c.counts[i]} link_opacity={0.7} node_width={18.} class="h-96" /> }
}
```

### App onboarding

Retention through onboarding steps.

```rust
#[component]
fn onboarding() {
    let data = use_signal(channels(&[
        ("iOS", &[3000., 2600., 2100., 1900., 1500.]),
        ("Android", &[4200., 3300., 2400., 2000., 1400.]),
        ("Web", &[2500., 1800., 1500., 1300., 1100.]),
    ]));
    rsx! { <SankeyFunnel data={data} stages={stages(&["Install", "Open", "Profile", "First task", "Day 7"])} name={|c: &Channel| c.name} value={|c: &Channel, i: usize| c.counts[i]} class="h-96" /> }
}
```

### More spacing

node_padding 28.

```rust
#[component]
fn spaced() {
    let data = use_signal(signup());
    rsx! { <SankeyFunnel data={data} stages={steps()} name={|c: &Channel| c.name} value={|c: &Channel, i: usize| c.counts[i]} node_padding={28.} class="h-96" /> }
}
```
