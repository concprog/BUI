//! Per-glyph font fallback at scale: text in a script the UI font lacks,
//! at many sizes and both weights, finds its fallback face every time (each
//! size and weight of a fallback face takes a font slot; with too few slots
//! the glyphs past the last one drew as empty boxes).

use std::path::PathBuf;

use blenderui::prelude::*;
use blenderui::sys;

/// The fallback fonts of a script, most wanted first (as `gpu/intern/gpu_fonts.c` lists them).
const KOREAN: &[&str] = &["malgun.ttf", "AppleSDGothicNeo.ttc", "NotoSansCJK-Regular.ttc", "NotoSansKR-Regular.otf",
                          "NotoSansCJKkr-Regular.otf"];
const CHINESE: &[&str] = &["msyh.ttc", "YuGothM.ttc", "simsun.ttc", "PingFang.ttc", "Hiragino Sans GB.ttc",
                           "NotoSansCJK-Regular.ttc", "NotoSansSC-Regular.otf", "NotoSansCJKsc-Regular.otf"];
const JAPANESE: &[&str] = &["YuGothM.ttc", "meiryo.ttc", "msgothic.ttc", "Hiragino Sans GB.ttc", "NotoSansCJK-Regular.ttc",
                            "NotoSansJP-Regular.otf", "NotoSansCJKjp-Regular.otf"];

/// The first of `names` in the font folders the text engine reads fallback files from, the
/// way it opens them (`<folder>/<name>`), found apart from its glyph fallback (so a fallback
/// that fails can't make a test skip itself).
fn installed(names: &[&'static str]) -> Option<&'static str> {
    let mut dirs = Vec::new();
    let mut buf = [0u8; 600];
    for i in 0.. {
        // SAFETY: `buf` is writable for its length.
        let n = unsafe { sys::BUI_text_font_dir(i, buf.as_mut_ptr().cast(), buf.len() as i32) };
        if n <= 0 {
            break;
        }
        dirs.push(PathBuf::from(String::from_utf8_lossy(&buf[..n as usize]).into_owned()));
    }
    names.iter().copied().find(|n| dirs.iter().any(|d| d.join(n).is_file()))
}

/// The advance of `s` at `size` px (bold or not), in the default UI font.
fn width(s: &str, size: f32, bold: bool) -> f32 {
    let mut w = 0.0f32;
    let flags = if bold { sys::BUI_FONT_BOLD } else { 0 };
    // SAFETY: the pointers are valid for the call; the family is a static empty string.
    let ok = unsafe { sys::BUI_text_measure(c"".as_ptr(), flags, size, s.as_ptr().cast(), s.len() as i32, &raw mut w) };
    assert_ne!(ok, 0, "a text engine");
    w
}

/// A Hangul syllable is about an em wide in any Korean face; the UI font's
/// empty box (what a glyph with no face draws) is half that.
#[test]
fn fallback_faces_never_run_out() {
    // the text engine is the UI thread's: inside an app (one at a time)
    Application::headless().run(|_| {
        let hangul = "\u{d55c}"; // 한
        if installed(KOREAN).is_none() {
            eprintln!("no Korean font installed: nothing to check");
            return;
        }
        assert!(width(hangul, 40.0, false) >= 30.0, "Hangul falls back to the installed Korean font");
        // one slot first, so each size's two (the UI font's, then the Korean
        // face's) straddle the end of the table at some point; then 2 weights x
        // 120 sizes, far more slots than the table holds
        width("x", 7.0, false);
        for bold in [false, true] {
            for size in 10..130 {
                let size = size as f32;
                let w = width(hangul, size, bold);
                assert!(w > size * 0.75, "Hangul at {size} px (bold {bold}) is {w} px wide: an empty box");
            }
        }
    });
}

struct Line {
    text: &'static str,
}
impl Render for Line {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().id("line").text_size(px(48.0)).child(self.text)
    }
}

/// The pixels of `text` alone in a fresh 160 x 80 window.
fn drawn(cx: &mut App, text: &'static str) -> Vec<u32> {
    let opts = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(Bounds { origin: point(px(0.0), px(0.0)), size: size(px(160.0), px(80.0)) })),
        ..Default::default()
    };
    let win = cx.open_window(opts, |_, cx| cx.new(|_| Line { text })).unwrap();
    let (_, _, pixels) = cx.draw(win.id);
    cx.close_window(win.id);
    pixels
}

/// Han characters are drawn in the face of the run they are in (Japanese
/// with kana, Korean with Hangul, else Chinese), not in the face that last
/// drew one: a Chinese line looks the same after a Japanese one.
#[test]
fn a_runs_han_face_is_its_own() {
    Application::headless().run(|cx| {
        // a Chinese face and a Japanese one, apart (one CJK collection for both: nothing to tell apart)
        let (Some(zh), Some(ja)) = (installed(CHINESE), installed(JAPANESE)) else {
            eprintln!("no Chinese or no Japanese font installed: nothing to check");
            return;
        };
        if zh == ja {
            eprintln!("one font draws Chinese and Japanese here ({zh}): nothing to check");
            return;
        }
        assert!(width("\u{9aa8}", 40.0, false) >= 30.0, "Han falls back to an installed CJK font");
        // 骨: its Chinese and Japanese forms differ
        let chinese = drawn(cx, "\u{9aa8}");
        let japanese = drawn(cx, "\u{9aa8}\u{3042}"); // 骨あ: Japanese
        // the first glyph's columns (8 px body margin, 48 px glyph)
        let first = |px: &[u32]| px.chunks(160).flat_map(|row| row[..60].to_vec()).collect::<Vec<u32>>();
        assert!(first(&chinese) != first(&japanese), "a Japanese run draws 骨 in {ja}, a Chinese one in {zh}");
        assert!(chinese == drawn(cx, "\u{9aa8}"), "the Chinese line changed after a Japanese one");
    });
}
