use blenderui::element::AnyElement;
use blenderui::style::TextAlign;
use crate::geom::point;
use blenderui::{SharedString};
use crate::component;

use super::scene::{
    Frame, Note, Scene, Shape, Show, Zoom, hexagon, rect, scene_view, stage, text_width,
};
use crate::derive::data_key;
use crate::format::{fixed, trimmed};
use crate::plot::{TEXT_SIZE, Tooltip};
use crate::thresholds::{fallback, steps};
use crate::{Dataset, Legend, Thresholds};

fn share(value: f64) -> SharedString {
    format!("{}%", fixed(value * 100., 1)).into()
}

#[derive(Default)]
struct Market {
    columns: Vec<SharedString>,
    segments: Vec<SharedString>,
    values: Vec<f64>,
    labelled: bool,
    gap: f32,
}

impl Market {
    fn value(&self, column: usize, segment: usize) -> f64 {
        self.values
            .get(column * self.segments.len() + segment)
            .copied()
            .filter(|value| value.is_finite() && *value > 0.)
            .unwrap_or(0.)
    }
}

fn arrange_mekko(mekko: &Market, frame: &Frame) -> Scene {
    let mut scene = Scene::default();
    let (left, bottom, top, right) = (40., 34., 8., 8.);
    let (width, height) = (
        (frame.width - left - right).max(1.),
        (frame.height - top - bottom).max(1.),
    );
    let shown: Vec<usize> = (0..mekko.segments.len())
        .filter(|segment| !frame.hidden.contains(segment))
        .collect();
    let totals: Vec<f64> = (0..mekko.columns.len())
        .map(|column| {
            shown
                .iter()
                .map(|segment| mekko.value(column, *segment))
                .sum()
        })
        .collect();
    let grand: f64 = totals.iter().sum();
    for step in 0..=4 {
        let y = top + height * (1. - step as f32 / 4.);
        scene.shapes.push(Shape::line(
            vec![point(left, y), point(left + width, y)],
            frame.ink.rule,
            1.,
            None,
        ));
        scene.label(
            format!("{}%", step * 25),
            point(left - 6., y - TEXT_SIZE / 2.),
            TextAlign::Right,
            TEXT_SIZE,
            frame.ink.muted,
        );
    }
    if grand <= 0. {
        return scene;
    }
    let usable = (width - mekko.gap * (mekko.columns.len().saturating_sub(1)) as f32).max(1.);
    let mut x = left;
    for (column, name) in mekko.columns.iter().enumerate() {
        let total = totals[column];
        let span = (total / grand) as f32 * usable;
        let mut y = top + height;
        for segment in &shown {
            let value = mekko.value(column, *segment);
            if value <= 0. {
                continue;
            }
            let tall = (value / total) as f32 * height;
            y -= tall;
            let color = frame.ink.colors.series(*segment);
            let tip = scene.note(Note {
                title: format!("{name} · {}", mekko.segments[*segment]).into(),
                accent: Some(color),
                rows: vec![
                    (None, "Value".into(), trimmed(value)),
                    (None, "Share of column".into(), share(value / total)),
                    (None, "Share of total".into(), share(value / grand)),
                ],
            });
            scene.shapes.push(
                Shape::filled(rect(x, y, span, tall), color, Some(tip))
                    .outlined(frame.ink.background, 1.),
            );
            let text = share(value / total);
            if mekko.labelled && span > text_width(&text, TEXT_SIZE) + 6. && tall > TEXT_SIZE + 4. {
                scene.label(
                    text,
                    point(x + span / 2., y + tall / 2. - TEXT_SIZE / 2.),
                    TextAlign::Center,
                    TEXT_SIZE,
                    frame.ink.background,
                );
            }
        }
        let below = top + height + 4.;
        if span > text_width(name, TEXT_SIZE) {
            scene.label(
                name.clone(),
                point(x + span / 2., below),
                TextAlign::Center,
                TEXT_SIZE,
                frame.ink.text,
            );
        }
        let part = share(total / grand);
        if span > text_width(&part, TEXT_SIZE) {
            scene.label(
                part,
                point(x + span / 2., below + TEXT_SIZE + 2.),
                TextAlign::Center,
                TEXT_SIZE,
                frame.ink.muted,
            );
        }
        x += span + mekko.gap;
    }
    scene
}

#[component]
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub fn mekko<
    T: 'static,
    D: Dataset<T>,
    N: Fn(&T) -> &str + 'static,
    V: Fn(&T, usize) -> f64 + 'static,
>(
    data: D,
    segments: Vec<SharedString>,
    name: N,
    value: V,
    show_values: Option<bool>,
    gap: Option<f32>,
    legend: Option<Legend>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
) -> AnyElement {
    let state = stage::<Market>();
    let (labelled, gap) = (show_values.unwrap_or(true), gap.unwrap_or(2.).max(0.));
    let key = data_key(&data, (&segments, labelled, gap.to_bits()));
    let groups = segments.clone();
    state.hold(
        key, || data.rows(|rows| {
            
            Market {
                columns: rows
                    .iter()
                    .map(|item| SharedString::from(name(item).to_owned()))
                    .collect(),
                values: rows
                    .iter()
                    .flat_map(|item| {
                        (0..segments.len())
                            .map(|segment| value(item, segment))
                            .collect::<Vec<_>>()
                    })
                    .collect(),
                segments,
                labelled,
                gap,
            }
        }));
    scene_view(
        state,
        arrange_mekko,
        Show {
            groups,
            legend,
            tooltip,
            clickable: false,
            zoom: Zoom::Off,
            class,
        })
}

