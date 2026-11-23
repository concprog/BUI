mod layout;
mod options;
mod paint;

use blenderui::element::{AnyElement, Div, FluentBuilder, IntoElement};
use blenderui::hooks::use_signal;
use blenderui::{Pixels, SharedString, canvas};

pub use options::{GraphMode, JustifyMode, PercentChangeColor, StatColorMode};

use crate::bar_gauge::{BarOrientation, GaugeColor, Reduce, Source, Text, TextMode};
use crate::mappings::ValueMappings;
use crate::single_stat::{Times, Values};
use crate::component;
use crate::{Calc, Dataset, Thresholds};
use layout::Sizes;
use paint::{Look, Ui, paint};

#[component]
pub fn stat<
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
    color_mode: Option<StatColorMode>,
    graph_mode: Option<GraphMode>,
    justify_mode: Option<JustifyMode>,
    text_mode: Option<TextMode>,
    orientation: Option<BarOrientation>,
    wide_layout: Option<bool>,
    show_percent_change: Option<bool>,
    percent_change_color: Option<PercentChangeColor>,
    title_size: Option<Pixels>,
    value_size: Option<Pixels>,
    percent_size: Option<Pixels>,
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
    let graph_mode = graph_mode.unwrap_or_default();
    let percent = show_percent_change.unwrap_or(false);
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
        let flags = (graph_mode != GraphMode::None, percent, x.is_some());
        let key = Values::key(&source, revision, text, flags);
        if !values.with(|values| values.fresh(key)) {
            values.update_untracked(|slot| slot.refresh(key, &source, &times, text, flags.0, flags.1));
        }
    });

    let theme = crate::ui::theme();
    let ui = Ui {
        text: theme.foreground,
        dark: theme.dark,
    };
    let look = Look {
        color_mode: color_mode.unwrap_or_default(),
        graph_mode,
        justify: justify_mode.unwrap_or_default(),
        text_mode: text_mode.unwrap_or_default(),
        orientation: orientation.unwrap_or_default(),
        wide: wide_layout.unwrap_or(true),
        percent,
        percent_color: percent_change_color.unwrap_or_default(),
        sizes: Sizes {
            title: title_size.map(|size| size.0),
            value: value_size.map(|size| size.0),
            percent: percent_size.map(|size| size.0),
        },
        range: (min, max),
        thresholds,
        color: color.unwrap_or_default(),
        mappings,
    };

    canvas(move |bounds, painter| paint(bounds.into(), &look, values, ui, painter))
        .class("size-full min-w-0 min-h-0")
        .when_some(class, Div::class)
        .into_any_element()
}
