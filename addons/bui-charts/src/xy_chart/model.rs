use std::cell::OnceCell;
use std::collections::BTreeSet;
use std::hash::{DefaultHasher, Hash, Hasher};

use blenderui::{Hsla, SharedString};

use super::series::{Color, Fields, Paint, XYSeries};
use super::style::Style;
use crate::calc::Calc;
use crate::data::Revision;
use crate::format::trimmed;
use crate::legend::{Entry, Legend, LegendItem};
use crate::names::common_label;
use crate::plot::{AxisConfig, Extent, Range, Scale, finite};
use crate::theme::ChartColors;

#[derive(Default)]
pub(crate) struct Cache {
    key: Option<u64>,
    pub columns: Vec<Columns>,
}

#[derive(Default)]
pub(crate) struct Columns {
    pub xs: Vec<f64>,
    pub ys: Vec<f64>,
    sizes: Vec<f64>,
    tints: Vec<f64>,
    diameters: Vec<f32>,
    point_size: f32,
    lut: Vec<Hsla>,
    tones: Vec<u8>,
    fallback: Hsla,
    x: Extent,
    y: Extent,
    calcs: [OnceCell<Option<f64>>; Calc::COUNT],
}

impl Columns {
    fn load<T>(&mut self, rows: &[T], series: &XYSeries<T>) {
        self.calcs = Default::default();
        if series.style.hide_from().viz {
            for column in [&mut self.xs, &mut self.ys, &mut self.sizes, &mut self.tints] {
                column.clear();
            }
        } else {
            series.x.fill(rows, &mut self.xs);
            series.y.fill(rows, &mut self.ys);
            match &series.size {
                Some(size) => size.fill(rows, &mut self.sizes),
                None => self.sizes.clear(),
            }
            match series.tint() {
                Some((value, _)) => value.fill(rows, &mut self.tints),
                None => self.tints.clear(),
            }
        }
        self.x = Extent::of(&self.xs);
        self.y = Extent::of(&self.ys);
    }

    fn finish<T>(
        &mut self,
        series: &XYSeries<T>,
        sizes: Range,
        tints: Range,
        colors: &ChartColors,
    ) {
        self.point_size = series.style.point_size();
        self.fallback = colors.fallback;
        self.diameters.clear();
        if let Some((min, max)) = sizes.filter(|(min, max)| max > min && !self.sizes.is_empty()) {
            let (smallest, largest) = series.style.size_range();
            let (smallest, largest) = (smallest * smallest, largest * largest);
            let span = max - min;
            self.diameters.extend(self.sizes.iter().map(|value| {
                let t = if value.is_finite() {
                    ((value - min) / span) as f32
                } else {
                    0.
                };
                (smallest + t * (largest - smallest)).max(0.).sqrt()
            }));
        }
        self.tones.clear();
        match series.tint() {
            Some((_, scheme)) => {
                scheme.palette(colors, &mut self.lut);
                let (extent, len) = (tints.unwrap_or_default(), self.lut.len());
                self.tones.extend(
                    self.tints
                        .iter()
                        .map(|value| scheme.tone(*value, extent, len)),
                );
            }
            None => self.lut.clear(),
        }
    }

    pub(crate) fn diameter(&self, index: usize) -> f32 {
        self.diameters
            .get(index)
            .copied()
            .unwrap_or(self.point_size)
    }

    pub(crate) fn fill(&self, index: usize, base: Hsla) -> Hsla {
        self.tones.get(index).map_or(base, |tone| {
            self.lut
                .get(usize::from(*tone))
                .copied()
                .unwrap_or(self.fallback)
        })
    }

    pub(crate) fn calc(&self, calc: Calc) -> Option<f64> {
        *self.calcs[calc as usize].get_or_init(|| calc.apply(&self.ys))
    }
}

impl Cache {
    pub(crate) fn fresh(&self, key: Option<u64>) -> bool {
        key.is_some() && self.key == key
    }

    pub(crate) fn store(&mut self, key: Option<u64>, columns: Vec<Columns>) {
        self.key = key;
        self.columns = columns;
    }
}

pub(crate) fn fingerprint<T>(
    revision: Revision,
    theme: (u64, bool),
    series: &[XYSeries<T>],
) -> u64 {
    let mut state = DefaultHasher::new();
    (revision, theme, series.len()).hash(&mut state);
    for series in series {
        series.fingerprint(&mut state);
    }
    state.finish()
}

