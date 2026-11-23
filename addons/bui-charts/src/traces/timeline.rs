use std::collections::BTreeSet;

use blenderui::element::{AnyElement, Div, FluentBuilder, IntoElement, ParentElement};
use blenderui::html::div;
use blenderui::{
    ClickEvent, Hsla, KeyDownEvent, MouseButton, MouseDownEvent, MouseMoveEvent, SharedString, Styled, black,
    canvas, px, relative, rgb, white,
};

use super::data::SpanKind;
use super::detail::span_detail;
use super::format::duration;
use super::tone::Tone;
use super::trace::{Span, Trace};
use super::view::{Row, SpanBarLabel, View, collapse_all, collapse_one, expand_one, row_id};
use crate::color::mix;
use crate::geom::Point;
use crate::icon::{IconName, icon};

pub(crate) const ROW_HEIGHT: f32 = 28.;
const HEADER_HEIGHT: f32 = 38.;
const INDENT: f32 = 17.;
const MIN_COLUMN: f32 = 0.2;
const MAX_COLUMN: f32 = 0.85;
const TICKS: usize = 5;
const CRITICAL_FILL: u32 = 0x0020_2226;
const CRITICAL_EDGE: u32 = 0x00c7_d0d9;
const MARKER_STEP: f64 = 0.002;

type Change = fn(&Trace, &mut BTreeSet<u32>);

pub(crate) fn fraction(trace: &Trace, micros: u64, (start, end): (f64, f64)) -> f64 {
    let total = trace.duration.max(1) as f64;
    let at = micros.saturating_sub(trace.start) as f64 / total;
    (at - start) / (end - start).max(f64::EPSILON)
}

/// A color as CSS text (for gradients, which only stylesheets draw).
fn css(color: Hsla) -> String {
    let byte = |channel: f32| (channel.clamp(0., 1.) * 255.).round() as u8;
    format!("rgba({}, {}, {}, {})", byte(color.r), byte(color.g), byte(color.b), color.a)
}

fn icon_button(id: &'static str, name: IconName, tone: Tone, change: Change, view: View) -> Div {
    div()
        .id(id)
        .class("flex items-center justify-center size-6 rounded-md cursor-pointer")
        .hover(move |style| style.bg(tone.accent))
        .on_click(move |_: &ClickEvent, _, _| apply(view, change))
        .child(icon(name, tone.muted))
}

fn apply(view: View, change: Change) {
    let mut collapsed = view.collapsed.get();
    view.trace.with(|trace| change(trace, &mut collapsed));
    view.collapsed.set(collapsed);
}

