use blenderui::hooks::Signal;
use blenderui::{Hsla, Painter, SharedString, rgb, white};

use super::layout::{Fit, Piece, Sizes, place};
use super::options::{GraphMode, JustifyMode, PercentChangeColor, StatColorMode};
use crate::bar_gauge::{BarOrientation, GaugeColor, TextMode};
use crate::color::{Hsl, mix};
use crate::geom::{Bounds, point, size};
use crate::mappings::{StateValue, ValueMappings};
use crate::single_stat::{Item, Sparkline, Values, cells, percent_text};
use crate::thresholds::{Thresholds, fallback, steps};
use crate::trend::shade::{Shade, brighten};
use crate::{ChartTheme, text};

const SPACING: f32 = 3.;
const RADIUS: f32 = 2.;
/// Bands the background gradient of a cell is painted in (gpui painted a 120° gradient).
const BACKGROUND_BANDS: usize = 16;
const ONE: SharedString = SharedString::Borrowed("1");
const EMPTY: SharedString = SharedString::Borrowed("");

pub(crate) struct Look {
    pub color_mode: StatColorMode,
    pub graph_mode: GraphMode,
    pub justify: JustifyMode,
    pub text_mode: TextMode,
    pub orientation: BarOrientation,
    pub wide: bool,
    pub percent: bool,
    pub percent_color: PercentChangeColor,
    pub sizes: Sizes,
    pub range: (Option<f64>, Option<f64>),
    pub thresholds: Option<Thresholds>,
    pub color: GaugeColor,
    pub mappings: Option<&'static ValueMappings>,
}

#[derive(Clone, Copy)]
pub(crate) struct Ui {
    pub text: Hsla,
    pub dark: bool,
}

pub(crate) fn contrast(color: Hsla, dark: bool) -> Hsla {
    let light = rgb(0xf7f8fa);
    let ink = rgb(0x202226);
    if color.a < 0.3 {
        return if dark { light } else { ink };
    }
    let brightness = (color.r * 299. + color.g * 587. + color.b * 114.) / 1000. * 255.;
    if brightness > 180. { ink } else { light }
}

fn longest<'a>(items: impl Iterator<Item = &'a SharedString>) -> &'a SharedString {
    items.max_by_key(|text| text.chars().count()).unwrap_or(&EMPTY)
}

fn mapped(mappings: Option<&ValueMappings>, item: &Item) -> Option<(SharedString, Hsla)> {
    let found = mappings?.find(&StateValue::Number(item.value))?;
    let text = if found.text.is_empty() { item.text.clone() } else { found.text };
    Some((text, found.color))
}

pub(crate) fn paint(bounds: Bounds, stat: &Look, values: Signal<Values>, ui: Ui, painter: &mut Painter) {
    values.with(|values| paint_values(bounds, stat, values, ui, painter));
}

/// A cell's background in `color`: a gradient (`Background`) or solid (`BackgroundSolid`).
fn background(cell: Bounds, mode: StatColorMode, color: Hsla, dark: bool, painter: &mut Painter) {
    match mode {
        StatColorMode::Background => {
            let factor = if dark { 1. } else { -0.7 };
            let base = Hsl::from(color);
            let shift = |hue: f32, light: f32| -> Hsla {
                Hsl { h: (base.h + hue / 360.).rem_euclid(1.), l: (base.l - light * factor / 100.).clamp(0., 1.), ..base }
                    .into()
            };
            let (from, to) = (shift(8., 15.), shift(-8., 5.));
            let band = cell.size.width / BACKGROUND_BANDS as f32;
            painter.with_clip(cell.into(), |painter| {
                for index in 0..BACKGROUND_BANDS {
                    // whole pixels: bands that meet on a pixel edge leave no seam
                    let left = band.mul_add(index as f32, cell.left()).round();
                    let right = band.mul_add((index + 1) as f32, cell.left()).round();
                    let tone = mix(from, to, (index as f32 + 0.5) / BACKGROUND_BANDS as f32);
                    let strip = Bounds::new(point(left, cell.top()), size(right - left, cell.size.height));
                    painter.fill(strip.into(), tone);
                }
            });
        }
        StatColorMode::BackgroundSolid => painter.quad(cell.into(), RADIUS, color, color, 0.),
        StatColorMode::Value | StatColorMode::None => {}
    }
}

