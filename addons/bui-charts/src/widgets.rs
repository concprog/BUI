//! The small controls the charts' panels share (buttons, pills, scrolling row lists), in the colors of
//! the kit theme.

use std::ops::Range;

use blenderui::element::{Div, ElementId, FluentBuilder, IntoElement, ParentElement};
use blenderui::html::div;
use blenderui::{App, ClickEvent, Hsla, SharedString, Styled, Window, px, uniform_list};

use crate::color::{darker, lighter};
use crate::icon::{IconName, icon};

pub(crate) const ROW: f32 = 24.;

#[derive(Clone, Copy)]
pub(crate) struct Ui {
    pub text: Hsla,
    pub muted: Hsla,
    pub border: Hsla,
    pub accent: Hsla,
    pub popover: Hsla,
    pub popover_text: Hsla,
    pub mute_bar: Hsla,
}

impl Ui {
    pub(crate) fn of(theme: &crate::ui::Theme) -> Self {
        let base = theme.secondary;
        Self {
            text: theme.foreground,
            muted: theme.muted_foreground,
            border: theme.border,
            accent: theme.accent,
            popover: theme.popover,
            popover_text: theme.popover_foreground,
            mute_bar: if theme.dark { lighter(base, 0.1) } else { darker(base, 0.1) },
        }
    }
}

pub(crate) fn ui() -> Ui {
    Ui::of(&crate::ui::theme())
}

pub(crate) fn button(
    id: impl Into<ElementId>,
    content: impl IntoElement,
    active: bool,
    ui: Ui,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> Div {
    div()
        .id(id)
        .class("flex items-center gap-1 px-2 h-6 rounded-md border text-xs cursor-pointer whitespace-nowrap")
        .border_color(ui.border)
        .text_color(ui.text)
        .when(active, |el| el.bg(ui.accent))
        .on_click(on_click)
        .child(content)
}

/// A button showing an icon in the text color.
pub(crate) fn icon_button(
    id: impl Into<ElementId>,
    name: IconName,
    active: bool,
    ui: Ui,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> Div {
    button(id, icon(name, ui.text), active, ui, on_click)
}

pub(crate) fn pill(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    ui: Ui,
    on_close: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> Div {
    div()
        .class("flex items-center gap-1 pl-2 pr-1 h-6 rounded-full text-xs")
        .bg(ui.accent)
        .text_color(ui.text)
        .child(div().class("truncate max-w-[240px]").child(label.into()))
        .child(div().id(id).class("px-1 cursor-pointer").on_click(on_close).child(icon(IconName::Close, ui.muted).size(12.)))
}

/// A virtualized list of `count` rows, `ROW` tall, filling what is left of its column.
pub(crate) fn rows<R: IntoElement>(
    id: &'static str,
    count: usize,
    render: impl Fn(Range<usize>, &mut Window, &mut App) -> Vec<R> + 'static,
) -> Div {
    div()
        .class("relative flex flex-col flex-1 min-h-0")
        .child(uniform_list(id, count, render).item_height(px(ROW)).size_full())
}

/// A column that scrolls its content vertically, filling what is left of its parent.
pub(crate) fn scroll_area(id: &'static str, content: impl IntoElement) -> Div {
    div()
        .class("relative flex flex-col flex-1 min-h-0")
        .child(div().id(id).size_full().overflow_y_scroll().child(content))
}