/// The span tree and its timeline: the column of names, the bars, the details of open spans.
pub(crate) fn timeline(view: View) -> Div {
    let tone = Tone::of();
    let column = view.column.get();
    let ticks = ticks(view);
    let selection = view.zoom.get().map(|(anchor, at)| {
        let (low, high) = if anchor < at { (anchor, at) } else { (at, anchor) };
        div()
            .class("absolute top-0 h-full")
            .left(relative(low as f32))
            .w(relative((high - low) as f32))
            .bg(tone.danger.opacity(0.15))
    });
    let resizing = view.resizing.get();
    // the row list is copied out: each row reads the cache again
    let order = view.cache.with(|cache| cache.rows.clone());
    let rows = order.iter().enumerate().map(|(index, row)| match *row {
        Row::Span(span) => span_row(view, span, &order[..index]),
        Row::Detail(span) => span_detail(view, span).into_any_element(),
    });
    let tick_labels = ticks.into_iter().enumerate().flat_map(|(index, (at, label))| {
        let text = div()
            .class("absolute bottom-1 px-1 whitespace-nowrap")
            .text_color(tone.muted)
            .map(|el| if index + 1 == TICKS { el.right(px(0.)) } else { el.left(relative(at)) })
            .child(label);
        let line = div().class("absolute top-0 h-full w-[1px]").left(relative(at)).bg(tone.border);
        [text, line]
    });

    div()
        .id("trace-timeline")
        .class("relative flex flex-col flex-1 min-h-0 text-xs")
        .text_color(tone.text)
        .on_key_down(move |event: &KeyDownEvent, _, _| shortcut(view, event))
        .on_mouse_move(move |event: &MouseMoveEvent, window, _| {
            view.scroll_pending(window);
            drag(view, event);
        })
        .on_mouse_up(MouseButton::Left, move |_, _, _| release(view))
        .child(
            canvas(move |bounds, _| view.area.update_untracked(|area| *area = bounds.into()))
                .class("absolute inset-0 size-full pointer-events-none"),
        )
        .child(
            div()
                .class("flex flex-none border-b")
                .h(px(HEADER_HEIGHT))
                .bg(tone.secondary)
                .border_color(tone.border)
                .child(
                    div()
                        .class("flex items-center gap-1 px-2 flex-none overflow-hidden")
                        .w(relative(column))
                        .child(div().class("font-medium flex-1 truncate").child("Service & Operation"))
                        .child(icon_button("expand-one", IconName::ChevronDown, tone, expand_one, view))
                        .child(icon_button("collapse-one", IconName::ChevronUp, tone, collapse_one, view))
                        .child(icon_button("expand-all", IconName::Maximize, tone, |_, collapsed| collapsed.clear(), view))
                        .child(icon_button("collapse-all", IconName::Minimize, tone, collapse_all, view)),
                )
                .child(
                    div()
                        .id("trace-ticks")
                        .class("relative flex-1 h-full overflow-hidden cursor-crosshair")
                        .on_mouse_down(MouseButton::Left, move |event: &MouseDownEvent, _, _| start_zoom(view, event))
                        .children(tick_labels)
                        .child(selection),
                ),
        )
        .child(div().id("trace-rows").class("relative flex flex-col flex-1 min-h-0 overflow-y-scroll").children(rows))
        .child(
            div()
                .id("column-resizer")
                .class("absolute top-0 bottom-0 w-[6px] cursor-col-resize")
                .left(relative(column))
                .ml(px(-3.))
                .when(resizing, move |el| el.bg(tone.primary.opacity(0.4)))
                .hover(move |style| style.bg(tone.border))
                .on_mouse_down(MouseButton::Left, move |_, _, _| view.resizing.set(true)),
        )
}

fn shortcut(view: View, event: &KeyDownEvent) {
    let change: Change = match event.keystroke.key.as_str() {
        "]" | "bracketright" => collapse_all,
        "[" | "bracketleft" => |_, collapsed| collapsed.clear(),
        "p" => collapse_one,
        "o" => expand_one,
        _ => return,
    };
    apply(view, change);
}

fn ticks(view: View) -> Vec<(f32, SharedString)> {
    let (start, end) = view.range.get();
    let total = view.trace.with(|trace| trace.duration as f64);
    (0..TICKS)
        .map(|index| {
            let at = index as f64 / (TICKS - 1) as f64;
            (at as f32, duration((end - start).mul_add(at, start) * total))
        })
        .collect()
}

/// Where `x` is along the bars (0..=1).
fn along(view: View, x: f32) -> f64 {
    let area = view.area.get();
    let column = view.column.get();
    let left = area.left() + area.size.width * column;
    let width = (area.size.width * (1. - column)).max(1.);
    f64::from(((x - left) / width).clamp(0., 1.))
}

fn start_zoom(view: View, event: &MouseDownEvent) {
    let at = along(view, Point::from(event.position).x);
    view.zoom.set(Some((at, at)));
}

fn drag(view: View, event: &MouseMoveEvent) {
    if event.pressed_button.is_none() {
        // released outside the timeline
        if view.resizing.get() || view.zoom.get().is_some() {
            release(view);
        }
        return;
    }
    let x = Point::from(event.position).x;
    if view.resizing.get() {
        let area = view.area.get();
        let column = ((x - area.left()) / area.size.width.max(1.)).clamp(MIN_COLUMN, MAX_COLUMN);
        view.column.set(column);
    }
    if let Some((anchor, _)) = view.zoom.get() {
        view.zoom.set(Some((anchor, along(view, x))));
    }
}

