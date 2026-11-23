use std::collections::HashMap;

use std::hash::{DefaultHasher, Hash, Hasher};

use blenderui::SharedString;
use blenderui::hooks::Signal;

use crate::data::Revision;

use super::data::{SpanRow, SpanStatus, Tag};

#[derive(Default)]
pub struct Trace {
    pub(crate) id: SharedString,
    pub(crate) spans: Vec<Span>,
    pub(crate) index: HashMap<SharedString, u32>,
    pub(crate) start: u64,
    pub(crate) duration: u64,
    pub(crate) root: Option<u32>,
    pub(crate) services: Vec<Service>,
    pub(crate) header: HeaderTags,
}

pub(crate) struct Service {
    pub name: SharedString,
    pub spans: u32,
}

pub(crate) struct Span {
    pub row: SpanRow,
    pub depth: u32,
    pub parent: Option<u32>,
    pub follows: bool,
    pub end: u32,
    pub children: u32,
    pub service: u32,
    pub error: bool,
    pub aggregation: Option<Aggregation>,
}

impl Span {
    pub(crate) const fn finish(&self) -> u64 {
        self.row.start + self.row.duration
    }

    pub(crate) fn tag(&self, key: &str) -> Option<&Tag> {
        self.row.tags.iter().find(|tag| tag.key.as_ref() == key)
    }
}

#[derive(Default)]
pub(crate) struct HeaderTags {
    pub method: Option<SharedString>,
    pub status: Option<SharedString>,
    pub route: Option<(SharedString, SharedString)>,
}

#[derive(Default)]
pub(crate) struct Aggregation {
    pub summary: bool,
    pub outlier: bool,
    pub count: Option<f64>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub median: Option<f64>,
}

