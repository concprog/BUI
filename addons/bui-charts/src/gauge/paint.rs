use blenderui::hooks::Signal;
use blenderui::style::TextAlign;
use blenderui::{Hsla, Painter, SharedString, rgb};

use super::geometry::{
    Dimensions, Options, angles, dimensions, segment_count, to_rad, value_angles,
};
use super::options::{BarShape, EndpointMarker, GaugeEffects, GaugeShape};
use crate::bar_gauge::{BarOrientation, GaugeColor, Sizing, TextMode};
use crate::mappings::{StateValue, ValueMappings};
use crate::single_stat::{Sparkline, Values, cells};
use crate::thresholds::{Steps, Thresholds, fallback, steps};
use crate::trend::shade::{Shade, brighten, darken, lighten, saturate, spin};
use crate::color::{luminance, mix};
use crate::geom::{Bounds, Point, point, size};
use crate::path::Path;
use crate::{ChartColors, ChartTheme, text};

const SPACING: f32 = 16.;
const SLICE: f32 = 1.5;
const OVERLAP: f32 = 0.4;
const LINE_HEIGHT: f32 = 1.21;
const MARKER_MIN_ANGLE: f32 = 10.;
const GLOW_LAYERS: usize = 3;
const CENTER_LAYERS: usize = 20;

pub(crate) struct Look {
    pub shape: GaugeShape,
    pub bar_shape: BarShape,
    pub width_factor: f32,
    pub segments: usize,
    pub segment_spacing: f32,
    pub effects: GaugeEffects,
    pub endpoint: EndpointMarker,
    pub markers: bool,
    pub labels: Vec<(f64, SharedString)>,
    pub neutral: Option<f64>,
    pub sizing: Sizing,
    pub sparkline: bool,
    pub text_mode: TextMode,
    pub orientation: BarOrientation,
    pub value_size: Option<f32>,
    pub name_size: Option<f32>,
    pub range: (f64, f64),
    pub thresholds: Option<Thresholds>,
    pub color: GaugeColor,
    pub mappings: Option<&'static ValueMappings>,
    pub named: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct Ui {
    pub text: Hsla,
    pub muted: Hsla,
    pub track: Hsla,
    pub border: Hsla,
    pub dark: bool,
}

pub(crate) fn range(min: Option<f64>, max: Option<f64>) -> (f64, f64) {
    let (mut min, mut max) = (min.unwrap_or(0.), max.unwrap_or(100.));
    if (max - min).abs() < f64::EPSILON {
        min -= min * 0.1;
        max += max * 0.1;
    }
    (min, max)
}

pub(crate) fn bands(steps: Steps<'_>, (min, max): (f64, f64)) -> Vec<(f64, f64, Hsla)> {
    let span = max - min;
    let fraction = |value: f64| ((value - min) / span).clamp(0., 1.);
    let values: Vec<f64> = steps
        .steps
        .iter()
        .map(|(step, _)| {
            if step.is_finite() {
                steps.value(*step, (min, max))
            } else {
                min
            }
        })
        .collect();
    (0..values.len())
        .filter_map(|index| {
            let low = fraction(values[index]);
            let high = values.get(index + 1).map_or(1., |next| fraction(*next));
            (high > low).then_some((low, high, steps.steps[index].1))
        })
        .collect()
}

fn at(stops: &[(f32, Hsla)], fraction: f32) -> Hsla {
    let Some(first) = stops.first() else {
        return Hsla::default();
    };
    if fraction <= first.0 {
        return first.1;
    }
    stops
        .windows(2)
        .find(|pair| fraction <= pair[1].0)
        .map_or_else(
            || stops[stops.len() - 1].1,
            |pair| {
                let width = pair[1].0 - pair[0].0;
                let t = if width > 0. {
                    (fraction - pair[0].0) / width
                } else {
                    0.
                };
                mix(pair[0].1, pair[1].1, t)
            },
        )
}

fn gradient(
    look: &Look,
    steps: Steps<'_>,
    base: Hsla,
    fraction: f32,
    colors: &ChartColors,
    dark: bool,
) -> Vec<(f32, Hsla)> {
    match &look.color {
        GaugeColor::Thresholds => {
            let mut stops: Vec<(f32, Hsla)> = bands(steps, look.range)
                .into_iter()
                .map(|(low, _, color)| (low as f32, color))
                .collect();
            if let Some(last) = stops.last().copied() {
                stops.push((1., last.1));
            }
            stops
        }
        GaugeColor::Scheme(scheme) => (0..=16)
            .map(|index| {
                let t = index as f32 / 16.;
                (t, scheme.sample(colors, t).unwrap_or(colors.fallback))
            })
            .collect(),
        GaugeColor::Palette | GaugeColor::Fixed(_) => {
            let darker = darken(spin(base, -20.), if dark { 15. } else { 5. });
            let lighter = lighten(brighten(spin(saturate(base, 20.), 20.), 10.), 10.);
            let under = if dark {
                [(0., darker), (1., lighter)]
            } else {
                [(0., lighter), (1., darker)]
            };
            let (start, end) = if dark {
                (at(&under, 1. - fraction), under[1].1)
            } else {
                (under[0].1, at(&under, fraction))
            };
            vec![(0., start), (fraction, end), (1., end)]
        }
    }
}

fn guide(color: Hsla) -> Hsla {
    let dark = rgb(0x111217);
    let (a, b) = (luminance(dark), luminance(color));
    let ratio = (a.max(b) + 0.05) / (a.min(b) + 0.05);
    if ratio >= 4.5 {
        dark
    } else {
        rgb(0xfbfbfb)
    }
}

#[derive(Clone, Copy)]
struct Ring {
    center: Point,
    radius: f32,
    width: f32,
}

impl Ring {
    fn at(self, angle: f32, radius: f32) -> Point {
        let (sin, cos) = to_rad(angle).sin_cos();
        self.center + point(radius * cos, radius * sin)
    }

