use blenderui::element::{Div, FluentBuilder, ParentElement};
use blenderui::hooks::{Signal, use_signal};
use blenderui::html::div;
use blenderui::{Hsla, MouseButton, MouseDownEvent, MouseMoveEvent, Painter, SharedString, Styled, canvas, px, relative};

use super::format::duration;
use super::tone::Tone;
use super::trace::Span;
use super::view::{Drag, View};
use crate::geom::{Bounds, Point, point, size};

const HEIGHT: f32 = 60.;
const MIN_ROWS: f32 = 60.;
const MAX_ROWS: f32 = 200.;
const MIN_BAR: f32 = 2.;
const MAX_BAR: f32 = 6.;
const MIN_WIDTH: f32 = 5.;
const ALPHA: f32 = 0.8;
const GRAB: f32 = 5.;
const TICKS: usize = 5;

fn weight(span: &Span) -> f32 {
    span.aggregation
        .as_ref()
        .filter(|aggregation| aggregation.summary)
        .and_then(|aggregation| aggregation.count)
        .map_or(1., |count| count.max(1.) as f32)
}

/// The overview's pointer: where it is (x in the canvas) and the canvas' last bounds.
#[derive(Clone, Copy)]
struct Pointer {
    at: Signal<Option<f32>>,
    bounds: Signal<Bounds>,
}

fn paint(view: View, pointer: Pointer, tone: Tone, bounds: Bounds, painter: &mut Painter) {
    pointer.bounds.update_untracked(|slot| *slot = bounds);
    let width = bounds.size.width;
    let rect = |x: f32, y: f32, w: f32, h: f32| Bounds::new(bounds.origin + point(x, y), size(w, h)).into();
    view.trace.with(|trace| {
        view.cache.with(|cache| {
            let total: f32 = trace.spans.iter().map(weight).sum::<f32>().max(1.);
            let rows = total.clamp(MIN_ROWS, MAX_ROWS);
            let scale = HEIGHT / rows;
            let bar = (rows / total).clamp(MIN_BAR, MAX_BAR) * scale;
            let span_total = trace.duration.max(1) as f32;
            let mut offset = 0.;
            for span in &trace.spans {
                let weight = weight(span);
                let x = span.row.start.saturating_sub(trace.start) as f32 / span_total * width;
                let w = (span.row.duration as f32 / span_total * width).max(MIN_WIDTH);
                let y = offset * HEIGHT / total;
                let height = (bar * weight).min(HEIGHT - y).max(0.5);
                let color = cache.colors.get(span.service as usize).copied().unwrap_or(tone.muted);
                painter.fill(rect(x, y, w, height), color.opacity(ALPHA));
                offset += weight;
            }
        });
    });
    let line = |at: f32, color: Hsla, painter: &mut Painter| painter.fill(rect(at, 0., 1., HEIGHT), color);
    for tick in 1..TICKS - 1 {
        line(width * tick as f32 / (TICKS - 1) as f32, tone.strong, painter);
    }
    let (start, end) = view.range.get();
    let shade = tone.muted.opacity(0.15);
    let (left, right) = (start as f32 * width, end as f32 * width);
    painter.fill(rect(0., 0., left, HEIGHT), shade);
    painter.fill(rect(right, 0., width - right, HEIGHT), shade);
    let drag = view.drag.get();
    if let Some(Drag::Reframe { anchor, at }) = drag {
        let (low, high) = if anchor < at { (anchor, at) } else { (at, anchor) };
        let (low, high) = (low as f32 * width, high as f32 * width);
        painter.fill(rect(low, 0., high - low, HEIGHT), tone.primary.opacity(0.2));
        line(at as f32 * width, tone.primary, painter);
    }
    let hover = pointer.at.get();
    for (edge, active) in [(left, drag == Some(Drag::Start)), (right, drag == Some(Drag::End))] {
        let near = hover.is_some_and(|at| (at - edge).abs() <= GRAB);
        let color = if active || near { tone.primary } else { tone.strong };
        line(edge.min(width - 1.), color, painter);
        let handle = if active || near { 9. } else { 3. };
        painter.fill(rect(edge - handle / 2., (HEIGHT - 20.) / 2., handle, 20.), color);
    }
    if let Some(at) = hover.filter(|_| drag.is_none()) {
        line(at, tone.danger, painter);
    }
}

