use std::collections::{BTreeMap, BTreeSet};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::mem;

use blenderui::hooks::Signal;
use blenderui::{App, Hsla, SharedString, Window};

use super::colors;
use super::filter::{self, Section, SpanFilters};
use super::trace::Trace;
use crate::ChartTheme;
use crate::data::Revision;
use crate::geom::Bounds;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SpanBarLabel {
    #[default]
    Duration,
    None,
    Tag(&'static str),
}

pub struct Selected<'a> {
    pub trace_id: &'a str,
    pub span_id: &'a str,
    pub service: &'a str,
    pub operation: &'a str,
    pub start: u64,
    pub duration: u64,
}

#[derive(Clone, Copy)]
pub struct SpanAction {
    pub label: &'static str,
    pub run: fn(&Selected<'_>, &mut Window, &mut App),
}

type ShareFn = dyn Fn(&str, &str) -> SharedString;

#[derive(Default)]
pub(crate) struct ShareSlot(pub Option<Box<ShareFn>>);

impl ShareSlot {
    /// The link of a span (`""`: of the trace), when the app shares links.
    pub(crate) fn link(slot: Signal<Self>, trace: &str, span: &str) -> Option<SharedString> {
        slot.with(|slot| slot.0.as_ref().map(|share| share(trace, span)))
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Options {
    pub hide_header: bool,
    pub bar_label: SpanBarLabel,
    pub share: Signal<ShareSlot>,
    pub actions: &'static [SpanAction],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Row {
    Span(u32),
    Detail(u32),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Drag {
    Reframe { anchor: f64, at: f64 },
    Start,
    End,
}

#[derive(Default)]
pub(crate) struct Cache {
    data_key: Option<u64>,
    rows_key: Option<u64>,
    pub matches: Option<Vec<bool>>,
    pub hits: Vec<u32>,
    pub critical: Vec<bool>,
    pub sections: Vec<Section>,
    pub colors: Vec<Hsla>,
    pub error_below: Vec<bool>,
    pub rows: Vec<Row>,
    pub p90: u64,
    pub keys: Vec<SharedString>,
}

impl Cache {
    pub(crate) fn matched(&self, span: u32) -> bool {
        self.matches.as_ref().is_some_and(|matches| matches.get(span as usize).copied().unwrap_or(false))
    }

    pub(crate) fn sections_of(&self, span: u32) -> &[Section] {
        let start = self.sections.partition_point(|section| section.span < span);
        let end = self.sections.partition_point(|section| section.span <= span);
        &self.sections[start..end]
    }
}

#[derive(Clone, Copy)]
pub(crate) struct View {
    pub trace: Signal<Trace>,
    pub cache: Signal<Cache>,
    pub filters: Signal<SpanFilters>,
    pub range: Signal<(f64, f64)>,
    pub collapsed: Signal<BTreeSet<u32>>,
    pub details: Signal<BTreeSet<u32>>,
    pub open: Signal<BTreeMap<u32, u16>>,
    pub focus: Signal<Option<u32>>,
    pub cursor: Signal<Option<usize>>,
    pub column: Signal<f32>,
    pub guide: Signal<Option<u32>>,
    pub hover: Signal<Option<u32>>,
    pub overview: Signal<bool>,
    pub drag: Signal<Option<Drag>>,
    pub resizing: Signal<bool>,
    pub zoom: Signal<Option<(f64, f64)>>,
    pub marker: Signal<Option<(u32, u32)>>,
    /// the timeline's bounds at its last paint
    pub area: Signal<Bounds>,
    /// a span to scroll to once a handler can (one revealed while rendering, by `focused_span`)
    pub pending: Signal<Option<u32>>,
    pub copied: Signal<Option<SharedString>>,
    pub options: Options,
}

/// The element id of a span's row in the timeline.
pub(crate) fn row_id(span: u32) -> SharedString {
    format!("span-row-{span}").into()
}

impl View {
    /// Derives the rows and colors again when the trace, the filters or what is collapsed or
    /// open changed. Called once per render (its revisions are hooks).
    pub(crate) fn refresh(self, trace: Revision) {
        let data_key = {
            let mut state = DefaultHasher::new();
            (trace, Revision::of(self.filters), ChartTheme::revision()).hash(&mut state);
            state.finish()
        };
        let rows_key = {
            let mut state = DefaultHasher::new();
            (data_key, Revision::of(self.collapsed), Revision::of(self.details)).hash(&mut state);
            state.finish()
        };
        if self.cache.with(|cache| (cache.data_key, cache.rows_key)) == (Some(data_key), Some(rows_key)) {
            return;
        }
        let mut cache = Cache::default();
        self.cache.update_untracked(|slot| mem::swap(slot, &mut cache));
        if cache.data_key != Some(data_key) {
            self.refresh_data(&mut cache);
            cache.data_key = Some(data_key);
        }
        cache.rows.clear();
        self.refresh_rows(&mut cache);
        cache.rows_key = Some(rows_key);
        self.cache.update_untracked(|slot| *slot = cache);
    }

    fn refresh_data(self, cache: &mut Cache) {
        self.trace.with(|trace| {
            cache.sections = filter::critical_path(trace);
            cache.critical = vec![false; trace.spans.len()];
            for section in &cache.sections {
                cache.critical[section.span as usize] = true;
            }
            cache.matches = self.filters.with(|filters| filter::matches(trace, filters, &cache.critical));
            cache.hits = cache.matches.as_ref().map_or_else(Vec::new, |matches| {
                matches.iter().enumerate().filter(|(_, hit)| **hit).map(|(index, _)| index as u32).collect()
            });
            colors::assign(trace, &ChartTheme::colors(), &mut cache.colors);
            cache.error_below = vec![false; trace.spans.len()];
            for index in (0..trace.spans.len()).rev() {
                let span = &trace.spans[index];
                if let Some(parent) = span.parent
                    && (span.error || cache.error_below[index])
                {
                    cache.error_below[parent as usize] = true;
                }
            }
            let mut durations: Vec<u64> = trace.spans.iter().map(|span| span.row.duration).collect();
            durations.sort_unstable();
            cache.p90 = durations.get(durations.len() * 9 / 10).copied().unwrap_or_default();
            let mut keys: BTreeSet<SharedString> = BTreeSet::new();
            let mut logs: BTreeSet<SharedString> = BTreeSet::new();
            for span in &trace.spans {
                for tag in span.row.tags.iter().chain(&span.row.service_tags) {
                    keys.insert(tag.key.clone());
                }
                for log in &span.row.logs {
                    logs.extend(log.fields.iter().map(|field| field.key.clone()));
                }
            }
            for pseudo in ["kind", "status", "status.message", "library.name", "library.version", "trace.state", "id"] {
                keys.insert(SharedString::Borrowed(pseudo));
            }
            cache.keys = [
                filter::FilterKey::TEXT,
                filter::FilterKey::SERVICE,
                filter::FilterKey::SPAN_NAME,
                filter::FilterKey::DURATION,
            ]
            .into_iter()
            .map(SharedString::Borrowed)
            .chain(keys)
            .chain(logs)
            .collect();
        });
    }

    fn refresh_rows(self, cache: &mut Cache) {
        let only = self.filters.with(|filters| filters.matches_only) && !cache.hits.is_empty();
        self.trace.with(|trace| {
            self.collapsed.with(|collapsed| {
                self.details.with(|details| {
                    let mut index = 0u32;
                    let count = trace.spans.len() as u32;
                    while index < count {
                        let span = &trace.spans[index as usize];
                        if !only || cache.matched(index) {
                            cache.rows.push(Row::Span(index));
                            if details.contains(&index) {
                                cache.rows.push(Row::Detail(index));
                            }
                        }
                        index = if collapsed.contains(&index) { span.end } else { index + 1 };
                    }
                });
            });
        });
    }

    /// Opens the way to `span` (its ancestors, a filter hiding it), focuses it and scrolls to it:
    /// now when there is a window (a handler), else at the next pointer event over the timeline.
    pub(crate) fn reveal(self, span: u32, window: Option<&mut Window>) {
        let ancestors: Vec<u32> = self.trace.with(|trace| trace.ancestors(span).collect());
        self.collapsed.update(|collapsed| {
            for ancestor in ancestors {
                collapsed.remove(&ancestor);
            }
        });
        let hidden = self.cache.with(|cache| cache.matches.is_some() && !cache.matched(span));
        if hidden && self.filters.with(|filters| filters.matches_only) {
            self.filters.update(|filters| filters.matches_only = false);
        }
        self.focus.set(Some(span));
        match window {
            Some(window) => window.scroll_into_view(&row_id(span), 0.5, None),
            None => self.pending.update_untracked(|pending| *pending = Some(span)),
        }
    }

    pub(crate) fn go_to(self, span: u32, window: Option<&mut Window>) {
        self.details.update(|details| {
            details.insert(span);
        });
        self.reveal(span, window);
    }

    /// Scrolls to a span revealed while rendering.
    pub(crate) fn scroll_pending(self, window: &mut Window) {
        if let Some(span) = self.pending.get() {
            self.pending.update_untracked(|pending| *pending = None);
            window.scroll_into_view(&row_id(span), 0.5, None);
        }
    }
}

pub(crate) fn collapse_all(trace: &Trace, collapsed: &mut BTreeSet<u32>) {
    collapsed.extend(
        trace
            .spans
            .iter()
            .enumerate()
            .filter(|(_, span)| span.children > 0)
            .map(|(index, _)| index as u32),
    );
}

pub(crate) fn collapse_one(trace: &Trace, collapsed: &mut BTreeSet<u32>) {
    let mut nearest: Option<u32> = None;
    for (index, span) in trace.spans.iter().enumerate() {
        let index = index as u32;
        match nearest {
            Some(open) if span.depth <= trace.spans[open as usize].depth => {
                collapsed.insert(open);
                nearest = (span.children > 0).then_some(index);
            }
            _ if span.children > 0 && !collapsed.contains(&index) => nearest = Some(index),
            _ => {}
        }
    }
    if let Some(open) = nearest {
        collapsed.insert(open);
    }
}

pub(crate) fn expand_one(trace: &Trace, collapsed: &mut BTreeSet<u32>) {
    let mut depth: Option<u32> = None;
    let mut ready = true;
    for (index, span) in trace.spans.iter().enumerate() {
        if depth.is_some_and(|depth| span.depth <= depth) {
            ready = true;
        }
        if ready && collapsed.remove(&(index as u32)) {
            ready = false;
            depth = Some(span.depth);
        }
    }
}
