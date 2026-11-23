use blenderui::element::{Div, FluentBuilder, ParentElement};
use blenderui::hooks::Signal;
use blenderui::html::div;
use blenderui::{Hsla, Pixels, SharedString, Styled, canvas, relative};

use super::model::Cache;
use super::options::HeatmapTooltip;
use super::paint::Hover;
use crate::geom::{Bounds, point, size};
use crate::plot::corner;
use crate::trend::card;

pub(crate) struct TipData {
    pub x: SharedString,
    pub y: SharedString,
    pub value: SharedString,
    pub color: Hsla,
    pub histogram: Vec<(f32, bool)>,
    pub position: Option<f32>,
}

/// The palette of the cache, left to right.
fn scale(cache: Signal<Cache>) -> Div {
    canvas(move |bounds, painter| {
        let bounds = Bounds::from(bounds);
        cache.with(|cache| {
            let count = cache.palette.len().max(1) as f32;
            let width = bounds.size.width / count;
            for (index, color) in cache.palette.iter().enumerate() {
                let band = Bounds::new(
                    bounds.origin + point(width * index as f32, 0.),
                    size(width + 0.5, bounds.size.height),
                );
                painter.fill(band.into(), *color);
            }
        });
    })
    .class("absolute inset-0 size-full")
}

/// A marker on a color scale at `at` (0..=1).
fn marker(at: f32, color: Hsla, class: &'static str) -> Div {
    div().class(class).left(relative(at)).bg(color)
}

pub(crate) fn heatmap_tip(tooltip: HeatmapTooltip, hover: Hover, data: TipData, cache: Signal<Cache>) -> Div {
    let theme = crate::ui::theme();
    let (muted, accent) = (theme.popover_muted, theme.primary);
    let histogram = (tooltip.y_histogram && !data.histogram.is_empty()).then(|| {
        div().class("flex items-end gap-[1px] h-12 w-[180px]").children(data.histogram.into_iter().map(
            move |(height, current)| {
                div().class("flex-1 min-h-[1px]").h(relative(height)).bg(if current { accent } else { muted.opacity(0.5) })
            },
        ))
    });
    let foreground = theme.popover_foreground;
    let colors = tooltip.color_scale.then(|| {
        div()
            .class("relative h-2 w-[180px]")
            .child(scale(cache))
            .child(data.position.map(|at| marker(at, foreground, "absolute top-[-2px] h-3 w-[2px]")))
    });
    let row = |label: Div, value: SharedString| {
        div()
            .class("flex items-center justify-between gap-4")
            .child(label)
            .child(div().class("font-medium").child(value))
    };
    corner(card(), hover.at, hover.area)
        .class("overflow-hidden")
        .bg(theme.popover)
        .text_color(foreground)
        .border_color(theme.border)
        .when_some(tooltip.max_width, Styled::max_w)
        .when_some(tooltip.max_height, Styled::max_h)
        .child(div().class("font-semibold").text_color(muted).child(data.x))
        .child(histogram)
        .child(row(div().text_color(muted).child("Bucket"), data.y))
        .child(row(
            div()
                .class("flex items-center gap-2")
                .child(div().class("size-[10px] rounded-sm").bg(data.color))
                .child(div().text_color(muted).child("Value")),
            data.value,
        ))
        .child(colors)
}

pub(crate) fn color_bar(
    cache: Signal<Cache>,
    low: SharedString,
    high: SharedString,
    at: Option<f32>,
    vertical: bool,
    width: Option<Pixels>,
) -> Div {
    let theme = crate::ui::theme();
    div()
        .class("flex items-center gap-2 text-xs px-2 py-1")
        .when(vertical, |el| el.flex_col().justify_center())
        .when_some(width.filter(|width| width.0 > 0.), Styled::w)
        .text_color(theme.muted_foreground)
        .child(div().class("whitespace-nowrap").child(low))
        .child(
            div()
                .class("relative h-2 flex-1 min-w-[80px] max-w-[320px]")
                .child(scale(cache))
                .child(at.map(|at| marker(at, theme.foreground, "absolute top-[-3px] w-[2px] h-[14px]"))),
        )
        .child(div().class("whitespace-nowrap").child(high))
}
