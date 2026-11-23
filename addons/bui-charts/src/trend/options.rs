use crate::plot::{HideFrom, LineStyle};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum DrawStyle {
    #[default]
    Line,
    Bars,
    Points,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Interpolation {
    #[default]
    Linear,
    Smooth,
    StepBefore,
    StepAfter,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum GradientMode {
    #[default]
    None,
    Opacity,
    Hue,
    Scheme,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum PointsMode {
    #[default]
    Auto,
    Always,
    Never,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum NullSpan {
    #[default]
    Never,
    Always,
    Within(f64),
}

impl std::hash::Hash for NullSpan {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Never => 0u64.hash(state),
            Self::Always => 1u64.hash(state),
            Self::Within(limit) => (2u64, limit.to_bits()).hash(state),
        }
    }
}

impl NullSpan {
    pub(crate) fn bridges(self, gap: f64) -> bool {
        match self {
            Self::Never => false,
            Self::Always => true,
            Self::Within(limit) => gap <= limit,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum BarAlignment {
    Before,
    #[default]
    Center,
    After,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Stacking {
    #[default]
    None,
    Normal(&'static str),
    Percent(&'static str),
}

impl Stacking {
    pub const NORMAL: Self = Self::Normal("A");
    pub const PERCENT: Self = Self::Percent("A");

    pub(crate) const fn group(self) -> Option<(&'static str, bool)> {
        match self {
            Self::None => None,
            Self::Normal(group) => Some((group, false)),
            Self::Percent(group) => Some((group, true)),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ValueTransform {
    #[default]
    None,
    Constant,
    NegativeY,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ThresholdsStyle {
    #[default]
    Off,
    Line,
    Dashed,
    Area,
    LineAndArea,
    DashedAndArea,
    Series,
}

impl ThresholdsStyle {
    pub(crate) const fn area(self) -> bool {
        matches!(self, Self::Area | Self::LineAndArea | Self::DashedAndArea)
    }

    pub(crate) const fn line(self) -> Option<bool> {
        match self {
            Self::Line | Self::LineAndArea => Some(false),
            Self::Dashed | Self::DashedAndArea => Some(true),
            _ => None,
        }
    }
}

style!(Style {
    draw_style: DrawStyle = DrawStyle::Line,
    interpolation: Interpolation = Interpolation::Linear,
    line_width: f32 = 1.,
    line_style: LineStyle = LineStyle::Solid,
    fill_opacity: f32 = 0.,
    gradient: GradientMode = GradientMode::None,
    fill_below_to: &'static str = "",
    show_points: PointsMode = PointsMode::Auto,
    point_size: f32 = 5.,
    show_values: bool = false,
    span_nulls: NullSpan = NullSpan::Never,
    insert_nulls: f64 = f64::INFINITY,
    bar_alignment: BarAlignment = BarAlignment::Center,
    bar_width_factor: f32 = 0.6,
    bar_max_width: f32 = 200.,
    stacking: Stacking = Stacking::None,
    transform: ValueTransform = ValueTransform::None,
    thresholds_style: ThresholdsStyle = ThresholdsStyle::Off,
    hide_from: HideFrom = HideFrom { legend: false, tooltip: false, viz: false },
});

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Part {
    Open,
    High,
    Low,
    Close,
    Volume,
}
