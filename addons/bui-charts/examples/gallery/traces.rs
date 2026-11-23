use blenderui::prelude::*;
use bui_charts::*;


const T0: u64 = 1_729_000_000_000_000;
const MS: u64 = 1_000;

fn span(
    id: &'static str,
    service: &'static str,
    operation: &'static str,
    start: u64,
    length: u64,
) -> SpanRow {
    SpanRow::new(id, service, operation)
        .trace_id("4bf92f3577b34da6a3ce929d0e0e4736")
        .start(T0 + start * MS)
        .duration(length * MS)
}

fn checkout_spans() -> Vec<SpanRow> {
    vec![
        span("root", "frontend", "POST /checkout", 0, 812)
            .kind(SpanKind::Server)
            .tag("http.request.method", "POST")
            .tag("http.route", "/checkout")
            .tag("http.response.status_code", 500)
            .service_tag("k8s.pod.name", "frontend-7d9f")
            .service_tag("service.version", "2.4.1")
            .library("opentelemetry-js", Some("1.25.0".into())),
        span("gw", "api-gateway", "checkout", 12, 790).parent("root").kind(SpanKind::Client).tag("peer.service", "checkout"),
        span("co", "checkout", "PlaceOrder", 18, 770)
            .parent("gw")
            .kind(SpanKind::Server)
            .tag("rpc.system", "grpc")
            .tag("rpc.method", "PlaceOrder")
            .service_tag("k8s.namespace.name", "shop")
            .service_tag("host.name", "node-3"),
        span("cart", "cart", "GetCart", 30, 96).parent("co").kind(SpanKind::Client),
        span("redis", "redis", "HGETALL cart:42", 38, 70)
            .parent("cart")
            .kind(SpanKind::Server)
            .tag("db.system", "redis")
            .tag("db.statement", "HGETALL cart:42"),
        span("inv", "inventory", "GET /stock", 130, 140)
            .parent("co")
            .kind(SpanKind::Client)
            .tag("http.request.method", "GET")
            .tag("url.full", "http://inventory/stock?sku=42")
            .tag("http.response.status_code", 404),
        span("pay", "payment", "Charge", 280, 460)
            .parent("co")
            .kind(SpanKind::Client)
            .log(SpanLog::new(T0 + 300 * MS).name("retry").field("attempt", 1))
            .log(SpanLog::new(T0 + 520 * MS).name("retry").field("attempt", 2)),
        span("pay-srv", "payment", "ChargeCard", 290, 440).parent("pay").kind(SpanKind::Server),
        span("fraud", "fraud", "Score", 300, 120).parent("pay-srv").tag("model", "v7"),
        span("payment-db", "postgres", "INSERT payments", 430, 290)
            .parent("pay-srv")
            .tag("db.system", "postgresql")
            .tag("db.statement", "INSERT INTO payments (order_id, amount) VALUES ($1, $2)")
            .status(SpanStatus::Error)
            .status_message("deadlock detected")
            .tag("exception.type", "PgError")
            .tag("exception.message", "deadlock detected")
            .tag("exception.stacktrace", "PgError: deadlock detected\n  at insert (payments.rs:88)\n  at charge (payments.rs:41)"),
        span("email", "email", "SendConfirmation", 760, 40)
            .reference(SpanReference::new("4bf92f3577b34da6a3ce929d0e0e4736", "co"))
            .kind(SpanKind::Consumer)
            .tag("messaging.system", "kafka"),
        span("audit", "audit", "Record", 770, 25).parent("co").warning("clock skew adjusted"),
    ]
}

fn checkout() -> Trace {
    Trace::from_spans(checkout_spans())
}

fn big() -> Trace {
    let services = [
        "web", "auth", "orders", "stock", "billing", "search", "cache", "db",
    ];
    let mut rows = vec![span("big-0", "web", "GET /dashboard", 0, 3000)];
    let mut next = 1u64;
    for batch in 0..60u64 {
        let parent: SharedString = format!("big-{}", batch * 10).into();
        let parent = if batch == 0 {
            SharedString::Borrowed("big-0")
        } else {
            parent
        };
        for child in 0..10u64 {
            let id = next;
            next += 1;
            let start = batch * 48 + child * 3;
            let service = services[(id as usize) % services.len()];
            let mut row = SpanRow::new(
                SharedString::from(format!("big-{id}")),
                service,
                format!("op-{}", id % 17),
            )
            .trace_id("big")
            .start(T0 + start * MS + 1)
            .duration((20 + (id * 37) % 400) * MS)
            .parent(parent.clone());
            if id.is_multiple_of(97) {
                row = row.status(SpanStatus::Error);
            }
            rows.push(row);
        }
    }
    Trace::from_spans(rows)
}

