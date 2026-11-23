use blenderui::element::{AnyElement, Div, IntoElement, ParentElement};
use blenderui::html::div;
use blenderui::{SharedString, Styled, Window};

use super::data::Tag;
use super::format::{Rank, category, category_label, category_rank, duration, time_of_day};
use super::tone::Tone;
use super::trace::{Span, Trace};
use super::view::{Selected, ShareSlot, View};
use crate::icon::{IconName, icon};

const SPAN_ATTRIBUTES: u16 = 1;
const RESOURCE: u16 = 1 << 1;
const EVENTS: u16 = 1 << 2;
const WARNINGS: u16 = 1 << 3;
const STACK: u16 = 1 << 4;
const REFERENCES: u16 = 1 << 5;
const SUMMARY: u16 = 1 << 6;
const EXCEPTION: u16 = 1 << 7;
const DEFAULT_OPEN: u16 = SPAN_ATTRIBUTES | RESOURCE;
const WIDE: f32 = 1000.;

pub(crate) fn copy(text: SharedString, view: View, window: &mut Window) {
    window.write_clipboard(&text);
    view.copied.set(Some(text));
}

fn copy_button(id: String, text: SharedString, view: View, tone: Tone, copied: Option<&SharedString>) -> Div {
    let done = copied == Some(&text);
    div()
        .id(id)
        .class("flex items-center justify-center flex-none size-5 rounded cursor-pointer")
        .hover(move |style| style.bg(tone.accent))
        .on_click(move |_, window, _| copy(text.clone(), view, window))
        .child(icon(if done { IconName::Check } else { IconName::Copy }, if done { tone.success } else { tone.muted }).size(12.))
}

fn table(id: &'static str, tags: &[Tag], view: View, tone: Tone, copied: Option<&SharedString>) -> AnyElement {
    if tags.is_empty() {
        return div().class("py-1").text_color(tone.muted).child("No attributes").into_any_element();
    }
    div()
        .class("flex flex-col")
        .children(tags.iter().enumerate().map(|(index, tag)| {
            let value: SharedString = tag.value.to_string().into();
            div()
                .class("flex items-start gap-2 py-[2px] border-b")
                .border_color(tone.border)
                .child(div().class("w-1/3 flex-none truncate").text_color(tone.muted).child(tag.key.clone()))
                .child(div().class("flex-1 min-w-0").child(value.clone()))
                .child(copy_button(format!("{id}-{index}"), value, view, tone, copied))
        }))
        .into_any_element()
}

type Group = (Rank, Option<usize>, Vec<Tag>);

fn categorized(
    id: &'static str,
    tags: &[Tag],
    resource: bool,
    view: View,
    tone: Tone,
    copied: Option<&SharedString>,
) -> AnyElement {
    let mut groups: Vec<Group> = Vec::new();
    for tag in tags {
        let index = category(&tag.key);
        let rank = category_rank(index, resource);
        match groups.iter_mut().find(|(_, other, _)| *other == index) {
            Some((_, _, group)) => group.push(tag.clone()),
            None => groups.push((rank, index, vec![tag.clone()])),
        }
    }
    if groups.iter().all(|(_, index, _)| index.is_none()) {
        return table(id, tags, view, tone, copied);
    }
    groups.sort_by_key(|(rank, _, _)| *rank);
    div()
        .class("flex flex-col gap-2")
        .children(groups.into_iter().map(|(_, index, group)| {
            div()
                .class("flex flex-col gap-1")
                .child(
                    div()
                        .class("font-medium")
                        .text_color(tone.muted)
                        .child(format!("{} ({})", category_label(index), group.len())),
                )
                .child(table(id, &group, view, tone, copied))
        }))
        .into_any_element()
}

fn summary(tags: &[Tag]) -> SharedString {
    tags.iter().take(8).map(|tag| format!("{}={}", tag.key, tag.value)).collect::<Vec<_>>().join("  ").into()
}

/// A section of the detail pane that opens and closes (`bit` of the span's open sections).
struct Card {
    id: &'static str,
    title: SharedString,
    count: Option<usize>,
    bit: u16,
    preview: Option<SharedString>,
    body: AnyElement,
}

