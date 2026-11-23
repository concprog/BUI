use blenderui::element::{Div, FluentBuilder, ParentElement};
use blenderui::hooks::{Signal, use_signal};
use blenderui::html::{div, input, label, option, select};
use blenderui::{ChangeEvent, SharedString, Styled, Window};

use super::filter::{Filter, FilterKey, FilterOp, SpanFilters};
use super::tone::Tone;
use super::view::View;
use crate::icon::{IconName, icon};

/// The filter being written: its key, its operator (an index into the key's), its value.
#[derive(Clone, Copy)]
struct Draft {
    key: Signal<FilterKey>,
    op: Signal<usize>,
    value: Signal<String>,
}

fn add(view: View, draft: Draft) {
    let text = draft.value.with(|value| value.trim().to_string());
    if text.is_empty() {
        return;
    }
    let key = draft.key.get();
    let op = key.ops().get(draft.op.get()).copied().unwrap_or_else(|| key.ops()[0]);
    view.filters.update(|filters| filters.filters.push(Filter { key, op, value: text.into() }));
    draft.value.set(String::new());
    view.cursor.set(None);
}

fn toggle(filters: &mut SpanFilters, key: FilterKey, op: FilterOp, value: SharedString) {
    if filters.has(&key, op) {
        filters.filters.retain(|filter| !(filter.key == key && filter.op == op));
    } else {
        filters.filters.push(Filter { key, op, value });
    }
}

fn pill(id: &'static str, text: &'static str, active: bool, tone: Tone, on_click: impl Fn() + 'static) -> Div {
    div()
        .id(id)
        .class("px-2 h-6 flex items-center rounded-full border cursor-pointer whitespace-nowrap")
        .border_color(if active { tone.primary } else { tone.border })
        .when(active, move |el| el.bg(tone.primary.opacity(0.15)))
        .hover(move |style| style.bg(tone.accent))
        .on_click(move |_, _, _| on_click())
        .child(text)
}

/// Steps through the matching spans (wrapping around) and reveals the next one.
fn step(view: View, forward: bool, window: &mut Window) {
    let hits = view.cache.with(|cache| cache.hits.clone());
    if hits.is_empty() {
        return;
    }
    let last = hits.len() - 1;
    let next = match (view.cursor.get(), forward) {
        (None, true) => 0,
        (Some(index), true) if index >= last => 0,
        (Some(index), true) => index + 1,
        (None | Some(0), false) => last,
        (Some(index), false) => index - 1,
    };
    view.cursor.set(Some(next));
    view.reveal(hits[next], Some(window));
}

fn nav_button(id: &'static str, name: IconName, enabled: bool, tone: Tone, forward: bool, view: View) -> Div {
    div()
        .id(id)
        .class("flex items-center justify-center size-6 rounded-md border cursor-pointer")
        .border_color(tone.border)
        .when(!enabled, |el| el.opacity(0.5))
        .on_click(move |_, window, _| step(view, forward, window))
        .child(icon(name, tone.text))
}

