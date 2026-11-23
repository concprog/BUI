use blenderui::{Hsla, Painter, Rgba};

use crate::color::{Hsl, mix};
use crate::geom::{Bounds, point};
use crate::path::Path;
use crate::plot::LineStyle;

use crate::plot::Scale;
use crate::thresholds::Steps;

const MAX_STOPS: usize = 16;

#[derive(Clone, Copy)]
pub(crate) struct Stops {
    items: [(f32, Hsla); MAX_STOPS],
    len: usize,
    discrete: bool,
}

impl Stops {
    pub(crate) fn thresholds(
        steps: Steps<'_>,
        range: (f64, f64),
        scale: &Scale,
        alpha: f32,
    ) -> Self {
        let mut stops = Self::empty(true);
        for (index, (value, color)) in steps.steps.iter().enumerate() {
            let value = if index == 0 {
                f64::NEG_INFINITY
            } else {
                steps.value(*value, range)
            };
            stops.push(value, scale, with_alpha(*color, alpha));
        }
        stops
    }

    pub(crate) fn scheme(
        colors: &[Hsla],
        (min, max): (f64, f64),
        scale: &Scale,
        alpha: f32,
    ) -> Self {
        let mut stops = Self::empty(false);
        let last = colors.len().saturating_sub(1).max(1) as f64;
        for (index, color) in colors.iter().enumerate() {
            let value = (max - min).mul_add(index as f64 / last, min);
            stops.push(value, scale, with_alpha(*color, alpha));
        }
        stops
    }

    const fn empty(discrete: bool) -> Self {
        Self { items: [(0., Hsla::TRANSPARENT); MAX_STOPS], len: 0, discrete }
    }

    /// A continuous ramp between two stops.
    const fn ramp(low: (f32, Hsla), high: (f32, Hsla)) -> Self {
        let mut stops = Self::empty(false);
        stops.items[0] = low;
        stops.items[1] = high;
        stops.len = 2;
        stops
    }

    fn push(&mut self, value: f64, scale: &Scale, color: Hsla) {
        if self.len == MAX_STOPS {
            return;
        }
        let at = if value == f64::NEG_INFINITY {
            0.
        } else {
            scale.fraction(value).clamp(0., 1.)
        };
        self.items[self.len] = (if at.is_nan() { 0. } else { at }, color);
        self.len += 1;
    }

    pub(crate) fn color(&self, fraction: f32) -> Option<Hsla> {
        self.items[..self.len]
            .iter()
            .take_while(|(at, _)| *at <= fraction)
            .last()
            .or_else(|| self.items[..self.len].first())
            .map(|(_, color)| *color)
    }

    pub(crate) fn as_slice(&self) -> &[(f32, Hsla)] {
        &self.items[..self.len]
    }
}

#[derive(Clone, Copy)]
#[allow(clippy::large_enum_variant)]
pub(crate) enum Shade {
    Solid(Hsla),
    Vertical(Hsla, Hsla),
    Stops(Stops),
}

pub(crate) fn with_alpha(color: Hsla, alpha: f32) -> Hsla {
    if color.a < 1. { color } else { color.alpha(alpha) }
}

pub(crate) fn hue(color: Hsla, alpha: f32, dark: bool) -> (Hsla, Hsla) {
    let low = darken(spin(color, -25.), 5.);
    let high = brighten(saturate(spin(color, 20.), 20.), 10.);
    let bottom = if dark { darken(low, 10.) } else { low };
    (lighten(high, 10.).alpha(alpha), bottom.alpha(alpha))
}

pub(crate) fn spin(color: Hsla, degrees: f32) -> Hsla {
    let hsl = Hsl::from(color);
    Hsl { h: (hsl.h + degrees / 360.).rem_euclid(1.), ..hsl }.into()
}

pub(crate) fn darken(color: Hsla, amount: f32) -> Hsla {
    lighten(color, -amount)
}

pub(crate) fn lighten(color: Hsla, amount: f32) -> Hsla {
    let hsl = Hsl::from(color);
    Hsl { l: (hsl.l + amount / 100.).clamp(0., 1.), ..hsl }.into()
}

pub(crate) fn saturate(color: Hsla, amount: f32) -> Hsla {
    let hsl = Hsl::from(color);
    Hsl { s: (hsl.s + amount / 100.).clamp(0., 1.), ..hsl }.into()
}

