use std::hash::{DefaultHasher, Hash, Hasher};

use blenderui::{Hsla, SharedString};

use super::grid::Grid;
use super::options::{BucketLayout, Buckets, Calculation, HeatmapColor, OpacityScale};
use crate::data::{Accessor, Revision};
use crate::format::{fixed, trimmed};
use crate::plot::{AxisConfig, Scale, SelectionMode, VisibilityMode, Zoom};
use crate::theme::ChartColors;

pub(crate) struct Format(pub Option<Box<dyn Fn(f64) -> SharedString>>);

pub(crate) enum Source<T> {
    Samples {
        x: Accessor<T>,
        y: Accessor<T>,
        calc: Calculation,
    },
    Cells {
        x: Accessor<T>,
        y: Accessor<T>,
        value: Accessor<T>,
        x_end: Option<Accessor<T>>,
        y_end: Option<Accessor<T>>,
    },
    Buckets {
        x: Accessor<T>,
        names: Vec<SharedString>,
        fields: Vec<usize>,
        read: fn(&T, usize) -> f64,
        layout: BucketLayout,
        numeric: bool,
    },
}

impl<T> Source<T> {
    fn fingerprint<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Samples { x, y, calc } => {
                0u8.hash(state);
                x.fingerprint(state);
                y.fingerprint(state);
                for buckets in [calc.x_buckets, calc.y_buckets] {
                    match buckets {
                        Buckets::Auto => 0u64.hash(state),
                        Buckets::Count(count) => (1u64, count).hash(state),
                        Buckets::Size(size) => (2u64, size.to_bits()).hash(state),
                    }
                }
                calc.y_scale.hash(state);
            }
            Self::Cells {
                x,
                y,
                value,
                x_end,
                y_end,
            } => {
                1u8.hash(state);
                for accessor in [
                    Some(x),
                    Some(y),
                    Some(value),
                    x_end.as_ref(),
                    y_end.as_ref(),
                ] {
                    accessor.is_some().hash(state);
                    if let Some(accessor) = accessor {
                        accessor.fingerprint(state);
                    }
                }
            }
            Self::Buckets {
                x,
                names,
                fields,
                layout,
                numeric,
                ..
            } => {
                2u8.hash(state);
                x.fingerprint(state);
                (names, fields, layout, numeric).hash(state);
            }
        }
    }
}

fn color_key(color: &HeatmapColor, dark: bool, theme: (u64, bool)) -> u64 {
    let mut state = DefaultHasher::new();
    (dark, theme).hash(&mut state);
    match color {
        HeatmapColor::Scheme {
            scheme,
            steps,
            reverse,
        } => (0u8, scheme, steps, reverse).hash(&mut state),
        HeatmapColor::Theme {
            scheme,
            steps,
            reverse,
        } => (1u8, scheme, steps, reverse).hash(&mut state),
        HeatmapColor::Opacity { fill, scale, steps } => {
            let exponent = match scale {
                OpacityScale::Linear => None,
                OpacityScale::Exponential(exponent) => Some(exponent.to_bits()),
            };
            (
                2u8,
                [fill.r, fill.g, fill.b, fill.a].map(f32::to_bits),
                exponent,
                steps,
            )
                .hash(&mut state);
        }
    }
    state.finish()
}

#[derive(Default)]
pub(crate) struct Cache {
    key: Option<u64>,
    palette_key: Option<u64>,
    pub grid: Grid,
    pub palette: Vec<Hsla>,
    columns: [Vec<f64>; 5],
}

impl Cache {
    pub(crate) fn refresh<T>(
        &mut self,
        revision: Option<Revision>,
        rows: &[T],
        source: &Source<T>,
        x_time: bool,
    ) {
        let key = revision.map(|revision| {
            let mut state = DefaultHasher::new();
            (revision, x_time).hash(&mut state);
            source.fingerprint(&mut state);
            state.finish()
        });
        if key.is_some() && key == self.key {
            return;
        }
        self.key = key;
        let [xs, ys, values, x_ends, y_ends] = &mut self.columns;
        match source {
            Source::Samples { x, y, calc } => {
                x.fill(rows, xs);
                y.fill(rows, ys);
                self.grid.samples(xs, ys, *calc, x_time);
            }
            Source::Cells {
                x,
                y,
                value,
                x_end,
                y_end,
            } => {
                x.fill(rows, xs);
                y.fill(rows, ys);
                value.fill(rows, values);
                if let Some(end) = x_end {
                    end.fill(rows, x_ends);
                }
                if let Some(end) = y_end {
                    end.fill(rows, y_ends);
                }
                self.grid.cells(
                    xs,
                    ys,
                    values,
                    x_end.as_ref().map(|_| x_ends.as_slice()),
                    y_end.as_ref().map(|_| y_ends.as_slice()),
                );
            }
            Source::Buckets {
                x,
                names,
                fields,
                read,
                layout,
                numeric,
            } => {
                x.fill(rows, xs);
                let read = |row: usize, field: usize| read(&rows[row], fields[field]);
                self.grid.buckets(xs, names, read, *layout, *numeric);
            }
        }
    }

