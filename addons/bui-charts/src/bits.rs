use std::hash::{Hash, Hasher};

pub(crate) trait Bits {
    fn bits<H: Hasher>(&self, state: &mut H);
}

impl Bits for f32 {
    fn bits<H: Hasher>(&self, state: &mut H) {
        self.to_bits().hash(state);
    }
}

impl Bits for f64 {
    fn bits<H: Hasher>(&self, state: &mut H) {
        self.to_bits().hash(state);
    }
}

impl Bits for (f32, f32) {
    fn bits<H: Hasher>(&self, state: &mut H) {
        (self.0.to_bits(), self.1.to_bits()).hash(state);
    }
}

impl Bits for crate::plot::LineStyle {
    fn bits<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Solid => 0u8.hash(state),
            Self::Dash(pattern) => {
                1u8.hash(state);
                pattern.iter().for_each(|step| step.bits(state));
            }
            Self::Dot(gap) => {
                2u8.hash(state);
                gap.bits(state);
            }
        }
    }
}

macro_rules! hashed {
    ($($ty:ty),* $(,)?) => {$(
        impl Bits for $ty {
            fn bits<H: Hasher>(&self, state: &mut H) {
                self.hash(state);
            }
        }
    )*};
}

hashed!(
    bool,
    &'static str,
    crate::plot::HideFrom,
    crate::trend::NullSpan,
    crate::trend::DrawStyle,
    crate::trend::Interpolation,
    crate::trend::GradientMode,
    crate::trend::PointsMode,
    crate::trend::BarAlignment,
    crate::trend::Stacking,
    crate::trend::ValueTransform,
    crate::trend::ThresholdsStyle,
    crate::xy_chart::Show,
    crate::xy_chart::PointShape,
);
