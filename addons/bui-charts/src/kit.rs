//! The two kit calls blenderui keeps to itself, made here without allocating: text advance widths
//! (blenderui measures text only inside its own layout) and theme colors by a static key
//! (`blenderui::theme_color` copies its key into a new `CString` on every call). Every other kit use
//! goes through blenderui's safe API; this is the crate's only `unsafe`.

use std::cell::Cell;
use std::collections::HashMap;
use std::ffi::CStr;
use std::hash::{BuildHasher, BuildHasherDefault, DefaultHasher};

use blenderui::{Rgba, sys};

/// Widths kept before the cache starts over (labels repeat frame to frame; values come and go).
const WIDTHS_MAX: usize = 8192;

type Widths = HashMap<(u32, u64), f32, BuildHasherDefault<DefaultHasher>>;

thread_local! {
    static WIDTHS: Cell<Widths> = Cell::new(Widths::default());
}

/// The advance width of `text` in the UI font at `size` logical pixels.
pub(crate) fn text_width(text: &str, size: f32) -> f32 {
    if text.is_empty() || size <= 0. {
        return 0.;
    }
    let key = (size.to_bits(), BuildHasherDefault::<DefaultHasher>::default().hash_one(text));
    WIDTHS.with(|cell| {
        let mut widths = cell.take();
        let width = *widths.entry(key).or_insert_with(|| measure(text, size));
        if widths.len() > WIDTHS_MAX {
            widths.clear();
        }
        cell.set(widths);
        width
    })
}

fn measure(text: &str, size: f32) -> f32 {
    let mut width = 0.;
    let len = i32::try_from(text.len()).unwrap_or(i32::MAX);
    // SAFETY: the family is a NUL-terminated static (the UI font), `text` is valid for `len` bytes
    // and `width` is the one float the kit writes.
    let ok = unsafe { sys::BUI_text_measure(c"".as_ptr(), 0, size, text.as_ptr().cast(), len, &raw mut width) };
    if ok == 0 {
        // no text engine (as blenderui's layout falls back)
        return text.chars().count() as f32 * size * 0.55;
    }
    width
}

/// A number that changes whenever a theme value does.
pub(crate) fn theme_generation() -> i32 {
    // SAFETY: reads a counter.
    unsafe { sys::BK_theme_generation() }
}

/// A color of the active kit theme (`"text"`, `"region_bg"`, `"tooltip.inner"` ...; see
/// `BK_theme_color`).
pub(crate) fn theme_color(key: &CStr) -> Option<Rgba> {
    let mut out = [0f32; 4];
    // SAFETY: a NUL-terminated key and the four floats the kit writes.
    let ok = unsafe { sys::BK_theme_color(key.as_ptr(), out.as_mut_ptr()) };
    (ok != 0).then_some(Rgba { r: out[0], g: out[1], b: out[2], a: out[3] })
}

/// Whether the active kit theme is a dark one (the chart palette follows it).
pub(crate) fn dark() -> bool {
    blenderui::theme_name() == "dark"
}
