use std::collections::HashMap;

use blenderui::element::{AnyElement, Div, FluentBuilder, IntoElement, ParentElement};
use blenderui::hooks::use_signal;
use blenderui::html::div;
use blenderui::{Hsla, SharedString, Styled, px};

use super::{TrendSpec, XySpec, bars, deviation, field, series, trend_view, xy_view};
use crate::derive::{Build, Row, Table, column, sync};
use crate::plot::{AxisConfig, AxisPlacement, Tooltip, gutter};
use crate::trend::{Defaults, PointsMode, Style, TrendField};
use crate::widgets::scroll_area;
use crate::xy_chart::Style as XyStyle;
use crate::{ChartTheme, Dataset, Legend, component};

const DENSITY_POINTS: usize = 128;
const ROW_MIN: f32 = 56.;

fn groups<T>(rows: &[T], group: Option<&dyn Fn(&T) -> &str>) -> (Vec<usize>, Vec<SharedString>) {
    let mut names: Vec<SharedString> = Vec::new();
    let mut index: HashMap<String, usize> = HashMap::new();
    let assigned = rows
        .iter()
        .map(|item| {
            let label = group.map_or("", |group| group(item));
            *index.entry(label.to_owned()).or_insert_with(|| {
                names.push(SharedString::from(label.to_owned()));
                names.len() - 1
            })
        })
        .collect();
    (assigned, names)
}

fn category_axis(names: &[SharedString]) -> AxisConfig {
    let axis = AxisConfig::new()
        .min(-0.5)
        .max(names.len() as f64 - 0.5)
        .grid(false);
    if names.len() == 1 && names[0].is_empty() {
        axis.placement(AxisPlacement::Hidden)
    } else {
        axis.categories(names.iter().cloned())
    }
}

#[component]
pub fn barcode<
    T: 'static,
    D: Dataset<T>,
    V: Fn(&T) -> f64 + 'static,
    G: Fn(&T) -> &str + 'static,
>(
    data: D,
    value: V,
    group: Option<G>,
    tick_width: Option<f32>,
    #[into] color: Option<Hsla>,
    x_axis: Option<AxisConfig>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
) -> AnyElement {
    let table = use_signal(Table::default());
    sync(
        table,
        &data,
        group.is_some(),
        |rows| {
            let (assigned, names) = groups(
                rows,
                group.as_ref().map(|group| group as &dyn Fn(&T) -> &str),
            );
            let count = names.len();
            let mut cells: Vec<(f64, usize)> = rows
                .iter()
                .map(&value)
                .zip(assigned)
                .filter(|(value, _)| value.is_finite())
                .collect();
            cells.sort_by(|a, b| a.0.total_cmp(&b.0));
            Build {
                rows: cells
                    .into_iter()
                    .map(|(value, slot)| {
                        let mut out = vec![f64::NAN; 1 + count * 2];
                        out[0] = value;
                        out[1 + slot] = 0.7;
                        for index in 0..count {
                            out[1 + count + index] = index as f64 - 0.35;
                        }
                        out.into_boxed_slice()
                    })
                    .collect(),
                fields: names,
                ..Build::default()
            }
        });
    let names = table.with(|table| table.fields.clone());
    let count = names.len();
    let colors = ChartTheme::colors();
    let fields = names
        .iter()
        .enumerate()
        .map(|(index, name)| {
            field(
                name.clone(),
                index + 1,
                Some(color.unwrap_or_else(|| colors.series(index))),
                Style {
                    bar_max_width: Some(tick_width.unwrap_or(1.5)),
                    fill_opacity: Some(0.8),
                    ..bars(100.)
                },
            )
            .with_floor(column(1 + count + index))
        })
        .collect();
    trend_view(
        table,
        TrendSpec {
            fields,
            style: Style::default(),
            defaults: Defaults::new(x_axis.unwrap_or_default(), category_axis(&names)),
            legend: Some(Legend::hidden()),
            tooltip: tooltip.or_else(|| Some(Tooltip::hidden())),
            class,
        })
}

