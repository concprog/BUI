mod colors;
mod data;
mod detail;
mod filter;
mod format;
mod header;
mod import;
mod minimap;
mod search;
mod timeline;
mod tone;
mod trace;
mod view;

use std::collections::{BTreeMap, BTreeSet};

use blenderui::element::{AnyElement, Div, FluentBuilder, IntoElement, ParentElement};
use blenderui::hooks::use_signal;
use blenderui::html::div;
use blenderui::{SharedString, Styled};

pub use data::{SpanKind, SpanLog, SpanReference, SpanRow, SpanStatus, Tag, TagValue};
pub use filter::{Filter, FilterKey, FilterOp, SpanFilters};
pub use trace::{Trace, TraceData};
pub use view::{Selected as SelectedSpan, SpanAction, SpanBarLabel};

use crate::component;
use crate::geom::Bounds;
use header::trace_header;
use timeline::timeline;
use tone::Tone;
use view::{Options, ShareSlot, View};

const NAME_COLUMN: f32 = 0.4;

/// A trace as a tree of spans on a timeline (Grafana's trace view): a header with the trace's
/// summary, an overview to frame a time range in, span filters, and the spans with their details.
#[component]
pub fn traces<D: TraceData, F: Fn(&str, &str) -> SharedString + 'static>(
    data: D,
    filters: Option<SpanFilters>,
    focused_span: Option<SharedString>,
    hide_header_details: Option<bool>,
    share_link: Option<F>,
    actions: Option<&'static [SpanAction]>,
    bar_label: Option<SpanBarLabel>,
    name_column: Option<f32>,
    class: Option<SharedString>,
) -> AnyElement {
    let slot = use_signal(Trace::default());
    let revision = data.revision(slot);
    let trace = data.into_signal(slot);
    let seeded = use_signal(false);
    let filter_state = use_signal(SpanFilters::default());
    let share = use_signal(ShareSlot::default());
    let focused = use_signal(None::<SharedString>);
    if !seeded.get() {
        if let Some(filters) = filters {
            filter_state.update_untracked(|state| *state = filters);
        }
        seeded.update_untracked(|seeded| *seeded = true);
    }
    share.update_untracked(|slot| slot.0 = share_link.map(|link| Box::new(link) as Box<_>));
    let view = View {
        trace,
        cache: use_signal(view::Cache::default()),
        filters: filter_state,
        range: use_signal((0f64, 1f64)),
        collapsed: use_signal(BTreeSet::<u32>::new()),
        details: use_signal(BTreeSet::<u32>::new()),
        open: use_signal(BTreeMap::<u32, u16>::new()),
        focus: use_signal(None::<u32>),
        cursor: use_signal(None::<usize>),
        column: use_signal(name_column.unwrap_or(NAME_COLUMN).clamp(0.2, 0.85)),
        guide: use_signal(None::<u32>),
        hover: use_signal(None::<u32>),
        overview: use_signal(true),
        drag: use_signal(None::<view::Drag>),
        resizing: use_signal(false),
        zoom: use_signal(None::<(f64, f64)>),
        marker: use_signal(None::<(u32, u32)>),
        area: use_signal(Bounds::default()),
        pending: use_signal(None::<u32>),
        copied: use_signal(None::<SharedString>),
        options: Options {
            hide_header: hide_header_details.unwrap_or(false),
            bar_label: bar_label.unwrap_or_default(),
            share,
            actions: actions.unwrap_or(&[]),
        },
    };
    view.refresh(revision);
    if focused.with(|focused| *focused != focused_span) {
        let target = focused_span.as_ref().and_then(|id| trace.with(|trace| trace.index.get(id).copied()));
        focused.update_untracked(|focused| focused.clone_from(&focused_span));
        if let Some(target) = target {
            // its writes render again, with the rows they open
            view.go_to(target, None);
        }
    }

    let tone = Tone::of();
    if trace.with(Trace::is_empty) {
        return div()
            .class("flex size-full items-center justify-center text-sm")
            .text_color(tone.muted)
            .when_some(class, Div::class)
            .child("No data found in response")
            .into_any_element();
    }
    div()
        .class("flex flex-col size-full min-w-0 min-h-0")
        .bg(tone.background)
        .when_some(class, Div::class)
        .child(trace_header(view))
        .child(timeline(view))
        .into_any_element()
}
