use blenderui::{AnyElement, Hsla, SharedString};
use blenderui::hooks::use_signal;

use super::{QUIET, Sketch, XySpec, nice, padded, series, xy_equal, xy_line, xy_points, xy_view};
use crate::derive::{Build, Table, column, sync};
use crate::plot::{AxisConfig, AxisPlacement, LineStyle, Tooltip};
use crate::xy_chart::Style as XyStyle;
use crate::{ChartTheme, Dataset, Legend, Scheme, component};

const WIDTH: usize = 5;
const SMITH: usize = 7;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SmithGrid {
    #[default]
    Impedance,
    Admittance,
    Both,
}

const fn quiet(style: XyStyle) -> XyStyle {
    XyStyle {
        hide_from: Some(QUIET),
        ..style
    }
}

fn hidden_axis(reach: f64) -> AxisConfig {
    AxisConfig::new()
        .min(-reach)
        .max(reach)
        .placement(AxisPlacement::Hidden)
        .grid(false)
}

#[component]
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub fn quiver<
    T: 'static,
    D: Dataset<T>,
    X: Fn(&T) -> f64 + 'static,
    Y: Fn(&T) -> f64 + 'static,
    U: Fn(&T) -> f64 + 'static,
    V: Fn(&T) -> f64 + 'static,
>(
    data: D,
    x: X,
    y: Y,
    u: U,
    v: V,
    scale: Option<f64>,
    equal_aspect: Option<bool>,
    #[into] color: Option<Hsla>,
    x_axis: Option<AxisConfig>,
    y_axis: Option<AxisConfig>,
    legend: Option<Legend>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
) -> AnyElement {
    let table = use_signal(Table::default());
    sync(
        table,
        &data,
        scale.map(f64::to_bits),
        |rows| {
            let vectors: Vec<[f64; 4]> = rows
                .iter()
                .map(|item| [x(item), y(item), u(item), v(item)])
                .filter(|cells| cells.iter().all(|value| value.is_finite()))
                .collect();
            let reach = |pick: usize| {
                let (low, high) = vectors
                    .iter()
                    .fold((f64::INFINITY, f64::NEG_INFINITY), |(low, high), cells| {
                        (low.min(cells[pick]), high.max(cells[pick]))
                    });
                (high - low).max(0.)
            };
            let longest = vectors
                .iter()
                .map(|cells| cells[2].hypot(cells[3]))
                .fold(0., f64::max);
            let spacing = (reach(0) * reach(1) / vectors.len().max(1) as f64).sqrt();
            let factor = scale.unwrap_or_else(|| {
                if longest > 0. && spacing > 0. {
                    spacing * 0.9 / longest
                } else {
                    1.
                }
            });
            let mut sketch = Sketch::new(WIDTH);
            for [px, py, pu, pv] in &vectors {
                let to = (pu.mul_add(factor, *px), pv.mul_add(factor, *py));
                let length = (pu * factor).hypot(pv * factor);
                sketch.arrow(0, (*px, *py), to, (length * 0.3).min(spacing * 0.3));
            }
            sketch.rows.extend(
                vectors.iter().map(|[px, py, pu, pv]| {
                    padded(WIDTH, &[(2, *px), (3, *py), (4, pu.hypot(*pv))])
                }),
            );
            Build {
                rows: sketch.rows,
                fields: ["x", "y", "x", "y", "Magnitude"]
                    .map(SharedString::Borrowed)
                    .to_vec(),
                ..Build::default()
            }
        });
    let tint = color.unwrap_or_else(|| ChartTheme::colors().series(0));
    let spec = XySpec {
        series: vec![
            series("Vectors", 0, 1, Some(tint), xy_line(1.5, LineStyle::Solid)),
            series("Points", 2, 3, Some(tint), quiet(xy_points(3.))),
        ],
        style: XyStyle::default(),
        x_axis: x_axis.unwrap_or_default(),
        y_axis: y_axis.unwrap_or_default(),
        legend: legend.or_else(|| Some(Legend::hidden())),
        tooltip,
        class,
    };
    if equal_aspect.unwrap_or(true) {
        xy_equal(table, spec)
    } else {
        xy_view(table, spec)
    }
}