    fn paint(self, from: f32, length: f32, color: Hsla, painter: &mut Painter) {
        if length <= 0. || self.width <= 0. {
            return;
        }
        let steps = (length / SLICE).ceil().max(1.) as usize;
        let (outer, inner) = (self.radius + self.width / 2., self.radius - self.width / 2.);
        let angle = |index: usize| from + length * index as f32 / steps as f32;
        let mut path = Path::with_capacity(steps * 2 + 3);
        path.move_to(self.at(from, outer));
        for index in 1..=steps {
            path.line_to(self.at(angle(index), outer));
        }
        for index in (0..=steps).rev() {
            path.line_to(self.at(angle(index), inner));
        }
        path.close();
        path.fill(painter, color);
    }

    fn shaded(self, from: f32, length: f32, shade: impl Fn(f32) -> Hsla, painter: &mut Painter) {
        let steps = (length / SLICE).ceil().max(1.) as usize;
        let slice = length / steps as f32;
        for index in 0..steps {
            let start = from + slice * index as f32;
            let extra = if index + 1 < steps { OVERLAP } else { 0. };
            self.paint(start, slice + extra, shade(start + slice / 2.), painter);
        }
    }

    fn dot(self, angle: f32, radius: f32, color: Hsla, painter: &mut Painter) {
        let center = self.at(angle, self.radius);
        let side = radius * 2.;
        painter.oval(Bounds::centered_at(center, size(side, side)).into(), color, Hsla::TRANSPARENT, 0.);
    }
}

struct Bar<'a> {
    ring: Ring,
    rounded: bool,
    stops: Option<&'a [(f32, Hsla)]>,
    origin: f32,
    span: f32,
}

impl Bar<'_> {
    fn color(&self, angle: f32, solid: Hsla) -> Hsla {
        self.stops.map_or(solid, |stops| {
            at(
                stops,
                ((angle - self.origin).rem_euclid(360.) / self.span).clamp(0., 1.),
            )
        })
    }

    fn arc(&self, from: f32, length: f32, solid: Hsla, painter: &mut Painter) {
        if length <= 0. {
            return;
        }
        if self.stops.is_some() {
            self.ring
                .shaded(from, length, |angle| self.color(angle, solid), painter);
        } else {
            self.ring.paint(from, length, solid, painter);
        }
        if self.rounded {
            let radius = self.ring.width / 2.;
            self.ring.dot(from, radius, self.color(from, solid), painter);
            self.ring.dot(
                from + length,
                radius,
                self.color(from + length, solid),
                painter,
            );
        }
    }
}

