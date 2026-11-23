use std::cell::OnceCell;
use std::collections::BTreeSet;
use std::hash::{DefaultHasher, Hash, Hasher};

use blenderui::{Hsla, SharedString};

use super::marks::Annotation;
use super::options::{Part, Style, ValueTransform};
use super::series::TrendField;
use crate::calc::Calc;
use crate::candlestick::Candles;
use crate::data::{Accessor, Revision};
use crate::format::trimmed;
use crate::legend::{Entry, Legend, LegendItem};
use crate::plot::{AxisConfig, AxisPlacement, Extent, Scale, SortOrder, Tooltip, TooltipMode};
use crate::theme::ChartColors;
use crate::{Scheme, Thresholds};

pub(crate) const UNSORTED: &str = "Values must be in ascending order";
const EXEMPLAR: u32 = 0xff00_ffb3;

#[derive(Default)]
pub(crate) struct Cache {
    key: Option<u64>,
    pub data: Data,
}

impl Cache {
    pub(crate) fn fresh(&self, key: Option<u64>) -> bool {
        key.is_some() && self.key == key
    }

    pub(crate) fn store(&mut self, key: Option<u64>, data: Data) {
        self.key = key;
        self.data = data;
    }
}

#[derive(Default)]
pub(crate) struct Data {
    pub xs: Vec<f64>,
    pub ends: Vec<f64>,
    pub lines: Vec<Line>,
    pub ascending: bool,
    scratch: Vec<f64>,
}

impl Data {
    pub(crate) fn center(&self, index: usize) -> f64 {
        let x = self.xs[index];
        self.ends.get(index).map_or(x, |end| (x + end) / 2.)
    }
}

#[derive(Default)]
pub(crate) struct Line {
    pub raw: Vec<f64>,
    pub values: Vec<f64>,
    pub base: Vec<f64>,
    pub percent: bool,
    calcs: [OnceCell<Option<f64>>; Calc::COUNT],
}

impl Line {
    pub(crate) fn calc(&self, calc: Calc) -> Option<f64> {
        *self.calcs[calc as usize].get_or_init(|| calc.apply(&self.raw))
    }

    fn extent(&self) -> Extent {
        Extent::of(&self.values).merge(Extent::of(&self.base))
    }
}

pub(crate) fn fingerprint<T>(
    revision: Revision,
    theme: (u64, bool),
    x: &Accessor<T>,
    ends: Option<&Accessor<T>>,
    fields: &[TrendField<T>],
    hidden: &BTreeSet<usize>,
) -> u64 {
    let mut state = DefaultHasher::new();
    (revision, theme, fields.len(), hidden, ends.is_some()).hash(&mut state);
    x.fingerprint(&mut state);
    if let Some(ends) = ends {
        ends.fingerprint(&mut state);
    }
    for field in fields {
        field.fingerprint(&mut state);
    }
    state.finish()
}

pub(crate) fn refresh<T>(
    data: &mut Data,
    rows: &[T],
    x: &Accessor<T>,
    ends: Option<&Accessor<T>>,
    fields: &[TrendField<T>],
    hidden: &BTreeSet<usize>,
    shift: f64,
) {
    x.fill(rows, &mut data.xs);
    match ends {
        Some(ends) => ends.fill(rows, &mut data.ends),
        None => data.ends.clear(),
    }
    data.ascending = ascending(&data.xs);
    data.lines.resize_with(fields.len(), Line::default);
    for (line, field) in data.lines.iter_mut().zip(fields) {
        line.calcs = Default::default();
        line.percent = false;
        line.base.clear();
        line.values.clear();
        if field.style.hide_from().viz || field.compare.is_some() {
            line.raw.clear();
            continue;
        }
        field.y.fill(rows, &mut line.raw);
        let sign = if field.style.transform() == ValueTransform::NegativeY {
            -1.
        } else {
            1.
        };
        line.values
            .extend(line.raw.iter().map(|value| value * sign));
        if let Some(floor) = &field.floor {
            floor.fill(rows, &mut line.base);
            for (value, base) in line.values.iter_mut().zip(&line.base) {
                *value += base;
            }
        }
    }
    if shift != 0. {
        compare(data, fields, shift);
    }
    stack(data, fields, hidden);
}