pub(crate) fn swarm(values: &[f64], spread: f64, step: f64, limit: f64) -> Vec<f64> {
    let mut order: Vec<usize> = (0..values.len()).collect();
    order.sort_by(|a, b| values[*a].total_cmp(&values[*b]));
    let mut placed: Vec<(f64, f64)> = Vec::new();
    let mut offsets = vec![0.; values.len()];
    for index in order {
        let value = values[index];
        let mut chosen = 0.;
        for attempt in 0..200 {
            let level = f64::from((attempt + 1) / 2) * step;
            let offset = if attempt % 2 == 0 { level } else { -level };
            if offset.abs() > limit {
                chosen = 0.;
                break;
            }
            let clear = placed
                .iter()
                .rev()
                .take_while(|(other, _)| value - other < spread)
                .all(|(other, at)| {
                    let (dx, dy) = ((value - other) / spread, (offset - at) / step);
                    dx.hypot(dy) >= 1.
                });
            if clear {
                chosen = offset;
                break;
            }
        }
        placed.push((value, chosen));
        offsets[index] = chosen;
    }
    offsets
}

#[component]
pub fn beeswarm<
    T: 'static,
    D: Dataset<T>,
    V: Fn(&T) -> f64 + 'static,
    G: Fn(&T) -> &str + 'static,
>(
    data: D,
    value: V,
    group: Option<G>,
    point_size: Option<f32>,
    x_axis: Option<AxisConfig>,
    legend: Option<Legend>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
) -> AnyElement {
    let table = use_signal(Table::default());
    sync(
        table,
        &data,
        group.is_some(),
        |rows| {
            let (assigned, names) = groups(
                rows,
                group.as_ref().map(|group| group as &dyn Fn(&T) -> &str),
            );
            let values: Vec<f64> = rows.iter().map(&value).collect();
            let finite = || values.iter().copied().filter(|value| value.is_finite());
            let span =
                finite().fold(f64::NEG_INFINITY, f64::max) - finite().fold(f64::INFINITY, f64::min);
            let spread = if span > 0. { span / 90. } else { 1. };
            let count = names.len();
            let mut out: Vec<Row> = Vec::with_capacity(values.len());
            for slot in 0..count {
                let members: Vec<usize> = (0..values.len())
                    .filter(|index| assigned[*index] == slot && values[*index].is_finite())
                    .collect();
                let picked: Vec<f64> = members.iter().map(|index| values[*index]).collect();
                let offsets = swarm(&picked, spread, 0.045, 0.45);
                for (value, offset) in picked.iter().zip(offsets) {
                    let mut cells = vec![f64::NAN; 1 + count];
                    cells[0] = *value;
                    cells[1 + slot] = slot as f64 + offset;
                    out.push(cells.into_boxed_slice());
                }
            }
            Build {
                rows: out,
                fields: names,
                ..Build::default()
            }
        });
    let names = table.with(|table| table.fields.clone());
    let list = names
        .iter()
        .enumerate()
        .map(|(index, name)| {
            series(
                if name.is_empty() {
                    SharedString::Borrowed("Values")
                } else {
                    name.clone()
                },
                0,
                index + 1,
                None,
                XyStyle {
                    point_size: Some(point_size.unwrap_or(5.)),
                    ..XyStyle::default()
                },
            )
        })
        .collect();
    xy_view(
        table,
        XySpec {
            series: list,
            style: XyStyle::default(),
            x_axis: x_axis.unwrap_or_default(),
            y_axis: category_axis(&names),
            legend: legend.or_else(|| Some(Legend::hidden())),
            tooltip,
            class,
        })
}

pub(crate) fn density(values: &[f64], grid: &[f64], bandwidth: f64) -> Vec<f64> {
    let scale = 1. / (values.len().max(1) as f64 * bandwidth * (2. * std::f64::consts::PI).sqrt());
    grid.iter()
        .map(|at| {
            values
                .iter()
                .map(|value| (-0.5 * ((at - value) / bandwidth).powi(2)).exp())
                .sum::<f64>()
                * scale
        })
        .collect()
}

fn silverman(values: &[f64]) -> f64 {
    let spread = deviation(values);
    let bandwidth = 1.06 * spread * (values.len().max(1) as f64).powf(-0.2);
    if bandwidth > 0. { bandwidth } else { 1. }
}

#[component]
pub fn ridgeline<
    T: 'static,
    D: Dataset<T>,
    V: Fn(&T) -> f64 + 'static,
    G: Fn(&T) -> &str + 'static,
