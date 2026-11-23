use std::hash::{Hash, Hasher};

use blenderui::{Hsla, Pixels};

use crate::calc::Calc;
use crate::thresholds::{Steps, percent};
use crate::{ChartColors, Scheme};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum BarDisplay {
    #[default]
    Gradient,
    Lcd,
    Basic,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ValueMode {
    #[default]
    Color,
    Text,
    Hidden,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum NamePlacement {
    #[default]
    Auto,
    Top,
    Left,
    Hidden,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TextMode {
    #[default]
    Auto,
    ValueAndName,
    Value,
    Name,
    None,
}

impl TextMode {
    pub(crate) const fn shows_name(self, count: usize, named: bool) -> bool {
        match self {
            Self::Auto => count != 1 || named,
            Self::Name | Self::ValueAndName => true,
            Self::Value | Self::None => false,
        }
    }

    pub(crate) const fn shows_value(self) -> bool {
        !matches!(self, Self::Name | Self::None)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum BarOrientation {
    #[default]
    Auto,
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum Sizing {
    #[default]
    Auto,
    Manual {
        min_width: Pixels,
        min_height: Pixels,
        max_height: Pixels,
    },
}

impl Sizing {
    const AUTO: (f32, f32, f32) = (8., 16., 300.);

    pub(crate) const fn limits(self) -> (f32, f32, f32) {
        match self {
            Self::Auto => Self::AUTO,
            Self::Manual {
                min_width,
                min_height,
                max_height,
            } => (min_width.0, min_height.0, max_height.0),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Reduce {
    Calc(Calc),
    All { limit: Option<usize> },
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum GaugeColor {
    #[default]
    Thresholds,
    Palette,
    Scheme(Scheme),
    Fixed(Hsla),
}

impl Hash for GaugeColor {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            Self::Scheme(scheme) => scheme.hash(state),
            Self::Fixed(color) => crate::color::hash(color, state),
            Self::Thresholds | Self::Palette => {}
        }
    }
}

impl GaugeColor {
    pub(crate) fn at(
        &self,
        value: f64,
        index: usize,
        steps: Steps<'_>,
        range: (f64, f64),
        colors: &ChartColors,
    ) -> Hsla {
        match self {
            Self::Thresholds => steps.color(value, range),
            Self::Palette => Some(colors.series(index)),
            Self::Scheme(scheme) => scheme.sample(
                colors,
                percent(value, range.0, range.1).clamp(0., 1.) as f32,
            ),
            Self::Fixed(color) => Some(*color),
        }
        .unwrap_or(colors.fallback)
    }
}
