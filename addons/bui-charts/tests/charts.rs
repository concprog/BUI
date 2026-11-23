//! Charts driven headless through blenderui's simulated input: hover (tooltip and crosshair), the
//! legend, drag-to-zoom and double-click home, the node graph's zoom buttons, and theme switching.

use std::sync::Mutex;

use blenderui::prelude::*;
use bui_charts::*;

#[derive(Clone)]
struct Sample {
    time: f64,
    cpu: f64,
    mem: f64,
}

fn samples() -> Vec<Sample> {
    (0..60)
        .map(|step| {
            let t = f64::from(step);
            Sample { time: 1_729_000_000_000. + t * 60_000., cpu: 50. + 30. * (t / 7.).sin(), mem: 40. + 10. * (t / 11.).cos() }
        })
        .collect()
}

static SELECTED: Mutex<Option<SelectedRange>> = Mutex::new(None);

#[component]
fn series_chart() {
    let data = use_signal(samples());
    rsx! {
        <div id="chart" class="absolute inset-0 flex flex-col p-4" style="background: #ffffff">
            <TimeSeries data={data} x={|s: &Sample| s.time} class="flex-1" legend={Legend::list()}
                on_select={|range: SelectedRange, _: &mut Window, _: &mut App| *SELECTED.lock().unwrap() = Some(range)}>
                <TrendSeries name="cpu" y={|s: &Sample| s.cpu} line_width={3.} />
                <TrendSeries name="mem" y={|s: &Sample| s.mem} line_width={3.} />
            </TimeSeries>
        </div>
    }
}

struct Root;

impl Render for Root {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        rsx! { <series_chart /> }
    }
}

const W: f32 = 800.;
const H: f32 = 400.;

fn open(cx: &mut App, root: impl Render) -> WindowId {
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(Bounds { origin: point(px(0.), px(0.)), size: size(px(W), px(H)) })),
        ..Default::default()
    };
    let win = cx.open_window(options, |_, cx| cx.new(|_| root)).unwrap();
    frame(cx, win.id);
    win.id
}

fn frame(cx: &mut App, id: WindowId) -> Vec<u32> {
    cx.draw(id);
    cx.draw(id).2
}

fn hex(c: Rgba) -> [i32; 3] {
    [c.r, c.g, c.b].map(|v| (v * 255.).round() as i32)
}

/// Pixels within a few steps of `color`.
fn count(pixels: &[u32], color: Rgba) -> usize {
    let [r, g, b] = hex(color);
    pixels
        .iter()
        .filter(|p| {
            let (pr, pg, pb) = (i32::from((*p >> 16) as u8), i32::from((*p >> 8) as u8), i32::from(**p as u8));
            (pr - r).abs() + (pg - g).abs() + (pb - b).abs() < 24
        })
        .count()
}

