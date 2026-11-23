# Traces: API reference

`<Traces>` shows one distributed trace: every span of a request as it passed through your services, on a shared timeline. It's a port of Grafana's Traces panel (Grafana's trace view); see [Coming from Grafana](#coming-from-grafana).

```rust
use blenderui::prelude::*;
use bui_charts::*;
```

The buttons' icons are line icons bui-charts draws itself: nothing to register.

- [`<Traces>`](#traces)
- [`Trace`](#trace)
- [`SpanRow`](#spanrow)
- [`SpanFilters`](#spanfilters)
- [What's on screen](#whats-on-screen)
- [Custom actions and share links](#custom-actions-and-share-links)
- [Enums and types](#enums-and-types)
- [Coming from Grafana](#coming-from-grafana)

Not sure which option fits? See the [guide](guide.md).

---

## `<Traces>`

| Prop | Type | Default |
| --- | --- | --- |
| `data` | `Trace` or `Signal<Trace>` | **required** |
| `filters` | [`SpanFilters`](#spanfilters) | none |
| `focused_span` | span ID | none |
| `hide_header_details` | `bool` | `false` |
| `share_link` | `Fn(&str, &str) -> SharedString`: trace ID and span ID to a URL | none: no Share button |
| `actions` | `&'static [SpanAction]` | none |
| `bar_label` | [`SpanBarLabel`](#enums-and-types) | `Duration` |
| `name_column` | `f32`, share of the width | `0.4` (draggable between `0.2` and `0.85`) |
| `class` | Tailwind classes | fills its parent |
| `key` | anything `Hash` (taken by `rsx!`: hook state follows it) | – |

- `filters` sets the search the view starts with. After that, people change it in the filter bar.
- `focused_span` expands the span's parents, opens its details and scrolls to it. Change it to jump to another span.
- With `hide_header_details`, only the title row stays above the timeline: no banner, details, overview or filter bar.

An empty trace shows "No data found in response".

## `Trace`

| Constructor | Use it for |
| --- | --- |
| `Trace::from_spans(rows)` | spans you build yourself with [`SpanRow`](#spanrow) |
| `Trace::from_jaeger_json(text)` | a Jaeger API response (`{"data": [{ "traceID", "spans", "processes" }]}`), or a single trace object |
| `Trace::from_otlp_json(text)` | OpenTelemetry's JSON export (`{"resourceSpans": …}`) |

The JSON constructors return `Result<Trace, serde_json::Error>`; parts they don't recognise are skipped.

| Method | Returns |
| --- | --- |
| `id()` | the trace ID |
| `name()` | `"service: operation"` of the root span |
| `duration()` | µs from the first span's start to the last span's end |
| `len()` / `is_empty()` | the number of spans |

**How spans are arranged.**
- Each span sits under its parent. Without a parent, it sits under the span its first reference points to.
- Children are ordered by start time. Spans whose parent isn't in the trace go at the top level.
- Spans with a start time of 0 are dropped.
- A repeated span ID gets `_1`, `_2`, … added.
- A tag repeated with the same value is kept once, and the span gets a warning about it.

Keep the trace in a signal (`use_signal(Trace::from_spans(…))`). A `Trace` passed directly is arranged again on every render of your component.

## `SpanRow`

```rust
SpanRow::new("b7ad6b71", "checkout", "PlaceOrder")
    .parent("53995c3f")
    .trace_id("4bf92f3577b34da6")
    .start(1_729_000_000_000_000)
    .duration(812_000)
    .kind(SpanKind::Server)
    .tag("http.request.method", "POST")
    .tag("http.response.status_code", 500)
    .service_tag("k8s.pod.name", "checkout-7d9f")
    .log(SpanLog::new(1_729_000_000_300_000).name("retry").field("attempt", 1))
    .status(SpanStatus::Error)
```

| Method | Argument |
| --- | --- |
| `new(span_id, service, operation)` | text |
| `parent` | parent span ID |
| `trace_id` | text |
| `service_namespace` | text; shown as `namespace/service` |
| `start` | µs since the Unix epoch (`u64`) |
| `duration` | µs (`u64`) |
| `tag(key, value)` | span attribute |
| `service_tag(key, value)` | resource attribute |
| `log(SpanLog)` | an event: `SpanLog::new(µs).name(…).field(key, value)` |
| `reference(SpanReference)` | a link to another span: `SpanReference::new(trace_id, span_id).tag(…)` |
| `kind` | [`SpanKind`](#enums-and-types) |
| `status` / `status_message` | [`SpanStatus`](#enums-and-types) / text |
| `library(name, version)` | instrumentation library |
| `trace_state` | W3C trace state |
| `warning` / `stack_trace` | text; repeatable |
| `error_color` | colour of this span's error icon |

Tag values can be text, whole numbers, decimals or `bool`. A span counts as an error when its status is `Error` or it has the tag `error = true`.

## `SpanFilters`

```rust
SpanFilters::new()
    .service(FilterOp::Eq, "checkout")
    .duration(FilterOp::Ge, "100ms")
    .tag("http.response.status_code", FilterOp::Eq, "500")
    .text("timeout")
    .matches_only(true)
```

| Method | Matches spans where |
| --- | --- |
| `service(op, value)` | the service name `=`, `!=`, contains (`=~`) or doesn't contain (`!~`) the value |
| `span_name(op, value)` | the same, for the operation name |
| `duration(op, value)` | the duration is `=`, `>=`, `<=`, `>` or `<` the value: `"250"` (µs), `"500ns"`, `"120us"`, `"120µs"`, `"80ms"`, `"2s"`, `"1m"`, `"1h"` |
| `tag(key, op, value)` | a span, resource or event attribute matches. Also understands `kind`, `status`, `status.message`, `library.name`, `library.version`, `trace.state` and `id` |
| `text(query)` | any word of the query appears in the span's names, attributes, events, kind, status or library (ignoring case), or a word is the span ID |
| `critical_path_only(bool)` | the span is on the critical path |
| `matches_only(bool)` | – : hides every span that doesn't match, instead of tinting the ones that do |

All filters must match. With `Contains` / `NotContains`, a tag filter matches keys and values that contain the text.

## What's on screen

**Header.** The root service and operation, the HTTP method and status, and a ⋮ menu with Copy link (when you pass `share_link`) and Export as JSON (written to `Trace-<last 6 of the id>.json` in the working directory: blenderui has no save dialog). Below that:
- A banner for the most telling error span (deepest, then longest), or a 4xx span, with its share of the trace and a Go to span button.
- Trace ID (click to copy), start time, duration, number of services and spans, and the route.
- **Overview:** a minimap of the whole trace. Drag across it to zoom the timeline, drag the edges to adjust the zoom, and use Reset selection to zoom out.
- **Filter bar:**
  - Pick a key, an operator and a value, then press Add or Enter.
  - The chips show active filters.
  - The Critical path, Errors and High latency (slowest 10%) pills toggle common filters.
  - The arrows step through matches, showing "i/N matches".
  - Show all spans switches matches-only on and off.

**Timeline.**
- The name column shows a span's service only where it changes from the row above.
- The ⌄ / › chevron expands and collapses a span. The header buttons, or the `o` `p` `[` `]` keys after clicking the timeline, expand one level, collapse one level, expand everything and collapse everything.
- Indent guides highlight every row that shares the ancestor under the pointer.
- A collapsed client span shows the server it called. A client span with a `peer.service` tag shows the peer.
- Bars use one colour per service. Thin dark segments mark the critical path, and ticks mark events; hover a tick to read them.
- Hover a bar for `service::operation | duration`.
- Drag across the timeline header to zoom. Drag the column divider to resize the name column.
- Rows are tinted for errors, search matches and the focused span.

**Span details** (click a row).
- Service, duration, start time, child count, kind, status, library and trace state.
- An exception box when the span recorded one.
- Span and resource attributes grouped by category (HTTP, Database, Kubernetes, Cloud, …), with a copy button on every value.
- Events, warnings, stack traces and references (click one to go to that span).
- Your actions and Share, and the span ID (click to copy).

## Custom actions and share links

```rust
fn open_logs(span: &SelectedSpan<'_>, window: &mut Window, cx: &mut App) {
    println!("logs for {} in {}", span.span_id, span.service);
}

const ACTIONS: &[SpanAction] = &[SpanAction { label: "Logs", run: open_logs }];

<Traces
    data={trace}
    actions={ACTIONS}
    share_link={|trace: &str, span: &str| format!("https://traces.example.com/{trace}?span={span}").into()}
/>
```

`SelectedSpan` gives `trace_id`, `span_id`, `service`, `operation`, `start` and `duration`. The Share button copies the link for the open span; Copy link in the ⋮ menu copies it with an empty span ID.

## Enums and types

| Type | Values |
| --- | --- |
| `SpanKind` | `Internal`, `Server`, `Client`, `Producer`, `Consumer` |
| `SpanStatus` | `Unset` (default), `Ok`, `Error` |
| `SpanBarLabel` | `Duration` (default), `None`, `Tag(&str)`: the label after each name |
| `FilterOp` | `Eq`, `NotEq`, `Contains`, `NotContains`, `Gt`, `Ge`, `Lt`, `Le` |
| `FilterKey` | `Service`, `SpanName`, `Duration`, `Text`, `Tag(key)` |
| `TagValue` | `Text`, `Int`, `Float`, `Bool`; made from `&str`, `String`, numbers and `bool` |
| `SpanAction` | `{ label: &'static str, run: fn(&SelectedSpan, &mut Window, &mut App) }` |

Durations are shown the way Grafana shows them: `850μs`, `1.23ms`, `1.5s`, `1h 5m`.

## Coming from Grafana

| Grafana | Here |
| --- | --- |
| data frame with `traceID`, `spanID`, `parentSpanID`, `operationName`, `serviceName`, `startTime`, `duration`, tags, logs… | [`SpanRow`](#spanrow), or `Trace::from_jaeger_json` / `from_otlp_json` |
| Span filters, Show matches only, Select critical path | `filters` |
| `focusedSpanId` | `focused_span` |
| `hideHeaderDetails` | `hide_header_details` |
| `createFocusSpanLink` | `share_link` |
| `createSpanLink` (logs, metrics, profiles links) | `actions` |
| Span bar label setting | `bar_label` |

Not available here, because they need a Grafana server: data source links to logs, metrics and profiles, the Pyroscope span flame graph, plugin extensions, and the adaptive traces banner. The span colours come from your [theme's](../theming.md) palette rather than Grafana's.
