use std::borrow::Cow;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::mem::discriminant;
use std::ops::Deref;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock, RwLock};

use anyhow::{Context as _, Result};
use blenderui::{Hsla, SharedString};
use serde::Deserialize;

use crate::color::{self, mix};
use crate::kit;

const CATPPUCCIN: &str = include_str!("../themes/catppuccin.json");
const GRADIENT_STEPS: usize = 32;
pub(crate) const NO_TONE: u8 = u8::MAX;

static GENERATION: AtomicU64 = AtomicU64::new(0);

static BUNDLED: LazyLock<Arc<ChartTheme>> =
    LazyLock::new(|| Arc::new(ChartTheme::parse(CATPPUCCIN).expect("the bundled chart theme is valid")));

/// The theme [`ChartTheme::load`] set (the bundled one until then).
static LOADED: RwLock<Option<Arc<ChartTheme>>> = RwLock::new(None);

#[cfg(test)]
pub(crate) fn bundled() -> &'static ChartColors {
    &BUNDLED.light
}

#[derive(Deserialize)]
struct ThemeSet {
    themes: Vec<ThemeConfig>,
}

#[derive(Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum Mode {
    Light,
    Dark,
}

fn good() -> String {
    "#73bf69".into()
}

fn bad() -> String {
    "#f2495c".into()
}

#[derive(Deserialize)]
struct ThemeConfig {
    mode: Mode,
    #[serde(default)]
    colors: HashMap<String, String>,
    palette: Vec<String>,
    #[serde(default)]
    schemes: HashMap<String, Vec<String>>,
    fallback: String,
    #[serde(default = "good")]
    good: String,
    #[serde(default = "bad")]
    bad: String,
    #[serde(default)]
    neutral: Option<String>,
    hover: String,
}

/// A color as theme files write it: CSS (`#rrggbb`, `#rrggbbaa`, `rgb(..)`, names) or a Tailwind
/// palette name with an optional opacity (`blue-500`, `red-600/50`).
fn parse_color(text: &str) -> Option<Hsla> {
    if let Some(color) = blenderui::parse_color(text) {
        return Some(color);
    }
    let (name, opacity) = match text.split_once('/') {
        Some((name, percent)) => (name, percent.trim().parse::<f32>().ok()? / 100.),
        None => (text, 1.),
    };
    let rule = blenderui::tailwind::compile(&format!("bg-{}", name.trim()))?.css;
    let value = rule.split("background-color:").nth(1)?.split([';', '}']).next()?;
    blenderui::parse_color(value).map(|color| color.opacity(opacity))
}

impl ThemeConfig {
    fn color(&self, name: &str) -> Result<Hsla> {
        let value = self.colors.get(name).map_or(name, String::as_str);
        parse_color(value).with_context(|| format!("invalid chart color `{name}`"))
    }

    fn colors(&self, names: &[String]) -> Result<Vec<Hsla>> {
        names.iter().map(|name| self.color(name)).collect()
    }

