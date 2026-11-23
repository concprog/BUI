use std::collections::HashMap;

use blenderui::{AnyElement, Hsla, SharedString};
use crate::component;
use blenderui::hooks::use_signal;

use super::{
    HIDDEN, QUIET, Sketch, TrendSpec, XySpec, bars, categories, dots, field, lines, magnitude,
    padded, percent, series, trend_view, xy_line, xy_view,
};
use crate::derive::{Build, Table, column, row, sync};
use crate::plot::{AxisConfig, AxisPlacement, LineStyle, SortOrder, Tooltip};
use crate::trend::{Defaults, PointsMode, Style, ValueTransform};
use crate::xy_chart::{PointShape, Style as XyStyle};
use crate::{ChartTheme, Dataset, Legend};

fn ordered(values: &[f64], sort: SortOrder) -> Vec<usize> {
    let mut order: Vec<usize> = (0..values.len()).collect();
    match sort {
        SortOrder::Ascending => order.sort_by(|a, b| values[*a].total_cmp(&values[*b])),
        SortOrder::Descending => order.sort_by(|a, b| values[*b].total_cmp(&values[*a])),
        SortOrder::None => {}
    }
    order
}

const fn axis(horizontal: bool, defaults: &mut Defaults) {
    defaults.vertical = horizontal;
}

#[component]
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub fn pareto<
    T: 'static,
    D: Dataset<T>,
    N: Fn(&T) -> &str + 'static,
    V: Fn(&T) -> f64 + 'static,
>(
    data: D,
    name: N,
    value: V,
    threshold: Option<f64>,
    #[into] color: Option<Hsla>,
    #[into] line_color: Option<Hsla>,
    show_values: Option<bool>,
    y_axis: Option<AxisConfig>,
    legend: Option<Legend>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
) -> AnyElement {
    let table = use_signal(Table::default());
    let threshold = threshold.unwrap_or(80.);
    sync(
        table,
        &data,
        threshold.to_bits(),
        |rows| {
            let values: Vec<f64> = rows.iter().map(&value).collect();
            let order = ordered(&values, SortOrder::Descending);
            let total: f64 = values.iter().filter(|value| value.is_finite()).sum();
            let mut running = 0.;
            Build {
                rows: order
                    .iter()
                    .enumerate()
                    .map(|(slot, index)| {
                        running += values[*index].max(0.);
                        let share = if total > 0. {
                            running / total * 100.
                        } else {
                            0.
                        };
                        row([slot as f64, values[*index], share, threshold])
                    })
                    .collect(),
                names: order
                    .iter()
                    .map(|index| SharedString::from(name(&rows[*index]).to_owned()))
                    .collect(),
                fields: Vec::new(),
            }
        });
    let names = table.with(|table| table.names.clone());
    let colors = ChartTheme::colors();
    let share = AxisConfig::new()
        .label("%")
        .placement(AxisPlacement::Right)
        .min(0.)
        .max(100.)
        .format(percent)
        .grid(false);
    let line = line_color.unwrap_or_else(|| colors.series(1));
    let fields = vec![
        field(
            "Value",
            1,
            color,
            Style {
                show_values: Some(show_values.unwrap_or(false)),
                ..bars(0.7)
            },
        ),
        field(
            "Cumulative %",
            2,
            Some(line),
            Style {
                show_points: Some(PointsMode::Always),
                point_size: Some(6.),
                ..lines(2., false)
            },
        )
        .with_axis(share.clone()),
        field(
            format!("{} %", crate::format::trimmed(threshold)),
            3,
            Some(colors.bad),
            Style {
                transform: Some(ValueTransform::Constant),
                line_style: Some(LineStyle::Dash(&[6., 4.])),
                hide_from: Some(QUIET),
                ..lines(1., false)
            },
        )
        .with_axis(share),
    ];
    trend_view(
        table,
        TrendSpec {
            fields,
            style: Style::default(),
            defaults: Defaults::new(categories(&names), y_axis.unwrap_or_default()),
            legend,
            tooltip: tooltip.or_else(|| Some(Tooltip::multi())),
            class,
        })
}

