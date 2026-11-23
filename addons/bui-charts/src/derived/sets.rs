use std::f64::consts::PI;

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

#[derive(Default)]
pub(crate) struct Sets {
    pub names: Vec<SharedString>,
    pub combos: Vec<(u64, f64)>,
}

impl Sets {
    fn parse<T>(
        rows: &[T],
        sets: impl Fn(&T) -> &str,
        value: Option<&dyn Fn(&T) -> f64>,
        separator: &str,
    ) -> Self {
        let mut found = Self::default();
        for item in rows {
            let amount = value.map_or(1., |value| value(item));
            if !(amount.is_finite() && amount > 0.) {
                continue;
            }
            let mut mask = 0u64;
            for part in sets(item)
                .split(separator)
                .map(str::trim)
                .filter(|part| !part.is_empty())
            {
                let slot = found
                    .names
                    .iter()
                    .position(|known| known == part)
                    .unwrap_or_else(|| {
                        found.names.push(SharedString::from(part.to_owned()));
                        found.names.len() - 1
                    });
                if slot < 64 {
                    mask |= 1 << slot;
                }
            }
            if mask == 0 {
                continue;
            }
            match found.combos.iter_mut().find(|(known, _)| *known == mask) {
                Some((_, total)) => *total += amount,
                None => found.combos.push((mask, amount)),
            }
        }
        found
    }

    pub(crate) fn size(&self, mask: u64) -> f64 {
        self.combos
            .iter()
            .filter(|(combo, _)| combo & mask == mask)
            .map(|(_, value)| value)
            .sum()
    }

    fn label(&self, mask: u64, joiner: &str) -> SharedString {
        self.names
            .iter()
            .enumerate()
            .filter(|(slot, _)| mask & (1 << slot) != 0)
            .map(|(_, name)| name.as_ref())
            .collect::<Vec<_>>()
            .join(joiner)
            .into()
    }
}

fn overlap(a: f64, b: f64, gap: f64) -> f64 {
    if gap >= a + b {
        return 0.;
    }
    if gap <= (a - b).abs() {
        return PI * a.min(b).powi(2);
    }
    let x = (gap * gap + a * a - b * b) / (2. * gap * a);
    let y = (gap * gap + b * b - a * a) / (2. * gap * b);
    let part = |radius: f64, cosine: f64| radius * radius * cosine.clamp(-1., 1.).acos();
    let core = 0.5
        * ((-gap + a + b) * (gap + a - b) * (gap - a + b) * (gap + a + b))
            .max(0.)
            .sqrt();
    part(a, x) + part(b, y) - core
}

pub(crate) fn spacing(a: f64, b: f64, shared: f64) -> f64 {
    if shared <= 0. {
        return a + b + (a + b) * 0.08;
    }
    let target = shared.min(PI * a.min(b).powi(2) * 0.999);
    let (mut low, mut high) = ((a - b).abs(), a + b);
    for _ in 0..60 {
        let middle = f64::midpoint(low, high);
        if overlap(a, b, middle) > target {
            low = middle;
        } else {
            high = middle;
        }
    }
    f64::midpoint(low, high)
}

#[derive(Default)]
struct Euler {
    sets: Sets,
    circles: Vec<(f64, f64, f64)>,
    opacity: f32,
}

fn place(sets: &Sets) -> Vec<(f64, f64, f64)> {
    let count = sets.names.len().min(64);
    let radii: Vec<f64> = (0..count)
        .map(|slot| (sets.size(1 << slot) / PI).sqrt())
        .collect();
    let gaps = |a: usize, b: usize| spacing(radii[a], radii[b], sets.size((1 << a) | (1 << b)));
    let mut circles: Vec<(f64, f64, f64)> = Vec::with_capacity(count);
    for slot in 0..count {
        let spot = match slot {
            0 => (0., 0.),
            1 => (gaps(0, 1), 0.),
            _ => {
                let span: f64 = radii.iter().take(slot).sum::<f64>() * 2. + radii[slot] * 2.;
                let mut best = ((0., 0.), f64::INFINITY);
                for column in 0..=60 {
                    for line in 0..=60 {
                        let at = (
                            span * (f64::from(column) / 60. - 0.5) * 2.,
                            span * (f64::from(line) / 60. - 0.5) * 2.,
                        );
                        let error: f64 = circles
                            .iter()
                            .enumerate()
                            .map(|(other, (x, y, _))| {
                                ((at.0 - x).hypot(at.1 - y) - gaps(other, slot)).powi(2)
                            })
                            .sum();
                        if error < best.1 {
                            best = (at, error);
                        }
                    }
                }
                best.0
            }
        };
        circles.push((spot.0, spot.1, radii[slot]));
    }
    circles
}

