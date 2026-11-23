use std::collections::HashMap;
use std::f32::consts::PI;

use blenderui::element::AnyElement;
use blenderui::style::TextAlign;
use crate::geom::point;
use blenderui::{SharedString};
use crate::component;

use super::scene::{
    Frame, Note, Scene, Shape, Show, Zoom, ellipse, rect, scene_view, stage, text_width,
};
use crate::derive::data_key;
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

fn index_of(map: &mut HashMap<String, usize>, names: &mut Vec<SharedString>, label: &str) -> usize {
    *map.entry(label.to_owned()).or_insert_with(|| {
        names.push(SharedString::from(label.to_owned()));
        names.len() - 1
    })
}

#[derive(Default)]
struct Grid {
    rows: Vec<SharedString>,
    columns: Vec<SharedString>,
    cells: Vec<(usize, usize, f64)>,
    labelled: bool,
}

fn arrange_grid(grid: &Grid, frame: &Frame) -> Scene {
    let mut scene = Scene::default();
    let shown: Vec<usize> = (0..grid.rows.len())
        .filter(|row| !frame.hidden.contains(row))
        .collect();
    if shown.is_empty() || grid.columns.is_empty() {
        return scene;
    }
    let slot: HashMap<usize, usize> = shown
        .iter()
        .enumerate()
        .map(|(slot, row)| (*row, slot))
        .collect();
    let count = grid.columns.len();
    let mut values = vec![0f64; shown.len() * count];
    for (row, column, value) in &grid.cells {
        if let Some(slot) = slot.get(row) {
            values[slot * count + column] += value;
        }
    }
    let rows: Vec<f64> = (0..shown.len())
        .map(|slot| values[slot * count..(slot + 1) * count].iter().sum())
        .collect();
    let columns: Vec<f64> = (0..count)
        .map(|column| {
            (0..shown.len())
                .map(|slot| values[slot * count + column])
                .sum()
        })
        .collect();
    let grand: f64 = rows.iter().sum();
    let widest = values.iter().copied().fold(0., f64::max);
    let tallest = columns.iter().copied().fold(0., f64::max);
    let busiest = rows.iter().copied().fold(0., f64::max);
    if grand <= 0. {
        return scene;
    }
    let label_room = shown
        .iter()
        .map(|row| text_width(&grid.rows[*row], TEXT_SIZE))
        .fold(0., f32::max)
        .min(frame.width / 3.);
    let (left, top, right, bottom) = (label_room + 14., 66., 8., 8.);
    let (width, height) = (
        (frame.width - left - right).max(1.),
        (frame.height - top - bottom).max(1.),
    );
    let (cell_width, cell_height) = (width / count as f32, height / shown.len() as f32);
    let cell = (cell_width.min(cell_height) - 4.).max(1.);
    let thickness = (cell_height - 4.).max(1.) / busiest as f32;
    for (column, name) in grid.columns.iter().enumerate() {
        let x = left + (column as f32 + 0.5) * cell_width;
        let tip = scene.note(Note {
            title: name.clone(),
            accent: None,
            rows: vec![
                (None, "Total".into(), trimmed(columns[column])),
                (None, "Share".into(), share(columns[column], grand)),
            ],
        });
        let tall = (columns[column] / tallest) as f32 * 34.;
        scene.shapes.push(Shape::filled(
            rect(x - cell_width * 0.25, 52. - tall, cell_width * 0.5, tall),
            frame.ink.muted.opacity(0.55),
            Some(tip),
        ));
        if cell_width > text_width(name, TEXT_SIZE) {
            scene.label(
                name.clone(),
                point(x, 0.),
                TextAlign::Center,
                TEXT_SIZE,
                frame.ink.text,
            );
        }
    }
    for (slot, row) in shown.iter().enumerate() {
        let color = frame.ink.colors.series(*row);
        let middle = top + (slot as f32 + 0.5) * cell_height;
        let mut rest = rows[slot];
        let mut edges = vec![(left, rest)];
        for column in 0..count {
            rest -= values[slot * count + column];
            edges.push((left + (column as f32 + 1.) * cell_width, rest.max(0.)));
        }
        let mut band: Vec<_> = edges
            .iter()
            .map(|(x, remaining)| point(*x, middle - *remaining as f32 * thickness / 2.))
            .collect();
        band.extend(
            edges
                .iter()
                .rev()
                .map(|(x, remaining)| point(*x, middle + *remaining as f32 * thickness / 2.)),
        );
        scene
            .shapes
            .push(Shape::filled(band, color.opacity(0.22), None));
        scene.label(
            grid.rows[*row].clone(),
            point(left - 8., middle - TEXT_SIZE / 2.),
            TextAlign::Right,
            TEXT_SIZE,
            frame.ink.text,
        );
        for column in 0..count {
            let value = values[slot * count + column];
            if value <= 0. {
                continue;
            }
            let side = cell * (value / widest).sqrt() as f32;
            let x = left + (column as f32 + 0.5) * cell_width;
            let tip = scene.note(Note {
                title: format!("{} → {}", grid.rows[*row], grid.columns[column]).into(),
                accent: Some(color),
                rows: vec![
                    (None, "Value".into(), trimmed(value)),
                    (None, "Share of row".into(), share(value, rows[slot])),
                    (
                        None,
                        "Share of column".into(),
                        share(value, columns[column]),
                    ),
                    (None, "Share of total".into(), share(value, grand)),
                ],
            });
            scene.shapes.push(Shape::filled(
                rect(x - side / 2., middle - side / 2., side, side),
                color,
                Some(tip),
            ));
            let text = trimmed(value);
            if grid.labelled && side > text_width(&text, TEXT_SIZE) + 4. && side > TEXT_SIZE + 2. {
                scene.label(
                    text,
                    point(x, middle - TEXT_SIZE / 2.),
                    TextAlign::Center,
                    TEXT_SIZE,
                    frame.ink.background,
                );
            }
        }
    }
    scene
}

