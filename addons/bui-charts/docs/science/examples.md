# Science charts: examples

Every example here also runs in the gallery (`cargo run --example gallery`), under **Charts** in the sidebar.

The examples share these imports and this sample data:

```rust
use blenderui::prelude::*;
use bui_charts::*;

use crate::distribution::normal;
use crate::finance::noise;
```

## `<Oscilloscope>`

This section also uses:

```rust
#[derive(Clone, PartialEq)]
pub struct Probe {
    pub time: f64,
    pub one: f64,
    pub two: f64,
}

fn capture(frequency: f64) -> Vec<Probe> {
    (0..2000u64)
        .map(|index| {
            let t = index as f64 / 20_000.;
            Probe {
                time: t,
                one: (std::f64::consts::TAU * frequency * t).sin() * 2. + 0.05 * normal(index),
                two: if (frequency * t).fract() < 0.5 {
                    1.
                } else {
                    -1.
                },
            }
        })
        .collect()
}
```

### Triggered

Two channels, starting at the first rising crossing of 0.5 V on CH1, 12 ms wide.

```rust
#[component]
fn triggered() {
    let data = use_signal(capture(250.));
    rsx! {
        <Oscilloscope data={data} x={|p: &Probe| p.time} trigger={0.5} span={0.012} class="h-72">
            <TrendSeries name="CH1" y={|p: &Probe| p.one} />
            <TrendSeries name="CH2" y={|p: &Probe| p.two} />
        </Oscilloscope>
    }
}
```

### Free running

No trigger: the whole capture.

```rust
#[component]
fn free_run() {
    let data = use_signal(capture(400.));
    rsx! {
        <Oscilloscope data={data} x={|p: &Probe| p.time} class="h-72">
            <TrendSeries name="CH1" y={|p: &Probe| p.one} color={rgb(0xdf8e1d)} />
        </Oscilloscope>
    }
}
```

### 1 kHz, 4 ms

A fast signal triggered at zero over a short window.

```rust
#[component]
fn fast_edge() {
    let data = use_signal(capture(1000.));
    rsx! {
        <Oscilloscope data={data} x={|p: &Probe| p.time} trigger={0.} span={0.004} class="h-72">
            <TrendSeries name="CH1" y={|p: &Probe| p.one} />
            <TrendSeries name="CH2" y={|p: &Probe| p.two} />
        </Oscilloscope>
    }
}
```

### 50 Hz, 40 ms

A slow signal across a long window.

```rust
#[component]
fn slow_sweep() {
    let data = use_signal(capture(50.));
    rsx! {
        <Oscilloscope data={data} x={|p: &Probe| p.time} trigger={0.} span={0.04} class="h-72">
            <TrendSeries name="CH1" y={|p: &Probe| p.one} />
            <TrendSeries name="CH2" y={|p: &Probe| p.two} />
        </Oscilloscope>
    }
}
```

### Trigger at 1.5 V

A high threshold starts the sweep late on the rising edge.

```rust
#[component]
fn high_trigger() {
    let data = use_signal(capture(150.));
    rsx! {
        <Oscilloscope data={data} x={|p: &Probe| p.time} trigger={1.5} span={0.02} class="h-72">
            <TrendSeries name="CH1" y={|p: &Probe| p.one} color={rgb(0xd20f39)} />
        </Oscilloscope>
    }
}
```

### Legend on the right

Channel names in a side legend.

```rust
#[component]
fn legend_right() {
    let data = use_signal(capture(250.));
    rsx! {
        <Oscilloscope data={data} x={|p: &Probe| p.time} trigger={0.5} span={0.012} legend={Legend::list().right()} class="h-72">
            <TrendSeries name="CH1" y={|p: &Probe| p.one} />
            <TrendSeries name="CH2" y={|p: &Probe| p.two} />
        </Oscilloscope>
    }
}
```

### No legend

Legend hidden for a cleaner trace.

```rust
#[component]
fn legend_hidden() {
    let data = use_signal(capture(250.));
    rsx! {
        <Oscilloscope data={data} x={|p: &Probe| p.time} trigger={0.5} span={0.012} legend={Legend::hidden()} class="h-72">
            <TrendSeries name="CH1" y={|p: &Probe| p.one} />
            <TrendSeries name="CH2" y={|p: &Probe| p.two} />
        </Oscilloscope>
    }
}
```

### No tooltip

Tooltip hidden.

```rust
#[component]
fn no_tooltip() {
    let data = use_signal(capture(250.));
    rsx! {
        <Oscilloscope data={data} x={|p: &Probe| p.time} trigger={0.5} span={0.012} tooltip={Tooltip::hidden()} class="h-72">
            <TrendSeries name="CH1" y={|p: &Probe| p.one} />
            <TrendSeries name="CH2" y={|p: &Probe| p.two} />
        </Oscilloscope>
    }
}
```

### Axis label

A labelled voltage axis.

```rust
#[component]
fn labelled() {
    let data = use_signal(capture(250.));
    rsx! {
        <Oscilloscope data={data} x={|p: &Probe| p.time} trigger={0.5} span={0.012} y_axis={AxisConfig::new().label("Volts")} class="h-72">
            <TrendSeries name="CH1" y={|p: &Probe| p.one} />
        </Oscilloscope>
    }
}
```

### Clock channel

The square wave alone, free running over 10 ms.

```rust
#[component]
fn clock_only() {
    let data = use_signal(capture(500.));
    rsx! {
        <Oscilloscope data={data} x={|p: &Probe| p.time} span={0.01} class="h-72">
            <TrendSeries name="CH2" y={|p: &Probe| p.two} color={rgb(0x179299)} />
        </Oscilloscope>
    }
}
```

### Custom colours

Each channel in its own colour.

```rust
#[component]
fn custom_colours() {
    let data = use_signal(capture(300.));
    rsx! {
        <Oscilloscope data={data} x={|p: &Probe| p.time} trigger={0.} span={0.01} class="h-72">
            <TrendSeries name="CH1" y={|p: &Probe| p.one} color={rgb(0x8839ef)} />
            <TrendSeries name="CH2" y={|p: &Probe| p.two} color={rgb(0xfe640b)} />
        </Oscilloscope>
    }
}
```

## `<EfficientFrontier>`

This section also uses:

```rust
fn returns() -> Vec<f64> {
    vec![0.06, 0.09, 0.12, 0.15]
}

fn covariance() -> Vec<Vec<f64>> {
    vec![
        vec![0.0025, 0.0010, 0.0008, 0.0006],
        vec![0.0010, 0.0090, 0.0030, 0.0020],
        vec![0.0008, 0.0030, 0.0200, 0.0060],
        vec![0.0006, 0.0020, 0.0060, 0.0400],
    ]
}
```

### Four assets

Random portfolios coloured by Sharpe ratio, the efficient frontier, the capital market line and the max-Sharpe portfolio.

```rust
#[component]
fn four_assets() {
    rsx! { <EfficientFrontier returns={returns()} covariance={covariance()} risk_free={0.03} class="h-80" /> }
}
```

### Fewer portfolios

300 random portfolios and no risk-free rate.

```rust
#[component]
fn sparse() {
    rsx! { <EfficientFrontier returns={returns()} covariance={covariance()} portfolios={300} scheme={Scheme::Blues} class="h-80" /> }
}
```

### 5% risk-free rate

A higher risk-free rate tilts the capital market line.

```rust
#[component]
fn high_rate() {
    rsx! { <EfficientFrontier returns={returns()} covariance={covariance()} risk_free={0.05} class="h-80" /> }
}
```

### 2000 portfolios

A denser cloud in the Reds scheme.

```rust
#[component]
fn dense() {
    rsx! { <EfficientFrontier returns={returns()} covariance={covariance()} portfolios={2000} risk_free={0.03} scheme={Scheme::Reds} class="h-80" /> }
}
```

### Legend on the right

Series names in a side legend.

```rust
#[component]
fn legend_right() {
    rsx! { <EfficientFrontier returns={returns()} covariance={covariance()} risk_free={0.03} legend={Legend::list().right()} class="h-80" /> }
}
```

### No legend

Legend hidden.

```rust
#[component]
fn legend_hidden() {
    rsx! { <EfficientFrontier returns={returns()} covariance={covariance()} risk_free={0.03} legend={Legend::hidden()} class="h-80" /> }
}
```

### No tooltip

Tooltip hidden.

```rust
#[component]
fn no_tooltip() {
    rsx! { <EfficientFrontier returns={returns()} covariance={covariance()} risk_free={0.03} tooltip={Tooltip::hidden()} class="h-80" /> }
}
```