fn compare<T>(data: &mut Data, fields: &[TrendField<T>], shift: f64) {
    let tolerance = data
        .xs
        .windows(2)
        .map(|pair| pair[1] - pair[0])
        .filter(|gap| *gap > 0.)
        .fold(f64::INFINITY, f64::min)
        / 2.;
    for (index, field) in fields.iter().enumerate() {
        let Some(source) = field.compare else {
            continue;
        };
        let mut raw = std::mem::take(&mut data.lines[index].raw);
        let xs = &data.xs;
        let values = &data.lines[source].raw;
        raw.extend(xs.iter().map(|x| {
            let target = x - shift;
            let split = xs.partition_point(|at| *at < target);
            [split.checked_sub(1), Some(split)]
                .into_iter()
                .flatten()
                .filter(|at| {
                    xs.get(*at)
                        .is_some_and(|at| (at - target).abs() <= tolerance)
                })
                .find_map(|at| values.get(at).copied())
                .unwrap_or(f64::NAN)
        }));
        let line = &mut data.lines[index];
        line.values.extend_from_slice(&raw);
        line.raw = raw;
    }
}

fn ascending(xs: &[f64]) -> bool {
    let mut previous = f64::NEG_INFINITY;
    xs.iter().filter(|x| x.is_finite()).all(|&x| {
        let ordered = x >= previous;
        previous = x;
        ordered
    })
}

fn stack<T>(data: &mut Data, fields: &[TrendField<T>], hidden: &BTreeSet<usize>) {
    let member = |index: usize| {
        let style = &fields[index].style;
        (!hidden.contains(&index) && !style.hide_from().viz && fields[index].floor.is_none())
            .then(|| style.stacking().group())
            .flatten()
    };
    let len = data.xs.len();
    for first in 0..fields.len() {
        let Some(key) = member(first) else { continue };
        if (0..first).any(|earlier| member(earlier) == Some(key)) {
            continue;
        }
        let (_, percent) = key;
        let group = || (first..fields.len()).filter(move |index| member(*index) == Some(key));
        let scratch = &mut data.scratch;
        scratch.clear();
        scratch.resize(len * 3, 0.);
        let (totals, sums) = scratch.split_at_mut(len);
        let (positive, negative) = sums.split_at_mut(len);
        if percent {
            for index in group() {
                for (total, value) in totals.iter_mut().zip(&data.lines[index].values) {
                    if value.is_finite() {
                        *total += value.abs();
                    }
                }
            }
        }
        for index in group() {
            let line = &mut data.lines[index];
            line.percent = percent;
            line.base.clear();
            for i in 0..len {
                let raw = line.values.get(i).copied().unwrap_or(f64::NAN);
                let value = if percent && totals[i] > 0. {
                    raw / totals[i]
                } else {
                    raw
                };
                let sum = if value < 0. {
                    &mut negative[i]
                } else {
                    &mut positive[i]
                };
                line.base.push(*sum);
                if value.is_finite() {
                    *sum += value;
                    line.values[i] = *sum;
                }
            }
        }
    }
}

pub(crate) struct Plotted {
    pub index: usize,
    pub part: Option<Part>,
    pub name: SharedString,
    pub color: Hsla,
    pub style: Style,
    pub hidden: bool,
    pub axis: usize,
    pub below: Option<usize>,
    pub thresholds: Option<Thresholds>,
    pub scheme: Option<Scheme>,
}

pub(crate) struct YAxis {
    pub config: AxisConfig,
    pub scale: Scale,
    pub right: bool,
    pub color: Hsla,
}

