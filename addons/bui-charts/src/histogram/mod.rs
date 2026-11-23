mod buckets;

use std::hash::{DefaultHasher, Hash, Hasher};

use blenderui::element::AnyElement;
use blenderui::hooks::{Signal, use_signal};
use blenderui::{App, Hsla, SharedString, Window};

use crate::component;

use crate::data::{Accessor, Revision};
use crate::plot::{AxisConfig, HideFrom, SelectedRange, Tooltip};
use crate::trend::{
    Chart, Defaults, DrawStyle, GradientMode, PointsMode, Stacking, Style, TrendField, chart,
};
use crate::{Dataset, Legend, Schema, Thresholds};
use buckets::Row;

const BUCKET_COUNT: usize = 30;
const FILL_OPACITY: f32 = 0.8;
const MIN_FIELD: &str = "xMin";
const MAX_FIELD: &str = "xMax";

pub struct HistogramField<T> {
    name: Option<SharedString>,
    value: Accessor<T>,
    color: Option<Hsla>,
    style: Style,
}

#[component]
pub fn histogram_series<T: 'static, V: Fn(&T) -> f64 + 'static>(
    name: Option<SharedString>,
    value: V,
    #[into] color: Option<Hsla>,
    fill_opacity: Option<f32>,
    line_width: Option<f32>,
    gradient: Option<GradientMode>,
    hide_from: Option<HideFrom>,
) -> HistogramField<T> {
    HistogramField {
        name,
        value: Accessor::row(value),
        color,
        style: Style {
            line_width,
            fill_opacity,
            gradient,
            hide_from,
            ..Style::default()
        },
    }
}

#[derive(Default)]
struct Table {
    key: Option<u64>,
    rows: Vec<Row>,
    size: f64,
}

#[derive(Clone, Copy)]
struct Buckets(Signal<Table>);

fn read(row: &Row, field: usize) -> f64 {
    row[field]
}

impl Dataset<Row> for Buckets {
    fn read<R>(&self, f: impl FnOnce(&[Row], Option<Schema<'_, Row>>) -> R) -> R {
        self.0.with(|table| f(&table.rows, None))
    }

    /// The table is rebuilt (without a write the signal counts) when its key changes.
    fn revision(&self) -> Option<Revision> {
        self.0.with(|table| table.key).map(|key| Revision { source: key, version: 0 })
    }
}

struct Source<T> {
    starts: Option<Accessor<T>>,
    ends: Option<Accessor<T>>,
    fields: Vec<HistogramField<T>>,
}

#[derive(Clone, Copy)]
struct Binning {
    count: usize,
    size: Option<f64>,
    offset: f64,
    combine: bool,
}

impl<T> Source<T> {
    fn fingerprint(&self, revision: Revision, binning: Binning) -> u64 {
        let mut state = DefaultHasher::new();
        (
            revision,
            self.fields.len(),
            binning.count,
            binning.size.map(f64::to_bits),
            binning.offset.to_bits(),
            binning.combine,
        )
            .hash(&mut state);
        for accessor in [&self.starts, &self.ends].into_iter().flatten() {
            accessor.fingerprint(&mut state);
        }
        for field in &self.fields {
            field.value.fingerprint(&mut state);
        }
        state.finish()
    }