### Uncorrelated pair

Two assets with zero correlation.

```rust
#[component]
fn uncorrelated_pair() {
    rsx! { <EfficientFrontier returns={vec![0.06, 0.15]} covariance={pair(0.)} risk_free={0.03} scheme={Scheme::Purples} class="h-80" /> }
}
```

### Highly correlated pair

A 0.9 correlation leaves little diversification benefit.

```rust
#[component]
fn correlated_pair() {
    rsx! { <EfficientFrontier returns={vec![0.06, 0.15]} covariance={pair(0.9)} risk_free={0.03} scheme={Scheme::Oranges} class="h-80" /> }
}
```

### Negatively correlated pair

A -0.8 correlation pulls the frontier sharply left.

```rust
#[component]
fn hedged_pair() {
    rsx! { <EfficientFrontier returns={vec![0.06, 0.15]} covariance={pair(-0.8)} risk_free={0.03} scheme={Scheme::Greens} class="h-80" /> }
}
```

### Equal returns

Four assets with the same expected return differ only in risk.

```rust
#[component]
fn equal_returns() {
    rsx! { <EfficientFrontier returns={vec![0.08, 0.08, 0.08, 0.08]} covariance={covariance()} class="h-80" /> }
}
```

## `<PhasePortrait>`

This section also uses:

```rust
fn pendulum(x: f64, y: f64) -> (f64, f64) {
    (y, -x.sin() - 0.15 * y)
}
```

### Damped pendulum

Direction field and trajectories spiralling into rest.

```rust
#[component]
fn damped() {
    rsx! {
        <PhasePortrait
            field={pendulum}
            starts={vec![(-3., 2.5), (2.5, 0.), (0.5, 2.), (-1.5, -2.)]}
            x_range={(-4., 4.)}
            y_range={(-3., 3.)}
            duration={30.}
            x_label="θ"
            y_label="ω"
            class="h-80"
        />
    }
}
```

### Lotka–Volterra

Closed orbits of predator and prey populations.

```rust
#[component]
fn predator_prey() {
    rsx! {
        <PhasePortrait
            field={|x: f64, y: f64| (x * (1. - 0.5 * y), y * (0.5 * x - 1.))}
            starts={vec![(1., 0.5), (2., 1.), (3., 2.)]}
            x_range={(0., 6.)}
            y_range={(0., 5.)}
            duration={20.}
            x_label="Prey"
            y_label="Predators"
            color={rgb(0xd20f39)}
            class="h-80"
        />
    }
}
```

### Undamped pendulum

Closed orbits around the stable point.

```rust
#[component]
fn undamped() {
    rsx! {
        <PhasePortrait
            field={|x: f64, y: f64| (y, -x.sin())}
            starts={vec![(0.5, 0.), (1.5, 0.), (2.5, 0.), (0., 2.5)]}
            x_range={(-4., 4.)}
            y_range={(-3., 3.)}
            duration={20.}
            x_label="θ"
            y_label="ω"
            class="h-80"
        />
    }
}
```

### Van der Pol oscillator

Trajectories from inside and outside converge on a limit cycle.

```rust
#[component]
fn van_der_pol() {
    rsx! {
        <PhasePortrait
            field={|x: f64, y: f64| (y, 1.5 * (1. - x * x) * y - x)}
            starts={vec![(0.1, 0.), (3., 3.), (-3., -2.)]}
            x_range={(-4., 4.)}
            y_range={(-5., 5.)}
            duration={20.}
            color={rgb(0x8839ef)}
            class="h-80"
        />
    }
}
```

### Saddle point

Trajectories approach along one axis and leave along the other.

```rust
#[component]
fn saddle() {
    rsx! {
        <PhasePortrait
            field={|x: f64, y: f64| (x, -y)}
            starts={vec![(0.1, 2.), (-0.1, 2.), (0.1, -2.), (-0.1, -2.), (2., 0.1), (-2., 0.1)]}
            x_range={(-3., 3.)}
            y_range={(-3., 3.)}
            duration={3.}
            color={rgb(0xfe640b)}
            class="h-80"
        />
    }
}
```

### Stable spiral

A linear system spiralling into the origin.

```rust
#[component]
fn stable_spiral() {
    rsx! {
        <PhasePortrait
            field={|x: f64, y: f64| (-0.2 * x + y, -x - 0.2 * y)}
            starts={vec![(3., 0.), (-2., 2.)]}
            x_range={(-4., 4.)}
            y_range={(-4., 4.)}
            duration={25.}
            color={rgb(0x179299)}
            class="h-80"
        />
    }
}
```

### Fine integration step

step 0.01 with four arrows per trajectory.

```rust
#[component]
fn fine_steps() {
    rsx! {
        <PhasePortrait
            field={pendulum}
            starts={vec![(-3., 2.5), (2.5, 0.)]}
            x_range={(-4., 4.)}
            y_range={(-3., 3.)}
            duration={30.}
            step={0.01}
            arrows={4}
            class="h-80"
        />
    }
}
```

### One arrow

arrows 1 marks only the direction of each trajectory.

```rust
#[component]
fn few_arrows() {
    rsx! {
        <PhasePortrait
            field={pendulum}
            starts={vec![(-3., 2.5), (2.5, 0.), (0.5, 2.)]}
            x_range={(-4., 4.)}
            y_range={(-3., 3.)}
            duration={30.}
            arrows={1}
            color={rgb(0x40a02b)}
            class="h-80"
        />
    }
}
```

### Legend on the right

Trajectory names in a side legend.

```rust
#[component]
fn legend_right() {
    rsx! {
        <PhasePortrait
            field={pendulum}
            starts={vec![(-3., 2.5), (2.5, 0.), (0.5, 2.)]}
            x_range={(-4., 4.)}
            y_range={(-3., 3.)}
            duration={30.}
            legend={Legend::list().right()}
            class="h-80"
        />
    }
}
```

### Tooltip enabled

Hover a trajectory to read it.

```rust
#[component]
fn with_tooltip() {
    rsx! {
        <PhasePortrait
            field={pendulum}
            starts={vec![(-3., 2.5), (2.5, 0.)]}
            x_range={(-4., 4.)}
            y_range={(-3., 3.)}
            duration={30.}
            tooltip={Tooltip::single()}
            class="h-80"
        />
    }
}
```

## `<PoincareMap>`

This section also uses:

```rust
#[derive(Clone, PartialEq)]
pub struct Beat {
    pub interval: f64,
}

fn heart(seed: u64) -> Vec<Beat> {
    let mut interval = 800.;
    (0..400u64)
        .map(|index| {
            interval += (800. - interval) * 0.2 + 25. * normal(seed * 1000 + index);
            Beat { interval }
        })
        .collect()
}

fn logistic(rate: f64) -> Vec<Beat> {
    let mut x = 0.2;
    (0..600)
        .map(|_| {
            x = rate * x * (1. - x);
            Beat { interval: x }
        })
        .collect()
}
```

### Heart rate variability

Each RR interval against the next.

```rust
#[component]
fn heart_rate() {
    let data = use_signal(heart(1));
    rsx! { <PoincareMap data={data} value={|b: &Beat| b.interval} label="RR" class="h-80" /> }
}
```

### Logistic map

Chaotic values trace the parabola they come from.

```rust
#[component]
fn chaos() {
    let data = use_signal(logistic(3.9));
    rsx! { <PoincareMap data={data} value={|b: &Beat| b.interval} point_size={3.} color={rgb(0x8839ef)} class="h-80" /> }
}
```

### Lag 5

Each value against the one 5 steps later.

```rust
#[component]
fn lagged() {
    let data = use_signal(heart(2));
    rsx! { <PoincareMap data={data} value={|b: &Beat| b.interval} lag={5} label="RR" identity={false} class="h-80" /> }
}
```

### Period two

At r = 3.2 the orbit alternates between two clusters.

```rust
#[component]
fn period_two() {
    let data = use_signal(logistic(3.2));
    rsx! { <PoincareMap data={data} value={|b: &Beat| b.interval} point_size={6.} color={rgb(0xd20f39)} class="h-80" /> }
}
```

### Edge of chaos

At r = 3.57 the points fill a handful of tight bands.

```rust
#[component]
fn edge_of_chaos() {
    let data = use_signal(logistic(3.57));
    rsx! { <PoincareMap data={data} value={|b: &Beat| b.interval} point_size={3.} color={rgb(0xfe640b)} class="h-80" /> }
}
```

### Chaos, lag 2

The second iterate of the logistic map folds twice.