    fn resolve(&self) -> Result<ChartColors> {
        Ok(ChartColors {
            palette: self.colors(&self.palette)?,
            schemes: self
                .schemes
                .iter()
                .map(|(name, stops)| Ok((name.clone().into(), self.colors(stops)?)))
                .collect::<Result<_>>()?,
            fallback: self.color(&self.fallback)?,
            good: self.color(&self.good)?,
            bad: self.color(&self.bad)?,
            neutral: self.color(self.neutral.as_deref().unwrap_or(&self.fallback))?,
            hover: self.color(&self.hover)?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct ChartColors {
    pub palette: Vec<Hsla>,
    pub schemes: HashMap<SharedString, Vec<Hsla>>,
    pub fallback: Hsla,
    pub good: Hsla,
    pub bad: Hsla,
    pub neutral: Hsla,
    pub hover: Hsla,
}

impl ChartColors {
    #[must_use]
    pub fn series(&self, index: usize) -> Hsla {
        match self.palette.len() {
            0 => self.fallback,
            len => self.palette[index % len],
        }
    }
}

#[derive(Clone, Debug)]
pub struct ChartTheme {
    light: ChartColors,
    dark: ChartColors,
}

/// The colors of the active chart theme for the kit's light or dark mode (see [`ChartTheme::colors`]).
#[derive(Clone)]
pub struct Colors {
    theme: Arc<ChartTheme>,
    dark: bool,
}

impl Deref for Colors {
    type Target = ChartColors;

    fn deref(&self) -> &ChartColors {
        if self.dark { &self.theme.dark } else { &self.theme.light }
    }
}

impl ChartTheme {
    pub fn parse(json: &str) -> Result<Self> {
        let set: ThemeSet = serde_json::from_str(json)?;
        let pick = |mode: Mode| {
            set.themes
                .iter()
                .find(|theme| theme.mode == mode)
                .or_else(|| set.themes.first())
                .context("a chart theme set needs at least one theme")?
                .resolve()
        };
        Ok(Self { light: pick(Mode::Light)?, dark: pick(Mode::Dark)? })
    }

    /// Makes the themes of `json` the charts' (every chart repaints with them on its next render).
    pub fn load(json: &str) -> Result<()> {
        let theme = Arc::new(Self::parse(json)?);
        *LOADED.write().unwrap_or_else(std::sync::PoisonError::into_inner) = Some(theme);
        GENERATION.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    pub(crate) fn revision() -> (u64, bool) {
        (GENERATION.load(Ordering::Relaxed), kit::dark())
    }

    /// The chart colors for the kit theme's mode (`blenderui::set_theme("light")` picks the light ones).
    #[must_use]
    pub fn colors() -> Colors {
        let theme = LOADED
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
            .unwrap_or_else(|| BUNDLED.clone());
        Colors { theme, dark: kit::dark() }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Scheme {
    GreenYellowRed,
    RedYellowGreen,
    BlueYellowRed,
    YellowRed,
    BluePurple,
    YellowBlue,
    Blues,
    Reds,
    Greens,
    Purples,
    Oranges,
    Named(SharedString),
    Gradient(Cow<'static, [Hsla]>),
    Thresholds(Cow<'static, [(f64, Hsla)]>),
}

impl Hash for Scheme {
    fn hash<H: Hasher>(&self, state: &mut H) {
        discriminant(self).hash(state);
        match self {
            Self::Named(name) => name.hash(state),
            Self::Gradient(stops) => stops.iter().for_each(|stop| color::hash(stop, state)),
            Self::Thresholds(steps) => steps.iter().for_each(|(threshold, stop)| {
                threshold.to_bits().hash(state);
                color::hash(stop, state);
            }),
            _ => {}
        }
    }
}

impl Scheme {
    pub fn gradient(stops: impl Into<Cow<'static, [Hsla]>>) -> Self {
        Self::Gradient(stops.into())
    }

    pub fn thresholds(steps: impl Into<Cow<'static, [(f64, Hsla)]>>) -> Self {
        Self::Thresholds(steps.into())
    }

    fn name(&self) -> Option<&str> {
        Some(match self {
            Self::GreenYellowRed => "green-yellow-red",
            Self::RedYellowGreen => "red-yellow-green",
            Self::BlueYellowRed => "blue-yellow-red",
            Self::YellowRed => "yellow-red",
            Self::BluePurple => "blue-purple",
            Self::YellowBlue => "yellow-blue",
            Self::Blues => "blues",
            Self::Reds => "reds",
            Self::Greens => "greens",
            Self::Purples => "purples",
            Self::Oranges => "oranges",
            Self::Named(name) => name.as_ref(),
            Self::Gradient(_) | Self::Thresholds(_) => return None,
        })
    }

    pub(crate) fn stops<'a>(&'a self, colors: &'a ChartColors) -> &'a [Hsla] {
        match self {
            Self::Thresholds(_) => &[],
            Self::Gradient(stops) => stops,
            named => named.name().and_then(|name| colors.schemes.get(name)).map_or(&[][..], Vec::as_slice),
        }
    }

    pub(crate) fn sample(&self, colors: &ChartColors, t: f32) -> Option<Hsla> {
        match self.stops(colors) {
            [] => None,
            [only] => Some(*only),
            stops => Some(sample(stops, t.clamp(0., 1.))),
        }
    }

    pub(crate) fn palette(&self, colors: &ChartColors, lut: &mut Vec<Hsla>) {
        lut.clear();
        if let Self::Thresholds(steps) = self {
            return lut.extend(steps.iter().take(NO_TONE.into()).map(|(_, color)| *color));
        }
        let stops = self.stops(colors);
        match stops {
            [] => {}
            [only] => lut.push(*only),
            _ => lut.extend((0..GRADIENT_STEPS).map(|step| sample(stops, step as f32 / (GRADIENT_STEPS - 1) as f32))),
        }
    }

    pub(crate) fn tone(&self, value: f64, (min, max): (f64, f64), len: usize) -> u8 {
        if !value.is_finite() || len == 0 {
            return NO_TONE;
        }
        let index = if let Self::Thresholds(steps) = self {
            steps[..len].iter().rposition(|(threshold, _)| value >= *threshold).unwrap_or(0)
        } else {
            let span = max - min;
            let t = if span > 0. { ((value - min) / span).clamp(0., 1.) } else { 0. };
            (t * (len - 1) as f64).round() as usize
        };
        index as u8
    }
}

fn sample(stops: &[Hsla], t: f32) -> Hsla {
    let position = t * (stops.len() - 1) as f32;
    let index = (position.floor() as usize).min(stops.len() - 2);
    mix(stops[index], stops[index + 1], position - index as f32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use blenderui::Rgba;

    #[test]
    fn bundled_theme_defines_every_builtin_scheme() {
        let theme = ChartTheme::parse(CATPPUCCIN).unwrap();
        let builtin = [
            Scheme::GreenYellowRed,
            Scheme::RedYellowGreen,
            Scheme::BlueYellowRed,
            Scheme::YellowRed,
            Scheme::BluePurple,
            Scheme::YellowBlue,
            Scheme::Blues,
            Scheme::Reds,
            Scheme::Greens,
            Scheme::Purples,
            Scheme::Oranges,
        ];
        for colors in [&theme.light, &theme.dark] {
            assert_eq!(colors.palette.len(), 12);
            for scheme in &builtin {
                assert!(colors.schemes.contains_key(scheme.name().unwrap()), "{scheme:?}");
            }
        }
    }

    #[test]
    fn gradients_and_steps_pick_colors() {
        let (red, blue) = (Rgba { r: 1., g: 0., b: 0., a: 1. }, Rgba { r: 0., g: 0., b: 1., a: 1. });
        let mut lut = Vec::new();
        let gradient = Scheme::gradient(vec![red, blue]);
        gradient.palette(&BUNDLED.light, &mut lut);
        let at = |value| lut[usize::from(gradient.tone(value, (0., 10.), lut.len()))];
        assert_eq!((at(0.), at(10.)), (red, blue));
        assert_eq!(gradient.tone(f64::NAN, (0., 10.), lut.len()), NO_TONE);

        let steps = Scheme::thresholds(vec![(0., red), (5., blue)]);
        steps.palette(&BUNDLED.light, &mut lut);
        assert_eq!(lut, [red, blue]);
        assert_eq!((steps.tone(-3., (0., 0.), 2), steps.tone(7., (0., 0.), 2)), (0, 1));
    }

    #[test]
    fn invalid_colors_are_reported() {
        let json = r##"{ "themes": [{ "mode": "light", "palette": ["nope"], "fallback": "#000000", "hover": "#ffffff" }] }"##;
        assert!(ChartTheme::parse(json).is_err());
    }

    #[test]
    fn tailwind_names_parse() {
        let blue = parse_color("blue-500").expect("a Tailwind color");
        assert!(blue.b > blue.r && blue.a > 0.99);
        assert!((parse_color("red-600/50").expect("with opacity").a - 0.5).abs() < 1e-3);
    }
}