pub(crate) struct Model {
    pub series: Vec<Plotted>,
    pub count: usize,
    pub x: Scale,
    pub x_axis: AxisConfig,
    pub axes: Vec<YAxis>,
    pub thresholds: Option<Thresholds>,
    pub scheme: Option<Scheme>,
    pub vertical: bool,
    pub zones: &'static [i32],
    pub notes: Vec<Annotation>,
    pub exemplars: Vec<(f64, f64)>,
    pub exemplar_color: Hsla,
    pub candles: Option<Candles>,
    pub layer: Option<u64>,
}

pub(crate) struct Defaults {
    pub x_axis: AxisConfig,
    pub y_axis: AxisConfig,
    pub thresholds: Option<Thresholds>,
    pub scheme: Option<Scheme>,
    pub vertical: bool,
    pub zones: &'static [i32],
    pub notes: Vec<Annotation>,
    pub exemplars: Vec<(f64, f64)>,
    pub exemplar_color: Option<Hsla>,
    pub candles: Option<Candles>,
}

impl Defaults {
    pub(crate) const fn new(x_axis: AxisConfig, y_axis: AxisConfig) -> Self {
        Self {
            x_axis,
            y_axis,
            thresholds: None,
            scheme: None,
            vertical: false,
            zones: &[],
            notes: Vec::new(),
            exemplars: Vec::new(),
            exemplar_color: None,
            candles: None,
        }
    }
}

pub(crate) struct TipRow {
    pub color: Hsla,
    pub name: SharedString,
    pub value: SharedString,
}

pub(crate) struct TipData {
    pub title: SharedString,
    pub rows: Vec<TipRow>,
}

fn percent_unit(value: f64) -> SharedString {
    format!("{}%", trimmed(value * 100.)).into()
}

impl Model {
    pub(crate) fn new<T>(
        fields: Vec<TrendField<T>>,
        data: &Data,
        defaults: Defaults,
        hidden: &BTreeSet<usize>,
        colors: &ChartColors,
    ) -> Self {
        let Defaults {
            x_axis,
            y_axis,
            thresholds,
            scheme,
            vertical,
            zones,
            notes,
            exemplars,
            exemplar_color,
            candles,
        } = defaults;
        let count = fields.len();
        let mut configs: Vec<AxisConfig> = Vec::new();
        let mut chart_axis = Some(y_axis);
        let mut series: Vec<Plotted> = Vec::with_capacity(count);
        for (index, field) in fields.into_iter().enumerate() {
            if field.style.hide_from().viz {
                continue;
            }
            let axis = match field.y_axis {
                None => {
                    if let Some(config) = chart_axis.take() {
                        configs.insert(0, config);
                        for series in &mut series {
                            series.axis += 1;
                        }
                    }
                    0
                }
                Some(config) => {
                    let shared = config.label.as_ref().and_then(|label| {
                        configs
                            .iter()
                            .position(|other| other.label.as_ref() == Some(label))
                    });
                    shared.unwrap_or_else(|| {
                        configs.push(config);
                        configs.len() - 1
                    })
                }
            };
            series.push(Plotted {
                index,
                part: field.part,
                name: field
                    .name
                    .unwrap_or_else(|| format!("Series {}", index + 1).into()),
                color: field.color.unwrap_or_else(|| colors.series(index)),
                style: field.style,
                hidden: hidden.contains(&index),
                axis,
                below: None,
                thresholds: field.thresholds,
                scheme: field.scheme,
            });
        }
        if chart_axis.is_some() && configs.is_empty() {
            configs.extend(chart_axis);
        }
        for position in 0..series.len() {
            let target = series[position].style.fill_below_to();
            if !target.is_empty() {
                series[position].below = series
                    .iter()
                    .position(|other| other.name.as_ref() == target);
            }
        }

        let x = Scale::exact(
            &x_axis,
            Extent::of(&data.xs)
                .merge(Extent::of(&data.ends))
                .for_scale(x_axis.scale),
        );
        let mut has_left = false;
        let axes = configs
            .into_iter()
            .enumerate()
            .map(|(axis, mut config)| {
                let members = || {
                    series
                        .iter()
                        .filter(move |series| series.axis == axis && !series.hidden)
                };
                let percent = members().all(|series| data.lines[series.index].percent)
                    && members().next().is_some();
                if percent {
                    config.min = config.min.or(Some(0.));
                    config.max = config.max.or(Some(1.));
                    if config.format.is_none() && config.decimals.is_none() {
                        config.format = Some(percent_unit);
                    }
                }
                let extent = members()
                    .map(|series| data.lines[series.index].extent())
                    .fold(Extent::default(), Extent::merge);
                let volume = candles.is_some_and(|look| look.mode.price())
                    && members().next().is_some()
                    && members().all(|series| series.part == Some(Part::Volume));
                if volume && let Some((_, high)) = extent.for_scale(config.scale) {
                    config.min = Some(0.);
                    config.max = Some(high * 7.);
                }
                let scale = Scale::new(&config, extent.for_scale(config.scale));
                let right = match config.placement {
                    AxisPlacement::Right => true,
                    AxisPlacement::Auto => has_left,
                    _ => false,
                };
                if !right && config.visible() {
                    has_left = true;
                }
                let color = members()
                    .next()
                    .map_or(colors.fallback, |series| series.color);
                YAxis {
                    config,
                    scale,
                    right,
                    color,
                }
            })
            .collect();

        Self {
            series,
            count,
            x,
            x_axis,
            axes,
            thresholds,
            scheme,
            vertical,
            zones,
            notes,
            exemplars,
            exemplar_color: exemplar_color.unwrap_or_else(|| blenderui::rgba(EXEMPLAR)),
            candles,
            layer: None,
        }
    }

