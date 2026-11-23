//! Renders every example of the gallery (every chart kind, `examples/gallery`) headless: each one
//! builds, lays out and paints something besides its background, and a pointer move over it does not
//! panic.
#![allow(clippy::wildcard_imports, clippy::needless_pass_by_value, dead_code)]

#[macro_use]
#[path = "../examples/gallery/registry.rs"]
mod registry;

#[path = "../examples/gallery/bar_gauge.rs"]
mod bar_gauge;
#[path = "../examples/gallery/candlestick.rs"]
mod candlestick;
#[path = "../examples/gallery/category.rs"]
mod category;
#[path = "../examples/gallery/dashboard.rs"]
mod dashboard;
#[path = "../examples/gallery/dashboards.rs"]
mod dashboards;
#[path = "../examples/gallery/diagram.rs"]
mod diagram;
#[path = "../examples/gallery/distribution.rs"]
mod distribution;
#[path = "../examples/gallery/earth.rs"]
mod earth;
#[path = "../examples/gallery/finance.rs"]
mod finance;
#[path = "../examples/gallery/flame_graph.rs"]
mod flame_graph;
#[path = "../examples/gallery/flow.rs"]
mod flow;
#[path = "../examples/gallery/fractal.rs"]
mod fractal;
#[path = "../examples/gallery/gauge.rs"]
mod gauge;
#[path = "../examples/gallery/heatmap.rs"]
mod heatmap;
#[path = "../examples/gallery/histogram.rs"]
mod histogram;
#[path = "../examples/gallery/hybrid.rs"]
mod hybrid;
#[path = "../examples/gallery/matrix.rs"]
mod matrix;
#[path = "../examples/gallery/node_graph.rs"]
mod node_graph;
#[path = "../examples/gallery/quality.rs"]
mod quality;
#[path = "../examples/gallery/science.rs"]
mod science;
#[path = "../examples/gallery/stat.rs"]
mod stat;
#[path = "../examples/gallery/state_timeline.rs"]
mod state_timeline;
#[path = "../examples/gallery/status_history.rs"]
mod status_history;
#[path = "../examples/gallery/time_series.rs"]
mod time_series;
#[path = "../examples/gallery/traces.rs"]
mod traces;
#[path = "../examples/gallery/trend.rs"]
mod trend;
#[path = "../examples/gallery/xy.rs"]
mod xy;

use blenderui::prelude::*;
pub use registry::page;
use registry::{CHARTS, Example};

struct One(&'static Example);

impl Render for One {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let example = self.0;
        bui_panels::ui::scope(window, cx, || {
            div().class("absolute inset-0 flex flex-col p-2").bg(page().0).text_color(page().1).child((example.render)())
        })
    }
}

/// Pixels that are not the page background.
fn drawn(pixels: &[u32], background: u32) -> usize {
    pixels.iter().filter(|p| **p != background).count()
}

#[test]
fn every_example_renders() {
    Application::headless().run(|cx| {
        blenderui::set_theme("dark");
        let background = {
            let [r, g, b] = [page().0.r, page().0.g, page().0.b].map(|v| (v * 255.).round() as u32);
            (r << 16) | (g << 8) | b
        };
        let mut blank = Vec::new();
        let mut count = 0;
        for chart in CHARTS {
            for example in chart.examples {
                let options = WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(Bounds {
                        origin: point(px(0.), px(0.)),
                        size: size(px(900.), px(520.)),
                    })),
                    ..Default::default()
                };
                let win = cx.open_window(options, move |_, cx| cx.new(|_| One(example))).unwrap();
                cx.draw(win.id);
                cx.simulate_mouse_move(win.id, 450., 260.);
                let (_, _, pixels) = cx.draw(win.id);
                // the title and description are not drawn here: anything is the chart's
                if drawn(&pixels, background) < 400 {
                    blank.push(format!("{} / {} ({} pixels)", chart.name, example.title, drawn(&pixels, background)));
                    if let Ok(dir) = std::env::var("CHART_DUMP") {
                        let mut out = b"P6
900 520
255
".to_vec();
                        for p in &pixels {
                            out.extend_from_slice(&[(p >> 16) as u8, (p >> 8) as u8, *p as u8]);
                        }
                        std::fs::write(format!("{dir}/blank-{}.ppm", blank.len()), out).unwrap();
                    }
                }
                cx.close_window(win.id);
                count += 1;
            }
        }
        assert!(count > 300, "{count} examples");
        assert!(blank.is_empty(), "nothing drawn by: {blank:?}");
    });
}
