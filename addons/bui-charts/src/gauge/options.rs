#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum GaugeShape {
    #[default]
    Gauge,
    Circle,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum BarShape {
    #[default]
    Flat,
    Rounded,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum EndpointMarker {
    #[default]
    Point,
    Glow,
    None,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[allow(clippy::struct_excessive_bools)]
pub struct GaugeEffects {
    pub bar_glow: bool,
    pub center_glow: bool,
    pub gradient: bool,
}

impl GaugeEffects {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            bar_glow: false,
            center_glow: false,
            gradient: false,
        }
    }

    #[must_use]
    pub const fn bar_glow(mut self) -> Self {
        self.bar_glow = true;
        self
    }

    #[must_use]
    pub const fn center_glow(mut self) -> Self {
        self.center_glow = true;
        self
    }

    #[must_use]
    pub const fn gradient(mut self) -> Self {
        self.gradient = true;
        self
    }

    #[must_use]
    pub const fn all() -> Self {
        Self::new().bar_glow().center_glow().gradient()
    }
}
