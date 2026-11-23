mod geometry;
mod options;
mod paint;

use blenderui::element::{AnyElement, Div, FluentBuilder, IntoElement};
use blenderui::hooks::use_signal;
use blenderui::{Pixels, SharedString, canvas};

pub use options::{BarShape, EndpointMarker, GaugeEffects, GaugeShape};

use crate::bar_gauge::{BarOrientation, GaugeColor, Reduce, Sizing, Source, Text, TextMode};
use crate::mappings::ValueMappings;
use crate::single_stat::{Times, Values};
use crate::thresholds::{ThresholdsMode, fallback, steps};
use crate::component;
use crate::{Calc, ChartTheme, Dataset, Thresholds};
use paint::{Look, Ui, bands, paint, range};

const WIDTH_FACTOR: f32 = 0.5;
const SEGMENT_SPACING: f32 = 0.3;

#[component]
#[allow(clippy::too_many_lines)]
pub fn gauge<
    T: 'static,
    D: Dataset<T>,
    N: Fn(&T) -> &str + 'static,
    V: Fn(&T) -> f64 + 'static,
    X: Fn(&T) -> f64 + 'static,
    F: Fn(f64) -> SharedString + 'static,
>(
    data: D,
    name: Option<N>,
    value: Option<V>,
    x: Option<X>,
    x_field: Option<&'static str>,
    reduce: Option<Reduce>,
    fields: Option<&'static [&'static str]>,
    shape: Option<GaugeShape>,
    bar_shape: Option<BarShape>,
    bar_width_factor: Option<f32>,
    segment_count: Option<usize>,
    segment_spacing: Option<f32>,
    effects: Option<GaugeEffects>,
    endpoint_marker: Option<EndpointMarker>,
    show_threshold_markers: Option<bool>,
    show_threshold_labels: Option<bool>,
    neutral: Option<f64>,
    sizing: Option<Sizing>,
    sparkline: Option<bool>,
    text_mode: Option<TextMode>,
    orientation: Option<BarOrientation>,
    title_size: Option<Pixels>,
    value_size: Option<Pixels>,
    min: Option<f64>,
    max: Option<f64>,
    thresholds: Option<Thresholds>,
    color: Option<GaugeColor>,
    mappings: Option<&'static ValueMappings>,
    format: Option<F>,
    decimals: Option<usize>,
    class: Option<SharedString>,
) -> AnyElement {
    let values = use_signal(Values::default());
    let sparkline = sparkline.unwrap_or(true);
    let text = Text {
        format: format
            .as_ref()
            .map(|format| format as &dyn Fn(f64) -> SharedString),
        decimals,
    };
    let revision = data.revision();
    data.read(|rows, schema| {
        let skip = schema
            .zip(x_field)
            .and_then(|(schema, field)| schema.names.iter().position(|name| name.as_ref() == field));
        let source = Source {
            rows,
            name: name.as_ref().map(|name| name as &dyn Fn(&T) -> &str),
            value: value.as_ref().map(|value| value as &dyn Fn(&T) -> f64),
            schema,
            fields,
            skip,
            reduce: reduce.unwrap_or(Reduce::Calc(Calc::Last)),
        };
        let times = match (&x, schema.zip(skip)) {
            (Some(x), _) => Times::Row(x),
            (None, Some((schema, field))) => Times::Field(schema.read, field),
            (None, None) => Times::Index,
        };
        let flags = (sparkline, false, x.is_some());
        let key = Values::key(&source, revision, text, flags);
        if !values.with(|values| values.fresh(key)) {
            values.update_untracked(|slot| slot.refresh(key, &source, &times, text, sparkline, false));
        }
    });

    let range = range(min, max);
    let labels = if show_threshold_labels.unwrap_or(false) {
        let fallback = fallback(&ChartTheme::colors());
        let steps = steps(thresholds.as_ref(), &fallback);
        let mut points: Vec<f64> = bands(steps, range)
            .iter()
            .map(|(low, _, _)| (range.1 - range.0).mul_add(*low, range.0))
            .collect();
        points.push(range.1);
        points.extend(neutral.filter(|neutral| !points.contains(neutral)));
        points
            .into_iter()
            .map(|point| {
                let label = if steps.mode == ThresholdsMode::Percentage {
                    let percent = (point - range.0) / (range.1 - range.0) * 100.;
                    format!("{}%", crate::format::trimmed(percent)).into()
                } else {
                    text.of(point)
                };
                (point, label)
            })
            .collect()
    } else {
        Vec::new()
    };
    let theme = crate::ui::theme();
    let ui = Ui {
        text: theme.foreground,
        muted: theme.muted_foreground,
        track: theme.muted,
        border: theme.border,
        dark: theme.dark,
    };
    let look = Look {
        shape: shape.unwrap_or_default(),
        bar_shape: bar_shape.unwrap_or_default(),
        width_factor: bar_width_factor.unwrap_or(WIDTH_FACTOR),
        segments: segment_count.unwrap_or(1).max(1),
        segment_spacing: segment_spacing.unwrap_or(SEGMENT_SPACING),
        effects: effects.unwrap_or_default(),
        endpoint: endpoint_marker.unwrap_or_default(),
        markers: show_threshold_markers.unwrap_or(true),
        labels,
        neutral,
        sizing: sizing.unwrap_or_default(),
        sparkline,
        text_mode: text_mode.unwrap_or_default(),
        orientation: orientation.unwrap_or_default(),
        value_size: value_size.map(|size| size.0),
        name_size: title_size.map(|size| size.0),
        range,
        thresholds,
        color: color.unwrap_or_default(),
        mappings,
        named: name.is_some(),
    };

    canvas(move |bounds, painter| paint(bounds.into(), &look, values, ui, painter))
        .class("size-full min-w-0 min-h-0")
        .when_some(class, Div::class)
        .into_any_element()
}
