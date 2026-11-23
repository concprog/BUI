use std::borrow::Cow;

use blenderui::element::{AnyElement, Div, FluentBuilder, IntoElement, ParentElement};
use blenderui::hooks::use_signal;
use blenderui::html::div;
use blenderui::{Hsla, SharedString, rgb};

use super::{
    Sketch, TrendSpec, XySpec, bars, field, padded, series, trend_view, xy_line as line,
    xy_points as points, xy_view,
};
use crate::data::Accessor;
use crate::derive::{Build, Row, Table, column, row, sync, sync_with};
use crate::plot::{AxisConfig, LineStyle, ScaleKind, Tooltip};
use crate::xy_chart::{PointShape, Style as XyStyle};
use crate::{ChartTheme, Dataset, Legend, Scheme, component};

fn noise(seed: u64) -> f64 {
    let mut z = seed.wrapping_add(0x9e37_79b9_7f4a_7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    (z ^ (z >> 31)) as f64 / u64::MAX as f64
}

fn inverse(matrix: &[Vec<f64>]) -> Option<Vec<Vec<f64>>> {
    let size = matrix.len();
    let mut work: Vec<Vec<f64>> = matrix
        .iter()
        .enumerate()
        .map(|(index, values)| {
            let mut extended = values.clone();
            extended.resize(size, 0.);
            extended.extend((0..size).map(|column| if column == index { 1. } else { 0. }));
            extended
        })
        .collect();
    for pivot in 0..size {
        let best =
            (pivot..size).max_by(|a, b| work[*a][pivot].abs().total_cmp(&work[*b][pivot].abs()))?;
        if work[best][pivot].abs() < 1e-12 {
            return None;
        }
        work.swap(pivot, best);
        let lead = work[pivot][pivot];
        for value in &mut work[pivot] {
            *value /= lead;
        }
        for other in 0..size {
            if other != pivot {
                let factor = work[other][pivot];
                let source = work[pivot].clone();
                for (value, base) in work[other].iter_mut().zip(source) {
                    *value -= factor * base;
                }
            }
        }
    }
    Some(
        work.into_iter()
            .map(|values| values[size..].to_vec())
            .collect(),
    )
}

fn product(matrix: &[Vec<f64>], vector: &[f64]) -> Vec<f64> {
    matrix
        .iter()
        .map(|values| values.iter().zip(vector).map(|(a, b)| a * b).sum())
        .collect()
}

fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(a, b)| a * b).sum()
}

fn risk(weights: &[f64], covariance: &[Vec<f64>]) -> f64 {
    dot(weights, &product(covariance, weights)).max(0.).sqrt()
}

fn share(value: f64) -> SharedString {
    format!("{:.1}%", value * 100.).into()
}

