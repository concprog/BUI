use blenderui::{AnyElement, Hsla, SharedString};
use crate::component;
use blenderui::hooks::use_signal;

use super::{
    QUIET, TrendSpec, XySpec, deviation, dots, field, lines, mean, series, trend_view, xy_line,
    xy_view,
};
use crate::derive::{Build, Row, Table, row, sync};
use crate::plot::{AxisConfig, LineStyle, Tooltip};
use crate::trend::{Defaults, Style, ValueTransform};
use crate::xy_chart::Style as XyStyle;
use crate::{ChartTheme, Dataset, Legend};

const D2: f64 = 1.128;

pub(crate) fn moving_sigma(values: &[f64]) -> f64 {
    let finite: Vec<f64> = values
        .iter()
        .copied()
        .filter(|value| value.is_finite())
        .collect();
    let ranges: Vec<f64> = finite
        .windows(2)
        .map(|pair| (pair[1] - pair[0]).abs())
        .collect();
    if ranges.is_empty() {
        return 0.;
    }
    ranges.iter().sum::<f64>() / ranges.len() as f64 / D2
}

#[component]
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub fn control_chart<
    T: 'static,
    D: Dataset<T>,
    X: Fn(&T) -> f64 + 'static,
    V: Fn(&T) -> f64 + 'static,
>(
    data: D,
    x: X,
    value: V,
    center: Option<f64>,
    sigma: Option<f64>,
    limit: Option<f64>,
    zones: Option<bool>,
    #[into] color: Option<Hsla>,
    x_axis: Option<AxisConfig>,
    y_axis: Option<AxisConfig>,
    legend: Option<Legend>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
) -> AnyElement {
    let table = use_signal(Table::default());
    let width = limit.unwrap_or(3.);
    sync(
        table,
        &data,
        (
            center.map(f64::to_bits),
            sigma.map(f64::to_bits),
            width.to_bits(),
        ),
        |rows| {
            let values: Vec<f64> = rows.iter().map(&value).collect();
            let middle = center.unwrap_or_else(|| mean(&values));
            let spread = sigma.unwrap_or_else(|| moving_sigma(&values));
            let (upper, lower) = (middle + width * spread, middle - width * spread);
            Build {
                rows: rows
                    .iter()
                    .zip(&values)
                    .map(|(item, value)| {
                        let out = if *value > upper || *value < lower {
                            *value
                        } else {
                            f64::NAN
                        };
                        row([
                            x(item),
                            *value,
                            middle,
                            upper,
                            lower,
                            out,
                            middle + spread,
                            middle - spread,
                            middle + 2. * spread,
                            middle - 2. * spread,
                        ])
                    })
                    .collect(),
                ..Build::default()
            }
        });
    let colors = ChartTheme::colors();
    let flat = |dash: LineStyle, hide: bool| Style {
        transform: Some(ValueTransform::Constant),
        line_style: Some(dash),
        hide_from: hide.then_some(QUIET),
        ..lines(1.5, false)
    };
    let mut fields = vec![
        field(
            "Value",
            1,
            Some(color.unwrap_or_else(|| colors.series(0))),
            lines(1.5, false),
        ),
        field(
            "Centre",
            2,
            Some(colors.neutral),
            flat(LineStyle::Solid, false),
        ),
        field(
            "UCL",
            3,
            Some(colors.bad),
            flat(LineStyle::Dash(&[6., 4.]), false),
        ),
        field(
            "LCL",
            4,
            Some(colors.bad),
            flat(LineStyle::Dash(&[6., 4.]), false),
        ),
        field("Out of control", 5, Some(colors.bad), dots(9.)),
    ];
    if zones.unwrap_or(false) {
        for (index, name) in [(6, "+1σ"), (7, "−1σ"), (8, "+2σ"), (9, "−2σ")] {
            fields.push(field(
                name,
                index,
                Some(colors.neutral.opacity(0.6)),
                flat(LineStyle::Dot(3.), true),
            ));
        }
    }
    trend_view(
        table,
        TrendSpec {
            fields,
            style: Style::default(),
            defaults: Defaults::new(
                x_axis.unwrap_or_else(|| AxisConfig::new().time()),
                y_axis.unwrap_or_default(),
            ),
            legend,
            tooltip: tooltip.or_else(|| Some(Tooltip::multi())),
            class,
        })
}

