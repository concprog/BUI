//! Hue, saturation and lightness of blenderui's colors: `Hsla` is blenderui's name for its `Rgba`
//! (straight alpha), so the shading the charts do in HSL (lighten, spin, saturate) converts here.

use std::hash::{Hash, Hasher};

use blenderui::Rgba;

/// A color as hue, saturation, lightness and alpha, each 0..=1 (gpui's `Hsla` fields).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Hsl {
    pub h: f32,
    pub s: f32,
    pub l: f32,
    pub a: f32,
}

impl From<Rgba> for Hsl {
    fn from(c: Rgba) -> Self {
        let (max, min) = (c.r.max(c.g).max(c.b), c.r.min(c.g).min(c.b));
        let l = f32::midpoint(max, min);
        let delta = max - min;
        if delta <= f32::EPSILON {
            return Self { h: 0., s: 0., l, a: c.a };
        }
        let s = if l > 0.5 { delta / (2. - max - min) } else { delta / (max + min) };
        let h = if (max - c.r).abs() <= f32::EPSILON {
            (c.g - c.b) / delta + if c.g < c.b { 6. } else { 0. }
        } else if (max - c.g).abs() <= f32::EPSILON {
            (c.b - c.r) / delta + 2.
        } else {
            (c.r - c.g) / delta + 4.
        };
        Self { h: h / 6., s, l, a: c.a }
    }
}

impl From<Hsl> for Rgba {
    fn from(c: Hsl) -> Self {
        blenderui::hsla(c.h, c.s, c.l, c.a)
    }
}

/// The color part way from `a` to `b` (`t` 0..=1), channel by channel.
pub(crate) fn mix(a: Rgba, b: Rgba, t: f32) -> Rgba {
    let lerp = |x: f32, y: f32| (y - x).mul_add(t, x);
    Rgba { r: lerp(a.r, b.r), g: lerp(a.g, b.g), b: lerp(a.b, b.b), a: lerp(a.a, b.a) }
}

/// WCAG relative luminance.
pub(crate) fn luminance(color: Rgba) -> f32 {
    let channel = |c: f32| if c <= 0.03928 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) };
    0.0722f32.mul_add(channel(color.b), 0.2126f32.mul_add(channel(color.r), 0.7152 * channel(color.g)))
}

/// Hashes a color by its bits (cache keys).
pub(crate) fn hash<H: Hasher>(color: &Rgba, state: &mut H) {
    [color.r, color.g, color.b, color.a].map(f32::to_bits).hash(state);
}

/// gpui-kit's `Colorize::lighten`: lightness scaled up by `factor` (0..=1).
pub(crate) fn lighter(color: Rgba, factor: f32) -> Rgba {
    let mut hsl = Hsl::from(color);
    hsl.l = (hsl.l * (1. + factor.clamp(0., 1.))).min(1.);
    hsl.into()
}

/// gpui-kit's `Colorize::darken`: lightness scaled down by `factor` (0..=1).
pub(crate) fn darker(color: Rgba, factor: f32) -> Rgba {
    let mut hsl = Hsl::from(color);
    hsl.l *= 1. - factor.clamp(0., 1.);
    hsl.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hsl_round_trips() {
        for hex in [0x1e66f5, 0xd20f39, 0x40a02b, 0x808080, 0xffffff, 0x000000, 0xdf8e1d] {
            let rgb = blenderui::rgb(hex);
            let back = Rgba::from(Hsl::from(rgb));
            for (a, b) in [(rgb.r, back.r), (rgb.g, back.g), (rgb.b, back.b)] {
                assert!((a - b).abs() < 1e-4, "{hex:06x}");
            }
        }
        assert!((Hsl::from(blenderui::rgb(0xff0000)).h).abs() < 1e-6);
        assert!((Hsl::from(blenderui::rgb(0x0000ff)).h - 2. / 3.).abs() < 1e-6);
    }
}
