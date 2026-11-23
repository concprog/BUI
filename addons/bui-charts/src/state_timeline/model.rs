use std::hash::{DefaultHasher, Hash, Hasher};

use blenderui::{Hsla, SharedString};

use super::options::{StateLane, Style};
use crate::bar_gauge::GaugeColor;
use crate::data::{Accessor, Revision};
use crate::format::trimmed;
use crate::mappings::{StateValue, ValueMappings};
use crate::theme::ChartColors;
use crate::thresholds::{Steps, ThresholdsMode, fallback, steps};
use crate::{Scheme, Thresholds};

const GRADIENT_SAMPLES: usize = 16;

#[derive(Clone, Debug, PartialEq)]
enum Key {
    Mapping(usize),
    Step(usize),
    Value(StateValue),
}

pub(crate) struct Segment {
    pub start: f64,
    pub end: f64,
    pub color: Hsla,
    pub text: SharedString,
}

pub(crate) struct Lane {
    pub name: SharedString,
    pub segments: Vec<Segment>,
    pub fill_opacity: f32,
    pub line_width: f32,
    pub tooltip: bool,
}

impl Lane {
    pub(crate) fn at(&self, x: f64) -> Option<&Segment> {
        let index = self
            .segments
            .partition_point(|segment| segment.start <= x)
            .checked_sub(1)?;
        self.segments.get(index).filter(|segment| x < segment.end)
    }
}

pub(crate) enum LegendItems {
    None,
    List(Vec<(SharedString, Hsla)>),
    Gradient(Vec<Hsla>, SharedString, SharedString),
}

#[derive(Default)]
pub(crate) struct Cache {
    key: Option<u64>,
    pub xs: Vec<f64>,
    pub lanes: Vec<Lane>,
    pub legend: Option<LegendItems>,
    pub range: (f64, f64),
}

pub(crate) struct Defaults<'a> {
    pub mappings: Option<&'static ValueMappings>,
    pub color: &'a GaugeColor,
    pub thresholds: Option<&'a Thresholds>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub merge: bool,
    pub samples: bool,
    pub end: Option<f64>,
}

struct Resolved {
    key: Key,
    text: SharedString,
    color: Hsla,
}

fn step_label(steps: Steps<'_>, index: usize) -> SharedString {
    let unit = if steps.mode == ThresholdsMode::Percentage {
        "%"
    } else {
        ""
    };
    let value = |index: usize| trimmed(steps.steps[index].0);
    if index == 0 {
        steps.steps.get(1).map_or_else(
            || SharedString::Borrowed("Base"),
            |_| format!("< {}{unit}", value(1)).into(),
        )
    } else {
        format!("{}{unit}+", value(index)).into()
    }
}

fn step_index(steps: Steps<'_>, value: f64, range: (f64, f64)) -> usize {
    steps
        .steps
        .iter()
        .enumerate()
        .skip(1)
        .take_while(|(_, (step, _))| value >= steps.value(*step, range))
        .last()
        .map_or(0, |(index, _)| index)
}

struct Resolver<'a> {
    mappings: Option<&'static ValueMappings>,
    color: &'a GaugeColor,
    steps: Steps<'a>,
    range: (f64, f64),
    lane: usize,
    merge: bool,
    colors: &'a ChartColors,
    distinct: &'a mut Vec<SharedString>,
}

impl Resolver<'_> {
    fn resolve(&mut self, value: &StateValue) -> Option<Resolved> {
        if let Some(mapped) = self.mappings.and_then(|mappings| mappings.find(value)) {
            return Some(Resolved {
                key: Key::Mapping(mapped.index),
                text: mapped.text,
                color: mapped.color,
            });
        }
        if value.is_null() {
            return None;
        }
        let Some(number) = value.number() else {
            let text = value.text();
            let index = self
                .distinct
                .iter()
                .position(|seen| *seen == text)
                .unwrap_or_else(|| {
                    self.distinct.push(text.clone());
                    self.distinct.len() - 1
                });
            let color = match self.color {
                GaugeColor::Fixed(color) => *color,
                _ => self.colors.series(index),
            };
            return Some(Resolved {
                key: Key::Value(value.clone()),
                text,
                color,
            });
        };
        Some(match self.color {
            GaugeColor::Thresholds => {
                let index = step_index(self.steps, number, self.range);
                let color = self
                    .steps
                    .steps
                    .get(index)
                    .map_or(self.colors.fallback, |(_, color)| *color);
                if self.merge {
                    Resolved {
                        key: Key::Step(index),
                        text: step_label(self.steps, index),
                        color,
                    }
                } else {
                    Resolved {
                        key: Key::Value(value.clone()),
                        text: trimmed(number),
                        color,
                    }
                }
            }
            GaugeColor::Scheme(scheme) => Resolved {
                key: Key::Value(value.clone()),
                text: trimmed(number),
                color: sample(scheme, number, self.range, self.colors),
            },
            GaugeColor::Palette => Resolved {
                key: Key::Value(value.clone()),
                text: trimmed(number),
                color: self.colors.series(self.lane),
            },
            GaugeColor::Fixed(color) => Resolved {
                key: Key::Value(value.clone()),
                text: trimmed(number),
                color: *color,
            },
        })
    }
}