const JAEGER: &str = r#"{"data":[{"traceID":"6a1f","spans":[
{"traceID":"6a1f","spanID":"a","operationName":"HTTP GET /api/users","references":[],"startTime":1729000000000000,"duration":180000,"tags":[{"key":"span.kind","value":"server"},{"key":"http.method","value":"GET"},{"key":"http.status_code","value":200}],"logs":[],"processID":"p1"},
{"traceID":"6a1f","spanID":"b","operationName":"SELECT users","references":[{"refType":"CHILD_OF","traceID":"6a1f","spanID":"a"}],"startTime":1729000000020000,"duration":90000,"tags":[{"key":"db.system","value":"mysql"}],"logs":[{"timestamp":1729000000050000,"fields":[{"key":"event","value":"slow query"}]}],"processID":"p2"},
{"traceID":"6a1f","spanID":"c","operationName":"render","references":[{"refType":"CHILD_OF","traceID":"6a1f","spanID":"a"}],"startTime":1729000000120000,"duration":50000,"tags":[],"logs":[],"processID":"p1"}
],"processes":{"p1":{"serviceName":"users-api","tags":[{"key":"hostname","value":"api-1"}]},"p2":{"serviceName":"mysql","tags":[]}}}]}"#;

const OTLP: &str = r#"{"resourceSpans":[{"resource":{"attributes":[{"key":"service.name","value":{"stringValue":"orders"}},{"key":"service.namespace","value":{"stringValue":"shop"}},{"key":"k8s.pod.name","value":{"stringValue":"orders-5c8"}}]},
"scopeSpans":[{"scope":{"name":"opentelemetry-go","version":"1.28.0"},"spans":[
{"traceId":"9c2e","spanId":"01","name":"POST /orders","kind":2,"startTimeUnixNano":"1729000000000000000","endTimeUnixNano":"1729000000420000000","attributes":[{"key":"http.request.method","value":{"stringValue":"POST"}},{"key":"http.response.status_code","value":{"intValue":"201"}}],"status":{"code":1}},
{"traceId":"9c2e","spanId":"02","parentSpanId":"01","name":"publish order.created","kind":4,"startTimeUnixNano":"1729000000300000000","endTimeUnixNano":"1729000000340000000","attributes":[{"key":"messaging.system","value":{"stringValue":"kafka"}}],"events":[{"timeUnixNano":"1729000000310000000","name":"ack","attributes":[]}]}
]}]}]}"#;

fn open_logs(selected: &SelectedSpan<'_>, _: &mut Window, _: &mut App) {
    println!("logs for {} {}", selected.service, selected.span_id);
}

const ACTIONS: &[SpanAction] = &[SpanAction {
    label: "Logs",
    run: open_logs,
}];

#[component]
fn full_view() {
    let trace = use_signal(checkout());
    rsx! { <Traces data={trace} class="h-[720px]" /> }
}

#[component]
fn embedded() {
    let trace = use_signal(checkout());
    rsx! { <Traces data={trace} hide_header_details={true} class="h-[420px]" /> }
}

#[component]
fn errors_only() {
    let trace = use_signal(checkout());
    rsx! {
        <Traces data={trace} filters={SpanFilters::new().tag("status", FilterOp::Eq, "error").matches_only(true)} class="h-[560px]" />
    }
}

#[component]
fn critical_path() {
    let trace = use_signal(checkout());
    rsx! { <Traces data={trace} filters={SpanFilters::new().critical_path_only(true)} class="h-[560px]" /> }
}

#[component]
fn text_search() {
    let trace = use_signal(checkout());
    rsx! {
        <Traces data={trace} filters={SpanFilters::new().text("redis postgres").duration(FilterOp::Ge, "50ms")} class="h-[560px]" />
    }
}

