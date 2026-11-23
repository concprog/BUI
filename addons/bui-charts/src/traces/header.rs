use blenderui::element::{Div, ParentElement};
use blenderui::hooks::{Signal, use_signal};
use blenderui::html::div;
use blenderui::{Hsla, SharedString, Styled, Window};

use super::detail::copy;
use super::format::{ago, clock, duration};
use super::minimap::minimap;
use super::search::filter_bar;
use super::tone::Tone;
use super::trace::{Span, Trace};
use super::view::{ShareSlot, View};
use crate::icon::{IconName, icon};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Severity {
    Error,
    Warning,
}

fn status_code(span: &Span) -> Option<u16> {
    ["http.response.status_code", "http.status_code"]
        .iter()
        .find_map(|key| span.tag(key))
        .and_then(|tag| tag.value.number())
        .map(|code| code as u16)
}

fn severity(span: &Span) -> Option<Severity> {
    let code = status_code(span);
    if span.error || code.is_some_and(|code| code >= 500) {
        Some(Severity::Error)
    } else if code.is_some_and(|code| (400..500).contains(&code)) {
        Some(Severity::Warning)
    } else {
        None
    }
}

fn banner(trace: &Trace) -> Option<(u32, Severity)> {
    let best = |wanted: Severity| {
        trace
            .spans
            .iter()
            .enumerate()
            .filter(|(_, span)| severity(span) == Some(wanted))
            .max_by(|(_, a), (_, b)| {
                (a.depth, a.row.duration)
                    .cmp(&(b.depth, b.row.duration))
                    .then(b.row.start.cmp(&a.row.start))
            })
            .map(|(index, _)| (index as u32, wanted))
    };
    best(Severity::Error).or_else(|| best(Severity::Warning))
}

fn path(span: &Span) -> SharedString {
    let method = ["http.request.method", "http.method"]
        .iter()
        .find_map(|key| span.tag(key))
        .map(|tag| tag.value.to_string());
    let target = [
        "http.route",
        "http.target",
        "url.path",
        "http.path",
        "http.url",
        "url.full",
    ]
    .iter()
    .find_map(|key| span.tag(key))
    .map(|tag| {
        let text = tag.value.to_string();
        text.split_once("://")
            .and_then(|(_, rest)| rest.find('/').map(|at| rest[at..].to_string()))
            .unwrap_or(text)
    });
    match (method, target) {
        (Some(method), Some(target)) => format!("{method} {target}").into(),
        _ => span.row.operation.clone(),
    }
}

fn percent(part: u64, total: u64) -> SharedString {
    let share = (part as f64 / total.max(1) as f64 * 1000.).round() / 10.;
    if share == 0. && part > 0 {
        "<0.1".into()
    } else {
        format!("{}", share.min(100.)).into()
    }
}

fn badge(text: SharedString, color: Hsla) -> Div {
    div().class("px-2 py-[1px] rounded-sm text-xs font-medium").bg(color.opacity(0.18)).text_color(color).child(text)
}

fn export(trace: &Trace) -> String {
    let spans: Vec<serde_json::Value> = trace
        .spans
        .iter()
        .map(|span| {
            let row = &span.row;
            let tags: Vec<serde_json::Value> = row
                .tags
                .iter()
                .map(|tag| serde_json::json!({ "key": tag.key.as_ref(), "value": tag.value.to_string() }))
                .collect();
            let references: Vec<serde_json::Value> = row
                .parent
                .iter()
                .map(|parent| serde_json::json!({ "refType": "CHILD_OF", "traceID": trace.id.as_ref(), "spanID": parent.as_ref() }))
                .chain(row.references.iter().map(|reference| {
                    serde_json::json!({ "refType": "FOLLOWS_FROM", "traceID": reference.trace_id.as_ref(), "spanID": reference.span_id.as_ref() })
                }))
                .collect();
            serde_json::json!({
                "traceID": trace.id.as_ref(),
                "spanID": row.span_id.as_ref(),
                "operationName": row.operation.as_ref(),
                "startTime": row.start,
                "duration": row.duration,
                "references": references,
                "tags": tags,
                "processID": format!("p{}", span.service),
            })
        })
        .collect();
    let processes: serde_json::Map<String, serde_json::Value> = trace
        .services
        .iter()
        .enumerate()
        .map(|(index, service)| {
            (
                format!("p{index}"),
                serde_json::json!({ "serviceName": service.name.as_ref(), "tags": [] }),
            )
        })
        .collect();
    serde_json::json!({ "data": [{ "traceID": trace.id.as_ref(), "spans": spans, "processes": processes }] }).to_string()
}