pub(crate) fn brighten(color: Hsla, amount: f32) -> Hsla {
    let lift = |channel: f32| (channel + amount / 100.).clamp(0., 1.);
    Rgba { r: lift(color.r), g: lift(color.g), b: lift(color.b), a: color.a }
}

/// Bands a gradient is drawn in: gpui painted gradients, the painter paints solid colors, so a
/// ramp is that many bands of the color at their middle (finer than the eye tells apart).
const RAMP_BANDS: usize = 24;

pub(crate) fn paint(path: &Path, shade: &Shade, plot: Bounds, painter: &mut Painter) {
    paint_along(path, shade, plot, false, painter);
}

/// Fills `path` with `shade`, its gradient running up the plot (`across`: left to right).
pub(crate) fn paint_along(path: &Path, shade: &Shade, plot: Bounds, across: bool, painter: &mut Painter) {
    match *shade {
        Shade::Solid(color) => path.fill(painter, color),
        Shade::Vertical(top, bottom) => ramp(path, plot, across, (0., bottom), (1., top), painter),
        Shade::Stops(ref stops) => banded(path, stops, plot, across, painter),
    }
}

/// Strokes `path` with `shade` (a gradient picks the color of each band of the plot).
pub(crate) fn stroke_along(
    path: &Path,
    shade: &Shade,
    plot: Bounds,
    across: bool,
    width: f32,
    style: LineStyle,
    painter: &mut Painter,
) {
    match *shade {
        Shade::Solid(color) => path.stroke_styled(painter, color, width, style),
        _ => bands(shade, plot, across, |band, color| {
            painter.with_clip(band.into(), |painter| path.stroke_styled(painter, color, width, style));
        }),
    }
}

/// The part of the plot from `low` to `high` (fractions up it, or across it).
/// Its edges are on whole pixels: bands meeting part way through a pixel would each cover it partly,
/// and a translucent fill would show a seam there.
fn band(plot: Bounds, across: bool, low: f32, high: f32) -> Bounds {
    if across {
        let width = plot.size.width;
        let (from, to) = ((plot.left() + width * low).round(), (plot.left() + width * high).round());
        Bounds::from_corners(point(from, plot.top()), point(to, plot.bottom()))
    } else {
        let height = plot.size.height;
        let (from, to) = ((plot.bottom() - height * high).round(), (plot.bottom() - height * low).round());
        Bounds::from_corners(point(plot.left(), from), point(plot.right(), to))
    }
}

/// Each band of `shade` over the plot and its color, bottom (or left) first.
fn bands(shade: &Shade, plot: Bounds, across: bool, mut each: impl FnMut(Bounds, Hsla)) {
    let ramp = |(from, low): (f32, Hsla), (to, high): (f32, Hsla), each: &mut dyn FnMut(Bounds, Hsla)| {
        if to <= from {
            return;
        }
        let pixels = if across { plot.size.width } else { plot.size.height } * (to - from);
        let count = ((pixels / 2.).ceil() as usize).clamp(1, RAMP_BANDS);
        for index in 0..count {
            let (a, b) = (index as f32 / count as f32, (index + 1) as f32 / count as f32);
            let color = mix(low, high, f32::midpoint(a, b));
            each(band(plot, across, (to - from).mul_add(a, from), (to - from).mul_add(b, from)), color);
        }
    };
    match *shade {
        Shade::Solid(color) => each(band(plot, across, -1., 2.), color),
        Shade::Vertical(top, bottom) => ramp((0., bottom), (1., top), &mut each),
        Shade::Stops(ref stops) => {
            let items = stops.as_slice();
            let Some(first) = items.first() else { return };
            each(band(plot, across, -1., first.0), first.1);
            for pair in items.windows(2) {
                let ((low, from), (high, to)) = (pair[0], pair[1]);
                if stops.discrete {
                    if high > low {
                        each(band(plot, across, low, high), from);
                    }
                } else {
                    ramp((low, from), (high, to), &mut each);
                }
            }
            if let Some(last) = items.last() {
                each(band(plot, across, last.0, 2.), last.1);
            }
        }
    }
}

fn ramp(path: &Path, plot: Bounds, across: bool, low: (f32, Hsla), high: (f32, Hsla), painter: &mut Painter) {
    banded(path, &Stops::ramp(low, high), plot, across, painter);
}

fn banded(path: &Path, stops: &Stops, plot: Bounds, across: bool, painter: &mut Painter) {
    bands(&Shade::Stops(*stops), plot, across, |band, color| path.fill_clipped(painter, band, color));
}