#[component]
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub fn lollipop<
    T: 'static,
    D: Dataset<T>,
    N: Fn(&T) -> &str + 'static,
    V: Fn(&T) -> f64 + 'static,
>(
    data: D,
    name: N,
    value: V,
    horizontal: Option<bool>,
    sort: Option<SortOrder>,
    #[into] color: Option<Hsla>,
    point_size: Option<f32>,
    show_values: Option<bool>,
    value_axis: Option<AxisConfig>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
) -> AnyElement {
    let table = use_signal(Table::default());
    let sort = sort.unwrap_or_default();
    sync(
        table,
        &data,
        sort,
        |rows| {
            let values: Vec<f64> = rows.iter().map(&value).collect();
            let order = ordered(&values, sort);
            Build {
                rows: order
                    .iter()
                    .enumerate()
                    .map(|(slot, index)| row([slot as f64, values[*index]]))
                    .collect(),
                names: order
                    .iter()
                    .map(|index| SharedString::from(name(&rows[*index]).to_owned()))
                    .collect(),
                fields: Vec::new(),
            }
        });
    let names = table.with(|table| table.names.clone());
    let color = color.unwrap_or_else(|| ChartTheme::colors().series(0));
    let fields = vec![
        field(
            "Stick",
            1,
            Some(color),
            Style {
                bar_max_width: Some(2.),
                hide_from: Some(HIDDEN),
                ..bars(0.05)
            },
        ),
        field(
            "Value",
            1,
            Some(color),
            Style {
                show_values: Some(show_values.unwrap_or(false)),
                ..dots(point_size.unwrap_or(12.))
            },
        ),
    ];
    let mut defaults = Defaults::new(categories(&names), value_axis.unwrap_or_default());
    axis(horizontal.unwrap_or(true), &mut defaults);
    trend_view(
        table,
        TrendSpec {
            fields,
            style: Style::default(),
            defaults,
            legend: Some(Legend::hidden()),
            tooltip,
            class,
        })
}

#[component]
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub fn dumbbell<
    T: 'static,
    D: Dataset<T>,
    N: Fn(&T) -> &str + 'static,
    A: Fn(&T) -> f64 + 'static,
    B: Fn(&T) -> f64 + 'static,
>(
    data: D,
    name: N,
    start: A,
    end: B,
    start_label: Option<SharedString>,
    end_label: Option<SharedString>,
    #[into] start_color: Option<Hsla>,
    #[into] end_color: Option<Hsla>,
    horizontal: Option<bool>,
    sort: Option<SortOrder>,
    point_size: Option<f32>,
    value_axis: Option<AxisConfig>,
    legend: Option<Legend>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
) -> AnyElement {
    let table = use_signal(Table::default());
    let sort = sort.unwrap_or_default();
    sync(
        table,
        &data,
        sort,
        |rows| {
            let ends: Vec<f64> = rows.iter().map(&end).collect();
            let order = ordered(&ends, sort);
            Build {
                rows: order
                    .iter()
                    .enumerate()
                    .map(|(slot, index)| {
                        let (a, b) = (start(&rows[*index]), ends[*index]);
                        row([slot as f64, a, b, a.min(b), (a - b).abs()])
                    })
                    .collect(),
                names: order
                    .iter()
                    .map(|index| SharedString::from(name(&rows[*index]).to_owned()))
                    .collect(),
                fields: Vec::new(),
            }
        });
    let names = table.with(|table| table.names.clone());
    let colors = ChartTheme::colors();
    let size = point_size.unwrap_or(12.);
    let fields = vec![
        field(
            "Span",
            4,
            Some(colors.neutral),
            Style {
                bar_max_width: Some(3.),
                hide_from: Some(HIDDEN),
                ..bars(0.05)
            },
        )
        .with_floor(column(3)),
        field(
            start_label.unwrap_or(SharedString::Borrowed("Start")),
            1,
            Some(start_color.unwrap_or_else(|| colors.series(0))),
            dots(size),
        ),
        field(
            end_label.unwrap_or(SharedString::Borrowed("End")),
            2,
            Some(end_color.unwrap_or_else(|| colors.series(1))),
            dots(size),
        ),
    ];
    let mut defaults = Defaults::new(categories(&names), value_axis.unwrap_or_default());
    axis(horizontal.unwrap_or(true), &mut defaults);
    trend_view(
        table,
        TrendSpec {
            fields,
            style: Style::default(),
            defaults,
            legend,
            tooltip: tooltip.or_else(|| Some(Tooltip::multi())),
            class,
        })
}