fn sample(scheme: &Scheme, value: f64, (low, high): (f64, f64), colors: &ChartColors) -> Hsla {
    let t = if high > low {
        ((value - low) / (high - low)).clamp(0., 1.)
    } else {
        0.5
    };
    scheme.sample(colors, t as f32).unwrap_or(colors.fallback)
}

pub(crate) fn fingerprint<T>(
    revision: Revision,
    theme: (u64, bool),
    x: &Accessor<T>,
    lanes: &[StateLane<T>],
    defaults: &Defaults<'_>,
) -> u64 {
    let mut state = DefaultHasher::new();
    (
        revision,
        theme,
        lanes.len(),
        defaults.merge,
        defaults.samples,
        defaults.min.map(f64::to_bits),
        defaults.max.map(f64::to_bits),
    )
        .hash(&mut state);
    (
        defaults.end.map(f64::to_bits),
        defaults.mappings.map(std::ptr::from_ref),
    )
        .hash(&mut state);
    defaults.color.hash(&mut state);
    defaults.thresholds.hash(&mut state);
    x.fingerprint(&mut state);
    for lane in lanes {
        lane.value.fingerprint(&mut state);
        (
            &lane.name,
            lane.mappings.map(std::ptr::from_ref),
            &lane.color,
        )
            .hash(&mut state);
        lane.thresholds.hash(&mut state);
        let style = &lane.style;
        (
            style.fill_opacity().to_bits(),
            style.line_width().to_bits(),
            style.insert_nulls().to_bits(),
            style.hide_from(),
        )
            .hash(&mut state);
        style.span_nulls().hash(&mut state);
    }
    state.finish()
}

impl Cache {
    pub(crate) fn fresh(&self, key: Option<u64>) -> bool {
        key.is_some() && self.key == key
    }

