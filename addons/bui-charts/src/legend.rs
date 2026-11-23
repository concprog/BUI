use std::collections::BTreeSet;

use blenderui::element::{Div, FluentBuilder, ParentElement};
use blenderui::hooks::Signal;
use blenderui::html::div;
use blenderui::{ClickEvent, Hsla, Pixels, SharedString, Styled};

use crate::calc::{Calc, CalcSet};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LegendMode {
    #[default]
    List,
    Table,
    Hidden,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LegendPlacement {
    #[default]
    Bottom,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LegendSort {
    Name,
    Calc(Calc),
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Legend {
    pub(crate) mode: LegendMode,
    pub(crate) placement: LegendPlacement,
    pub(crate) calcs: CalcSet,
    pub(crate) sort_by: Option<LegendSort>,
    pub(crate) sort_desc: bool,
    pub(crate) width: Option<Pixels>,
    pub(crate) limit: Option<usize>,
    pub(crate) wrap: bool,
}

impl Legend {
    #[must_use]
    pub fn list() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn table() -> Self {
        Self {
            mode: LegendMode::Table,
            ..Self::default()
        }
    }

    #[must_use]
    pub fn hidden() -> Self {
        Self {
            mode: LegendMode::Hidden,
            ..Self::default()
        }
    }

    #[must_use]
    pub fn bottom(self) -> Self {
        self.placement(LegendPlacement::Bottom)
    }

    #[must_use]
    pub fn right(self) -> Self {
        self.placement(LegendPlacement::Right)
    }

    #[must_use]
    pub fn calcs(mut self, calcs: impl IntoIterator<Item = Calc>) -> Self {
        self.calcs = calcs.into_iter().collect();
        self
    }

    with! {
        mode: LegendMode,
        placement: LegendPlacement,
        sort_by: LegendSort,
        sort_desc: bool,
        width: Pixels,
        limit: usize,
        wrap: bool,
    }

    pub(crate) fn visible(&self) -> bool {
        self.mode != LegendMode::Hidden
    }

    pub(crate) fn on_right(&self) -> bool {
        self.placement == LegendPlacement::Right
    }
}

pub(crate) struct LegendItem {
    pub index: usize,
    pub name: SharedString,
    pub color: Hsla,
    pub disabled: bool,
    pub values: [Option<(&'static str, SharedString)>; Calc::COUNT],
}

fn toggle(hidden: &mut BTreeSet<usize>, count: usize, index: usize, append: bool) {
    if append {
        if !hidden.remove(&index) {
            hidden.insert(index);
        }
    } else if hidden.len() + 1 == count && !hidden.contains(&index) {
        hidden.clear();
    } else {
        hidden.clear();
        hidden.extend((0..count).filter(|other| *other != index));
    }
}

/// The legend beside or below a chart: a click shows one series alone (again: all of them), a
/// Ctrl+click shows or hides one.
pub(crate) fn legend_panel(
    legend: Legend,
    items: Vec<LegendItem>,
    hidden: Signal<BTreeSet<usize>>,
    count: usize,
) -> Div {
    let muted = crate::ui::theme().muted_foreground;
    let stacked = legend.on_right() || legend.mode == LegendMode::Table;
    let calcs = if legend.mode == LegendMode::Table { legend.calcs } else { CalcSet::default() };
    let inline = legend.mode == LegendMode::List;
    let wrap = legend.wrap;
    let width = legend.width.filter(|_| legend.on_right());
    let header = (!calcs.is_empty()).then(|| {
        div()
            .class("flex gap-2")
            .text_color(muted)
            .child(div().class("flex-1"))
            .children(calcs.iter().map(|calc| div().class("w-16 text-right").child(calc.label())))
    });
    div()
        .class("flex gap-x-4 gap-y-1 p-2 text-xs overflow-hidden")
        .map(|el| if stacked { el.flex_col() } else { el.flex_wrap() })
        .when_some(width, |el, width| el.w(width).flex_none())
        .child(header)
        .children(items.into_iter().map(move |item| {
            let index = item.index;
            div()
                .id(format!("legend-{index}"))
                .class("flex items-center gap-2 cursor-pointer")
                .when(item.disabled, |el| el.opacity(0.4))
                .on_click(move |event: &ClickEvent, _, _| {
                    let append = event.down.modifiers.control;
                    hidden.update(|hidden| toggle(hidden, count, index, append));
                })
                .child(div().class("w-[14px] h-[4px] rounded-full flex-none").bg(item.color))
                .child(
                    div()
                        // a list item is as wide as its name; a stacked one shares the row with values
                        .class(if stacked { "flex-1 min-w-0" } else { "flex-none whitespace-nowrap" })
                        .when(stacked && !wrap, |el| el.class("truncate"))
                        .child(item.name),
                )
                .children(item.values.into_iter().flatten().map(move |(calc, value)| {
                    div()
                        .class("flex justify-end gap-1")
                        .when(!inline, |el| el.class("w-16"))
                        .child(inline.then(|| div().text_color(muted).child(calc)))
                        .child(div().when(inline, |el| el.text_color(muted)).child(value))
                }))
        }))
}

pub(crate) struct Entry<'a> {
    pub index: usize,
    pub name: &'a SharedString,
    pub color: Hsla,
    pub hidden: bool,
}

impl Legend {
    pub(crate) fn items<'a>(
        self,
        entries: impl Iterator<Item = Entry<'a>>,
        calc: impl Fn(usize, Calc) -> Option<f64>,
        text: impl Fn(usize, f64) -> SharedString,
    ) -> Vec<LegendItem> {
        let mut items: Vec<(f64, LegendItem)> = entries
            .map(|entry| {
                let mut values: [Option<(&'static str, SharedString)>; Calc::COUNT] =
                    Default::default();
                for (slot, kind) in values.iter_mut().zip(self.calcs.iter()) {
                    let value = calc(entry.index, kind)
                        .map_or(SharedString::Borrowed("-"), |value| {
                            text(entry.index, value)
                        });
                    *slot = Some((kind.label(), value));
                }
                let key = match self.sort_by {
                    Some(LegendSort::Calc(kind)) => {
                        calc(entry.index, kind).unwrap_or(f64::NEG_INFINITY)
                    }
                    _ => 0.,
                };
                let item = LegendItem {
                    index: entry.index,
                    name: entry.name.clone(),
                    color: entry.color,
                    disabled: entry.hidden,
                    values,
                };
                (key, item)
            })
            .collect();
        match self.sort_by {
            Some(LegendSort::Name) => items.sort_by(|a, b| a.1.name.cmp(&b.1.name)),
            Some(LegendSort::Calc(_)) => items.sort_by(|a, b| a.0.total_cmp(&b.0)),
            None => {}
        }
        if self.sort_by.is_some() && self.sort_desc {
            items.reverse();
        }
        items.truncate(self.limit.unwrap_or(usize::MAX));
        items.into_iter().map(|(_, item)| item).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn click_isolates_and_restores() {
        let mut hidden = BTreeSet::new();
        toggle(&mut hidden, 3, 1, false);
        assert_eq!(hidden, BTreeSet::from([0, 2]));
        toggle(&mut hidden, 3, 1, false);
        assert!(hidden.is_empty());
    }

    #[test]
    fn modifier_click_toggles_one() {
        let mut hidden = BTreeSet::new();
        toggle(&mut hidden, 3, 2, true);
        assert_eq!(hidden, BTreeSet::from([2]));
        toggle(&mut hidden, 3, 2, true);
        assert!(hidden.is_empty());
    }
}