    pub(crate) fn fingerprint(&self, data: Option<u64>) -> Option<u64> {
        let data = data?;
        let mut state = DefaultHasher::new();
        (data, self.vertical).hash(&mut state);
        let (low, high) = self.x.range();
        (low.to_bits(), high.to_bits()).hash(&mut state);
        for axis in &self.axes {
            let (low, high) = axis.scale.range();
            (low.to_bits(), high.to_bits(), axis.right).hash(&mut state);
        }
        for series in &self.series {
            (
                series.index,
                series.hidden,
                series.axis,
                series.part,
                series.below,
            )
                .hash(&mut state);
            crate::color::hash(&series.color, &mut state);
            series.style.fingerprint(&mut state);
            series.thresholds.hash(&mut state);
            series.scheme.hash(&mut state);
        }
        self.thresholds.hash(&mut state);
        self.scheme.hash(&mut state);
        if let Some(look) = self.candles {
            (
                look.mode,
                look.style,
                look.strategy,
                look.volume_alpha.to_bits(),
            )
                .hash(&mut state);
            for color in [look.up, look.down, look.flat] {
                crate::color::hash(&color, &mut state);
            }
        }
        Some(state.finish())
    }

    pub(crate) fn visible(&self) -> impl Iterator<Item = &Plotted> {
        self.series.iter().filter(|series| !series.hidden)
    }

    pub(crate) fn legend_items(&self, legend: Legend, data: &Data) -> Vec<LegendItem> {
        legend.items(
            self.series
                .iter()
                .filter(|series| !series.style.hide_from().legend)
                .map(|series| Entry {
                    index: series.index,
                    name: &series.name,
                    color: series.color,
                    hidden: series.hidden,
                }),
            |index, calc| data.lines[index].calc(calc),
            |index, value| self.value_text(data, index, value),
        )
    }

    fn value_text(&self, data: &Data, index: usize, value: f64) -> SharedString {
        let axis = self
            .series
            .iter()
            .find(|series| series.index == index)
            .map(|series| &self.axes[series.axis].config);
        match axis {
            Some(axis) if !data.lines[index].percent => axis.text(value, None),
            _ => trimmed(value),
        }
    }

