use blenderui::SharedString;

use super::axis::AxisConfig;
use super::scale::{MAX_TICKS, Scale};
use crate::text;

pub(crate) const PADDING: f32 = 8.;
/// The axis text size, and the gap around it (gpui-kit's plot labels).
pub(crate) const TEXT_SIZE: f32 = 10.;
pub(crate) const TEXT_GAP: f32 = 2.;
pub(crate) const TEXT_HEIGHT: f32 = TEXT_SIZE + TEXT_GAP;
pub(crate) const LABEL_HEIGHT: f32 = TEXT_HEIGHT + TEXT_GAP * 2.;
pub(crate) const X_TICK_SPACING: f32 = 80.;
pub(crate) const Y_TICK_SPACING: f32 = 40.;
const CATEGORY_GAP: f32 = 24.;

pub(crate) struct Labels {
    items: [(f32, SharedString); MAX_TICKS],
    len: usize,
}

impl Labels {
    pub(crate) fn new(
        scale: &Scale,
        axis: &AxisConfig,
        length: f32,
        spacing: f32,
        offset: impl Fn(f32) -> f32,
    ) -> Self {
        if let Some(names) = &axis.categories {
            let (low, high) = scale.range();
            let (first, last) = (low.min(high).ceil().max(0.), low.max(high).floor());
            let count = names.len() as f64;
            let items = (first as usize..=last.min(count - 1.).max(-1.) as usize)
                .filter(|index| (*index as f64) <= last && (*index as f64) < count)
                .map(|index| (offset(scale.fraction(index as f64)), names[index].clone()));
            // names along a horizontal axis (laid out X_TICK_SPACING apart) also keep their widths apart
            return Self::spaced(items, spacing.min(CATEGORY_GAP), spacing >= X_TICK_SPACING);
        }
        let ticks = scale.ticks(length, spacing);
        let mut labels = Self { items: Default::default(), len: ticks.as_slice().len() };
        for (slot, value) in labels.items.iter_mut().zip(ticks.as_slice()) {
            *slot = (
                offset(scale.fraction(*value)),
                axis.label_text(*value, ticks.step, scale.range().1 - scale.range().0),
            );
        }
        labels
    }

    pub(crate) fn positions(&self) -> impl Iterator<Item = f32> + '_ {
        self.items[..self.len].iter().map(|(at, _)| *at)
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (f32, &SharedString)> + '_ {
        self.items[..self.len].iter().map(|(at, text)| (*at, text))
    }

    pub(crate) fn width(&self, axis: &AxisConfig) -> f32 {
        if !axis.visible() {
            return 0.;
        }
        axis.width.map_or_else(
            || {
                let widest = self.items[..self.len]
                    .iter()
                    .map(|(_, label)| text::measure(label, TEXT_SIZE))
                    .fold(0., f32::max);
                TEXT_GAP.mul_add(3., widest)
            },
            |width| width.0,
        )
    }

    pub(crate) fn explicit(items: impl Iterator<Item = (f32, SharedString)>, spacing: f32) -> Self {
        Self::spaced(items, spacing, false)
    }

    /// Labels at least `spacing` apart, and with `widths` far enough apart that their text doesn't
    /// meet.
    fn spaced(items: impl Iterator<Item = (f32, SharedString)>, spacing: f32, widths: bool) -> Self {
        let mut labels = Self { items: Default::default(), len: 0 };
        let mut last: Option<(f32, f32)> = None;
        for (at, text) in items {
            if labels.len == MAX_TICKS {
                break;
            }
            let half = if widths { text::measure(&text, TEXT_SIZE) / 2. } else { 0. };
            if last.is_none_or(|(previous, before)| (at - previous).abs() >= spacing.max(before + half + TEXT_GAP * 2.)) {
                labels.items[labels.len] = (at, text);
                labels.len += 1;
                last = Some((at, half));
            }
        }
        labels
    }
}

pub(crate) fn gutter(axis: &AxisConfig) -> f32 {
    if axis.visible() {
        TEXT_HEIGHT + TEXT_GAP + if axis.label.is_some() { LABEL_HEIGHT } else { 0. }
    } else {
        0.
    }
}