```rust
#[component]
fn chaos_lag_two() {
    let data = use_signal(logistic(3.9));
    rsx! { <PoincareMap data={data} value={|b: &Beat| b.interval} lag={2} point_size={3.} color={rgb(0x179299)} class="h-80" /> }
}
```

### Lag 2 heart rate

Each RR interval against the one two beats later.

```rust
#[component]
fn calm_heart() {
    let data = use_signal(heart(3));
    rsx! { <PoincareMap data={data} value={|b: &Beat| b.interval} lag={2} label="RR" color={rgb(0x40a02b)} class="h-80" /> }
}
```

### Lag 10

Long lags decorrelate the cloud.

```rust
#[component]
fn long_lag() {
    let data = use_signal(heart(4));
    rsx! { <PoincareMap data={data} value={|b: &Beat| b.interval} lag={10} label="RR" point_size={4.} class="h-80" /> }
}
```

### No tooltip

Tooltip hidden.

```rust
#[component]
fn no_tooltip() {
    let data = use_signal(heart(5));
    rsx! { <PoincareMap data={data} value={|b: &Beat| b.interval} label="RR" tooltip={Tooltip::hidden()} class="h-80" /> }
}
```

### Custom label

Axes named Interval with small points.

```rust
#[component]
fn named() {
    let data = use_signal(heart(6));
    rsx! { <PoincareMap data={data} value={|b: &Beat| b.interval} label="Interval" point_size={2.} color={rgb(0x1e66f5)} class="h-80" /> }
}
```

## `<Bifurcation>`

### Logistic map

The period-doubling route to chaos.

```rust
#[component]
fn logistic() {
    rsx! { <Bifurcation map={|r: f64, x: f64| r * x * (1. - x)} parameter={(2.5, 4.)} class="h-80" /> }
}
```

### Zoomed in

More detail between r = 3.5 and 3.9.

```rust
#[component]
fn zoomed() {
    rsx! { <Bifurcation map={|r: f64, x: f64| r * x * (1. - x)} parameter={(3.5, 3.9)} steps={400} keep={100} color={rgb(0xd20f39)} class="h-80" /> }
}
```

### Sine map

The same cascade for x → μ·sin(πx).

```rust
#[component]
fn sine() {
    rsx! { <Bifurcation map={|r: f64, x: f64| r * (std::f64::consts::PI * x).sin()} parameter={(0.6, 1.)} x_label="μ" class="h-80" /> }
}
```

### Period-3 window

The window of stability near r = 3.83 inside the chaos.

```rust
#[component]
fn period_three() {
    rsx! { <Bifurcation map={|r: f64, x: f64| r * x * (1. - x)} parameter={(3.8, 3.9)} steps={600} keep={150} color={rgb(0x8839ef)} class="h-80" /> }
}
```

### Tent map

The tent map goes chaotic at μ = 1 with no period doubling.

```rust
#[component]
fn tent() {
    rsx! { <Bifurcation map={|r: f64, x: f64| r * x.min(1. - x)} parameter={(1., 2.)} x_label="μ" color={rgb(0x179299)} class="h-80" /> }
}
```

### Quadratic map

x → x² + c from c = −2 to 0.25, the real slice of the Mandelbrot set.

```rust
#[component]
fn quadratic() {
    rsx! { <Bifurcation map={|c: f64, x: f64| x * x + c} parameter={(-2., 0.25)} start={0.} x_label="c" y_label="x" color={rgb(0xfe640b)} class="h-80" /> }
}
```

### Ricker model

A population model with an exponential growth term.

```rust
#[component]
fn ricker() {
    rsx! { <Bifurcation map={|r: f64, x: f64| x * (r * (1. - x)).exp()} parameter={(1.5, 3.2)} start={0.5} x_label="r" y_label="Population" color={rgb(0x40a02b)} class="h-80" /> }
}
```

### Fine points

More samples with the smallest points.

```rust
#[component]
fn fine_points() {
    rsx! { <Bifurcation map={|r: f64, x: f64| r * x * (1. - x)} parameter={(2.5, 4.)} steps={800} keep={200} point_size={1.} class="h-80" /> }
}
```

### Long transient

transient 500 discards more iterations from another start.

```rust
#[component]
fn long_transient() {
    rsx! { <Bifurcation map={|r: f64, x: f64| r * x * (1. - x)} parameter={(3., 3.6)} transient={500} start={0.7} y_label="Population" class="h-80" /> }
}
```

### Bold points

Few samples drawn large between r = 2.8 and 3.6.

```rust
#[component]
fn bold_points() {
    rsx! { <Bifurcation map={|r: f64, x: f64| r * x * (1. - x)} parameter={(2.8, 3.6)} steps={200} keep={40} point_size={3.} color={rgb(0x1e66f5)} class="h-80" /> }
}
```

## `<HrDiagram>`

This section also uses:

```rust
#[derive(Clone, PartialEq)]
pub struct Star {
    pub kelvin: f64,
    pub luminosity: f64,
    pub radius: f64,
}

fn stars() -> Vec<Star> {
    let mut out = Vec::new();
    for index in 0..250u64 {
        let t = 3000. + noise(index) * 27_000.;
        let luminosity = (t / 5800.).powf(5.5) * 10f64.powf(0.3 * normal(index + 300));
        out.push(Star {
            kelvin: t,
            luminosity,
            radius: (luminosity.sqrt() / (t / 5800.).powi(2)).max(0.1),
        });
    }
    for index in 0..40u64 {
        let t = 3500. + noise(index + 700) * 1500.;
        let luminosity = 100. * 10f64.powf(noise(index + 800) * 2.5);
        out.push(Star {
            kelvin: t,
            luminosity,
            radius: luminosity.sqrt() / (t / 5800.).powi(2),
        });
    }
    for index in 0..30u64 {
        let t = 8000. + noise(index + 900) * 20_000.;
        let luminosity = 0.001 * 10f64.powf(noise(index + 950) * 1.5);
        out.push(Star {
            kelvin: t,
            luminosity,
            radius: 0.1,
        });
    }
    out
}
```

### Stars

Hot stars on the left, bright stars on top, coloured by temperature: main sequence, giants and white dwarfs.

```rust
#[component]
fn sky() {
    let data = use_signal(stars());
    rsx! { <HrDiagram data={data} temperature={|s: &Star| s.kelvin} luminosity={|s: &Star| s.luminosity} class="h-96" /> }
}
```

### Sized by radius

Point size by stellar radius.

```rust
#[component]
fn sized() {
    let data = use_signal(stars());
    rsx! { <HrDiagram data={data} temperature={|s: &Star| s.kelvin} luminosity={|s: &Star| s.luminosity} radius={|s: &Star| s.radius} class="h-96" /> }
}
```

### Large points

Bigger points for a sparse sky.

```rust
#[component]
fn large_points() {
    let data = use_signal(stars());
    rsx! { <HrDiagram data={data} temperature={|s: &Star| s.kelvin} luminosity={|s: &Star| s.luminosity} point_size={9.} class="h-96" /> }
}
```

### Small points

Smaller points to separate crowded stars.

```rust
#[component]
fn small_points() {
    let data = use_signal(stars());
    rsx! { <HrDiagram data={data} temperature={|s: &Star| s.kelvin} luminosity={|s: &Star| s.luminosity} point_size={2.5} class="h-96" /> }
}
```

### No tooltip

Tooltip hidden.

```rust
#[component]
fn no_tooltip() {
    let data = use_signal(stars());
    rsx! { <HrDiagram data={data} temperature={|s: &Star| s.kelvin} luminosity={|s: &Star| s.luminosity} tooltip={Tooltip::hidden()} class="h-96" /> }
}
```

### Sized, no tooltip

Radius sizing without the tooltip.

```rust
#[component]
fn sized_no_tooltip() {
    let data = use_signal(stars());
    rsx! { <HrDiagram data={data} temperature={|s: &Star| s.kelvin} luminosity={|s: &Star| s.luminosity} radius={|s: &Star| s.radius} tooltip={Tooltip::hidden()} class="h-96" /> }
}
```

### Main sequence

Only the diagonal band of hydrogen-burning stars.

```rust
#[component]
fn main_sequence() {
    let data = use_signal(stars().into_iter().take(250).collect::<Vec<_>>());
    rsx! { <HrDiagram data={data} temperature={|s: &Star| s.kelvin} luminosity={|s: &Star| s.luminosity} class="h-96" /> }
}
```

### Red giants

Cool, luminous and very large stars.

```rust
#[component]
fn giants() {
    let data = use_signal(stars().into_iter().skip(250).take(40).collect::<Vec<_>>());
    rsx! { <HrDiagram data={data} temperature={|s: &Star| s.kelvin} luminosity={|s: &Star| s.luminosity} radius={|s: &Star| s.radius} class="h-96" /> }
}
```