fn arrange_euler(euler: &Euler, frame: &Frame) -> Scene {
    let mut scene = Scene::default();
    let shown: Vec<usize> = (0..euler.circles.len())
        .filter(|slot| !frame.hidden.contains(slot))
        .collect();
    if shown.is_empty() {
        return scene;
    }
    let (mut left, mut right, mut top, mut bottom) = (
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
    );
    for slot in &shown {
        let (x, y, r) = euler.circles[*slot];
        (left, right, top, bottom) = (
            left.min(x - r),
            right.max(x + r),
            top.min(y - r),
            bottom.max(y + r),
        );
    }
    let margin = 28.;
    let scale = (f64::from(frame.width - margin * 2.) / (right - left))
        .min(f64::from(frame.height - margin * 2.) / (bottom - top));
    let offset = (
        (f64::from(frame.width) - (right - left) * scale) / 2. - left * scale,
        (f64::from(frame.height) - (bottom - top) * scale) / 2. - top * scale,
    );
    let at =
        |(x, y): (f64, f64)| point((x * scale + offset.0) as f32, (y * scale + offset.1) as f32);
    let middle = (f64::midpoint(left, right), f64::midpoint(top, bottom));
    let total: f64 = euler.sets.combos.iter().map(|(_, value)| value).sum();
    for slot in &shown {
        let (x, y, r) = euler.circles[*slot];
        let color = frame.ink.colors.series(*slot);
        let size = euler.sets.size(1 << slot);
        let only = euler
            .sets
            .combos
            .iter()
            .find(|(mask, _)| *mask == 1 << slot)
            .map_or(0., |(_, value)| *value);
        let tip = scene.note(Note {
            title: euler.sets.names[*slot].clone(),
            accent: Some(color),
            rows: vec![
                (None, "Size".into(), trimmed(size)),
                (None, "Only this set".into(), trimmed(only)),
                (
                    None,
                    "Share of all".into(),
                    format!("{}%", fixed(size / total.max(f64::MIN_POSITIVE) * 100., 1)).into(),
                ),
            ],
        });
        let radius = (r * scale) as f32;
        scene.shapes.push(
            Shape::filled(
                ellipse(at((x, y)), (radius, radius), 96),
                color.opacity(euler.opacity),
                Some(tip),
            )
            .outlined(color, 2.),
        );
        let away = (x - middle.0, y - middle.1);
        let length = away.0.hypot(away.1);
        let direction = if length > 1e-9 {
            (away.0 / length, away.1 / length)
        } else {
            (0., -1.)
        };
        let edge = at((
            x + direction.0 * (r + 14. / scale),
            y + direction.1 * (r + 14. / scale),
        ));
        let align = if direction.0 > 0.3 {
            TextAlign::Left
        } else if direction.0 < -0.3 {
            TextAlign::Right
        } else {
            TextAlign::Center
        };
        scene.label(
            euler.sets.names[*slot].clone(),
            point(edge.x, edge.y - TEXT_SIZE / 2.),
            align,
            TEXT_SIZE,
            color,
        );
    }
    let samples = 90;
    let mut sums: Vec<(u64, f64, f64, f64)> = Vec::new();
    for column in 0..samples {
        for line in 0..samples {
            let probe = (
                left + (right - left) * (f64::from(column) + 0.5) / f64::from(samples),
                top + (bottom - top) * (f64::from(line) + 0.5) / f64::from(samples),
            );
            let mask = shown
                .iter()
                .filter(|slot| {
                    let (x, y, r) = euler.circles[**slot];
                    (probe.0 - x).hypot(probe.1 - y) < r
                })
                .fold(0u64, |mask, slot| mask | (1 << slot));
            if mask == 0 {
                continue;
            }
            match sums.iter_mut().find(|entry| entry.0 == mask) {
                Some(entry) => {
                    (entry.1, entry.2, entry.3) =
                        (entry.1 + probe.0, entry.2 + probe.1, entry.3 + 1.);
                }
                None => sums.push((mask, probe.0, probe.1, 1.)),
            }
        }
    }
    let visible = shown.iter().fold(0u64, |mask, slot| mask | (1 << slot));
    for (mask, x, y, count) in sums {
        let value: f64 = euler
            .sets
            .combos
            .iter()
            .filter(|(combo, _)| combo & visible == mask)
            .map(|(_, value)| value)
            .sum();
        if value > 0. {
            let spot = at((x / count, y / count));
            scene.label(
                trimmed(value),
                point(spot.x, spot.y - TEXT_SIZE / 2.),
                TextAlign::Center,
                TEXT_SIZE,
                frame.ink.text,
            );
        }
    }
    scene
}