#[component]
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub fn slope<
    T: 'static,
    D: Dataset<T>,
    N: Fn(&T) -> &str + 'static,
    A: Fn(&T) -> f64 + 'static,
    B: Fn(&T) -> f64 + 'static,
>(
    data: D,
    name: N,
    before: A,
    after: B,
    before_label: Option<SharedString>,
    after_label: Option<SharedString>,
    color_by_change: Option<bool>,
    show_values: Option<bool>,
    y_axis: Option<AxisConfig>,
    legend: Option<Legend>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
) -> AnyElement {
    let table = use_signal(Table::default());
    sync(
        table,
        &data,
        (),
        |rows| {
            let mut first = vec![0.];
            let mut second = vec![1.];
            first.extend(rows.iter().map(&before));
            second.extend(rows.iter().map(&after));
            Build {
                rows: vec![first.into_boxed_slice(), second.into_boxed_slice()],
                names: Vec::new(),
                fields: rows
                    .iter()
                    .map(|item| SharedString::from(name(item).to_owned()))
                    .collect(),
            }
        });
    let (fields_names, deltas): (Vec<SharedString>, Vec<f64>) = table.with(|table| {
        let deltas = (1..=table.fields.len())
            .map(|index| {
                let value = |row: usize| {
                    table
                        .rows
                        .get(row)
                        .and_then(|row| row.get(index))
                        .copied()
                        .unwrap_or(0.)
                };
                value(1) - value(0)
            })
            .collect();
        (table.fields.clone(), deltas)
    });
    let colors = ChartTheme::colors();
    let by_change = color_by_change.unwrap_or(true);
    let fields = fields_names
        .into_iter()
        .zip(deltas)
        .enumerate()
        .map(|(index, (name, delta))| {
            let color = if by_change {
                if delta >= 0. { colors.good } else { colors.bad }
            } else {
                colors.series(index)
            };
            field(
                name,
                index + 1,
                Some(color),
                Style {
                    show_points: Some(PointsMode::Always),
                    point_size: Some(8.),
                    show_values: Some(show_values.unwrap_or(true)),
                    ..lines(2., false)
                },
            )
        })
        .collect();
    let labels = [
        before_label.unwrap_or(SharedString::Borrowed("Before")),
        after_label.unwrap_or(SharedString::Borrowed("After")),
    ];
    let x_axis = AxisConfig::new().categories(labels).min(-0.25).max(1.25);
    trend_view(
        table,
        TrendSpec {
            fields,
            style: Style::default(),
            defaults: Defaults::new(x_axis, y_axis.unwrap_or_default()),
            legend: legend.or_else(|| Some(Legend::list().right())),
            tooltip,
            class,
        })
}

#[component]
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub fn bump<
    T: 'static,
    D: Dataset<T>,
    X: Fn(&T) -> f64 + 'static,
    N: Fn(&T) -> &str + 'static,
    V: Fn(&T) -> f64 + 'static,
