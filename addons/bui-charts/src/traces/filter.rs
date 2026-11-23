use blenderui::SharedString;

use super::data::Tag;
use super::trace::{Span, Trace};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FilterOp {
    Eq,
    NotEq,
    Contains,
    NotContains,
    Gt,
    Ge,
    Lt,
    Le,
}

impl FilterOp {
    pub(crate) const TEXT: [Self; 4] = [Self::Eq, Self::NotEq, Self::Contains, Self::NotContains];
    pub(crate) const DURATION: [Self; 5] = [Self::Eq, Self::Ge, Self::Le, Self::Gt, Self::Lt];

    pub(crate) const fn symbol(self) -> &'static str {
        match self {
            Self::Eq => "=",
            Self::NotEq => "!=",
            Self::Contains => "=~",
            Self::NotContains => "!~",
            Self::Gt => ">",
            Self::Ge => ">=",
            Self::Lt => "<",
            Self::Le => "<=",
        }
    }

    const fn negated(self) -> bool {
        matches!(self, Self::NotEq | Self::NotContains)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum FilterKey {
    Service,
    SpanName,
    Duration,
    Text,
    Tag(SharedString),
}

impl FilterKey {
    pub(crate) const SERVICE: &'static str = "service.name";
    pub(crate) const SPAN_NAME: &'static str = "span.name";
    pub(crate) const DURATION: &'static str = "duration";
    pub(crate) const TEXT: &'static str = "Text search";

    pub(crate) fn label(&self) -> SharedString {
        match self {
            Self::Service => Self::SERVICE.into(),
            Self::SpanName => Self::SPAN_NAME.into(),
            Self::Duration => Self::DURATION.into(),
            Self::Text => Self::TEXT.into(),
            Self::Tag(key) => key.clone(),
        }
    }

    pub(crate) fn parse(label: &SharedString) -> Self {
        match label.as_ref() {
            Self::SERVICE => Self::Service,
            Self::SPAN_NAME => Self::SpanName,
            Self::DURATION => Self::Duration,
            Self::TEXT => Self::Text,
            _ => Self::Tag(label.clone()),
        }
    }

    pub(crate) fn ops(&self) -> &'static [FilterOp] {
        match self {
            Self::Duration => &FilterOp::DURATION,
            Self::Text => &FilterOp::TEXT[2..3],
            _ => &FilterOp::TEXT,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Filter {
    pub key: FilterKey,
    pub op: FilterOp,
    pub value: SharedString,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct SpanFilters {
    pub(crate) filters: Vec<Filter>,
    pub(crate) matches_only: bool,
    pub(crate) critical_path_only: bool,
}

impl SpanFilters {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn filter(mut self, key: FilterKey, op: FilterOp, value: impl Into<SharedString>) -> Self {
        self.filters.push(Filter {
            key,
            op,
            value: value.into(),
        });
        self
    }

    #[must_use]
    pub fn service(self, op: FilterOp, value: impl Into<SharedString>) -> Self {
        self.filter(FilterKey::Service, op, value)
    }

    #[must_use]
    pub fn span_name(self, op: FilterOp, value: impl Into<SharedString>) -> Self {
        self.filter(FilterKey::SpanName, op, value)
    }

    #[must_use]
    pub fn duration(self, op: FilterOp, value: impl Into<SharedString>) -> Self {
        self.filter(FilterKey::Duration, op, value)
    }

    #[must_use]
    pub fn tag(
        self,
        key: impl Into<SharedString>,
        op: FilterOp,
        value: impl Into<SharedString>,
    ) -> Self {
        self.filter(FilterKey::Tag(key.into()), op, value)
    }

    #[must_use]
    pub fn text(self, query: impl Into<SharedString>) -> Self {
        self.filter(FilterKey::Text, FilterOp::Contains, query)
    }

    #[must_use]
    pub const fn matches_only(mut self, matches_only: bool) -> Self {
        self.matches_only = matches_only;
        self
    }

    #[must_use]
    pub const fn critical_path_only(mut self, critical_path_only: bool) -> Self {
        self.critical_path_only = critical_path_only;
        self
    }

    pub(crate) fn active(&self) -> impl Iterator<Item = &Filter> {
        self.filters.iter().filter(|filter| {
            !filter.value.is_empty()
                && !matches!(&filter.key, FilterKey::Tag(key) if key.is_empty())
        })
    }

    pub(crate) fn has(&self, key: &FilterKey, op: FilterOp) -> bool {
        self.filters
            .iter()
            .any(|filter| &filter.key == key && filter.op == op)
    }
}

pub(crate) fn parse_duration(text: &str) -> Option<f64> {
    const UNITS: [(&str, f64); 7] = [
        ("ns", 0.001),
        ("us", 1.),
        ("µs", 1.),
        ("ms", 1e3),
        ("s", 1e6),
        ("m", 6e7),
        ("h", 3.6e9),
    ];
    let text = text.trim();
    let (number, scale) = UNITS
        .iter()
        .find_map(|(unit, scale)| text.find(unit).map(|at| (&text[..at], *scale)))
        .unwrap_or((text, 1.));
    let value = number.trim().parse::<f64>().ok()? * scale;
    (value.is_finite() && value != 0.).then_some(value)
}

fn compare(op: FilterOp, actual: &str, wanted: &str) -> bool {
    let hit = match op {
        FilterOp::Eq | FilterOp::NotEq => actual == wanted,
        FilterOp::Contains | FilterOp::NotContains => actual.contains(wanted),
        _ => false,
    };
    hit != op.negated()
}

fn tag_hit(tags: &[Tag], key: &str, op: FilterOp, wanted: &str) -> bool {
    tags.iter().any(|tag| match op {
        FilterOp::Contains | FilterOp::NotContains => {
            tag.key.contains(key) && tag.value.to_string().contains(wanted)
        }
        _ => tag.key.as_ref() == key && tag.value.to_string() == wanted,
    })
}

fn pseudo(span: &Span, key: &str) -> Option<SharedString> {
    let row = &span.row;
    match key {
        "kind" => row.kind.map(|kind| kind.label().into()),
        "status" => Some(row.status.label().into()),
        "status.message" => row.status_message.clone(),
        "library.name" => row.library.as_ref().map(|(name, _)| name.clone()),
        "library.version" => row
            .library
            .as_ref()
            .and_then(|(_, version)| version.clone()),
        "trace.state" => row.trace_state.clone(),
        "id" => Some(row.span_id.clone()),
        _ => None,
    }
}

fn tag_matches(span: &Span, key: &str, op: FilterOp, wanted: &str) -> bool {
    let row = &span.row;
    let base = match op {
        FilterOp::NotEq => FilterOp::Eq,
        FilterOp::NotContains => FilterOp::Contains,
        other => other,
    };
    let hit = tag_hit(&row.tags, key, base, wanted)
        || tag_hit(&row.service_tags, key, base, wanted)
        || row
            .logs
            .iter()
            .any(|log| tag_hit(&log.fields, key, base, wanted))
        || pseudo(span, key).is_some_and(|value| match base {
            FilterOp::Contains => wanted.contains(value.as_ref()),
            _ => value.as_ref() == wanted,
        });
    hit != op.negated()
}

fn text_matches(span: &Span, terms: &[String]) -> bool {
    let row = &span.row;
    let found = |text: &str| {
        let text = text.to_lowercase();
        terms.iter().any(|term| text.contains(term.as_str()))
    };
    let tags = |tags: &[Tag]| {
        tags.iter()
            .any(|tag| found(&tag.key) || found(&tag.value.to_string()))
    };
    terms.iter().any(|term| row.span_id.as_ref() == term)
        || found(&row.operation)
        || found(&row.service)
        || tags(&row.tags)
        || tags(&row.service_tags)
        || row.kind.is_some_and(|kind| found(kind.label()))
        || found(row.status.label())
        || row.status_message.as_ref().is_some_and(|text| found(text))
        || row.library.as_ref().is_some_and(|(name, version)| {
            found(name) || version.as_ref().is_some_and(|version| found(version))
        })
        || row.trace_state.as_ref().is_some_and(|text| found(text))
        || row
            .logs
            .iter()
            .any(|log| log.name.as_ref().is_some_and(|name| found(name)) || tags(&log.fields))
}

fn matches_filter(span: &Span, filter: &Filter, terms: &[String]) -> bool {
    let value = filter.value.as_ref();
    match &filter.key {
        FilterKey::Service => compare(filter.op, &span.row.service, value),
        FilterKey::SpanName => compare(filter.op, &span.row.operation, value),
        FilterKey::Duration => parse_duration(value).is_some_and(|limit| {
            let actual = span.row.duration as f64;
            match filter.op {
                FilterOp::Eq => (actual - limit).abs() < f64::EPSILON,
                FilterOp::Ge => actual >= limit,
                FilterOp::Le => actual <= limit,
                FilterOp::Gt => actual > limit,
                FilterOp::Lt => actual < limit,
                _ => false,
            }
        }),
        FilterKey::Text => text_matches(span, terms),
        FilterKey::Tag(key) => tag_matches(span, key, filter.op, value),
    }
}

pub(crate) fn matches(
    trace: &Trace,
    filters: &SpanFilters,
    critical: &[bool],
) -> Option<Vec<bool>> {
    let active: Vec<&Filter> = filters.active().collect();
    if active.is_empty() && !filters.critical_path_only {
        return None;
    }
    let terms: Vec<String> = active
        .iter()
        .filter(|filter| filter.key == FilterKey::Text)
        .flat_map(|filter| filter.value.split_whitespace().map(str::to_lowercase))
        .collect();
    Some(
        trace
            .spans
            .iter()
            .enumerate()
            .map(|(index, span)| {
                active
                    .iter()
                    .all(|filter| matches_filter(span, filter, &terms))
                    && (!filters.critical_path_only
                        || critical.get(index).copied().unwrap_or(false))
            })
            .collect(),
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Section {
    pub span: u32,
    pub start: u64,
    pub end: u64,
}

pub(crate) fn critical_path(trace: &Trace) -> Vec<Section> {
    let count = trace.spans.len();
    if count == 0 {
        return Vec::new();
    }
    let mut valid = vec![true; count];
    let mut bounds: Vec<(u64, u64)> = trace
        .spans
        .iter()
        .map(|span| (span.row.start, span.finish()))
        .collect();
    let root_end = trace.spans[0].end as usize;
    for index in 0..count {
        let span = &trace.spans[index];
        let inside = index < root_end;
        let parent_ok = span.parent.is_none_or(|parent| valid[parent as usize]);
        if !inside || span.follows || !parent_ok {
            valid[index] = false;
            continue;
        }
        if let Some(parent) = span.parent {
            let (low, high) = bounds[parent as usize];
            let (start, end) = bounds[index];
            if end < low || start > high {
                valid[index] = false;
                continue;
            }
            bounds[index] = (start.max(low), end.min(high));
        }
    }
    let last_child = |index: u32, before: Option<u64>| -> Option<u32> {
        trace
            .children(index)
            .filter(|child| valid[*child as usize])
            .filter(|child| before.is_none_or(|limit| bounds[*child as usize].1 < limit))
            .max_by_key(|child| bounds[*child as usize].1)
    };
    let mut sections = Vec::new();
    let mut current = Some((0u32, None::<u64>));
    let mut guard = count * 4 + 4;
    while let Some((index, returning)) = current {
        guard -= 1;
        if guard == 0 {
            break;
        }
        let (start, end) = bounds[index as usize];
        let until = returning.unwrap_or(end);
        current = if let Some(child) = last_child(index, returning) {
            let from = bounds[child as usize].1;
            if from < until {
                sections.push(Section {
                    span: index,
                    start: from,
                    end: until,
                });
            }
            Some((child, None))
        } else {
            if start < until {
                sections.push(Section {
                    span: index,
                    start,
                    end: until,
                });
            }
            trace.spans[index as usize]
                .parent
                .filter(|parent| valid[*parent as usize])
                .map(|parent| (parent, Some(start)))
        };
    }
    sections.sort_by_key(|section| (section.span, section.start));
    sections
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traces::data::{SpanRow, SpanStatus};

    fn trace() -> Trace {
        Trace::from_spans([
            SpanRow::new("a", "api", "GET /checkout")
                .start(100)
                .duration(1000),
            SpanRow::new("b", "cart", "load")
                .parent("a")
                .start(200)
                .duration(300)
                .tag("http.status_code", 200),
            SpanRow::new("c", "db", "SELECT")
                .parent("a")
                .start(600)
                .duration(400)
                .status(SpanStatus::Error),
        ])
    }

    #[test]
    fn parses_durations_to_micros() {
        assert_eq!(parse_duration("120ms"), Some(120_000.));
        assert_eq!(parse_duration("2s"), Some(2_000_000.));
        assert_eq!(parse_duration("500ns"), Some(0.5));
        assert_eq!(parse_duration("15"), Some(15.));
        assert_eq!(parse_duration("0ms"), None);
    }

    #[test]
    fn filters_combine_with_and() {
        let trace = trace();
        let only = |filters: SpanFilters| matches(&trace, &filters, &[]).unwrap();
        assert_eq!(
            only(SpanFilters::new().service(FilterOp::Contains, "ca")),
            [false, true, false]
        );
        assert_eq!(
            only(SpanFilters::new().duration(FilterOp::Ge, "350us")),
            [true, false, true]
        );
        assert_eq!(
            only(SpanFilters::new().tag("status", FilterOp::Eq, "error")),
            [false, false, true]
        );
        assert_eq!(
            only(SpanFilters::new().tag("http.status_code", FilterOp::Eq, "200")),
            [false, true, false]
        );
        assert_eq!(
            only(SpanFilters::new().text("select checkout")),
            [true, false, true]
        );
    }

    #[test]
    fn critical_path_walks_back_from_the_last_child() {
        let sections = critical_path(&trace());
        assert_eq!(
            sections,
            [
                Section {
                    span: 0,
                    start: 100,
                    end: 200
                },
                Section {
                    span: 0,
                    start: 500,
                    end: 600
                },
                Section {
                    span: 0,
                    start: 1000,
                    end: 1100
                },
                Section {
                    span: 1,
                    start: 200,
                    end: 500
                },
                Section {
                    span: 2,
                    start: 600,
                    end: 1000
                },
            ]
        );
    }
}