#[component]
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub fn hodograph<
    T: 'static,
    D: Dataset<T>,
    U: Fn(&T) -> f64 + 'static,
    V: Fn(&T) -> f64 + 'static,
    H: Fn(&T) -> f64 + 'static,
>(
    data: D,
    u: U,
    v: V,
    height: Option<H>,
    ring_step: Option<f64>,
    unit: Option<SharedString>,
    scheme: Option<Scheme>,
    #[into] color: Option<Hsla>,
    legend: Option<Legend>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
) -> AnyElement {
    let table = use_signal(Table::default());
    sync(
        table,
        &data,
        (ring_step.map(f64::to_bits), height.is_some()),
        |rows| {
            let path: Vec<[f64; 3]> = rows
                .iter()
                .enumerate()
                .map(|(index, item)| {
                    [
                        u(item),
                        v(item),
                        height.as_ref().map_or(index as f64, |height| height(item)),
                    ]
                })
                .collect();
            let fastest = path
                .iter()
                .map(|cells| cells[0].hypot(cells[1]))
                .filter(|value| value.is_finite())
                .fold(0., f64::max);
            let step = ring_step
                .filter(|step| *step > 0.)
                .unwrap_or_else(|| nice(fastest / 4.));
            let rings = (fastest / step).ceil().max(1.) as usize;
            let reach = step * rings as f64;
            let mut sketch = Sketch::new(WIDTH);
            for ring in 1..=rings {
                sketch.ellipse(0, (0., 0.), (step * ring as f64, step * ring as f64), 96);
            }
            sketch.trace(0, [(-reach, 0.), (reach, 0.)]);
            sketch.trace(0, [(0., -reach), (0., reach)]);
            sketch.rows.extend(path.iter().map(|[east, north, level]| {
                padded(WIDTH, &[(2, *east), (3, *north), (4, *level)])
            }));
            Build {
                rows: sketch.rows,
                fields: ["u", "v", "u", "v", "Height"]
                    .map(SharedString::Borrowed)
                    .to_vec(),
                ..Build::default()
            }
        });
    let colors = ChartTheme::colors();
    let reach = table.with(|table| {
        table.rows.iter().filter_map(|row| row.first().copied()).filter(|value| value.is_finite()).fold(1., f64::max)
    });
    let unit = unit.unwrap_or(SharedString::Borrowed("kt"));
    let axis = |name: &str| {
        AxisConfig::new()
            .min(-reach * 1.05)
            .max(reach * 1.05)
            .label(format!("{name} ({unit})"))
    };
    xy_equal(
        table,
        XySpec {
            series: vec![
                series(
                    "Rings",
                    0,
                    1,
                    Some(colors.neutral.opacity(0.5)),
                    quiet(xy_line(1., LineStyle::Solid)),
                ),
                series(
                    "Wind",
                    2,
                    3,
                    Some(color.unwrap_or_else(|| colors.series(0))),
                    xy_line(2., LineStyle::Solid),
                ),
                series("Levels", 2, 3, None, quiet(xy_points(7.)))
                    .tinted(column(4), scheme.unwrap_or(Scheme::BlueYellowRed)),
            ],
            style: XyStyle::default(),
            x_axis: axis("u"),
            y_axis: axis("v"),
            legend: legend.or_else(|| Some(Legend::hidden())),
            tooltip,
            class,
        })
}

fn reflection(resistance: f64, reactance: f64) -> (f64, f64) {
    let denominator = (resistance + 1.).mul_add(resistance + 1., reactance * reactance);
    (
        resistance.mul_add(resistance, reactance.mul_add(reactance, -1.)) / denominator,
        2. * reactance / denominator,
    )
}