>(
    data: D,
    x: X,
    name: N,
    value: V,
    highest_first: Option<bool>,
    smooth: Option<bool>,
    x_axis: Option<AxisConfig>,
    legend: Option<Legend>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
) -> AnyElement {
    let table = use_signal(Table::default());
    let highest = highest_first.unwrap_or(true);
    sync(
        table,
        &data,
        highest,
        |rows| {
            let mut items: Vec<SharedString> = Vec::new();
            let mut index: HashMap<String, usize> = HashMap::new();
            let mut periods: Vec<f64> = Vec::new();
            for item in rows {
                let label = name(item);
                if !index.contains_key(label) {
                    index.insert(label.to_owned(), items.len());
                    items.push(SharedString::from(label.to_owned()));
                }
                let at = x(item);
                if !periods
                    .iter()
                    .any(|period| (period - at).abs() < f64::EPSILON)
                {
                    periods.push(at);
                }
            }
            periods.sort_by(f64::total_cmp);
            let mut grid = vec![vec![f64::NAN; items.len()]; periods.len()];
            for item in rows {
                let (Some(column), Some(period)) = (
                    index.get(name(item)),
                    periods
                        .iter()
                        .position(|period| (period - x(item)).abs() < f64::EPSILON),
                ) else {
                    continue;
                };
                grid[period][*column] = value(item);
            }
            let rows = periods
                .iter()
                .zip(&grid)
                .map(|(period, values)| {
                    let mut order: Vec<usize> = (0..values.len())
                        .filter(|item| values[*item].is_finite())
                        .collect();
                    order.sort_by(|a, b| {
                        let ordering = values[*a].total_cmp(&values[*b]);
                        if highest {
                            ordering.reverse()
                        } else {
                            ordering
                        }
                    });
                    let mut ranks = vec![f64::NAN; values.len() + 1];
                    ranks[0] = *period;
                    for (rank, item) in order.iter().enumerate() {
                        ranks[item + 1] = (rank + 1) as f64;
                    }
                    ranks.into_boxed_slice()
                })
                .collect();
            Build {
                rows,
                names: Vec::new(),
                fields: items,
            }
        });
    let names = table.with(|table| table.fields.clone());
    let count = names.len();
    let fields = names
        .into_iter()
        .enumerate()
        .map(|(index, name)| {
            field(
                name,
                index + 1,
                None,
                Style {
                    show_points: Some(PointsMode::Always),
                    point_size: Some(12.),
                    ..lines(3., smooth.unwrap_or(true))
                },
            )
        })
        .collect();
    let y_axis = AxisConfig::new()
        .reverse(true)
        .min(0.5)
        .max(count as f64 + 0.5)
        .decimals(0)
        .label("Rank");
    trend_view(
        table,
        TrendSpec {
            fields,
            style: Style::default(),
            defaults: Defaults::new(
                x_axis.unwrap_or_else(|| AxisConfig::new().decimals(0)),
                y_axis,
            ),
            legend: legend.or_else(|| Some(Legend::list().right())),
            tooltip: tooltip.or_else(|| Some(Tooltip::multi().sort(SortOrder::Descending))),
            class,
        })
}

#[component]
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub fn back_to_back<
    T: 'static,
    D: Dataset<T>,
    N: Fn(&T) -> &str + 'static,
    L: Fn(&T) -> f64 + 'static,
    R: Fn(&T) -> f64 + 'static,
>(
    data: D,
    name: N,
    left: L,
    right: R,
    left_label: Option<SharedString>,
    right_label: Option<SharedString>,
    #[into] left_color: Option<Hsla>,
    #[into] right_color: Option<Hsla>,
    legend: Option<Legend>,
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
                .enumerate()
                .map(|(slot, item)| row([slot as f64, left(item), right(item)]))
                .collect(),
            names: rows
                .iter()
                .map(|item| SharedString::from(name(item).to_owned()))
                .collect(),
            fields: Vec::new(),
        });
    let names = table.with(|table| table.names.clone());
    let colors = ChartTheme::colors();
    let fields = vec![
        field(
            left_label.unwrap_or(SharedString::Borrowed("Left")),
            1,
            Some(left_color.unwrap_or_else(|| colors.series(0))),
            Style {
                transform: Some(ValueTransform::NegativeY),
                ..bars(0.8)
            },
        ),
        field(
            right_label.unwrap_or(SharedString::Borrowed("Right")),
            2,
            Some(right_color.unwrap_or_else(|| colors.series(1))),
            bars(0.8),
        ),
    ];
    let mut defaults = Defaults::new(
        categories(&names),
        AxisConfig::new().format(magnitude).centered_zero(true),
    );
    defaults.vertical = true;
    trend_view(
        table,
        TrendSpec {
            fields,
            style: Style::default(),
            defaults,
            legend,
            tooltip: tooltip.or_else(|| Some(Tooltip::multi())),
            class,
        })
}