impl Card {
    fn render(self, open: bool, span: u32, view: View, tone: Tone) -> AnyElement {
        let bit = self.bit;
        div()
            .class("flex flex-col rounded-md border overflow-hidden min-w-0")
            .border_color(tone.border)
            .bg(tone.background)
            .child(
                div()
                    .id(self.id)
                    .class("flex items-center gap-2 px-2 py-1 cursor-pointer")
                    .hover(move |style| style.bg(tone.accent))
                    .on_click(move |_, _, _| view.open.update(|open| *open.entry(span).or_insert(0) ^= bit))
                    .child(icon(if open { IconName::ChevronDown } else { IconName::ChevronRight }, tone.text).size(12.))
                    .child(div().class("font-medium whitespace-nowrap").child(self.title))
                    .child(self.count.map(|count| {
                        div().class("px-1 rounded-sm").bg(tone.secondary).text_color(tone.muted).child(count.to_string())
                    }))
                    .child(
                        self.preview
                            .filter(|_| !open)
                            .map(|preview| div().class("truncate").text_color(tone.muted).child(preview)),
                    ),
            )
            .child(open.then(|| div().class("px-2 pb-2").child(self.body)))
            .into_any_element()
    }
}

struct Exception {
    kind: Option<SharedString>,
    message: Option<SharedString>,
    stack: Option<SharedString>,
}

fn exception(span: &Span) -> Option<Exception> {
    let value = |tags: &[Tag], keys: &[&str]| {
        tags.iter().find(|tag| keys.contains(&tag.key.as_ref())).map(|tag| SharedString::from(tag.value.to_string()))
    };
    let from = |tags: &[Tag]| Exception {
        kind: value(tags, &["exception.type", "exception._type"]),
        message: value(tags, &["exception.message"]),
        stack: value(tags, &["exception.stacktrace", "exception.stack_trace"]),
    };
    let logged = span
        .row
        .logs
        .iter()
        .find(|log| {
            log.name.as_deref() == Some("exception") || log.fields.iter().any(|field| field.key.starts_with("exception."))
        })
        .map(|log| from(&log.fields));
    let mut found = logged.unwrap_or_else(|| from(&span.row.tags));
    if found.stack.is_none() {
        found.stack = span.row.stack_traces.first().cloned();
    }
    (found.kind.is_some() || found.message.is_some() || found.stack.is_some()).then_some(found)
}

