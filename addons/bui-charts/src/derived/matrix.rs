use blenderui::element::{AnyElement, Div, FluentBuilder, IntoElement, ParentElement};
use blenderui::hooks::use_signal;
use blenderui::html::div;
use blenderui::{SharedString, rsx};

use crate::derive::{Build, Row, Rows, Table, row, sync, sync_with};
use crate::heatmap::{Heatmap, HeatmapColor, HeatmapScheme};
use crate::mappings::StateValue;
use crate::plot::{AxisConfig, AxisPlacement, HideFrom, SelectedRange, Tooltip, VisibilityMode};
use crate::state_timeline::{Cells, Reader, StateLane, Style, Timeline, timeline};
use crate::{Dataset, Legend, component};

pub(crate) fn fft(re: &mut [f64], im: &mut [f64]) {
    let size = re.len();
    let mut target = 0;
    for index in 1..size {
        let mut bit = size >> 1;
        while target & bit != 0 {
            target ^= bit;
            bit >>= 1;
        }
        target |= bit;
        if index < target {
            re.swap(index, target);
            im.swap(index, target);
        }
    }
    let mut length = 2;
    while length <= size {
        let angle = -2. * std::f64::consts::PI / length as f64;
        for start in (0..size).step_by(length) {
            for offset in 0..length / 2 {
                let (sin, cos) = (angle * offset as f64).sin_cos();
                let (a, b) = (start + offset, start + offset + length / 2);
                let (tr, ti) = (
                    re[b].mul_add(cos, -im[b] * sin),
                    re[b].mul_add(sin, im[b] * cos),
                );
                re[b] = re[a] - tr;
                im[b] = im[a] - ti;
                re[a] += tr;
                im[a] += ti;
            }
        }
        length <<= 1;
    }
}

#[component]
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub fn spectrogram<
    T: 'static,
    D: Dataset<T>,
    X: Fn(&T) -> f64 + 'static,
    V: Fn(&T) -> f64 + 'static,
>(
    data: D,
    x: X,
    value: V,
    window_size: Option<usize>,
    overlap: Option<f64>,
    scheme: Option<HeatmapScheme>,
    class: Option<SharedString>,
) -> AnyElement {
    let table = use_signal(Table::default());
    let size = window_size.unwrap_or(256).max(8).next_power_of_two();
    let overlap = overlap.unwrap_or(0.5).clamp(0., 0.95);
    sync(
        table,
        &data,
        (size, overlap.to_bits()),
        |rows| {
            let times: Vec<f64> = rows.iter().map(&x).collect();
            let samples: Vec<f64> = rows
                .iter()
                .map(|item| {
                    let sample = value(item);
                    if sample.is_finite() { sample } else { 0. }
                })
                .collect();
            let mut gaps: Vec<f64> = times
                .windows(2)
                .map(|pair| pair[1] - pair[0])
                .filter(|gap| *gap > 0.)
                .collect();
            gaps.sort_by(f64::total_cmp);
            let step = gaps.get(gaps.len() / 2).copied().unwrap_or(1.);
            let rate = 1. / step;
            let hop = ((size as f64 * (1. - overlap)) as usize).max(1);
            let hann: Vec<f64> = (0..size)
                .map(|index| {
                    0.5 - 0.5 * (2. * std::f64::consts::PI * index as f64 / (size - 1) as f64).cos()
                })
                .collect();
            let (mut re, mut im) = (vec![0.; size], vec![0.; size]);
            let mut out: Vec<Row> = Vec::new();
            let mut start = 0;
            while start + size <= samples.len() {
                for index in 0..size {
                    re[index] = samples[start + index] * hann[index];
                    im[index] = 0.;
                }
                fft(&mut re, &mut im);
                let (from, to) = (times[start], times[start] + hop as f64 * step);
                for bin in 0..size / 2 {
                    let power = re[bin].mul_add(re[bin], im[bin] * im[bin]) / size as f64;
                    let decibels = 10. * (power + 1e-12).log10();
                    let frequency = bin as f64 * rate / size as f64;
                    out.push(row([
                        from,
                        frequency,
                        decibels,
                        to,
                        frequency + rate / size as f64,
                    ]));
                }
                start += hop;
            }
            Build {
                rows: out,
                ..Build::default()
            }
        });
    boxed(class, rsx! {
            <Heatmap
                data={Rows(table)}
                x={|r: &Row| r[0]}
                y={|r: &Row| r[1]}
                value={|r: &Row| r[2]}
                x_end={|r: &Row| r[3]}
                y_end={|r: &Row| r[4]}
                color={HeatmapColor::scheme(scheme.unwrap_or(HeatmapScheme::Magma))}
                x_axis={AxisConfig::new().label("Time (s)")}
                y_axis={AxisConfig::new().label("Frequency (Hz)")}
                cell_decimals={1}
                class="size-full"
            />
    })
}

