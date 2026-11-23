use blenderui::prelude::*;
use bui_charts::*;


fn noise(seed: u64) -> f64 {
    let mut z = seed.wrapping_add(0x9e37_79b9_7f4a_7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    (z ^ (z >> 31)) as f64 / u64::MAX as f64
}

fn normal(seed: u64) -> f64 {
    let (a, b) = (noise(seed * 2 + 1).max(1e-9), noise(seed * 2 + 2));
    (-2. * a.ln()).sqrt() * (std::f64::consts::TAU * b).cos()
}

#[derive(Clone, PartialEq)]
struct Request {
    get: f64,
    post: f64,
    size: f64,
}

fn requests(seed: u64) -> Vec<Request> {
    (0..800u64)
        .map(|index| {
            let n = seed * 10_000 + index;
            Request {
                get: (120. + 25. * normal(n)).max(0.),
                post: (180. + 40. * normal(n + 5_000)).max(0.),
                size: (noise(n * 3) * noise(n * 7) * 64.).floor(),
            }
        })
        .collect()
}

#[derive(Clone, PartialEq)]
struct Bucket {
    low: f64,
    high: f64,
    count: f64,
}

fn buckets() -> Vec<Bucket> {
    [
        (0., 50., 12.),
        (50., 100., 48.),
        (100., 250., 130.),
        (250., 500., 74.),
        (500., 1000., 22.),
        (1000., 2500., 6.),
    ]
    .map(|(low, high, count)| Bucket { low, high, count })
    .into()
}

fn scores() -> Frame<2> {
    Frame::from_columns([
        (
            "math",
            (0..300u64)
                .map(|i| (70. + 12. * normal(i)).clamp(0., 100.))
                .collect::<Vec<_>>(),
        ),
        (
            "reading",
            (0..300u64)
                .map(|i| (64. + 15. * normal(i + 900)).clamp(0., 100.))
                .collect::<Vec<_>>(),
        ),
    ])
}

#[component]
fn latency() {
    let data = use_signal(requests(1));
    rsx! {
        <Histogram data={data} class="h-64">
            <HistogramSeries name="GET ms" value={|r: &Request| r.get} />
        </Histogram>
    }
}

#[component]
fn overlaid() {
    let data = use_signal(requests(2));
    rsx! {
        <Histogram data={data} fill_opacity={0.4} class="h-64">
            <HistogramSeries name="GET" value={|r: &Request| r.get} />
            <HistogramSeries name="POST" value={|r: &Request| r.post} />
        </Histogram>
    }
}

#[component]
fn stacked() {
    let data = use_signal(requests(3));
    rsx! {
        <Histogram data={data} stacking={Stacking::NORMAL} class="h-64">
            <HistogramSeries name="GET" value={|r: &Request| r.get} />
            <HistogramSeries name="POST" value={|r: &Request| r.post} />
        </Histogram>
    }
}

#[component]
fn combined() {
    let data = use_signal(requests(4));
    rsx! {
        <Histogram data={data} combine={true} class="h-64">
            <HistogramSeries name="GET" value={|r: &Request| r.get} />
            <HistogramSeries name="POST" value={|r: &Request| r.post} />
        </Histogram>
    }
}

#[component]
fn bucket_size() {
    let data = use_signal(requests(5));
    rsx! {
        <Histogram data={data} bucket_size={25.} class="h-64">
            <HistogramSeries name="POST ms" value={|r: &Request| r.post} />
        </Histogram>
    }
}

#[component]
fn offset() {
    let data = use_signal(requests(6));
    rsx! {
        <Histogram data={data} bucket_size={20.} bucket_offset={10.} class="h-64">
            <HistogramSeries name="GET ms" value={|r: &Request| r.get} />
        </Histogram>
    }
}

#[component]
fn few_buckets() {
    let data = use_signal(requests(7));
    rsx! {
        <Histogram data={data} bucket_count={8} line_width={2.} class="h-64">
            <HistogramSeries name="Payload KB" value={|r: &Request| r.size} />
        </Histogram>
    }
}

#[component]
fn pre_counted() {
    let data = use_signal(buckets());
    rsx! {
        <Histogram data={data} x_min={|b: &Bucket| b.low} x_max={|b: &Bucket| b.high} class="h-64">
            <HistogramSeries name="requests" value={|b: &Bucket| b.count} />
        </Histogram>
    }
}

#[component]
fn bucket_frame() {
    let data = use_signal(Frame::new(["xMin", "xMax", "pods"]).with_rows([
        [0., 1., 3.],
        [1., 2., 9.],
        [2., 3., 14.],
        [3., 4., 8.],
        [4., 5., 2.],
    ]));
    rsx! { <Histogram data={data} class="h-64" /> }
}

#[component]
fn gradient() {
    let data = use_signal(scores());
    rsx! {
        <Histogram data={data} gradient={GradientMode::Opacity} fill_opacity={0.9} bucket_size={5.} class="h-64" />
    }
}

#[component]
fn hue() {
    let data = use_signal(requests(8));
    rsx! {
        <Histogram data={data} gradient={GradientMode::Hue} class="h-64">
            <HistogramSeries name="POST ms" value={|r: &Request| r.post} color={rgb(0x8839ef)} />
        </Histogram>
    }
}

#[component]
fn legend_right() {
    let data = use_signal(scores());
    rsx! {
        <Histogram
            data={data}
            bucket_size={10.}
            legend={Legend::list().right()}
            tooltip={Tooltip::multi()}
            y_axis={AxisConfig::new().label("students")}
            class="h-64"
        />
    }
}

#[component]
fn live() {
    let round = use_signal(1u64);
    let data = use_signal(requests(1));
    rsx! {
        <div class="flex flex-col gap-2 h-64">
            <div
                id="resample"
                class="cursor-pointer text-sm underline"
                onClick={move || {
                    let next = round.get() + 1;
                    round.set(next);
                    data.set(requests(next));
                }}
            >
                "Take a new sample"
            </div>
            <Histogram data={data} class="flex-1">
                <HistogramSeries name="GET ms" value={|r: &Request| r.get} />
                <HistogramSeries name="POST ms" value={|r: &Request| r.post} />
            </Histogram>
        </div>
    }
}

examples! {
    latency: "Latency" => "800 request times, 30 buckets sized automatically.",
    overlaid: "Two series" => "GET and POST bucketed together and drawn over each other.",
    stacked: "Stacked" => "The same two series stacked.",
    combined: "Combined" => "combine merges every series into one Count.",
    bucket_size: "Bucket size" => "Fixed 25 ms buckets.",
    offset: "Bucket offset" => "20 ms buckets that start at 10, 30, 50…",
    few_buckets: "Bucket count" => "bucket_count 8 on a skewed distribution, thicker outline.",
    pre_counted: "Pre-counted buckets" => "Rows that are already buckets, of uneven widths.",
    bucket_frame: "xMin / xMax frame" => "A frame with xMin and xMax fields is read as buckets.",
    gradient: "Opacity gradient" => "Every numeric field of a frame, with an opacity gradient.",
    hue: "Hue gradient" => "A custom colour with a hue gradient.",
    legend_right: "Legend and tooltip" => "Legend on the right, every series in the tooltip.",
    live: "Live data" => "A signal as data: buckets are recounted when it changes.",}
