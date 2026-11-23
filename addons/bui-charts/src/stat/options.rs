#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum StatColorMode {
    #[default]
    Value,
    Background,
    BackgroundSolid,
    None,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum GraphMode {
    #[default]
    Area,
    Line,
    None,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum JustifyMode {
    #[default]
    Auto,
    Center,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum PercentChangeColor {
    #[default]
    Standard,
    Inverted,
    SameAsValue,
}
