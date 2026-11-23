use std::sync::LazyLock;

use blenderui::{Hsla, Rgba, hsla};

use crate::color::Hsl;
use regex::Regex;

const PACKAGE_PATTERNS: [&str; 10] = [
    r"^(?<p>([^/]*/)*)(.*\.php+)(.*)$",
    r"^(?<p>([^/]*/)*)(.*\.py+)(.*)$",
    r"^(?<p>([^/]*/)*)(.*\.rb+)(.*)$",
    r"^(\./node_modules/)?(?<p>[^/]*)(.*\.?(jsx?|tsx?)?):(.*):(.*)$",
    r"^(?<p>.*?/.*?\.|.*?\.|.+)(.*)$",
    r"^(?<p>.+/)(.+\.)(.+)$",
    r"^(?<p>.+)\.(.+)\.(.+)\(.*\)$",
    r"^(?<p>.+?):.*$",
    r"^(?<p>[^:]+)",
    r"^(?<p>.+)$",
];

static PACKAGES: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    PACKAGE_PATTERNS
        .iter()
        .filter_map(|pattern| Regex::new(pattern).ok())
        .collect()
});

const HSL_PALETTE: [(f32, f32, f32); 8] = [
    (24., 69., 60.),
    (34., 65., 65.),
    (194., 52., 61.),
    (163., 45., 55.),
    (211., 48., 60.),
    (246., 40., 65.),
    (305., 63., 79.),
    (47., 100., 73.),
];

const RGB_PALETTE: [u32; 16] = [
    0xb7dbab, 0xf4d598, 0x4e92f9, 0xf9ba8f, 0xf29191, 0x82b5d8, 0xe5a8e2, 0xaea2e0, 0x9ac48a,
    0xf2c96d, 0x65c5db, 0xf9934e, 0xea6460, 0x5195ce, 0xd683ce, 0x806eb7,
];

pub(crate) const PALETTE_SIZE: usize = HSL_PALETTE.len() + RGB_PALETTE.len();

pub(crate) const COLOR_BLIND: [u32; 3] = [0x1a85ff, 0x948e8e, 0xdc3220];

pub(crate) fn palette(index: usize) -> Hsla {
    match HSL_PALETTE.get(index) {
        Some(&(h, s, l)) => hsla(h / 360., s / 100., l / 100., 1.),
        None => blenderui::rgb(RGB_PALETTE[index - HSL_PALETTE.len()]),
    }
}

pub(crate) fn package_name(label: &str) -> &str {
    PACKAGES
        .iter()
        .find_map(|pattern| pattern.captures(label)?.name("p"))
        .map_or("", |name| name.as_str())
}

pub(crate) fn package_color(label: &str, light: bool) -> Hsla {
    let color = palette(murmur3(package_name(label)) as usize % PALETTE_SIZE);
    if light { brighten(color, 0.15) } else { color }
}

pub(crate) fn value_color(value: f64, total: f64, (min, max): (f64, f64)) -> Hsla {
    let intensity = (value / total / (max - min)).min(1.) as f32;
    hsla(
        (50. - 50. * intensity) / 360.,
        1.,
        7.0f32.mul_add(intensity, 65.) / 100.,
        1.,
    )
}

pub(crate) fn diff_color(left: f64, right: f64, totals: (f64, f64), scale: [Hsla; 3]) -> Hsla {
    let (total_left, total_right) = totals;
    if total_left == 0. || total_right == 0. {
        return scale[1];
    }
    let percent = |ticks: f64, total: f64| (10_000. * ticks / total).round() / 100.;
    let (before, after) = (percent(left, total_left), percent(right, total_right));
    let change = ((after - before) / before * 100.).clamp(-100., 100.) as f32;
    if change < 0. {
        mix(scale[1], scale[0], -change / 100.)
    } else {
        mix(scale[1], scale[2], change / 100.)
    }
}

pub(crate) fn lighten(color: Hsla, amount: f32) -> Hsla {
    let hsl = Hsl::from(color);
    Hsl { l: (hsl.l + amount).clamp(0., 1.), ..hsl }.into()
}

fn brighten(color: Hsla, amount: f32) -> Hsla {
    let channel = |value: f32| (value + amount).clamp(0., 1.);
    Rgba { r: channel(color.r), g: channel(color.g), b: channel(color.b), a: color.a }
}

fn mix(from: Hsla, to: Hsla, t: f32) -> Hsla {
    Hsla { a: 1., ..crate::color::mix(from, to, t) }
}

pub(crate) fn murmur3(key: &str) -> u32 {
    const C1: u32 = 0xcc9e_2d51;
    const C2: u32 = 0x1b87_3593;
    let scramble = |k: u32| k.wrapping_mul(C1).rotate_left(15).wrapping_mul(C2);
    let mut hash = 0u32;
    let mut length = 0u32;
    let mut block = 0u32;
    let mut filled = 0;
    for unit in key.encode_utf16() {
        block |= u32::from(unit & 0xff) << (8 * filled);
        filled += 1;
        length += 1;
        if filled == 4 {
            hash = (hash ^ scramble(block))
                .rotate_left(13)
                .wrapping_mul(5)
                .wrapping_add(0xe654_6b64);
            block = 0;
            filled = 0;
        }
    }
    if filled > 0 {
        hash ^= scramble(block);
    }
    hash ^= length;
    hash ^= hash >> 16;
    hash = hash.wrapping_mul(0x85eb_ca6b);
    hash ^= hash >> 13;
    hash = hash.wrapping_mul(0xc2b2_ae35);
    hash ^ (hash >> 16)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn murmur3_matches_the_reference() {
        assert_eq!(murmur3(""), 0);
        assert_eq!(murmur3("hello"), 0x248b_fa47);
        assert_eq!(murmur3("hello, world"), 0x149b_bb7f);
    }

    #[test]
    fn package_names_follow_grafana() {
        assert_eq!(package_name("net/http.(*conn).serve"), "net/http.");
        assert_eq!(package_name("runtime.mallocgc"), "runtime.");
        assert_eq!(package_name("lex"), "lex");
    }

    #[test]
    fn value_colors_run_from_yellow_to_red() {
        let cold = value_color(0., 100., (0., 1.));
        let hot = value_color(100., 100., (0., 1.));
        let (cold, hot) = (Hsl::from(cold), Hsl::from(hot));
        assert!(cold.h > hot.h);
        assert!(hot.h.abs() < 1e-6);
    }
}
