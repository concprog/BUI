use std::collections::HashMap;

use blenderui::{AnyElement, SharedString};

use crate::component;
use crate::geom::point;

use super::scene::{Area, At, Frame, Note, Scene, Shape, Show, Zoom, scene_view, stage};
use crate::derive::data_key;
use crate::format::{fixed, trimmed};
use crate::plot::Tooltip;
use crate::{Dataset, HeatmapScheme};

#[derive(Clone, Debug, PartialEq)]
pub struct Region {
    pub name: SharedString,
    pub rings: Vec<Vec<(f64, f64)>>,
}

impl Region {
    pub fn new(name: impl Into<SharedString>, rings: Vec<Vec<(f64, f64)>>) -> Self {
        Self {
            name: name.into(),
            rings,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Projection {
    #[default]
    Equirectangular,
    Mercator,
    Plane,
}

impl Projection {
    fn apply(self, (x, y): (f64, f64)) -> (f64, f64) {
        match self {
            Self::Equirectangular | Self::Plane => (x, y),
            Self::Mercator => {
                let latitude = y.clamp(-85., 85.).to_radians();
                (
                    x,
                    (std::f64::consts::FRAC_PI_4 + latitude / 2.)
                        .tan()
                        .ln()
                        .to_degrees(),
                )
            }
        }
    }
}

type Shaped = (SharedString, Vec<Vec<(f64, f64)>>, f64);

#[derive(Default)]
struct Map {
    shapes: Vec<Shaped>,
    range: (f64, f64),
    scheme: HeatmapScheme,
    decimals: Option<usize>,
    bar: bool,
}

fn arrange_map(map: &Map, frame: &Frame) -> Scene {
    let mut scene = Scene::default();
    let room = if map.bar { 56. } else { 8. };
    let area = Area::inset(frame.width, frame.height, (8., 8., room, 8.));
    let (mut left, mut right, mut bottom, mut top) = (
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
    );
    for (_, rings, _) in &map.shapes {
        for (x, y) in rings.iter().flatten() {
            (left, right, bottom, top) = (left.min(*x), right.max(*x), bottom.min(*y), top.max(*y));
        }
    }
    if left >= right || bottom >= top {
        return scene;
    }
    let scale =
        (f64::from(area.width()) / (right - left)).min(f64::from(area.height()) / (top - bottom));
    let offset = (
        f64::from(area.left) + (f64::from(area.width()) - (right - left) * scale) / 2.,
        f64::from(area.top) + (f64::from(area.height()) - (top - bottom) * scale) / 2.,
    );
    let at = |(x, y): (f64, f64)| -> At {
        point(
            ((x - left) * scale + offset.0) as f32,
            ((top - y) * scale + offset.1) as f32,
        )
    };
    let (low, high) = map.range;
    let span = (high - low).max(f64::MIN_POSITIVE);
    for (name, rings, value) in &map.shapes {
        let filled = value.is_finite();
        let color = if filled {
            map.scheme.sample(((value - low) / span).clamp(0., 1.))
        } else {
            frame.ink.rule
        };
        let shown = if filled {
            map.decimals
                .map_or_else(|| trimmed(*value), |decimals| fixed(*value, decimals))
        } else {
            SharedString::Borrowed("No data")
        };
        let tip = scene.note(Note {
            title: name.clone(),
            accent: filled.then_some(color),
            rows: vec![(None, "Value".into(), shown)],
        });
        for ring in rings {
            let points: Vec<At> = ring.iter().copied().map(at).collect();
            scene
                .shapes
                .push(Shape::filled(points, color, Some(tip)).outlined(frame.ink.background, 0.8));
        }
    }
    if map.bar {
        let right = frame.width - 46.;
        scene.scale_bar(
            |t| map.scheme.sample(t),
            map.range,
            Area {
                left: right,
                top: 24.,
                right: right + 12.,
                bottom: frame.height - 24.,
            },
            &frame.ink,
        );
    }
    scene
}

#[component]
pub fn choropleth<
    T: 'static,
    D: Dataset<T>,
    R: Dataset<Region>,
    N: Fn(&T) -> &str + 'static,
    V: Fn(&T) -> f64 + 'static,
>(
    data: D,
    regions: R,
    name: N,
    value: V,
    projection: Option<Projection>,
    scheme: Option<HeatmapScheme>,
    range: Option<(f64, f64)>,
    decimals: Option<usize>,
    scale_bar: Option<bool>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
) -> AnyElement {
    let state = stage::<Map>();
    let (projection, scheme, bar) = (
        projection.unwrap_or_default(),
        scheme.unwrap_or(HeatmapScheme::Blues),
        scale_bar.unwrap_or(true),
    );
    let settings = (
        projection,
        scheme,
        range.map(|(low, high)| (low.to_bits(), high.to_bits())),
        decimals,
        bar,
    );
    let key = regions.revision().and_then(|revision| data_key(&data, (revision, settings)));
    state.hold(
        key,
        || {
            // two datasets read at once: the names borrow from the rows
            let shapes: Vec<Shaped> = data.rows(|rows| {
                let values: HashMap<&str, f64> = rows.iter().map(|item| (name(item), value(item))).collect();
                regions.rows(|regions| regions
                .iter()
                .map(|region| {
                    let rings = region
                        .rings
                        .iter()
                        .map(|ring| {
                            ring.iter()
                                .map(|corner| projection.apply(*corner))
                                .collect()
                        })
                        .collect();
                    (
                        region.name.clone(),
                        rings,
                        values
                            .get(region.name.as_ref())
                            .copied()
                            .unwrap_or(f64::NAN),
                    )
                })
                .collect())
            });
            let found = shapes
                .iter()
                .map(|shape| shape.2)
                .filter(|value| value.is_finite())
                .fold((f64::INFINITY, f64::NEG_INFINITY), |(low, high), value| {
                    (low.min(value), high.max(value))
                });
            Map {
                shapes,
                range: range.unwrap_or(if found.0 <= found.1 { found } else { (0., 1.) }),
                scheme,
                decimals,
                bar,
            }
        });
    scene_view(
        state,
        arrange_map,
        Show {
            groups: Vec::new(),
            legend: None,
            tooltip,
            clickable: false,
            zoom: Zoom::Off,
            class,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mercator_stretches_high_latitudes() {
        let (_, equator) = Projection::Mercator.apply((0., 0.));
        let (_, north) = Projection::Mercator.apply((0., 60.));
        assert!(equator.abs() < 1e-9);
        assert!(north > 60.);
    }
}
