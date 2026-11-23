use blenderui::style::TextAlign;
use blenderui::{AnyElement, SharedString};

use super::scene::{Area, At, Frame, Note, Scene, Shape, Show, Zoom, ellipse, scene_view, stage};
use crate::geom::point;
use crate::{Dataset, component};
use crate::derive::data_key;
use crate::format::{fixed, trimmed};
use crate::plot::{TEXT_SIZE, Tooltip};

const KAPPA: f64 = 0.2857;
const KELVIN: f64 = 273.15;
const DRY: f64 = 287.04;
const HEAT: f64 = 1005.7;
const LATENT: f64 = 2.501e6;
const EPSILON: f64 = 0.622;

pub(crate) fn theta(celsius: f64, pressure: f64) -> f64 {
    (celsius + KELVIN) * (1000. / pressure).powf(KAPPA)
}

fn vapour(celsius: f64) -> f64 {
    6.112 * (17.67 * celsius / (celsius + 243.5)).exp()
}

pub(crate) fn saturation(celsius: f64, pressure: f64) -> f64 {
    let pressure_vapour = vapour(celsius);
    EPSILON * pressure_vapour / (pressure - pressure_vapour).max(1e-3)
}

fn dew_for(mixing: f64, pressure: f64) -> f64 {
    let partial = mixing * pressure / (EPSILON + mixing);
    let log = (partial / 6.112).ln();
    243.5 * log / (17.67 - log)
}

fn moist_slope(celsius: f64, pressure: f64) -> f64 {
    let kelvin = celsius + KELVIN;
    let mixing = saturation(celsius, pressure);
    (DRY * kelvin + LATENT * mixing)
        / (pressure * (HEAT + LATENT * LATENT * mixing * EPSILON / (DRY * kelvin * kelvin)))
}