#[component]
#[allow(
    clippy::needless_pass_by_value,
    clippy::too_many_arguments,
    clippy::too_many_lines
)]
pub fn efficient_frontier(
    returns: Vec<f64>,
    covariance: Vec<Vec<f64>>,
    risk_free: Option<f64>,
    portfolios: Option<usize>,
    scheme: Option<Scheme>,
    legend: Option<Legend>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
) -> AnyElement {
    let table = use_signal(Table::default());
    let rate = risk_free.unwrap_or(0.);
    let count = portfolios.unwrap_or(2000);
    let settings = (
        crate::derive::bits(&returns),
        covariance
            .iter()
            .map(|values| crate::derive::bits(values))
            .collect::<Vec<_>>(),
        rate.to_bits(),
        count,
    );
    sync_with(
        table,
        settings,
        || {
            let assets = returns.len();
            let mut rows: Vec<Row> = Vec::new();
            let width = 11;
            for index in 0..count {
                let mut weights: Vec<f64> = (0..assets)
                    .map(|asset| -noise((index * 31 + asset) as u64 + 7).max(1e-9).ln())
                    .collect();
                let total: f64 = weights.iter().sum();
                for weight in &mut weights {
                    *weight /= total;
                }
                let (expected, spread) = (dot(&weights, &returns), risk(&weights, &covariance));
                let sharpe = if spread > 0. {
                    (expected - rate) / spread
                } else {
                    0.
                };
                rows.push(padded(width, &[(0, spread), (1, expected), (2, sharpe)]));
            }
            if let Some(inverse) = inverse(&covariance) {
                let ones = vec![1.; assets];
                let (with_returns, with_ones) =
                    (product(&inverse, &returns), product(&inverse, &ones));
                let (a, b, c) = (
                    dot(&ones, &with_returns),
                    dot(&returns, &with_returns),
                    dot(&ones, &with_ones),
                );
                let d = b.mul_add(c, -a * a);
                if d > 0. && c > 0. {
                    let floor = a / c;
                    let top = returns.iter().copied().fold(f64::NEG_INFINITY, f64::max) * 1.2;
                    for step in 0..=100 {
                        let target = floor + (top - floor) * f64::from(step) / 100.;
                        let variance = (c * target).mul_add(target, -2. * a * target) + b;
                        rows.push(padded(
                            width,
                            &[(3, (variance / d).max(0.).sqrt()), (4, target)],
                        ));
                    }
                }
                let excess: Vec<f64> = returns.iter().map(|value| value - rate).collect();
                let raw = product(&inverse, &excess);
                let sum: f64 = raw.iter().sum();
                if sum.abs() > 1e-12 {
                    let weights: Vec<f64> = raw.iter().map(|weight| weight / sum).collect();
                    let (expected, spread) = (dot(&weights, &returns), risk(&weights, &covariance));
                    rows.push(padded(width, &[(7, spread), (8, expected)]));
                    let slope = if spread > 0. {
                        (expected - rate) / spread
                    } else {
                        0.
                    };
                    rows.push(padded(width, &[(9, 0.), (10, rate)]));
                    rows.push(padded(
                        width,
                        &[(9, spread * 1.4), (10, slope.mul_add(spread * 1.4, rate))],
                    ));
                }
            }
            for (asset, expected) in returns.iter().enumerate() {
                let variance = covariance
                    .get(asset)
                    .and_then(|values| values.get(asset))
                    .copied()
                    .unwrap_or(0.);
                rows.push(padded(
                    width,
                    &[(5, variance.max(0.).sqrt()), (6, *expected)],
                ));
            }
            Build {
                rows,
                ..Build::default()
            }
        });
    let colors = ChartTheme::colors();
    let list = vec![
        series("Portfolios", 0, 1, None, points(3.))
            .tinted(column(2), scheme.unwrap_or(Scheme::BlueYellowRed)),
        series(
            "Efficient frontier",
            3,
            4,
            Some(colors.series(0)),
            line(2.5, LineStyle::Solid),
        ),
        series(
            "Capital market line",
            9,
            10,
            Some(colors.neutral),
            line(1.5, LineStyle::Dash(&[6., 4.])),
        ),
        series(
            "Assets",
            5,
            6,
            Some(colors.series(1)),
            XyStyle {
                point_shape: Some(PointShape::Square),
                ..points(9.)
            },
        ),
        series("Max Sharpe", 7, 8, Some(colors.bad), points(12.)),
    ];
    xy_view(
        table,
        XySpec {
            series: list,
            style: XyStyle::default(),
            x_axis: AxisConfig::new().label("Risk (σ)").format(share).min(0.),
            y_axis: AxisConfig::new().label("Expected return").format(share),
            legend,
            tooltip,
            class,
        })
}

fn rk4(field: &impl Fn(f64, f64) -> (f64, f64), (x, y): (f64, f64), step: f64) -> (f64, f64) {
    let (k1x, k1y) = field(x, y);
    let (k2x, k2y) = field(x + step / 2. * k1x, y + step / 2. * k1y);
    let (k3x, k3y) = field(x + step / 2. * k2x, y + step / 2. * k2y);
    let (k4x, k4y) = field(x + step * k3x, y + step * k3y);
    (
        x + step / 6. * (k1x + 2. * k2x + 2. * k3x + k4x),
        y + step / 6. * (k1y + 2. * k2y + 2. * k3y + k4y),
    )
}

