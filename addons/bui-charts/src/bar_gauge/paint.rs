use std::collections::BTreeSet;

use blenderui::hooks::Signal;
use blenderui::{Hsla, Painter, SharedString};

use super::layout::{Labels, Region, Slots, is_vertical, parts, spacing};
use super::model::{Bar, Bars};
use super::options::{BarDisplay, BarOrientation, GaugeColor, Sizing, TextMode, ValueMode};
use crate::color::mix;
use crate::geom::{Bounds, Point, point, size};
use crate::thresholds::{Steps, Thresholds, fallback, percent, steps};
use crate::{ChartColors, ChartTheme, text};

const RADIUS: f32 = 2.;
const LCD_CELL: f32 = 12.;
const BASIC_FILL: f32 = 0.35;
const BASIC_EDGE: f32 = 2.;
const LIT: f32 = 0.95;
const SHADE: f32 = 0.55;
const UNLIT: f32 = 0.18;
/// The most bands a gradient between two colors is painted in.
const BLEND_BANDS: usize = 32;

pub(crate) struct Gauge {
    pub display: BarDisplay,
    pub value_mode: ValueMode,
    pub text_mode: TextMode,
    pub orientation: BarOrientation,
    pub show_unfilled: bool,
    pub sizing: Sizing,
    pub labels: Labels,
    pub range: (f64, f64),
    pub thresholds: Option<Thresholds>,
    pub color: GaugeColor,
    pub named: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct Ui {
    pub text: Hsla,
    pub track: Hsla,
}

#[derive(Clone, Copy)]
pub(crate) struct Store {
    pub bars: Signal<Bars>,
    pub hidden: Signal<BTreeSet<usize>>,
}

pub(crate) fn paint(bounds: Bounds, gauge: &Gauge, store: Store, ui: Ui, painter: &mut Painter) {
    let colors = ChartTheme::colors();
    let fallback = fallback(&colors);
    let steps = steps(gauge.thresholds.as_ref(), &fallback);
    store.hidden.with(|hidden| {
        store.bars.with(|bars| {
            let bars = &bars.items;
            let (count, title, value) = widest(gauge, bars, hidden);
            let vertical = is_vertical(gauge.orientation, bounds.size);
            let slots = Slots::new(bounds, count, vertical, spacing(gauge.display), gauge.sizing);
            let parts = parts(slots.size, vertical, &gauge.labels, title, value, slots.overflow);
            painter.with_clip(bounds.into(), |painter| {
                let shown = bars.iter().enumerate().filter(|(index, _)| !hidden.contains(index));
                for (slot, (index, bar)) in shown.enumerate() {
                    let origin = slots.at(slot).origin;
                    let color = gauge.color.at(bar.value, index, steps, gauge.range, &colors);
                    let track = Bounds::new(origin + parts.track.origin, parts.track.size);
                    draw_bar(track, vertical, bar.value, color, index, gauge, steps, &colors, ui, painter);
                    let value_color = if gauge.value_mode == ValueMode::Color { color } else { ui.text };
                    label(parts.title, &bar.name, origin, ui.text, painter);
                    label(parts.value, &bar.text, origin, value_color, painter);
                }
            });
        });
    });
}

/// How many bars show, and their longest name and value text (what the labels are sized for).
fn widest<'a>(
    gauge: &Gauge,
    bars: &'a [Bar],
    hidden: &BTreeSet<usize>,
) -> (usize, Option<&'a SharedString>, Option<&'a SharedString>) {
    let visible = || bars.iter().enumerate().filter(|(index, _)| !hidden.contains(index)).map(|(_, bar)| bar);
    let count = visible().count();
    let longest = |pick: fn(&Bar) -> &SharedString| visible().map(pick).max_by_key(|text| text.len());
    let title = gauge.text_mode.shows_name(count, gauge.named).then(|| longest(|bar| &bar.name)).flatten();
    let shows_value = gauge.text_mode.shows_value() && gauge.value_mode != ValueMode::Hidden;
    let value = shows_value.then(|| longest(|bar| &bar.text)).flatten();
    (count, title, value)
}