impl Trace {
    pub fn from_spans(rows: impl IntoIterator<Item = SpanRow>) -> Self {
        let mut rows: Vec<SpanRow> = rows.into_iter().filter(|row| row.start != 0).collect();
        if rows.is_empty() {
            return Self::default();
        }
        rename_duplicates(&mut rows);
        for row in &mut rows {
            let duplicates = dedupe(&mut row.tags);
            row.warnings.extend(duplicates);
            dedupe(&mut row.service_tags);
            row.tags.sort_by_cached_key(|tag| tag.key.to_lowercase());
            row.service_tags
                .sort_by_cached_key(|tag| tag.key.to_lowercase());
        }

        let ids: HashMap<SharedString, usize> = rows
            .iter()
            .enumerate()
            .map(|(index, row)| (row.span_id.clone(), index))
            .collect();
        let parent_of = |row: &SpanRow| -> Option<(usize, bool)> {
            match &row.parent {
                Some(parent) => ids.get(parent).map(|index| (*index, false)),
                None => row
                    .references
                    .first()
                    .and_then(|reference| ids.get(&reference.span_id))
                    .map(|index| (*index, true)),
            }
        };
        let mut children: Vec<Vec<usize>> = vec![Vec::new(); rows.len()];
        let mut roots = Vec::new();
        let mut follows = vec![false; rows.len()];
        for (index, row) in rows.iter().enumerate() {
            match parent_of(row).filter(|(parent, _)| *parent != index) {
                Some((parent, reference)) => {
                    children[parent].push(index);
                    follows[index] = reference;
                }
                None => roots.push(index),
            }
        }
        let by_start = |list: &mut Vec<usize>| list.sort_by_key(|index| rows[*index].start);
        by_start(&mut roots);
        children.iter_mut().for_each(by_start);

        let mut order = Vec::with_capacity(rows.len());
        let mut depths = vec![0u32; rows.len()];
        let mut stack: Vec<(usize, u32)> = roots.iter().rev().map(|root| (*root, 0)).collect();
        let mut seen = vec![false; rows.len()];
        while let Some((index, depth)) = stack.pop() {
            if std::mem::replace(&mut seen[index], true) {
                continue;
            }
            depths[index] = depth;
            order.push(index);
            stack.extend(
                children[index]
                    .iter()
                    .rev()
                    .map(|child| (*child, depth + 1)),
            );
        }
        let mut position = vec![0u32; rows.len()];
        for (at, index) in order.iter().enumerate() {
            position[*index] = at as u32;
        }

        let start = rows.iter().map(|row| row.start).min().unwrap_or(0);
        let finish = rows
            .iter()
            .map(|row| row.start + row.duration)
            .max()
            .unwrap_or(start);
        let id = rows
            .iter()
            .find(|row| !row.trace_id.is_empty())
            .map(|row| row.trace_id.clone())
            .unwrap_or_default();
        let parents: Vec<Option<u32>> = order
            .iter()
            .map(|index| parent_of(&rows[*index]).map(|(parent, _)| position[parent]))
            .collect();
        let child_counts: Vec<u32> = order
            .iter()
            .map(|index| children[*index].len() as u32)
            .collect();
        let follows: Vec<bool> = order.iter().map(|index| follows[*index]).collect();

        let mut slots: Vec<Option<SpanRow>> = rows.into_iter().map(Some).collect();
        let mut services: Vec<Service> = Vec::new();
        let mut spans: Vec<Span> = Vec::with_capacity(order.len());
        for (at, index) in order.iter().enumerate() {
            let Some(row) = slots[*index].take() else {
                continue;
            };
            let name = display(&row);
            let service = services
                .iter()
                .position(|service| service.name == name)
                .unwrap_or_else(|| {
                    services.push(Service { name, spans: 0 });
                    services.len() - 1
                });
            services[service].spans += 1;
            let error = row.status == SpanStatus::Error
                || row
                    .tags
                    .iter()
                    .any(|tag| tag.key.as_ref() == "error" && tag.value.is_true());
            let aggregation = aggregation(&row.tags);
            spans.push(Span {
                depth: depths[*index],
                parent: parents[at],
                follows: follows[at],
                end: at as u32 + 1,
                children: child_counts[at],
                service: service as u32,
                error,
                aggregation,
                row,
            });
        }
        for at in (0..spans.len()).rev() {
            if let Some(parent) = spans[at].parent {
                let end = spans[at].end;
                let parent = &mut spans[parent as usize];
                parent.end = parent.end.max(end);
            }
        }
        let index = spans
            .iter()
            .enumerate()
            .map(|(at, span)| (span.row.span_id.clone(), at as u32))
            .collect();
        let mut trace = Self {
            id,
            spans,
            index,
            start,
            duration: finish - start,
            root: None,
            services,
            header: HeaderTags::default(),
        };
        trace.root = trace.find_root();
        trace.header = trace.find_header();
        trace
    }

    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.spans.len()
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.spans.is_empty()
    }

    #[must_use]
    pub const fn duration(&self) -> u64 {
        self.duration
    }

    #[must_use]
    pub fn name(&self) -> SharedString {
        self.root.map_or_else(SharedString::default, |root| {
            let span = &self.spans[root as usize];
            format!(
                "{}: {}",
                self.services[span.service as usize].name, span.row.operation
            )
            .into()
        })
    }

    pub(crate) fn service(&self, span: &Span) -> &SharedString {
        &self.services[span.service as usize].name
    }

    pub(crate) fn children(&self, index: u32) -> impl Iterator<Item = u32> + '_ {
        let span = &self.spans[index as usize];
        let mut next = index + 1;
        std::iter::from_fn(move || {
            (next < span.end).then(|| {
                let child = next;
                next = self.spans[child as usize].end;
                child
            })
        })
    }

    pub(crate) fn ancestors(&self, index: u32) -> impl Iterator<Item = u32> + '_ {
        std::iter::successors(self.spans[index as usize].parent, |parent| {
            self.spans[*parent as usize].parent
        })
    }

    fn find_root(&self) -> Option<u32> {
        let linked = |span: &Span| {
            span.parent.is_some()
                || span.row.references.iter().any(|reference| {
                    reference.trace_id == self.id && self.index.contains_key(&reference.span_id)
                })
        };
        self.spans
            .iter()
            .enumerate()
            .filter(|(_, span)| !linked(span))
            .min_by_key(|(_, span)| (span.row.references.len(), span.row.start))
            .or_else(|| self.spans.first().map(|span| (0, span)))
            .map(|(index, _)| index as u32)
    }

    fn find_header(&self) -> HeaderTags {
        const OTEL: [&str; 3] = [
            "http.request.method",
            "http.response.status_code",
            "http.route",
        ];
        const LEGACY: [&str; 2] = ["http.method", "http.status_code"];
        const ROUTES: [&str; 3] = ["http.url", "http.target", "http.path"];
        let text = |tag: &Tag| SharedString::from(tag.value.to_string());
        let found = |keys: &[&str]| {
            self.spans
                .iter()
                .find(|span| keys.iter().any(|key| span.tag(key).is_some()))
        };
        if let Some(span) = found(&OTEL) {
            return HeaderTags {
                method: span.tag(OTEL[0]).map(text),
                status: span.tag(OTEL[1]).map(text),
                route: span.tag(OTEL[2]).map(|tag| (tag.key.clone(), text(tag))),
            };
        }
        let legacy: Vec<&str> = LEGACY.iter().chain(&ROUTES).copied().collect();
        found(&legacy).map_or_else(HeaderTags::default, |span| HeaderTags {
            method: span.tag(LEGACY[0]).map(text),
            status: span.tag(LEGACY[1]).map(text),
            route: ROUTES
                .iter()
                .find_map(|key| span.tag(key))
                .map(|tag| (tag.key.clone(), text(tag))),
        })
    }
}