fn fields(trace: &Trace, span: &Span) -> Vec<(&'static str, SharedString)> {
    let row = &span.row;
    let relative = row.start.saturating_sub(trace.start) as f64;
    let mut fields: Vec<(&'static str, SharedString)> = vec![
        ("Service", trace.service(span).clone()),
        ("Duration", duration(row.duration as f64)),
        ("Start Time", format!("{} ({})", duration(relative), time_of_day(row.start)).into()),
    ];
    if let Some(aggregation) = span.aggregation.as_ref().filter(|aggregation| aggregation.summary) {
        fields.push(("End Time", duration(relative + row.duration as f64)));
        if let (Some(min), Some(max)) = (aggregation.min, aggregation.max) {
            let median = aggregation
                .median
                .map(|median| format!("{} (median) | ", duration(median / 1000.)))
                .unwrap_or_default();
            fields[1].1 = format!("{} (min) | {median}{} (max)", duration(min / 1000.), duration(max / 1000.)).into();
        }
    }
    if span.children > 0 {
        fields.push(("Child Count", span.children.to_string().into()));
    }
    if let Some(kind) = row.kind {
        fields.push(("Kind", kind.label().into()));
    }
    if row.status != super::data::SpanStatus::Unset {
        fields.push(("Status", row.status.label().into()));
    }
    let optional: [(&'static str, Option<&SharedString>); 4] = [
        ("Status Message", row.status_message.as_ref()),
        ("Library Name", row.library.as_ref().map(|(name, _)| name)),
        ("Library Version", row.library.as_ref().and_then(|(_, version)| version.as_ref())),
        ("Trace State", row.trace_state.as_ref()),
    ];
    fields.extend(optional.into_iter().filter_map(|(label, value)| Some((label, value?.clone()))));
    fields
}

fn chip_button(id: String, tone: Tone) -> Div {
    div()
        .id(id)
        .class("flex items-center gap-1 px-2 h-6 rounded-md border cursor-pointer")
        .border_color(tone.border)
        .hover(move |style| style.bg(tone.accent))
}

/// The detail pane under an open span: its fields, attributes, events, links and actions.
#[allow(clippy::too_many_lines)]
pub(crate) fn span_detail(view: View, span: u32) -> Div {
    let tone = Tone::of();
    let wide = view.area.get().size.width >= WIDE;
    let bits = DEFAULT_OPEN ^ view.open.with(|open| open.get(&span).copied().unwrap_or(0));
    let is_open = |bit: u16| bits & bit != 0;
    let copied = view.copied.get();
    let copied = copied.as_ref();
    let actions = view.options.actions;
    view.trace.with(|trace| {
        let entry = &trace.spans[span as usize];
        let row = &entry.row;
        let summary_span = entry.aggregation.as_ref().is_some_and(|aggregation| aggregation.summary);
        let span_id = row.span_id.clone();
        let trace_id = trace.id.clone();
        let share = ShareSlot::link(view.options.share, &trace_id, &span_id);
        let span_tags: Vec<Tag> = row.tags.iter().filter(|tag| !tag.key.starts_with("aggregation.")).cloned().collect();
        let aggregation_tags: Vec<Tag> =
            row.tags.iter().filter(|tag| tag.key.starts_with("aggregation.")).cloned().collect();
        let mut cards = vec![
            Card {
                id: "span-attributes",
                title: "Span attributes".into(),
                count: Some(span_tags.len()),
                bit: SPAN_ATTRIBUTES,
                preview: Some(summary(&span_tags)),
                body: categorized("span-tag", &span_tags, false, view, tone, copied),
            },
            Card {
                id: "resource-attributes",
                title: if summary_span {
                    "Resource attributes (inherited from slowest span)".into()
                } else {
                    "Resource attributes".into()
                },
                count: Some(row.service_tags.len()),
                bit: RESOURCE,
                preview: Some(summary(&row.service_tags)),
                body: categorized("resource-tag", &row.service_tags, true, view, tone, copied),
            },
        ];
        if summary_span {
            cards.insert(
                0,
                Card {
                    id: "summary-attributes",
                    title: "Summary attributes".into(),
                    count: Some(aggregation_tags.len()),
                    bit: SUMMARY,
                    preview: Some(summary(&aggregation_tags)),
                    body: table("summary-tag", &aggregation_tags, view, tone, copied),
                },
            );
        }
        if !row.logs.is_empty() {
            let mut logs: Vec<&super::data::SpanLog> = row.logs.iter().collect();
            logs.sort_by_key(|log| log.timestamp);
            let events = logs.iter().map(|log| {
                let at = duration(log.timestamp.saturating_sub(trace.start) as f64);
                let title = match &log.name {
                    Some(name) if name.chars().count() > 20 => {
                        format!("{at} ({}...)", name.chars().take(20).collect::<String>())
                    }
                    Some(name) => format!("{at} ({name})"),
                    None => at.to_string(),
                };
                div()
                    .class("flex flex-col gap-1")
                    .child(div().class("font-medium").child(title))
                    .child(table("event-field", &log.fields, view, tone, copied))
            });
            cards.push(Card {
                id: "events",
                title: "Events".into(),
                count: Some(logs.len()),
                bit: EVENTS,
                preview: None,
                body: div()
                    .class("flex flex-col gap-2")
                    .children(events)
                    .child(
                        div()
                            .text_color(tone.muted)
                            .child("Event timestamps are relative to the start time of the full trace."),
                    )
                    .into_any_element(),
            });
        }
        if !row.warnings.is_empty() {
            cards.push(Card {
                id: "warnings",
                title: "Warnings".into(),
                count: Some(row.warnings.len()),
                bit: WARNINGS,
                preview: None,
                body: div()
                    .class("flex flex-col gap-1")
                    .children(row.warnings.iter().map(|warning| div().child(warning.clone())))
                    .into_any_element(),
            });
        }
        if !row.stack_traces.is_empty() {
            cards.push(Card {
                id: "stack-traces",
                title: "Stack trace".into(),
                count: None,
                bit: STACK,
                preview: None,
                body: div()
                    .class("flex flex-col gap-2 font-mono")
                    .children(row.stack_traces.iter().map(|stack| div().child(stack.clone())))
                    .into_any_element(),
            });
        }
        if row.references.len() > 1 || (row.references.len() == 1 && entry.follows) {
            let links = row.references.iter().enumerate().map(|(index, reference)| {
                let target = (reference.trace_id == trace.id || reference.trace_id.is_empty())
                    .then(|| trace.index.get(&reference.span_id).copied())
                    .flatten();
                let title: SharedString = target.map_or_else(
                    || SharedString::Borrowed("View Linked Span"),
                    |target| {
                        let other = &trace.spans[target as usize];
                        format!("{} {}", trace.service(other), other.row.operation).into()
                    },
                );
                div()
                    .class("flex flex-col gap-1")
                    .child(
                        div()
                            .id(format!("reference-{index}"))
                            .class("font-medium cursor-pointer underline")
                            .on_click(move |_, window, _| {
                                if let Some(target) = target {
                                    view.go_to(target, Some(window));
                                }
                            })
                            .child(title),
                    )
                    .child(div().text_color(tone.muted).child(format!("TraceID: {}", reference.trace_id)))
                    .child(div().text_color(tone.muted).child(format!("SpanID: {}", reference.span_id)))
                    .child(table("reference-tag", &reference.tags, view, tone, copied))
            });
            cards.push(Card {
                id: "references",
                title: "References".into(),
                count: Some(row.references.len()),
                bit: REFERENCES,
                preview: None,
                body: div().class("flex flex-col gap-2").children(links).into_any_element(),
            });
        }
        let exception_box = exception(entry).map(|exception| {
            let headline: SharedString = match (exception.kind, exception.message) {
                (Some(kind), Some(message)) => format!("{kind}: {message}").into(),
                (Some(kind), None) => kind,
                (None, Some(message)) => message,
                (None, None) => "Exception".into(),
            };
            let open = is_open(EXCEPTION);
            div()
                .class("flex flex-col gap-1 p-2 rounded-md border")
                .border_color(tone.danger.opacity(0.5))
                .bg(tone.danger.opacity(0.06))
                .child(
                    div()
                        .class("flex items-center gap-2 font-medium")
                        .text_color(tone.danger)
                        .child(icon(IconName::CircleAlert, tone.danger).size(12.))
                        .child(headline),
                )
                .child(exception.stack.map(|stack| {
                    div()
                        .class("flex flex-col gap-1")
                        .child(
                            div()
                                .id("exception-stack")
                                .class("flex items-center gap-1 cursor-pointer")
                                .text_color(tone.muted)
                                .on_click(move |_, _, _| {
                                    view.open.update(|open| *open.entry(span).or_insert(0) ^= EXCEPTION);
                                })
                                .child(
                                    icon(if open { IconName::ChevronDown } else { IconName::ChevronRight }, tone.muted)
                                        .size(12.),
                                )
                                .child("Stacktrace"),
                        )
                        .child(open.then(|| div().class("font-mono").child(stack)))
                }))
        });
        let (mut left, mut right): (Vec<AnyElement>, Vec<AnyElement>) = (Vec::new(), Vec::new());
        for (index, card) in cards.into_iter().enumerate() {
            let open = is_open(card.bit);
            let card = card.render(open, span, view, tone);
            if wide && index % 2 == 1 {
                right.push(card);
            } else {
                left.push(card);
            }
        }
        let selected_service = trace.service(entry).clone();
        let (start, length) = (row.start, row.duration);
        let action_buttons = actions.iter().enumerate().map(|(index, action)| {
            let (trace_id, span_id, service, operation) =
                (trace_id.clone(), span_id.clone(), selected_service.clone(), row.operation.clone());
            chip_button(format!("span-action-{index}"), tone)
                .on_click(move |_, window, cx| {
                    let selected = Selected {
                        trace_id: &trace_id,
                        span_id: &span_id,
                        service: &service,
                        operation: &operation,
                        start,
                        duration: length,
                    };
                    (action.run)(&selected, window, cx);
                })
                .child(action.label)
        });
        let footer_id = span_id.clone();
        div()
            .id(format!("span-detail-{span}"))
            .class("flex flex-col flex-none gap-2 p-3 w-full border-y text-xs")
            .bg(tone.secondary)
            .border_color(tone.border)
            .text_color(tone.text)
            .child(
                div()
                    .class("flex items-center gap-2")
                    .child(div().class("text-sm font-semibold truncate").child(row.operation.clone()))
                    .child(summary_span.then(|| div().text_color(tone.muted).child("(summary)")))
                    .child(div().class("flex-1"))
                    .children(action_buttons)
                    .child(share.map(|link| {
                        chip_button("share-span".into(), tone)
                            .on_click(move |_, window, _| copy(link.clone(), view, window))
                            .child(icon(IconName::ExternalLink, tone.text).size(12.))
                            .child("Share")
                    })),
            )
            .child(div().class("flex flex-wrap gap-x-4 gap-y-1").children(fields(trace, entry).into_iter().map(
                |(label, value)| {
                    div()
                        .class("flex gap-1")
                        .child(div().text_color(tone.muted).child(format!("{label}:")))
                        .child(div().class("font-medium").child(value))
                },
            )))
            .child(exception_box)
            .child(
                div()
                    .class("flex gap-2 w-full")
                    .child(div().class("flex flex-col gap-2 flex-1 min-w-0").children(left))
                    .child(wide.then(|| div().class("flex flex-col gap-2 flex-1 min-w-0").children(right))),
            )
            .child(
                div()
                    .class("flex items-center justify-end gap-1")
                    .text_color(tone.muted)
                    .child(icon(IconName::ExternalLink, tone.muted).size(12.))
                    .child(
                        div()
                            .id("footer-span-id")
                            .class("cursor-pointer")
                            .on_click(move |_, window, _| copy(footer_id.clone(), view, window))
                            .child(format!("SpanID: {span_id}")),
                    ),
            )
    })
}
