use std::collections::HashMap;

use blenderui::SharedString;
use serde_json::Value;

use super::data::{SpanKind, SpanLog, SpanReference, SpanRow, SpanStatus, Tag, TagValue};
use super::trace::Trace;

fn text(value: &Value) -> SharedString {
    match value {
        Value::String(text) => text.clone().into(),
        Value::Null => SharedString::default(),
        other => other.to_string().into(),
    }
}

fn field<'a>(value: &'a Value, keys: &[&str]) -> Option<&'a Value> {
    keys.iter().find_map(|key| value.get(key))
}

fn items<'a>(value: &'a Value, keys: &[&str]) -> &'a [Value] {
    field(value, keys)
        .and_then(Value::as_array)
        .map_or(&[], Vec::as_slice)
}

fn number(value: Option<&Value>) -> Option<u64> {
    match value? {
        Value::Number(number) => number
            .as_u64()
            .or_else(|| number.as_f64().map(|value| value as u64)),
        Value::String(text) => text.parse().ok(),
        _ => None,
    }
}

fn plain(value: &Value) -> TagValue {
    match value {
        Value::Bool(flag) => TagValue::Bool(*flag),
        Value::Number(number) => number.as_i64().map_or_else(
            || TagValue::Float(number.as_f64().unwrap_or(f64::NAN)),
            TagValue::Int,
        ),
        other => TagValue::Text(text(other)),
    }
}

fn jaeger_tags(value: &Value, key: &str) -> Vec<Tag> {
    items(value, &[key])
        .iter()
        .map(|tag| Tag {
            key: text(tag.get("key").unwrap_or(&Value::Null)),
            value: plain(tag.get("value").unwrap_or(&Value::Null)),
        })
        .collect()
}

impl Trace {
    pub fn from_jaeger_json(json: &str) -> Result<Self, serde_json::Error> {
        let root: Value = serde_json::from_str(json)?;
        let trace = match &root {
            Value::Array(traces) => traces.first().unwrap_or(&Value::Null),
            object => object
                .get("data")
                .and_then(|data| {
                    data.as_array()
                        .and_then(|traces| traces.first())
                        .or(Some(data))
                })
                .unwrap_or(object),
        };
        let processes = trace.get("processes");
        let process = |id: &Value| processes.and_then(|processes| processes.get(id.as_str()?));
        let rows = items(trace, &["spans"]).iter().map(|span| {
            let mut row = SpanRow::new(
                text(field(span, &["spanID", "spanId"]).unwrap_or(&Value::Null)),
                "",
                text(span.get("operationName").unwrap_or(&Value::Null)),
            );
            row.trace_id = text(field(span, &["traceID", "traceId"]).unwrap_or(&Value::Null));
            row.start = number(span.get("startTime")).unwrap_or(0);
            row.duration = number(span.get("duration")).unwrap_or(0);
            row.tags = jaeger_tags(span, "tags");
            if let Some(process) = span
                .get("processID")
                .and_then(process)
                .or_else(|| span.get("process"))
            {
                row.service = text(process.get("serviceName").unwrap_or(&Value::Null));
                row.service_tags = jaeger_tags(process, "tags");
            }
            for reference in items(span, &["references"]) {
                let id = text(field(reference, &["spanID", "spanId"]).unwrap_or(&Value::Null));
                let child =
                    reference.get("refType").and_then(Value::as_str) != Some("FOLLOWS_FROM");
                if child && row.parent.is_none() {
                    row.parent = Some(id);
                } else {
                    row.references.push(SpanReference {
                        trace_id: text(
                            field(reference, &["traceID", "traceId"]).unwrap_or(&Value::Null),
                        ),
                        span_id: id,
                        tags: Vec::new(),
                    });
                }
            }
            row.logs = items(span, &["logs"])
                .iter()
                .map(|log| SpanLog {
                    timestamp: number(log.get("timestamp")).unwrap_or(0),
                    name: None,
                    fields: jaeger_tags(log, "fields"),
                })
                .collect();
            row.warnings = items(span, &["warnings"]).iter().map(text).collect();
            let tag = |key: &str| {
                row.tags
                    .iter()
                    .find(|tag| tag.key.as_ref() == key)
                    .map(|tag| tag.value.to_string())
            };
            row.kind = tag("span.kind").and_then(|kind| SpanKind::parse(&kind));
            if tag("otel.status_code").as_deref() == Some("ERROR") {
                row.status = SpanStatus::Error;
            }
            row.status_message = tag("otel.status_description").map(Into::into);
            row.library = tag("otel.library.name")
                .or_else(|| tag("otel.scope.name"))
                .map(|name| {
                    let version = tag("otel.library.version").or_else(|| tag("otel.scope.version"));
                    (name.into(), version.map(Into::into))
                });
            row
        });
        Ok(Self::from_spans(rows.collect::<Vec<_>>()))
    }