    fn tabulate(&self, rows: &[T], binning: Binning) -> (Vec<Row>, f64) {
        let column = |accessor: &Accessor<T>| {
            let mut values = Vec::with_capacity(rows.len());
            accessor.fill(rows, &mut values);
            values
        };
        let mut values: Vec<Vec<f64>> = self
            .fields
            .iter()
            .map(|field| column(&field.value))
            .collect();
        if self.starts.is_some() || self.ends.is_some() {
            let starts = self.starts.as_ref().map(column).unwrap_or_default();
            let ends = self.ends.as_ref().map(column).unwrap_or_default();
            let table = buckets::counted(&starts, &ends, &values, binning.combine);
            let size = table.first().map_or(1., |row| row[1] - row[0]);
            return (table, size);
        }
        for series in &mut values {
            series.retain(|value| value.is_finite());
        }
        let size = binning.size.filter(|size| *size > 0.).unwrap_or_else(|| {
            let mut all: Vec<f64> = values.iter().flatten().copied().collect();
            all.sort_by(f64::total_cmp);
            buckets::auto_size(&all, binning.count)
        });
        (
            buckets::count(&values, size, binning.offset, binning.combine),
            size,
        )
    }
}

#[component]
pub fn histogram<
    T: 'static,
    D: Dataset<T>,
    L: Fn(&T) -> f64 + 'static,
    H: Fn(&T) -> f64 + 'static,
>(
    data: D,
    x_min: Option<L>,
    x_max: Option<H>,
    bucket_count: Option<usize>,
    bucket_size: Option<f64>,
    bucket_offset: Option<f64>,
    combine: Option<bool>,
    fill_opacity: Option<f32>,
    gradient: Option<GradientMode>,
    line_width: Option<f32>,
    stacking: Option<Stacking>,
    hide_from: Option<HideFrom>,
    thresholds: Option<Thresholds>,
    scheme: Option<crate::Scheme>,
    x_axis: Option<AxisConfig>,
    y_axis: Option<AxisConfig>,
    legend: Option<Legend>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
    children: Children<HistogramField<T>>,
) -> AnyElement {
    let table = use_signal(Table::default());
    let mut source = Source {
        starts: x_min.map(Accessor::row),
        ends: x_max.map(Accessor::row),
        fields: children,
    };
    if source.fields.is_empty() {
        data.schema(|schema| {
        let Some(schema) = schema else { return };
        let position = |name: &str| schema.names.iter().position(|field| field.as_ref() == name);
        let (low, high) = (position(MIN_FIELD), position(MAX_FIELD));
        if source.starts.is_none() && source.ends.is_none() {
            source.starts = low.map(|field| Accessor::Field(schema.read, field));
            source.ends = high.map(|field| Accessor::Field(schema.read, field));
        }
        source.fields = (0..schema.names.len())
            .filter(|field| Some(*field) != low && Some(*field) != high)
            .map(|field| HistogramField {
                name: Some(schema.names[field].clone()),
                value: Accessor::Field(schema.read, field),
                color: None,
                style: Style::default(),
            })
            .collect();
        });
    }
    let binning = Binning {
        count: bucket_count.unwrap_or(BUCKET_COUNT).max(1),
        size: bucket_size,
        offset: bucket_offset.unwrap_or(0.),
        combine: combine.unwrap_or(false),
    };
    let key = data
        .revision()
        .map(|revision| source.fingerprint(revision, binning));
    if key.is_none() || table.with(|table| table.key) != key {
        let (rows, size) = data.rows(|rows| source.tabulate(rows, binning));
        // a key that changes on every render when the data can't tell its revision
        let key = key.or_else(|| Some(table.with(|table| table.key.map_or(0, |key| key.wrapping_add(1)))));
        table.update_untracked(|table| {
            table.key = key;
            table.rows = rows;
            table.size = size;
        });
    }

    let defaults = Style {
        draw_style: Some(DrawStyle::Bars),
        show_points: Some(PointsMode::Never),
        fill_opacity: Some(fill_opacity.unwrap_or(FILL_OPACITY)),
        gradient,
        line_width,
        stacking,
        hide_from,
        ..Style::default()
    };
    let fields = if binning.combine {
        vec![TrendField::new(
            "Count".into(),
            Accessor::Field(read, 2),
            None,
            Style::default(),
        )]
    } else {
        source
            .fields
            .into_iter()
            .enumerate()
            .map(|(index, field)| {
                TrendField::new(
                    field
                        .name
                        .unwrap_or_else(|| format!("Series {}", index + 1).into()),
                    Accessor::Field(read, index + 2),
                    field.color,
                    field.style,
                )
            })
            .collect()
    };
    let mut x_axis = x_axis.unwrap_or_default();
    if x_axis.decimals.is_none() && x_axis.format.is_none() {
        x_axis.decimals = Some(buckets::decimals(table.with(|table| table.size)));
    }
    let mut y_axis = y_axis.unwrap_or_default();
    y_axis.soft_min = y_axis.soft_min.or(Some(0.));
    chart(
        Chart {
            data: Buckets(table),
            x: Some(Accessor::Field(read, 0)),
            x_field: None,
            ends: Some(Accessor::Field(read, 1)),
            fields,
            style: defaults,
            defaults: Defaults {
                x_axis,
                y_axis,
                thresholds,
                scheme,
                vertical: false,
                zones: &[],
                notes: Vec::new(),
                exemplars: Vec::new(),
                exemplar_color: None,
                candles: None,
            },
            annotations: None,
            compare: None,
            keyboard: true,
            legend,
            tooltip,
            class,
        },
        None::<fn(SelectedRange, &mut Window, &mut App)>)
}