    pub(crate) fn paint(
        &mut self,
        color: &HeatmapColor,
        dark: bool,
        theme: (u64, bool),
        colors: &ChartColors,
    ) {
        let key = color_key(color, dark, theme);
        if self.palette_key == Some(key) {
            return;
        }
        self.palette_key = Some(key);
        match *color {
            HeatmapColor::Scheme {
                scheme,
                steps,
                reverse,
            } => {
                scheme.quantize(steps, dark, reverse, &mut self.palette);
            }
            HeatmapColor::Theme {
                ref scheme,
                steps,
                reverse,
            } => {
                self.palette.clear();
                let last = steps.max(2) - 1;
                self.palette.extend((0..=last).map(|index| {
                    scheme
                        .sample(colors, index as f32 / last as f32)
                        .unwrap_or(colors.fallback)
                }));
                if reverse {
                    self.palette.reverse();
                }
            }
            HeatmapColor::Opacity { fill, scale, steps } => {
                self.palette.clear();
                let last = steps.max(2) - 1;
                self.palette.extend((0..=last).map(|index| {
                    let t = index as f32 / last as f32;
                    let alpha = match scale {
                        OpacityScale::Linear => t,
                        OpacityScale::Exponential(exponent) => t.powf(exponent),
                    };
                    fill.alpha(alpha)
                }));
            }
        }
    }
}

pub(crate) struct Model {
    pub x: Scale,
    pub y: Scale,
    pub x_axis: AxisConfig,
    pub y_axis: AxisConfig,
    pub color_range: (f64, f64),
    pub hide_le: Option<f64>,
    pub hide_ge: Option<f64>,
    pub gap: f32,
    pub radius: f32,
    pub show_value: VisibilityMode,
    pub decimals: Option<usize>,
    pub exemplars: Vec<(f64, f64)>,
    pub exemplar_color: Hsla,
    pub selection: SelectionMode,
}

impl Model {
    pub(crate) fn hidden(&self, value: f64) -> bool {
        !value.is_finite()
            || self.hide_le.is_some_and(|limit| value <= limit)
            || self.hide_ge.is_some_and(|limit| value >= limit)
    }

    pub(crate) fn tone(&self, value: f64, palette: &[Hsla]) -> Option<Hsla> {
        let (low, high) = self.color_range;
        let span = high - low;
        let t = if span > 0. {
            ((value - low) / span).clamp(0., 1.)
        } else {
            1.
        };
        let last = palette.len().checked_sub(1)?;
        palette.get((t * last as f64).round() as usize).copied()
    }

    pub(crate) fn cell_text(&self, value: f64, format: &Format) -> SharedString {
        if let Some(format) = &format.0 {
            return format(value);
        }
        self.decimals
            .map_or_else(|| trimmed(value), |decimals| fixed(value, decimals))
    }

    pub(crate) fn scales(
        grid: &Grid,
        x_axis: &AxisConfig,
        y_axis: &AxisConfig,
        zoom: Zoom,
    ) -> (Scale, Scale) {
        let mut x_axis = x_axis.clone();
        let mut y_axis = y_axis.clone();
        if let Some((low, high)) = zoom.x {
            x_axis.min = Some(low);
            x_axis.max = Some(high);
        }
        if let Some((low, high)) = zoom.y {
            y_axis.min = Some(low);
            y_axis.max = Some(high);
        }
        if grid.layout.is_none() {
            if let Some(base) = grid.log {
                y_axis.scale = crate::plot::ScaleKind::Log { base };
            }
        } else {
            y_axis.scale = crate::plot::ScaleKind::Linear;
        }
        let span = |edges: &[f64]| Some((*edges.first()?, *edges.last()?));
        (
            Scale::exact(&x_axis, span(&grid.x_edges)),
            Scale::exact(&y_axis, span(&grid.y_edges)),
        )
    }
}