#[component]
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
#[must_use]
pub fn attention_map(
    tokens: Vec<SharedString>,
    queries: Option<Vec<SharedString>>,
    weights: Vec<f64>,
    scheme: Option<HeatmapScheme>,
    show_values: Option<bool>,
    class: Option<SharedString>,
) -> AnyElement {
    let table = use_signal(Table::default());
    let queries = queries.unwrap_or_else(|| tokens.clone());
    let (columns, lines) = (tokens.len(), queries.len());
    sync_with(
        table,
        (crate::derive::bits(&weights), columns, lines),
        || Build {
            rows: (0..lines)
                .flat_map(|line| (0..columns).map(move |column| (line, column)))
                .map(|(line, column)| {
                    let weight = weights
                        .get(line * columns + column)
                        .copied()
                        .unwrap_or(f64::NAN);
                    let (x, y) = (column as f64, line as f64);
                    row([x - 0.5, y - 0.5, weight, x + 0.5, y + 0.5])
                })
                .collect(),
            ..Build::default()
        });
    let x_axis = AxisConfig::new()
        .categories(tokens)
        .min(-0.5)
        .max(columns as f64 - 0.5)
        .label("Key");
    let y_axis = AxisConfig::new()
        .categories(queries)
        .min(-0.5)
        .max(lines as f64 - 0.5)
        .reverse(true)
        .label("Query");
    let shown = if show_values.unwrap_or(columns <= 12) {
        VisibilityMode::Auto
    } else {
        VisibilityMode::Never
    };
    boxed(class, rsx! {
            <Heatmap
                data={Rows(table)}
                x={|r: &Row| r[0]}
                y={|r: &Row| r[1]}
                value={|r: &Row| r[2]}
                x_end={|r: &Row| r[3]}
                y_end={|r: &Row| r[4]}
                color={HeatmapColor::scheme(scheme.unwrap_or(HeatmapScheme::Blues))}
                x_axis={x_axis}
                y_axis={y_axis}
                show_value={shown}
                cell_decimals={2}
                cell_gap={1.}
                class="size-full"
            />
    })
}

pub(crate) fn apportion(values: &[f64], cells: usize) -> Vec<usize> {
    let total: f64 = values
        .iter()
        .copied()
        .filter(|value| value.is_finite() && *value > 0.)
        .sum();
    if total <= 0. {
        return vec![0; values.len()];
    }
    let exact: Vec<f64> = values
        .iter()
        .map(|value| {
            if value.is_finite() && *value > 0. {
                value / total * cells as f64
            } else {
                0.
            }
        })
        .collect();
    let mut counts: Vec<usize> = exact.iter().map(|share| share.floor() as usize).collect();
    let mut rest = cells.saturating_sub(counts.iter().sum());
    let mut order: Vec<usize> = (0..values.len()).collect();
    order.sort_by(|a, b| {
        (exact[*b] - exact[*b].floor()).total_cmp(&(exact[*a] - exact[*a].floor()))
    });
    for index in order {
        if rest == 0 {
            break;
        }
        if exact[index] > 0. {
            counts[index] += 1;
            rest -= 1;
        }
    }
    counts
}