    pub fn from_otlp_json(json: &str) -> Result<Self, serde_json::Error> {
        let root: Value = serde_json::from_str(json)?;
        let mut rows = Vec::new();
        for resource in items(&root, &["resourceSpans", "resource_spans"]) {
            let resource_tags = otlp_attributes(resource.get("resource").unwrap_or(&Value::Null));
            let lookup = |key: &str| {
                resource_tags
                    .iter()
                    .find(|tag| tag.key.as_ref() == key)
                    .map(|tag| SharedString::from(tag.value.to_string()))
            };
            let service = lookup("service.name").unwrap_or_else(|| "unknown_service".into());
            let namespace = lookup("service.namespace");
            for scope in items(
                resource,
                &["scopeSpans", "scope_spans", "instrumentationLibrarySpans"],
            ) {
                let library = field(scope, &["scope", "instrumentationLibrary"]).map(|scope| {
                    let version = scope
                        .get("version")
                        .map(text)
                        .filter(|version| !version.is_empty());
                    (text(scope.get("name").unwrap_or(&Value::Null)), version)
                });
                for span in items(scope, &["spans"]) {
                    rows.push(otlp_span(
                        span,
                        &service,
                        namespace.as_ref(),
                        &resource_tags,
                        library.clone(),
                    ));
                }
            }
        }
        Ok(Self::from_spans(rows))
    }
}

fn otlp_value(value: &Value) -> TagValue {
    if let Some(text) = value.get("stringValue") {
        return TagValue::Text(self::text(text));
    }
    if let Some(flag) = value.get("boolValue").and_then(Value::as_bool) {
        return TagValue::Bool(flag);
    }
    if let Some(number) = value.get("intValue") {
        return number
            .as_i64()
            .or_else(|| number.as_str().and_then(|text| text.parse().ok()))
            .map_or_else(|| TagValue::Text(text(number)), TagValue::Int);
    }
    if let Some(number) = value.get("doubleValue").and_then(Value::as_f64) {
        return TagValue::Float(number);
    }
    let nested = field(value, &["arrayValue", "kvlistValue"]).unwrap_or(value);
    TagValue::Text(nested.to_string().into())
}

fn otlp_attributes(value: &Value) -> Vec<Tag> {
    items(value, &["attributes"])
        .iter()
        .map(|attribute| Tag {
            key: text(attribute.get("key").unwrap_or(&Value::Null)),
            value: otlp_value(attribute.get("value").unwrap_or(&Value::Null)),
        })
        .collect()
}

fn hex(value: Option<&Value>) -> SharedString {
    value.map(text).unwrap_or_default()
}

