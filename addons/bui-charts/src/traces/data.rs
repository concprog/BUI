use std::fmt;

use blenderui::{Hsla, SharedString};

#[derive(Clone, Debug, PartialEq)]
pub enum TagValue {
    Text(SharedString),
    Int(i64),
    Float(f64),
    Bool(bool),
}

impl fmt::Display for TagValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Text(text) => f.write_str(text),
            Self::Int(value) => write!(f, "{value}"),
            Self::Float(value) => write!(f, "{value}"),
            Self::Bool(value) => write!(f, "{value}"),
        }
    }
}

impl TagValue {
    pub(crate) fn is_true(&self) -> bool {
        match self {
            Self::Bool(value) => *value,
            Self::Text(text) => text.as_ref() == "true",
            _ => false,
        }
    }

    pub(crate) fn number(&self) -> Option<f64> {
        match self {
            Self::Int(value) => Some(*value as f64),
            Self::Float(value) => Some(*value),
            Self::Text(text) => text.parse().ok().filter(|value: &f64| value.is_finite()),
            Self::Bool(_) => None,
        }
    }
}

macro_rules! tag_value {
    ($($from:ty => $variant:ident($convert:expr)),* $(,)?) => {$(
        impl From<$from> for TagValue {
            fn from(value: $from) -> Self {
                Self::$variant($convert(value))
            }
        }
    )*};
}

