use blenderui::style::TextAlign;
use blenderui::{AnyElement, Hsla, SharedString, rgb};

use crate::component;
use crate::geom::point;

use super::scene::{
    Area, At, Frame, Note, Scene, Shape, Show, Zoom, rect, scene_view, stage, text_width,
};
use crate::derive::data_key;
use crate::format::{fixed, trimmed};
use crate::plot::{TEXT_SIZE, Tooltip};
use crate::{Dataset, Legend};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum LogoAlphabet {
    #[default]
    Auto,
    Dna,
    Rna,
    Protein,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum LogoUnits {
    #[default]
    Bits,
    Probability,
}

const DNA: &str = "ACGT";
const RNA: &str = "ACGU";
const PROTEIN: &str = "ACDEFGHIKLMNPQRSTVWY";

impl LogoAlphabet {
    fn resolve(self, sequences: &[String]) -> Self {
        if self != Self::Auto {
            return self;
        }
        let letters = || {
            sequences
                .iter()
                .flat_map(|sequence| sequence.chars())
                .filter(char::is_ascii_alphabetic)
        };
        if letters().all(|letter| DNA.contains(letter)) {
            Self::Dna
        } else if letters().all(|letter| RNA.contains(letter)) {
            Self::Rna
        } else {
            Self::Protein
        }
    }

    const fn letters(self) -> &'static str {
        match self {
            Self::Dna | Self::Auto => DNA,
            Self::Rna => RNA,
            Self::Protein => PROTEIN,
        }
    }
}

fn tint(letter: char, ink: Hsla) -> Hsla {
    let hex = match letter {
        'A' => 0x40a02b,
        'C' => 0x1e66f5,
        'G' => 0xdf8e1d,
        'T' | 'U' => 0xd20f39,
        _ => return ink,
    };
    rgb(hex)
}

fn residue(letter: char, ink: Hsla) -> Hsla {
    let hex = match letter {
        'S' | 'T' | 'Y' | 'N' | 'Q' => 0x40a02b,
        'K' | 'R' | 'H' => 0x1e66f5,
        'D' | 'E' => 0xd20f39,
        'G' | 'P' => 0x8839ef,
        'C' => 0xdf8e1d,
        _ => return ink,
    };
    rgb(hex)
}

fn band(from: f32, to: f32, inner: (f32, f32)) -> Vec<(f32, f32)> {
    let steps = 24;
    let arc = |(rx, ry): (f32, f32)| {
        (0..=steps).map(move |step| {
            let angle = (to - from)
                .mul_add(step as f32 / steps as f32, from)
                .to_radians();
            (
                rx.mul_add(angle.cos(), 0.5),
                (-ry).mul_add(angle.sin(), 0.5),
            )
        })
    };
    let mut points: Vec<(f32, f32)> = arc((0.5, 0.5)).collect();
    points.extend(arc(inner).collect::<Vec<_>>().into_iter().rev());
    points
}

type Piece = (Vec<(f32, f32)>, bool);
type Column = Vec<(char, f64, f64)>;

fn glyph(letter: char) -> Option<Vec<Piece>> {
    let hollow = (0.3, 0.32);
    Some(match letter {
        'A' => vec![
            (
                vec![
                    (0., 1.),
                    (0.38, 0.),
                    (0.62, 0.),
                    (1., 1.),
                    (0.79, 1.),
                    (0.69, 0.72),
                    (0.31, 0.72),
                    (0.21, 1.),
                ],
                false,
            ),
            (vec![(0.37, 0.56), (0.5, 0.2), (0.63, 0.56)], true),
        ],
        'T' => vec![(
            vec![
                (0., 0.),
                (1., 0.),
                (1., 0.17),
                (0.59, 0.17),
                (0.59, 1.),
                (0.41, 1.),
                (0.41, 0.17),
                (0., 0.17),
            ],
            false,
        )],
        'C' => vec![(band(42., 318., hollow), false)],
        'G' => vec![
            (band(42., 330., hollow), false),
            (
                vec![
                    (0.55, 0.5),
                    (1., 0.5),
                    (1., 0.86),
                    (0.83, 0.86),
                    (0.83, 0.64),
                    (0.55, 0.64),
                ],
                false,
            ),
        ],
        'U' => vec![
            (band(180., 360., hollow), false),
            (vec![(0., 0.), (0.2, 0.), (0.2, 0.5), (0., 0.5)], false),
            (vec![(0.8, 0.), (1., 0.), (1., 0.5), (0.8, 0.5)], false),
        ],
        _ => return None,
    })
}