fn smith_grid(sketch: &mut Sketch, steps: &[f64], column: usize, sign: f64) {
    sketch.ellipse(column, (0., 0.), (1., 1.), 192);
    sketch.trace(column, [(-1., 0.), (1., 0.)]);
    for r in steps {
        let radius = 1. / (1. + r);
        sketch.ellipse(column, (sign * r / (1. + r), 0.), (radius, radius), 144);
    }
    for x in steps.iter().flat_map(|x| [*x, -*x]) {
        sketch.clipped(
            column,
            (0..=360).map(|step| {
                let t = std::f64::consts::TAU * f64::from(step) / 360.;
                (
                    sign * t.cos().mul_add(1. / x.abs(), 1.),
                    sign * t.sin().mul_add(1. / x.abs(), 1. / x),
                )
            }),
            |(a, b)| a.hypot(b) <= 1. + 1e-9,
        );
    }
}

#[component]
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub fn smith_chart<
    T: 'static,
    D: Dataset<T>,
    R: Fn(&T) -> f64 + 'static,
    X: Fn(&T) -> f64 + 'static,
>(
    data: D,
    resistance: R,
    reactance: X,
    impedance: Option<f64>,
    grid: Option<Vec<f64>>,
    overlay: Option<SmithGrid>,
    #[into] color: Option<Hsla>,
    point_size: Option<f32>,
    legend: Option<Legend>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
) -> AnyElement {
    let table = use_signal(Table::default());
    let reference = impedance.filter(|value| *value > 0.).unwrap_or(50.);
    let steps = grid.unwrap_or_else(|| vec![0.2, 0.5, 1., 2., 5.]);
    let overlay = overlay.unwrap_or_default();
    sync(
        table,
        &data,
        (reference.to_bits(), crate::derive::bits(&steps), overlay),
        |rows| {
            let mut sketch = Sketch::new(SMITH);
            if overlay != SmithGrid::Admittance {
                smith_grid(&mut sketch, &steps, 0, 1.);
            }
            if overlay != SmithGrid::Impedance {
                smith_grid(&mut sketch, &steps, 5, -1.);
            }
            sketch.rows.extend(rows.iter().map(|item| {
                let (r, x) = (resistance(item), reactance(item));
                let (a, b) = reflection(r / reference, x / reference);
                padded(SMITH, &[(2, a), (3, b), (4, a.hypot(b))])
            }));
            Build {
                rows: sketch.rows,
                fields: ["Re Γ", "Im Γ", "Re Γ", "Im Γ", "|Γ|", "Re Γ", "Im Γ"]
                    .map(SharedString::Borrowed)
                    .to_vec(),
                ..Build::default()
            }
        });
    let colors = ChartTheme::colors();
    let tint = color.unwrap_or_else(|| colors.series(0));
    xy_equal(
        table,
        XySpec {
            series: vec![
                series(
                    "Grid",
                    0,
                    1,
                    Some(colors.neutral.opacity(0.45)),
                    quiet(xy_line(1., LineStyle::Solid)),
                ),
                series(
                    "Admittance grid",
                    5,
                    6,
                    Some(colors.series(1).opacity(0.4)),
                    quiet(xy_line(1., LineStyle::Dash(&[4., 3.]))),
                ),
                series(
                    "Impedance",
                    2,
                    3,
                    Some(tint),
                    XyStyle {
                        show: Some(crate::xy_chart::Show::PointsAndLines),
                        point_size: Some(point_size.unwrap_or(5.)),
                        line_width: Some(2.),
                        ..XyStyle::default()
                    },
                ),
            ],
            style: XyStyle::default(),
            x_axis: hidden_axis(1.05),
            y_axis: hidden_axis(1.05),
            legend: legend.or_else(|| Some(Legend::hidden())),
            tooltip,
            class,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matched_load_sits_in_the_centre() {
        let (a, b) = reflection(1., 0.);
        assert!(a.abs() < 1e-12 && b.abs() < 1e-12);
        let (a, b) = reflection(0., 1.);
        assert!((a.hypot(b) - 1.).abs() < 1e-12);
    }
}
