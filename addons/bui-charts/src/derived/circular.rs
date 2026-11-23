use std::f32::consts::TAU;
use std::hash::{DefaultHasher, Hash, Hasher};

use blenderui::element::AnyElement;
use blenderui::style::TextAlign;
use crate::geom::point;
use blenderui::{SharedString};
use crate::component;

use super::matrix::apportion;
use super::scene::{
    At, Frame, Note, Scene, Shape, Show, Zoom, curve, polar, scene_view, stage, text_width, wedge,
};
use crate::derive::data_key;
use crate::format::{fixed, trimmed};
use crate::plot::{TEXT_SIZE, Tooltip};
use crate::{Dataset, Legend};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CircosLink {
    pub source: usize,
    pub from: (f64, f64),
    pub target: usize,
    pub to: (f64, f64),
    pub weight: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CircosBin {
    pub segment: usize,
    pub start: f64,
    pub end: f64,
    pub value: f64,
}

#[derive(Default)]
struct Ring {
    segments: Vec<(SharedString, f64)>,
    links: Vec<CircosLink>,
    tracks: Vec<Vec<CircosBin>>,
    gap: f32,
    ticks: Option<f64>,
    opacity: f32,
}

fn side(at: At, center: At) -> TextAlign {
    if (at.x - center.x).abs() < 4. {
        TextAlign::Center
    } else if at.x > center.x {
        TextAlign::Left
    } else {
        TextAlign::Right
    }
}

fn range(from: f64, to: f64) -> SharedString {
    format!("{}–{}", trimmed(from), trimmed(to)).into()
}

fn arrange_circos(ring: &Ring, frame: &Frame) -> Scene {
    let mut scene = Scene::default();
    let count = ring.segments.len();
    let total: f64 = ring.segments.iter().map(|(_, length)| length.max(0.)).sum();
    if count == 0 || total <= 0. {
        return scene;
    }
    let center = point(frame.width / 2., frame.height / 2.);
    let outer = (frame.width.min(frame.height) / 2. - 34.).max(20.);
    let band = outer - 10.;
    let gap = ring.gap.to_radians().min(TAU / count as f32 * 0.5);
    let unit = (TAU - gap * count as f32) / total as f32;
    let mut starts = Vec::with_capacity(count);
    let mut angle = 0.;
    for (index, (name, length)) in ring.segments.iter().enumerate() {
        starts.push(angle);
        let end = angle + *length as f32 * unit;
        let color = frame.ink.colors.series(index);
        let tip = scene.note(Note {
            title: name.clone(),
            accent: Some(color),
            rows: vec![(None, "Length".into(), trimmed(*length))],
        });
        scene.shapes.push(Shape::filled(
            wedge(center, (band, outer), (angle, end)),
            color,
            Some(tip),
        ));
        let at = polar(center, outer + 14., f32::midpoint(angle, end));
        scene.label(
            name.clone(),
            point(at.x, at.y - TEXT_SIZE / 2.),
            side(at, center),
            TEXT_SIZE,
            frame.ink.text,
        );
        if let Some(step) = ring.ticks.filter(|step| *step > 0.) {
            let mut mark = step;
            while mark < *length {
                let tick = angle + mark as f32 * unit;
                scene.shapes.push(Shape::line(
                    vec![polar(center, outer, tick), polar(center, outer + 4., tick)],
                    frame.ink.muted,
                    1.,
                    None,
                ));
                mark += step;
            }
        }
        angle = end + gap;
    }
    let place = |segment: usize, position: f64| {
        starts
            .get(segment)
            .map(|start| start + position as f32 * unit)
    };
    let tracks = ring.tracks.len();
    let thickness = if tracks > 0 {
        (outer * 0.38 / tracks as f32).min(34.)
    } else {
        0.
    };
    let mut inner = band - 4.;
    for (track, bins) in ring.tracks.iter().enumerate() {
        let peak = bins
            .iter()
            .map(|bin| bin.value)
            .filter(|value| value.is_finite())
            .fold(0., f64::max);
        let (base, color) = (inner - thickness, frame.ink.colors.series(count + track));
        scene.shapes.push(Shape::filled(
            wedge(center, (base, inner - 1.), (0., TAU)),
            frame.ink.rule.opacity(0.35),
            None,
        ));
        for bin in bins {
            let (Some(from), Some(to)) =
                (place(bin.segment, bin.start), place(bin.segment, bin.end))
            else {
                continue;
            };
            if !(bin.value.is_finite() && bin.value > 0. && peak > 0.) {
                continue;
            }
            let height = (bin.value / peak) as f32 * (thickness - 2.);
            let tip = scene.note(Note {
                title: ring.segments[bin.segment].0.clone(),
                accent: Some(color),
                rows: vec![
                    (None, "Range".into(), range(bin.start, bin.end)),
                    (None, "Value".into(), trimmed(bin.value)),
                ],
            });
            scene.shapes.push(Shape::filled(
                wedge(center, (base, base + height), (from, to.max(from + 0.002))),
                color,
                Some(tip),
            ));
        }
        inner = base - 3.;
    }
    let reach = inner - 2.;
    let widest = ring.links.iter().map(|link| link.weight).fold(0., f64::max);
    for link in &ring.links {
        let ends = (
            place(link.source, link.from.0),
            place(link.source, link.from.1),
            place(link.target, link.to.0),
            place(link.target, link.to.1),
        );
        let (Some(a0), Some(a1), Some(b0), Some(b1)) = ends else {
            continue;
        };
        let color = frame.ink.colors.series(link.source);
        let tip = scene.note(Note {
            title: format!(
                "{} → {}",
                ring.segments[link.source].0, ring.segments[link.target].0
            )
            .into(),
            accent: Some(color),
            rows: vec![
                (None, "From".into(), range(link.from.0, link.from.1)),
                (None, "To".into(), range(link.to.0, link.to.1)),
                (None, "Weight".into(), trimmed(link.weight)),
            ],
        });
        if (a1 - a0).abs() < 0.004 && (b1 - b0).abs() < 0.004 {
            let width = if widest > 0. {
                (link.weight / widest) as f32 * 3. + 0.8
            } else {
                1.
            };
            let points: Vec<At> = curve(
                polar(center, reach, a0),
                center,
                polar(center, reach, b0),
                32,
            )
            .collect();
            scene.shapes.push(Shape::line(
                points,
                color.opacity(ring.opacity),
                width,
                Some(tip),
            ));
            continue;
        }
        let arc = |(start, end): (f32, f32)| {
            let steps = ((end - start).abs() / TAU * 96.).ceil().max(2.) as usize;
            (0..=steps).map(move |step| {
                polar(
                    center,
                    reach,
                    (end - start).mul_add(step as f32 / steps as f32, start),
                )
            })
        };
        let mut points: Vec<At> = arc((a0, a1)).collect();
        points.extend(
            curve(
                polar(center, reach, a1),
                center,
                polar(center, reach, b0),
                24,
            )
            .skip(1),
        );
        points.extend(arc((b0, b1)).skip(1));
        points.extend(
            curve(
                polar(center, reach, b1),
                center,
                polar(center, reach, a0),
                24,
            )
            .skip(1),
        );
        scene.shapes.push(Shape::filled(
            points,
            color.opacity(ring.opacity),
            Some(tip),
        ));
    }
    scene
}

#[component]
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub fn circos(
    segments: Vec<(SharedString, f64)>,
    links: Option<Vec<CircosLink>>,
    tracks: Option<Vec<Vec<CircosBin>>>,
    gap: Option<f32>,
    ticks: Option<f64>,
    link_opacity: Option<f32>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
) -> AnyElement {
    let state = stage::<Ring>();
    let (links, tracks) = (links.unwrap_or_default(), tracks.unwrap_or_default());
    let (gap, opacity) = (gap.unwrap_or(2.), link_opacity.unwrap_or(0.55));
    let key = {
        let mut hasher = DefaultHasher::new();
        for (name, length) in &segments {
            (name, length.to_bits()).hash(&mut hasher);
        }
        for link in &links {
            [link.from.0, link.from.1, link.to.0, link.to.1, link.weight]
                .map(f64::to_bits)
                .hash(&mut hasher);
            (link.source, link.target).hash(&mut hasher);
        }
        for bins in &tracks {
            bins.len().hash(&mut hasher);
            for bin in bins {
                (
                    bin.segment,
                    [bin.start, bin.end, bin.value].map(f64::to_bits),
                )
                    .hash(&mut hasher);
            }
        }
        (gap.to_bits(), ticks.map(f64::to_bits), opacity.to_bits()).hash(&mut hasher);
        Some(hasher.finish())
    };
    state.hold(
        key, || Ring {
            segments,
            links,
            tracks,
            gap,
            ticks,
            opacity,
        });
    scene_view(
        state,
        arrange_circos,
        Show {
            groups: Vec::new(),
            legend: None,
            tooltip,
            clickable: false,
            zoom: Zoom::Wheel,
            class,
        })
}

#[derive(Default)]
struct Wheel {
    names: Vec<SharedString>,
    values: Vec<f64>,
    rings: usize,
    sectors: usize,
    hole: f32,
    gap: f32,
}

fn arrange_wheel(wheel: &Wheel, frame: &Frame) -> Scene {
    let mut scene = Scene::default();
    let visible: Vec<f64> = wheel
        .values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            if frame.hidden.contains(&index) {
                0.
            } else {
                value.max(0.)
            }
        })
        .collect();
    let cells = wheel.rings * wheel.sectors;
    let counts = apportion(&visible, cells);
    let total: f64 = visible.iter().sum();
    let center = point(frame.width / 2., frame.height / 2.);
    let outer = (frame.width.min(frame.height) / 2. - 8.).max(10.);
    let inner = outer * wheel.hole;
    let depth = (outer - inner) / wheel.rings as f32;
    let width = TAU / wheel.sectors as f32;
    let notes: Vec<usize> = wheel
        .names
        .iter()
        .enumerate()
        .map(|(index, name)| {
            scene.note(Note {
                title: name.clone(),
                accent: Some(frame.ink.colors.series(index)),
                rows: vec![
                    (None, "Value".into(), trimmed(visible[index])),
                    (
                        None,
                        "Share".into(),
                        format!(
                            "{}%",
                            fixed(visible[index] / total.max(f64::MIN_POSITIVE) * 100., 1)
                        )
                        .into(),
                    ),
                    (
                        None,
                        "Cells".into(),
                        counts.get(index).copied().unwrap_or(0).to_string().into(),
                    ),
                ],
            })
        })
        .collect();
    let owners: Vec<Option<usize>> = counts
        .iter()
        .enumerate()
        .flat_map(|(index, count)| std::iter::repeat_n(Some(index), *count))
        .chain(std::iter::repeat(None))
        .take(cells)
        .collect();
    let pad = wheel.gap * 0.5;
    for sector in 0..wheel.sectors {
        for ring in 0..wheel.rings {
            let owner = owners[sector * wheel.rings + ring];
            let (low, high) = (
                inner + depth * ring as f32,
                inner + depth * (ring + 1) as f32,
            );
            let (start, end) = (width * sector as f32, width * (sector + 1) as f32);
            let shrink = pad * depth;
            let spin = (shrink / high.max(1.)).min(width * 0.45);
            let color = owner.map_or(frame.ink.rule, |owner| frame.ink.colors.series(owner));
            scene.shapes.push(
                Shape::filled(
                    wedge(
                        center,
                        (low + shrink / 2., high - shrink / 2.),
                        (start + spin / 2., end - spin / 2.),
                    ),
                    color,
                    owner.map(|owner| notes[owner]),
                )
                .loose(shrink + 1.),
            );
        }
    }
    let hidden_total = wheel
        .values
        .iter()
        .filter(|value| value.is_finite())
        .sum::<f64>();
    if inner > TEXT_SIZE * 2. {
        scene.label(
            trimmed(total),
            point(center.x, center.y - TEXT_SIZE),
            TextAlign::Center,
            TEXT_SIZE + 2.,
            frame.ink.text,
        );
        let caption = if (total - hidden_total).abs() < f64::EPSILON {
            "total"
        } else {
            "shown"
        };
        if text_width(caption, TEXT_SIZE) < inner * 2. {
            scene.label(
                caption,
                point(center.x, center.y + 2.),
                TextAlign::Center,
                TEXT_SIZE,
                frame.ink.muted,
            );
        }
    }
    scene
}