#[component]
#[allow(
    clippy::needless_pass_by_value,
    clippy::too_many_arguments,
    clippy::too_many_lines
)]
pub fn phase_portrait<F: Fn(f64, f64) -> (f64, f64) + 'static>(
    field: F,
    starts: Vec<(f64, f64)>,
    x_range: (f64, f64),
    y_range: (f64, f64),
    duration: Option<f64>,
    step: Option<f64>,
    arrows: Option<usize>,
    x_label: Option<SharedString>,
    y_label: Option<SharedString>,
    #[into] color: Option<Hsla>,
    legend: Option<Legend>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
) -> AnyElement {
    let table = use_signal(Table::default());
    let (time, delta, density) = (
        duration.unwrap_or(10.),
        step.unwrap_or(0.01),
        arrows.unwrap_or(15),
    );
    let settings = (
        starts
            .iter()
            .map(|(x, y)| (x.to_bits(), y.to_bits()))
            .collect::<Vec<_>>(),
        [x_range.0, x_range.1, y_range.0, y_range.1].map(f64::to_bits),
        time.to_bits(),
        delta.to_bits(),
        density,
    );
    sync_with(
        table,
        settings,
        || {
            let mut sketch = Sketch::new(4);
            let (width, height) = (x_range.1 - x_range.0, y_range.1 - y_range.0);
            let cell = (width / density as f64).min(height / density as f64) * 0.4;
            for column in 0..=density {
                for line in 0..=density {
                    let x = x_range.0 + width * column as f64 / density as f64;
                    let y = y_range.0 + height * line as f64 / density as f64;
                    let (dx, dy) = field(x, y);
                    let length = dx.hypot(dy);
                    if length > 0. && length.is_finite() {
                        let (ux, uy) = (dx / length * cell, dy / length * cell);
                        sketch.arrow(
                            0,
                            (x - ux / 2., y - uy / 2.),
                            (x + ux / 2., y + uy / 2.),
                            cell * 0.35,
                        );
                    }
                }
            }
            let steps = (time / delta).ceil() as usize;
            let (left, right) = (x_range.0 - width, x_range.1 + width);
            let (bottom, top) = (y_range.0 - height, y_range.1 + height);
            for start in &starts {
                let mut at = *start;
                sketch.point(2, at);
                for _ in 0..steps {
                    at = rk4(&field, at, delta);
                    let inside = at.0.is_finite()
                        && at.1.is_finite()
                        && (left..=right).contains(&at.0)
                        && (bottom..=top).contains(&at.1);
                    if !inside {
                        break;
                    }
                    sketch.point(2, at);
                }
                sketch.lift();
            }
            Build {
                rows: sketch.rows,
                ..Build::default()
            }
        });
    let colors = ChartTheme::colors();
    let list = vec![
        series(
            "Direction field",
            0,
            1,
            Some(colors.neutral.opacity(0.6)),
            line(1., LineStyle::Solid),
        ),
        series(
            "Trajectories",
            2,
            3,
            Some(color.unwrap_or_else(|| colors.series(0))),
            line(2., LineStyle::Solid),
        ),
    ];
    xy_view(
        table,
        XySpec {
            series: list,
            style: XyStyle::default(),
            x_axis: AxisConfig::new()
                .min(x_range.0)
                .max(x_range.1)
                .label(x_label.unwrap_or(SharedString::Borrowed("x"))),
            y_axis: AxisConfig::new()
                .min(y_range.0)
                .max(y_range.1)
                .label(y_label.unwrap_or(SharedString::Borrowed("y"))),
            legend,
            tooltip: tooltip.or_else(|| Some(Tooltip::hidden())),
            class,
        })
}

#[component]
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub fn poincare_map<T: 'static, D: Dataset<T>, V: Fn(&T) -> f64 + 'static>(
    data: D,
    value: V,
    lag: Option<usize>,
    identity: Option<bool>,
    label: Option<SharedString>,
    #[into] color: Option<Hsla>,
    point_size: Option<f32>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
) -> AnyElement {
    let table = use_signal(Table::default());
    let lag = lag.unwrap_or(1).max(1);
    sync(
        table,
        &data,
        lag,
        |rows| {
            let values: Vec<f64> = rows.iter().map(&value).collect();
            let mut out: Vec<Row> = values
                .iter()
                .zip(values.iter().skip(lag))
                .map(|(now, next)| padded(4, &[(0, *now), (1, *next)]))
                .collect();
            let finite = || values.iter().copied().filter(|value| value.is_finite());
            let (low, high) = (
                finite().fold(f64::INFINITY, f64::min),
                finite().fold(f64::NEG_INFINITY, f64::max),
            );
            out.push(padded(4, &[(2, low), (3, low)]));
            out.push(padded(4, &[(2, high), (3, high)]));
            Build {
                rows: out,
                ..Build::default()
            }
        });
    let colors = ChartTheme::colors();
    let name = label.unwrap_or(SharedString::Borrowed("x"));
    let mut list = vec![series(
        format!("{name}ₙ → {name}ₙ₊{lag}"),
        0,
        1,
        Some(color.unwrap_or_else(|| colors.series(0))),
        points(point_size.unwrap_or(4.)),
    )];
    if identity.unwrap_or(true) {
        list.push(series(
            "y = x",
            2,
            3,
            Some(colors.neutral),
            line(1., LineStyle::Dash(&[5., 4.])),
        ));
    }
    xy_view(
        table,
        XySpec {
            series: list,
            style: XyStyle::default(),
            x_axis: AxisConfig::new().label(format!("{name}ₙ")),
            y_axis: AxisConfig::new().label(format!("{name}ₙ₊{lag}")),
            legend: Some(Legend::hidden()),
            tooltip,
            class,
        })
}