    #[allow(clippy::too_many_lines)]
    pub(crate) fn refresh<T>(
        &mut self,
        key: Option<u64>,
        rows: &[T],
        x: &Accessor<T>,
        lanes: &[StateLane<T>],
        defaults: &Defaults<'_>,
        colors: &ChartColors,
    ) {
        self.key = key;
        x.fill(rows, &mut self.xs);
        let xs = &self.xs;
        let finite = || xs.iter().copied().filter(|x| x.is_finite());
        let first = finite().reduce(f64::min).unwrap_or(0.);
        let last = finite().reduce(f64::max).unwrap_or(1.);
        let mut gaps: Vec<f64> = xs
            .windows(2)
            .map(|pair| pair[1] - pair[0])
            .filter(|gap| *gap > 0.)
            .collect();
        gaps.sort_by(f64::total_cmp);
        let typical = gaps.get(gaps.len() / 2).copied().unwrap_or(1.);
        let half = if defaults.samples { typical / 2. } else { 0. };
        let end = defaults.end.unwrap_or(last + typical - half);
        self.range = (first - half, end);
        let fallback_steps = fallback(colors);
        let mut distinct: Vec<SharedString> = Vec::new();
        let mut values: Vec<StateValue> = Vec::with_capacity(rows.len());
        self.lanes.clear();
        let mut used_mappings: Vec<&'static ValueMappings> = Vec::new();
        let mut used_steps: Option<Steps<'_>> = None;
        let mut used_scheme: Option<(&Scheme, (f64, f64))> = None;
        let mut any_text = false;
        for (lane_index, lane) in lanes.iter().enumerate() {
            let style = &lane.style;
            if style.hide_from().viz {
                continue;
            }
            values.clear();
            values.extend(rows.iter().map(|row| lane.value.read(row)));
            let numbers = || values.iter().filter_map(StateValue::number);
            let range = (
                defaults
                    .min
                    .or_else(|| numbers().reduce(f64::min))
                    .unwrap_or(0.),
                defaults
                    .max
                    .or_else(|| numbers().reduce(f64::max))
                    .unwrap_or(1.),
            );
            let color = lane.color.as_ref().unwrap_or(defaults.color);
            let mappings = lane.mappings.or(defaults.mappings);
            let lane_steps = steps(
                lane.thresholds.as_ref().or(defaults.thresholds),
                &fallback_steps,
            );
            if let Some(mappings) = mappings
                && !used_mappings
                    .iter()
                    .any(|seen| std::ptr::eq(*seen, mappings))
            {
                used_mappings.push(mappings);
            }
            match color {
                GaugeColor::Thresholds => used_steps = used_steps.or(Some(lane_steps)),
                GaugeColor::Scheme(scheme) => used_scheme = used_scheme.or(Some((scheme, range))),
                _ => {}
            }
            let mut resolver = Resolver {
                mappings,
                color,
                steps: lane_steps,
                range,
                lane: lane_index,
                merge: defaults.merge,
                colors,
                distinct: &mut distinct,
            };
            let resolved: Vec<Option<Resolved>> =
                values.iter().map(|value| resolver.resolve(value)).collect();
            any_text |= values
                .iter()
                .any(|value| matches!(value, StateValue::Text(_) | StateValue::Bool(_)));
            let segments = if defaults.samples {
                sampled(xs, &resolved, half)
            } else {
                merged(xs, &resolved, style, defaults.merge, end)
            };
            self.lanes.push(Lane {
                name: lane
                    .name
                    .clone()
                    .unwrap_or_else(|| format!("Series {}", lane_index + 1).into()),
                segments,
                fill_opacity: style.fill_opacity(),
                line_width: style.line_width(),
                tooltip: !style.hide_from().tooltip,
            });
        }
        self.legend = Some(if !used_mappings.is_empty() {
            let mut items: Vec<(SharedString, Hsla)> = Vec::new();
            for mappings in used_mappings {
                for index in 0..mappings.len() {
                    let item = mappings.entry(index);
                    if !items.iter().any(|seen| seen.0 == item.0) {
                        items.push(item);
                    }
                }
            }
            LegendItems::List(items)
        } else if let Some(steps) = used_steps {
            LegendItems::List(
                (0..steps.steps.len())
                    .map(|index| (step_label(steps, index), steps.steps[index].1))
                    .collect(),
            )
        } else if any_text {
            LegendItems::List(
                distinct
                    .iter()
                    .enumerate()
                    .map(|(index, text)| (text.clone(), colors.series(index)))
                    .collect(),
            )
        } else if let Some((scheme, range)) = used_scheme {
            let last = (GRADIENT_SAMPLES - 1) as f32;
            LegendItems::Gradient(
                (0..GRADIENT_SAMPLES)
                    .map(|index| {
                        scheme
                            .sample(colors, index as f32 / last)
                            .unwrap_or(colors.fallback)
                    })
                    .collect(),
                trimmed(range.0),
                trimmed(range.1),
            )
        } else {
            LegendItems::None
        });
    }
}

fn sampled(xs: &[f64], resolved: &[Option<Resolved>], half: f64) -> Vec<Segment> {
    xs.iter()
        .zip(resolved)
        .filter(|(at, _)| at.is_finite())
        .filter_map(|(&at, state)| {
            state.as_ref().map(|state| Segment {
                start: at - half,
                end: at + half,
                color: state.color,
                text: state.text.clone(),
            })
        })
        .collect()
}

