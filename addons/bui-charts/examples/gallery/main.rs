//! Every chart, with examples: `cargo run --example gallery`.
//!
//! Headless: `cargo run --example gallery -- --screenshot out.ppm [--chart "Gauge"] [--dark]
//! [--size 1280x900]` writes one frame of that chart's page (a PPM).
#![allow(clippy::wildcard_imports, clippy::needless_pass_by_value)]

#[macro_use]
mod registry;
mod app;

mod bar_gauge;
mod candlestick;
mod category;
mod dashboard;
mod dashboards;
mod diagram;
mod distribution;
mod earth;
mod finance;
mod flame_graph;
mod flow;
mod fractal;
mod gauge;
mod heatmap;
mod histogram;
mod hybrid;
mod matrix;
mod node_graph;
mod quality;
mod science;
mod stat;
mod state_timeline;
mod status_history;
mod time_series;
mod traces;
mod trend;
mod xy;

use blenderui::prelude::*;

pub use registry::page;
use app::GalleryApp;
use registry::CHARTS;

fn arg(name: &str) -> Option<String> {
    std::env::args().skip_while(|a| a != name).nth(1)
}

fn main() {
    let screenshot = arg("--screenshot");
    let start = arg("--chart").and_then(|name| CHARTS.iter().position(|chart| chart.name == name)).unwrap_or(0);
    let dark = std::env::args().any(|a| a == "--dark");
    let (width, height) = arg("--size")
        .and_then(|s| s.split_once('x').and_then(|(w, h)| Some((w.parse().ok()?, h.parse().ok()?))))
        .unwrap_or((1280., 900.));
    let app = if screenshot.is_some() { Application::headless() } else { Application::new() };
    app.run(move |cx: &mut App| {
        if screenshot.is_some() {
            blenderui::set_theme(if dark { "dark" } else { "light" });
        }
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds { origin: point(px(0.), px(0.)), size: size(px(width), px(height)) })),
            ..Default::default()
        };
        let alone = screenshot.is_some() && arg("--chart").is_some();
        let window = cx.open_window(options, move |_, cx| cx.new(|_| GalleryApp { start, alone })).expect("a window");
        if let Some(path) = screenshot {
            for _ in 0..4 {
                cx.draw(window.id);
                cx.advance_clock(std::time::Duration::from_millis(300));
            }
            if let Some((x, y)) = arg("--hover").and_then(|s| s.split_once(',').and_then(|(x, y)| Some((x.parse().ok()?, y.parse().ok()?)))) {
                cx.simulate_mouse_move(window.id, x, y);
            }
            let (w, h, pixels) = cx.draw(window.id);
            let mut out = format!("P6\n{w} {h}\n255\n").into_bytes();
            for p in &pixels {
                out.extend_from_slice(&[(p >> 16) as u8, (p >> 8) as u8, *p as u8]);
            }
            std::fs::write(&path, out).expect("writing the screenshot");
            println!("wrote {path} ({w}x{h})");
        }
    });
}