    pub(crate) fn tooltip(
        &self,
        data: &Data,
        index: usize,
        single: Option<usize>,
        tooltip: Tooltip,
    ) -> Option<TipData> {
        let x = *data.xs.get(index)?;
        let mut rows: Vec<(f64, TipRow)> = self
            .visible()
            .filter(|series| !series.style.hide_from().tooltip)
            .filter(|series| tooltip.mode == TooltipMode::Multi || Some(series.index) == single)
            .filter_map(|series| {
                let value = *data.lines[series.index].raw.get(index)?;
                let shown = value.is_finite() && !(tooltip.hide_zeros && value == 0.);
                shown.then(|| {
                    let row = TipRow {
                        color: series.color,
                        name: series.name.clone(),
                        value: self.value_text(data, series.index, value),
                    };
                    (value, row)
                })
            })
            .collect();
        if rows.is_empty() {
            return None;
        }
        if tooltip.sort != SortOrder::None {
            rows.sort_by(|a, b| a.0.total_cmp(&b.0));
            if tooltip.sort == SortOrder::Descending {
                rows.reverse();
            }
        }
        Some(TipData {
            title: match data.ends.get(index) {
                Some(end) => format!(
                    "{} - {}",
                    self.x_axis.text(x, None),
                    self.x_axis.text(*end, None)
                )
                .into(),
                None => self.x_axis.text(x, None),
            },
            rows: rows.into_iter().map(|(_, row)| row).collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trend::options::Stacking;

    #[test]
    fn compare_shifts_values_forward() {
        let rows = [[0., 10., 0.], [1., 20., 0.], [2., 30., 0.], [3., 40., 0.]];
        let mut twin = field(Stacking::None);
        twin.compare = Some(0);
        let fields = [
            TrendField {
                y: Accessor::Field(|row: &[f64; 3], index| row[index], 1),
                ..field(Stacking::None)
            },
            twin,
        ];
        let mut data = Data::default();
        let x = Accessor::Field(|row: &[f64; 3], index| row[index], 0);
        refresh(&mut data, &rows, &x, None, &fields, &BTreeSet::new(), 1.);
        assert!(data.lines[1].values[0].is_nan());
        assert_eq!(&data.lines[1].values[1..], &[10., 20., 30.]);
    }

    fn field(stacking: Stacking) -> TrendField<[f64; 3]> {
        TrendField {
            name: None,
            y: Accessor::Field(|row, index| row[index], 0),
            color: None,
            y_axis: None,
            thresholds: None,
            scheme: None,
            style: Style {
                stacking: Some(stacking),
                ..Style::default()
            },
            compare: None,
            part: None,
            floor: None,
        }
    }

    fn fields(stacking: Stacking) -> Vec<TrendField<[f64; 3]>> {
        (1..3)
            .map(|index| TrendField {
                y: Accessor::Field(|row: &[f64; 3], index| row[index], index),
                ..field(stacking)
            })
            .collect()
    }

    #[test]
    fn rejects_unsorted_x() {
        assert!(ascending(&[1., 2., f64::NAN, 2., 5.]));
        assert!(!ascending(&[1., 3., 2.]));
    }

    #[test]
    fn stacks_positive_values_on_top_of_each_other() {
        let rows = [[0., 1., 2.], [1., 3., -1.]];
        let mut data = Data::default();
        let x = Accessor::Field(|row: &[f64; 3], index| row[index], 0);
        refresh(
            &mut data,
            &rows,
            &x,
            None,
            &fields(Stacking::NORMAL),
            &BTreeSet::new(),
            0.,
        );
        assert_eq!(data.lines[1].base, [1., 0.]);
        assert_eq!(data.lines[1].values, [3., -1.]);
    }

    #[test]
    fn percent_stacking_sums_to_one() {
        let rows = [[0., 1., 3.]];
        let mut data = Data::default();
        let x = Accessor::Field(|row: &[f64; 3], index| row[index], 0);
        refresh(
            &mut data,
            &rows,
            &x,
            None,
            &fields(Stacking::PERCENT),
            &BTreeSet::new(),
            0.,
        );
        assert!((data.lines[1].values[0] - 1.).abs() < 1e-9);
        assert!((data.lines[0].values[0] - 0.25).abs() < 1e-9);
    }
}