#[derive(Default)]
struct Comb {
    names: Vec<SharedString>,
    values: Vec<f64>,
    thresholds: Option<Thresholds>,
    range: (f64, f64),
    decimals: Option<usize>,
    unit: SharedString,
    columns: Option<usize>,
}

impl Comb {
    fn text(&self, value: f64) -> SharedString {
        let number = self
            .decimals
            .map_or_else(|| trimmed(value), |decimals| fixed(value, decimals));
        format!("{number}{}", self.unit).into()
    }
}

fn fit(count: usize, width: f32, height: f32, fixed: Option<usize>) -> (usize, f32) {
    let root = 3f32.sqrt();
    let radius = |columns: usize| {
        let rows = count.div_ceil(columns);
        let across = if rows > 1 {
            columns as f32 + 0.5
        } else {
            columns as f32
        };
        (width / (root * across)).min(height / (1.5 * rows as f32 + 0.5))
    };
    fixed.map_or_else(
        || {
            (1..=count.max(1))
                .map(|columns| (columns, radius(columns)))
                .fold(
                    (1, 0.),
                    |best, next| if next.1 > best.1 { next } else { best },
                )
        },
        |columns| (columns.max(1), radius(columns.max(1))),
    )
}

fn arrange_comb(comb: &Comb, frame: &Frame) -> Scene {
    let mut scene = Scene::default();
    let count = comb.names.len();
    if count == 0 {
        return scene;
    }
    let (columns, radius) = fit(count, frame.width - 8., frame.height - 8., comb.columns);
    let rows = count.div_ceil(columns);
    let step = 3f32.sqrt() * radius;
    let across = if rows > 1 {
        columns as f32 + 0.5
    } else {
        columns as f32
    };
    let origin = point(
        (frame.width - step * across) / 2. + step / 2.,
        (frame.height - radius * (1.5 * rows as f32 + 0.5)) / 2. + radius,
    );
    let base = fallback(frame.ink.colors);
    let scale = steps(comb.thresholds.as_ref(), &base);
    let white = blenderui::white();
    for (index, (name, value)) in comb.names.iter().zip(&comb.values).enumerate() {
        let (row, column) = (index / columns, index % columns);
        let center = point(
            origin.x + step * (column as f32 + if row % 2 == 1 { 0.5 } else { 0. }),
            origin.y + 1.5 * radius * row as f32,
        );
        let color = if value.is_finite() {
            scale
                .color(*value, comb.range)
                .unwrap_or(frame.ink.colors.neutral)
        } else {
            frame.ink.colors.neutral.opacity(0.4)
        };
        let shown = comb.text(*value);
        let tip = scene.note(Note {
            title: name.clone(),
            accent: Some(color),
            rows: vec![(None, "Value".into(), shown.clone())],
        });
        scene.shapes.push(
            Shape::filled(hexagon(center, radius * 0.96), color, Some(tip))
                .outlined(frame.ink.background, 1.),
        );
        let big = (radius * 0.34).clamp(9., 26.);
        let small = (radius * 0.2).clamp(8., 14.);
        if radius > 22. && text_width(&shown, big) < step * 0.9 {
            scene.label(
                shown,
                point(center.x, center.y - big * 0.55),
                TextAlign::Center,
                big,
                white,
            );
            let mut label = name.to_string();
            while label.len() > 1 && text_width(&label, small) > step * 0.85 {
                label.pop();
            }
            if label.len() < name.len() {
                label.push('…');
            }
            scene.label(
                label,
                point(center.x, center.y + big * 0.55),
                TextAlign::Center,
                small,
                white.opacity(0.85),
            );
        }
    }
    scene
}

#[component]
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub fn honey_comb<
    T: 'static,
    D: Dataset<T>,
    N: Fn(&T) -> &str + 'static,
    V: Fn(&T) -> f64 + 'static,
>(
    data: D,
    name: N,
    value: V,
    thresholds: Option<Thresholds>,
    min: Option<f64>,
    max: Option<f64>,
    decimals: Option<usize>,
    unit: Option<SharedString>,
    columns: Option<usize>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
) -> AnyElement {
    let state = stage::<Comb>();
    let range = (min.unwrap_or(0.), max.unwrap_or(100.));
    let unit = unit.unwrap_or_default();
    let key = data_key(
        &data,
        (
            &thresholds,
            range.0.to_bits(),
            range.1.to_bits(),
            decimals,
            &unit,
            columns,
        ));
    state.hold(
        key, || data.rows(|rows| {
            
            Comb {
                names: rows
                    .iter()
                    .map(|item| SharedString::from(name(item).to_owned()))
                    .collect(),
                values: rows.iter().map(&value).collect(),
                thresholds,
                range,
                decimals,
                unit,
                columns,
            }
        }));
    scene_view(
        state,
        arrange_comb,
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
    fn honeycomb_prefers_the_largest_cells() {
        let (columns, radius) = fit(12, 600., 300., None);
        assert!(columns >= 4);
        assert!(radius > 40.);
        assert_eq!(fit(5, 100., 100., Some(2)).0, 2);
    }
}