fn longest<'a>(texts: impl Iterator<Item = &'a SharedString>) -> &'a str {
    texts.max_by_key(|text| text.chars().count()).map_or("", |text| text)
}

#[allow(clippy::too_many_lines)]
pub(crate) fn paint(
    bounds: Bounds,
    look: &Look,
    values: Signal<Values>,
    ui: Ui,
    painter: &mut Painter,
) {
    values.with(|values| paint_values(bounds, look, values, ui, painter));
}

#[allow(clippy::too_many_lines)]
fn paint_values(bounds: Bounds, look: &Look, values: &Values, ui: Ui, painter: &mut Painter) {
    let colors = ChartTheme::colors();
    let colors: &ChartColors = &colors;
    let fallback = fallback(colors);
    let steps = steps(look.thresholds.as_ref(), &fallback);
    let count = values.items.len();
    let minimum = match look.sizing {
        Sizing::Auto => (0., 0.),
        Sizing::Manual {
            min_width,
            min_height,
            ..
        } => (min_width.0, min_height.0),
    };
    let slots = cells(bounds, count, look.orientation, SPACING, minimum);
    let (show_value, show_name) = match look.text_mode {
        TextMode::Auto => (true, count > 1 || look.named),
        TextMode::ValueAndName => (true, true),
        TextMode::Value => (true, false),
        TextMode::Name => (false, true),
        TextMode::None => (false, false),
    };
    let resolved: Vec<(SharedString, Hsla)> = values
        .items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            look.mappings
                .and_then(|mappings| mappings.find(&StateValue::Number(item.value)))
                .map_or_else(
                    || {
                        (
                            item.text.clone(),
                            look.color.at(item.value, index, steps, look.range, colors),
                        )
                    },
                    |found| {
                        let text = if found.text.is_empty() {
                            item.text.clone()
                        } else {
                            found.text
                        };
                        (text, found.color)
                    },
                )
        })
        .collect();
    let aligned = count > 1;
    let value_align = longest(resolved.iter().map(|(text, _)| text));
    let name_align = longest(values.items.iter().map(|item| &item.name));
    let (start, span) = angles(look.shape);
    let rounded = look.bar_shape == BarShape::Rounded;
    let (min, max) = look.range;
    let fraction = |value: f64| ((value - min) / (max - min)) as f32;
    let threshold_bands = look
        .markers
        .then(|| bands(steps, look.range))
        .filter(|_| max > min);
    let mut labels: Vec<(&str, Point, f32, Hsla)> = Vec::new();
    for ((item, (text, color)), cell) in values.items.iter().zip(&resolved).zip(slots) {
        let (width, height) = (cell.size.width, cell.size.height);
        let dims: Dimensions = dimensions(
            width,
            height,
            look.shape,
            &Options {
                glow: look.effects.bar_glow,
                rounded,
                width_factor: look.width_factor,
                markers: threshold_bands.is_some(),
                labels: !look.labels.is_empty(),
            },
        );
        if dims.radius <= 0. {
            continue;
        }
        let center = cell.origin + point(dims.center_x, dims.center_y);
        let ring = Ring {
            center,
            radius: dims.radius,
            width: dims.bar_width,
        };
        let value_fraction = if item.value.is_finite() {
            fraction(item.value)
        } else {
            0.
        };
        let (offset, length) = value_angles(value_fraction, look.neutral.map(fraction), span);
        let stops = look.effects.gradient.then(|| {
            gradient(
                look,
                steps,
                *color,
                value_fraction.clamp(0., 1.),
                colors,
                ui.dark,
            )
        });
        let cap = if look.shape == GaugeShape::Gauge && rounded {
            dims.bar_width / (std::f32::consts::PI * (dims.radius + dims.bar_width) * 2.) * 360.
        } else {
            0.
        };
        let bar = Bar {
            ring,
            rounded,
            stops: stops.as_deref(),
            origin: start - cap,
            span: span + cap * 2.,
        };
        if look.effects.bar_glow {
            let glow = 0.75 + dims.bar_width * 0.08;
            for layer in 1..=GLOW_LAYERS {
                let halo = Ring {
                    width: dims.bar_width + glow * 2. * layer as f32,
                    ..ring
                };
                halo.paint(
                    start + offset,
                    length,
                    bar.color(start + offset + length, *color).opacity(0.12),
                    painter,
                );
            }
        }
        if look.segments > 1 {
            let (segments, between) =
                segment_count(&dims, look.segment_spacing, look.segments, span);
            let arc = span / segments as f32 - between;
            for index in 0..segments {
                let value = (max - min).mul_add(index as f64 / segments as f64, min);
                let angle = fraction(value) * span;
                let track = angle < offset || angle >= offset + length;
                let from = start + span / segments as f32 * index as f32 + 0.01;
                let segment_color = if track {
                    ui.border
                } else if stops.is_some() {
                    bar.color(from, *color)
                } else {
                    look.color.at(value, 0, steps, look.range, colors)
                };
                if stops.is_some() && !track {
                    ring.shaded(from, arc, |angle| bar.color(angle, *color), painter);
                } else {
                    ring.paint(from, arc, segment_color, painter);
                }
            }
        } else {
            let track = Bar { stops: None, ..bar };
            if offset > 0. {
                track.arc(start, offset, ui.track, painter);
            }
            track.arc(
                start + offset + length,
                span - offset - length,
                ui.track,
                painter,
            );
            bar.arc(start + offset, length, *color, painter);
            let end = start + offset + length;
            match look.endpoint {
                EndpointMarker::Point if rounded => {
                    let dot = (dims.bar_width / 2. * 0.4).min(8.);
                    if length > MARKER_MIN_ANGLE {
                        ring.dot(
                            start + offset,
                            dot,
                            guide(bar.color(start + offset, *color)).opacity(0.5),
                            painter,
                        );
                    }
                    if length > 0. {
                        ring.dot(end, dot, guide(bar.color(end, *color)).opacity(0.5), painter);
                    }
                }
                EndpointMarker::Glow if ui.dark && length > MARKER_MIN_ANGLE => {
                    let tip = bar.color(end, *color);
                    let tint = mix(tip, blenderui::white(), 0.8).opacity(0.85);
                    let shift = if rounded {
                        (dims.bar_width / (2. * dims.radius)).to_degrees()
                    } else {
                        0.
                    };
                    let focus = ring.at(end + shift, dims.radius);
                    let reach = dims.bar_width * 1.5;
                    let from = (end - 80.).max(start + offset);
                    ring.shaded(
                        from,
                        end - from,
                        |angle| {
                            let spot = ring.at(angle, dims.radius);
                            let distance = (spot.x - focus.x).hypot(spot.y - focus.y);
                            tint.opacity(0.85 * (1. - distance / reach).max(0.))
                        },
                        painter,
                    );
                }
                _ => {}
            }
        }
        if look.effects.center_glow {
            let clip = match look.shape {
                GaugeShape::Gauge => Bounds::new(
                    center - point(dims.radius, dims.radius),
                    size(
                        dims.radius * 2.,
                        dims.radius * (1. + to_rad(super::geometry::ARC_END).sin()),
                    ),
                ),
                GaugeShape::Circle => cell,
            };
            let alpha = 1. - 0.75f32.powf(1. / CENTER_LAYERS as f32);
            painter.with_clip(clip.into(), |painter| {
                for layer in 0..CENTER_LAYERS {
                    let radius = dims.radius * 0.9 * (1. - layer as f32 / CENTER_LAYERS as f32);
                    let side = radius * 2.;
                    let disc = Bounds::centered_at(center, size(side, side));
                    painter.oval(disc.into(), color.opacity(alpha), Hsla::TRANSPARENT, 0.);
                }
            });
        }
        if let Some(bands) = &threshold_bands {
            let marker = Ring {
                center,
                radius: dims.marker_radius,
                width: dims.marker_width,
            };
            for (low, high, band) in bands {
                let from = start + *low as f32 * span;
                let length = (*high - *low) as f32 * span;
                match &stops {
                    Some(_) => marker.shaded(from, length, |angle| bar.color(angle, *band), painter),
                    None => marker.paint(from, length, *band, painter),
                }
            }
        }
        if !look.labels.is_empty() {
            let radius = dims.label_size.mul_add(-1.2, dims.label_radius);
            let circle = span >= 360.;
            let path_length = span / 360. * std::f32::consts::TAU * radius;
            let shown: Vec<&(f64, SharedString)> = look
                .labels
                .iter()
                .filter(|(value, _)| *value >= min && *value <= max)
                .collect();
            let lowest = shown
                .iter()
                .map(|(value, _)| *value)
                .fold(f64::INFINITY, f64::min);
            let highest = shown
                .iter()
                .map(|(value, _)| *value)
                .fold(f64::NEG_INFINITY, f64::max);
            for (value, label) in shown {
                let measured = text::measure(label, dims.label_size);
                let mut offset = fraction(*value) * path_length;
                if circle {
                    if value.total_cmp(&lowest).is_eq() {
                        offset += measured * 0.65;
                    } else if value.total_cmp(&highest).is_eq() {
                        offset -= measured * 0.65;
                    }
                } else {
                    if value.total_cmp(&lowest).is_eq() && offset - measured / 2. < 0. {
                        offset += measured / 2.;
                    }
                    if value.total_cmp(&highest).is_eq() && offset + measured / 2. > path_length {
                        offset -= measured / 2.;
                    }
                }
                let angle = start + offset / path_length.max(1.) * span;
                let (sin, cos) = to_rad(angle).sin_cos();
                let reach = (measured / 2.).mul_add(cos.abs(), dims.label_size / 2. * sin.abs());
                let anchor = ring.at(angle, radius + reach);
                labels.push((
                    label,
                    anchor - point(0., dims.label_size / 2.),
                    dims.label_size,
                    ui.text,
                ));
            }
        }
        let radius = dims.radius;
        let max_text = radius.mul_add(2., -dims.bar_width) - radius / 7.;
        let scaled = radius.powf(0.86);
        let (value_room, name_room) = if show_value && show_name {
            (0.82 * scaled, 0.45 * scaled)
        } else {
            (0.82 * scaled, radius / 4.)
        };
        let value_text: &str = if aligned { value_align } else { text };
        let name_text: &str = if aligned { name_align } else { &item.name };
        let value_font = look
            .value_size
            .unwrap_or_else(|| text::fit(value_text, max_text, value_room, LINE_HEIGHT))
            .max(1.);
        let name_font = look
            .name_size
            .unwrap_or_else(|| text::fit(name_text, max_text, name_room, LINE_HEIGHT))
            .max(10.);
        let (value_height, name_height) = (value_font * LINE_HEIGHT, name_font * LINE_HEIGHT);
        let value_y = if show_name {
            dims.center_y - name_height * 0.3
        } else {
            dims.center_y
        };
        let name_y = if show_value {
            value_height.mul_add(0.7, value_y)
        } else {
            dims.center_y
        };
        let series = item.series.clone().filter(|_| look.sparkline);
        let mut lift = 0.;
        if look.shape == GaugeShape::Gauge {
            lift += (value_height + name_height) / 4.;
        }
        if series.is_some() {
            lift += 8.;
        }
        let x = cell.left() + dims.center_x;
        if show_value {
            labels.push((
                text,
                point(x, cell.top() + (value_y - lift - value_font / 2.)),
                value_font,
                ui.text,
            ));
        }
        if show_name && !item.name.is_empty() {
            labels.push((
                &item.name,
                point(x, cell.top() + (name_y - lift - name_font / 2.)),
                name_font,
                if show_value { ui.muted } else { ui.text },
            ));
        }
        if let Some(series) = series {
            let both = show_value && show_name;
            let spark_height = radius / 4.;
            let factor = if look.shape == GaugeShape::Gauge {
                1.4
            } else {
                1.6
            };
            let spark_width = radius.mul_add(factor, -dims.bar_width);
            let top = match look.shape {
                GaugeShape::Gauge => dims.bottom_y - spark_height - 8.,
                GaugeShape::Circle => height / 2. + radius / if both { 3.3 } else { 4. },
            };
            Sparkline {
                xs: &values.xs,
                ys: &values.ys[series],
                line: *color,
                fill: Some(Shade::Vertical(color.opacity(0.4), color.opacity(0.))),
                smooth: true,
            }
            .paint(
                Bounds::new(
                    cell.origin + point(dims.center_x - spark_width / 2., top),
                    size(spark_width, spark_height),
                ),
                painter,
            );
        }
    }
    for (label, origin, font, color) in labels {
        text::draw(painter, label, origin, TextAlign::Center, font, color);
    }
}