#[component]
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub fn waterfall<
    T: 'static,
    D: Dataset<T>,
    N: Fn(&T) -> &str + 'static,
    V: Fn(&T) -> f64 + 'static,
>(
    data: D,
    name: N,
    value: V,
    total: Option<bool>,
    total_label: Option<SharedString>,
    #[into] increase: Option<Hsla>,
    #[into] decrease: Option<Hsla>,
    #[into] total_color: Option<Hsla>,
    horizontal: Option<bool>,
    show_values: Option<bool>,
    value_axis: Option<AxisConfig>,
    legend: Option<Legend>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
) -> AnyElement {
    let table = use_signal(Table::default());
    let with_total = total.unwrap_or(true);
    let total_label = total_label.unwrap_or(SharedString::Borrowed("Total"));
    sync(
        table,
        &data,
        (with_total, total_label.clone()),
        |rows| {
            let mut running = 0.;
            let mut out: Vec<_> = rows
                .iter()
                .enumerate()
                .map(|(slot, item)| {
                    let delta = value(item);
                    let before = running;
                    running += if delta.is_finite() { delta } else { 0. };
                    let (up, down) = if delta >= 0. {
                        (delta, f64::NAN)
                    } else {
                        (f64::NAN, delta.abs())
                    };
                    row([
                        slot as f64,
                        up,
                        down,
                        f64::NAN,
                        before.min(running),
                        f64::NAN,
                    ])
                })
                .collect();
            let mut names: Vec<SharedString> = rows
                .iter()
                .map(|item| SharedString::from(name(item).to_owned()))
                .collect();
            if with_total {
                out.push(row([
                    rows.len() as f64,
                    f64::NAN,
                    f64::NAN,
                    running.abs(),
                    running.min(0.),
                    0.,
                ]));
                names.push(total_label.clone());
            }
            Build {
                rows: out,
                names,
                fields: Vec::new(),
            }
        });
    let names = table.with(|table| table.names.clone());
    let colors = ChartTheme::colors();
    let labelled = show_values.unwrap_or(true);
    let bar = |hide: bool| Style {
        show_values: Some(labelled),
        hide_from: hide.then_some(QUIET),
        ..bars(0.7)
    };
    let fields = vec![
        field(
            "Increase",
            1,
            Some(increase.unwrap_or(colors.good)),
            bar(false),
        )
        .with_floor(column(4)),
        field(
            "Decrease",
            2,
            Some(decrease.unwrap_or(colors.bad)),
            bar(false),
        )
        .with_floor(column(4)),
        field(
            total_label,
            3,
            Some(total_color.unwrap_or_else(|| colors.series(0))),
            bar(false),
        )
        .with_floor(column(4)),
    ];
    let mut defaults = Defaults::new(categories(&names), value_axis.unwrap_or_default());
    axis(horizontal.unwrap_or(false), &mut defaults);
    trend_view(
        table,
        TrendSpec {
            fields,
            style: Style::default(),
            defaults,
            legend,
            tooltip,
            class,
        })
}

#[component]
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub fn bullet<
    T: 'static,
    D: Dataset<T>,
    N: Fn(&T) -> &str + 'static,
    V: Fn(&T) -> f64 + 'static,
    G: Fn(&T) -> f64 + 'static,
    R: Fn(&T) -> [f64; 3] + 'static,
