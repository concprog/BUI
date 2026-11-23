use std::f64::consts::PI;

use blenderui::{Hsla, Rgba};

use super::d3_data::{BASIS, RAMPS};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum HeatmapScheme {
    BrBG,
    PiYG,
    PRGn,
    PuOr,
    RdBu,
    RdGy,
    RdYlBu,
    RdYlGn,
    Spectral,
    Blues,
    Greens,
    Greys,
    #[default]
    Oranges,
    Purples,
    Reds,
    Turbo,
    Cividis,
    Viridis,
    Magma,
    Inferno,
    Plasma,
    Warm,
    Cool,
    Cubehelix,
    BuGn,
    BuPu,
    GnBu,
    OrRd,
    PuBuGn,
    PuBu,
    PuRd,
    RdPu,
    YlGnBu,
    YlGn,
    YlOrBr,
    YlOrRd,
    Rainbow,
    Sinebow,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Invert {
    Always,
    Dark,
    Light,
}

type Rgb = [f64; 3];

const A: f64 = -0.148_61;
const B: f64 = 1.782_77;
const C: f64 = -0.292_27;
const D: f64 = -0.906_49;
const E: f64 = 1.972_94;

fn channel(hex: &str, index: usize, offset: usize) -> f64 {
    let at = index * 6 + offset * 2;
    hex.get(at..at + 2)
        .and_then(|pair| u8::from_str_radix(pair, 16).ok())
        .map_or(0., f64::from)
}

fn basis(t1: f64, v0: f64, v1: f64, v2: f64, v3: f64) -> f64 {
    let (t2, t3) = (t1 * t1, t1 * t1 * t1);
    ((1. - 3. * t1 + 3. * t2 - t3) * v0
        + (4. - 6. * t2 + 3. * t3) * v1
        + (1. + 3. * t1 + 3. * t2 - 3. * t3) * v2
        + t3 * v3)
        / 6.
}

fn spline(hex: &str, t: f64) -> Rgb {
    let n = hex.len() / 6 - 1;
    let (i, t) = if t <= 0. {
        (0, 0.)
    } else if t >= 1. {
        (n - 1, 1.)
    } else {
        ((t * n as f64).floor() as usize, t)
    };
    std::array::from_fn(|offset| {
        let value = |index: usize| channel(hex, index, offset);
        let (v1, v2) = (value(i), value(i + 1));
        let v0 = if i > 0 { value(i - 1) } else { 2. * v1 - v2 };
        let v3 = if i < n - 1 {
            value(i + 2)
        } else {
            2. * v2 - v1
        };
        basis((t - i as f64 / n as f64) * n as f64, v0, v1, v2, v3)
    })
}

fn ramp(hex: &str, t: f64) -> Rgb {
    let n = hex.len() / 6;
    let index = ((t * n as f64).floor().max(0.) as usize).min(n - 1);
    std::array::from_fn(|offset| channel(hex, index, offset))
}

fn cubehelix(h: f64, s: f64, l: f64) -> Rgb {
    let h = (h + 120.).to_radians();
    let a = s * l * (1. - l);
    let (cos, sin) = (h.cos(), h.sin());
    [
        255. * a.mul_add(A.mul_add(cos, B * sin), l),
        255. * a.mul_add(C.mul_add(cos, D * sin), l),
        255. * a.mul_add(E * cos, l),
    ]
}

fn cubehelix_long(from: Rgb, to: Rgb, t: f64) -> Rgb {
    let lerp = |a: f64, b: f64| (b - a).mul_add(t, a);
    cubehelix(
        lerp(from[0], to[0]),
        lerp(from[1], to[1]),
        lerp(from[2], to[2]),
    )
}

fn polynomial(t: f64, coefficients: [[f64; 6]; 3]) -> Rgb {
    let t = t.clamp(0., 1.);
    coefficients.map(|c| {
        c[5].mul_add(t, c[4])
            .mul_add(t, c[3])
            .mul_add(t, c[2])
            .mul_add(t, c[1])
            .mul_add(t, c[0])
    })
}

impl HeatmapScheme {
    pub const ALL: [Self; 38] = [
        Self::BrBG,
        Self::PiYG,
        Self::PRGn,
        Self::PuOr,
        Self::RdBu,
        Self::RdGy,
        Self::RdYlBu,
        Self::RdYlGn,
        Self::Spectral,
        Self::Blues,
        Self::Greens,
        Self::Greys,
        Self::Oranges,
        Self::Purples,
        Self::Reds,
        Self::Turbo,
        Self::Cividis,
        Self::Viridis,
        Self::Magma,
        Self::Inferno,
        Self::Plasma,
        Self::Warm,
        Self::Cool,
        Self::Cubehelix,
        Self::BuGn,
        Self::BuPu,
        Self::GnBu,
        Self::OrRd,
        Self::PuBuGn,
        Self::PuBu,
        Self::PuRd,
        Self::RdPu,
        Self::YlGnBu,
        Self::YlGn,
        Self::YlOrBr,
        Self::YlOrRd,
        Self::Rainbow,
        Self::Sinebow,
    ];

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::BrBG => "BrBG",
            Self::PiYG => "PiYG",
            Self::PRGn => "PRGn",
            Self::PuOr => "PuOr",
            Self::RdBu => "RdBu",
            Self::RdGy => "RdGy",
            Self::RdYlBu => "RdYlBu",
            Self::RdYlGn => "RdYlGn",
            Self::Spectral => "Spectral",
            Self::Blues => "Blues",
            Self::Greens => "Greens",
            Self::Greys => "Greys",
            Self::Oranges => "Oranges",
            Self::Purples => "Purples",
            Self::Reds => "Reds",
            Self::Turbo => "Turbo",
            Self::Cividis => "Cividis",
            Self::Viridis => "Viridis",
            Self::Magma => "Magma",
            Self::Inferno => "Inferno",
            Self::Plasma => "Plasma",
            Self::Warm => "Warm",
            Self::Cool => "Cool",
            Self::Cubehelix => "Cubehelix",
            Self::BuGn => "BuGn",
            Self::BuPu => "BuPu",
            Self::GnBu => "GnBu",
            Self::OrRd => "OrRd",
            Self::PuBuGn => "PuBuGn",
            Self::PuBu => "PuBu",
            Self::PuRd => "PuRd",
            Self::RdPu => "RdPu",
            Self::YlGnBu => "YlGnBu",
            Self::YlGn => "YlGn",
            Self::YlOrBr => "YlOrBr",
            Self::YlOrRd => "YlOrRd",
            Self::Rainbow => "Rainbow",
            Self::Sinebow => "Sinebow",
        }
    }

    const fn invert(self) -> Invert {
        match self {
            Self::BrBG
            | Self::PiYG
            | Self::PRGn
            | Self::PuOr
            | Self::RdBu
            | Self::RdGy
            | Self::RdYlBu
            | Self::RdYlGn
            | Self::Spectral
            | Self::Rainbow
            | Self::Sinebow => Invert::Always,
            Self::Turbo
            | Self::Cividis
            | Self::Viridis
            | Self::Magma
            | Self::Inferno
            | Self::Plasma
            | Self::Warm
            | Self::Cool
            | Self::Cubehelix => Invert::Light,
            _ => Invert::Dark,
        }
    }

    fn rgb(self, t: f64) -> Rgb {
        let table = |entries: &[(&str, &'static str)], name: &str| {
            entries
                .iter()
                .find(|(key, _)| key.eq_ignore_ascii_case(name))
                .map_or("000000000000", |(_, hex)| *hex)
        };
        match self {
            Self::Viridis | Self::Magma | Self::Inferno | Self::Plasma => {
                ramp(table(&RAMPS, self.name()), t)
            }
            Self::Turbo => polynomial(
                t,
                [
                    [34.61, 1172.33, -10793.56, 33300.12, -38394.49, 14825.05],
                    [23.31, 557.33, 1225.33, -3574.96, 1073.77, 707.56],
                    [27.2, 3211.1, -15327.97, 27814., -22569.18, 6838.66],
                ],
            ),
            Self::Cividis => polynomial(
                t,
                [
                    [-4.54, -35.34, 2381.73, -6402.7, 7024.72, -2710.57],
                    [32.49, 170.73, 52.82, -131.46, 176.58, -67.37],
                    [81.24, 442.36, -2482.43, 6167.24, -6614.94, 2475.67],
                ],
            ),
            Self::Cubehelix => cubehelix_long([300., 0.5, 0.], [-240., 0.5, 1.], t),
            Self::Warm => cubehelix_long([-100., 0.75, 0.35], [80., 1.5, 0.8], t),
            Self::Cool => cubehelix_long([260., 0.75, 0.35], [80., 1.5, 0.8], t),
            Self::Rainbow => {
                let t = if (0. ..=1.).contains(&t) {
                    t
                } else {
                    t - t.floor()
                };
                let ts = (t - 0.5).abs();
                cubehelix(
                    360f64.mul_add(t, -100.),
                    1.5f64.mul_add(-ts, 1.5),
                    0.9f64.mul_add(-ts, 0.8),
                )
            }
            Self::Sinebow => {
                let t = (0.5 - t) * PI;
                [0., PI / 3., PI * 2. / 3.].map(|shift| 255. * (t + shift).sin().powi(2))
            }
            basis_scheme => spline(table(&BASIS, basis_scheme.name()), t),
        }
    }

    pub(crate) fn sample(self, t: f64) -> Hsla {
        let [r, g, b] = self
            .rgb(t)
            .map(|value| (value.round().clamp(0., 255.) / 255.) as f32);
        Rgba { r, g, b, a: 1. }
    }

    pub(crate) fn quantize(self, steps: usize, dark: bool, reverse: bool, out: &mut Vec<Hsla>) {
        out.clear();
        let last = steps.max(2) - 1;
        out.extend((0..=last).map(|index| self.sample(index as f64 / last as f64)));
        let invert = match self.invert() {
            Invert::Always => true,
            Invert::Dark => dark,
            Invert::Light => !dark,
        };
        if invert != reverse {
            out.reverse();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(scheme: HeatmapScheme, t: f64) -> String {
        let rgb = scheme.sample(t);
        [rgb.r, rgb.g, rgb.b]
            .map(|channel| format!("{:02x}", (channel * 255.).round() as u8))
            .concat()
    }

    #[test]
    fn matches_d3_scale_chromatic() {
        let expected = [
            (
                HeatmapScheme::Spectral,
                ["9e0142", "dd4b4a", "fbf8b0", "4799b3", "5e4fa2"],
            ),
            (
                HeatmapScheme::Blues,
                ["f7fbff", "ddeaf7", "6daed5", "0d5299", "08306b"],
            ),
            (
                HeatmapScheme::Oranges,
                ["fff5eb", "fee4ca", "fb8d3d", "aa3703", "7f2704"],
            ),
            (
                HeatmapScheme::Viridis,
                ["440154", "472e7c", "21918c", "a8db34", "fde725"],
            ),
            (
                HeatmapScheme::Magma,
                ["000004", "1e1149", "b73779", "fec185", "fcfdbf"],
            ),
            (
                HeatmapScheme::Turbo,
                ["23171b", "436df1", "95fb51", "ce310d", "900c00"],
            ),
            (
                HeatmapScheme::Cividis,
                ["002051", "13376d", "7f7c75", "e1ca5d", "fdea45"],
            ),
            (
                HeatmapScheme::Cubehelix,
                ["000000", "1a1f3d", "a07949", "c9e5f0", "ffffff"],
            ),
            (
                HeatmapScheme::Warm,
                ["6e40aa", "a23db3", "ff5e63", "cecd36", "aff05b"],
            ),
            (
                HeatmapScheme::Cool,
                ["6e40aa", "5b5ccf", "1ac7c2", "71f65b", "aff05b"],
            ),
            (
                HeatmapScheme::Rainbow,
                ["6e40aa", "d63ea6", "aff05b", "3f80e1", "6e40aa"],
            ),
            (
                HeatmapScheme::Sinebow,
                ["ff4040", "d7a403", "00bfbf", "d703a4", "ff4040"],
            ),
            (
                HeatmapScheme::RdYlGn,
                ["a50026", "de4331", "f9f7ae", "34a255", "006837"],
            ),
            (
                HeatmapScheme::YlOrRd,
                ["ffffcc", "ffec9f", "fd893c", "bb0524", "800026"],
            ),
        ];
        for (scheme, colors) in expected {
            for (t, color) in [0., 0.13, 0.5, 0.87, 1.].into_iter().zip(colors) {
                assert_eq!(hex(scheme, t), color, "{} at {t}", scheme.name());
            }
        }
    }
}
