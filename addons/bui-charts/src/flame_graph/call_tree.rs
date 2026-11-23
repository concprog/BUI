use std::ops::Range;

use blenderui::element::{Div, FluentBuilder, ParentElement};
use blenderui::html::div;
use blenderui::{Hsla, SharedString, Styled, Window, px};

use super::header::search_field;
use super::model::{CallState, Column, Flame, MAX_MATCHES, Root, percent, percent_text};
use super::table::{diff_text, header_cell, toggled};
use super::tree::NONE;
use crate::ChartTheme;
use crate::icon::{IconName, icon};
use crate::widgets::{ROW, Ui, icon_button, pill, rows, ui};

const INDENT: f32 = 14.;
const VALUE_WIDTH: f32 = 110.;
const BAR_WIDTH: f32 = 160.;
const COMPACT_WIDTH: f32 = 1080.;
/// The call tree's row list (its scroll offset is kept by this id).
const LIST: &str = "flame-calls";

struct Line {
    node: u32,
    depth: u32,
    label: u32,
    name: SharedString,
    expandable: bool,
    expanded: bool,
    share: f32,
    color: Hsla,
    cells: [(SharedString, Hsla); 3],
    selected: bool,
}

fn reroot(flame: Flame, root: Root, window: &mut Window) {
    flame.calls.update(|calls| *calls = CallState { root, ..CallState { sort: calls.sort, ..CallState::default() } });
    window.set_scroll_offset(LIST, 0., 0.);
}

/// Opens the call tree down to the current match and scrolls it into the middle of the list.
fn reveal(flame: Flame) {
    let index = flame.call_match.get();
    let Some(node) = flame.cache.with(|cache| cache.call_matches.get(index).copied()) else { return };
    let root = flame.calls.with(|calls| calls.root);
    let mut parents = Vec::new();
    flame.cache.with(|cache| {
        let tree = &cache.call_tree(root).tree;
        let mut current = node;
        loop {
            let parent = tree.node(current).parent;
            if parent == NONE {
                break;
            }
            parents.push(parent);
            current = parent;
        }
    });
    flame.calls.update(|calls| calls.expanded.extend(parents));
    flame.scroll_to.set(Some(node));
}

fn step(flame: Flame, forward: bool) {
    let count = flame.cache.with(|cache| cache.call_matches.len());
    if count == 0 {
        return;
    }
    flame.call_match.update(|index| *index = if forward { (*index + 1) % count } else { (*index + count - 1) % count });
    reveal(flame);
}

/// The call tree: functions under their callers, searchable, rerooted at callers or callees.
pub(crate) fn call_tree_pane(flame: Flame) -> Div {
    let ui = ui();
    if flame.call_reveal.get() {
        flame.call_reveal.update_untracked(|pending| *pending = false);
        reveal(flame);
    }
    let sort = flame.calls.with(|calls| calls.sort);
    let diff = flame.profile.with(super::profile::Profile::is_diff);
    let wide = flame.width.get() >= COMPACT_WIDTH;
    let columns: &[(&str, Column)] = if diff {
        &[
            ("Function", Column::Symbol),
            ("Baseline", Column::Baseline),
            ("Comparison", Column::Comparison),
            ("Diff %", Column::Diff),
        ]
    } else {
        &[("Function", Column::Symbol), ("Self", Column::Own), ("Total", Column::Total)]
    };
    let (count, matches, invalid, scroll_row) = flame.cache.with(|cache| {
        let row = flame.scroll_to.get().and_then(|node| cache.call_rows.iter().position(|(row, _)| *row == node));
        (cache.call_rows.len(), cache.call_matches.len(), cache.call_invalid, row)
    });
    let root_label = match flame.calls.with(|calls| calls.root) {
        Root::All => None,
        Root::Callers(_) => Some("Clear callers view"),
        Root::Callees(_) => Some("Clear callees view"),
    };
    let query = flame.call_query.get();
    let status: Option<SharedString> = match (query.is_empty(), invalid, matches) {
        (true, _, _) => None,
        (false, true, _) => Some("Invalid regex pattern".into()),
        (false, false, 0) => Some("No matches found".into()),
        (false, false, count) => {
            let plus = if count >= MAX_MATCHES { "+" } else { "" };
            Some(format!("{} of {count}{plus}", flame.call_match.get() + 1).into())
        }
    };
    let clear_root =
        root_label.map(|label| pill("clear-root", label, ui, move |_, window, _| reroot(flame, Root::All, window)));
    let navigation = (matches > 0).then(|| {
        div()
            .class("flex gap-1")
            .child(icon_button("previous-match", IconName::ChevronUp, false, ui, move |_, _, _| {
                step(flame, false);
            }))
            .child(icon_button("next-match", IconName::ChevronDown, false, ui, move |_, _, _| {
                step(flame, true);
            }))
    });
    let value_columns = columns.len() - 1;
    let list = rows(LIST, count, move |range, window, _| {
        // a match revealed since the last frame: centre its row
        if let Some(index) = scroll_row {
            let height = window.bounds_of(LIST).map_or(0., |bounds| bounds.size.height);
            window.set_scroll_offset(LIST, 0., (index as f32 * ROW - height / 2.).max(0.));
            flame.scroll_to.update_untracked(|target| *target = None);
        }
        lines(flame, range, ui).into_iter().map(|line| row(line, value_columns, wide, flame, ui)).collect()
    });
    let field = search_field("flame-call-search", "Search functions…", query, move |value| {
        flame.call_query.set(value);
        flame.call_match.set(0);
        flame.call_reveal.set(true);
    })
    .class("flex-1");

    div()
        .class("flex flex-col size-full min-h-0 text-xs p-2 gap-2")
        .text_color(ui.text)
        .child(div().class("flex items-center gap-2").child(field).child(status).child(navigation).child(clear_root))
        .child(
            div()
                .class("flex items-center gap-2 h-6 pl-1 pr-4 border-b overflow-hidden flex-none")
                .border_color(ui.border)
                .children(columns.iter().copied().map(|(label, column)| {
                    header_cell(label, column, sort, ui, move |_, _, _| {
                        flame.calls.update(|calls| calls.sort = toggled(calls.sort, column));
                    })
                }))
                .child(wide.then(|| div().class("flex-none").w(px(BAR_WIDTH))))
                .child(div().class("flex-none w-[52px]")),
        )
        .child(list)
}

