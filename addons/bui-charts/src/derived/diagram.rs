use std::collections::HashMap;
use std::f32::consts::{PI, TAU};

use blenderui::element::AnyElement;
use blenderui::style::TextAlign;
use crate::geom::point;
use blenderui::{Hsla, SharedString};
use crate::color::Hsl;
use crate::component;

use super::scene::{
    At, Frame, Note, Scene, Shape, Show, Zoom, curve, ellipse, polar, scene_view, stage,
    text_width, wedge,
};
use crate::derive::{bits, data_key};
use crate::format::{fixed, trimmed};
use crate::plot::{TEXT_SIZE, Tooltip};
use crate::{Dataset, Legend};

fn share(part: f64, whole: f64) -> SharedString {
    if whole > 0. {
        format!("{}%", fixed(part / whole * 100., 1)).into()
    } else {
        SharedString::Borrowed("–")
    }
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

fn shade(color: Hsla, depth: usize) -> Hsla {
    let hsl = Hsl::from(color);
    Hsl { l: (hsl.l + 0.07 * depth as f32).min(0.85), ..hsl }.into()
}

#[derive(Default)]
struct Tree {
    names: Vec<SharedString>,
    parents: Vec<Option<usize>>,
    children: Vec<Vec<usize>>,
    values: Vec<f64>,
    depths: Vec<usize>,
    levels: Option<usize>,
}

impl Tree {
    fn branch(&self, mut node: usize) -> usize {
        while let Some(parent) = self.parents[node] {
            if self.parents[parent].is_none() {
                return self.children[parent]
                    .iter()
                    .position(|child| *child == node)
                    .unwrap_or(0);
            }
            node = parent;
        }
        0
    }

    fn height(&self, node: usize) -> usize {
        self.children[node]
            .iter()
            .map(|child| 1 + self.height(*child))
            .max()
            .unwrap_or(0)
    }
}

fn grow<T>(
    rows: &[T],
    path: impl Fn(&T) -> &str,
    value: impl Fn(&T) -> f64,
    separator: &str,
    root: SharedString,
) -> Tree {
    let mut tree = Tree {
        names: vec![root],
        parents: vec![None],
        children: vec![Vec::new()],
        values: vec![0.],
        depths: vec![0],
        levels: None,
    };
    let mut index: HashMap<(usize, String), usize> = HashMap::new();
    for item in rows {
        let amount = value(item);
        if !(amount.is_finite() && amount > 0.) {
            continue;
        }
        let mut node = 0;
        for part in path(item)
            .split(separator)
            .map(str::trim)
            .filter(|part| !part.is_empty())
        {
            node = *index.entry((node, part.to_owned())).or_insert_with(|| {
                let next = tree.names.len();
                tree.names.push(SharedString::from(part.to_owned()));
                tree.parents.push(Some(node));
                tree.children.push(Vec::new());
                tree.values.push(0.);
                tree.depths.push(tree.depths[node] + 1);
                tree.children[node].push(next);
                next
            });
        }
        tree.values[node] += amount;
    }
    for node in (1..tree.names.len()).rev() {
        if let Some(parent) = tree.parents[node] {
            tree.values[parent] += tree.values[node];
        }
    }
    tree
}

fn arrange_sunburst(tree: &Tree, frame: &Frame) -> Scene {
    let mut scene = Scene::default();
    let total = tree.values[0];
    for (node, name) in tree.names.iter().enumerate() {
        let parent = tree.parents[node].map_or(total, |parent| tree.values[parent]);
        scene.note(Note {
            title: name.clone(),
            accent: (node > 0).then(|| frame.ink.colors.series(tree.branch(node))),
            rows: vec![
                (None, "Value".into(), trimmed(tree.values[node])),
                (
                    None,
                    "Share of parent".into(),
                    share(tree.values[node], parent),
                ),
                (
                    None,
                    "Share of total".into(),
                    share(tree.values[node], total),
                ),
            ],
        });
    }
    let focus = frame
        .focus
        .filter(|node| *node < tree.names.len())
        .unwrap_or(0);
    let center = point(frame.width / 2., frame.height / 2.);
    let outer = (frame.width.min(frame.height) / 2. - 6.).max(10.);
    let hole = outer * 0.2;
    let depth = tree
        .height(focus)
        .min(tree.levels.unwrap_or(usize::MAX))
        .max(1);
    let ring = (outer - hole) / depth as f32;
    let back = tree.parents[focus].unwrap_or(0);
    scene.shapes.push(Shape::filled(
        ellipse(center, (hole, hole), 64),
        frame.ink.rule,
        Some(back),
    ));
    scene.label(
        tree.names[focus].clone(),
        point(center.x, center.y - TEXT_SIZE),
        TextAlign::Center,
        TEXT_SIZE,
        frame.ink.text,
    );
    scene.label(
        trimmed(tree.values[focus]),
        point(center.x, center.y + 1.),
        TextAlign::Center,
        TEXT_SIZE,
        frame.ink.muted,
    );
    let mut stack = vec![(focus, 0f32, TAU, 0usize)];
    while let Some((node, from, to, level)) = stack.pop() {
        let amount = tree.values[node];
        let mut start = from;
        for child in &tree.children[node] {
            let span = if amount > 0. {
                (tree.values[*child] / amount) as f32 * (to - from)
            } else {
                0.
            };
            let (inner, edge) = (hole + ring * level as f32, hole + ring * (level + 1) as f32);
            if level < depth && span > 0.002 {
                let color = shade(
                    frame.ink.colors.series(tree.branch(*child)),
                    tree.depths[*child].saturating_sub(1),
                );
                scene.shapes.push(
                    Shape::filled(
                        wedge(center, (inner, edge), (start, start + span)),
                        color,
                        Some(*child),
                    )
                    .outlined(frame.ink.background, 1.),
                );
                let middle = f32::midpoint(inner, edge);
                let name = &tree.names[*child];
                if span * middle > text_width(name, TEXT_SIZE) + 4. && ring > TEXT_SIZE + 4. {
                    let at = polar(center, middle, start + span / 2.);
                    scene.label(
                        name.clone(),
                        point(at.x, at.y - TEXT_SIZE / 2.),
                        TextAlign::Center,
                        TEXT_SIZE,
                        frame.ink.background,
                    );
                }
                stack.push((*child, start, start + span, level + 1));
            }
            start += span;
        }
    }
    scene
}

#[component]
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub fn sunburst<
    T: 'static,
    D: Dataset<T>,
    P: Fn(&T) -> &str + 'static,
    V: Fn(&T) -> f64 + 'static,
>(
    data: D,
    path: P,
    value: V,
    separator: Option<SharedString>,
    root: Option<SharedString>,
    levels: Option<usize>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
) -> AnyElement {
    let state = stage::<Tree>();
    let separator = separator.unwrap_or(SharedString::Borrowed("/"));
    let root = root.unwrap_or(SharedString::Borrowed("Total"));
    let key = data_key(&data, (&separator, &root, levels));
    state.hold(
        key, || data.rows(|rows| Tree {
            levels,
            ..grow(rows, &path, &value, &separator, root)
        }));
    scene_view(
        state,
        arrange_sunburst,
        Show {
            groups: Vec::new(),
            legend: None,
            tooltip,
            clickable: true,
            zoom: Zoom::Off,
            class,
        })
}

#[derive(Default)]
struct Chords {
    names: Vec<SharedString>,
    matrix: Vec<f64>,
    pad: f32,
    opacity: f32,
}

impl Chords {
    fn flow(&self, from: usize, to: usize) -> f64 {
        self.matrix
            .get(from * self.names.len() + to)
            .copied()
            .filter(|value| value.is_finite() && *value > 0.)
            .unwrap_or(0.)
    }
}

fn arrange_chord(chords: &Chords, frame: &Frame) -> Scene {
    let mut scene = Scene::default();
    let count = chords.names.len();
    let shown: Vec<usize> = (0..count)
        .filter(|group| !frame.hidden.contains(group))
        .collect();
    let totals: Vec<f64> = (0..count)
        .map(|from| shown.iter().map(|to| chords.flow(from, *to)).sum())
        .collect();
    let grand: f64 = shown.iter().map(|group| totals[*group]).sum();
    if grand <= 0. {
        return scene;
    }
    let center = point(frame.width / 2., frame.height / 2.);
    let outer = (frame.width.min(frame.height) / 2. - 36.).max(12.);
    let inner = outer - 10.;
    let pad = chords.pad.min(TAU / shown.len().max(1) as f32 * 0.5);
    let scale = (TAU - pad * shown.len() as f32) / grand as f32;
    let mut slots: HashMap<(usize, usize), (f32, f32)> = HashMap::new();
    let mut angle = 0.;
    for group in &shown {
        let start = angle;
        for to in &shown {
            let span = chords.flow(*group, *to) as f32 * scale;
            slots.insert((*group, *to), (angle, angle + span));
            angle += span;
        }
        let color = frame.ink.colors.series(*group);
        let incoming: f64 = shown.iter().map(|from| chords.flow(*from, *group)).sum();
        let tip = scene.note(Note {
            title: chords.names[*group].clone(),
            accent: Some(color),
            rows: vec![
                (None, "Out".into(), trimmed(totals[*group])),
                (None, "In".into(), trimmed(incoming)),
            ],
        });
        scene.shapes.push(Shape::filled(
            wedge(center, (inner, outer), (start, angle)),
            color,
            Some(tip),
        ));
        let at = polar(center, outer + 8., f32::midpoint(start, angle));
        let align = side(at, center);
        scene.label(
            chords.names[*group].clone(),
            point(at.x, at.y - TEXT_SIZE / 2.),
            align,
            TEXT_SIZE,
            frame.ink.text,
        );
        angle += pad;
    }
    for (index, a) in shown.iter().enumerate() {
        for b in &shown[index..] {
            let (there, back) = (chords.flow(*a, *b), chords.flow(*b, *a));
            if there + back <= 0. {
                continue;
            }
            let (from, to) = (slots[&(*a, *b)], slots[&(*b, *a)]);
            let arc = |(start, end): (f32, f32)| {
                let steps = ((end - start) / TAU * 96.).ceil().max(2.) as usize;
                (0..=steps).map(move |step| {
                    polar(
                        center,
                        inner,
                        (end - start).mul_add(step as f32 / steps as f32, start),
                    )
                })
            };
            let mut points: Vec<At> = arc(from).collect();
            points.extend(
                curve(
                    polar(center, inner, from.1),
                    center,
                    polar(center, inner, to.0),
                    24,
                )
                .skip(1),
            );
            points.extend(arc(to).skip(1));
            points.extend(
                curve(
                    polar(center, inner, to.1),
                    center,
                    polar(center, inner, from.0),
                    24,
                )
                .skip(1),
            );
            let source = if there >= back { *a } else { *b };
            let tip = scene.note(Note {
                title: format!("{} ↔ {}", chords.names[*a], chords.names[*b]).into(),
                accent: Some(frame.ink.colors.series(source)),
                rows: vec![
                    (
                        None,
                        format!("{} → {}", chords.names[*a], chords.names[*b]).into(),
                        trimmed(there),
                    ),
                    (
                        None,
                        format!("{} → {}", chords.names[*b], chords.names[*a]).into(),
                        trimmed(back),
                    ),
                ],
            });
            scene.shapes.push(Shape::filled(
                points,
                frame.ink.colors.series(source).opacity(chords.opacity),
                Some(tip),
            ));
        }
    }
    scene
}

#[component]
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub fn chord_diagram(
    names: Vec<SharedString>,
    matrix: Vec<f64>,
    pad_angle: Option<f32>,
    ribbon_opacity: Option<f32>,
    legend: Option<Legend>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
) -> AnyElement {
    let state = stage::<Chords>();
    let (pad, opacity) = (
        pad_angle.unwrap_or(2.).to_radians(),
        ribbon_opacity.unwrap_or(0.65),
    );
    let key = {
        use std::hash::{DefaultHasher, Hash, Hasher};
        let mut state = DefaultHasher::new();
        (&names, bits(&matrix), pad.to_bits(), opacity.to_bits()).hash(&mut state);
        Some(state.finish())
    };
    let groups = names.clone();
    state.hold(
        key, || Chords {
            names,
            matrix,
            pad,
            opacity,
        });
    scene_view(
        state,
        arrange_chord,
        Show {
            groups,
            legend: legend.or_else(|| Some(Legend::hidden())),
            tooltip,
            clickable: false,
            zoom: Zoom::Wheel,
            class,
        })
}

#[derive(Default)]
struct Arcs {
    names: Vec<SharedString>,
    links: Vec<(usize, usize, f64)>,
}

fn arrange_arcs(arcs: &Arcs, frame: &Frame) -> Scene {
    let mut scene = Scene::default();
    let count = arcs.names.len();
    if count == 0 {
        return scene;
    }
    let (left, right) = (24., frame.width - 24.);
    let base = frame.height - 44.;
    let x = |node: usize| {
        if count > 1 {
            left + (right - left) * node as f32 / (count - 1) as f32
        } else {
            f32::midpoint(left, right)
        }
    };
    let widest = arcs
        .links
        .iter()
        .map(|(_, _, value)| *value)
        .fold(0., f64::max);
    let reach = arcs
        .links
        .iter()
        .map(|(a, b, _)| (x(*a) - x(*b)).abs() / 2.)
        .fold(0f32, f32::max);
    let squash = if reach > 0. {
        ((base - 8.) / reach).min(1.)
    } else {
        1.
    };
    let mut degree = vec![0f64; count];
    for (from, to, value) in &arcs.links {
        degree[*from] += value;
        degree[*to] += value;
        let (a, b) = (x(*from), x(*to));
        let radius = (b - a).abs() / 2.;
        let middle = f32::midpoint(a, b);
        let points: Vec<At> = (0..=48)
            .map(|step| {
                let t = PI * step as f32 / 48.;
                point(
                    radius.mul_add(t.cos(), middle),
                    (-radius * squash).mul_add(t.sin(), base),
                )
            })
            .collect();
        let color = frame.ink.colors.series(*from);
        let tip = scene.note(Note {
            title: format!("{} → {}", arcs.names[*from], arcs.names[*to]).into(),
            accent: Some(color),
            rows: vec![(None, "Value".into(), trimmed(*value))],
        });
        let width = if widest > 0. {
            (*value / widest) as f32 * 7. + 1.
        } else {
            1.
        };
        scene
            .shapes
            .push(Shape::line(points, color.opacity(0.6), width, Some(tip)));
    }
    let busiest = degree.iter().copied().fold(0., f64::max);
    let spacing = (right - left) / count.max(2).saturating_sub(1) as f32;
    for (node, name) in arcs.names.iter().enumerate() {
        let color = frame.ink.colors.series(node);
        let size = if busiest > 0. {
            3. + 6. * (degree[node] / busiest).sqrt() as f32
        } else {
            4.
        };
        let tip = scene.note(Note {
            title: name.clone(),
            accent: Some(color),
            rows: vec![(None, "Total".into(), trimmed(degree[node]))],
        });
        scene.shapes.push(Shape::filled(
            ellipse(point(x(node), base), (size, size), 24),
            color,
            Some(tip),
        ));
        let level = if spacing < text_width(name, TEXT_SIZE) + 4. {
            node % 2
        } else {
            0
        };
        scene.label(
            name.clone(),
            point(x(node), base + 10. + level as f32 * (TEXT_SIZE + 4.)),
            TextAlign::Center,
            TEXT_SIZE,
            frame.ink.text,
        );
    }
    scene
}

#[component]
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub fn arc_diagram<
    T: 'static,
    D: Dataset<T>,
    S: Fn(&T) -> &str + 'static,
    G: Fn(&T) -> &str + 'static,
    V: Fn(&T) -> f64 + 'static,
>(
    data: D,
    source: S,
    target: G,
    value: Option<V>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
) -> AnyElement {
    let state = stage::<Arcs>();
    let key = data_key(&data, value.is_some());
    state.hold(
        key, || data.rows(|rows| {
            let mut arcs = Arcs::default();
            let mut index: HashMap<String, usize> = HashMap::new();
            let mut node = |label: &str, names: &mut Vec<SharedString>| {
                *index.entry(label.to_owned()).or_insert_with(|| {
                    names.push(SharedString::from(label.to_owned()));
                    names.len() - 1
                })
            };
            for item in rows {
                let amount = value.as_ref().map_or(1., |value| value(item));
                if !(amount.is_finite() && amount > 0.) {
                    continue;
                }
                let from = node(source(item), &mut arcs.names);
                let to = node(target(item), &mut arcs.names);
                if from != to {
                    arcs.links.push((from, to, amount));
                }
            }
            arcs
        }));
    scene_view(
        state,
        arrange_arcs,
        Show {
            groups: Vec::new(),
            legend: None,
            tooltip,
            clickable: false,
            zoom: Zoom::WheelAndDrag,
            class,
        })
}

#[derive(Default)]
struct Fishbone {
    effect: SharedString,
    categories: Vec<SharedString>,
    causes: Vec<Vec<SharedString>>,
}

fn arrange_fishbone(bone: &Fishbone, frame: &Frame) -> Scene {
    let mut scene = Scene::default();
    let head = (text_width(&bone.effect, TEXT_SIZE + 1.) + 20.)
        .min(frame.width * 0.3)
        .max(80.);
    let spine = frame.height / 2.;
    let (start, end) = (12., frame.width - head - 8.);
    let ink = frame.ink.muted;
    scene.shapes.push(Shape::line(
        vec![point(start, spine), point(end, spine)],
        ink,
        2.5,
        None,
    ));
    scene.shapes.push(Shape::filled(
        vec![
            point(end, spine - 7.),
            point(end + 10., spine),
            point(end, spine + 7.),
        ],
        ink,
        None,
    ));
    let box_height = TEXT_SIZE * 2.4;
    scene.shapes.push(
        Shape::filled(
            super::scene::rect(end + 12., spine - box_height / 2., head - 6., box_height),
            frame.ink.colors.series(0),
            None,
        )
        .outlined(frame.ink.background, 1.),
    );
    scene.label(
        bone.effect.clone(),
        point(end + 12. + (head - 6.) / 2., spine - TEXT_SIZE / 2.),
        TextAlign::Center,
        TEXT_SIZE + 1.,
        blenderui::white(),
    );
    let count = bone.categories.len();
    let upper = count.div_ceil(2);
    let slots = upper.max(1);
    let gap = (end - start - 20.) / slots as f32;
    let reach = (spine - 28.).max(20.);
    for (index, (category, causes)) in bone.categories.iter().zip(&bone.causes).enumerate() {
        let (slot, above) = if index < upper {
            (index, true)
        } else {
            (index - upper, false)
        };
        let foot = start + 20. + gap * (slot as f32 + 1.) - 10.;
        let sign = if above { -1. } else { 1. };
        let top = point(foot - reach * 0.55, spine + sign * reach);
        let color = frame.ink.colors.series(index);
        let tip = scene.note(Note {
            title: category.clone(),
            accent: Some(color),
            rows: causes
                .iter()
                .map(|cause| (None, cause.clone(), SharedString::default()))
                .collect(),
        });
        scene.shapes.push(Shape::line(
            vec![top, point(foot, spine)],
            color,
            2.,
            Some(tip),
        ));
        let title_y = if above {
            top.y - TEXT_SIZE - 4.
        } else {
            top.y + 4.
        };
        scene.label(
            category.clone(),
            point(top.x, title_y),
            TextAlign::Center,
            TEXT_SIZE,
            color,
        );
        let room = (gap * 0.9).max(40.);
        for (number, cause) in causes.iter().enumerate() {
            let t = (number as f32 + 1.) / (causes.len() as f32 + 1.);
            let along = point(
                (foot - top.x).mul_add(1. - t, top.x),
                (spine - top.y).mul_add(1. - t, top.y),
            );
            let length = (text_width(cause, TEXT_SIZE) + 8.).min(room);
            scene.shapes.push(Shape::line(
                vec![point(along.x - length, along.y), along],
                ink.opacity(0.6),
                1.,
                Some(tip),
            ));
            let mut label = cause.to_string();
            while label.len() > 1 && text_width(&label, TEXT_SIZE) > length - 4. {
                label.pop();
            }
            if label.len() < cause.len() {
                label.push('…');
            }
            scene.label(
                label,
                point(along.x - 4., along.y - TEXT_SIZE - 1.),
                TextAlign::Right,
                TEXT_SIZE,
                frame.ink.text,
            );
        }
    }
    scene
}

#[component]
#[allow(clippy::needless_pass_by_value)]
pub fn ishikawa<
    T: 'static,
    D: Dataset<T>,
    C: Fn(&T) -> &str + 'static,
    K: Fn(&T) -> &str + 'static,
>(
    data: D,
    effect: SharedString,
    category: C,
    cause: K,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
) -> AnyElement {
    let state = stage::<Fishbone>();
    let key = data_key(&data, &effect);
    state.hold(
        key, || data.rows(|rows| {
            let mut bone = Fishbone {
                effect,
                ..Fishbone::default()
            };
            for item in rows {
                let label = category(item);
                let slot = bone
                    .categories
                    .iter()
                    .position(|known| known == label)
                    .unwrap_or_else(|| {
                        bone.categories.push(SharedString::from(label.to_owned()));
                        bone.causes.push(Vec::new());
                        bone.categories.len() - 1
                    });
                bone.causes[slot].push(SharedString::from(cause(item).to_owned()));
            }
            bone
        }));
    scene_view(
        state,
        arrange_fishbone,
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
    fn sunburst_sums_up_the_paths() {
        let rows = [("a/b", 2.), ("a/c", 3.), ("d", 5.)];
        let tree = grow(&rows, |row| row.0, |row| row.1, "/", "Total".into());
        assert!((tree.values[0] - 10.).abs() < 1e-9);
        assert!((tree.values[1] - 5.).abs() < 1e-9);
        assert_eq!(tree.height(0), 2);
        assert_eq!(tree.branch(2), 0);
        assert_eq!(tree.branch(4), 1);
    }

    #[test]
    fn ishikawa_survives_tiny_frames() {
        let black = blenderui::black();
        let colors = crate::ChartColors {
            palette: vec![black],
            schemes: std::collections::HashMap::default(),
            fallback: black,
            good: black,
            bad: black,
            neutral: black,
            hover: black,
        };
        let hidden = std::collections::BTreeSet::new();
        let bone = Fishbone {
            effect: "Late delivery of the release".into(),
            categories: vec!["People".into(), "Process".into(), "Tools".into()],
            causes: vec![vec!["Training gap".into()], vec![], vec!["Flaky CI".into()]],
        };
        for (width, height) in [(0., 0.), (60., 60.), (1., 400.), (400., 1.), (250., 120.)] {
            let frame = Frame {
                width,
                height,
                ink: super::super::scene::Ink {
                    colors: &colors,
                    text: black,
                    muted: black,
                    rule: black,
                    background: black,
                },
                hidden: &hidden,
                focus: None,
            };
            let scene = arrange_fishbone(&bone, &frame);
            assert!(!scene.shapes.is_empty());
        }
    }
}
