use blenderui::element::{Div, ParentElement};
use blenderui::html::{div, input};
use blenderui::{ChangeEvent, SharedString, Styled};

use super::model::Flame;
use super::options::{LabelAlign, Pane, PaneLayout};
use crate::icon::{IconName, icon};
use crate::widgets::{Ui, button, ui};

pub(crate) const SPLIT_WIDTH: f32 = 800.;

fn reset(flame: Flame) {
    flame.search.set(SharedString::default());
    flame.highlight.set(None);
    flame.focus.set(None);
    flame.sandwich.set(None);
}

const fn with_pane(view: PaneLayout, slot: usize, pane: Pane) -> PaneLayout {
    match (view, slot) {
        (PaneLayout::Single(_), _) => PaneLayout::Single(pane),
        (PaneLayout::Split(_, right), 0) => PaneLayout::Split(pane, right),
        (PaneLayout::Split(left, _), _) => PaneLayout::Split(left, pane),
    }
}

/// A search field bound to `value`: `on_change` hears every edit.
pub(crate) fn search_field(
    id: &'static str,
    placeholder: &'static str,
    value: SharedString,
    on_change: impl Fn(SharedString) + 'static,
) -> Div {
    input()
        .id(id)
        .attr("placeholder", placeholder)
        .value(value)
        .class("h-7 px-2 rounded-md text-xs")
        .on_input(move |event: &ChangeEvent, _, _| on_change(event.value.clone().into()))
}

/// The search field, the reset, the pane layout and the label alignment above a flame graph.
pub(crate) fn flame_header(flame: Flame) -> Div {
    let ui = ui();
    let search = flame.search.get();
    let view = flame.layout.get();
    let align = flame.align.get();
    let can_split = flame.vertical || flame.width.get() >= SPLIT_WIDTH;
    let resettable = !search.is_empty() || flame.focus.get().is_some() || flame.sandwich.get().is_some();
    let (primary, secondary) = view.panes();
    let modes = can_split.then(|| {
        div()
            .class("flex gap-1")
            .child(button("single", "Single", secondary.is_none(), ui, move |_, _, _| {
                flame.layout.set(PaneLayout::Single(primary));
            }))
            .child(button("split", "Split", secondary.is_some(), ui, move |_, _, _| {
                let other = if primary == Pane::FlameGraph { Pane::TopTable } else { Pane::FlameGraph };
                flame.layout.set(PaneLayout::Split(primary, secondary.unwrap_or(other)));
            }))
    });
    let slots: [Option<Pane>; 2] = [Some(primary), secondary.filter(|_| can_split)];
    let swap = secondary.filter(|_| can_split).map(|right| {
        let arrows = div().class("flex").child(icon(IconName::ArrowLeft, ui.text)).child(icon(IconName::ArrowRight, ui.text));
        button("swap", arrows, false, ui, move |_, _, _| flame.layout.set(PaneLayout::Split(right, primary)))
    });
    let reset_button = resettable.then(|| button("reset", "Reset", false, ui, move |_, _, _| reset(flame)));
    let clear = (!search.is_empty())
        .then(|| button("clear", "Clear", false, ui, move |_, _, _| flame.search.set(SharedString::default())));
    let field = search_field("flame-search", "Search…", search, move |value| {
        flame.highlight.set(None);
        flame.search.set(value);
    })
    .class("w-[240px]");

    div()
        .class("flex flex-wrap items-center gap-2 p-2 text-xs border-b")
        .border_color(ui.border)
        .text_color(ui.text)
        .child(field)
        .child(clear)
        .child(reset_button)
        .child(div().class("flex-1"))
        .child(modes)
        .children(slots.into_iter().enumerate().filter_map(|(slot, pane)| pane_picker(slot, pane, flame, ui)))
        .child(swap)
        .child(
            div()
                .class("flex gap-1")
                .child(button("align-left", "Left", align == LabelAlign::Left, ui, move |_, _, _| {
                    flame.align.set(LabelAlign::Left);
                }))
                .child(button("align-right", "Right", align == LabelAlign::Right, ui, move |_, _, _| {
                    flame.align.set(LabelAlign::Right);
                })),
        )
}

fn pane_picker(slot: usize, pane: Option<Pane>, flame: Flame, ui: Ui) -> Option<Div> {
    let current = pane?;
    Some(div().class("flex gap-1").children(Pane::ALL.into_iter().map(move |choice| {
        button(format!("pane-{}", slot * 3 + choice as usize), choice.label(), choice == current, ui, move |_, _, _| {
            flame.layout.update(|view| *view = with_pane(*view, slot, choice));
        })
    })))
}
