use blenderui::{Pixels, SharedString};

use super::scale::ScaleKind;
use crate::format::{fixed, step_decimals, trimmed};
use crate::time;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AxisPlacement {
    #[default]
    Auto,
    Top,
    Right,
    Bottom,
    Left,
    Hidden,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AxisColorMode {
    #[default]
    Text,
    Series,
}

type Format = fn(f64) -> SharedString;

#[derive(Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct AxisConfig {
    pub(crate) placement: AxisPlacement,
    pub(crate) label: Option<SharedString>,
    pub(crate) width: Option<Pixels>,
    pub(crate) soft_min: Option<f64>,
    pub(crate) soft_max: Option<f64>,
    pub(crate) min: Option<f64>,
    pub(crate) max: Option<f64>,
    pub(crate) scale: ScaleKind,
    pub(crate) grid: bool,
    pub(crate) border: bool,
    pub(crate) centered_zero: bool,
    pub(crate) color_mode: AxisColorMode,
    pub(crate) decimals: Option<usize>,
    pub(crate) format: Option<Format>,
    pub(crate) time: bool,
    pub(crate) utc_offset: i32,
    pub(crate) reverse: bool,
    pub(crate) categories: Option<Vec<SharedString>>,
}

impl Default for AxisConfig {
    fn default() -> Self {
        Self {
            placement: AxisPlacement::Auto,
            label: None,
            width: None,
            soft_min: None,
            soft_max: None,
            min: None,
            max: None,
            scale: ScaleKind::Linear,
            grid: true,
            border: false,
            centered_zero: false,
            color_mode: AxisColorMode::Text,
            decimals: None,
            format: None,
            time: false,
            utc_offset: 0,
            reverse: false,
            categories: None,
        }
    }
}

impl AxisConfig {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    with! {
        placement: AxisPlacement,
        width: Pixels,
        soft_min: f64,
        soft_max: f64,
        min: f64,
        max: f64,
        scale: ScaleKind,
        grid: bool,
        border: bool,
        centered_zero: bool,
        color_mode: AxisColorMode,
        decimals: usize,
        format: Format,
        utc_offset: i32,
        reverse: bool,
    }

    #[must_use]
    pub const fn time(mut self) -> Self {
        self.time = true;
        self
    }

    #[must_use]
    pub fn categories<S: Into<SharedString>>(mut self, names: impl IntoIterator<Item = S>) -> Self {
        self.categories = Some(names.into_iter().map(Into::into).collect());
        self
    }

    pub(crate) fn category(&self, value: f64) -> Option<SharedString> {
        let names = self.categories.as_ref()?;
        let index = value.round();
        ((value - index).abs() < 1e-6 && index >= 0.)
            .then(|| names.get(index as usize).cloned())
            .flatten()
    }

    #[must_use]
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub(crate) fn visible(&self) -> bool {
        self.placement != AxisPlacement::Hidden
    }

    pub(crate) fn text(&self, value: f64, step: Option<f64>) -> SharedString {
        self.label_text(value, step, 0.)
    }

    pub(crate) fn label_text(&self, value: f64, step: Option<f64>, span: f64) -> SharedString {
        if self.categories.is_some() {
            return self.category(value).unwrap_or_default();
        }
        if let Some(format) = self.format {
            return format(value);
        }
        if self.time {
            return step.map_or_else(
                || time::full(value, self.utc_offset),
                |step| time::tick(value, step, span, self.utc_offset),
            );
        }
        self.decimals
            .or_else(|| step.map(step_decimals))
            .map_or_else(|| trimmed(value), |decimals| fixed(value, decimals))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_follows_the_tick_step() {
        let axis = AxisConfig::new();
        assert_eq!(axis.text(20., Some(10.)), "20");
        assert_eq!(axis.text(0.5, Some(0.5)), "0.5");
        assert_eq!(axis.text(7.5, Some(2.5)), "7.5");
        assert_eq!(axis.text(1.25, None), "1.25");
        assert_eq!(axis.decimals(2).text(1., None), "1.00");
    }
}