fn lines(flame: Flame, range: Range<usize>, ui: Ui) -> Vec<Line> {
    let colors = ChartTheme::colors();
    let selected_match = flame.call_match.get();
    flame.calls.with(|calls| {
        flame.profile.with(|profile| {
            flame.cache.with(|cache| {
                let diff = profile.is_diff();
                let tree = &cache.call_tree(calls.root).tree;
                let total = tree.total();
                let main = cache.main.tree.nodes.first().copied().unwrap_or_default();
                let (left, right) = (main.value - main.right, main.right);
                let selected = cache.call_matches.get(selected_match).copied();
                let end = range.end.min(cache.call_rows.len());
                let start = range.start.min(end);
                cache.call_rows[start..end]
                    .iter()
                    .map(|&(node, depth)| {
                        let entry = *tree.node(node);
                        let (own, _) = tree.self_values(node, profile);
                        let cells = if diff {
                            let (before, after) = (percent(entry.value - entry.right, left), percent(entry.right, right));
                            [
                                (percent_text(before), ui.text),
                                (percent_text(after), ui.text),
                                diff_text(before, after, colors.good, colors.bad, ui),
                            ]
                        } else {
                            let units = flame.units;
                            [
                                (format!("{} ({})", units.text(own), percent_text(percent(own, total))).into(), ui.text),
                                (
                                    format!("{} ({})", units.text(entry.value), percent_text(percent(entry.value, total)))
                                        .into(),
                                    ui.text,
                                ),
                                ("".into(), ui.text),
                            ]
                        };
                        Line {
                            node,
                            depth,
                            label: entry.label,
                            name: profile.label(entry.label).clone(),
                            expandable: entry.end > node + 1,
                            expanded: calls.expanded.contains(&node),
                            share: (entry.value / total) as f32,
                            color: cache.package[entry.label as usize],
                            cells,
                            selected: selected == Some(node),
                        }
                    })
                    .collect()
            })
        })
    })
}

fn row(line: Line, value_columns: usize, wide: bool, flame: Flame, ui: Ui) -> Div {
    let Line { node, depth, label, name, expandable, expanded, share, color, cells, selected } = line;
    let chevron =
        expandable.then(|| icon(if expanded { IconName::ChevronDown } else { IconName::ChevronRight }, ui.muted).size(12.));
    let bar = wide.then(|| {
        div()
            .class("flex-none h-2 rounded-full overflow-hidden")
            .w(px(BAR_WIDTH))
            .bg(ui.border)
            .child(div().class("h-full").w(px(BAR_WIDTH * share.clamp(0., 1.))).bg(color))
    });
    div()
        .class("flex items-center gap-2 pl-1 pr-4 w-full overflow-hidden")
        .h(px(ROW))
        .when(selected, |el| el.bg(ui.accent))
        .child(
            div()
                .class("flex items-center flex-1 min-w-0 gap-1")
                .pl(px(depth as f32 * INDENT))
                .child(
                    div()
                        .id(format!("expand-{node}"))
                        .class("flex items-center w-4 flex-none cursor-pointer")
                        .on_click(move |_, _, _| {
                            flame.calls.update(|calls| {
                                if !calls.expanded.remove(&node) {
                                    calls.expanded.insert(node);
                                }
                            });
                        })
                        .child(chevron),
                )
                .child(div().class("truncate").child(name)),
        )
        .children(cells.into_iter().take(value_columns).map(|(text, color)| {
            div().class("flex-none text-right truncate").w(px(VALUE_WIDTH)).text_color(color).child(text)
        }))
        .child(bar)
        .child(
            div()
                .class("flex gap-1 flex-none w-[52px]")
                .child(icon_button(format!("callers-{node}"), IconName::ArrowUp, false, ui, move |_, window, _| {
                    reroot(flame, Root::Callers(label), window);
                }))
                .child(icon_button(format!("callees-{node}"), IconName::ArrowDown, false, ui, move |_, window, _| {
                    reroot(flame, Root::Callees(label), window);
                })),
        )
}
