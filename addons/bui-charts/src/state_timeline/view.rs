use blenderui::element::{Div, FluentBuilder, ParentElement};
use blenderui::hooks::Signal;
use blenderui::html::div;
use blenderui::{Hsla, SharedString, Styled, canvas};

use super::model::{Cache, LegendItems};
use super::paint::Hover;
use crate::color::mix;
use crate::geom::{Bounds, point, size};
use crate::icon::{IconName, icon};
use crate::plot::{Tooltip, corner};
use crate::trend::card;

/// Slices a legend's gradient is drawn in.
const RAMP: usize = 32;

pub(crate) struct TipRow {
    pub lane: SharedString,
    pub color: Hsla,
    pub value: SharedString,
    pub start: SharedString,
    pub length: SharedString,
}

pub(crate) fn state_tip(tooltip: Tooltip, hover: Hover, title: SharedString, rows: Vec<TipRow>) -> Div {
    let theme = crate::ui::theme();
    let muted = theme.popover_muted;
    let single = rows.len() == 1;
    corner(card(), hover.at, hover.area)
        .class("overflow-hidden")
        .bg(theme.popover)
        .text_color(theme.popover_foreground)
        .border_color(theme.border)
        .when_some(tooltip.max_width, Styled::max_w)
        .when_some(tooltip.max_height, Styled::max_h)
        .child(div().class("font-semibold").text_color(muted).child(title))
        .children(rows.into_iter().map(move |row| {
            let span = (single && !row.start.is_empty()).then(|| {
                div()
                    .class("flex justify-between gap-4")
                    .text_color(muted)
                    .child(div().child(row.start))
                    .child(div().child(row.length))
            });
            div()
                .class("flex flex-col")
                .child(
                    div()
                        .class("flex items-center justify-between gap-4")
                        .child(
                            div()
                                .class("flex items-center gap-2")
                                .child(div().class("size-[10px] rounded-sm").bg(row.color))
                                .child(div().text_color(muted).child(row.lane)),
                        )
                        .child(div().class("font-medium").child(row.value)),
                )
                .child(span)
        }))
}

/// A gradient from `from` to `to`, left to right.
fn ramp(from: Hsla, to: Hsla) -> Div {
    canvas(move |bounds, painter| {
        let bounds = Bounds::from(bounds);
        let width = bounds.size.width / RAMP as f32;
        for index in 0..RAMP {
            let slice = Bounds::new(
                bounds.origin + point(width * index as f32, 0.),
                size(width + 0.5, bounds.size.height),
            );
            painter.fill(slice.into(), mix(from, to, (index as f32 + 0.5) / RAMP as f32));
        }
    })
    .class("h-2 w-40")
}

pub(crate) fn state_legend(cache: Signal<Cache>, vertical: bool) -> Div {
    let muted = crate::ui::theme().muted_foreground;
    let (items, gradient) = cache.with(|cache| match &cache.legend {
        Some(LegendItems::List(list)) => (list.clone(), None),
        Some(LegendItems::Gradient(stops, low, high)) => (
            Vec::new(),
            stops.first().copied().zip(stops.last().copied()).map(|(from, to)| (from, to, low.clone(), high.clone())),
        ),
        _ => (Vec::new(), None),
    });
    let bar = gradient.map(|(from, to, low, high)| {
        div().class("flex items-center gap-2").child(div().child(low)).child(ramp(from, to)).child(div().child(high))
    });
    div()
        .class("flex flex-wrap items-center gap-x-4 gap-y-1 px-2 py-1 text-xs")
        .when(vertical, |el| el.flex_col().items_start().flex_nowrap())
        .text_color(muted)
        .children(items.into_iter().map(|(label, color)| {
            div().class("flex items-center gap-2").child(div().class("w-3 h-1 rounded-sm").bg(color)).child(label)
        }))
        .child(bar)
}

pub(crate) fn pager(page: Signal<usize>, pages: usize) -> Div {
    let theme = crate::ui::theme();
    let (muted, border) = (theme.muted_foreground, theme.border);
    let current = page.get().min(pages.saturating_sub(1));
    let step = |id: &'static str, name: IconName, dim: bool| {
        div()
            .id(id)
            .class("flex items-center justify-center size-6 rounded-md border cursor-pointer")
            .border_color(border)
            .when(dim, |el| el.opacity(0.4))
            .child(icon(name, muted))
    };
    div()
        .class("flex items-center justify-center gap-2 py-1 text-xs")
        .text_color(muted)
        .child(
            step("previous-page", IconName::ChevronLeft, current == 0)
                .on_click(move |_, _, _| page.update(|page| *page = page.saturating_sub(1))),
        )
        .child(div().child(format!("{} / {pages}", current + 1)))
        .child(
            step("next-page", IconName::ChevronRight, current + 1 >= pages)
                .on_click(move |_, _, _| page.update(|page| *page = (*page + 1).min(pages.saturating_sub(1)))),
        )
}