/// Writes the trace as Jaeger JSON (`Trace-<last six of its id>.json`) into the working folder:
/// blenderui has no save dialog (the kit's `BK_file_dialog` is not in its API).
fn save(view: View) {
    let (json, name) = view.trace.with(|trace| {
        (export(trace), format!("Trace-{}.json", &trace.id[trace.id.len().saturating_sub(6)..]))
    });
    let path = std::env::current_dir().unwrap_or_default().join(name);
    if std::fs::write(&path, json).is_ok() {
        view.copied.set(Some(path.display().to_string().into()));
    }
}

fn menu_row(id: &'static str, tone: Tone) -> Div {
    div().id(id).class("px-2 py-1 rounded cursor-pointer").hover(move |style| style.bg(tone.accent))
}

fn trace_menu(view: View, menu: Signal<bool>, tone: Tone) -> Div {
    let share = view.options.share.with(|slot| slot.0.is_some());
    div()
        .class("absolute right-0 top-8 z-50 flex flex-col p-1 rounded-md border shadow-md min-w-[160px]")
        .bg(tone.popover)
        .text_color(tone.popover_text)
        .border_color(tone.border)
        .child(share.then(|| {
            menu_row("copy-trace-link", tone)
                .on_click(move |_, window: &mut Window, _| {
                    let id = view.trace.with(|trace| trace.id.clone());
                    if let Some(link) = ShareSlot::link(view.options.share, &id, "") {
                        copy(link, view, window);
                    }
                    menu.set(false);
                })
                .child("Copy link")
        }))
        .child(
            menu_row("export-trace", tone)
                .on_click(move |_, _, _| {
                    menu.set(false);
                    save(view);
                })
                .child("Export as JSON"),
        )
}