fn merged(
    xs: &[f64],
    resolved: &[Option<Resolved>],
    style: &Style,
    merge: bool,
    end: f64,
) -> Vec<Segment> {
    let mut segments: Vec<Segment> = Vec::new();
    let mut open: Option<(usize, f64)> = None;
    let mut previous: Option<f64> = None;
    let close = |segments: &mut Vec<Segment>, (row, start): (usize, f64), end: f64| {
        if let Some(state) = &resolved[row]
            && end > start
        {
            segments.push(Segment {
                start,
                end,
                color: state.color,
                text: state.text.clone(),
            });
        }
    };
    for (index, &at) in xs.iter().enumerate() {
        if !at.is_finite() {
            continue;
        }
        if let (Some(state), Some(last_x)) = (open, previous)
            && at - last_x > style.insert_nulls()
        {
            close(&mut segments, state, last_x + style.insert_nulls());
            open = None;
        }
        match &resolved[index] {
            None => {
                if let Some(state) = open {
                    let next = (index + 1..xs.len()).find(|later| resolved[*later].is_some());
                    let gap =
                        next.map_or(f64::INFINITY, |later| xs[later] - previous.unwrap_or(at));
                    if !style.span_nulls().bridges(gap) {
                        close(&mut segments, state, at);
                        open = None;
                    }
                }
            }
            Some(current) => {
                let same = open.is_some_and(|(row, _)| {
                    merge
                        && resolved[row]
                            .as_ref()
                            .is_some_and(|state| state.key == current.key)
                });
                if !same {
                    if let Some(state) = open {
                        close(&mut segments, state, at);
                    }
                    open = Some((index, at));
                }
                previous = Some(at);
            }
        }
    }
    if let Some(state) = open {
        close(&mut segments, state, end);
    }
    segments
}

#[cfg(test)]
mod tests {
    use blenderui::{green, red};

    use super::*;
    use crate::state_timeline::options::{Reader, Style};
    use crate::trend::NullSpan;

    fn lane(span_nulls: NullSpan) -> StateLane<[f64; 2]> {
        StateLane {
            name: None,
            value: Reader::Field(|row, field| row[field], 1),
            mappings: None,
            color: None,
            thresholds: None,
            style: Style {
                span_nulls: Some(span_nulls),
                ..Style::default()
            },
        }
    }

    fn segments(
        rows: &[[f64; 2]],
        span_nulls: NullSpan,
        merge: bool,
        color: &GaugeColor,
        mappings: Option<&'static ValueMappings>,
    ) -> Vec<(f64, f64, SharedString)> {
        let defaults = Defaults {
            mappings,
            color,
            thresholds: None,
            min: None,
            max: None,
            merge,
            samples: false,
            end: Some(10.),
        };
        let mut cache = Cache::default();
        let x = Accessor::Field(|row: &[f64; 2], field| row[field], 0);
        cache.refresh(
            None,
            rows,
            &x,
            &[lane(span_nulls)],
            &defaults,
            crate::theme::bundled(),
        );
        cache.lanes[0]
            .segments
            .iter()
            .map(|segment| (segment.start, segment.end, segment.text.clone()))
            .collect()
    }

    const ROWS: [[f64; 2]; 5] = [[0., 1.], [2., 1.], [4., f64::NAN], [6., 2.], [8., 2.]];

    #[test]
    fn merges_equal_neighbours_and_breaks_at_nulls() {
        let fixed = GaugeColor::Fixed(green());
        assert_eq!(
            segments(&ROWS, NullSpan::Never, true, &fixed, None),
            [(0., 4., "1".into()), (6., 10., "2".into())]
        );
        assert_eq!(
            segments(&ROWS, NullSpan::Always, true, &fixed, None),
            [(0., 6., "1".into()), (6., 10., "2".into())]
        );
        assert_eq!(
            segments(&ROWS, NullSpan::Never, false, &fixed, None).len(),
            4
        );
    }

    #[test]
    fn mappings_and_threshold_steps_name_the_states() {
        static MAPPINGS: std::sync::LazyLock<ValueMappings> = std::sync::LazyLock::new(|| {
            ValueMappings::new()
                .value("1", "Up", green())
                .value("2", "Down", red())
        });
        let texts: Vec<SharedString> = segments(
            &ROWS,
            NullSpan::Never,
            true,
            &GaugeColor::Fixed(green()),
            Some(&MAPPINGS),
        )
        .into_iter()
        .map(|(_, _, text)| text)
        .collect();
        assert_eq!(texts, ["Up", "Down"]);
        let steps = segments(
            &[[0., 10.], [1., 20.], [2., 90.]],
            NullSpan::Never,
            true,
            &GaugeColor::Thresholds,
            None,
        );
        assert_eq!(steps, [(0., 2., "< 80".into()), (2., 10., "80+".into())]);
    }
}
