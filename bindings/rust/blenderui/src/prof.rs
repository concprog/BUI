//! `BUI_PROFILE=1`: per-phase frame timings, averaged and printed to stderr
//! every 60 frames (`BUI_PROFILE=each`: every frame, with its total). Costs
//! one thread-local bool check when off.

use std::cell::Cell;
use std::time::Instant;

/// The timed phases of a frame.
#[derive(Clone, Copy)]
pub(crate) enum Phase {
    FrameInterval,
    Render,
    Cascade,
    Build,
    Boxes,
    Taffy,
    Place,
    Paint,
    Flush,
    Widgets,
}

const NAMES: [&str; 10] = [
    "frame interval", "render", "  cascade", "build", "boxes", "taffy", "place", "paint", "flush",
    "widgets+hover",
];

thread_local! {
    static ON: bool = std::env::var_os("BUI_PROFILE").is_some();
    static EACH: u32 = if std::env::var_os("BUI_PROFILE").is_some_and(|v| v == "each") { 1 } else { 60 };
    static ACC: [Cell<f64>; 10] = const { [const { Cell::new(0.0) }; 10] };
    static FRAMES: Cell<u32> = const { Cell::new(0) };
}

pub(crate) fn on() -> bool {
    ON.with(|o| *o)
}

/// Times its scope into a phase.
pub(crate) struct Span(Option<(Phase, Instant)>);

pub(crate) fn span(phase: Phase) -> Span {
    Span(on().then(|| (phase, Instant::now())))
}

impl Drop for Span {
    fn drop(&mut self) {
        if let Some((phase, t)) = self.0 {
            add(phase, t.elapsed().as_secs_f64() * 1000.0);
        }
    }
}

/// Add `ms` to a phase.
pub(crate) fn add(phase: Phase, ms: f64) {
    ACC.with(|a| {
        let c = &a[phase as usize];
        c.set(c.get() + ms);
    });
}

/// End of a frame: print the averages every 60 frames (every frame for `each`).
pub(crate) fn frame_done() {
    if !on() {
        return;
    }
    let every = EACH.with(|e| *e);
    let n = FRAMES.with(|f| {
        f.set(f.get() + 1);
        f.get()
    });
    if n < every {
        return;
    }
    FRAMES.with(|f| f.set(0));
    let per = f64::from(every);
    let mut total = 0.0;
    let line: Vec<String> = ACC.with(|a| {
        NAMES.iter().zip(a.iter())
            .filter(|(_, c)| c.get() > 0.0)
            .map(|(name, c)| {
                let ms = c.replace(0.0) / per;
                // cascade is part of render; the interval is between frames
                total += if name.starts_with(' ') || *name == NAMES[0] { 0.0 } else { ms };
                format!("{name} {ms:.2}")
            })
            .collect()
    });
    eprintln!("frame ms: {}  = {total:.2}", line.join("  "));
}