### White dwarfs

Hot but faint remnants below the main sequence.

```rust
#[component]
fn white_dwarfs() {
    let data = use_signal(stars().into_iter().skip(290).collect::<Vec<_>>());
    rsx! { <HrDiagram data={data} temperature={|s: &Star| s.kelvin} luminosity={|s: &Star| s.luminosity} point_size={6.} class="h-96" /> }
}
```

### Cool stars

Only stars below 5000 K.

```rust
#[component]
fn cool_stars() {
    let data = use_signal(
        stars()
            .into_iter()
            .filter(|s| s.kelvin < 5000.)
            .collect::<Vec<_>>(),
    );
    rsx! { <HrDiagram data={data} temperature={|s: &Star| s.kelvin} luminosity={|s: &Star| s.luminosity} class="h-96" /> }
}
```

### Hot stars

Stars above 10 000 K sized by radius.

```rust
#[component]
fn hot_stars() {
    let data = use_signal(
        stars()
            .into_iter()
            .filter(|s| s.kelvin > 10_000.)
            .collect::<Vec<_>>(),
    );
    rsx! { <HrDiagram data={data} temperature={|s: &Star| s.kelvin} luminosity={|s: &Star| s.luminosity} radius={|s: &Star| s.radius} class="h-96" /> }
}
```

## `<Ramachandran>`

This section also uses:

```rust
#[derive(Clone, PartialEq)]
pub struct Residue {
    pub phi: f64,
    pub psi: f64,
}

fn protein() -> Vec<Residue> {
    (0..400u64)
        .map(|index| {
            let (phi, psi) = match index % 5 {
                0 | 1 => (-63., -43.),
                2 | 3 => (-120., 130.),
                _ => (60., 45.),
            };
            Residue {
                phi: phi + 12. * normal(index),
                psi: psi + 14. * normal(index + 2000),
            }
        })
        .collect()
}

fn cluster(count: u64, centre: (f64, f64), spread: f64, seed: u64) -> Vec<Residue> {
    (0..count)
        .map(|index| Residue {
            phi: centre.0 + spread * normal(seed + index),
            psi: centre.1 + spread * normal(seed + index + 5000),
        })
        .collect()
}
```

### Backbone angles

α-helix, β-sheet and left-handed clusters.

```rust
#[component]
fn backbone() {
    let data = use_signal(protein());
    rsx! { <Ramachandran data={data} phi={|r: &Residue| r.phi} psi={|r: &Residue| r.psi} class="h-96" /> }
}
```

### Small points

Smaller points for dense data.

```rust
#[component]
fn small() {
    let data = use_signal(protein());
    rsx! { <Ramachandran data={data} phi={|r: &Residue| r.phi} psi={|r: &Residue| r.psi} point_size={2.5} color={rgb(0x179299)} class="h-96" /> }
}
```

### α-helix only

A tight cluster near φ = −63°, ψ = −43°.

```rust
#[component]
fn helix() {
    let data = use_signal(cluster(300, (-63., -43.), 8., 100));
    rsx! { <Ramachandran data={data} phi={|r: &Residue| r.phi} psi={|r: &Residue| r.psi} color={rgb(0xd20f39)} class="h-96" /> }
}
```

### β-sheet only

A looser cluster in the upper left.

```rust
#[component]
fn sheet() {
    let data = use_signal(cluster(300, (-120., 130.), 15., 200));
    rsx! { <Ramachandran data={data} phi={|r: &Residue| r.phi} psi={|r: &Residue| r.psi} color={rgb(0x1e66f5)} class="h-96" /> }
}
```

### Left-handed helix

A rare cluster on the positive φ side.

```rust
#[component]
fn left_handed() {
    let data = use_signal(cluster(120, (60., 45.), 10., 300));
    rsx! { <Ramachandran data={data} phi={|r: &Residue| r.phi} psi={|r: &Residue| r.psi} point_size={5.} color={rgb(0x8839ef)} class="h-96" /> }
}
```

### Polyproline II

Extended chains near φ = −75°, ψ = 145°.

```rust
#[component]
fn polyproline() {
    let data = use_signal(cluster(200, (-75., 145.), 9., 400));
    rsx! { <Ramachandran data={data} phi={|r: &Residue| r.phi} psi={|r: &Residue| r.psi} color={rgb(0xfe640b)} class="h-96" /> }
}
```

### Disordered region

Wide scatter with no preferred conformation.

```rust
#[component]
fn disordered() {
    let data = use_signal(cluster(500, (-90., 30.), 60., 500));
    rsx! { <Ramachandran data={data} phi={|r: &Residue| r.phi} psi={|r: &Residue| r.psi} point_size={2.} color={rgb(0x7c7f93)} class="h-96" /> }
}
```

### Large points

Bigger points to emphasise the clusters.

```rust
#[component]
fn large() {
    let data = use_signal(protein());
    rsx! { <Ramachandran data={data} phi={|r: &Residue| r.phi} psi={|r: &Residue| r.psi} point_size={7.} color={rgb(0x40a02b)} class="h-96" /> }
}
```

### No tooltip

Tooltip hidden.

```rust
#[component]
fn no_tooltip() {
    let data = use_signal(protein());
    rsx! { <Ramachandran data={data} phi={|r: &Residue| r.phi} psi={|r: &Residue| r.psi} tooltip={Tooltip::hidden()} class="h-96" /> }
}
```

### Compact

A shorter chart in a custom colour.

```rust
#[component]
fn compact() {
    let data = use_signal(protein());
    rsx! { <Ramachandran data={data} phi={|r: &Residue| r.phi} psi={|r: &Residue| r.psi} point_size={3.} color={rgb(0xdf8e1d)} class="h-72" /> }
}
```

## `<Harker>`

This section also uses:

```rust
#[derive(Clone, PartialEq)]
pub struct Rock {
    pub silica: f64,
    pub alumina: f64,
    pub iron: f64,
    pub magnesia: f64,
    pub lime: f64,
    pub soda: f64,
    pub potash: f64,
}

fn suite() -> Vec<Rock> {
    (0..60u64)
        .map(|index| {
            let silica = 45. + noise(index) * 30.;
            let spread = |seed: u64| 0.4 * normal(index * 7 + seed);
            Rock {
                silica,
                alumina: 18. - (silica - 45.) * 0.12 + spread(1),
                iron: 12. - (silica - 45.) * 0.35 + spread(2),
                magnesia: 9. - (silica - 45.) * 0.28 + spread(3),
                lime: 11. - (silica - 45.) * 0.33 + spread(4),
                soda: 2.5 + (silica - 45.) * 0.04 + spread(5),
                potash: 0.5 + (silica - 45.) * 0.12 + spread(6),
            }
        })
        .collect()
}
```

### Volcanic suite

Six oxides against silica, one small plot each.

```rust
#[component]
fn volcanic() {
    let data = use_signal(suite());
    rsx! {
        <Harker data={data} silica={|r: &Rock| r.silica} class="h-[520px]">
            <Oxide name="Al₂O₃" value={|r: &Rock| r.alumina} />
            <Oxide name="FeO" value={|r: &Rock| r.iron} />
            <Oxide name="MgO" value={|r: &Rock| r.magnesia} />
            <Oxide name="CaO" value={|r: &Rock| r.lime} />
            <Oxide name="Na₂O" value={|r: &Rock| r.soda} />
            <Oxide name="K₂O" value={|r: &Rock| r.potash} />
        </Harker>
    }
}
```

### Two oxides

Two columns in a custom colour.

```rust
#[component]
fn two_columns() {
    let data = use_signal(suite());
    rsx! {
        <Harker data={data} silica={|r: &Rock| r.silica} columns={2} color={rgb(0xd20f39)} class="h-96">
            <Oxide name="MgO" value={|r: &Rock| r.magnesia} />
            <Oxide name="K₂O" value={|r: &Rock| r.potash} />
        </Harker>
    }
}
```

### One oxide

A single plot of alumina against silica.

```rust
#[component]
fn single() {
    let data = use_signal(suite());
    rsx! {
        <Harker data={data} silica={|r: &Rock| r.silica} color={rgb(0x179299)} class="h-72">
            <Oxide name="Al₂O₃" value={|r: &Rock| r.alumina} />
        </Harker>
    }
}
```

### Mafic oxides

FeO, MgO and CaO in three columns, all falling with silica.

