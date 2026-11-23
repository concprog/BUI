use blenderui::element::{Div, FluentBuilder, ParentElement};
use blenderui::html::div;
use blenderui::Styled;

use super::layout::Hit;
use super::model::TooltipData;
use crate::plot::{Tooltip, corner};
use crate::trend::card;

/// The card of the hovered point.
pub(crate) fn tooltip_card(tooltip: Tooltip, hit: Hit, data: TooltipData) -> Div {
    let theme = crate::ui::theme();
    corner(card(), hit.at, hit.area)
        .class("overflow-hidden")
        .bg(theme.popover)
        .text_color(theme.popover_foreground)
        .border_color(theme.border)
        .when_some(tooltip.max_width, Styled::max_w)
        .when_some(tooltip.max_height, Styled::max_h)
        .child(
            div()
                .class("flex items-center gap-2 font-semibold")
                .child(div().class("size-[10px] rounded-full").bg(data.color))
                .child(data.title),
        )
        .children(data.rows.into_iter().flatten().map(|(label, value)| {
            div()
                .class("flex justify-between gap-4")
                .child(div().text_color(theme.muted_foreground).child(label))
                .child(div().class("font-medium").child(value))
        }))
}