fn label(region: Option<Region>, content: &str, offset: Point, color: Hsla, painter: &mut Painter) {
    if let Some(region) = region {
        text::draw(painter, content, region.origin(offset), region.align, region.font, color);
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_bar(
    track: Bounds,
    vertical: bool,
    value: f64,
    color: Hsla,
    index: usize,
    gauge: &Gauge,
    steps: Steps<'_>,
    colors: &ChartColors,
    ui: Ui,
    painter: &mut Painter,
) {
    let axis = Axis { track, vertical };
    if gauge.display == BarDisplay::Lcd {
        return lcd(axis, value, index, gauge, steps, colors, painter);
    }
    if gauge.show_unfilled {
        painter.quad(track.into(), RADIUS, ui.track, ui.track, 0.);
    }
    let length = (percent(value, gauge.range.0, gauge.range.1) as f32 * axis.extent()).max(1.);
    let bar = axis.span(0., length);
    match gauge.display {
        BarDisplay::Basic => {
            painter.quad(bar.into(), RADIUS, color.alpha(BASIC_FILL), color, 0.);
            painter.fill(axis.span(length - BASIC_EDGE, length).into(), color);
        }
        BarDisplay::Gradient | BarDisplay::Lcd => {
            painter.with_clip(bar.into(), |painter| gradient(axis, value, color, gauge, steps, colors, painter));
        }
    }
}

#[derive(Clone, Copy)]
struct Axis {
    track: Bounds,
    vertical: bool,
}

impl Axis {
    const fn extent(self) -> f32 {
        if self.vertical { self.track.size.height } else { self.track.size.width }
    }

    fn span(self, from: f32, to: f32) -> Bounds {
        let (from, to) = (from.max(0.), to.max(from.max(0.)));
        let track = self.track;
        if self.vertical {
            Bounds::new(point(track.left(), track.bottom() - to), size(track.size.width, to - from))
        } else {
            Bounds::new(point(track.left() + from, track.top()), size(to - from, track.size.height))
        }
    }

    /// `start` at `from` running to `end` at `to` (gpui painted a linear gradient; the painter
    /// paints it as bands a pixel or two wide).
    fn blend(self, from: f32, to: f32, start: Hsla, end: Hsla, painter: &mut Painter) {
        if to <= from {
            return;
        }
        if start == end {
            return painter.fill(self.span(from, to).into(), start);
        }
        let count = (((to - from) / 2.).ceil() as usize).clamp(1, BLEND_BANDS);
        let step = (to - from) / count as f32;
        // band edges on whole pixels: bands sharing a pixel would each cover it partly (a seam)
        let snap = |at: f32| {
            if self.vertical {
                self.track.bottom() - (self.track.bottom() - at).round()
            } else {
                (self.track.left() + at).round() - self.track.left()
            }
        };
        for band in 0..count {
            let at = step.mul_add(band as f32, from);
            let color = mix(start, end, (band as f32 + 0.5) / count as f32);
            let (a, b) = (snap(at), if band + 1 == count { to } else { snap(at + step) });
            painter.fill(self.span(if band == 0 { from } else { a }, b).into(), color);
        }
    }
}

fn gradient(
    axis: Axis,
    value: f64,
    color: Hsla,
    gauge: &Gauge,
    steps: Steps<'_>,
    colors: &ChartColors,
    painter: &mut Painter,
) {
    let extent = axis.extent();
    match &gauge.color {
        GaugeColor::Thresholds => {
            let Some(((_, base), rest)) = steps.steps.split_first() else {
                return painter.fill(axis.span(0., extent).into(), color);
            };
            let (mut from, mut previous, mut last) = (0., *base, 0.);
            for (index, (step, next)) in rest.iter().enumerate() {
                let position = steps.fraction(*step, gauge.range) as f32 * extent;
                if value < steps.value(*step, gauge.range) {
                    break;
                }
                let offset = f32::midpoint(position, last).round();
                let hold = if index == 0 { f32::midpoint(from, offset) } else { from };
                axis.blend(from, hold, previous, previous, painter);
                axis.blend(hold, offset, previous, *next, painter);
                (from, previous, last) = (offset, *next, position);
            }
            axis.blend(from, extent, previous, previous, painter);
        }
        GaugeColor::Scheme(scheme) => match scheme.stops(colors) {
            [] | [_] => painter.fill(axis.span(0., extent).into(), color),
            stops => {
                let step = extent / (stops.len() - 1) as f32;
                for (index, pair) in stops.windows(2).enumerate() {
                    let from = step * index as f32;
                    axis.blend(from, from + step, pair[0], pair[1], painter);
                }
            }
        },
        GaugeColor::Palette | GaugeColor::Fixed(_) => painter.fill(axis.span(0., extent).into(), color),
    }
}

fn lcd(
    axis: Axis,
    value: f64,
    index: usize,
    gauge: &Gauge,
    steps: Steps<'_>,
    colors: &ChartColors,
    painter: &mut Painter,
) {
    let extent = axis.extent();
    let cells = (extent / LCD_CELL).floor();
    if cells < 1. {
        return;
    }
    let gap = spacing(BarDisplay::Lcd);
    let cell = (gap.mul_add(-cells, extent) / cells).floor().max(1.);
    let (min, max) = gauge.range;
    for position in 0..cells as usize {
        let at = (max - min).mul_add(position as f64 / f64::from(cells), min);
        let tone = gauge.color.at(at, index, steps, gauge.range, colors);
        let start = (cell + gap) * position as f32;
        let bounds = axis.span(start, start + cell);
        let (background, border) =
            if !value.is_nan() && at <= value { (tone.alpha(LIT), tone.alpha(SHADE)) } else { (tone.alpha(UNLIT), tone.alpha(0.)) };
        painter.quad(bounds.into(), RADIUS, background, border, 1.);
    }
}