>(
    data: D,
    name: N,
    value: V,
    target: Option<G>,
    ranges: Option<R>,
    #[into] color: Option<Hsla>,
    #[into] target_color: Option<Hsla>,
    horizontal: Option<bool>,
    value_axis: Option<AxisConfig>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
) -> AnyElement {
    let table = use_signal(Table::default());
    sync(
        table,
        &data,
        (target.is_some(), ranges.is_some()),
        |rows| {
            let largest = rows
                .iter()
                .flat_map(|item| {
                    let bands = ranges.as_ref().map_or([f64::NAN; 3], |ranges| ranges(item));
                    [
                        value(item),
                        target.as_ref().map_or(f64::NAN, |target| target(item)),
                        bands[2],
                    ]
                })
                .filter(|value| value.is_finite())
                .fold(0., f64::max);
            let tick = largest * 0.006;
            Build {
                rows: rows
                    .iter()
                    .enumerate()
                    .map(|(slot, item)| {
                        let bands = ranges.as_ref().map_or([f64::NAN; 3], |ranges| ranges(item));
                        let goal = target.as_ref().map_or(f64::NAN, |target| target(item));
                        row([
                            slot as f64,
                            bands[2],
                            bands[1],
                            bands[0],
                            value(item),
                            tick * 2.,
                            goal - tick,
                        ])
                    })
                    .collect(),
                names: rows
                    .iter()
                    .map(|item| SharedString::from(name(item).to_owned()))
                    .collect(),
                fields: Vec::new(),
            }
        });
    let names = table.with(|table| table.names.clone());
    let colors = ChartTheme::colors();
    let band = colors.neutral;
    let shade = || Style {
        hide_from: Some(QUIET),
        ..bars(0.75)
    };
    let fields = vec![
        field("Satisfactory", 1, Some(band.opacity(0.25)), shade()),
        field("Fair", 2, Some(band.opacity(0.45)), shade()),
        field("Poor", 3, Some(band.opacity(0.7)), shade()),
        field(
            "Value",
            4,
            Some(color.unwrap_or_else(|| colors.series(0))),
            bars(0.3),
        ),
        field(
            "Target",
            5,
            Some(target_color.unwrap_or(colors.bad)),
            Style {
                hide_from: Some(HIDDEN),
                ..bars(0.6)
            },
        )
        .with_floor(column(6)),
    ];
    let mut defaults = Defaults::new(
        categories(&names),
        value_axis.unwrap_or_else(|| AxisConfig::new().min(0.)),
    );
    axis(horizontal.unwrap_or(true), &mut defaults);
    trend_view(
        table,
        TrendSpec {
            fields,
            style: Style::default(),
            defaults,
            legend: Some(Legend::hidden()),
            tooltip: tooltip.or_else(|| Some(Tooltip::multi())),
            class,
        })
}

#[component]
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub fn funnel<
    T: 'static,
    D: Dataset<T>,
    N: Fn(&T) -> &str + 'static,
    V: Fn(&T) -> f64 + 'static,