#[component]
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub fn bifurcation<F: Fn(f64, f64) -> f64 + 'static>(
    map: F,
    parameter: (f64, f64),
    steps: Option<usize>,
    transient: Option<usize>,
    keep: Option<usize>,
    start: Option<f64>,
    x_label: Option<SharedString>,
    y_label: Option<SharedString>,
    #[into] color: Option<Hsla>,
    point_size: Option<f32>,
    class: Option<SharedString>,
) -> AnyElement {
    let table = use_signal(Table::default());
    let (count, settle, sample, seed) = (
        steps.unwrap_or(300),
        transient.unwrap_or(300),
        keep.unwrap_or(60),
        start.unwrap_or(0.5),
    );
    sync_with(
        table,
        (
            parameter.0.to_bits(),
            parameter.1.to_bits(),
            count,
            settle,
            sample,
            seed.to_bits(),
        ),
        || {
            let mut rows: Vec<Row> = Vec::new();
            let mut orbit: Vec<f64> = Vec::with_capacity(sample);
            for step in 0..count {
                let r = parameter.0
                    + (parameter.1 - parameter.0) * step as f64 / (count.max(2) - 1) as f64;
                let mut x = seed;
                for _ in 0..settle {
                    x = map(r, x);
                }
                orbit.clear();
                for _ in 0..sample {
                    x = map(r, x);
                    if !x.is_finite() {
                        break;
                    }
                    orbit.push(x);
                }
                orbit.sort_unstable_by(f64::total_cmp);
                orbit.dedup_by(|a, b| (*a - *b).abs() <= 1e-6 * (1. + b.abs()));
                rows.extend(orbit.iter().map(|x| row([r, *x])));
            }
            Build {
                rows,
                ..Build::default()
            }
        });
    let colors = ChartTheme::colors();
    let list = vec![series(
        "Attractor",
        0,
        1,
        Some(color.unwrap_or_else(|| colors.series(0)).opacity(0.6)),
        XyStyle {
            point_stroke_width: Some(0.),
            ..points(point_size.unwrap_or(1.5))
        },
    )];
    xy_view(
        table,
        XySpec {
            series: list,
            style: XyStyle::default(),
            x_axis: AxisConfig::new()
                .min(parameter.0)
                .max(parameter.1)
                .label(x_label.unwrap_or(SharedString::Borrowed("r"))),
            y_axis: AxisConfig::new()
                .label(y_label.unwrap_or(SharedString::Borrowed("x"))),
            legend: Some(Legend::hidden()),
            tooltip: Some(Tooltip::hidden()),
            class,
        })
}

fn blackbody() -> Scheme {
    Scheme::Gradient(Cow::Owned(
        [
            0xff3800, 0xff8912, 0xffb46b, 0xffd6a5, 0xfff4ea, 0xe3e9ff, 0xcad7ff, 0x9bb0ff,
        ]
        .map(|hex| Hsla::from(rgb(hex)))
        .to_vec(),
    ))
}

#[component]
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub fn hr_diagram<
    T: 'static,
    D: Dataset<T>,
    K: Fn(&T) -> f64 + 'static,
    L: Fn(&T) -> f64 + 'static,
    R: Fn(&T) -> f64 + 'static,
>(
    data: D,
    temperature: K,
    luminosity: L,
    radius: Option<R>,
    point_size: Option<f32>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
) -> AnyElement {
    let table = use_signal(Table::default());
    sync(
        table,
        &data,
        radius.is_some(),
        |rows| Build {
            rows: rows
                .iter()
                .map(|item| {
                    let kelvin = temperature(item);
                    row([
                        kelvin,
                        luminosity(item),
                        kelvin.log10(),
                        radius.as_ref().map_or(1., |radius| radius(item)),
                    ])
                })
                .collect(),
            ..Build::default()
        });
    let mut stars = series("Stars", 0, 1, None, points(point_size.unwrap_or(6.)))
        .tinted(column(2), blackbody());
    if radius.is_some() {
        stars = stars.sized(column(3));
    }
    xy_view(
        table,
        XySpec {
            series: vec![stars],
            style: XyStyle {
                size_range: Some((3., 30.)),
                point_stroke_width: Some(0.),
                ..XyStyle::default()
            },
            x_axis: AxisConfig::new()
                .scale(ScaleKind::Log { base: 10. })
                .reverse(true)
                .label("Temperature (K)"),
            y_axis: AxisConfig::new()
                .scale(ScaleKind::Log { base: 10. })
                .label("Luminosity (L☉)"),
            legend: Some(Legend::hidden()),
            tooltip,
            class,
        })
}

#[component]
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub fn ramachandran<
    T: 'static,
    D: Dataset<T>,
    P: Fn(&T) -> f64 + 'static,
    S: Fn(&T) -> f64 + 'static,
>(
    data: D,
    phi: P,
    psi: S,
    #[into] color: Option<Hsla>,
    point_size: Option<f32>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
) -> AnyElement {
    let table = use_signal(Table::default());
    sync(
        table,
        &data,
        (),
        |rows| Build {
            rows: rows
                .iter()
                .map(|item| row([phi(item), psi(item)]))
                .collect(),
            ..Build::default()
        });
    let colors = ChartTheme::colors();
    let degrees = |label: &'static str| {
        AxisConfig::new()
            .min(-180.)
            .max(180.)
            .label(label)
            .format(|value| format!("{value:.0}°").into())
    };
    xy_view(
        table,
        XySpec {
            series: vec![series(
                "Residues",
                0,
                1,
                Some(color.unwrap_or_else(|| colors.series(0))),
                points(point_size.unwrap_or(4.)),
            )],
            style: XyStyle::default(),
            x_axis: degrees("φ"),
            y_axis: degrees("ψ"),
            legend: Some(Legend::hidden()),
            tooltip,
            class,
        })
}