#[derive(Default)]
struct Logo {
    alphabet: LogoAlphabet,
    units: LogoUnits,
    start: i64,
    columns: Vec<Column>,
    ceiling: f64,
}

fn profile(
    sequences: &[String],
    alphabet: LogoAlphabet,
    units: LogoUnits,
    correction: bool,
) -> (Vec<Column>, f64) {
    let letters: Vec<char> = alphabet.letters().chars().collect();
    let size = letters.len() as f64;
    let ceiling = if units == LogoUnits::Bits {
        size.log2()
    } else {
        1.
    };
    let length = sequences.iter().map(String::len).max().unwrap_or(0);
    let columns = (0..length)
        .map(|position| {
            let mut counts = vec![0f64; letters.len()];
            for sequence in sequences {
                if let Some(letter) = sequence
                    .as_bytes()
                    .get(position)
                    .map(|byte| byte.to_ascii_uppercase() as char)
                    && let Some(slot) = letters.iter().position(|known| *known == letter)
                {
                    counts[slot] += 1.;
                }
            }
            let total: f64 = counts.iter().sum();
            if total <= 0. {
                return Vec::new();
            }
            let entropy: f64 = counts
                .iter()
                .filter(|count| **count > 0.)
                .map(|count| {
                    let share = count / total;
                    -share * share.log2()
                })
                .sum();
            let small = if correction {
                (size - 1.) / (2. * std::f64::consts::LN_2 * total)
            } else {
                0.
            };
            let information = (ceiling - entropy - small).max(0.);
            let mut stack: Vec<(char, f64, f64)> = letters
                .iter()
                .zip(&counts)
                .filter(|(_, count)| **count > 0.)
                .map(|(letter, count)| {
                    let share = count / total;
                    (
                        *letter,
                        share,
                        if units == LogoUnits::Bits {
                            share * information
                        } else {
                            share
                        },
                    )
                })
                .collect();
            stack.sort_by(|a, b| a.2.total_cmp(&b.2));
            stack
        })
        .collect();
    (columns, ceiling)
}

fn position(start: i64, index: usize) -> i64 {
    start.saturating_add(i64::try_from(index).unwrap_or(i64::MAX))
}

fn arrange_logo(logo: &Logo, frame: &Frame) -> Scene {
    let mut scene = Scene::default();
    let area = Area::inset(frame.width, frame.height, (40., 10., 10., 24.));
    let count = logo.columns.len();
    if count == 0 || logo.ceiling <= 0. {
        return scene;
    }
    let ticks = if logo.units == LogoUnits::Bits {
        logo.ceiling.ceil() as usize
    } else {
        4
    };
    for tick in 0..=ticks {
        let value = logo.ceiling * tick as f64 / ticks as f64;
        let y = area.bottom - area.height() * tick as f32 / ticks as f32;
        scene.shapes.push(Shape::line(
            vec![point(area.left - 4., y), point(area.left, y)],
            frame.ink.muted,
            1.,
            None,
        ));
        scene.label(
            fixed(value, if logo.units == LogoUnits::Bits { 0 } else { 2 }),
            point(area.left - 6., y - TEXT_SIZE / 2.),
            TextAlign::Right,
            TEXT_SIZE,
            frame.ink.muted,
        );
    }
    scene.shapes.push(Shape::line(
        vec![
            point(area.left, area.top),
            point(area.left, area.bottom),
            point(area.right, area.bottom),
        ],
        frame.ink.muted,
        1.,
        None,
    ));
    let caption = if logo.units == LogoUnits::Bits {
        "bits"
    } else {
        "p"
    };
    scene.label(
        caption,
        point(4., area.top),
        TextAlign::Left,
        TEXT_SIZE,
        frame.ink.muted,
    );
    let width = area.width() / count as f32;
    let every = ((text_width("000", TEXT_SIZE) + 4.) / width).ceil().max(1.) as usize;
    let unit = area.height() / logo.ceiling as f32;
    for (index, column) in logo.columns.iter().enumerate() {
        let left = area.left + width * index as f32 + width * 0.05;
        let span = width * 0.9;
        let mut bottom = area.bottom;
        for (letter, share, height) in column {
            let tall = *height as f32 * unit;
            if tall < 0.5 {
                continue;
            }
            let top = bottom - tall;
            let color = if logo.alphabet == LogoAlphabet::Protein {
                residue(*letter, frame.ink.text)
            } else {
                tint(*letter, frame.ink.text)
            };
            let tip = scene.note(Note {
                title: format!("{letter} at {}", position(logo.start, index)).into(),
                accent: Some(color),
                rows: vec![
                    (
                        None,
                        "Frequency".into(),
                        format!("{}%", fixed(share * 100., 1)).into(),
                    ),
                    (
                        None,
                        "Height".into(),
                        format!("{} {caption}", fixed(*height, 2)).into(),
                    ),
                ],
            });
            let map =
                |(x, y): (f32, f32)| -> At { point(x.mul_add(span, left), y.mul_add(tall, top)) };
            if let Some(pieces) = glyph(*letter) {
                for (piece, hole) in pieces {
                    let points: Vec<At> = piece.into_iter().map(map).collect();
                    scene.shapes.push(Shape::filled(
                        points,
                        if hole { frame.ink.background } else { color },
                        Some(tip),
                    ));
                }
            } else {
                scene.shapes.push(Shape::filled(
                    rect(left, top, span, tall),
                    color.opacity(0.18),
                    Some(tip),
                ));
                let size = tall.min(span * 1.3) * 0.85;
                if size > 6. {
                    scene.label(
                        letter.to_string(),
                        point(left + span / 2., top + (tall - size) / 2.),
                        TextAlign::Center,
                        size,
                        color,
                    );
                }
            }
            bottom = top;
        }
        if index % every == 0 {
            scene.label(
                (position(logo.start, index)).to_string(),
                point(left + span / 2., area.bottom + 4.),
                TextAlign::Center,
                TEXT_SIZE,
                frame.ink.muted,
            );
        }
    }
    scene
}