>(
    data: D,
    name: N,
    value: V,
    show_values: Option<bool>,
    horizontal: Option<bool>,
    #[into] color: Option<Hsla>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
) -> AnyElement {
    let table = use_signal(Table::default());
    sync(
        table,
        &data,
        (),
        |rows| {
            let values: Vec<f64> = rows.iter().map(&value).collect();
            let widest = values
                .iter()
                .copied()
                .filter(|value| value.is_finite())
                .fold(0., f64::max);
            let count = values.len();
            Build {
                rows: values
                    .iter()
                    .enumerate()
                    .map(|(slot, amount)| {
                        let mut cells = vec![f64::NAN; count + 2];
                        cells[0] = slot as f64;
                        cells[slot + 1] = *amount;
                        cells[count + 1] = (widest - amount) / 2.;
                        cells.into_boxed_slice()
                    })
                    .collect(),
                names: rows
                    .iter()
                    .map(|item| SharedString::from(name(item).to_owned()))
                    .collect(),
                fields: Vec::new(),
            }
        });
    let names = table.with(|table| table.names.clone());
    let colors = ChartTheme::colors();
    let count = names.len();
    let fields = names
        .iter()
        .enumerate()
        .map(|(index, stage)| {
            field(
                stage.clone(),
                index + 1,
                Some(color.unwrap_or_else(|| colors.series(index))),
                Style {
                    show_values: Some(show_values.unwrap_or(true)),
                    ..bars(0.85)
                },
            )
            .with_floor(column(count + 1))
        })
        .collect();
    let mut defaults = Defaults::new(
        categories(&names),
        AxisConfig::new()
            .placement(AxisPlacement::Hidden)
            .min(0.)
            .grid(false),
    );
    axis(horizontal.unwrap_or(true), &mut defaults);
    trend_view(
        table,
        TrendSpec {
            fields,
            style: Style::default(),
            defaults,
            legend: Some(Legend::hidden()),
            tooltip,
            class,
        })
}

#[component]
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub fn forest<
    T: 'static,
    D: Dataset<T>,
    N: Fn(&T) -> &str + 'static,
    E: Fn(&T) -> f64 + 'static,
    L: Fn(&T) -> f64 + 'static,
    H: Fn(&T) -> f64 + 'static,
    W: Fn(&T) -> f64 + 'static,
>(
    data: D,
    name: N,
    estimate: E,
    low: L,
    high: H,
    weight: Option<W>,
    null: Option<f64>,
    #[into] color: Option<Hsla>,
    value_axis: Option<AxisConfig>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
) -> AnyElement {
    let table = use_signal(Table::default());
    let line = null.unwrap_or(0.);
    sync(
        table,
        &data,
        (line.to_bits(), weight.is_some()),
        |rows| {
            let count = rows.len();
            let mut sketch = Sketch::new(7);
            for (index, item) in rows.iter().enumerate() {
                let y = (count - 1 - index) as f64;
                sketch.trace(0, [(low(item), y), (high(item), y)]);
            }
            sketch.trace(2, [(line, -0.5), (line, count as f64 - 0.5)]);
            sketch
                .rows
                .extend(rows.iter().enumerate().map(|(index, item)| {
                    let size = weight.as_ref().map_or(1., |weight| weight(item));
                    padded(
                        7,
                        &[
                            (4, estimate(item)),
                            (5, (count - 1 - index) as f64),
                            (6, size),
                        ],
                    )
                }));
            Build {
                rows: sketch.rows,
                names: rows
                    .iter()
                    .rev()
                    .map(|item| SharedString::from(name(item).to_owned()))
                    .collect(),
                fields: [
                    "Interval", "Study", "Null", "Study", "Estimate", "Study", "Weight",
                ]
                .map(SharedString::Borrowed)
                .to_vec(),
            }
        });
    let colors = ChartTheme::colors();
    let tint = color.unwrap_or_else(|| colors.series(0));
    let names = table.with(|table| table.names.clone());
    let mut marks = series(
        "Estimate",
        4,
        5,
        Some(tint),
        XyStyle {
            point_shape: Some(PointShape::Square),
            point_size: Some(9.),
            size_range: Some((6., 20.)),
            fill_opacity: Some(1.),
            ..XyStyle::default()
        },
    );
    if weight.is_some() {
        marks = marks.sized(column(6));
    }
    xy_view(
        table,
        XySpec {
            series: vec![
                series("Interval", 0, 1, Some(tint), xy_line(1.5, LineStyle::Solid)),
                series(
                    "Null",
                    2,
                    3,
                    Some(colors.neutral),
                    xy_line(1., LineStyle::Dash(&[4., 4.])),
                ),
                marks,
            ],
            style: XyStyle::default(),
            x_axis: value_axis.unwrap_or_default(),
            y_axis: categories(&names),
            legend: Some(Legend::hidden()),
            tooltip,
            class,
        })
}