pub struct OxideField<T> {
    name: SharedString,
    value: Accessor<T>,
}

#[component]
pub fn oxide<T: 'static, V: Fn(&T) -> f64 + 'static>(
    name: SharedString,
    value: V,
) -> OxideField<T> {
    OxideField {
        name,
        value: Accessor::row(value),
    }
}

#[component]
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub fn harker<T: 'static, D: Dataset<T>, S: Fn(&T) -> f64 + 'static>(
    data: D,
    silica: S,
    columns: Option<usize>,
    #[into] color: Option<Hsla>,
    point_size: Option<f32>,
    class: Option<SharedString>,
    children: Children<OxideField<T>>,
) -> AnyElement {
    let table = use_signal(Table::default());
    let oxides = children;
    let fingerprint: Vec<SharedString> = oxides.iter().map(|oxide| oxide.name.clone()).collect();
    sync(
        table,
        &data,
        fingerprint,
        |rows| {
            let mut column = Vec::new();
            let mut cells: Vec<Vec<f64>> = rows.iter().map(|item| vec![silica(item)]).collect();
            for oxide in &oxides {
                oxide.value.fill(rows, &mut column);
                for (row, value) in cells.iter_mut().zip(&column) {
                    row.push(*value);
                }
            }
            Build {
                rows: cells.into_iter().map(Vec::into_boxed_slice).collect(),
                ..Build::default()
            }
        });
    let colors = ChartTheme::colors();
    let tint = color.unwrap_or_else(|| colors.series(0));
    let size = point_size.unwrap_or(5.);
    let per_row = columns.unwrap_or(3).max(1);
    let mut grid: Vec<Div> = Vec::new();
    for (start, chunk) in oxides.chunks(per_row).enumerate() {
        let mut cells: Vec<Div> = Vec::new();
        for (offset, oxide) in chunk.iter().enumerate() {
            let index = start * per_row + offset;
            let name = oxide.name.clone();
            let element = xy_view(
                    table,
                    XySpec {
                        series: vec![series(name.clone(), 0, index + 1, Some(tint), points(size))],
                        style: XyStyle::default(),
                        x_axis: AxisConfig::new().label("SiO₂ wt%"),
                        y_axis: AxisConfig::new().label(format!("{name} wt%")),
                        legend: Some(Legend::hidden()),
                        tooltip: None,
                        class: None,
                    });
            cells.push(div().class("flex-1 min-w-0 min-h-0").child(element));
        }
        for _ in chunk.len()..per_row {
            cells.push(div().class("flex-1"));
        }
        grid.push(div().class("flex flex-1 min-h-0 gap-2").children(cells));
    }
    div().class("flex flex-col size-full gap-2").when_some(class, Div::class).children(grid).into_any_element()
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Complex {
    re: f64,
    im: f64,
}

impl Complex {
    const fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    fn add(self, other: Self) -> Self {
        Self::new(self.re + other.re, self.im + other.im)
    }

    fn sub(self, other: Self) -> Self {
        Self::new(self.re - other.re, self.im - other.im)
    }

    fn mul(self, other: Self) -> Self {
        Self::new(
            self.re.mul_add(other.re, -self.im * other.im),
            self.re.mul_add(other.im, self.im * other.re),
        )
    }

    fn div(self, other: Self) -> Self {
        let scale = other.re.mul_add(other.re, other.im * other.im);
        Self::new(
            self.re.mul_add(other.re, self.im * other.im) / scale,
            self.im.mul_add(other.re, -self.re * other.im) / scale,
        )
    }

    fn distance(self, other: Self) -> f64 {
        (self.re - other.re).hypot(self.im - other.im)
    }
}

pub(crate) fn roots(coefficients: &[f64]) -> Vec<(f64, f64)> {
    let start = coefficients
        .iter()
        .position(|value| value.abs() > 1e-12)
        .unwrap_or(coefficients.len());
    let poly = &coefficients[start..];
    let degree = poly.len().saturating_sub(1);
    if degree == 0 {
        return Vec::new();
    }
    let lead = poly[0];
    let monic: Vec<f64> = poly.iter().map(|value| value / lead).collect();
    let evaluate = |at: Complex| {
        monic.iter().fold(Complex::new(0., 0.), |sum, value| {
            sum.mul(at).add(Complex::new(*value, 0.))
        })
    };
    let seed = Complex::new(0.4, 0.9);
    let mut found: Vec<Complex> = (0..degree)
        .map(|index| {
            let mut power = Complex::new(1., 0.);
            for _ in 0..index {
                power = power.mul(seed);
            }
            power
        })
        .collect();
    for _ in 0..500 {
        let mut moved = 0f64;
        for index in 0..degree {
            let current = found[index];
            let mut denominator = Complex::new(1., 0.);
            for (other, value) in found.iter().enumerate() {
                if other != index {
                    denominator = denominator.mul(current.sub(*value));
                }
            }
            let next = current.sub(evaluate(current).div(denominator));
            moved = moved.max(next.distance(current));
            found[index] = next;
        }
        if moved < 1e-12 {
            break;
        }
    }
    found.into_iter().map(|root| (root.re, root.im)).collect()
}

#[component]
#[allow(
    clippy::needless_pass_by_value,
    clippy::too_many_arguments,
    clippy::too_many_lines
)]
#[must_use]
pub fn root_locus(
    numerator: Vec<f64>,
    denominator: Vec<f64>,
    gain: Option<(f64, f64)>,
    steps: Option<usize>,
    legend: Option<Legend>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
) -> AnyElement {
    let table = use_signal(Table::default());
    let (low, high) = gain.unwrap_or((0., 100.));
    let count = steps.unwrap_or(400).max(2);
    sync_with(
        table,
        (
            crate::derive::bits(&numerator),
            crate::derive::bits(&denominator),
            low.to_bits(),
            high.to_bits(),
            count,
        ),
        || {
            let order = denominator.len().max(numerator.len());
            let pad = |values: &[f64]| {
                let mut out = vec![0.; order - values.len()];
                out.extend_from_slice(values);
                out
            };
            let (top, bottom) = (pad(&numerator), pad(&denominator));
            let poles = roots(&denominator);
            let zeros = roots(&numerator);
            let branches = poles.len();
            let width = branches * 2 + 4;
            let mut rows: Vec<Row> = Vec::new();
            let mut previous: Vec<(f64, f64)> = poles.clone();
            for step in 0..count {
                let fraction = step as f64 / (count - 1) as f64;
                let k = if low > 0. {
                    low * (high / low).powf(fraction)
                } else {
                    (high - low).mul_add(fraction * fraction, low)
                };
                let polynomial: Vec<f64> = bottom
                    .iter()
                    .zip(&top)
                    .map(|(d, n)| k.mul_add(*n, *d))
                    .collect();
                let mut found = roots(&polynomial);
                let mut ordered = Vec::with_capacity(branches);
                for last in &previous {
                    let Some(best) = (0..found.len()).min_by(|a, b| {
                        let distance = |root: (f64, f64)| (root.0 - last.0).hypot(root.1 - last.1);
                        distance(found[*a]).total_cmp(&distance(found[*b]))
                    }) else {
                        break;
                    };
                    ordered.push(found.swap_remove(best));
                }
                let mut cells = vec![f64::NAN; width];
                for (branch, root) in ordered.iter().enumerate() {
                    cells[branch * 2] = root.0;
                    cells[branch * 2 + 1] = root.1;
                }
                rows.push(cells.into_boxed_slice());
                if ordered.len() == branches {
                    previous = ordered;
                }
            }
            for pole in &poles {
                rows.push(padded(width, &[(width - 4, pole.0), (width - 3, pole.1)]));
            }
            for zero in &zeros {
                rows.push(padded(width, &[(width - 2, zero.0), (width - 1, zero.1)]));
            }
            Build {
                rows,
                fields: (0..branches)
                    .map(|branch| SharedString::from(format!("Branch {}", branch + 1)))
                    .collect(),
                ..Build::default()
            }
        });
    let branches = table.with(|table| table.fields.clone());
    let width = branches.len() * 2 + 4;
    let colors = ChartTheme::colors();
    let mut list: Vec<_> = branches
        .into_iter()
        .enumerate()
        .map(|(index, name)| {
            series(
                name,
                index * 2,
                index * 2 + 1,
                Some(colors.series(index)),
                line(2., LineStyle::Solid),
            )
        })
        .collect();
    list.push(series(
        "Poles",
        width - 4,
        width - 3,
        Some(colors.bad),
        XyStyle {
            point_shape: Some(PointShape::Square),
            ..points(10.)
        },
    ));
    list.push(series(
        "Zeros",
        width - 2,
        width - 1,
        Some(colors.good),
        XyStyle {
            fill_opacity: Some(0.),
            point_stroke_width: Some(2.),
            ..points(10.)
        },
    ));
    xy_view(
        table,
        XySpec {
            series: list,
            style: XyStyle::default(),
            x_axis: AxisConfig::new().label("Real"),
            y_axis: AxisConfig::new().label("Imaginary"),
            legend,
            tooltip,
            class,
        })
}