>(
    data: D,
    value: V,
    group: G,
    bandwidth: Option<f64>,
    overlap: Option<f64>,
    fill_opacity: Option<f32>,
    x_axis: Option<AxisConfig>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
) -> AnyElement {
    let table = use_signal(Table::default());
    let height = overlap.unwrap_or(1.6);
    sync(
        table,
        &data,
        (bandwidth.map(f64::to_bits), height.to_bits()),
        |rows| {
            let (assigned, names) = groups(rows, Some(&group as &dyn Fn(&T) -> &str));
            let values: Vec<f64> = rows.iter().map(&value).collect();
            let count = names.len();
            let sets: Vec<Vec<f64>> = (0..count)
                .map(|slot| {
                    (0..values.len())
                        .filter(|index| assigned[*index] == slot && values[*index].is_finite())
                        .map(|index| values[index])
                        .collect()
                })
                .collect();
            let widths: Vec<f64> = sets
                .iter()
                .map(|set| bandwidth.unwrap_or_else(|| silverman(set)))
                .collect();
            let widest = widths.iter().copied().fold(0., f64::max);
            let finite = || values.iter().copied().filter(|value| value.is_finite());
            let (low, high) = (
                finite().fold(f64::INFINITY, f64::min) - widest * 3.,
                finite().fold(f64::NEG_INFINITY, f64::max) + widest * 3.,
            );
            let grid: Vec<f64> = (0..DENSITY_POINTS)
                .map(|step| low + (high - low) * step as f64 / (DENSITY_POINTS - 1) as f64)
                .collect();
            let curves: Vec<Vec<f64>> = sets
                .iter()
                .zip(&widths)
                .map(|(set, width)| density(set, &grid, *width))
                .collect();
            let peak = curves.iter().flatten().copied().fold(0., f64::max);
            Build {
                rows: grid
                    .iter()
                    .enumerate()
                    .map(|(step, at)| {
                        let mut cells = vec![*at];
                        cells.extend(curves.iter().map(|curve| {
                            if peak > 0. {
                                curve[step] / peak * height
                            } else {
                                0.
                            }
                        }));
                        cells.extend((0..count).map(|slot| (count - 1 - slot) as f64));
                        cells.into_boxed_slice()
                    })
                    .collect(),
                fields: names,
                ..Build::default()
            }
        });
    let names = table.with(|table| table.fields.clone());
    let count = names.len();
    let colors = ChartTheme::colors();
    let fields: Vec<TrendField<Row>> = names
        .iter()
        .enumerate()
        .map(|(slot, name)| {
            field(
                name.clone(),
                slot + 1,
                Some(colors.series(slot)),
                Style {
                    fill_opacity: Some(fill_opacity.unwrap_or(0.75)),
                    line_width: Some(1.5),
                    show_points: Some(PointsMode::Never),
                    ..Style::default()
                },
            )
            .with_floor(column(1 + count + slot))
        })
        .collect();
    let positions: Vec<SharedString> = names.iter().rev().cloned().collect();
    let y_axis = AxisConfig::new()
        .categories(positions)
        .min(-0.2)
        .max(count as f64 - 1. + height)
        .grid(false);
    trend_view(
        table,
        TrendSpec {
            fields,
            style: Style::default(),
            defaults: Defaults::new(x_axis.unwrap_or_default(), y_axis),
            legend: Some(Legend::hidden()),
            tooltip: tooltip.or_else(|| Some(Tooltip::hidden())),
            class,
        })
}