```rust
#[component]
fn mafic_oxides() {
    let data = use_signal(suite());
    rsx! {
        <Harker data={data} silica={|r: &Rock| r.silica} columns={3} color={rgb(0x40a02b)} class="h-72">
            <Oxide name="FeO" value={|r: &Rock| r.iron} />
            <Oxide name="MgO" value={|r: &Rock| r.magnesia} />
            <Oxide name="CaO" value={|r: &Rock| r.lime} />
        </Harker>
    }
}
```

### Alkalis

Na₂O and K₂O rise with silica.

```rust
#[component]
fn alkalis() {
    let data = use_signal(suite());
    rsx! {
        <Harker data={data} silica={|r: &Rock| r.silica} columns={2} color={rgb(0x8839ef)} class="h-72">
            <Oxide name="Na₂O" value={|r: &Rock| r.soda} />
            <Oxide name="K₂O" value={|r: &Rock| r.potash} />
        </Harker>
    }
}
```

### Large points

Bigger points for a small suite.

```rust
#[component]
fn large_points() {
    let data = use_signal(suite());
    rsx! {
        <Harker data={data} silica={|r: &Rock| r.silica} columns={2} point_size={6.} class="h-96">
            <Oxide name="MgO" value={|r: &Rock| r.magnesia} />
            <Oxide name="CaO" value={|r: &Rock| r.lime} />
        </Harker>
    }
}
```

### Small points

Smaller points in a custom colour.

```rust
#[component]
fn small_points() {
    let data = use_signal(suite());
    rsx! {
        <Harker data={data} silica={|r: &Rock| r.silica} columns={2} point_size={2.} color={rgb(0xfe640b)} class="h-96">
            <Oxide name="FeO" value={|r: &Rock| r.iron} />
            <Oxide name="Al₂O₃" value={|r: &Rock| r.alumina} />
        </Harker>
    }
}
```

### Basalts

Only samples below 55 wt% silica.

```rust
#[component]
fn basalts() {
    let data = use_signal(
        suite()
            .into_iter()
            .filter(|r| r.silica < 55.)
            .collect::<Vec<_>>(),
    );
    rsx! {
        <Harker data={data} silica={|r: &Rock| r.silica} columns={2} color={rgb(0x1e66f5)} class="h-96">
            <Oxide name="MgO" value={|r: &Rock| r.magnesia} />
            <Oxide name="CaO" value={|r: &Rock| r.lime} />
        </Harker>
    }
}
```

### Rhyolites

Only samples above 65 wt% silica.

```rust
#[component]
fn rhyolites() {
    let data = use_signal(
        suite()
            .into_iter()
            .filter(|r| r.silica > 65.)
            .collect::<Vec<_>>(),
    );
    rsx! {
        <Harker data={data} silica={|r: &Rock| r.silica} columns={2} color={rgb(0xd20f39)} class="h-96">
            <Oxide name="Na₂O" value={|r: &Rock| r.soda} />
            <Oxide name="K₂O" value={|r: &Rock| r.potash} />
        </Harker>
    }
}
```

### Four columns

Four oxides in a single row.

```rust
#[component]
fn four_columns() {
    let data = use_signal(suite());
    rsx! {
        <Harker data={data} silica={|r: &Rock| r.silica} columns={4} class="h-72">
            <Oxide name="Al₂O₃" value={|r: &Rock| r.alumina} />
            <Oxide name="FeO" value={|r: &Rock| r.iron} />
            <Oxide name="MgO" value={|r: &Rock| r.magnesia} />
            <Oxide name="CaO" value={|r: &Rock| r.lime} />
        </Harker>
    }
}
```

## `<RootLocus>`

### 1 / s(s+2)(s+4)

Three branches; two cross into the right half-plane as gain rises.

```rust
#[component]
fn third_order() {
    rsx! { <RootLocus numerator={vec![1.]} denominator={vec![1., 6., 8., 0.]} gain={(0.01, 200.)} class="h-80" /> }
}
```

### A zero at −3

One branch ends at the zero, the others bend around it.

```rust
#[component]
fn with_zero() {
    rsx! { <RootLocus numerator={vec![1., 3.]} denominator={vec![1., 3., 2., 0.]} gain={(0.01, 100.)} class="h-80" /> }
}
```

### Complex poles

Branches leaving a pair of complex poles.

```rust
#[component]
fn complex_poles() {
    rsx! { <RootLocus numerator={vec![1.]} denominator={vec![1., 4., 13., 0.]} gain={(0.01, 300.)} class="h-80" /> }
}
```

### 1 / s(s+1)

Two poles meet on the real axis and break away.

```rust
#[component]
fn integrator() {
    rsx! { <RootLocus numerator={vec![1.]} denominator={vec![1., 1., 0.]} gain={(0.01, 50.)} class="h-80" /> }
}
```

### Four real poles

Poles at −1, −2, −3 and −4 collide pairwise.

```rust
#[component]
fn fourth_order() {
    rsx! { <RootLocus numerator={vec![1.]} denominator={vec![1., 10., 35., 50., 24.]} gain={(0.01, 300.)} class="h-80" /> }
}
```

### Unstable pole

A pole in the right half-plane that gain can pull across.

```rust
#[component]
fn unstable_pole() {
    rsx! { <RootLocus numerator={vec![1., 3.]} denominator={vec![1., 1., -2., 0.]} gain={(0.01, 40.)} class="h-80" /> }
}
```

### Two complex zeros

A pair of zeros attracts two of the branches.

```rust
#[component]
fn two_zeros() {
    rsx! { <RootLocus numerator={vec![1., 4., 8.]} denominator={vec![1., 5., 6., 0.]} gain={(0.01, 100.)} class="h-80" /> }
}
```

### Legend on the right

Poles, zeros and branches in a side legend.

```rust
#[component]
fn legend_right() {
    rsx! { <RootLocus numerator={vec![1.]} denominator={vec![1., 6., 8., 0.]} gain={(0.01, 200.)} legend={Legend::list().right()} class="h-80" /> }
}
```

### No legend

Legend hidden.

```rust
#[component]
fn legend_hidden() {
    rsx! { <RootLocus numerator={vec![1.]} denominator={vec![1., 6., 8., 0.]} gain={(0.01, 200.)} legend={Legend::hidden()} class="h-80" /> }
}
```

### No tooltip

Tooltip hidden.

```rust
#[component]
fn no_tooltip() {
    rsx! { <RootLocus numerator={vec![1., 3.]} denominator={vec![1., 3., 2., 0.]} gain={(0.01, 100.)} tooltip={Tooltip::hidden()} class="h-80" /> }
}
```

### 1000 gain steps

A smoother locus from more samples.

```rust
#[component]
fn fine_sampling() {
    rsx! { <RootLocus numerator={vec![1.]} denominator={vec![1., 4., 13., 0.]} gain={(0.01, 300.)} steps={1000} class="h-80" /> }
}
```

### Low gain range

Gain from 0.01 to 10 shows only the start of each branch.

```rust
#[component]
fn low_gain() {
    rsx! { <RootLocus numerator={vec![1.]} denominator={vec![1., 6., 8., 0.]} gain={(0.01, 10.)} class="h-80" /> }
}
```

## `<RadiationPattern>`

This section also uses:

```rust
#[derive(Clone, PartialEq)]
pub struct Lobe {
    pub angle: f64,
    pub dipole: f64,
    pub array: f64,
    pub omni: f64,
}

fn decibels(value: f64) -> f64 {
    20. * value.abs().max(1e-4).log10()
}

fn lobes() -> Vec<Lobe> {
    (0..=180)
        .map(|step| {
            let angle = f64::from(step) * 2.;
            let theta = angle.to_radians();
            let dipole =
                (std::f64::consts::FRAC_PI_2 * theta.cos()).cos() / theta.sin().abs().max(1e-6);
            let phase = std::f64::consts::PI * theta.cos();
            let array = (4. * phase / 2.).sin() / (4. * (phase / 2.).sin()).abs().max(1e-6);
            Lobe {
                angle,
                dipole: decibels(dipole),
                array: decibels(array),
                omni: -3. + 0.5 * (theta * 3.).cos(),
            }
        })
        .collect()
}
```

### Dipole and array

Gain in dB against angle; the array's narrow main lobe and side lobes show clearly.

```rust
#[component]
fn antennas() {
    let data = use_signal(lobes());
    rsx! {
        <RadiationPattern data={data} angle={|l: &Lobe| l.angle} class="h-96">
            <TrendSeries name="Dipole" y={|l: &Lobe| l.dipole} />
            <TrendSeries name="4-element array" y={|l: &Lobe| l.array} />
        </RadiationPattern>
    }
}
```

### 20 dB range

A shallower range and filled patterns for near-omnidirectional antennas.