fn level(values: impl Iterator<Item = f64>) -> f64 {
    let (low, high) = values
        .filter(|value| value.is_finite())
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(low, high), value| {
            (low.min(value), high.max(value))
        });
    if low <= high {
        f64::midpoint(low, high)
    } else {
        0.
    }
}

pub(crate) fn fold_traces(samples: &[(f64, f64)], window: f64, offset: f64, sketch: &mut Sketch) {
    let phase = |time: f64| (time - offset).rem_euclid(window);
    let mut previous: Option<(f64, f64, f64)> = None;
    for &(time, value) in samples {
        if !(time.is_finite() && value.is_finite()) {
            previous = None;
            sketch.lift();
            continue;
        }
        let now = phase(time);
        if let Some((before, last, then)) = previous
            && now < before - window / 2.
        {
            let span = time - then;
            let fraction = if span > 0. {
                (window - before) / span
            } else {
                0.
            };
            let edge = (value - last).mul_add(fraction, last);
            sketch.point(0, (window, edge));
            sketch.lift();
            sketch.point(0, (0., edge));
        }
        sketch.point(0, (now, value));
        previous = Some((now, value, time));
    }
    sketch.lift();
}

pub(crate) fn crossings(
    samples: &[(f64, f64)],
    threshold: f64,
    window: f64,
    offset: f64,
    bins: usize,
) -> Vec<f64> {
    let mut counts = vec![0.; bins.max(1)];
    for pair in samples.windows(2) {
        let ((t0, v0), (t1, v1)) = (pair[0], pair[1]);
        if (v0 - threshold) * (v1 - threshold) < 0. {
            let at = (threshold - v0) / (v1 - v0) * (t1 - t0) + t0;
            let slot = ((at - offset).rem_euclid(window) / window * counts.len() as f64) as usize;
            let last = counts.len() - 1;
            counts[slot.min(last)] += 1.;
        }
    }
    counts
}