#[component]
#[allow(clippy::too_many_lines)]
pub fn horizon<T: 'static, D: Dataset<T>, X: Fn(&T) -> f64 + 'static>(
    data: D,
    x: X,
    bands: Option<usize>,
    #[into] positive: Option<Hsla>,
    #[into] negative: Option<Hsla>,
    x_axis: Option<AxisConfig>,
    class: Option<SharedString>,
    children: Children<TrendField<T>>,
) -> AnyElement {
    let table = use_signal(Table::default());
    let rows_of = children;
    let levels = bands.unwrap_or(3).max(1);
    let names: Vec<SharedString> = rows_of
        .iter()
        .enumerate()
        .map(|(index, field)| {
            field
                .name
                .clone()
                .unwrap_or_else(|| format!("Series {}", index + 1).into())
        })
        .collect();
    sync(
        table,
        &data,
        (levels, names.clone()),
        |rows| {
            let mut column_values = Vec::new();
            let series: Vec<Vec<f64>> = rows_of
                .iter()
                .map(|field| {
                    field.y.fill(rows, &mut column_values);
                    column_values.clone()
                })
                .collect();
            let steps: Vec<f64> = series
                .iter()
                .map(|values| {
                    let top = values
                        .iter()
                        .copied()
                        .filter(|value| value.is_finite())
                        .fold(0f64, |a, b| a.max(b.abs()));
                    if top > 0. { top / levels as f64 } else { 1. }
                })
                .collect();
            Build {
                rows: rows
                    .iter()
                    .enumerate()
                    .map(|(index, item)| {
                        let mut cells = vec![x(item)];
                        for (values, step) in series.iter().zip(&steps) {
                            let value = values[index];
                            for band in 0..levels * 2 {
                                let (sign, level) = if band < levels {
                                    (1., band)
                                } else {
                                    (-1., band - levels)
                                };
                                let part = (value * sign - level as f64 * step).clamp(0., *step);
                                cells.push(if value.is_finite() { part } else { f64::NAN });
                            }
                        }
                        cells.push(0.);
                        cells.into_boxed_slice()
                    })
                    .collect(),
                names: steps
                    .iter()
                    .map(|step| SharedString::from(step.to_string()))
                    .collect(),
                ..Build::default()
            }
        });
    let (up, down, muted) = {
        let colors = ChartTheme::colors();
        (
            positive.unwrap_or(colors.good),
            negative.unwrap_or(colors.bad),
            colors.neutral,
        )
    };
    let step_labels = table.with(|table| table.names.clone());
    let count = names.len();
    let mut rows: Vec<AnyElement> = Vec::with_capacity(count);
    for (index, name) in names.into_iter().enumerate() {
        let step = step_labels.get(index).cloned().unwrap_or_default();
        let fields: Vec<TrendField<Row>> = (0..levels * 2)
            .map(|band| {
                let (tint, level) = if band < levels {
                    (up, band)
                } else {
                    (down, band - levels)
                };
                field(
                    format!("band {band}"),
                    1 + index * levels * 2 + band,
                    Some(tint),
                    Style {
                        fill_opacity: Some(0.3 + 0.7 * (level + 1) as f32 / levels as f32),
                        line_width: Some(0.),
                        show_points: Some(PointsMode::Never),
                        ..Style::default()
                    },
                )
            })
            .collect();
        let last = index + 1 == count;
        let base_axis = x_axis.clone().unwrap_or_default();
        let reserve = gutter(&base_axis);
        let axis = if last {
            base_axis
        } else {
            base_axis.placement(AxisPlacement::Hidden)
        };
        let step_value: f64 = step.parse().unwrap_or(1.);
        // one trend per row, told apart by their order (as hooks are)
        let chart = {
            trend_view(
                table,
                TrendSpec {
                    fields,
                    style: Style::default(),
                    defaults: Defaults::new(
                        axis,
                        AxisConfig::new()
                            .placement(AxisPlacement::Hidden)
                            .min(0.)
                            .max(step_value),
                    ),
                    legend: Some(Legend::hidden()),
                    tooltip: Some(Tooltip::hidden()),
                    class: None,
                })
        };
        let floor = ROW_MIN + if last { reserve } else { 0. };
        rows.push(
            div()
                .class("flex flex-1 items-stretch")
                .min_h(px(floor))
                .child(
                    div()
                        .class("flex flex-col justify-center w-24 text-xs px-2")
                        .text_color(muted)
                        .child(div().child(name))
                        .child(div().class("opacity-70").child(format!("band {}", crate::format::trimmed(step_value)))),
                )
                .child(div().class("flex-1 min-w-0 min-h-0").child(chart))
                .into_any_element(),
        );
    }
    let body = div().class("flex flex-col w-full min-h-full").children(rows);
    div()
        .class("flex flex-col size-full")
        .when_some(class, Div::class)
        .child(scroll_area("horizon-scroll", body))
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swarm_keeps_points_apart() {
        let offsets = swarm(&[1., 1., 1., 1.], 0.5, 0.1, 1.);
        let mut sorted = offsets;
        sorted.sort_by(f64::total_cmp);
        sorted.dedup();
        assert_eq!(sorted.len(), 4);
    }

    #[test]
    fn density_integrates_to_about_one() {
        let grid: Vec<f64> = (0..400).map(|step| -5. + f64::from(step) * 0.025).collect();
        let curve = density(&[0.], &grid, 1.);
        let area: f64 = curve.iter().sum::<f64>() * 0.025;
        assert!((area - 1.).abs() < 0.01);
    }
}
