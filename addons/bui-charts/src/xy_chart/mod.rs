mod layout;
mod model;
mod paint;
mod series;
mod style;
mod tooltip;

use std::collections::BTreeSet;
use std::mem;
use std::ops::RangeInclusive;

use blenderui::element::{AnyElement, Div, FluentBuilder, IntoElement, ParentElement};
use blenderui::hooks::use_signal;
use blenderui::html::div;
use blenderui::{MouseButton, SharedString, Styled, canvas};

pub use series::{Color, Series, SeriesProps, XYSeries, series};
pub(crate) use style::Style;
pub use style::{PointShape, Show};

use crate::component;
use crate::legend::legend_panel;
use crate::plot::{AxisConfig, Drag, HideFrom, LineStyle, Tooltip, Zoom};
use crate::surface::use_surface;
use crate::{ChartTheme, Dataset, Legend};
use layout::{Hit, Layout};
use model::{Cache, Model};
use paint::{Palette, Pointer, paint};
use tooltip::tooltip_card;

#[component]
pub fn x_y_chart<T: 'static, D: Dataset<T>>(
    data: D,
    show: Option<Show>,
    point_size: Option<f32>,
    size_range: Option<RangeInclusive<f32>>,
    point_shape: Option<PointShape>,
    point_stroke_width: Option<f32>,
    fill_opacity: Option<f32>,
    line_width: Option<f32>,
    line_style: Option<LineStyle>,
    hide_from: Option<HideFrom>,
    x_axis: Option<AxisConfig>,
    y_axis: Option<AxisConfig>,
    legend: Option<Legend>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
    children: Children<XYSeries<T>>,
) -> AnyElement {
    let series = children;
    let style = Style {
        show,
        point_size,
        size_range: size_range.map(RangeInclusive::into_inner),
        point_shape,
        point_stroke_width,
        fill_opacity,
        line_width,
        line_style,
        hide_from,
    };
    plot(
        Plot {
            data,
            series,
            style,
            x_axis,
            y_axis,
            legend,
            tooltip,
            class,
            equal: None,
        })
}

pub(crate) struct Plot<T, D> {
    pub data: D,
    pub series: Vec<XYSeries<T>>,
    pub style: Style,
    pub x_axis: Option<AxisConfig>,
    pub y_axis: Option<AxisConfig>,
    pub legend: Option<Legend>,
    pub tooltip: Option<Tooltip>,
    pub class: Option<SharedString>,
    pub equal: Option<f64>,
}

#[allow(clippy::too_many_lines)]
pub(crate) fn plot<T: 'static, D: Dataset<T>>(options: Plot<T, D>) -> AnyElement {
    let hover = use_signal(None::<Hit>);
    let hidden = use_signal(BTreeSet::<usize>::new());
    let cache = use_signal(Cache::default());
    let zoom = use_signal(Zoom::default());
    let drag = use_signal(None::<Drag>);
    let surface = use_surface::<Layout>();
    let Plot { data, mut series, style: defaults, x_axis, y_axis, legend, tooltip, class, equal } = options;
    let legend = legend.unwrap_or_default();
    let tooltip = tooltip.unwrap_or_default();
    if series.is_empty() {
        series = data.schema(|schema| schema.map_or_else(Vec::new, series::auto));
    }
    for series in &mut series {
        series.style = series.style.or(defaults);
    }

    let colors = ChartTheme::colors();
    let key = data.revision().map(|revision| model::fingerprint(revision, ChartTheme::revision(), &series));
    if !cache.with(|cache| cache.fresh(key)) {
        let mut columns = Vec::new();
        cache.update_untracked(|cache| mem::swap(&mut cache.columns, &mut columns));
        data.rows(|rows| model::refresh(&mut columns, rows, &series, &colors));
        cache.update_untracked(|cache| cache.store(key, columns));
    }

    let (model, tip, items) = cache.with(|cache| {
        let columns = &cache.columns;
        let axes = zoomed(zoom.get(), x_axis.unwrap_or_default(), y_axis.unwrap_or_default());
        let mut model = hidden.with(|hidden| Model::new(series, columns, axes, hidden, &colors));
        model.equal = equal;
        let tip = hover
            .get()
            .filter(|_| tooltip.visible())
            .and_then(|hit| Some((hit, model.tooltip(columns, hit.series, hit.index)?)));
        let items = legend.visible().then(|| model.legend_items(legend, columns));
        (model, tip, items)
    });
    let theme = crate::ui::theme();
    let palette = Palette { text: theme.muted_foreground, grid: theme.chart_grid, border: theme.border, hover: colors.hover };

    if model.series.is_empty() {
        return div()
            .class("flex size-full items-center justify-center text-sm")
            .text_color(palette.text)
            .when_some(class, Div::class)
            .child("No data")
            .into_any_element();
    }

    let count = model.count;
    surface.set(model);
    let pointer = Pointer { surface, cache, hover, zoom, drag, tips: tooltip.visible() };
    let card = tip.map(|(hit, data)| tooltip_card(tooltip, hit, data));
    let panel = items.map(|items| legend_panel(legend, items, hidden, count));
    let plot = canvas(move |bounds, painter| surface.paint(bounds.into(), |layout| paint(layout, pointer, palette, painter)))
        .class("absolute inset-0 size-full")
        .on_mouse_move(move |event, _, _| paint::on_move(pointer, event))
        .on_hover(move |inside, _, _| {
            if !inside {
                paint::on_leave(pointer);
            }
        })
        .on_mouse_down(MouseButton::Left, move |event, _, _| paint::on_down(pointer, event))
        .on_mouse_up(MouseButton::Left, move |_, _, _| paint::on_up(pointer))
        .on_scroll_wheel(move |event, _, cx| paint::on_wheel(pointer, event, cx));

    div()
        .class("flex size-full min-w-0 min-h-0")
        .map(|el| if legend.on_right() { el.flex_row() } else { el.flex_col() })
        .when_some(class, Div::class)
        .child(div().class("relative flex-1 min-w-0 min-h-0").child(plot).child(card))
        .child(panel)
        .into_any_element()
}

const fn zoomed(zoom: Zoom, mut x_axis: AxisConfig, mut y_axis: AxisConfig) -> (AxisConfig, AxisConfig) {
    if let Some((low, high)) = zoom.x {
        (x_axis.min, x_axis.max) = (Some(low), Some(high));
    }
    if let Some((low, high)) = zoom.y {
        (y_axis.min, y_axis.max) = (Some(low), Some(high));
    }
    (x_axis, y_axis)
}