#[component]
#[allow(
    clippy::needless_pass_by_value,
    clippy::too_many_arguments,
    clippy::too_many_lines
)]
pub fn eye_diagram<
    T: 'static,
    D: Dataset<T>,
    X: Fn(&T) -> f64 + 'static,
    V: Fn(&T) -> f64 + 'static,
>(
    data: D,
    x: X,
    value: V,
    period: f64,
    offset: Option<f64>,
    spans: Option<usize>,
    threshold: Option<f64>,
    jitter: Option<bool>,
    bins: Option<usize>,
    trace_opacity: Option<f32>,
    #[into] color: Option<Hsla>,
    x_axis: Option<AxisConfig>,
    y_axis: Option<AxisConfig>,
    class: Option<SharedString>,
) -> AnyElement {
    let traces = use_signal(Table::default());
    let histogram = use_signal(Table::default());
    let cycle = period.abs().max(f64::MIN_POSITIVE) * spans.unwrap_or(2).max(1) as f64;
    let (start, count, strip) = (
        offset.unwrap_or(0.),
        bins.unwrap_or(60).max(1),
        jitter.unwrap_or(true),
    );
    let settings = (
        cycle.to_bits(),
        start.to_bits(),
        threshold.map(f64::to_bits),
        count,
    );
    let samples = |rows: &[T]| -> Vec<(f64, f64)> {
        rows.iter().map(|item| (x(item), value(item))).collect()
    };
    sync(
        traces,
        &data,
        settings,
        |rows| {
            let mut sketch = Sketch::new(2);
            fold_traces(&samples(rows), cycle, start, &mut sketch);
            Build {
                rows: sketch.rows,
                ..Build::default()
            }
        });
    if strip {
        sync(
            histogram,
            &data,
            settings,
            |rows| {
                let pairs = samples(rows);
                let middle = threshold.unwrap_or_else(|| level(pairs.iter().map(|pair| pair.1)));
                let width = cycle / count as f64;
                Build {
                    rows: crossings(&pairs, middle, cycle, start, count)
                        .into_iter()
                        .enumerate()
                        .map(|(slot, total)| row([(slot as f64 + 0.5) * width, total]))
                        .collect(),
                    ..Build::default()
                }
            });
    }
    let (tint, bar) = {
        let colors = ChartTheme::colors();
        (color.unwrap_or_else(|| colors.series(0)), colors.series(1))
    };
    let time = x_axis.unwrap_or_default().min(0.).max(cycle);
    let eye = xy_view(
            traces,
            XySpec {
                series: vec![series(
                    "Traces",
                    0,
                    1,
                    Some(tint.opacity(trace_opacity.unwrap_or(0.25))),
                    line(1., LineStyle::Solid),
                )],
                style: XyStyle::default(),
                x_axis: time.clone(),
                y_axis: y_axis.unwrap_or_default(),
                legend: Some(Legend::hidden()),
                tooltip: Some(Tooltip::hidden()),
                class: None,
            });
    let spread = strip.then(|| {
            trend_view(
                histogram,
                TrendSpec {
                    fields: vec![field("Crossings", 1, Some(bar), bars(1.))],
                    style: crate::trend::Style::default(),
                    defaults: crate::trend::Defaults::new(
                        time,
                        AxisConfig::new().min(0.).label("Crossings"),
                    ),
                    legend: Some(Legend::hidden()),
                    tooltip: None,
                    class: None,
                })
    });
    div()
        .class("flex flex-col size-full min-w-0 min-h-0")
        .when_some(class, Div::class)
        .child(div().class("flex-1 min-h-0").child(eye))
        .child(spread.map(|strip| div().class("h-1/4 min-h-[64px]").child(strip)))
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eye_folds_on_the_window() {
        let samples: Vec<(f64, f64)> = (0..8)
            .map(|step| (f64::from(step) * 0.5, f64::from(step % 2)))
            .collect();
        let mut sketch = Sketch::new(2);
        fold_traces(&samples, 2., 0., &mut sketch);
        let breaks = sketch.rows.iter().filter(|row| row[0].is_nan()).count();
        assert_eq!(breaks, 2);
        assert!((crossings(&samples, 0.5, 2., 0., 4).iter().sum::<f64>() - 7.).abs() < 1e-9);
    }

    #[test]
    fn roots_of_a_quadratic() {
        let mut found = roots(&[1., -3., 2.]);
        found.sort_by(|a, b| a.0.total_cmp(&b.0));
        assert!((found[0].0 - 1.).abs() < 1e-9 && (found[1].0 - 2.).abs() < 1e-9);
        let complex = roots(&[1., 0., 1.]);
        assert!(
            complex
                .iter()
                .all(|(re, im)| re.abs() < 1e-9 && (im.abs() - 1.).abs() < 1e-9)
        );
    }

    #[test]
    fn inverse_of_a_diagonal() {
        let found = inverse(&[vec![2., 0.], vec![0., 4.]]).unwrap_or_default();
        assert!((found[0][0] - 0.5).abs() < 1e-12 && (found[1][1] - 0.25).abs() < 1e-12);
    }
}

