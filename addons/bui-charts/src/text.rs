//! Chart text on the painter: measured with the UI font, placed as gpui placed a shaped line (the
//! line box `size` tall at `origin`, the font's content area centred in it).

use blenderui::style::TextAlign;
use blenderui::{Hsla, Painter};

use crate::geom::Point;
use crate::kit;

const MEASURE_SIZE: f32 = 14.;
/// The UI font's (Inter's) ascent and descent, in em.
const ASCENT: f32 = 1984. / 2048.;
const DESCENT: f32 = 494. / 2048.;
/// Where the kit's text item puts the baseline below its top, in em.
const ITEM_BASELINE: f32 = 0.72;

pub(crate) fn measure(text: &str, size: f32) -> f32 {
    kit::text_width(text, size)
}

/// The largest size `text` can take to fit `width` and `height`, `line_height` em per line.
pub(crate) fn fit(text: &str, width: f32, height: f32, line_height: f32) -> f32 {
    let by_width = width / (measure(text, MEASURE_SIZE) + 2.) * MEASURE_SIZE;
    by_width.min(height / line_height).max(0.)
}

/// Draws `text` with its line box's top at `origin.y`, aligned to `origin.x`.
pub(crate) fn draw(
    painter: &mut Painter,
    text: &str,
    origin: Point,
    align: TextAlign,
    size: f32,
    color: Hsla,
) {
    if text.is_empty() || size <= 0. || color.is_transparent() {
        return;
    }
    let shift = match align {
        TextAlign::Left | TextAlign::Justify => 0.,
        TextAlign::Right => measure(text, size),
        TextAlign::Center => measure(text, size) / 2.,
    };
    let baseline = (ASCENT - DESCENT).mul_add(0.5, 0.5) * size;
    painter.text(origin.x - shift, origin.y + baseline - ITEM_BASELINE * size, text, size, color);
}