pub(crate) fn moist_adiabat(start: f64, from: f64, to: f64, step: f64) -> Vec<(f64, f64)> {
    let mut out = vec![(start, from)];
    let (mut celsius, mut pressure) = (start, from);
    let delta = if to < from { -step.abs() } else { step.abs() };
    while (to - pressure) * delta.signum() > 0. {
        let half = moist_slope(celsius, pressure) * delta / 2.;
        let slope = moist_slope(celsius + half, pressure + delta / 2.);
        celsius += slope * delta;
        pressure += delta;
        out.push((celsius, pressure));
    }
    out
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
enum Kind {
    #[default]
    Stuve,
    Tephigram,
}

impl Kind {
    fn raw(self, celsius: f64, pressure: f64) -> (f64, f64) {
        match self {
            Self::Stuve => (celsius, -pressure.powf(KAPPA)),
            Self::Tephigram => {
                let a = celsius + KELVIN;
                let b = KELVIN * theta(celsius, pressure).ln();
                (
                    (a + b) / std::f64::consts::SQRT_2,
                    (b - a) / std::f64::consts::SQRT_2,
                )
            }
        }
    }
}

#[derive(Default)]
struct Sounding {
    kind: Kind,
    levels: Vec<(f64, f64, f64)>,
    pressures: (f64, f64),
    temperatures: (f64, f64),
    lines: (bool, bool, bool),
}

struct Map {
    kind: Kind,
    area: Area,
    origin: (f64, f64),
    scale: (f64, f64),
}

impl Map {
    fn new(sounding: &Sounding, area: Area) -> Self {
        let kind = sounding.kind;
        let (bottom, top) = sounding.pressures;
        let (cold, warm) = sounding.temperatures;
        let low = kind.raw(cold, bottom);
        let high = kind.raw(warm, bottom);
        let floor = kind
            .raw(f64::midpoint(cold, warm), bottom)
            .1
            .min(low.1)
            .min(high.1);
        let ceiling = kind.raw(f64::midpoint(cold, warm) - 50., top).1;
        let (width, height) = (high.0 - low.0, ceiling - floor);
        let mut scale = (
            f64::from(area.width()) / width,
            f64::from(area.height()) / height,
        );
        if kind == Kind::Tephigram {
            let even = scale.0.min(scale.1);
            scale = (even, even);
        }
        Self {
            kind,
            area,
            origin: (low.0, floor),
            scale,
        }
    }

    fn at(&self, celsius: f64, pressure: f64) -> At {
        let (x, up) = self.kind.raw(celsius, pressure);
        point(
            ((x - self.origin.0) * self.scale.0) as f32 + self.area.left,
            self.area.bottom - ((up - self.origin.1) * self.scale.1) as f32,
        )
    }
}

fn span(from: f64, to: f64, steps: usize) -> impl Iterator<Item = f64> {
    (0..=steps).map(move |step| (to - from).mul_add(step as f64 / steps as f64, from))
}

const ISOBARS: [f64; 10] = [1000., 900., 800., 700., 600., 500., 400., 300., 200., 100.];
const MIXING: [f64; 9] = [0.4, 1., 2., 3., 5., 8., 12., 16., 20.];

fn arrange_sounding(sounding: &Sounding, frame: &Frame) -> Scene {
    let mut scene = Scene::default();
    let area = Area::inset(frame.width, frame.height, (44., 12., 12., 22.));
    let map = Map::new(sounding, area);
    let (bottom, top) = sounding.pressures;
    let (cold, warm) = sounding.temperatures;
    let rule = frame.ink.rule;
    let ink = &frame.ink;
    scene.shapes.push(Shape::line(
        vec![
            point(area.left, area.top),
            point(area.right, area.top),
            point(area.right, area.bottom),
            point(area.left, area.bottom),
            point(area.left, area.top),
        ],
        ink.muted,
        1.,
        None,
    ));
    for pressure in ISOBARS
        .iter()
        .copied()
        .filter(|pressure| *pressure <= bottom && *pressure >= top)
    {
        let points: Vec<At> = span(cold - 120., warm + 60., 90)
            .map(|celsius| map.at(celsius, pressure))
            .collect();
        if let Some(start) = super::scene::clip(points.iter().copied(), area)
            .first()
            .and_then(|run| run.first().copied())
        {
            scene.label(
                trimmed(pressure),
                point(start.x - 4., start.y - TEXT_SIZE / 2.),
                TextAlign::Right,
                TEXT_SIZE,
                ink.muted,
            );
        }
        scene.lines(points, area, rule, 1., None);
    }
    let mut celsius = (cold / 10.).floor() * 10. - 100.;
    while celsius <= warm + 0.5 {
        let points: Vec<At> = span(bottom, top, 60)
            .map(|pressure| map.at(celsius, pressure))
            .collect();
        if let Some(foot) = points.first().copied().filter(|foot| area.holds(*foot)) {
            scene.label(
                trimmed(celsius),
                point(foot.x, area.bottom + 4.),
                TextAlign::Center,
                TEXT_SIZE,
                ink.muted,
            );
        }
        scene.lines(points, area, rule, 1., None);
        celsius += 10.;
    }
    let (adiabats, moist, mixing) = sounding.lines;
    if adiabats {
        let color = frame.ink.colors.series(1).opacity(0.35);
        for kelvin in (22..=48).map(|step| f64::from(step) * 10.) {
            let points = span(bottom, top, 60)
                .map(|pressure| map.at(kelvin * (pressure / 1000.).powf(KAPPA) - KELVIN, pressure));
            scene.lines(points, area, color, 1., None);
        }
    }
    if moist {
        let color = frame.ink.colors.series(2).opacity(0.45);
        for start in (-5..=10).map(|step| f64::from(step) * 4.) {
            let mut path = moist_adiabat(start, 1000., top, 10.);
            if bottom > 1000. {
                let mut below = moist_adiabat(start, 1000., bottom, 10.);
                below.reverse();
                below.pop();
                below.append(&mut path);
                path = below;
            }
            scene.lines(
                path.iter()
                    .map(|(celsius, pressure)| map.at(*celsius, *pressure)),
                area,
                color,
                1.,
                None,
            );
        }
    }
    if mixing {
        let color = frame.ink.colors.series(4).opacity(0.55);
        for grams in MIXING {
            let points: Vec<At> = span(bottom, 600_f64.max(top), 20)
                .map(|pressure| map.at(dew_for(grams / 1000., pressure), pressure))
                .collect();
            if let Some(end) = points.last().copied().filter(|end| area.holds(*end)) {
                scene.label(
                    trimmed(grams),
                    point(end.x, end.y - TEXT_SIZE - 1.),
                    TextAlign::Center,
                    TEXT_SIZE - 2.,
                    color.opacity(1.),
                );
            }
            scene.lines(points, area, color, 1., None);
        }
    }
    let (heat, dew) = (frame.ink.colors.bad, frame.ink.colors.good);
    let profile = |pick: fn(&(f64, f64, f64)) -> f64| -> Vec<At> {
        sounding
            .levels
            .iter()
            .filter(|level| pick(level).is_finite())
            .map(|level| map.at(pick(level), level.0))
            .collect()
    };
    scene.lines(profile(|level| level.2), area, dew, 2.5, None);
    scene.lines(profile(|level| level.1), area, heat, 2.5, None);
    for (pressure, temperature, dewpoint) in &sounding.levels {
        if !temperature.is_finite() {
            continue;
        }
        let mut rows = vec![(
            Some(heat),
            SharedString::from("Temperature"),
            format!("{} °C", fixed(*temperature, 1)).into(),
        )];
        if dewpoint.is_finite() {
            rows.push((
                Some(dew),
                "Dew point".into(),
                format!("{} °C", fixed(*dewpoint, 1)).into(),
            ));
            rows.push((
                None,
                "Mixing ratio".into(),
                format!(
                    "{} g/kg",
                    fixed(saturation(*dewpoint, *pressure) * 1000., 1)
                )
                .into(),
            ));
        }
        rows.push((
            None,
            "θ".into(),
            format!("{} K", fixed(theta(*temperature, *pressure), 1)).into(),
        ));
        let tip = scene.note(Note {
            title: format!("{} hPa", trimmed(*pressure)).into(),
            accent: None,
            rows,
        });
        let at = map.at(*temperature, *pressure);
        if area.holds(at) {
            scene
                .shapes
                .push(Shape::filled(ellipse(at, (3.5, 3.5), 12), heat, Some(tip)));
        }
        let wet = map.at(*dewpoint, *pressure);
        if dewpoint.is_finite() && area.holds(wet) {
            scene
                .shapes
                .push(Shape::filled(ellipse(wet, (3.5, 3.5), 12), dew, Some(tip)));
        }
    }
    scene
}

struct Options {
    kind: Kind,
    pressures: (f64, f64),
    temperatures: (f64, f64),
    lines: (bool, bool, bool),
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
}

fn thermo<T: 'static, D: Dataset<T>>(
    data: &D,
    read: impl Fn(&T) -> (f64, f64, f64),
    options: Options,
) -> AnyElement {
    let state = stage::<Sounding>();
    let settings = (
        options.kind,
        options.pressures.0.to_bits(),
        options.pressures.1.to_bits(),
        options.temperatures.0.to_bits(),
        options.temperatures.1.to_bits(),
        options.lines,
    );
    let key = data_key(data, settings);
    state.hold(
        key,
        || {
            let mut levels: Vec<(f64, f64, f64)> = data.rows(|rows| {
                rows.iter().map(&read).filter(|level| level.0.is_finite() && level.0 > 0.).collect()
            });
            levels.sort_by(|a, b| b.0.total_cmp(&a.0));
            Sounding {
                kind: options.kind,
                levels,
                pressures: options.pressures,
                temperatures: options.temperatures,
                lines: options.lines,
            }
        });
    scene_view(
        state,
        arrange_sounding,
        Show {
            groups: Vec::new(),
            legend: None,
            tooltip: options.tooltip,
            clickable: false,
            zoom: Zoom::Wheel,
            class: options.class,
        })
}

