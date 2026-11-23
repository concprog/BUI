use blenderui::element::AnyElement;
use blenderui::{App, SharedString, Window};

use crate::component;

use crate::bar_gauge::GaugeColor;
use crate::legend::Legend;
use crate::mappings::{StateValue, ValueMappings};
use crate::plot::{AxisConfig, HideFrom, SelectedRange, Tooltip, VisibilityMode};
use crate::state_timeline::{Cells, Reader, StateLane, Style, Timeline, timeline};
use crate::{Dataset, Thresholds};

const COL_WIDTH: f32 = 0.9;
const LINE_WIDTH: f32 = 1.;

#[component]
pub fn status_history<
    T: 'static,
    D: Dataset<T>,
    X: Fn(&T) -> f64 + 'static,
    S: Fn(SelectedRange, &mut Window, &mut App) + 'static,
>(
    data: D,
    x: Option<X>,
    x_field: Option<&'static str>,
    show_value: Option<VisibilityMode>,
    row_height: Option<f32>,
    col_width: Option<f32>,
    per_page: Option<usize>,
    mappings: Option<&'static ValueMappings>,
    color: Option<GaugeColor>,
    thresholds: Option<Thresholds>,
    min: Option<f64>,
    max: Option<f64>,
    fill_opacity: Option<f32>,
    line_width: Option<f32>,
    hide_from: Option<HideFrom>,
    x_axis: Option<AxisConfig>,
    y_axis: Option<AxisConfig>,
    legend: Option<Legend>,
    tooltip: Option<Tooltip>,
    on_select: Option<S>,
    class: Option<SharedString>,
    children: Children<StateLane<T>>,
) -> AnyElement {
    let lanes = children;
    timeline(
        Timeline {
            data,
            x,
            x_field,
            cells: Cells::Samples {
                width: col_width.unwrap_or(COL_WIDTH),
            },
            show_value,
            row_height,
            per_page,
            mappings,
            color,
            thresholds,
            min,
            max,
            style: Style {
                fill_opacity,
                line_width: line_width.or(Some(LINE_WIDTH)),
                span_nulls: None,
                insert_nulls: None,
                hide_from,
            },
            x_axis,
            y_axis,
            legend,
            tooltip,
            class,
            lanes,
        },
        on_select)
}

#[component]
pub fn status_row<T: 'static, V: Into<StateValue>, F: Fn(&T) -> V + 'static>(
    name: Option<SharedString>,
    value: F,
    mappings: Option<&'static ValueMappings>,
    color: Option<GaugeColor>,
    thresholds: Option<Thresholds>,
    fill_opacity: Option<f32>,
    line_width: Option<f32>,
    hide_from: Option<HideFrom>,
) -> StateLane<T> {
    StateLane {
        name,
        value: Reader::Row(Box::new(move |row| value(row).into())),
        mappings,
        color,
        thresholds,
        style: Style {
            fill_opacity,
            line_width,
            span_nulls: None,
            insert_nulls: None,
            hide_from,
        },
    }
}