#[component]
fn focused() {
    let trace = use_signal(checkout());
    rsx! { <Traces data={trace} focused_span={SharedString::Borrowed("payment-db")} class="h-[720px]" /> }
}

#[component]
fn tag_labels() {
    let trace = use_signal(checkout());
    rsx! {
        <Traces data={trace} hide_header_details={true} bar_label={SpanBarLabel::Tag("db.system")} name_column={0.3} class="h-[420px]" />
    }
}

#[component]
fn share_and_actions() {
    let trace = use_signal(checkout());
    rsx! {
        <Traces
            data={trace}
            hide_header_details={true}
            share_link={|trace: &str, span: &str| format!("https://traces.example.com/{trace}?span={span}").into()}
            actions={ACTIONS}
            focused_span={SharedString::Borrowed("pay")}
            class="h-[620px]"
        />
    }
}

#[component]
fn jaeger() {
    let trace = use_signal(Trace::from_jaeger_json(JAEGER).unwrap_or_default());
    rsx! { <Traces data={trace} class="h-[520px]" /> }
}

#[component]
fn otlp() {
    let trace = use_signal(Trace::from_otlp_json(OTLP).unwrap_or_default());
    rsx! { <Traces data={trace} class="h-[520px]" /> }
}

#[component]
fn large() {
    let trace = use_signal(big());
    rsx! { <Traces data={trace} class="h-[720px]" /> }
}

#[component]
fn live() {
    let trace = use_signal(checkout());
    let count = use_signal(0u64);
    rsx! {
        <div class="flex flex-col gap-2 h-[560px]">
            <div
                id="add-span"
                class="cursor-pointer text-sm underline"
                onClick={move || {
                    let next = count.get() + 1;
                    count.set(next);
                    let mut rows = checkout_spans();
                    for extra in 0..next {
                        rows.push(span("audit", "audit", "Retry", 780 + extra * 5, 4).parent("co"));
                    }
                    trace.update(|trace| *trace = Trace::from_spans(rows));
                }}
            >
                "Add a retry span"
            </div>
            <Traces data={trace} hide_header_details={true} class="flex-1" />
        </div>
    }
}

#[component]
fn summary() {
    let trace = use_signal(Trace::from_spans([
        span("s0", "gateway", "GET /feed", 0, 400),
        span("s1", "feed", "LoadPosts", 20, 300)
            .parent("s0")
            .tag("aggregation.is_summary", true)
            .tag("aggregation.span_count", 48)
            .tag("aggregation.duration_min_ns", 2_000_000.)
            .tag("aggregation.duration_median_ns", 6_000_000.)
            .tag("aggregation.duration_max_ns", 41_000_000.),
        span("s2", "feed", "Rank", 330, 50).parent("s0"),
    ]));
    rsx! { <Traces data={trace} class="h-[480px]" /> }
}

#[component]
fn empty() {
    let trace = use_signal(Trace::default());
    rsx! { <Traces data={trace} class="h-40" /> }
}

examples! {
    full_view: "Trace view" => "Header, minimap, filters and the span timeline. Click a span for its details.",
    embedded: "Embedded" => "hide_header_details keeps only the title above the timeline.",
    errors_only: "Errors only" => "Preset filters: status = error, showing matches only.",
    critical_path: "Critical path" => "Only spans on the critical path match.",
    text_search: "Search" => "Free text plus a duration filter; step through matches with the arrows.",
    focused: "Focused span" => "Opens and scrolls to a span by id.",
    tag_labels: "Labels from a tag" => "A tag instead of the duration next to each name, with a narrow name column.",
    share_and_actions: "Share link and actions" => "Your own buttons in the span details, plus a Share link.",
    jaeger: "From Jaeger JSON" => "Trace::from_jaeger_json reads a Jaeger API response.",
    otlp: "From OTLP JSON" => "Trace::from_otlp_json reads OpenTelemetry's JSON export.",
    large: "601 spans" => "Only the visible rows are drawn.",
    live: "Live data" => "A signal as data: the view updates when the trace changes.",
    summary: "Summary spans" => "Aggregated spans from Grafana's adaptive traces.",
    empty: "Empty" => "What an empty trace shows.",}