```rust
#[component]
fn shallow() {
    let data = use_signal(lobes());
    rsx! {
        <RadiationPattern data={data} angle={|l: &Lobe| l.angle} min={-20.} levels={4} fill_opacity={0.25} class="h-96">
            <TrendSeries name="Omni" y={|l: &Lobe| l.omni} />
            <TrendSeries name="Dipole" y={|l: &Lobe| l.dipole} />
        </RadiationPattern>
    }
}
```

### Dipole

A single figure-eight pattern from −30 to 0 dB.

```rust
#[component]
fn dipole_only() {
    let data = use_signal(lobes());
    rsx! {
        <RadiationPattern data={data} angle={|l: &Lobe| l.angle} min={-30.} max={0.} class="h-96">
            <TrendSeries name="Dipole" y={|l: &Lobe| l.dipole} color={rgb(0xd20f39)} />
        </RadiationPattern>
    }
}
```

### Array

The four-element array over 40 dB with four rings.

```rust
#[component]
fn array_only() {
    let data = use_signal(lobes());
    rsx! {
        <RadiationPattern data={data} angle={|l: &Lobe| l.angle} min={-40.} levels={4} class="h-96">
            <TrendSeries name="4-element array" y={|l: &Lobe| l.array} color={rgb(0x8839ef)} />
        </RadiationPattern>
    }
}
```

### Filled lobes

fill_opacity 0.4 shades each pattern.

```rust
#[component]
fn filled() {
    let data = use_signal(lobes());
    rsx! {
        <RadiationPattern data={data} angle={|l: &Lobe| l.angle} min={-30.} fill_opacity={0.4} class="h-96">
            <TrendSeries name="Dipole" y={|l: &Lobe| l.dipole} />
            <TrendSeries name="4-element array" y={|l: &Lobe| l.array} />
        </RadiationPattern>
    }
}
```

### Thick lines

line_width 3 for a dipole against an omnidirectional reference.

```rust
#[component]
fn thick_lines() {
    let data = use_signal(lobes());
    rsx! {
        <RadiationPattern data={data} angle={|l: &Lobe| l.angle} min={-30.} line_width={3.} class="h-96">
            <TrendSeries name="Dipole" y={|l: &Lobe| l.dipole} />
            <TrendSeries name="Omni" y={|l: &Lobe| l.omni} />
        </RadiationPattern>
    }
}
```

### Twelve rings

A 60 dB range with a fine grid.

```rust
#[component]
fn many_rings() {
    let data = use_signal(lobes());
    rsx! {
        <RadiationPattern data={data} angle={|l: &Lobe| l.angle} min={-60.} levels={12} class="h-96">
            <TrendSeries name="4-element array" y={|l: &Lobe| l.array} />
        </RadiationPattern>
    }
}
```

### Legend on the right

Pattern names in a side legend.

```rust
#[component]
fn legend_right() {
    let data = use_signal(lobes());
    rsx! {
        <RadiationPattern data={data} angle={|l: &Lobe| l.angle} min={-30.} legend={Legend::list().right()} class="h-96">
            <TrendSeries name="Dipole" y={|l: &Lobe| l.dipole} />
            <TrendSeries name="4-element array" y={|l: &Lobe| l.array} />
        </RadiationPattern>
    }
}
```

### No legend

Legend hidden.

```rust
#[component]
fn legend_hidden() {
    let data = use_signal(lobes());
    rsx! {
        <RadiationPattern data={data} angle={|l: &Lobe| l.angle} min={-30.} legend={Legend::hidden()} class="h-96">
            <TrendSeries name="Dipole" y={|l: &Lobe| l.dipole} />
            <TrendSeries name="4-element array" y={|l: &Lobe| l.array} />
        </RadiationPattern>
    }
}
```

### No tooltip

Tooltip hidden.

```rust
#[component]
fn no_tooltip() {
    let data = use_signal(lobes());
    rsx! {
        <RadiationPattern data={data} angle={|l: &Lobe| l.angle} min={-30.} tooltip={Tooltip::hidden()} class="h-96">
            <TrendSeries name="Dipole" y={|l: &Lobe| l.dipole} />
            <TrendSeries name="4-element array" y={|l: &Lobe| l.array} />
        </RadiationPattern>
    }
}
```

## `<Quiver>`

This section also uses:

```rust
#[derive(Clone, PartialEq)]
pub struct Arrow {
    pub x: f64,
    pub y: f64,
    pub u: f64,
    pub v: f64,
}

fn grid(field: impl Fn(f64, f64) -> (f64, f64)) -> Vec<Arrow> {
    (0..15)
        .flat_map(|column| (0..15).map(move |line| (column, line)))
        .map(|(column, line)| {
            let (x, y) = (f64::from(column) / 7. - 1., f64::from(line) / 7. - 1.);
            let (u, v) = field(x, y);
            Arrow { x, y, u, v }
        })
        .collect()
}
```

### Vortex

A rotating flow; arrow length follows speed, scaled so the longest fits its cell.

```rust
#[component]
fn vortex() {
    let data = use_signal(grid(|x, y| (-y, x)));
    rsx! { <Quiver data={data} x={|a: &Arrow| a.x} y={|a: &Arrow| a.y} u={|a: &Arrow| a.u} v={|a: &Arrow| a.v} class="h-96" /> }
}
```

### Electric dipole

The field direction between a positive and a negative charge.

```rust
#[component]
fn dipole() {
    let data = use_signal(grid(|x, y| {
        let charge = |cx: f64, sign: f64| {
            let (dx, dy) = (x - cx, y);
            let r = (dx * dx + dy * dy).max(0.02).powf(1.5);
            (sign * dx / r, sign * dy / r)
        };
        let (a, b) = (charge(-0.5, 1.), charge(0.5, -1.));
        let (u, v) = (a.0 + b.0, a.1 + b.1);
        let length = u.hypot(v).max(1e-9);
        (u / length, v / length)
    }));
    rsx! { <Quiver data={data} x={|a: &Arrow| a.x} y={|a: &Arrow| a.y} u={|a: &Arrow| a.u} v={|a: &Arrow| a.v} color={rgb(0xd20f39)} class="h-96" /> }
}
```

### Shear flow

A fixed scale and a stretched aspect.

```rust
#[component]
fn shear() {
    let data = use_signal(grid(|_, y| (y + 1., 0.2 * (y * 3.).sin())));
    rsx! { <Quiver data={data} x={|a: &Arrow| a.x} y={|a: &Arrow| a.y} u={|a: &Arrow| a.u} v={|a: &Arrow| a.v} scale={0.06} equal_aspect={false} class="h-72" /> }
}
```

### Point source

A radial flow with arrows pointing away from the origin.

```rust
#[component]
fn source() {
    let data = use_signal(grid(|x, y| {
        let r = (x * x + y * y).max(0.02);
        (x / r, y / r)
    }));
    rsx! { <Quiver data={data} x={|a: &Arrow| a.x} y={|a: &Arrow| a.y} u={|a: &Arrow| a.u} v={|a: &Arrow| a.v} color={rgb(0x40a02b)} class="h-96" /> }
}
```

### Saddle flow

Inflow along y and outflow along x.

```rust
#[component]
fn saddle() {
    let data = use_signal(grid(|x, y| (x, -y)));
    rsx! { <Quiver data={data} x={|a: &Arrow| a.x} y={|a: &Arrow| a.y} u={|a: &Arrow| a.u} v={|a: &Arrow| a.v} color={rgb(0x8839ef)} class="h-96" /> }
}
```

### Uniform field

Every arrow identical.

```rust
#[component]
fn uniform() {
    let data = use_signal(grid(|_, _| (1., 0.5)));
    rsx! { <Quiver data={data} x={|a: &Arrow| a.x} y={|a: &Arrow| a.y} u={|a: &Arrow| a.u} v={|a: &Arrow| a.v} color={rgb(0x179299)} class="h-96" /> }
}
```

### Wave field

Sinusoidal components in both directions.

```rust
#[component]
fn wave() {
    let data = use_signal(grid(|x, y| ((y * 4.).sin(), (x * 4.).cos())));
    rsx! { <Quiver data={data} x={|a: &Arrow| a.x} y={|a: &Arrow| a.y} u={|a: &Arrow| a.u} v={|a: &Arrow| a.v} color={rgb(0xfe640b)} class="h-96" /> }
}
```

### Long arrows

scale 0.3 lets arrows overlap their neighbours.