/// The trace's summary: its root service and operation, a banner for the worst failing span,
/// the trace's facts, the overview and the span filters.
#[allow(clippy::too_many_lines)]
pub(crate) fn trace_header(view: View) -> Div {
    let tone = Tone::of();
    let menu = use_signal(false);
    let hide = view.options.hide_header;
    let overview = view.overview.get();
    let copied = view.copied.get();
    let top = view.trace.with(|trace| {
        let root = trace.root.map(|root| &trace.spans[root as usize]);
        let service = root.map(|root| trace.service(root).clone()).unwrap_or_default();
        let operation = root.map(|root| root.row.operation.clone()).unwrap_or_default();
        let alert = banner(trace);
        let status = trace.header.status.clone();
        let status_color = match status.as_ref().and_then(|status| status.chars().next()) {
            Some('2') => tone.success,
            Some('4') => tone.warning,
            Some('5') => tone.danger,
            _ => tone.muted,
        };
        let headline_icon = match alert.map(|(_, severity)| severity) {
            Some(Severity::Error) => Some((IconName::CircleAlert, tone.danger)),
            Some(Severity::Warning) => Some((IconName::TriangleAlert, tone.warning)),
            None => status.as_ref().filter(|status| status.starts_with('2')).map(|_| (IconName::CircleCheck, tone.success)),
        };
        let banner_view = alert.filter(|_| !hide).map(|(index, severity)| {
            let span = &trace.spans[index as usize];
            let (color, glyph) = if severity == Severity::Error {
                (tone.danger, IconName::CircleAlert)
            } else {
                (tone.warning, IconName::TriangleAlert)
            };
            let detail = format!(
                "{} · {}% of trace",
                duration(span.row.duration as f64),
                percent(span.row.duration, trace.duration)
            );
            div()
                .class("flex items-center gap-2 px-2 py-1 rounded-md")
                .bg(color.opacity(0.1))
                .child(icon(glyph, color))
                .child(div().class("font-medium").child(trace.service(span).clone()))
                .child(div().class("truncate").child(path(span)))
                .child(div().text_color(tone.muted).child(detail))
                .child(div().class("flex-1"))
                .child(
                    div()
                        .id("go-to-span")
                        .class("px-2 h-6 flex items-center rounded-md border cursor-pointer")
                        .border_color(tone.border)
                        .on_click(move |_, window, _| view.go_to(index, Some(window)))
                        .child("Go to span"),
                )
        });
        let id = trace.id.clone();
        let meta = (!hide).then(|| {
            let facts: [(SharedString, SharedString); 4] = [
                ("Start time".into(), format!("{} ({})", clock(trace.start), ago(trace.start)).into()),
                ("Duration".into(), duration(trace.duration as f64)),
                ("Services".into(), trace.services.len().to_string().into()),
                ("Spans".into(), trace.spans.len().to_string().into()),
            ];
            let route = trace.header.route.clone().map(|(key, value)| {
                let mut label = key.rsplit('.').next().unwrap_or("route").to_string();
                if let Some(first) = label.get_mut(0..1) {
                    first.make_ascii_uppercase();
                }
                (SharedString::from(label), value)
            });
            let done = copied.as_ref() == Some(&id);
            div()
                .class("flex flex-wrap items-center gap-x-4 gap-y-1")
                .child(
                    div().class("flex gap-1").child(div().text_color(tone.muted).child("Trace ID")).child(
                        div()
                            .id("copy-trace-id")
                            .class("flex items-center gap-1 font-medium cursor-pointer")
                            .on_click(move |_, window, _| {
                                let id = view.trace.with(|trace| trace.id.clone());
                                copy(id, view, window);
                            })
                            .child(id.clone())
                            .child(icon(if done { IconName::Check } else { IconName::Copy }, tone.text).size(12.)),
                    ),
                )
                .children(facts.into_iter().chain(route).map(|(label, value)| {
                    div()
                        .class("flex gap-1 min-w-0")
                        .child(div().text_color(tone.muted).child(label))
                        .child(div().class("font-medium truncate max-w-[240px]").child(value))
                }))
        });
        (service, operation, trace.header.method.clone(), status, status_color, headline_icon, banner_view, meta)
    });
    let (service, operation, method, status, status_color, headline_icon, banner_view, meta) = top;
    div()
        .class("flex flex-col flex-none gap-2 p-2 border-b text-xs")
        .border_color(tone.border)
        .text_color(tone.text)
        .child(
            div()
                .class("relative flex items-center gap-2")
                .child(headline_icon.map(|(glyph, color)| icon(glyph, color)))
                .child(div().class("text-base font-semibold truncate").child(service))
                .child(div().class("text-sm truncate").text_color(tone.muted).child(operation))
                .child(method.map(|method| badge(method, tone.info)))
                .child(status.map(|status| badge(status, status_color)))
                .child(div().class("flex-1"))
                .child((!hide).then(|| {
                    div()
                        .id("trace-menu")
                        .class("flex items-center justify-center size-7 rounded-md cursor-pointer")
                        .hover(move |style| style.bg(tone.accent))
                        .on_click(move |_, _, _| menu.update(|open| *open = !*open))
                        .child(icon(IconName::EllipsisVertical, tone.text))
                }))
                .child(menu.get().then(|| trace_menu(view, menu, tone))),
        )
        .child(banner_view)
        .child(meta)
        .child((!hide).then(|| {
            div()
                .class("flex flex-col gap-1")
                .child(
                    div()
                        .id("toggle-overview")
                        .class("flex items-center gap-1 font-medium cursor-pointer")
                        .on_click(move |_, _, _| view.overview.update(|open| *open = !*open))
                        .child(icon(if overview { IconName::ChevronDown } else { IconName::ChevronRight }, tone.text).size(12.))
                        .child("Overview"),
                )
                .child(overview.then(|| minimap(view)))
        }))
        .child((!hide).then(|| filter_bar(view)))
}