#[allow(clippy::too_many_arguments)]
fn options(
    kind: Kind,
    pressure_range: Option<(f64, f64)>,
    temperature_range: Option<(f64, f64)>,
    adiabats: Option<bool>,
    moist: Option<bool>,
    mixing: Option<bool>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
) -> Options {
    Options {
        kind,
        pressures: pressure_range.unwrap_or((1050., 100.)),
        temperatures: temperature_range.unwrap_or(if kind == Kind::Stuve {
            (-80., 40.)
        } else {
            (-40., 40.)
        }),
        lines: (
            adiabats.unwrap_or(true),
            moist.unwrap_or(true),
            mixing.unwrap_or(true),
        ),
        tooltip,
        class,
    }
}

#[component]
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub fn stuve<
    T: 'static,
    D: Dataset<T>,
    P: Fn(&T) -> f64 + 'static,
    K: Fn(&T) -> f64 + 'static,
    W: Fn(&T) -> f64 + 'static,
>(
    data: D,
    pressure: P,
    temperature: K,
    dewpoint: Option<W>,
    pressure_range: Option<(f64, f64)>,
    temperature_range: Option<(f64, f64)>,
    adiabats: Option<bool>,
    moist: Option<bool>,
    mixing: Option<bool>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
) -> AnyElement {
    thermo(
        &data,
        |item| {
            (
                pressure(item),
                temperature(item),
                dewpoint.as_ref().map_or(f64::NAN, |dew| dew(item)),
            )
        },
        options(
            Kind::Stuve,
            pressure_range,
            temperature_range,
            adiabats,
            moist,
            mixing,
            tooltip,
            class,
        ))
}

#[component]
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub fn tephigram<
    T: 'static,
    D: Dataset<T>,
    P: Fn(&T) -> f64 + 'static,
    K: Fn(&T) -> f64 + 'static,
    W: Fn(&T) -> f64 + 'static,
>(
    data: D,
    pressure: P,
    temperature: K,
    dewpoint: Option<W>,
    pressure_range: Option<(f64, f64)>,
    temperature_range: Option<(f64, f64)>,
    adiabats: Option<bool>,
    moist: Option<bool>,
    mixing: Option<bool>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
) -> AnyElement {
    thermo(
        &data,
        |item| {
            (
                pressure(item),
                temperature(item),
                dewpoint.as_ref().map_or(f64::NAN, |dew| dew(item)),
            )
        },
        options(
            Kind::Tephigram,
            pressure_range,
            temperature_range,
            adiabats,
            moist,
            mixing,
            tooltip,
            class,
        ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn physics_matches_reference_values() {
        assert!((theta(0., 1000.) - 273.15).abs() < 1e-9);
        assert!((theta(-50., 500.) - 223.15 * 2f64.powf(KAPPA)).abs() < 1e-9);
        assert!((saturation(20., 1000.) * 1000. - 14.7).abs() < 0.3);
        assert!((dew_for(saturation(10., 850.), 850.) - 10.).abs() < 1e-6);
        let path = moist_adiabat(20., 1000., 500., 10.);
        let end = path.last().copied().unwrap_or_default();
        assert!((end.1 - 500.).abs() < 1e-9 && end.0 < -5. && end.0 > -15.);
    }
}