/// The span filters: key, operator and value to add one, the active ones as chips, quick
/// toggles, and stepping through the matches.
#[allow(clippy::too_many_lines)]
pub(crate) fn filter_bar(view: View) -> Div {
    let tone = Tone::of();
    let draft = Draft { key: use_signal(FilterKey::Text), op: use_signal(0usize), value: use_signal(String::new()) };
    let key = draft.key.get();
    let keys = view.cache.with(|cache| cache.keys.clone());
    let (hits, p90) = view.cache.with(|cache| (cache.hits.len(), cache.p90));
    let total = view.trace.with(|trace| trace.spans.len());
    let filters = view.filters.get();
    let cursor = view.cursor.get();
    let any_filter = filters.active().next().is_some() || filters.critical_path_only;
    let summary: SharedString = match (any_filter, hits, cursor) {
        (false, _, _) => format!("{total} spans").into(),
        (true, 0, _) => "0 matches".into(),
        (true, count, Some(index)) => format!("{}/{count} matches", index + 1).into(),
        (true, count, None) => format!("{count} matches").into(),
    };
    let chips = filters.filters.iter().enumerate().map(|(index, filter)| {
        div()
            .class("flex items-center gap-1 pl-2 pr-1 h-6 rounded-full")
            .bg(tone.secondary)
            .child(
                div()
                    .class("truncate max-w-[280px]")
                    .child(format!("{} {} {}", filter.key.label(), filter.op.symbol(), filter.value)),
            )
            .child(
                div()
                    .id(format!("remove-filter-{index}"))
                    .class("cursor-pointer")
                    .on_click(move |_, _, _| {
                        view.filters.update(|filters| {
                            filters.filters.remove(index);
                        });
                        view.cursor.set(None);
                    })
                    .child(icon(IconName::Close, tone.muted).size(12.)),
            )
    });
    let errors = filters.has(&FilterKey::Tag("status".into()), FilterOp::Eq);
    let slow = filters.has(&FilterKey::Duration, FilterOp::Ge);
    let chosen_label = key.label();
    let key_options = keys.iter().map(|name| option().selected(*name == chosen_label).child(name.clone()));
    let op = draft.op.get();
    let op_options = key.ops().iter().enumerate().map(|(index, op_kind)| option().selected(index == op).child(op_kind.symbol()));
    let value = draft.value.get();

    div()
        .class("flex flex-col gap-2")
        .child(
            div()
                .class("flex flex-wrap items-center gap-2")
                .child(
                    select()
                        .id("filter-key")
                        .class("w-[200px]")
                        .on_change(move |event: &ChangeEvent, _, _| {
                            let picked = view.cache.with(|cache| cache.keys.get(event.number as usize).cloned());
                            if let Some(picked) = picked {
                                draft.key.set(FilterKey::parse(&picked));
                                draft.op.set(0);
                            }
                        })
                        .children(key_options),
                )
                .child(
                    select()
                        .id("filter-op")
                        .class("w-[80px]")
                        .on_change(move |event: &ChangeEvent, _, _| draft.op.set(event.number as usize))
                        .children(op_options),
                )
                .child(
                    input()
                        .id("filter-value")
                        .class("flex-1 min-w-[160px] h-7")
                        .placeholder("Filter by attribute or text")
                        .value(value)
                        .on_input(move |event: &ChangeEvent, _, _| draft.value.update_untracked(|value| value.clone_from(&event.value)))
                        .on_change(move |event: &ChangeEvent, _, _| {
                            draft.value.update_untracked(|value| value.clone_from(&event.value));
                            add(view, draft);
                        }),
                )
                .child(
                    div()
                        .id("add-filter")
                        .class("px-2 h-7 flex items-center gap-1 rounded-md border cursor-pointer")
                        .border_color(tone.border)
                        .hover(move |style| style.bg(tone.accent))
                        .on_click(move |_, _, _| add(view, draft))
                        .child(icon(IconName::Plus, tone.text).size(12.))
                        .child("Add"),
                ),
        )
        .child(
            div()
                .class("flex flex-wrap items-center gap-2")
                .children(chips)
                .child(pill("pill-critical", "Critical path", filters.critical_path_only, tone, move || {
                    view.filters.update(|filters| filters.critical_path_only = !filters.critical_path_only);
                    view.cursor.set(None);
                }))
                .child(pill("pill-errors", "Errors", errors, tone, move || {
                    view.filters.update(|filters| toggle(filters, FilterKey::Tag("status".into()), FilterOp::Eq, "error".into()));
                    view.cursor.set(None);
                }))
                .child(pill("pill-latency", "High latency", slow, tone, move || {
                    view.filters.update(|filters| {
                        if filters.has(&FilterKey::Duration, FilterOp::Ge) {
                            filters.filters.retain(|filter| filter.key != FilterKey::Duration);
                        } else {
                            filters.filters.push(Filter { key: FilterKey::Duration, op: FilterOp::Ge, value: format!("{p90}us").into() });
                        }
                    });
                    view.cursor.set(None);
                }))
                .child(div().class("flex-1"))
                .child(nav_button("previous-match", IconName::ChevronUp, hits > 0, tone, false, view))
                .child(nav_button("next-match", IconName::ChevronDown, hits > 0, tone, true, view))
                .child(div().class("whitespace-nowrap").text_color(tone.muted).child(summary))
                .child(
                    div()
                        .class("flex items-center gap-1")
                        .child(
                            input()
                                .id("show-all-spans")
                                .input_type("checkbox")
                                .checked(!filters.matches_only)
                                .disabled(hits == 0)
                                .on_change(move |event: &ChangeEvent, _, _| {
                                    let only = !event.checked;
                                    view.filters.update(|filters| filters.matches_only = only);
                                }),
                        )
                        .child(label().child("Show all spans")),
                ),
        )
}
