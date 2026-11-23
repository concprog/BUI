use blenderui::{Hsla, Pixels};

use super::schemes::HeatmapScheme;
use crate::Scheme;
use crate::legend::LegendPlacement;

const DEFAULT_STEPS: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OpacityScale {
    Linear,
    Exponential(f32),
}

#[derive(Clone, Debug, PartialEq)]
pub enum HeatmapColor {
    Scheme {
        scheme: HeatmapScheme,
        steps: usize,
        reverse: bool,
    },
    Theme {
        scheme: Scheme,
        steps: usize,
        reverse: bool,
    },
    Opacity {
        fill: Hsla,
        scale: OpacityScale,
        steps: usize,
    },
}

impl Default for HeatmapColor {
    fn default() -> Self {
        Self::scheme(HeatmapScheme::Oranges)
    }
}

impl HeatmapColor {
    #[must_use]
    pub const fn scheme(scheme: HeatmapScheme) -> Self {
        Self::Scheme {
            scheme,
            steps: DEFAULT_STEPS,
            reverse: false,
        }
    }

    #[must_use]
    pub const fn theme(scheme: Scheme) -> Self {
        Self::Theme {
            scheme,
            steps: DEFAULT_STEPS,
            reverse: false,
        }
    }

    pub fn opacity(fill: impl Into<Hsla>) -> Self {
        Self::Opacity {
            fill: fill.into(),
            scale: OpacityScale::Linear,
            steps: DEFAULT_STEPS,
        }
    }

    #[must_use]
    pub fn steps(mut self, count: usize) -> Self {
        let count = count.clamp(2, 128);
        match &mut self {
            Self::Scheme { steps, .. }
            | Self::Theme { steps, .. }
            | Self::Opacity { steps, .. } => {
                *steps = count;
            }
        }
        self
    }

    #[must_use]
    pub const fn reverse(mut self, reversed: bool) -> Self {
        if let Self::Scheme { reverse, .. } | Self::Theme { reverse, .. } = &mut self {
            *reverse = reversed;
        }
        self
    }

    #[must_use]
    pub const fn exponent(mut self, exponent: f32) -> Self {
        if let Self::Opacity { scale, .. } = &mut self {
            *scale = OpacityScale::Exponential(exponent.clamp(0.1, 2.));
        }
        self
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Buckets {
    #[default]
    Auto,
    Count(usize),
    Size(f64),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum BucketScale {
    #[default]
    Linear,
    Log2,
    Log10,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Calculation {
    pub x_buckets: Buckets,
    pub y_buckets: Buckets,
    pub y_scale: BucketScale,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum BucketLayout {
    #[default]
    Auto,
    Le,
    Ge,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HeatmapTooltip {
    pub(crate) visible: bool,
    pub(crate) y_histogram: bool,
    pub(crate) color_scale: bool,
    pub(crate) max_width: Option<Pixels>,
    pub(crate) max_height: Option<Pixels>,
}

impl Default for HeatmapTooltip {
    fn default() -> Self {
        Self::single()
    }
}

impl HeatmapTooltip {
    #[must_use]
    pub const fn single() -> Self {
        Self {
            visible: true,
            y_histogram: false,
            color_scale: false,
            max_width: None,
            max_height: None,
        }
    }

    #[must_use]
    pub const fn hidden() -> Self {
        Self {
            visible: false,
            ..Self::single()
        }
    }

    with! {
        y_histogram: bool,
        color_scale: bool,
        max_width: Pixels,
        max_height: Pixels,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HeatmapLegend {
    pub(crate) visible: bool,
    pub(crate) placement: LegendPlacement,
    pub(crate) width: Option<Pixels>,
}

impl Default for HeatmapLegend {
    fn default() -> Self {
        Self::shown()
    }
}

impl HeatmapLegend {
    #[must_use]
    pub const fn shown() -> Self {
        Self {
            visible: true,
            placement: LegendPlacement::Bottom,
            width: None,
        }
    }

    #[must_use]
    pub const fn hidden() -> Self {
        Self {
            visible: false,
            ..Self::shown()
        }
    }

    #[must_use]
    pub const fn right(mut self) -> Self {
        self.placement = LegendPlacement::Right;
        self
    }

    with! {
        width: Pixels,
    }
}