#[component]
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub fn sankey_matrix<
    T: 'static,
    D: Dataset<T>,
    S: Fn(&T) -> &str + 'static,
    G: Fn(&T) -> &str + 'static,
    V: Fn(&T) -> f64 + 'static,
>(
    data: D,
    source: S,
    target: G,
    value: V,
    show_values: Option<bool>,
    legend: Option<Legend>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
) -> AnyElement {
    let state = stage::<Grid>();
    let labelled = show_values.unwrap_or(true);
    let key = data_key(&data, labelled);
    state.hold(
        key, || data.rows(|items| {
            let mut grid = Grid {
                labelled,
                ..Grid::default()
            };
            let (mut rows, mut columns) = (HashMap::new(), HashMap::new());
            for item in items {
                let amount = value(item);
                if !(amount.is_finite() && amount > 0.) {
                    continue;
                }
                let row = index_of(&mut rows, &mut grid.rows, source(item));
                let column = index_of(&mut columns, &mut grid.columns, target(item));
                grid.cells.push((row, column, amount));
            }
            grid
        }));
    let groups = state.held.with(|held| held.model.rows.clone());
    scene_view(
        state,
        arrange_grid,
        Show {
            groups,
            legend,
            tooltip,
            clickable: false,
            zoom: Zoom::Off,
            class,
        })
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ForceLink {
    pub source: usize,
    pub target: usize,
    pub weight: f64,
}

#[derive(Default)]
struct Cluster {
    names: Vec<SharedString>,
    groups: Vec<usize>,
    values: Vec<f64>,
    group_names: Vec<SharedString>,
    links: Vec<ForceLink>,
    iterations: usize,
    distance: f32,
    labelled: bool,
}

fn squarify(weights: &[f64], (mut x, mut y, mut w, mut h): (f32, f32, f32, f32)) -> Vec<[f32; 4]> {
    let total: f64 = weights.iter().sum();
    let scale = if total > 0. {
        f64::from(w * h) / total
    } else {
        0.
    };
    let areas: Vec<f64> = weights.iter().map(|weight| weight * scale).collect();
    let mut placed = Vec::with_capacity(areas.len());
    let mut start = 0;
    while start < areas.len() {
        let side = f64::from(w.min(h)).max(1e-6);
        let worst = |sum: f64, smallest: f64| {
            (side * side * areas[start] / (sum * sum)).max(sum * sum / (side * side * smallest))
        };
        let mut sum = areas[start];
        let mut best = worst(sum, areas[start]);
        let mut end = start + 1;
        while end < areas.len() {
            let score = worst(sum + areas[end], areas[end]);
            if score > best {
                break;
            }
            best = score;
            sum += areas[end];
            end += 1;
        }
        let thick = (sum / side) as f32;
        let wide = w >= h;
        let mut offset = 0f32;
        for area in &areas[start..end] {
            let length = if thick > 0. { *area as f32 / thick } else { 0. };
            placed.push(if wide {
                [x, y + offset, thick, length]
            } else {
                [x + offset, y, length, thick]
            });
            offset += length;
        }
        if wide {
            x += thick;
            w -= thick;
        } else {
            y += thick;
            h -= thick;
        }
        start = end;
    }
    placed
}

struct Body {
    x: f32,
    y: f32,
    r: f32,
    pen: [f32; 4],
}

fn settle(
    bodies: &mut [Body],
    members: &[Vec<usize>],
    springs: &[(usize, usize, f32)],
    cluster: &Cluster,
) {
    let steps = cluster.iterations.max(1);
    for step in 0..steps {
        let alpha = 1. - step as f32 / steps as f32;
        for (a, b, weight) in springs {
            let (dx, dy) = (bodies[*b].x - bodies[*a].x, bodies[*b].y - bodies[*a].y);
            let length = dx.hypot(dy).max(0.01);
            let want = bodies[*a].r + bodies[*b].r + cluster.distance;
            let pull = (length - want) / length * 0.3 * alpha * weight;
            bodies[*a].x += dx * pull;
            bodies[*a].y += dy * pull;
            bodies[*b].x -= dx * pull;
            bodies[*b].y -= dy * pull;
        }
        for group in members {
            for (slot, a) in group.iter().enumerate() {
                for b in &group[slot + 1..] {
                    let (dx, dy) = (bodies[*b].x - bodies[*a].x, bodies[*b].y - bodies[*a].y);
                    let length = dx.hypot(dy).max(0.01);
                    let overlap = bodies[*a].r + bodies[*b].r + 1.5 - length;
                    if overlap > 0. {
                        let push = overlap / length / 2.;
                        bodies[*a].x -= dx * push;
                        bodies[*a].y -= dy * push;
                        bodies[*b].x += dx * push;
                        bodies[*b].y += dy * push;
                    }
                }
            }
        }
        for body in bodies.iter_mut() {
            let [x0, y0, x1, y1] = body.pen;
            body.x = body.x.clamp(x0 + body.r, (x1 - body.r).max(x0 + body.r));
            body.y = body.y.clamp(y0 + body.r, (y1 - body.r).max(y0 + body.r));
        }
    }
}

fn arrange_cluster(cluster: &Cluster, frame: &Frame) -> Scene {
    let mut scene = Scene::default();
    let mut totals = vec![0f64; cluster.group_names.len()];
    for (group, value) in cluster.groups.iter().zip(&cluster.values) {
        if !frame.hidden.contains(group) {
            totals[*group] += value;
        }
    }
    let mut order: Vec<usize> = (0..totals.len())
        .filter(|group| totals[*group] > 0.)
        .collect();
    order.sort_by(|a, b| totals[*b].total_cmp(&totals[*a]));
    if order.is_empty() {
        return scene;
    }
    let weights: Vec<f64> = order.iter().map(|group| totals[*group]).collect();
    let rects = squarify(
        &weights,
        (0., 0., frame.width.max(1.), frame.height.max(1.)),
    );
    let header = TEXT_SIZE + 6.;
    let mut pens = vec![[0f32; 4]; totals.len()];
    let mut members: Vec<Vec<usize>> = vec![Vec::new(); totals.len()];
    for (group, [x, y, w, h]) in order.iter().zip(&rects) {
        let color = frame.ink.colors.series(*group);
        let (x, y, w, h) = (x + 1., y + 1., (w - 2.).max(1.), (h - 2.).max(1.));
        scene.shapes.push(
            Shape::filled(rect(x, y, w, h), color.opacity(0.08), None)
                .outlined(color.opacity(0.5), 1.),
        );
        let title = &cluster.group_names[*group];
        if w > text_width(title, TEXT_SIZE) + 8. && h > header + 8. {
            scene.label(
                title.clone(),
                point(x + 4., y + 3.),
                TextAlign::Left,
                TEXT_SIZE,
                frame.ink.muted,
            );
        }
        let top = if h > header + 24. { y + header } else { y };
        pens[*group] = [x + 2., top + 2., x + w - 2., y + h - 2.];
    }
    let mut index = vec![usize::MAX; cluster.names.len()];
    let mut bodies: Vec<Body> = Vec::new();
    let mut ids: Vec<usize> = Vec::new();
    for (node, group) in cluster.groups.iter().enumerate() {
        if frame.hidden.contains(group) || cluster.values[node] <= 0. {
            continue;
        }
        let pen = pens[*group];
        let (aw, ah) = (pen[2] - pen[0], pen[3] - pen[1]);
        let part = (cluster.values[node] / totals[*group]) as f32;
        let r = (part * aw * ah * 0.5 / PI)
            .sqrt()
            .clamp(2.5, (aw.min(ah) / 2. - 1.).max(2.5));
        let turn = members[*group].len() as f32;
        let spread = aw.min(ah) * 0.35 * ((turn + 0.5) / (turn + 4.)).sqrt();
        let angle = turn * 2.399_963;
        index[node] = bodies.len();
        members[*group].push(bodies.len());
        ids.push(node);
        bodies.push(Body {
            x: f32::midpoint(pen[0], pen[2]) + spread * angle.cos(),
            y: f32::midpoint(pen[1], pen[3]) + spread * angle.sin(),
            r,
            pen,
        });
    }
    let strongest = cluster
        .links
        .iter()
        .map(|link| link.weight)
        .fold(0., f64::max);
    let springs: Vec<(usize, usize, f32)> = cluster
        .links
        .iter()
        .filter(|link| index[link.source] != usize::MAX && index[link.target] != usize::MAX)
        .map(|link| {
            (
                index[link.source],
                index[link.target],
                if strongest > 0. {
                    (0.4 + 0.6 * link.weight / strongest) as f32
                } else {
                    1.
                },
            )
        })
        .collect();
    settle(&mut bodies, &members, &springs, cluster);
    for (a, b, weight) in &springs {
        scene.shapes.push(Shape::line(
            vec![
                point(bodies[*a].x, bodies[*a].y),
                point(bodies[*b].x, bodies[*b].y),
            ],
            frame.ink.muted.opacity(0.5),
            weight.mul_add(1.5, 0.5),
            None,
        ));
    }
    let mut degree = vec![0usize; cluster.names.len()];
    for link in &cluster.links {
        degree[link.source] += 1;
        degree[link.target] += 1;
    }
    for (body, node) in bodies.iter().zip(&ids) {
        let color = frame.ink.colors.series(cluster.groups[*node]);
        let tip = scene.note(Note {
            title: cluster.names[*node].clone(),
            accent: Some(color),
            rows: vec![
                (
                    None,
                    "Group".into(),
                    cluster.group_names[cluster.groups[*node]].clone(),
                ),
                (None, "Value".into(), trimmed(cluster.values[*node])),
                (None, "Links".into(), degree[*node].to_string().into()),
            ],
        });
        scene.shapes.push(
            Shape::filled(
                ellipse(point(body.x, body.y), (body.r, body.r), 28),
                color,
                Some(tip),
            )
            .outlined(frame.ink.background, 1.),
        );
        if cluster.labelled && body.r * 2. > text_width(&cluster.names[*node], TEXT_SIZE) + 4. {
            scene.label(
                cluster.names[*node].clone(),
                point(body.x, body.y - TEXT_SIZE / 2.),
                TextAlign::Center,
                TEXT_SIZE,
                frame.ink.background,
            );
        }
    }
    scene
}

#[component]
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub fn treemap_force<
    T: 'static,
    D: Dataset<T>,
    N: Fn(&T) -> &str + 'static,
    G: Fn(&T) -> &str + 'static,
    V: Fn(&T) -> f64 + 'static,
>(
    data: D,
    name: N,
    group: G,
    value: V,
    links: Option<Vec<ForceLink>>,
    iterations: Option<usize>,
    distance: Option<f32>,
    show_labels: Option<bool>,
    legend: Option<Legend>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
) -> AnyElement {
    let state = stage::<Cluster>();
    let links = links.unwrap_or_default();
    let (iterations, distance, labelled) = (
        iterations.unwrap_or(240),
        distance.unwrap_or(14.).max(0.),
        show_labels.unwrap_or(true),
    );
    let tied: Vec<(usize, usize, u64)> = links
        .iter()
        .map(|link| (link.source, link.target, link.weight.to_bits()))
        .collect();
    let key = data_key(&data, (tied, iterations, distance.to_bits(), labelled));
    state.hold(
        key, || data.rows(|items| {
            let mut cluster = Cluster {
                links,
                iterations,
                distance,
                labelled,
                ..Cluster::default()
            };
            let mut known = HashMap::new();
            for item in items {
                cluster
                    .names
                    .push(SharedString::from(name(item).to_owned()));
                cluster
                    .groups
                    .push(index_of(&mut known, &mut cluster.group_names, group(item)));
                let amount = value(item);
                cluster.values.push(if amount.is_finite() {
                    amount.max(0.)
                } else {
                    0.
                });
            }
            let count = cluster.names.len();
            cluster.links.retain(|link| {
                link.source < count && link.target < count && link.source != link.target
            });
            cluster
        }));
    let groups = state.held.with(|held| held.model.group_names.clone());
    scene_view(
        state,
        arrange_cluster,
        Show {
            groups,
            legend,
            tooltip,
            clickable: false,
            zoom: Zoom::Off,
            class,
        })
}