fn segment(rows: &mut Vec<Row>, from: f64, to: f64, levels: [f64; 3]) {
    for at in [from, to] {
        rows.push(row([
            f64::NAN,
            f64::NAN,
            at,
            levels[0],
            levels[1],
            levels[2],
        ]));
    }
}

#[component]
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub fn bland_altman<
    T: 'static,
    D: Dataset<T>,
    A: Fn(&T) -> f64 + 'static,
    B: Fn(&T) -> f64 + 'static,
>(
    data: D,
    a: A,
    b: B,
    agreement: Option<f64>,
    a_label: Option<SharedString>,
    b_label: Option<SharedString>,
    #[into] color: Option<Hsla>,
    legend: Option<Legend>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
) -> AnyElement {
    let table = use_signal(Table::default());
    let z = agreement.unwrap_or(1.96);
    sync(
        table,
        &data,
        z.to_bits(),
        |rows| {
            let pairs: Vec<(f64, f64)> = rows
                .iter()
                .map(|item| {
                    let (first, second) = (a(item), b(item));
                    (f64::midpoint(first, second), first - second)
                })
                .collect();
            let differences: Vec<f64> = pairs.iter().map(|(_, difference)| *difference).collect();
            let (bias, spread) = (mean(&differences), deviation(&differences));
            let finite = || {
                pairs
                    .iter()
                    .map(|(average, _)| *average)
                    .filter(|value| value.is_finite())
            };
            let (low, high) = (
                finite().fold(f64::INFINITY, f64::min),
                finite().fold(f64::NEG_INFINITY, f64::max),
            );
            let mut out: Vec<Row> = pairs
                .iter()
                .map(|(average, difference)| {
                    row([
                        *average,
                        *difference,
                        f64::NAN,
                        f64::NAN,
                        f64::NAN,
                        f64::NAN,
                    ])
                })
                .collect();
            segment(
                &mut out,
                low,
                high,
                [bias, bias + z * spread, bias - z * spread],
            );
            Build {
                rows: out,
                ..Build::default()
            }
        });
    let colors = ChartTheme::colors();
    let first = a_label.unwrap_or(SharedString::Borrowed("A"));
    let second = b_label.unwrap_or(SharedString::Borrowed("B"));
    let spread = crate::format::trimmed(z);
    let list = vec![
        series(
            "Differences",
            0,
            1,
            Some(color.unwrap_or_else(|| colors.series(0))),
            XyStyle {
                point_size: Some(6.),
                ..XyStyle::default()
            },
        ),
        series(
            "Bias",
            2,
            3,
            Some(colors.neutral),
            xy_line(1.5, LineStyle::Solid),
        ),
        series(
            format!("+{spread} SD"),
            2,
            4,
            Some(colors.bad),
            xy_line(1.5, LineStyle::Dash(&[6., 4.])),
        ),
        series(
            format!("−{spread} SD"),
            2,
            5,
            Some(colors.bad),
            xy_line(1.5, LineStyle::Dash(&[6., 4.])),
        ),
    ];
    xy_view(
        table,
        XySpec {
            series: list,
            style: XyStyle::default(),
            x_axis: AxisConfig::new().label(format!("Mean of {first} and {second}")),
            y_axis: AxisConfig::new().label(format!("{first} − {second}")),
            legend,
            tooltip,
            class,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sigma_uses_the_average_moving_range() {
        let spread = moving_sigma(&[10., 12., 11., 13.]);
        assert!((spread - (5. / 3.) / D2).abs() < 1e-9);
    }
}