#[component]
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub fn venn<T: 'static, D: Dataset<T>, S: Fn(&T) -> &str + 'static, V: Fn(&T) -> f64 + 'static>(
    data: D,
    sets: S,
    value: Option<V>,
    separator: Option<SharedString>,
    fill_opacity: Option<f32>,
    legend: Option<Legend>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
) -> AnyElement {
    let state = stage::<Euler>();
    let separator = separator.unwrap_or(SharedString::Borrowed("&"));
    let opacity = fill_opacity.unwrap_or(0.25);
    let key = data_key(&data, (&separator, value.is_some(), opacity.to_bits()));
    state.hold(
        key, || data.rows(|rows| {
            let found = Sets::parse(
                rows,
                &sets,
                value.as_ref().map(|value| value as &dyn Fn(&T) -> f64),
                &separator,
            );
            Euler {
                circles: place(&found),
                sets: found,
                opacity,
            }
        }));
    let groups = state.held.with(|held| held.model.sets.names.clone());
    scene_view(
        state,
        arrange_euler,
        Show {
            groups,
            legend: legend.or_else(|| Some(Legend::hidden())),
            tooltip,
            clickable: false,
            zoom: Zoom::Off,
            class,
        })
}

#[derive(Default)]
struct Crossing {
    sets: Sets,
    order: Vec<usize>,
}