#[component]
pub fn sequence_logo<T: 'static, D: Dataset<T>, S: Fn(&T) -> &str + 'static>(
    data: D,
    sequence: S,
    alphabet: Option<LogoAlphabet>,
    units: Option<LogoUnits>,
    start: Option<i64>,
    correction: Option<bool>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
) -> AnyElement {
    let state = stage::<Logo>();
    let (alphabet, units, start, correction) = (
        alphabet.unwrap_or_default(),
        units.unwrap_or_default(),
        start.unwrap_or(1),
        correction.unwrap_or(true),
    );
    let key = data_key(&data, (alphabet, units, start, correction));
    state.hold(
        key,
        || {
            let sequences: Vec<String> =
                data.rows(|rows| rows.iter().map(|item| sequence(item).to_ascii_uppercase()).collect());
            let alphabet = alphabet.resolve(&sequences);
            let (columns, ceiling) = profile(&sequences, alphabet, units, correction);
            Logo {
                alphabet,
                units,
                start,
                columns,
                ceiling,
            }
        });
    scene_view(
        state,
        arrange_logo,
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
struct Cloud {
    words: Vec<(SharedString, f64, usize)>,
    groups: Vec<SharedString>,
    sizes: (f32, f32),
}

fn overlaps(a: (f32, f32, f32, f32), b: &(f32, f32, f32, f32)) -> bool {
    a.0 < b.0 + b.2 && b.0 < a.0 + a.2 && a.1 < b.1 + b.3 && b.1 < a.1 + a.3
}

fn arrange_cloud(cloud: &Cloud, frame: &Frame) -> Scene {
    let mut scene = Scene::default();
    let shown: Vec<&(SharedString, f64, usize)> = cloud
        .words
        .iter()
        .filter(|word| !frame.hidden.contains(&word.2))
        .collect();
    let (low, high) = shown
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(low, high), word| {
            (low.min(word.1), high.max(word.1))
        });
    let (smallest, largest) = cloud.sizes;
    let size = |weight: f64| {
        let t = if high > low {
            ((weight - low) / (high - low)).sqrt() as f32
        } else {
            1.
        };
        (largest - smallest).mul_add(t, smallest)
    };
    let fit = shown.first().map_or(1., |word| {
        (frame.width * 0.85 / text_width(&word.0, size(word.1) * 1.08)).min(1.)
    });
    let center = point(frame.width / 2., frame.height / 2.);
    let aspect = (frame.width / frame.height.max(1.)).clamp(0.5, 3.);
    let mut placed: Vec<(f32, f32, f32, f32)> = Vec::with_capacity(shown.len());
    for (word, weight, group) in shown {
        let font = (size(*weight) * fit).max(8.);
        let (width, height) = (text_width(word, font) * 1.08 + 4., font * 1.15);
        let mut spot = None;
        let mut t = 0f32;
        while t < 400. {
            let radius = 2.2 * t;
            let x = (radius * t.cos()).mul_add(aspect, center.x) - width / 2.;
            let y = radius.mul_add(t.sin(), center.y) - height / 2.;
            if x < 0. || y < 0. || x + width > frame.width || y + height > frame.height {
                if radius > frame.width.max(frame.height) {
                    break;
                }
            } else if !placed
                .iter()
                .any(|other| overlaps((x, y, width, height), other))
            {
                spot = Some((x, y));
                break;
            }
            t += 0.12;
        }
        let Some((x, y)) = spot else {
            continue;
        };
        placed.push((x, y, width, height));
        let color = frame.ink.colors.series(*group);
        let tip = scene.note(Note {
            title: word.clone(),
            accent: Some(color),
            rows: vec![(None, "Weight".into(), trimmed(*weight))],
        });
        scene.shapes.push(Shape::filled(
            rect(x, y, width, height),
            color.opacity(0.),
            Some(tip),
        ));
        scene.label(
            word.clone(),
            point(x + width / 2., y + (height - font) / 2.),
            TextAlign::Center,
            font,
            color,
        );
    }
    scene
}

#[component]
pub fn word_cloud<
    T: 'static,
    D: Dataset<T>,
    W: Fn(&T) -> &str + 'static,
    V: Fn(&T) -> f64 + 'static,
    G: Fn(&T) -> &str + 'static,
>(
    data: D,
    word: W,
    weight: V,
    group: Option<G>,
    max_words: Option<usize>,
    font_range: Option<(f32, f32)>,
    legend: Option<Legend>,
    tooltip: Option<Tooltip>,
    class: Option<SharedString>,
) -> AnyElement {
    let state = stage::<Cloud>();
    let (limit, sizes) = (max_words.unwrap_or(120), font_range.unwrap_or((11., 52.)));
    let key = data_key(
        &data,
        (limit, sizes.0.to_bits(), sizes.1.to_bits(), group.is_some()));
    state.hold(
        key,
        || {
            let mut groups: Vec<SharedString> = Vec::new();
            let mut words: Vec<(SharedString, f64, usize)> = data.rows(|rows| rows
                .iter()
                .filter(|item| weight(item).is_finite() && weight(item) > 0.)
                .map(|item| {
                    let slot = group.as_ref().map_or(0, |group| {
                        let label = group(item);
                        groups
                            .iter()
                            .position(|known| known == label)
                            .unwrap_or_else(|| {
                                groups.push(SharedString::from(label.to_owned()));
                                groups.len() - 1
                            })
                    });
                    (
                        SharedString::from(word(item).to_owned()),
                        weight(item),
                        slot,
                    )
                })
                .collect());
            words.sort_by(|a, b| b.1.total_cmp(&a.1));
            words.truncate(limit);
            if group.is_none() {
                for (index, entry) in words.iter_mut().enumerate() {
                    entry.2 = index % 8;
                }
            }
            Cloud {
                words,
                groups,
                sizes,
            }
        });
    let groups = state.held.with(|held| held.model.groups.clone());
    scene_view(
        state,
        arrange_cloud,
        Show {
            groups,
            legend,
            tooltip,
            clickable: false,
            zoom: Zoom::Wheel,
            class,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conserved_columns_carry_two_bits() {
        let sequences: Vec<String> = ["ACGT", "ACGA", "ACTT", "AGGC"]
            .iter()
            .map(|text| (*text).to_owned())
            .collect();
        let (columns, ceiling) = profile(&sequences, LogoAlphabet::Dna, LogoUnits::Bits, false);
        assert!((ceiling - 2.).abs() < 1e-9);
        assert!((columns[0][0].2 - 2.).abs() < 1e-9);
        assert!(columns[3].iter().map(|entry| entry.2).sum::<f64>() < 1.);
        assert_eq!(LogoAlphabet::Auto.resolve(&sequences), LogoAlphabet::Dna);
        assert!(overlaps((0., 0., 10., 10.), &(5., 5., 10., 10.)));
    }
}