fn release(view: View) {
    if view.resizing.get() {
        view.resizing.set(false);
    }
    if let Some((anchor, at)) = view.zoom.get() {
        view.zoom.set(None);
        let (low, high) = if anchor < at { (anchor, at) } else { (at, anchor) };
        if high - low > 0.005 {
            let (start, end) = view.range.get();
            let span = end - start;
            view.range.set((span.mul_add(low, start), span.mul_add(high, start)));
        }
    }
}

fn previous_service(trace: &Trace, before: &[Row]) -> Option<u32> {
    before.iter().rev().find_map(|row| match row {
        Row::Span(span) => Some(trace.spans[*span as usize].service),
        Row::Detail(_) => None,
    })
}

fn bar_label(view: View, span: &Span) -> Option<SharedString> {
    match view.options.bar_label {
        SpanBarLabel::None => None,
        SpanBarLabel::Duration => Some(format!("({})", duration(span.row.duration as f64)).into()),
        SpanBarLabel::Tag(key) => span
            .row
            .tags
            .iter()
            .chain(&span.row.service_tags)
            .find(|tag| tag.key.as_ref() == key)
            .map(|tag| format!("({})", tag.value).into()),
    }
}

struct Peer {
    color: Hsla,
    name: SharedString,
    operation: Option<SharedString>,
    child: Option<u32>,
}

fn peer(trace: &Trace, colors: &[Hsla], index: u32, collapsed: bool) -> Option<Peer> {
    let span = &trace.spans[index as usize];
    if span.row.kind != Some(SpanKind::Client) {
        return None;
    }
    if collapsed && span.children > 0 {
        let child = trace.children(index).find(|child| {
            let child = &trace.spans[*child as usize];
            child.row.kind == Some(SpanKind::Server) && child.service != span.service
        })?;
        let server = &trace.spans[child as usize];
        return Some(Peer {
            color: colors[server.service as usize],
            name: trace.service(server).clone(),
            operation: Some(server.row.operation.clone()),
            child: Some(child),
        });
    }
    if span.children > 0 {
        return None;
    }
    let peer = span.tag("peer.service")?;
    Some(Peer { color: colors[span.service as usize], name: peer.value.to_string().into(), operation: None, child: None })
}

