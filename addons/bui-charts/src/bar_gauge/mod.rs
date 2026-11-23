mod layout;
mod model;
mod options;
mod paint;

use std::collections::BTreeSet;
use std::mem;

use blenderui::element::{AnyElement, Div, FluentBuilder, IntoElement, ParentElement};
use blenderui::hooks::use_signal;
use blenderui::html::div;
use blenderui::{Pixels, SharedString, Styled, canvas};

pub(crate) use model::{Source, Text};
pub use options::{
    BarDisplay, BarOrientation, GaugeColor, NamePlacement, Reduce, Sizing, TextMode, ValueMode,
};

use crate::component;
use crate::legend::{LegendItem, legend_panel};
use crate::thresholds::{self, Thresholds};
use crate::{Calc, ChartTheme, Dataset, Legend};
use layout::Labels;
use model::Bars;
use paint::{Gauge, Store, Ui, paint};

const DEFAULT_MIN: f64 = 0.;
const DEFAULT_MAX: f64 = 100.;

#[component]
pub fn bar_gauge<
    T: 'static,
    D: Dataset<T>,
    N: Fn(&T) -> &str + 'static,
    V: Fn(&T) -> f64 + 'static,
    F: Fn(f64) -> SharedString + 'static,
>(
    data: D,
    name: Option<N>,
    value: Option<V>,
    reduce: Option<Reduce>,
    fields: Option<&'static [&'static str]>,
    display: Option<BarDisplay>,
    value_mode: Option<ValueMode>,
    name_placement: Option<NamePlacement>,
    text_mode: Option<TextMode>,
    orientation: Option<BarOrientation>,
    show_unfilled: Option<bool>,
    sizing: Option<Sizing>,
    title_size: Option<Pixels>,
    value_size: Option<Pixels>,
    min: Option<f64>,
    max: Option<f64>,
    thresholds: Option<Thresholds>,
    color: Option<GaugeColor>,
    format: Option<F>,
    decimals: Option<usize>,
    legend: Option<Legend>,
    class: Option<SharedString>,
) -> AnyElement {
    let bars = use_signal(Bars::default());
    let hidden = use_signal(BTreeSet::<usize>::new());

    let text = Text {
        format: format
            .as_ref()
            .map(|format| format as &dyn Fn(f64) -> SharedString),
        decimals,
    };
    let mut items = Vec::new();
    bars.update_untracked(|bars| mem::swap(&mut bars.items, &mut items));
    let revision = data.revision();
    let key = data.read(|rows, schema| {
        let source = Source {
            rows,
            name: name.as_ref().map(|name| name as &dyn Fn(&T) -> &str),
            value: value.as_ref().map(|value| value as &dyn Fn(&T) -> f64),
            schema,
            fields,
            skip: None,
            reduce: reduce.unwrap_or_else(|| {
                if value.is_some() { Reduce::All { limit: None } } else { Reduce::Calc(Calc::Last) }
            }),
        };
        let key = source.key(revision, text);
        if !bars.with(|bars| bars.fresh(key)) {
            source.fill(&mut items, text);
        }
        key
    });
    bars.update_untracked(|bars| bars.store(key, items));

    let legend = legend.unwrap_or_else(Legend::hidden);
    let range = (min.unwrap_or(DEFAULT_MIN), max.unwrap_or(DEFAULT_MAX));
    let gauge = Gauge {
        display: display.unwrap_or_default(),
        value_mode: value_mode.unwrap_or_default(),
        text_mode: text_mode.unwrap_or_default(),
        orientation: orientation.unwrap_or_default(),
        show_unfilled: show_unfilled.unwrap_or(true),
        sizing: sizing.unwrap_or_default(),
        labels: Labels {
            placement: name_placement.unwrap_or_default(),
            title: title_size.map(|size| size.0),
            value: value_size.map(|size| size.0),
        },
        range,
        thresholds,
        color: color.unwrap_or_default(),
        named: name.is_some(),
    };
    let store = Store { bars, hidden };
    let items = legend.visible().then(|| legend_items(&gauge, store));
    let count = bars.with(|bars| bars.items.len());
    let theme = crate::ui::theme();
    let ui = Ui { text: theme.foreground, track: theme.muted };
    let panel = items.map(|items| legend_panel(legend, items, hidden, count));

    div()
        .class("flex size-full min-w-0 min-h-0")
        .map(|el| if legend.on_right() { el.flex_row() } else { el.flex_col() })
        .when_some(class, Div::class)
        .child(
            div().class("relative flex-1 min-w-0 min-h-0").child(
                canvas(move |bounds, painter| paint(bounds.into(), &gauge, store, ui, painter))
                    .class("absolute inset-0 size-full"),
            ),
        )
        .child(panel)
        .into_any_element()
}

fn legend_items(gauge: &Gauge, store: Store) -> Vec<LegendItem> {
    let colors = ChartTheme::colors();
    let fallback = thresholds::fallback(&colors);
    let steps = thresholds::steps(gauge.thresholds.as_ref(), &fallback);
    store.hidden.with(|hidden| {
        store.bars.with(|bars| {
            bars.items
                .iter()
                .enumerate()
                .map(|(index, bar)| LegendItem {
                    index,
                    name: bar.name.clone(),
                    color: gauge.color.at(bar.value, index, steps, gauge.range, &colors),
                    disabled: hidden.contains(&index),
                    values: Default::default(),
                })
                .collect()
        })
    })
}