fn otlp_span(
    span: &Value,
    service: &SharedString,
    namespace: Option<&SharedString>,
    resource: &[Tag],
    library: Option<(SharedString, Option<SharedString>)>,
) -> SpanRow {
    let start = number(field(span, &["startTimeUnixNano", "start_time_unix_nano"])).unwrap_or(0);
    let end = number(field(span, &["endTimeUnixNano", "end_time_unix_nano"])).unwrap_or(start);
    let parent = hex(field(span, &["parentSpanId", "parent_span_id"]));
    let status = span.get("status").unwrap_or(&Value::Null);
    let code = status.get("code");
    let error = code.and_then(Value::as_u64) == Some(2)
        || code
            .and_then(Value::as_str)
            .is_some_and(|code| code.ends_with("ERROR"));
    let ok = code.and_then(Value::as_u64) == Some(1)
        || code
            .and_then(Value::as_str)
            .is_some_and(|code| code.ends_with("OK"));
    let kinds: HashMap<u64, SpanKind> = [
        (1, SpanKind::Internal),
        (2, SpanKind::Server),
        (3, SpanKind::Client),
        (4, SpanKind::Producer),
        (5, SpanKind::Consumer),
    ]
    .into();
    let kind = span.get("kind").and_then(|kind| {
        kind.as_u64()
            .and_then(|code| kinds.get(&code).copied())
            .or_else(|| kind.as_str().and_then(SpanKind::parse))
    });
    SpanRow {
        span_id: hex(field(span, &["spanId", "span_id"])),
        parent: (!parent.is_empty()).then_some(parent),
        trace_id: hex(field(span, &["traceId", "trace_id"])),
        operation: text(span.get("name").unwrap_or(&Value::Null)),
        service: service.clone(),
        service_namespace: namespace.cloned(),
        service_tags: resource
            .iter()
            .filter(|tag| !matches!(tag.key.as_ref(), "service.name" | "service.namespace"))
            .cloned()
            .collect(),
        start: start / 1_000,
        duration: end.saturating_sub(start) / 1_000,
        logs: items(span, &["events"])
            .iter()
            .map(|event| SpanLog {
                timestamp: number(field(event, &["timeUnixNano", "time_unix_nano"])).unwrap_or(0)
                    / 1_000,
                name: event.get("name").map(text),
                fields: otlp_attributes(event),
            })
            .collect(),
        references: items(span, &["links"])
            .iter()
            .map(|link| SpanReference {
                trace_id: hex(field(link, &["traceId", "trace_id"])),
                span_id: hex(field(link, &["spanId", "span_id"])),
                tags: otlp_attributes(link),
            })
            .collect(),
        tags: otlp_attributes(span),
        kind,
        status: if error {
            SpanStatus::Error
        } else if ok {
            SpanStatus::Ok
        } else {
            SpanStatus::Unset
        },
        status_message: status
            .get("message")
            .map(text)
            .filter(|message| !message.is_empty()),
        library,
        trace_state: field(span, &["traceState", "trace_state"])
            .map(text)
            .filter(|state| !state.is_empty()),
        ..SpanRow::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_jaeger_json() {
        let json = r#"{"data":[{"traceID":"t1","spans":[
            {"traceID":"t1","spanID":"a","operationName":"GET /","references":[],"startTime":1000,"duration":500,"tags":[{"key":"span.kind","type":"string","value":"server"}],"logs":[],"processID":"p1"},
            {"traceID":"t1","spanID":"b","operationName":"query","references":[{"refType":"CHILD_OF","traceID":"t1","spanID":"a"}],"startTime":1100,"duration":200,"tags":[{"key":"error","type":"bool","value":true}],"logs":[{"timestamp":1150,"fields":[{"key":"event","value":"retry"}]}],"processID":"p2"}
        ],"processes":{"p1":{"serviceName":"api","tags":[]},"p2":{"serviceName":"db","tags":[{"key":"host","value":"h1"}]}}}]}"#;
        let trace = Trace::from_jaeger_json(json).unwrap();
        assert_eq!(trace.id(), "t1");
        assert_eq!(trace.len(), 2);
        assert_eq!(trace.spans[1].parent, Some(0));
        assert!(trace.spans[1].error);
        assert_eq!(trace.spans[0].row.kind, Some(SpanKind::Server));
        assert_eq!(trace.service(&trace.spans[1]), "db");
        assert_eq!(trace.spans[1].row.logs.len(), 1);
    }

    #[test]
    fn reads_otlp_json() {
        let json = r#"{"resourceSpans":[{"resource":{"attributes":[{"key":"service.name","value":{"stringValue":"checkout"}},{"key":"host.name","value":{"stringValue":"h1"}}]},
            "scopeSpans":[{"scope":{"name":"otel-go","version":"1.2"},"spans":[
                {"traceId":"abc","spanId":"01","name":"POST /pay","kind":2,"startTimeUnixNano":"2000000","endTimeUnixNano":"5000000","attributes":[{"key":"http.request.method","value":{"stringValue":"POST"}}],"status":{"code":2,"message":"boom"}},
                {"traceId":"abc","spanId":"02","parentSpanId":"01","name":"charge","kind":"SPAN_KIND_CLIENT","startTimeUnixNano":"3000000","endTimeUnixNano":"4000000","events":[{"timeUnixNano":"3500000","name":"retry","attributes":[]}]}
            ]}]}]}"#;
        let trace = Trace::from_otlp_json(json).unwrap();
        assert_eq!(trace.len(), 2);
        assert_eq!(trace.duration(), 3_000);
        let root = &trace.spans[0];
        assert!(root.error);
        assert_eq!(root.row.status_message.as_deref(), Some("boom"));
        assert_eq!(
            root.row.library.as_ref().map(|(name, _)| name.as_ref()),
            Some("otel-go")
        );
        assert_eq!(trace.spans[1].row.kind, Some(SpanKind::Client));
        assert_eq!(trace.header.method.as_deref(), Some("POST"));
    }
}
