use blenderui::element::{Div, FluentBuilder, ParentElement};
use blenderui::html::div;
use blenderui::{SharedString, Styled};

use super::layout::Hover;
use super::marks::MarkHover;
use super::model::TipData;
use crate::plot::{Tooltip, corner};

/// The card of the series at the hovered x.
pub(crate) fn trend_tip(tooltip: Tooltip, hover: Hover, data: TipData) -> Div {
    let theme = crate::ui::theme();
    corner(card(), hover.at, hover.area)
        .class("overflow-hidden")
        .bg(theme.popover)
        .text_color(theme.popover_foreground)
        .border_color(theme.border)
        .when_some(tooltip.max_width, Styled::max_w)
        .when_some(tooltip.max_height, Styled::max_h)
        .child(div().class("font-semibold").text_color(theme.popover_muted).child(data.title))
        .children(data.rows.into_iter().map(|row| {
            div()
                .class("flex items-center justify-between gap-4")
                .child(
                    div()
                        .class("flex items-center gap-2")
                        .child(div().class("size-[10px] rounded-full").bg(row.color))
                        .child(row.name),
                )
                .child(div().class("font-medium").child(row.value))
        }))
}

/// The card of a hovered annotation or exemplar.
pub(crate) fn mark_tip(hover: MarkHover, title: SharedString, rows: Vec<SharedString>) -> Div {
    let theme = crate::ui::theme();
    corner(card(), hover.at, hover.area)
        .class("max-w-72")
        .bg(theme.popover)
        .text_color(theme.popover_foreground)
        .border_color(theme.border)
        .child(div().class("font-semibold").child(title))
        .children(
            rows.into_iter()
                .filter(|row| !row.is_empty())
                .map(|line| div().text_color(theme.popover_muted).child(line)),
        )
}

/// A floating card over a chart (the pointer passes through it). Its id, `chart-tooltip`, finds it
/// (tests, automation).
pub(crate) fn card() -> Div {
    div().id("chart-tooltip").class("absolute flex flex-col gap-1 p-2 rounded-md border shadow-md text-xs pointer-events-none")
}
