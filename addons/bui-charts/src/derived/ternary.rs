use blenderui::style::TextAlign;
use blenderui::{AnyElement, SharedString};

use super::scene::{
    Area, At, Frame, Ink, Note, Scene, Shape, Show, Zoom, ellipse, scene_view, stage,
};
use crate::derive::data_key;
use crate::format::{fixed, trimmed};
use crate::plot::{TEXT_SIZE, Tooltip};
use crate::geom::point;
use crate::{Dataset, HeatmapScheme, Legend, component};

const ROOT: f32 = 0.866_025_4;

#[derive(Clone, Copy)]
pub(crate) struct Triangle {
    pub origin: At,
    pub side: f32,
}

impl Triangle {
    fn fit(area: Area) -> Self {
        let side = area.width().min(area.height() / ROOT);
        Self {
            origin: point(
                area.left + (area.width() - side) / 2.,
                area.bottom - (area.height() - side * ROOT) / 2.,
            ),
            side,
        }
    }

    pub(crate) fn at(self, a: f64, b: f64, c: f64) -> At {
        let total = a + b + c;
        let (b, c) = if total > 0. {
            ((b / total) as f32, (c / total) as f32)
        } else {
            (0., 0.)
        };
        point(
            (c / 2. + b).mul_add(self.side, self.origin.x),
            (-c * ROOT).mul_add(self.side, self.origin.y),
        )
    }

    fn corners(self) -> [At; 3] {
        [
            self.at(1., 0., 0.),
            self.at(0., 1., 0.),
            self.at(0., 0., 1.),
        ]
    }

    pub(crate) fn draw(
        self,
        scene: &mut Scene,
        divisions: usize,
        names: &[SharedString; 3],
        ink: &Ink,
    ) {
        let steps = divisions.max(1);
        for step in 1..steps {
            let t = step as f64 / steps as f64;
            for (from, to) in [
                (self.at(t, 1. - t, 0.), self.at(t, 0., 1. - t)),
                (self.at(1. - t, t, 0.), self.at(0., t, 1. - t)),
                (self.at(1. - t, 0., t), self.at(0., 1. - t, t)),
            ] {
                scene
                    .shapes
                    .push(Shape::line(vec![from, to], ink.rule, 1., None));
            }
            let share = format!("{}", (t * 100.).round());
            let small = TEXT_SIZE - 2.;
            let along = self.at(1. - t, t, 0.);
            scene.label(
                share.clone(),
                point(along.x, along.y + 3.),
                TextAlign::Center,
                small,
                ink.muted,
            );
            let right = self.at(0., 1. - t, t);
            scene.label(
                share.clone(),
                point(right.x + 4., right.y - small / 2.),
                TextAlign::Left,
                small,
                ink.muted,
            );
            let left = self.at(1. - t, 0., t);
            scene.label(
                share,
                point(left.x - 4., left.y - small / 2.),
                TextAlign::Right,
                small,
                ink.muted,
            );
        }
        let [a, b, c] = self.corners();
        scene
            .shapes
            .push(Shape::line(vec![a, b, c, a], ink.muted, 1.5, None));
        scene.label(
            names[0].clone(),
            point(a.x - 4., a.y + TEXT_SIZE + 2.),
            TextAlign::Left,
            TEXT_SIZE,
            ink.text,
        );
        scene.label(
            names[1].clone(),
            point(b.x + 4., b.y + TEXT_SIZE + 2.),
            TextAlign::Right,
            TEXT_SIZE,
            ink.text,
        );
        scene.label(
            names[2].clone(),
            point(c.x, c.y - TEXT_SIZE - 6.),
            TextAlign::Center,
            TEXT_SIZE,
            ink.text,
        );
    }
}

fn percent(value: f64, total: f64) -> SharedString {
    if total > 0. {
        format!("{}%", fixed(value / total * 100., 1)).into()
    } else {
        SharedString::Borrowed("–")
    }
}

