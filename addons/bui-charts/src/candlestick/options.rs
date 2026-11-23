use blenderui::Hsla;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CandleMode {
    Candles,
    #[default]
    CandlesVolume,
    Volume,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CandleStyle {
    #[default]
    Candles,
    OhlcBars,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ColorStrategy {
    #[default]
    OpenClose,
    CloseClose,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Candles {
    pub mode: CandleMode,
    pub style: CandleStyle,
    pub strategy: ColorStrategy,
    pub up: Hsla,
    pub down: Hsla,
    pub flat: Hsla,
    pub volume_alpha: f32,
}

impl CandleMode {
    pub(crate) const fn price(self) -> bool {
        !matches!(self, Self::Volume)
    }

    pub(crate) const fn volume(self) -> bool {
        !matches!(self, Self::Candles)
    }
}