fn arrange_upset(upset: &Crossing, frame: &Frame) -> Scene {
    let mut scene = Scene::default();
    let count = upset.sets.names.len().min(64);
    let columns = upset.order.len();
    if count == 0 || columns == 0 {
        return scene;
    }
    let names = upset
        .sets
        .names
        .iter()
        .map(|name| text_width(name, TEXT_SIZE))
        .fold(0f32, f32::max)
        + 12.;
    let bars = (frame.width * 0.16).clamp(40., 120.);
    let left = bars + names;
    let row = ((frame.height * 0.45) / count as f32).clamp(10., 26.);
    let matrix = row * count as f32;
    let top_height = (frame.height - matrix - 12.).max(30.);
    let width = (frame.width - left - 8.) / columns as f32;
    let tallest = upset
        .order
        .iter()
        .map(|combo| upset.sets.combos[*combo].1)
        .fold(0., f64::max);
    let total: f64 = upset.sets.combos.iter().map(|(_, value)| value).sum();
    let color = frame.ink.colors.series(0);
    let base = top_height + 4.;
    for slot in 0..count {
        let y = base + row * slot as f32;
        if slot % 2 == 0 {
            scene.shapes.push(Shape::filled(
                rect(left, y, frame.width - left - 8., row),
                frame.ink.rule.opacity(0.5),
                None,
            ));
        }
        let size = upset.sets.size(1 << slot);
        let biggest = (0..count)
            .map(|other| upset.sets.size(1 << other))
            .fold(0., f64::max);
        let long = if biggest > 0. {
            (size / biggest) as f32 * (bars - 8.)
        } else {
            0.
        };
        let tip = scene.note(Note {
            title: upset.sets.names[slot].clone(),
            accent: Some(frame.ink.colors.series(1)),
            rows: vec![(None, "Set size".into(), trimmed(size))],
        });
        scene.shapes.push(Shape::filled(
            rect(bars - long, y + row * 0.2, long, row * 0.6),
            frame.ink.colors.series(1),
            Some(tip),
        ));
        scene.label(
            upset.sets.names[slot].clone(),
            point(left - 6., y + (row - TEXT_SIZE) / 2.),
            TextAlign::Right,
            TEXT_SIZE,
            frame.ink.text,
        );
    }
    for (column, combo) in upset.order.iter().enumerate() {
        let (mask, value) = upset.sets.combos[*combo];
        let x = left + width * column as f32;
        let tall = if tallest > 0. {
            (value / tallest) as f32 * (top_height - TEXT_SIZE - 6.)
        } else {
            0.
        };
        let tip = scene.note(Note {
            title: upset.sets.label(mask, " ∩ "),
            accent: Some(color),
            rows: vec![
                (None, "Size".into(), trimmed(value)),
                (
                    None,
                    "Share of all".into(),
                    format!("{}%", fixed(value / total.max(f64::MIN_POSITIVE) * 100., 1)).into(),
                ),
            ],
        });
        scene.shapes.push(Shape::filled(
            rect(x + width * 0.15, top_height - tall, width * 0.7, tall),
            color,
            Some(tip),
        ));
        let shown = trimmed(value);
        if text_width(&shown, TEXT_SIZE) < width {
            scene.label(
                shown,
                point(x + width / 2., top_height - tall - TEXT_SIZE - 2.),
                TextAlign::Center,
                TEXT_SIZE,
                frame.ink.muted,
            );
        }
        let dot = (row * 0.32).min(width * 0.35);
        let members: Vec<usize> = (0..count).filter(|slot| mask & (1 << slot) != 0).collect();
        if let (Some(first), Some(last)) = (members.first(), members.last()) {
            let center = |slot: usize| base + row * (slot as f32 + 0.5);
            scene.shapes.push(Shape::line(
                vec![
                    point(x + width / 2., center(*first)),
                    point(x + width / 2., center(*last)),
                ],
                color,
                2.,
                Some(tip),
            ));
        }
        for slot in 0..count {
            let filled = mask & (1 << slot) != 0;
            let tint = if filled { color } else { frame.ink.rule };
            scene.shapes.push(Shape::filled(
                ellipse(
                    point(x + width / 2., base + row * (slot as f32 + 0.5)),
                    (dot, dot),
                    20,
                ),
                tint,
                filled.then_some(tip),
            ));
        }
    }
    scene
}

#[component]
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub fn up_set<
    T: 'static,
    D: Dataset<T>,
    S: Fn(&T) -> &str + 'static,
    V: Fn(&T) -> f64 + 'static,
>(
    data: D,
    sets: S,
    value: Option<V>,
    separator: Option<SharedString>,
    max_intersections: Option<usize>,
    min_size: Option<f64>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
) -> AnyElement {
    let state = stage::<Crossing>();
    let separator = separator.unwrap_or(SharedString::Borrowed("&"));
    let (limit, floor) = (max_intersections.unwrap_or(20), min_size.unwrap_or(0.));
    let key = data_key(
        &data,
        (&separator, value.is_some(), limit, floor.to_bits()));
    state.hold(
        key, || data.rows(|rows| {
            let found = Sets::parse(
                rows,
                &sets,
                value.as_ref().map(|value| value as &dyn Fn(&T) -> f64),
                &separator,
            );
            let mut order: Vec<usize> = (0..found.combos.len())
                .filter(|combo| found.combos[*combo].1 >= floor)
                .collect();
            order.sort_by(|a, b| found.combos[*b].1.total_cmp(&found.combos[*a].1));
            order.truncate(limit);
            Crossing { sets: found, order }
        }));
    scene_view(
        state,
        arrange_upset,
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
    fn spacing_matches_the_overlap() {
        let gap = spacing(1., 1., 1.);
        assert!((overlap(1., 1., gap) - 1.).abs() < 1e-6);
        assert!(spacing(1., 1., 0.) > 2.);
    }

    #[test]
    fn sets_count_exact_combinations() {
        let rows = ["A", "A&B", "B", "A&B", "C"];
        let sets = Sets::parse(&rows, |row| row, None, "&");
        assert_eq!(sets.names.len(), 3);
        assert!((sets.size(1) - 3.).abs() < 1e-9);
        assert!((sets.size(0b11) - 2.).abs() < 1e-9);
        assert_eq!(sets.label(0b11, " ∩ "), "A ∩ B");
    }
}
