use std::borrow::Cow;

use blenderui::Hsla;

use crate::ChartColors;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ThresholdsMode {
    #[default]
    Absolute,
    Percentage,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Thresholds {
    pub(crate) mode: ThresholdsMode,
    pub(crate) steps: Cow<'static, [(f64, Hsla)]>,
}

impl std::hash::Hash for Thresholds {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.mode.hash(state);
        for (value, color) in self.steps.iter() {
            value.to_bits().hash(state);
            crate::color::hash(color, state);
        }
    }
}

impl Thresholds {
    pub fn absolute(steps: impl Into<Cow<'static, [(f64, Hsla)]>>) -> Self {
        Self {
            mode: ThresholdsMode::Absolute,
            steps: steps.into(),
        }
    }

    pub fn percentage(steps: impl Into<Cow<'static, [(f64, Hsla)]>>) -> Self {
        Self {
            mode: ThresholdsMode::Percentage,
            steps: steps.into(),
        }
    }
}

const DEFAULT_THRESHOLD: f64 = 80.;

pub(crate) type Fallback = [(f64, Hsla); 2];

pub(crate) const fn fallback(colors: &ChartColors) -> Fallback {
    [
        (f64::NEG_INFINITY, colors.good),
        (DEFAULT_THRESHOLD, colors.bad),
    ]
}

pub(crate) fn steps<'a>(thresholds: Option<&'a Thresholds>, fallback: &'a Fallback) -> Steps<'a> {
    thresholds.map_or(
        Steps {
            mode: ThresholdsMode::Absolute,
            steps: fallback,
        },
        |thresholds| Steps {
            mode: thresholds.mode,
            steps: &thresholds.steps,
        },
    )
}

#[derive(Clone, Copy)]
pub(crate) struct Steps<'a> {
    pub mode: ThresholdsMode,
    pub steps: &'a [(f64, Hsla)],
}

impl Steps<'_> {
    pub(crate) fn fraction(self, step: f64, (min, max): (f64, f64)) -> f64 {
        match self.mode {
            ThresholdsMode::Absolute => percent(step, min, max),
            ThresholdsMode::Percentage => step / 100.,
        }
    }

    pub(crate) fn value(self, step: f64, (min, max): (f64, f64)) -> f64 {
        match self.mode {
            ThresholdsMode::Absolute => step,
            ThresholdsMode::Percentage => (max - min).mul_add(step / 100., min),
        }
    }

    pub(crate) fn color(self, value: f64, range: (f64, f64)) -> Option<Hsla> {
        let (base, rest) = self.steps.split_first()?;
        Some(
            rest.iter()
                .take_while(|(step, _)| value >= self.value(*step, range))
                .last()
                .map_or(base.1, |(_, color)| *color),
        )
    }
}

pub(crate) fn percent(value: f64, min: f64, max: f64) -> f64 {
    let ratio = (value - min) / (max - min);
    if ratio.is_nan() { 0. } else { ratio.min(1.) }
}

#[cfg(test)]
mod tests {
    use blenderui::{blue, green, red};

    use super::*;

    #[test]
    fn thresholds_pick_the_highest_step_reached() {
        let steps = [(f64::NEG_INFINITY, green()), (50., blue()), (80., red())];
        let absolute = Steps {
            mode: ThresholdsMode::Absolute,
            steps: &steps,
        };
        assert_eq!(absolute.color(10., (0., 100.)), Some(green()));
        assert_eq!(absolute.color(50., (0., 100.)), Some(blue()));
        assert_eq!(absolute.color(99., (0., 100.)), Some(red()));
        let percentage = Steps {
            mode: ThresholdsMode::Percentage,
            steps: &steps,
        };
        assert_eq!(percentage.color(450., (0., 1000.)), Some(green()));
        assert_eq!(percentage.color(900., (0., 1000.)), Some(red()));
    }

    #[test]
    fn percent_caps_at_one_and_treats_nan_as_zero() {
        assert!((percent(50., 0., 100.) - 0.5).abs() < f64::EPSILON);
        assert!((percent(500., 0., 100.) - 1.).abs() < f64::EPSILON);
        assert!(percent(f64::NAN, 0., 100.).abs() < f64::EPSILON);
    }
}