pub(crate) fn refresh<T>(
    columns: &mut Vec<Columns>,
    rows: &[T],
    series: &[XYSeries<T>],
    colors: &ChartColors,
) {
    columns.resize_with(series.len(), Columns::default);
    for (columns, series) in columns.iter_mut().zip(series) {
        columns.load(rows, series);
    }
    let sizes = columns
        .iter()
        .flat_map(|columns| &columns.sizes)
        .fold(None, |range, value| finite(range, *value));
    let tints = columns
        .iter()
        .flat_map(|columns| &columns.tints)
        .fold(None, |range, value| finite(range, *value));
    for (columns, series) in columns.iter_mut().zip(series) {
        columns.finish(series, sizes, tints, colors);
    }
}

pub(crate) struct Plotted {
    pub index: usize,
    pub name: SharedString,
    pub color: Hsla,
    pub style: Style,
    pub hidden: bool,
    fields: Fields,
}

pub(crate) struct TooltipData {
    pub title: SharedString,
    pub color: Hsla,
    pub rows: [Option<(SharedString, SharedString)>; 4],
}

pub(crate) struct Model {
    pub series: Vec<Plotted>,
    pub count: usize,
    pub x: Scale,
    pub y: Scale,
    pub x_axis: AxisConfig,
    pub y_axis: AxisConfig,
    pub equal: Option<f64>,
}

impl Model {
    pub(crate) fn new<T>(
        series: Vec<XYSeries<T>>,
        columns: &[Columns],
        (mut x_axis, mut y_axis): (AxisConfig, AxisConfig),
        hidden: &BTreeSet<usize>,
        colors: &ChartColors,
    ) -> Self {
        let count = series.len();
        let plotted: Vec<Plotted> = series
            .into_iter()
            .enumerate()
            .filter(|(_, series)| !series.style.hide_from().viz)
            .map(|(index, series)| Plotted {
                index,
                name: series
                    .name
                    .unwrap_or_else(|| format!("Series {}", index + 1).into()),
                color: match series.color {
                    Some(Color(Paint::Fixed(color))) => color,
                    _ => colors.series(index),
                },
                style: series.style,
                hidden: hidden.contains(&index),
                fields: series.fields,
            })
            .collect();

        let extent = |pick: fn(&Columns) -> Extent| {
            plotted
                .iter()
                .filter(|series| !series.hidden)
                .map(|series| pick(&columns[series.index]))
                .fold(Extent::default(), Extent::merge)
        };
        let x = Scale::new(&x_axis, extent(|columns| columns.x).for_scale(x_axis.scale));
        let y = Scale::new(&y_axis, extent(|columns| columns.y).for_scale(y_axis.scale));
        if x_axis.label.is_none() {
            x_axis.label =
                common_label(plotted.iter().filter_map(|series| series.fields.x.as_ref()));
        }
        if y_axis.label.is_none() {
            y_axis.label =
                common_label(plotted.iter().filter_map(|series| series.fields.y.as_ref()));
        }
        Self {
            series: plotted,
            count,
            x,
            y,
            x_axis,
            y_axis,
            equal: None,
        }
    }

    fn find(&self, index: usize) -> Option<&Plotted> {
        self.series
            .iter()
            .find(|series| series.index == index && !series.hidden)
    }

    pub(crate) fn legend_items(&self, legend: Legend, columns: &[Columns]) -> Vec<LegendItem> {
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
            |index, calc| columns[index].calc(calc),
            |_, value| self.y_axis.text(value, None),
        )
    }

    pub(crate) fn tooltip(
        &self,
        columns: &[Columns],
        series: usize,
        index: usize,
    ) -> Option<TooltipData> {
        let series = self
            .find(series)
            .filter(|series| !series.style.hide_from().tooltip)?;
        let data = &columns[series.index];
        let (x, y) = (*data.xs.get(index)?, *data.ys.get(index)?);
        let fields = &series.fields;
        let label = |field: &Option<SharedString>,
                     fallback: &Option<SharedString>,
                     default: &'static str| {
            field
                .as_ref()
                .or(fallback.as_ref())
                .map_or_else(|| SharedString::Borrowed(default), SharedString::clone)
        };
        let extra = |field: &Option<SharedString>, default: &'static str, values: &[f64]| {
            values
                .get(index)
                .map(|value| (label(field, &None, default), trimmed(*value)))
        };
        Some(TooltipData {
            title: series.name.clone(),
            color: data.fill(index, series.color),
            rows: [
                Some((
                    label(&fields.x, &self.x_axis.label, "x"),
                    self.x_axis.text(x, None),
                )),
                Some((
                    label(&fields.y, &self.y_axis.label, "y"),
                    self.y_axis.text(y, None),
                )),
                extra(&fields.size, "size", &data.sizes),
                extra(&fields.color, "color", &data.tints),
            ],
        })
    }
}