fn names(names: Option<Vec<SharedString>>, fallback: [&'static str; 3]) -> [SharedString; 3] {
    let mut given = names.unwrap_or_default().into_iter();
    fallback.map(|name| {
        given
            .next()
            .unwrap_or(SharedString::Borrowed(name))
    })
}

fn groups<T>(rows: &[T], group: Option<&dyn Fn(&T) -> &str>) -> (Vec<usize>, Vec<SharedString>) {
    let mut names: Vec<SharedString> = Vec::new();
    let members = rows
        .iter()
        .map(|item| {
            group.map_or(0, |group| {
                let label = group(item);
                names
                    .iter()
                    .position(|known| known == label)
                    .unwrap_or_else(|| {
                        names.push(SharedString::from(label.to_owned()));
                        names.len() - 1
                    })
            })
        })
        .collect();
    (members, names)
}

#[derive(Default)]
struct Mixture {
    groups: Vec<SharedString>,
    names: [SharedString; 3],
    parts: Vec<[f64; 3]>,
    labels: Vec<SharedString>,
    members: Vec<usize>,
    size: f32,
    divisions: usize,
}

fn arrange_ternary(mixture: &Mixture, frame: &Frame) -> Scene {
    let mut scene = Scene::default();
    let triangle = Triangle::fit(Area::inset(frame.width, frame.height, (40., 28., 40., 24.)));
    triangle.draw(&mut scene, mixture.divisions, &mixture.names, &frame.ink);
    for (index, [a, b, c]) in mixture.parts.iter().enumerate() {
        let group = mixture.members[index];
        if frame.hidden.contains(&group) || !(a + b + c).is_finite() {
            continue;
        }
        let total = a + b + c;
        let color = frame.ink.colors.series(group);
        let tip = scene.note(Note {
            title: mixture
                .labels
                .get(index)
                .cloned()
                .unwrap_or_else(|| format!("Point {}", index + 1).into()),
            accent: Some(color),
            rows: mixture
                .names
                .iter()
                .zip([a, b, c])
                .map(|(name, part)| (None, name.clone(), percent(*part, total)))
                .collect(),
        });
        scene.shapes.push(
            Shape::filled(
                ellipse(triangle.at(*a, *b, *c), (mixture.size, mixture.size), 16),
                color.opacity(0.7),
                Some(tip),
            )
            .outlined(color, 1.),
        );
    }
    scene
}

#[component]
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub fn ternary<
    T: 'static,
    D: Dataset<T>,
    A: Fn(&T) -> f64 + 'static,
    B: Fn(&T) -> f64 + 'static,
    C: Fn(&T) -> f64 + 'static,
    N: Fn(&T) -> &str + 'static,
    G: Fn(&T) -> &str + 'static,
>(
    data: D,
    a: A,
    b: B,
    c: C,
    name: Option<N>,
    group: Option<G>,
    corners: Option<Vec<SharedString>>,
    divisions: Option<usize>,
    point_size: Option<f32>,
    legend: Option<Legend>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
) -> AnyElement {
    let state = stage::<Mixture>();
    let corners = names(corners, ["A", "B", "C"]);
    let (size, divisions) = (point_size.unwrap_or(5.), divisions.unwrap_or(10));
    let key = data_key(
        &data,
        (
            &corners,
            size.to_bits(),
            divisions,
            group.is_some(),
            name.is_some(),
        ));
    state.hold(
        key,
        || data.rows(|rows| {
            let (members, found) = groups(
                rows,
                group.as_ref().map(|group| group as &dyn Fn(&T) -> &str),
            );
            Mixture {
                groups: found,
                names: corners,
                parts: rows
                    .iter()
                    .map(|item| [a(item), b(item), c(item)])
                    .collect(),
                labels: name.as_ref().map_or_else(Vec::new, |name| {
                    rows.iter()
                        .map(|item| SharedString::from(name(item).to_owned()))
                        .collect()
                }),
                members,
                size,
                divisions,
            }
        }));
    let legends = state.held.with(|held| held.model.groups.clone());
    scene_view(
        state,
        arrange_ternary,
        Show {
            groups: legends,
            legend,
            tooltip,
            clickable: false,
            zoom: Zoom::Off,
            class,
        })
}

#[derive(Default)]
struct Mesh {
    names: [SharedString; 3],
    steps: usize,
    values: Vec<f64>,
    range: (f64, f64),
    levels: usize,
    scheme: HeatmapScheme,
    edges: bool,
}

impl Mesh {
    const fn index(&self, i: usize, j: usize) -> usize {
        i * (self.steps + 1) + j
    }

    fn vertex(&self, triangle: Triangle, row: usize, column: usize) -> (At, f64) {
        let steps = self.steps as f64;
        let (b, c) = (column as f64 / steps, row as f64 / steps);
        (
            triangle.at(1. - b - c, b, c),
            self.values[self.index(row, column)],
        )
    }
}

fn crossing(level: f64, (a, va): (At, f64), (b, vb): (At, f64)) -> Option<At> {
    if (va - level) * (vb - level) >= 0. || (va - vb).abs() < f64::EPSILON {
        return None;
    }
    let t = ((level - va) / (vb - va)) as f32;
    Some(point(
        (b.x - a.x).mul_add(t, a.x),
        (b.y - a.y).mul_add(t, a.y),
    ))
}

fn arrange_mesh(mesh: &Mesh, frame: &Frame) -> Scene {
    let mut scene = Scene::default();
    let triangle = Triangle::fit(Area::inset(frame.width, frame.height, (40., 28., 80., 24.)));
    let (low, high) = mesh.range;
    let span = (high - low).max(f64::MIN_POSITIVE);
    let mut cells: Vec<[(At, f64); 3]> = Vec::with_capacity(mesh.steps * mesh.steps);
    for i in 0..mesh.steps {
        for j in 0..mesh.steps - i {
            cells.push([
                mesh.vertex(triangle, i, j),
                mesh.vertex(triangle, i, j + 1),
                mesh.vertex(triangle, i + 1, j),
            ]);
            if j + 1 < mesh.steps - i {
                cells.push([
                    mesh.vertex(triangle, i, j + 1),
                    mesh.vertex(triangle, i + 1, j + 1),
                    mesh.vertex(triangle, i + 1, j),
                ]);
            }
        }
    }
    for cell in &cells {
        let value = (cell[0].1 + cell[1].1 + cell[2].1) / 3.;
        let color = mesh.scheme.sample(((value - low) / span).clamp(0., 1.));
        let tip = scene.note(Note {
            title: trimmed(value),
            accent: Some(color),
            rows: Vec::new(),
        });
        let mut shape = Shape::filled(
            cell.iter().map(|vertex| vertex.0).collect(),
            color,
            Some(tip),
        );
        shape = if mesh.edges {
            shape.outlined(frame.ink.background.opacity(0.4), 0.5)
        } else {
            shape.outlined(color, 0.5)
        };
        scene.shapes.push(shape);
    }
    for level in 1..=mesh.levels {
        let value = span.mul_add(level as f64 / (mesh.levels + 1) as f64, low);
        for cell in &cells {
            let hits: Vec<At> = [(0, 1), (1, 2), (2, 0)]
                .iter()
                .filter_map(|(from, to)| crossing(value, cell[*from], cell[*to]))
                .collect();
            if hits.len() == 2 {
                scene
                    .shapes
                    .push(Shape::line(hits, frame.ink.text.opacity(0.55), 1., None));
            }
        }
    }
    triangle.draw(&mut scene, 10, &mesh.names, &frame.ink);
    let right = (frame.width - 56.).max(0.);
    let scheme = mesh.scheme;
    scene.scale_bar(
        |t| scheme.sample(t),
        mesh.range,
        Area {
            left: right,
            top: 32.,
            right: right + 12.,
            bottom: frame.height - 32.,
        },
        &frame.ink,
    );
    scene
}

#[component]
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub fn ternary_contour<F: Fn(f64, f64, f64) -> f64 + 'static>(
    field: F,
    corners: Option<Vec<SharedString>>,
    resolution: Option<usize>,
    levels: Option<usize>,
    scheme: Option<HeatmapScheme>,
    mesh: Option<bool>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
) -> AnyElement {
    let state = stage::<Mesh>();
    let corners = names(corners, ["A", "B", "C"]);
    let (steps, count, scheme, edges) = (
        resolution.unwrap_or(30).clamp(2, 200),
        levels.unwrap_or(8),
        scheme.unwrap_or(HeatmapScheme::Viridis),
        mesh.unwrap_or(false),
    );
    let key = {
        use std::hash::{DefaultHasher, Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        (&corners, steps, count, scheme, edges).hash(&mut hasher);
        Some(hasher.finish())
    };
    state.hold(
        key,
        || {
            let n = steps as f64;
            let mut values = Vec::with_capacity((steps + 1) * (steps + 1));
            for i in 0..=steps {
                for j in 0..=steps {
                    let (b, c) = (j as f64 / n, i as f64 / n);
                    values.push(if i + j <= steps {
                        field(1. - b - c, b, c)
                    } else {
                        f64::NAN
                    });
                }
            }
            let range = values
                .iter()
                .copied()
                .filter(|value| value.is_finite())
                .fold((f64::INFINITY, f64::NEG_INFINITY), |(low, high), value| {
                    (low.min(value), high.max(value))
                });
            Mesh {
                names: corners,
                steps,
                values,
                range: if range.0 <= range.1 { range } else { (0., 1.) },
                levels: count,
                scheme,
                edges,
            }
        });
    scene_view(
        state,
        arrange_mesh,
        Show {
            groups: Vec::new(),
            legend: None,
            tooltip,
            clickable: false,
            zoom: Zoom::Off,
            class,
        })
}

#[derive(Default)]
struct Water {
    groups: Vec<SharedString>,
    samples: Vec<[f64; 6]>,
    labels: Vec<SharedString>,
    members: Vec<usize>,
    size: f32,
}

fn arrange_piper(water: &Water, frame: &Frame) -> Scene {
    let mut scene = Scene::default();
    let gap = 0.15;
    let area = Area::inset(frame.width, frame.height, (44., 24., 44., 24.));
    let height_units = gap / 2. * ROOT * 2. + 2. * ROOT;
    let side = (area.width() / (2. + gap)).min(area.height() / height_units);
    let left = area.left + (area.width() - side * (2. + gap)) / 2.;
    let base = area.bottom - (area.height() - side * height_units) / 2.;
    let cations = Triangle {
        origin: point(left, base),
        side,
    };
    let anions = Triangle {
        origin: point(left + side * (1. + gap), base),
        side,
    };
    let bottom = point(
        left + side * (1. + gap / 2.),
        base - side * gap / 2. * ROOT * 2.,
    );
    let diamond = |sodium: f64, chloride: f64| {
        let (u, v) = (sodium as f32, chloride as f32);
        point(
            ((u - v) * 0.5).mul_add(side, bottom.x),
            (-(u + v) * ROOT).mul_add(side, bottom.y),
        )
    };
    let label = |text: &str| SharedString::from(text.to_owned());
    cations.draw(
        &mut scene,
        5,
        &[label("Ca²⁺"), label("Na⁺+K⁺"), label("Mg²⁺")],
        &frame.ink,
    );
    anions.draw(
        &mut scene,
        5,
        &[label("HCO₃⁻"), label("Cl⁻"), label("SO₄²⁻")],
        &frame.ink,
    );
    for step in 1..5 {
        let t = f64::from(step) / 5.;
        scene.shapes.push(Shape::line(
            vec![diamond(t, 0.), diamond(t, 1.)],
            frame.ink.rule,
            1.,
            None,
        ));
        scene.shapes.push(Shape::line(
            vec![diamond(0., t), diamond(1., t)],
            frame.ink.rule,
            1.,
            None,
        ));
    }
    let outline = vec![
        diamond(0., 0.),
        diamond(1., 0.),
        diamond(1., 1.),
        diamond(0., 1.),
        diamond(0., 0.),
    ];
    scene
        .shapes
        .push(Shape::line(outline, frame.ink.muted, 1.5, None));
    scene.label(
        "SO₄²⁻+Cl⁻",
        point(diamond(0.5, 1.).x - 10., diamond(0.5, 1.).y - TEXT_SIZE),
        TextAlign::Right,
        TEXT_SIZE,
        frame.ink.text,
    );
    scene.label(
        "Na⁺+K⁺",
        point(diamond(1., 0.5).x + 10., diamond(1., 0.5).y - TEXT_SIZE),
        TextAlign::Left,
        TEXT_SIZE,
        frame.ink.text,
    );
    for (index, sample) in water.samples.iter().enumerate() {
        let group = water.members[index];
        if frame.hidden.contains(&group) {
            continue;
        }
        let [calcium, magnesium, sodium, bicarbonate, sulfate, chloride] = *sample;
        let (positive, negative) = (
            calcium + magnesium + sodium,
            bicarbonate + sulfate + chloride,
        );
        if !(positive > 0. && negative > 0.) {
            continue;
        }
        let color = frame.ink.colors.series(group);
        let tip = scene.note(Note {
            title: water
                .labels
                .get(index)
                .cloned()
                .unwrap_or_else(|| format!("Sample {}", index + 1).into()),
            accent: Some(color),
            rows: vec![
                (None, "Ca²⁺".into(), percent(calcium, positive)),
                (None, "Mg²⁺".into(), percent(magnesium, positive)),
                (None, "Na⁺+K⁺".into(), percent(sodium, positive)),
                (None, "HCO₃⁻".into(), percent(bicarbonate, negative)),
                (None, "SO₄²⁻".into(), percent(sulfate, negative)),
                (None, "Cl⁻".into(), percent(chloride, negative)),
            ],
        });
        for at in [
            cations.at(calcium, sodium, magnesium),
            anions.at(bicarbonate, chloride, sulfate),
            diamond(sodium / positive, (sulfate + chloride) / negative),
        ] {
            scene.shapes.push(
                Shape::filled(
                    ellipse(at, (water.size, water.size), 16),
                    color.opacity(0.75),
                    Some(tip),
                )
                .outlined(color, 1.),
            );
        }
    }
    scene
}

#[component]
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub fn piper<
    T: 'static,
    D: Dataset<T>,
    Ca: Fn(&T) -> f64 + 'static,
    Mg: Fn(&T) -> f64 + 'static,
    Na: Fn(&T) -> f64 + 'static,
    Hc: Fn(&T) -> f64 + 'static,
    So: Fn(&T) -> f64 + 'static,
    Cl: Fn(&T) -> f64 + 'static,
    N: Fn(&T) -> &str + 'static,
    G: Fn(&T) -> &str + 'static,
>(
    data: D,
    calcium: Ca,
    magnesium: Mg,
    sodium: Na,
    bicarbonate: Hc,
    sulfate: So,
    chloride: Cl,
    name: Option<N>,
    group: Option<G>,
    point_size: Option<f32>,
    legend: Option<Legend>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
) -> AnyElement {
    let state = stage::<Water>();
    let size = point_size.unwrap_or(5.);
    let key = data_key(&data, (size.to_bits(), group.is_some(), name.is_some()));
    state.hold(
        key,
        || data.rows(|rows| {
            let (members, found) = groups(
                rows,
                group.as_ref().map(|group| group as &dyn Fn(&T) -> &str),
            );
            Water {
                groups: found,
                samples: rows
                    .iter()
                    .map(|item| {
                        [
                            calcium(item),
                            magnesium(item),
                            sodium(item),
                            bicarbonate(item),
                            sulfate(item),
                            chloride(item),
                        ]
                    })
                    .collect(),
                labels: name.as_ref().map_or_else(Vec::new, |name| {
                    rows.iter()
                        .map(|item| SharedString::from(name(item).to_owned()))
                        .collect()
                }),
                members,
                size,
            }
        }));
    let legends = state.held.with(|held| held.model.groups.clone());
    scene_view(
        state,
        arrange_piper,
        Show {
            groups: legends,
            legend,
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
    fn corners_land_on_the_triangle() {
        let triangle = Triangle {
            origin: point(0., 100.),
            side: 100.,
        };
        let top = triangle.at(0., 0., 1.);
        assert!((top.x - 50.).abs() < 1e-3 && (top.y - (100. - 100. * ROOT)).abs() < 1e-3);
        let middle = triangle.at(1., 1., 1.);
        assert!((middle.x - 50.).abs() < 1e-3);
        assert!(
            crossing(0.5, (point(0., 0.), 0.), (point(10., 0.), 1.))
                .is_some_and(|at| (at.x - 5.).abs() < 1e-4)
        );
    }
}