```rust
#[component]
fn long_arrows() {
    let data = use_signal(grid(|x, y| (-y, x)));
    rsx! { <Quiver data={data} x={|a: &Arrow| a.x} y={|a: &Arrow| a.y} u={|a: &Arrow| a.u} v={|a: &Arrow| a.v} scale={0.3} class="h-96" /> }
}
```

### Short arrows

scale 0.03 shrinks them to ticks.

```rust
#[component]
fn short_arrows() {
    let data = use_signal(grid(|x, y| (-y, x)));
    rsx! { <Quiver data={data} x={|a: &Arrow| a.x} y={|a: &Arrow| a.y} u={|a: &Arrow| a.u} v={|a: &Arrow| a.v} scale={0.03} class="h-96" /> }
}
```

### Axis labels

Labelled x and y axes.

```rust
#[component]
fn labelled() {
    let data = use_signal(grid(|x, y| (-y, x)));
    rsx! { <Quiver data={data} x={|a: &Arrow| a.x} y={|a: &Arrow| a.y} u={|a: &Arrow| a.u} v={|a: &Arrow| a.v} x_axis={AxisConfig::new().label("x (m)")} y_axis={AxisConfig::new().label("y (m)")} class="h-96" /> }
}
```

### No tooltip

Tooltip hidden.

```rust
#[component]
fn no_tooltip() {
    let data = use_signal(grid(|x, y| (-y, x)));
    rsx! { <Quiver data={data} x={|a: &Arrow| a.x} y={|a: &Arrow| a.y} u={|a: &Arrow| a.u} v={|a: &Arrow| a.v} tooltip={Tooltip::hidden()} class="h-96" /> }
}
```

## `<Hodograph>`

This section also uses:

```rust
#[derive(Clone, PartialEq)]
pub struct Level {
    pub height: f64,
    pub u: f64,
    pub v: f64,
}

fn sounding(turn: f64, strength: f64) -> Vec<Level> {
    (0..=12)
        .map(|step| {
            let height = f64::from(step) * 0.75;
            let speed = strength * (5. + height * 5.5);
            let direction = (180. + turn * height).to_radians();
            Level {
                height,
                u: -speed * direction.sin(),
                v: -speed * direction.cos(),
            }
        })
        .collect()
}
```

### Veering winds

Winds turn clockwise with height, a curved hodograph; points are coloured from the ground up.

```rust
#[component]
fn veering() {
    let data = use_signal(sounding(18., 1.));
    rsx! { <Hodograph data={data} u={|l: &Level| l.u} v={|l: &Level| l.v} height={|l: &Level| l.height} class="h-96" /> }
}
```

### Straight-line shear

Little turning, rings every 10 m/s.

```rust
#[component]
fn straight() {
    let data = use_signal(sounding(2., 0.8));
    rsx! { <Hodograph data={data} u={|l: &Level| l.u} v={|l: &Level| l.v} height={|l: &Level| l.height} ring_step={10.} unit="m/s" scheme={Scheme::Purples} class="h-96" /> }
}
```

### Backing winds

Winds turn anticlockwise with height.

```rust
#[component]
fn backing() {
    let data = use_signal(sounding(-18., 1.));
    rsx! { <Hodograph data={data} u={|l: &Level| l.u} v={|l: &Level| l.v} height={|l: &Level| l.height} unit="m/s" class="h-96" /> }
}
```

### Strong veering

Fast, sharply turning winds with rings every 20 m/s.

```rust
#[component]
fn strong_veer() {
    let data = use_signal(sounding(30., 1.4));
    rsx! { <Hodograph data={data} u={|l: &Level| l.u} v={|l: &Level| l.v} height={|l: &Level| l.height} ring_step={20.} unit="m/s" scheme={Scheme::Oranges} class="h-96" /> }
}
```

### Weak winds

Light winds with rings every 5 m/s.

```rust
#[component]
fn weak_winds() {
    let data = use_signal(sounding(10., 0.4));
    rsx! { <Hodograph data={data} u={|l: &Level| l.u} v={|l: &Level| l.v} height={|l: &Level| l.height} ring_step={5.} unit="m/s" scheme={Scheme::Greens} class="h-96" /> }
}
```

### Single colour

Without a height accessor the points share one colour.

```rust
#[component]
fn no_height() {
    let data = use_signal(sounding(18., 1.));
    rsx! { <Hodograph data={data} u={|l: &Level| l.u} v={|l: &Level| l.v} color={rgb(0xd20f39)} class="h-96" /> }
}
```

### Reds scheme

Height mapped through the Reds scheme.

```rust
#[component]
fn reds() {
    let data = use_signal(sounding(18., 1.));
    rsx! { <Hodograph data={data} u={|l: &Level| l.u} v={|l: &Level| l.v} height={|l: &Level| l.height} scheme={Scheme::Reds} class="h-96" /> }
}
```

### Legend on the right

A side legend for the height scale.

```rust
#[component]
fn legend_right() {
    let data = use_signal(sounding(18., 1.));
    rsx! { <Hodograph data={data} u={|l: &Level| l.u} v={|l: &Level| l.v} height={|l: &Level| l.height} legend={Legend::list().right()} class="h-96" /> }
}
```

### No tooltip

Tooltip hidden.

```rust
#[component]
fn no_tooltip() {
    let data = use_signal(sounding(18., 1.));
    rsx! { <Hodograph data={data} u={|l: &Level| l.u} v={|l: &Level| l.v} height={|l: &Level| l.height} tooltip={Tooltip::hidden()} class="h-96" /> }
}
```

### Knots

Rings every 15 kt.

```rust
#[component]
fn knots() {
    let data = use_signal(sounding(25., 1.2));
    rsx! { <Hodograph data={data} u={|l: &Level| l.u} v={|l: &Level| l.v} height={|l: &Level| l.height} ring_step={15.} unit="kt" scheme={Scheme::Blues} class="h-96" /> }
}
```

## `<SmithChart>`

This section also uses:

```rust
#[derive(Clone, PartialEq)]
pub struct Sweep {
    pub resistance: f64,
    pub reactance: f64,
}

fn resonator(resistance: f64, inductance: f64, capacitance: f64) -> Vec<Sweep> {
    (0..=80)
        .map(|step| {
            let frequency = 1e6 * 10f64.powf(f64::from(step) / 40.);
            let omega = std::f64::consts::TAU * frequency;
            Sweep {
                resistance,
                reactance: omega * inductance - 1. / (omega * capacitance),
            }
        })
        .collect()
}

fn line(load: (f64, f64), reference: f64, turns: f64) -> Vec<Sweep> {
    (0..=120)
        .map(|step| {
            let beta = turns * std::f64::consts::PI * f64::from(step) / 120.;
            let (tan, (r, x)) = (beta.tan(), load);
            let (top_r, top_x) = (r, x + reference * tan);
            let (bottom_r, bottom_x) = (reference - x * tan, r * tan);
            let denominator = bottom_r.mul_add(bottom_r, bottom_x * bottom_x);
            Sweep {
                resistance: reference * top_r.mul_add(bottom_r, top_x * bottom_x) / denominator,
                reactance: reference * top_x.mul_add(bottom_r, -top_r * bottom_x) / denominator,
            }
        })
        .collect()
}

fn antenna() -> Vec<Sweep> {
    (0..=40)
        .map(|step| {
            let t = f64::from(step) / 40.;
            Sweep {
                resistance: 20. + 90. * (t * std::f64::consts::PI).sin(),
                reactance: -60. + 130. * t,
            }
        })
        .collect()
}
```

### Series RLC

A 30 Ω resonator swept from 1 to 100 MHz traces a circle of constant resistance.

```rust
#[component]
fn series_rlc() {
    let data = use_signal(resonator(30., 2e-6, 1.2e-9));
    rsx! { <SmithChart data={data} resistance={|s: &Sweep| s.resistance} reactance={|s: &Sweep| s.reactance} class="h-96" /> }
}
```

### Antenna match

Measured impedance against 75 Ω with a lighter grid.

```rust
#[component]
fn antenna_match() {
    let data = use_signal(antenna());
    rsx! { <SmithChart data={data} resistance={|s: &Sweep| s.resistance} reactance={|s: &Sweep| s.reactance} impedance={75.} grid={vec![0.25, 1., 4.]} color={rgb(0x8839ef)} class="h-96" /> }
}
```

### Admittance grid

overlay Admittance mirrors the grid for shunt matching.

```rust
#[component]
fn admittance() {
    let data = use_signal(antenna());
    rsx! { <SmithChart data={data} resistance={|s: &Sweep| s.resistance} reactance={|s: &Sweep| s.reactance} overlay={SmithGrid::Admittance} class="h-96" /> }
}
```

### Impedance and admittance