fn display(row: &SpanRow) -> SharedString {
    row.service_namespace.as_ref().map_or_else(
        || row.service.clone(),
        |namespace| format!("{namespace}/{}", row.service).into(),
    )
}

fn rename_duplicates(rows: &mut [SpanRow]) {
    let mut seen: HashMap<SharedString, usize> = HashMap::new();
    for row in rows {
        let count = seen.entry(row.span_id.clone()).or_insert(0);
        if *count > 0 {
            row.span_id = format!("{}_{count}", row.span_id).into();
        }
        *count += 1;
    }
}

fn dedupe(tags: &mut Vec<Tag>) -> Vec<SharedString> {
    let mut warnings = Vec::new();
    let mut kept: Vec<Tag> = Vec::with_capacity(tags.len());
    for tag in tags.drain(..) {
        if kept
            .iter()
            .any(|other| other.key == tag.key && other.value == tag.value)
        {
            warnings.push(format!("Duplicate tag \"{}:{}\"", tag.key, tag.value).into());
        } else {
            kept.push(tag);
        }
    }
    *tags = kept;
    warnings
}

fn aggregation(tags: &[Tag]) -> Option<Aggregation> {
    let get = |key: &str| tags.iter().find(|tag| tag.key.as_ref() == key);
    let number = |key: &str| get(key).and_then(|tag| tag.value.number());
    let flag = |key: &str| {
        get(key).is_some_and(|tag| matches!(tag.value, super::data::TagValue::Bool(true)))
    };
    let aggregation = Aggregation {
        summary: flag("aggregation.is_summary"),
        outlier: flag("aggregation.is_preserved_outlier"),
        count: number("aggregation.span_count"),
        min: number("aggregation.duration_min_ns"),
        max: number("aggregation.duration_max_ns"),
        median: number("aggregation.duration_median_ns"),
    };
    (aggregation.summary || aggregation.outlier || aggregation.count.is_some())
        .then_some(aggregation)
}

/// What `<Traces>` shows: a [`Trace`] (kept until one with another id, span count, start or
/// duration comes) or a signal holding one.
pub trait TraceData: 'static {
    /// The trace in a signal: `slot` (the chart's own) for an owned trace, else the given signal.
    fn into_signal(self, slot: Signal<Trace>) -> Signal<Trace>;

    /// Changes whenever the trace does (called while rendering).
    #[track_caller]
    fn revision(&self, slot: Signal<Trace>) -> Revision;
}

impl Trace {
    /// What tells one trace from another without reading every span.
    fn fingerprint(&self) -> u64 {
        let mut state = DefaultHasher::new();
        (&self.id, self.spans.len(), self.start, self.duration).hash(&mut state);
        state.finish()
    }
}

impl TraceData for Trace {
    fn into_signal(self, slot: Signal<Trace>) -> Signal<Trace> {
        if slot.with(Self::fingerprint) != self.fingerprint() {
            slot.update_untracked(|trace| *trace = self);
        }
        slot
    }

    fn revision(&self, _slot: Signal<Trace>) -> Revision {
        Revision { source: self.fingerprint(), version: 0 }
    }
}

impl TraceData for Signal<Trace> {
    fn into_signal(self, _slot: Signal<Trace>) -> Signal<Trace> {
        self
    }

    #[track_caller]
    fn revision(&self, _slot: Signal<Trace>) -> Revision {
        Revision::of(*self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn trace() -> Trace {
        Trace::from_spans([
            SpanRow::new("c", "db", "query")
                .parent("a")
                .start(30)
                .duration(10),
            SpanRow::new("a", "api", "GET /").start(10).duration(100),
            SpanRow::new("b", "cache", "get")
                .parent("a")
                .start(20)
                .duration(5),
            SpanRow::new("b", "cache", "get")
                .parent("a")
                .start(25)
                .duration(5),
            SpanRow::new("z", "late", "skip").start(0).duration(5),
        ])
    }

    #[test]
    fn orders_spans_as_a_tree_by_start_time() {
        let trace = trace();
        let ids: Vec<&str> = trace
            .spans
            .iter()
            .map(|span| span.row.span_id.as_ref())
            .collect();
        assert_eq!(ids, ["a", "b", "b_1", "c"]);
        assert_eq!(trace.spans[0].end, 4);
        assert_eq!(trace.children(0).count(), 3);
        assert_eq!(trace.duration, 100);
        assert_eq!(trace.name(), "api: GET /");
    }

    #[test]
    fn warns_about_duplicate_tags() {
        let trace = Trace::from_spans([SpanRow::new("a", "api", "op")
            .start(1)
            .tag("k", "v")
            .tag("k", "v")]);
        assert_eq!(trace.spans[0].row.tags.len(), 1);
        assert_eq!(trace.spans[0].row.warnings[0], "Duplicate tag \"k:v\"");
    }
}
