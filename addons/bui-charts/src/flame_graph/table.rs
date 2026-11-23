use std::ops::Range;

use blenderui::element::{Div, ElementId, FluentBuilder, ParentElement};
use blenderui::html::div;
use blenderui::{App, ClickEvent, Hsla, SharedString, Styled, Window, px};

use super::model::{Column, Flame, Sort, change, percent, percent_text};
use crate::ChartTheme;
use crate::icon::{IconName, icon};
use crate::widgets::{ROW, Ui, rows, ui};

const VALUE_WIDTH: f32 = 120.;
const ACTIONS_WIDTH: f32 = 52.;

pub(crate) const fn toggled(sort: Sort, column: Column) -> Sort {
    if sort.column as u8 == column as u8 { Sort { column, desc: !sort.desc } } else { Sort { column, desc: true } }
}

pub(crate) fn header_cell(
    label: &'static str,
    column: Column,
    current: Sort,
    ui: Ui,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> Div {
    let arrow = (current.column == column)
        .then(|| icon(if current.desc { IconName::ArrowDown } else { IconName::ArrowUp }, ui.muted).size(12.));
    let first = column == Column::Symbol;
    div()
        .id(format!("sort-{}", column as usize))
        .class("flex items-center gap-1 cursor-pointer whitespace-nowrap")
        .map(|el| if first { el.flex_1().min_w_0() } else { el.w(px(VALUE_WIDTH)).justify_end().flex_none() })
        .text_color(ui.muted)
        .on_click(on_click)
        .child(label)
        .child(arrow)
}

pub(crate) fn diff_text(before: f64, after: f64, good: Hsla, bad: Hsla, ui: Ui) -> (SharedString, Hsla) {
    let delta = change(before, after);
    match delta {
        delta if delta.is_infinite() => ("new".into(), bad),
        delta if (delta + 100.).abs() < f64::EPSILON => ("removed".into(), good),
        delta if delta >= 0. => (percent_text(delta), bad),
        delta if delta < 0. => (percent_text(delta), good),
        _ => ("".into(), ui.muted),
    }
}

struct Line {
    label: u32,
    name: SharedString,
    cells: [(SharedString, Hsla); 3],
    searched: bool,
    sandwiched: bool,
}

/// The functions by own or total value, with search and sandwich buttons.
pub(crate) fn top_table(flame: Flame) -> Div {
    let ui = ui();
    let sort = flame.table_sort.get();
    let diff = flame.profile.with(super::profile::Profile::is_diff);
    let columns: &[(&str, Column)] = if diff {
        &[("Symbol", Column::Symbol), ("Baseline", Column::Baseline), ("Comparison", Column::Comparison), ("Diff", Column::Diff)]
    } else {
        &[("Symbol", Column::Symbol), ("Self", Column::Own), ("Total", Column::Total)]
    };
    let count = flame.cache.with(|cache| cache.table.len());
    let value_columns = columns.len() - 1;
    let list = rows("flame-table", count, move |range, _, _| {
        lines(flame, range, ui).into_iter().map(|line| row(line, value_columns, flame, ui)).collect()
    });
    div()
        .class("flex flex-col size-full min-h-0 text-xs p-2")
        .text_color(ui.text)
        .child(
            div()
                .class("flex items-center gap-2 h-6 pl-1 pr-4 border-b overflow-hidden flex-none")
                .border_color(ui.border)
                .child(div().class("flex-none").w(px(ACTIONS_WIDTH)))
                .children(columns.iter().copied().map(|(label, column)| {
                    header_cell(label, column, sort, ui, move |_, _, _| {
                        flame.table_sort.update(|sort| *sort = toggled(*sort, column));
                    })
                })),
        )
        .child(list)
}

fn lines(flame: Flame, range: Range<usize>, ui: Ui) -> Vec<Line> {
    let colors = ChartTheme::colors();
    let highlight = flame.highlight.get();
    let sandwich = flame.sandwich.get();
    flame.profile.with(|profile| {
        flame.cache.with(|cache| {
            let diff = profile.is_diff();
            let root = cache.main.tree.nodes.first().copied().unwrap_or_default();
            let (left, right) = (root.value - root.right, root.right);
            let end = range.end.min(cache.table.len());
            let start = range.start.min(end);
            cache.table[start..end]
                .iter()
                .map(|&label| {
                    let totals = &cache.totals[label as usize];
                    let cells = if diff {
                        let (before, after) = (percent(totals.total, left), percent(totals.total_right, right));
                        [
                            (percent_text(before), ui.text),
                            (percent_text(after), ui.text),
                            diff_text(before, after, colors.good, colors.bad, ui),
                        ]
                    } else {
                        [
                            (flame.units.text(totals.own), ui.text),
                            (flame.units.text(totals.total), ui.text),
                            ("".into(), ui.text),
                        ]
                    };
                    Line {
                        label,
                        name: profile.label(label).clone(),
                        cells,
                        searched: highlight == Some(label),
                        sandwiched: sandwich == Some(label),
                    }
                })
                .collect()
        })
    })
}

fn action(
    id: impl Into<ElementId>,
    name: IconName,
    active: bool,
    ui: Ui,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> Div {
    div()
        .id(id)
        .class("flex flex-none items-center justify-center size-6 rounded-md border cursor-pointer")
        .border_color(ui.border)
        .when(active, |el| el.bg(ui.muted.opacity(0.3)))
        .on_click(on_click)
        .child(icon(name, ui.text).size(12.))
}

fn row(line: Line, value_columns: usize, flame: Flame, ui: Ui) -> Div {
    let Line { label, name, cells, searched, sandwiched } = line;
    div()
        .class("flex items-center gap-2 pl-1 pr-4 w-full overflow-hidden")
        .h(px(ROW))
        .when(searched, |el| el.bg(ui.accent))
        .child(
            div()
                .class("flex gap-1 flex-none")
                .w(px(ACTIONS_WIDTH))
                .child(action(format!("search-{label}"), IconName::Search, searched, ui, move |_, _, _| {
                    toggle_search(flame, label);
                }))
                .child(action(format!("sandwich-{label}"), IconName::ChevronsUpDown, sandwiched, ui, move |_, _, _| {
                    flame.focus.set(None);
                    flame.sandwich.update(|current| *current = if *current == Some(label) { None } else { Some(label) });
                })),
        )
        .child(div().class("flex-1 min-w-0 truncate").child(name))
        .children(cells.into_iter().take(value_columns).map(|(text, color)| {
            div().class("flex-none text-right truncate").w(px(VALUE_WIDTH)).text_color(color).child(text)
        }))
}

pub(crate) fn toggle_search(flame: Flame, label: u32) {
    if flame.highlight.get() == Some(label) {
        flame.highlight.set(None);
        flame.search.set(SharedString::default());
        return;
    }
    let pattern: SharedString = flame.profile.with(|profile| format!("^{}$", regex::escape(profile.label(label)))).into();
    flame.highlight.set(Some(label));
    flame.search.set(pattern);
}