/// Where `position` is across the overview (0..=1), and the overview's width.
fn position(pointer: Pointer, at: Point) -> (f32, f32) {
    let bounds = pointer.bounds.get();
    let width = bounds.size.width.max(1.);
    (((at.x - bounds.left()) / width).clamp(0., 1.), width)
}

fn press(view: View, pointer: Pointer, event: &MouseDownEvent) {
    let (at, width) = position(pointer, Point::from(event.position));
    let (start, end) = view.range.get();
    let near = |edge: f64| ((f64::from(at) - edge) * f64::from(width)).abs() <= f64::from(GRAB);
    let drag = if near(start) {
        Drag::Start
    } else if near(end) {
        Drag::End
    } else {
        Drag::Reframe { anchor: f64::from(at), at: f64::from(at) }
    };
    view.drag.set(Some(drag));
}

fn moved(view: View, pointer: Pointer, event: &MouseMoveEvent) {
    let (at, width) = position(pointer, Point::from(event.position));
    pointer.at.set(Some(at * width));
    let Some(drag) = view.drag.get() else { return };
    if event.pressed_button.is_none() {
        release(view);
        return;
    }
    let at = f64::from(at);
    let (start, end) = view.range.get();
    match drag {
        Drag::Start => view.range.set((at.min(end - 0.001), end)),
        Drag::End => view.range.set((start, at.max(start + 0.001))),
        Drag::Reframe { anchor, .. } => view.drag.set(Some(Drag::Reframe { anchor, at })),
    }
}

fn release(view: View) {
    let Some(drag) = view.drag.get() else { return };
    view.drag.set(None);
    if let Drag::Reframe { anchor, at } = drag {
        let (low, high) = if anchor < at { (anchor, at) } else { (at, anchor) };
        if high - low > 0.002 {
            view.range.set((low, high));
        }
    }
}

/// The overview of the whole trace: a span per bar; drag to frame a time range, or its edges.
pub(crate) fn minimap(view: View) -> Div {
    let tone = Tone::of();
    let pointer = Pointer { at: use_signal(None::<f32>), bounds: use_signal(Bounds::default()) };
    let total = view.trace.with(|trace| trace.duration as f64);
    let labels = (0..TICKS).map(|index| {
        let at = index as f64 / (TICKS - 1) as f64;
        let label: SharedString = duration(total * at);
        div()
            .class("absolute top-0 whitespace-nowrap")
            .map(|el| if index + 1 == TICKS { el.right(px(0.)) } else { el.left(relative(at as f32)) })
            .child(label)
    });
    let zoomed = view.range.get() != (0., 1.);
    div()
        .class("relative flex flex-col gap-1")
        .child(div().class("relative h-4 text-[11px]").text_color(tone.muted).children(labels))
        .child(
            canvas(move |bounds, painter| paint(view, pointer, tone, bounds.into(), painter))
                .id("trace-minimap")
                .class("w-full")
                .h(px(HEIGHT))
                .on_mouse_down(MouseButton::Left, move |event, _, _| press(view, pointer, event))
                .on_mouse_move(move |event, _, _| moved(view, pointer, event))
                .on_mouse_up(MouseButton::Left, move |_, _, _| release(view))
                .on_hover(move |inside: &bool, _, _| {
                    if !inside {
                        pointer.at.set(None);
                    }
                }),
        )
        .child(zoomed.then(|| {
            div()
                .id("reset-selection")
                .class("absolute right-1 top-6 px-2 h-6 flex items-center rounded-md border text-xs cursor-pointer")
                .bg(tone.background)
                .border_color(tone.border)
                .on_click(move |_, _, _| view.range.set((0., 1.)))
                .child("Reset selection")
        }))
}