tag_value! {
    &'static str => Text(SharedString::Borrowed),
    String => Text(SharedString::from),
    SharedString => Text(std::convert::identity),
    bool => Bool(std::convert::identity),
    i64 => Int(std::convert::identity),
    i32 => Int(i64::from),
    u32 => Int(i64::from),
    f64 => Float(std::convert::identity),
    f32 => Float(f64::from),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Tag {
    pub key: SharedString,
    pub value: TagValue,
}

impl Tag {
    pub fn new(key: impl Into<SharedString>, value: impl Into<TagValue>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SpanLog {
    pub(crate) timestamp: u64,
    pub(crate) name: Option<SharedString>,
    pub(crate) fields: Vec<Tag>,
}

impl SpanLog {
    #[must_use]
    pub fn new(timestamp: u64) -> Self {
        Self {
            timestamp,
            ..Self::default()
        }
    }

    #[must_use]
    pub fn name(mut self, name: impl Into<SharedString>) -> Self {
        self.name = Some(name.into());
        self
    }

    #[must_use]
    pub fn field(mut self, key: impl Into<SharedString>, value: impl Into<TagValue>) -> Self {
        self.fields.push(Tag::new(key, value));
        self
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SpanReference {
    pub(crate) trace_id: SharedString,
    pub(crate) span_id: SharedString,
    pub(crate) tags: Vec<Tag>,
}

impl SpanReference {
    pub fn new(trace_id: impl Into<SharedString>, span_id: impl Into<SharedString>) -> Self {
        Self {
            trace_id: trace_id.into(),
            span_id: span_id.into(),
            tags: Vec::new(),
        }
    }

    #[must_use]
    pub fn tag(mut self, key: impl Into<SharedString>, value: impl Into<TagValue>) -> Self {
        self.tags.push(Tag::new(key, value));
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SpanKind {
    Internal,
    Server,
    Client,
    Producer,
    Consumer,
}

impl SpanKind {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Internal => "internal",
            Self::Server => "server",
            Self::Client => "client",
            Self::Producer => "producer",
            Self::Consumer => "consumer",
        }
    }

    pub(crate) fn parse(text: &str) -> Option<Self> {
        let text = text.to_ascii_lowercase();
        let text = text.trim_start_matches("span_kind_");
        [
            Self::Internal,
            Self::Server,
            Self::Client,
            Self::Producer,
            Self::Consumer,
        ]
        .into_iter()
        .find(|kind| kind.label() == text)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SpanStatus {
    #[default]
    Unset,
    Ok,
    Error,
}

impl SpanStatus {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Unset => "unset",
            Self::Ok => "ok",
            Self::Error => "error",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SpanRow {
    pub(crate) span_id: SharedString,
    pub(crate) parent: Option<SharedString>,
    pub(crate) trace_id: SharedString,
    pub(crate) operation: SharedString,
    pub(crate) service: SharedString,
    pub(crate) service_namespace: Option<SharedString>,
    pub(crate) service_tags: Vec<Tag>,
    pub(crate) start: u64,
    pub(crate) duration: u64,
    pub(crate) logs: Vec<SpanLog>,
    pub(crate) references: Vec<SpanReference>,
    pub(crate) tags: Vec<Tag>,
    pub(crate) kind: Option<SpanKind>,
    pub(crate) status: SpanStatus,
    pub(crate) status_message: Option<SharedString>,
    pub(crate) library: Option<(SharedString, Option<SharedString>)>,
    pub(crate) trace_state: Option<SharedString>,
    pub(crate) warnings: Vec<SharedString>,
    pub(crate) stack_traces: Vec<SharedString>,
    pub(crate) error_color: Option<Hsla>,
}

impl SpanRow {
    pub fn new(
        span_id: impl Into<SharedString>,
        service: impl Into<SharedString>,
        operation: impl Into<SharedString>,
    ) -> Self {
        Self {
            span_id: span_id.into(),
            service: service.into(),
            operation: operation.into(),
            ..Self::default()
        }
    }

    #[must_use]
    pub fn parent(mut self, span_id: impl Into<SharedString>) -> Self {
        self.parent = Some(span_id.into());
        self
    }

    #[must_use]
    pub fn trace_id(mut self, trace_id: impl Into<SharedString>) -> Self {
        self.trace_id = trace_id.into();
        self
    }

    #[must_use]
    pub fn service_namespace(mut self, namespace: impl Into<SharedString>) -> Self {
        self.service_namespace = Some(namespace.into());
        self
    }

    #[must_use]
    pub fn service_tag(mut self, key: impl Into<SharedString>, value: impl Into<TagValue>) -> Self {
        self.service_tags.push(Tag::new(key, value));
        self
    }

    #[must_use]
    pub const fn start(mut self, micros: u64) -> Self {
        self.start = micros;
        self
    }

    #[must_use]
    pub const fn duration(mut self, micros: u64) -> Self {
        self.duration = micros;
        self
    }

    #[must_use]
    pub fn tag(mut self, key: impl Into<SharedString>, value: impl Into<TagValue>) -> Self {
        self.tags.push(Tag::new(key, value));
        self
    }

    #[must_use]
    pub fn log(mut self, log: SpanLog) -> Self {
        self.logs.push(log);
        self
    }

    #[must_use]
    pub fn reference(mut self, reference: SpanReference) -> Self {
        self.references.push(reference);
        self
    }

    #[must_use]
    pub const fn kind(mut self, kind: SpanKind) -> Self {
        self.kind = Some(kind);
        self
    }

    #[must_use]
    pub const fn status(mut self, status: SpanStatus) -> Self {
        self.status = status;
        self
    }

    #[must_use]
    pub fn status_message(mut self, message: impl Into<SharedString>) -> Self {
        self.status_message = Some(message.into());
        self
    }

    #[must_use]
    pub fn library(mut self, name: impl Into<SharedString>, version: Option<SharedString>) -> Self {
        self.library = Some((name.into(), version));
        self
    }

    #[must_use]
    pub fn trace_state(mut self, state: impl Into<SharedString>) -> Self {
        self.trace_state = Some(state.into());
        self
    }

    #[must_use]
    pub fn warning(mut self, warning: impl Into<SharedString>) -> Self {
        self.warnings.push(warning.into());
        self
    }

    #[must_use]
    pub fn stack_trace(mut self, trace: impl Into<SharedString>) -> Self {
        self.stack_traces.push(trace.into());
        self
    }

    #[must_use]
    pub fn error_color(mut self, color: impl Into<Hsla>) -> Self {
        self.error_color = Some(color.into());
        self
    }
}