Both grids together, the admittance one dashed.

```rust
#[component]
fn both_grids() {
    let data = use_signal(antenna());
    rsx! { <SmithChart data={data} resistance={|s: &Sweep| s.resistance} reactance={|s: &Sweep| s.reactance} overlay={SmithGrid::Both} class="h-96" /> }
}
```

### Along a line

A 100 Ω load seen through half a wavelength of 50 Ω line circles the centre.

```rust
#[component]
fn mismatched_line() {
    let data = use_signal(line((100., 0.), 50., 1.));
    rsx! { <SmithChart data={data} resistance={|s: &Sweep| s.resistance} reactance={|s: &Sweep| s.reactance} class="h-96" /> }
}
```

### Complex load

25 + j40 Ω along a quarter wave.

```rust
#[component]
fn complex_load() {
    let data = use_signal(line((25., 40.), 50., 0.5));
    rsx! { <SmithChart data={data} resistance={|s: &Sweep| s.resistance} reactance={|s: &Sweep| s.reactance} overlay={SmithGrid::Both} color={rgb(0xfe640b)} class="h-96" /> }
}
```

### Fine grid

More resistance circles and reactance arcs.

```rust
#[component]
fn fine_grid() {
    let data = use_signal(resonator(50., 1e-6, 2e-9));
    rsx! { <SmithChart data={data} resistance={|s: &Sweep| s.resistance} reactance={|s: &Sweep| s.reactance} grid={vec![0.1, 0.2, 0.3, 0.5, 0.7, 1., 1.5, 2., 3., 5., 10.]} class="h-96" /> }
}
```

### 25 Ω system

A different reference impedance.

```rust
#[component]
fn low_impedance() {
    let data = use_signal(resonator(8., 0.5e-6, 4e-9));
    rsx! { <SmithChart data={data} resistance={|s: &Sweep| s.resistance} reactance={|s: &Sweep| s.reactance} impedance={25.} class="h-96" /> }
}
```

### Near the centre

A well-matched load stays close to Γ = 0.

```rust
#[component]
fn matched() {
    let data = use_signal(resonator(50., 0.2e-6, 20e-9));
    rsx! { <SmithChart data={data} resistance={|s: &Sweep| s.resistance} reactance={|s: &Sweep| s.reactance} point_size={3.} color={rgb(0x40a02b)} class="h-96" /> }
}
```

### Spot measurements

Separate loads joined in order.

```rust
#[component]
fn points_only() {
    let data = use_signal(
        [
            (10., 5.),
            (25., -30.),
            (50., 0.),
            (80., 60.),
            (150., -40.),
            (300., 120.),
        ]
        .map(|(resistance, reactance)| Sweep {
            resistance,
            reactance,
        })
        .to_vec(),
    );
    rsx! { <SmithChart data={data} resistance={|s: &Sweep| s.resistance} reactance={|s: &Sweep| s.reactance} point_size={7.} class="h-96" /> }
}
```

## `<EyeDiagram>`

This section also uses:

```rust
#[derive(Clone, PartialEq)]
pub struct Sample {
    pub time: f64,
    pub volts: f64,
}

fn bits(seed: u64, count: u64) -> Vec<f64> {
    (0..count)
        .map(|index| {
            if noise(seed * 7919 + index) > 0.5 {
                1.
            } else {
                0.
            }
        })
        .collect()
}

fn waveform(
    symbols: &[f64],
    per: usize,
    rise: f64,
    noise_level: f64,
    jitter: f64,
    seed: u64,
) -> Vec<Sample> {
    let mut level = symbols.first().copied().unwrap_or(0.);
    let mut out = Vec::with_capacity(symbols.len() * per);
    for (index, symbol) in symbols.iter().enumerate() {
        let shift = (noise(seed * 31 + index as u64) - 0.5) * jitter;
        for step in 0..per {
            let time = index as f64 + step as f64 / per as f64;
            level += (symbol - level) * rise;
            let wobble = normal(seed * 1_000_003 + (index * per + step) as u64) * noise_level;
            out.push(Sample {
                time: time + shift / per as f64,
                volts: level + wobble,
            });
        }
    }
    out
}

fn nrz(seed: u64, rise: f64, noise_level: f64, jitter: f64) -> Vec<Sample> {
    waveform(&bits(seed, 400), 32, rise, noise_level, jitter, seed)
}

fn pam4(seed: u64) -> Vec<Sample> {
    let symbols: Vec<f64> = (0..400u64)
        .map(|index| (noise(seed * 13 + index) * 4.).floor().min(3.) / 3.)
        .collect();
    waveform(&symbols, 32, 0.25, 0.015, 0.4, seed)
}
```

### Clean NRZ

400 random bits folded on two unit intervals; the strip below counts threshold crossings.

```rust
#[component]
fn clean() {
    let data = use_signal(nrz(1, 0.3, 0.01, 0.2));
    rsx! { <EyeDiagram data={data} x={|s: &Sample| s.time} value={|s: &Sample| s.volts} period={1.} offset={0.5} class="h-96" /> }
}
```

### Noisy link

Vertical noise closes the eye.

```rust
#[component]
fn noisy() {
    let data = use_signal(nrz(2, 0.3, 0.06, 0.2));
    rsx! { <EyeDiagram data={data} x={|s: &Sample| s.time} value={|s: &Sample| s.volts} period={1.} offset={0.5} class="h-96" /> }
}
```

### Slow edges

Inter-symbol interference smears the transitions.

```rust
#[component]
fn bandwidth_limited() {
    let data = use_signal(nrz(3, 0.12, 0.01, 0.2));
    rsx! { <EyeDiagram data={data} x={|s: &Sample| s.time} value={|s: &Sample| s.volts} period={1.} offset={0.5} class="h-96" /> }
}
```

### Timing jitter

Wide crossings and a wider jitter histogram.

```rust
#[component]
fn jittery() {
    let data = use_signal(nrz(4, 0.3, 0.01, 4.));
    rsx! { <EyeDiagram data={data} x={|s: &Sample| s.time} value={|s: &Sample| s.volts} period={1.} offset={0.5} bins={80} class="h-96" /> }
}
```

### PAM4

Four levels give three eyes.

```rust
#[component]
fn four_level() {
    let data = use_signal(pam4(5));
    rsx! { <EyeDiagram data={data} x={|s: &Sample| s.time} value={|s: &Sample| s.volts} period={1.} offset={0.5} threshold={0.5} color={rgb(0x8839ef)} class="h-96" /> }
}
```

### Three unit intervals

spans 3.

```rust
#[component]
fn three_spans() {
    let data = use_signal(nrz(6, 0.3, 0.02, 0.4));
    rsx! { <EyeDiagram data={data} x={|s: &Sample| s.time} value={|s: &Sample| s.volts} period={1.} offset={0.5} spans={3} class="h-96" /> }
}
```

### Crossing at the edge

offset 0 puts the crossings at the edges.

```rust
#[component]
fn centred() {
    let data = use_signal(nrz(7, 0.3, 0.02, 0.4));
    rsx! { <EyeDiagram data={data} x={|s: &Sample| s.time} value={|s: &Sample| s.volts} period={1.} offset={0.} class="h-96" /> }
}
```

### No histogram

jitter false hides the strip.

```rust
#[component]
fn traces_only() {
    let data = use_signal(nrz(8, 0.25, 0.02, 0.4));
    rsx! { <EyeDiagram data={data} x={|s: &Sample| s.time} value={|s: &Sample| s.volts} period={1.} offset={0.5} jitter={false} class="h-80" /> }
}
```

### Low opacity

Faint traces build up a density picture.

```rust
#[component]
fn bright() {
    let data = use_signal(nrz(9, 0.3, 0.02, 0.4));
    rsx! { <EyeDiagram data={data} x={|s: &Sample| s.time} value={|s: &Sample| s.volts} period={1.} offset={0.5} trace_opacity={0.06} color={rgb(0x179299)} class="h-96" /> }
}
```

### Labelled units

Axis labels in ns and V.

```rust
#[component]
fn nanoseconds() {
    let data = use_signal(
        nrz(10, 0.3, 0.015, 0.3)
            .into_iter()
            .map(|sample| Sample {
                time: sample.time * 0.8,
                volts: sample.volts * 0.8 - 0.4,
            })
            .collect::<Vec<_>>(),
    );
    rsx! { <EyeDiagram data={data} x={|s: &Sample| s.time} value={|s: &Sample| s.volts} period={0.8} offset={0.4} x_axis={AxisConfig::new().label("ns")} y_axis={AxisConfig::new().label("V")} class="h-96" /> }
}
```
