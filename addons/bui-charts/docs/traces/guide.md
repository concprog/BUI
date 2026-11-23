# Traces: choosing options

How to pick between the options in the [API reference](api.md). Every link goes to a runnable example; the gallery has them all (`cargo run --example gallery`, then **Traces**).

## Getting a trace in

| You have | Use |
| --- | --- |
| spans from your own collector or database | `Trace::from_spans` with one `SpanRow` per span |
| a Jaeger query response, or a trace downloaded from Jaeger | `Trace::from_jaeger_json` |
| an OpenTelemetry JSON export (OTLP, the collector's file exporter) | `Trace::from_otlp_json` |

Build the trace once and keep it in a signal. Replace the signal's value when a new trace arrives.

Examples: [Trace view](examples.md#1-trace-view), [From Jaeger JSON](examples.md#9-from-jaeger-json), [From OTLP JSON](examples.md#10-from-otlp-json), [Live data](examples.md#12-live-data).

## How much to show

| Space | Use |
| --- | --- |
| a full page or a large panel | the default: header, overview, filters and timeline |
| a panel in a dashboard, or next to other views | `hide_header_details={true}`, maybe with a narrower `name_column` |

Examples: [Embedded](examples.md#2-embedded), [Labels from a tag](examples.md#7-labels-from-a-tag).

## Opening on the interesting part

| Question | Use |
| --- | --- |
| "What failed?" | `filters={SpanFilters::new().tag("status", FilterOp::Eq, "error").matches_only(true)}` |
| "What made it slow?" | `critical_path_only(true)`: the chain of spans the request actually waited on |
| "Where is this span?", from an alert or a log line | `focused_span` with its ID |
| "Which spans mention X?" | `text("X")`, combined with a duration filter to skip the quick ones |

`matches_only` hides everything else. Leave it off to keep the context and only tint the matches.

Examples: [Errors only](examples.md#3-errors-only), [Critical path](examples.md#4-critical-path), [Focused span](examples.md#6-focused-span), [Search](examples.md#5-search).

## Labels

The label after each name is the span's duration. Show a tag instead when it tells people more at a glance, such as `db.system`, `http.response.status_code` or a tenant ID: `bar_label={SpanBarLabel::Tag("db.system")}`. Use `SpanBarLabel::None` for a cleaner list.

Example: [Labels from a tag](examples.md#7-labels-from-a-tag).

## Hooking into your app

- `actions` adds buttons to every span's details: open its logs, jump to a dashboard, file a ticket. Each action is a plain function in a `static` list.
- `share_link` turns a trace and span ID into a URL in your app, behind the Share button and Copy link.

Example: [Share link and actions](examples.md#8-share-link-and-actions).

## Big traces

Only the rows on screen are drawn, so traces with thousands of spans scroll smoothly. Collapse everything with the header button, then expand one level at a time to find your way around. Zoom with the overview or the timeline header to read short spans inside a long trace.

Example: [601 spans](examples.md#11-601-spans).

## Summary spans

Grafana's adaptive traces can replace many similar spans with one summary span carrying `aggregation.*` tags. Pass those tags through; the view shows the count, the min / median / max durations and a shaded bar.

Example: [Summary spans](examples.md#13-summary-spans).