#[component]
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub fn warped_waffle<
    T: 'static,
    D: Dataset<T>,
    N: Fn(&T) -> &str + 'static,
    V: Fn(&T) -> f64 + 'static,
>(
    data: D,
    name: N,
    value: V,
    rings: Option<usize>,
    sectors: Option<usize>,
    hole: Option<f32>,
    gap: Option<f32>,
    legend: Option<Legend>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
) -> AnyElement {
    let state = stage::<Wheel>();
    let (rings, sectors) = (rings.unwrap_or(5).max(1), sectors.unwrap_or(20).max(1));
    let (hole, gap) = (
        hole.unwrap_or(0.35).clamp(0., 0.9),
        gap.unwrap_or(0.12).clamp(0., 0.8),
    );
    let key = data_key(&data, (rings, sectors, hole.to_bits(), gap.to_bits()));
    state.hold(
        key, || data.rows(|rows| {
            
            Wheel {
                names: rows
                    .iter()
                    .map(|item| SharedString::from(name(item).to_owned()))
                    .collect(),
                values: rows.iter().map(&value).collect(),
                rings,
                sectors,
                hole,
                gap,
            }
        }));
    let groups = state.held.with(|held| held.model.names.clone());
    scene_view(
        state,
        arrange_wheel,
        Show {
            groups,
            legend,
            tooltip,
            clickable: false,
            zoom: Zoom::Off,
            class,
        })
}