#[component]
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub fn waffle<
    T: 'static,
    D: Dataset<T>,
    N: Fn(&T) -> &str + 'static,
    V: Fn(&T) -> f64 + 'static,
>(
    data: D,
    name: N,
    value: V,
    rows: Option<usize>,
    columns: Option<usize>,
    gap: Option<f32>,
    legend: Option<Legend>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
) -> AnyElement {
    let (height, width) = (rows.unwrap_or(10).max(1), columns.unwrap_or(10).max(1));
    let (names, values): (Vec<SharedString>, Vec<f64>) =
        data.rows(|items| items.iter().map(|item| (SharedString::from(name(item).to_owned()), value(item))).unzip());
    let counts = apportion(&values, height * width);
    let mut cells: Vec<Option<usize>> = Vec::with_capacity(height * width);
    for (index, count) in counts.iter().enumerate() {
        cells.extend(std::iter::repeat_n(Some(index), *count));
    }
    cells.resize(height * width, None);
    let lanes: Vec<StateLane<f64>> = (0..height)
        .map(|line| {
            let picks: Vec<Option<SharedString>> = (0..width)
                .map(|column| cells[line * width + column].map(|index| names[index].clone()))
                .collect();
            StateLane {
                name: Some(SharedString::default()),
                value: Reader::Row(Box::new(move |at: &f64| {
                    picks
                        .get(*at as usize)
                        .cloned()
                        .flatten()
                        .map_or(StateValue::Null, StateValue::Text)
                })),
                mappings: None,
                color: None,
                thresholds: None,
                style: Style::default(),
            }
        })
        .collect();
    let positions: Vec<f64> = (0..width).map(|column| column as f64).collect();
    let spacing = gap.unwrap_or(0.12).clamp(0., 0.9);
    timeline(
        Timeline {
            data: positions,
            x: Some(|at: &f64| *at),
            x_field: None,
            cells: Cells::Samples {
                width: 1. - spacing,
            },
            show_value: Some(VisibilityMode::Never),
            row_height: Some(1. - spacing),
            per_page: Some(height),
            mappings: None,
            color: None,
            thresholds: None,
            min: None,
            max: None,
            style: Style {
                fill_opacity: Some(1.),
                line_width: Some(0.),
                span_nulls: None,
                insert_nulls: None,
                hide_from: Some(HideFrom::default()),
            },
            x_axis: Some(AxisConfig::new().placement(AxisPlacement::Hidden)),
            y_axis: Some(AxisConfig::new().placement(AxisPlacement::Hidden)),
            legend,
            tooltip,
            class,
            lanes,
        },
        None::<fn(SelectedRange, &mut blenderui::Window, &mut blenderui::App)>)
}

/// The chart in a box that takes the chart's classes.
fn boxed(class: Option<SharedString>, chart: impl IntoElement) -> AnyElement {
    div().class("size-full").when_some(class, Div::class).child(chart).into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fft_finds_a_pure_tone() {
        let size = 64;
        let mut re: Vec<f64> = (0..size)
            .map(|index| (2. * std::f64::consts::PI * 8. * index as f64 / size as f64).sin())
            .collect();
        let mut im = vec![0.; size];
        fft(&mut re, &mut im);
        let peak = (0..size / 2)
            .max_by(|a, b| re[*a].hypot(im[*a]).total_cmp(&re[*b].hypot(im[*b])))
            .unwrap_or(0);
        assert_eq!(peak, 8);
    }

    #[test]
    fn apportion_fills_every_cell() {
        let counts = apportion(&[1., 1., 1.], 100);
        assert_eq!(counts.iter().sum::<usize>(), 100);
        assert!(counts.iter().all(|count| (33..=34).contains(count)));
    }
}