#[allow(clippy::too_many_lines)]
fn paint_values(bounds: Bounds, stat: &Look, values: &Values, ui: Ui, painter: &mut Painter) {
    let colors = ChartTheme::colors();
    let (good, bad) = (colors.good, colors.bad);
    let fallback = fallback(&colors);
    let steps = steps(stat.thresholds.as_ref(), &fallback);
    let count = values.items.len();
    let range = (stat.range.0.unwrap_or(values.extent.0), stat.range.1.unwrap_or(values.extent.1));
    let mode = match stat.text_mode {
        TextMode::Auto if count == 1 => TextMode::Value,
        TextMode::Auto => TextMode::ValueAndName,
        mode => mode,
    };
    let resolved: Vec<(SharedString, Hsla)> = values
        .items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            mapped(stat.mappings, item)
                .unwrap_or_else(|| (item.text.clone(), stat.color.at(item.value, index, steps, range, &colors)))
        })
        .collect();
    let title_align = longest(values.items.iter().map(|item| &item.name));
    let value_align = match mode {
        TextMode::Name => title_align,
        TextMode::None => &ONE,
        _ => longest(resolved.iter().map(|(text, _)| text)),
    };
    let named = mode == TextMode::ValueAndName;
    let slots = cells(bounds, count, stat.orientation, SPACING, (0., 0.));
    let filled = matches!(stat.color_mode, StatColorMode::Background | StatColorMode::BackgroundSolid);
    for ((item, (text, color)), cell) in values.items.iter().zip(&resolved).zip(slots) {
        background(cell, stat.color_mode, *color, ui.dark, painter);
        let ink = if filled { contrast(*color, ui.dark) } else { ui.text };
        let value_color = match stat.color_mode {
            StatColorMode::Value => *color,
            StatColorMode::None => ui.text,
            _ => ink,
        };
        let change = item.change.filter(|_| stat.percent);
        let percent = change.map(|change| {
            let arrow = if change > 0. {
                "\u{2191} "
            } else if change < 0. {
                "\u{2193} "
            } else {
                ""
            };
            SharedString::from(format!("{arrow}{}", percent_text(change)))
        });
        let series = item.series.clone().filter(|_| stat.graph_mode != GraphMode::None);
        let shown = [
            named.then_some(&item.name).filter(|name| !name.is_empty()),
            match mode {
                TextMode::Name => Some(&item.name),
                TextMode::None => None,
                _ => Some(text),
            },
            percent.as_ref(),
        ];
        let fit = Fit {
            title: named.then_some(title_align),
            value: value_align,
            percent: percent.as_ref(),
            chart: series.is_some(),
            hidden: mode == TextMode::None,
            wide: stat.wide,
            justify: stat.justify,
            sizes: stat.sizes,
        };
        let placed = place(cell, shown, &fit);
        if let (Some(chart), Some(series)) = (placed.chart, series) {
            let (line, area) = if filled { (brighten(*color, 40.), white().opacity(0.4)) } else { (*color, color.opacity(0.2)) };
            Sparkline {
                xs: &values.xs,
                ys: &values.ys[series],
                line,
                fill: (stat.graph_mode == GraphMode::Area).then_some(Shade::Solid(area)),
                smooth: false,
            }
            .paint(chart, painter);
        }
        let percent_color = match stat.percent_color {
            _ if filled => ink,
            PercentChangeColor::SameAsValue => value_color,
            PercentChangeColor::Standard | PercentChangeColor::Inverted => {
                let sign = if stat.percent_color == PercentChangeColor::Inverted { -1. } else { 1. };
                if change.unwrap_or(0.) * sign > 0. { good } else { bad }
            }
        };
        let [title, value, percent] = shown;
        for (piece, text, color) in
            [(placed.title, title, ink), (placed.value, value, value_color), (placed.percent, percent, percent_color)]
        {
            if let (Some(Piece { origin, font, align }), Some(text)) = (piece, text) {
                text::draw(painter, text, origin, align, font, color);
            }
        }
    }
}