/// One span's row: its name column (guides, chevron, service, operation) and its bar.
#[allow(clippy::too_many_lines)]
fn span_row(view: View, index: u32, before: &[Row]) -> AnyElement {
    let tone = Tone::of();
    let column = view.column.get();
    let range = view.range.get();
    let collapsed = view.collapsed.with(|collapsed| collapsed.contains(&index));
    let expanded = view.details.with(|details| details.contains(&index));
    let focused = view.focus.get() == Some(index);
    let hovered = view.hover.get() == Some(index);
    let guide = view.guide.get();
    let matches_only = view.filters.with(|filters| filters.matches_only);
    let marker = view.marker.get();
    view.cache.with(|cache| {
        view.trace.with(|trace| {
            let span = &trace.spans[index as usize];
            let color = cache.colors[span.service as usize];
            let error = span.error || (collapsed && cache.error_below[index as usize]);
            let error_color = span.row.error_color.unwrap_or(tone.danger);
            let background = if focused {
                Some(tone.info.opacity(0.3))
            } else if expanded {
                Some(tone.secondary)
            } else if cache.matched(index) && !matches_only {
                Some(tone.warning.opacity(0.18))
            } else if span.error {
                Some(tone.danger.opacity(0.08))
            } else {
                None
            };
            let mut ancestors: Vec<u32> = trace.ancestors(index).collect();
            ancestors.reverse();
            let show_service = previous_service(trace, before) != Some(span.service);
            let service = trace.service(span).clone();
            let operation = span.row.operation.clone();
            let summary_bar = span.aggregation.as_ref().is_some_and(|aggregation| aggregation.summary);
            let summary = span
                .aggregation
                .as_ref()
                .filter(|aggregation| aggregation.summary)
                .and_then(|aggregation| aggregation.count)
                .map(|count| SharedString::from(format!("{count}")));
            let label = bar_label(view, span);
            let peer = peer(trace, &cache.colors, index, collapsed);
            let (low, high) = (fraction(trace, span.row.start, range), fraction(trace, span.finish(), range));
            let (bar_left, bar_width) = (low as f32, (high - low).max(0.) as f32);
            let label_left = low > 1. - high;
            let text = duration(span.row.duration as f64);
            let bar_text: SharedString = match (hovered, label_left) {
                (false, _) => text,
                (true, true) => format!("{service}::{operation} | {text}").into(),
                (true, false) => format!("{text} | {service}::{operation}").into(),
            };
            let strip = peer.as_ref().and_then(|peer| peer.child).map(|child| {
                let child = &trace.spans[child as usize];
                let (from, to) = (fraction(trace, child.row.start, range), fraction(trace, child.finish(), range));
                (from as f32, (to - from).max(0.) as f32)
            });
            let sections = cache.sections_of(index).iter().map(|section| {
                let (from, to) = (fraction(trace, section.start, range), fraction(trace, section.end, range));
                (from as f32, (to - from).max(0.) as f32)
            });
            let mut markers: Vec<(u32, f32)> = Vec::new();
            for log in &span.row.logs {
                let at = fraction(trace, log.timestamp, range);
                let key = (at / MARKER_STEP).round() as u32;
                if markers.last().is_none_or(|(last, _)| *last != key) {
                    markers.push((key, at as f32));
                }
            }
            let marker_card = marker.filter(|(span, _)| *span == index).and_then(|(_, group)| {
                let at = markers.iter().find(|(key, _)| *key == group).map(|(_, at)| *at)?;
                let logs = span
                    .row
                    .logs
                    .iter()
                    .filter(|log| (fraction(trace, log.timestamp, range) / MARKER_STEP).round() as u32 == group)
                    .map(|log| {
                        let when = duration(log.timestamp.saturating_sub(trace.start) as f64);
                        let name: SharedString = log.name.clone().unwrap_or_else(|| {
                            log.fields
                                .iter()
                                .map(|field| format!("{}={}", field.key, field.value))
                                .collect::<Vec<_>>()
                                .join(" ")
                                .into()
                        });
                        div()
                            .class("flex gap-2")
                            .child(div().text_color(tone.muted).child(when))
                            .child(div().class("truncate").child(name))
                    });
                Some(
                    div()
                        .class("absolute top-full z-50 p-2 rounded-md border shadow-md flex flex-col gap-1 max-w-[360px] pointer-events-none")
                        .left(relative(at))
                        .bg(tone.popover)
                        .text_color(tone.popover_text)
                        .border_color(tone.border)
                        .children(logs),
                )
            });
            let peer_view = peer.map(|peer| {
                div()
                    .class("flex items-center gap-1 flex-none")
                    .child(icon(IconName::ArrowRight, tone.text).size(12.))
                    .child(div().class("size-2 rounded-sm").bg(peer.color))
                    .child(div().class("font-medium").child(peer.name))
                    .child(peer.operation.map(|operation| div().text_color(tone.muted).child(operation)))
            });
            let bar = div().class("absolute rounded-sm min-w-[2px]").top(relative(0.3)).h(relative(0.4)).left(relative(bar_left)).w(relative(bar_width));
            let bar = if summary_bar {
                let (light, dark) = (mix(color, white(), 0.3), mix(color, black(), 0.55));
                bar.css("background", format!("linear-gradient(to right, {}, {})", css(light), css(dark)))
            } else {
                bar.bg(color)
            };
            let chevron = (span.children > 0)
                .then(|| icon(if collapsed { IconName::ChevronRight } else { IconName::ChevronDown }, tone.muted).size(12.));
            let guides = ancestors.into_iter().map(|ancestor| {
                div()
                    .id(format!("guide-{index}-{ancestor}"))
                    .class("flex-none h-full border-l")
                    .w(px(INDENT))
                    .border_color(if guide == Some(ancestor) { tone.strong } else { tone.border })
                    .on_hover(move |inside: &bool, _, _| view.guide.set(inside.then_some(ancestor)))
            });
            let names = div()
                .class("flex items-center gap-1 flex-none overflow-hidden pr-2 border-b-2")
                .w(relative(column))
                .border_color(color.opacity(0.81))
                .children(guides)
                .child(
                    div()
                        .id(format!("chevron-{index}"))
                        .class("flex items-center justify-center flex-none size-4")
                        .on_click(move |_, _, cx| {
                            cx.stop_propagation();
                            view.collapsed.update(|collapsed| {
                                if !collapsed.remove(&index) {
                                    collapsed.insert(index);
                                }
                            });
                        })
                        .child(chevron),
                )
                .child(error.then(|| div().class("flex-none").child(icon(IconName::CircleAlert, error_color).size(12.))))
                .child(show_service.then(|| div().class("font-medium whitespace-nowrap").when(collapsed, |el| el.class("italic")).child(service)))
                .child(peer_view)
                .child(div().class("truncate").text_color(tone.muted).child(operation))
                .child(summary.map(|count| div().class("px-1 rounded-sm text-[10px]").bg(color).text_color(tone.background).child(count)))
                .child(label.map(|label| div().class("flex-none whitespace-nowrap").text_color(tone.muted).child(label)));
            let grid = (1..TICKS - 1).map(|tick| {
                div()
                    .class("absolute top-0 h-full w-[1px]")
                    .left(relative(tick as f32 / (TICKS - 1) as f32))
                    .bg(tone.border.opacity(0.5))
            });
            let critical = sections.map(|(left, width)| {
                div()
                    .class("absolute border-x")
                    .top(relative(0.44))
                    .h(relative(0.11))
                    .left(relative(left))
                    .w(relative(width))
                    .bg(rgb(CRITICAL_FILL))
                    .border_color(rgb(CRITICAL_EDGE))
            });
            let marks = markers.into_iter().map(|(group, at)| {
                div()
                    .id(format!("marker-{index}-{group}"))
                    .class("absolute min-w-[1px] w-[1px]")
                    .top(relative(0.2))
                    .h(relative(0.6))
                    .left(relative(at))
                    .bg(tone.text)
                    .on_hover(move |inside: &bool, _, _| view.marker.set(inside.then_some((index, group))))
            });
            let bars = div()
                .class("absolute size-full overflow-hidden")
                .children(grid)
                .child(bar)
                .child(strip.map(|(left, width)| {
                    div()
                        .class("absolute")
                        .top(relative(0.35))
                        .h(relative(0.3))
                        .left(relative(left))
                        .w(relative(width))
                        .bg(mix(color, black(), 0.3))
                }))
                .children(critical)
                .children(marks)
                .child(
                    div()
                        .class("absolute top-0 h-full flex items-center px-1 whitespace-nowrap")
                        .text_color(tone.muted)
                        .map(|el| {
                            if label_left {
                                el.right(relative(1. - bar_left))
                            } else {
                                el.left(relative(bar_left + bar_width))
                            }
                        })
                        .child(bar_text),
                );
            div()
                .id(row_id(index))
                .class("flex w-full flex-none cursor-pointer")
                .h(px(ROW_HEIGHT))
                .when_some(background, Styled::bg)
                .hover(move |style| style.bg(tone.accent.opacity(0.5)))
                .on_hover(move |inside: &bool, _, _| {
                    let next = inside.then_some(index).or_else(|| view.hover.get().filter(|hovered| *hovered != index));
                    if view.hover.get() != next {
                        view.hover.set(next);
                    }
                })
                .on_click(move |_, _, _| {
                    view.focus.set(None);
                    view.details.update(|details| {
                        if !details.remove(&index) {
                            details.insert(index);
                        }
                    });
                })
                .child(names)
                .child(div().class("relative flex-1 h-full").child(bars).child(marker_card))
                .into_any_element()
        })
    })
}
