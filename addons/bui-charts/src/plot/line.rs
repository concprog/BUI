#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum LineStyle {
    #[default]
    Solid,
    Dash(&'static [f32]),
    Dot(f32),
}

impl LineStyle {
    pub const DASH: Self = Self::Dash(&[10., 10.]);
    pub const DOT: Self = Self::Dot(10.);

    #[must_use]
    pub const fn dash() -> Self {
        Self::DASH
    }

    #[must_use]
    pub const fn dot() -> Self {
        Self::DOT
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct HideFrom {
    pub legend: bool,
    pub tooltip: bool,
    pub viz: bool,
}