fn dump(name: &str, pixels: &[u32]) {
    if let Ok(dir) = std::env::var("CHART_DUMP") {
        let mut out = format!("P6
{} {}
255
", W as usize, H as usize).into_bytes();
        for p in pixels {
            out.extend_from_slice(&[(p >> 16) as u8, (p >> 8) as u8, *p as u8]);
        }
        std::fs::write(format!("{dir}/{name}.ppm"), out).unwrap();
    }
}

fn differ(a: &[u32], b: &[u32]) -> usize {
    a.iter().zip(b).filter(|(a, b)| a != b).count()
}

#[test]
fn hover_shows_the_tooltip_and_leaving_hides_it() {
    Application::headless().run(|cx| {
        blenderui::set_theme("light");
        let id = open(cx, Root);
        let plain = frame(cx, id);
        cx.simulate_mouse_move(id, W / 2., H / 2.);
        let hovered = frame(cx, id);
        dump("hover", &hovered);
        let popover = blenderui::theme_color("tooltip.inner").map(|[r, g, b, _]| Rgba { r, g, b, a: 1. }).unwrap();
        assert!(count(&hovered, popover) > 2000, "a tooltip card at the pointer");
        let before = count(&plain, popover);
        // the crosshair and the hover markers are drawn
        assert!(differ(&plain, &hovered) > 0);
        cx.simulate_mouse_move(id, W / 2., H + 50.);
        cx.simulate_mouse_move(id, -10., -10.);
        assert!(count(&frame(cx, id), popover) <= before + 50, "leaving the plot hides it");
    });
}

#[test]
fn legend_click_isolates_a_series_and_a_second_click_restores_it() {
    Application::headless().run(|cx| {
        blenderui::set_theme("light");
        let id = open(cx, Root);
        let mem = ChartTheme::colors().series(1);
        let before = count(&frame(cx, id), mem);
        assert!(before > 200, "the mem line is drawn: {before}");
        let cpu = cx.element_bounds(id, "legend-0").expect("a legend entry per series");
        let (x, y) = (cpu.origin.x + cpu.size.width / 2., cpu.origin.y + cpu.size.height / 2.);
        cx.simulate_click(id, x, y);
        let isolated = count(&frame(cx, id), mem);
        assert!(isolated < before / 4, "cpu alone: mem {before} -> {isolated} pixels");
        cx.simulate_click(id, x, y);
        let restored = count(&frame(cx, id), mem);
        assert!(restored > before * 3 / 4, "both again: {restored}");
    });
}

#[test]
fn dragging_zooms_and_reports_the_range_double_click_goes_home() {
    Application::headless().run(|cx| {
        blenderui::set_theme("light");
        let id = open(cx, Root);
        let home = frame(cx, id);
        cx.simulate_drag(id, (W * 0.3, H / 2.), (W * 0.6, H / 2.));
        let zoomed = frame(cx, id);
        let range = SELECTED.lock().unwrap().take().expect("on_select hears the dragged range");
        let (start, end) = (samples()[0].time, samples()[59].time);
        assert!(range.x.0 > start && range.x.1 < end && range.x.0 < range.x.1, "{range:?}");
        assert!(differ(&home, &zoomed) > 0, "the plot shows the range");
        cx.simulate_mouse_move(id, -10., -10.);
        let (x, y) = (W / 2., H / 2.);
        cx.simulate_press(id, x, y);
        cx.simulate_release(id, x, y);
        cx.simulate_press(id, x, y);
        cx.simulate_release(id, x, y);
        cx.simulate_mouse_move(id, -10., -10.);
        let back = frame(cx, id);
        let differing = differ(&home, &back);
        assert!(differing < home.len() / 200, "home again: {differing} pixels differ");
    });
}

#[test]
fn charts_follow_the_kit_theme() {
    Application::headless().run(|cx| {
        blenderui::set_theme("light");
        let light_series = ChartTheme::colors().series(0);
        let id = open(cx, Root);
        let light = frame(cx, id);
        blenderui::set_theme("dark");
        // a theme change repaints; the app re-renders for values views computed (as gpui-kit's theme
        // change notified every window)
        cx.notify_all();
        let dark_series = ChartTheme::colors().series(0);
        assert_ne!(hex(light_series), hex(dark_series), "the palette follows the mode");
        let dark = frame(cx, id);
        dump("dark", &dark);
        assert!(count(&light, light_series) > 100 && count(&dark, dark_series) > 100);
        assert!(count(&dark, light_series) < count(&light, light_series) / 4, "repainted in the dark palette");
        blenderui::set_theme("light");
    });
}

#[component]
fn graph() {
    let nodes = use_signal(vec![
        GraphNode::new("a").title("api").main_stat(12.),
        GraphNode::new("b").title("db").main_stat(4.),
        GraphNode::new("c").title("cache").main_stat(7.),
    ]);
    let edges = use_signal(vec![GraphEdge::new("a", "b"), GraphEdge::new("a", "c")]);
    rsx! {
        <div class="absolute inset-0 flex">
            <NodeGraph nodes={nodes} edges={edges} class="flex-1" />
        </div>
    }
}

struct GraphRoot;

impl Render for GraphRoot {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        rsx! { <graph /> }
    }
}

#[test]
fn node_graph_zoom_buttons_change_the_view() {
    Application::headless().run(|cx| {
        blenderui::set_theme("light");
        let id = open(cx, GraphRoot);
        let home = frame(cx, id);
        let zoom_in = cx.element_bounds(id, "zoom-in").expect("a zoom in button");
        cx.simulate_click(id, zoom_in.origin.x + 3., zoom_in.origin.y + zoom_in.size.height / 2.);
        let near = frame(cx, id);
        dump("graph", &home);
        dump("graph-near", &near);
        assert!(differ(&home, &near) > 0, "zooming in redraws the graph larger");
        let reset = cx.element_bounds(id, "reset-view").expect("a reset button");
        cx.simulate_click(id, reset.origin.x + reset.size.width / 2., reset.origin.y + reset.size.height / 2.);
        let back = frame(cx, id);
        let differing = differ(&home, &back);
        assert!(differing < home.len() / 200, "reset: {differing} pixels differ");
    });
}