pub(crate) fn trigger_time(times: &[f64], values: &[f64], level: f64) -> Option<f64> {
    values.windows(2).enumerate().find_map(|(index, pair)| {
        (pair[0] < level && pair[1] >= level).then(|| {
            let span = pair[1] - pair[0];
            let fraction = if span.abs() > 0. {
                (level - pair[0]) / span
            } else {
                0.
            };
            (times[index + 1] - times[index]).mul_add(fraction, times[index])
        })
    })
}

#[component]
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub fn oscilloscope<T: 'static, D: Dataset<T>, X: Fn(&T) -> f64 + 'static>(
    data: D,
    x: X,
    trigger: Option<f64>,
    span: Option<f64>,
    y_axis: Option<AxisConfig>,
    legend: Option<Legend>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
    children: Children<crate::trend::TrendField<T>>,
) -> AnyElement {
    use crate::trend::{Annotation, Annotations, Chart, Defaults, PointsMode, Style, chart};
    let mut fields = children;
    let (start, fired) = data.rows(|rows| {
        let times: Vec<f64> = rows.iter().map(&x).collect();
        let start = times.iter().copied().find(|value| value.is_finite()).unwrap_or(0.);
        let fired = trigger.and_then(|level| {
            let mut values = Vec::new();
            fields.first()?.y.fill(rows, &mut values);
            trigger_time(&times, &values, level)
        });
        (start, fired)
    });
    let origin = fired.unwrap_or(start);
    let mut axis = AxisConfig::new().label("Time (s)");
    if let Some(span) = span {
        axis = axis.min(origin).max(origin + span);
    }
    let colors = ChartTheme::colors();
    if let Some(level) = trigger {
        let mut marker = crate::trend::TrendField::new(
            SharedString::Borrowed("Trigger"),
            Accessor::row(move |_: &T| level),
            Some(colors.neutral),
            Style {
                transform: Some(crate::trend::ValueTransform::Constant),
                line_style: Some(LineStyle::Dash(&[4., 4.])),
                hide_from: Some(super::QUIET),
                ..Style::default()
            },
        );
        marker.style.line_width = Some(1.);
        fields.push(marker);
    }
    let notes = fired.map(|at| Annotations::Owned(vec![Annotation::at(at, "Trigger")]));
    chart(
        Chart {
            data,
            x: Some(Accessor::row(x)),
            x_field: None,
            ends: None,
            fields,
            style: Style {
                line_width: Some(1.5),
                show_points: Some(PointsMode::Never),
                ..Style::default()
            },
            defaults: Defaults::new(
                axis,
                y_axis.unwrap_or_else(|| AxisConfig::new().label("Volts")),
            ),
            annotations: notes,
            compare: None,
            keyboard: true,
            legend,
            tooltip: tooltip.or_else(|| Some(Tooltip::multi())),
            class,
        },
        None::<fn(crate::plot::SelectedRange, &mut blenderui::Window, &mut blenderui::App)>)
}
