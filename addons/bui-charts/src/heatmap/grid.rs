use blenderui::SharedString;

use super::options::{BucketLayout, BucketScale, Buckets, Calculation};
use crate::time;

const NICE: [f64; 4] = [1., 2., 2.5, 5.];
const X_BUCKETS: usize = 20;
const Y_BUCKETS: usize = 10;

#[derive(Default)]
pub(crate) struct Grid {
    pub x_edges: Vec<f64>,
    pub y_edges: Vec<f64>,
    pub labels: Vec<SharedString>,
    pub layout: Option<BucketLayout>,
    pub values: Vec<f64>,
    pub log: Option<f64>,
}

impl Grid {
    pub(crate) const fn columns(&self) -> usize {
        self.x_edges.len().saturating_sub(1)
    }

    pub(crate) const fn rows(&self) -> usize {
        self.y_edges.len().saturating_sub(1)
    }

    pub(crate) fn value(&self, column: usize, row: usize) -> f64 {
        self.values
            .get(column * self.rows() + row)
            .copied()
            .unwrap_or(f64::NAN)
    }

    pub(crate) fn column(&self, column: usize) -> &[f64] {
        let rows = self.rows();
        self.values
            .get(column * rows..(column + 1) * rows)
            .unwrap_or(&[])
    }

    pub(crate) fn range(&self, hidden: impl Fn(f64) -> bool) -> Option<(f64, f64)> {
        self.values
            .iter()
            .filter(|value| value.is_finite() && !hidden(**value))
            .fold(None, |range, value| match range {
                None => Some((*value, *value)),
                Some((low, high)) => Some((low.min(*value), high.max(*value))),
            })
    }

    fn clear(&mut self) {
        self.x_edges.clear();
        self.y_edges.clear();
        self.labels.clear();
        self.values.clear();
        self.layout = None;
        self.log = None;
    }
}

fn nice_incr(approx: f64, time: bool) -> f64 {
    if time {
        return time::STEPS
            .iter()
            .rposition(|step| *step <= approx)
            .map_or(time::STEPS[0], |index| time::STEPS[index]);
    }
    if !(approx.is_finite() && approx > 0.) {
        return 1.;
    }
    let magnitude = 10f64.powf(approx.log10().floor());
    let mut best = magnitude * NICE[0] / 10.;
    for scale in [magnitude / 10., magnitude, magnitude * 10.] {
        for nice in NICE {
            let incr = nice * scale;
            if incr <= approx * (1. + 1e-12) {
                best = best.max(incr);
            }
        }
    }
    best
}

fn increment(buckets: Buckets, default: usize, low: f64, high: f64, time: bool) -> f64 {
    match buckets {
        Buckets::Size(size) if size.is_finite() && size > 0. => size,
        Buckets::Count(count) => nice_incr((high - low) / (count.max(2) - 1) as f64, time),
        _ => nice_incr((high - low) / (default - 1) as f64, time),
    }
}

fn round_down(value: f64, incr: f64) -> f64 {
    (value / incr).floor() * incr
}

fn edges(starts: &mut Vec<f64>, out: &mut Vec<f64>, ends: Option<&[f64]>) {
    starts.sort_by(f64::total_cmp);
    starts.dedup_by(|a, b| (*a - *b).abs() <= f64::EPSILON * b.abs().max(1.));
    let step = starts
        .windows(2)
        .map(|pair| pair[1] - pair[0])
        .fold(f64::INFINITY, f64::min);
    let step = if step.is_finite() { step } else { 1. };
    out.clear();
    out.extend_from_slice(starts);
    let last = starts.last().copied().unwrap_or(0.);
    let end = ends
        .and_then(|ends| {
            ends.iter()
                .copied()
                .filter(|end| end.is_finite())
                .reduce(f64::max)
        })
        .filter(|end| *end > last)
        .unwrap_or(last + step);
    out.push(end);
}

fn index(edges: &[f64], value: f64) -> Option<usize> {
    let position = edges.partition_point(|edge| *edge <= value);
    position
        .checked_sub(1)
        .filter(|index| *index + 1 < edges.len())
}

impl Grid {
    pub(crate) fn samples(&mut self, xs: &[f64], ys: &[f64], calc: Calculation, x_time: bool) {
        self.clear();
        let base = match calc.y_scale {
            BucketScale::Linear => None,
            BucketScale::Log2 => Some(2f64),
            BucketScale::Log10 => Some(10f64),
        };
        let usable = |x: f64, y: f64| x.is_finite() && y.is_finite() && base.is_none_or(|_| y > 0.);
        let pairs = || xs.iter().zip(ys).filter(|(x, y)| usable(**x, **y));
        let Some(((min_x, max_x), (min_y, max_y))) =
            pairs().fold(None::<((f64, f64), (f64, f64))>, |range, (x, y)| {
                Some(match range {
                    None => ((*x, *x), (*y, *y)),
                    Some(((lx, hx), (ly, hy))) => {
                        ((lx.min(*x), hx.max(*x)), (ly.min(*y), hy.max(*y)))
                    }
                })
            })
        else {
            return;
        };
        let x_incr = increment(calc.x_buckets, X_BUCKETS, min_x, max_x, x_time);
        let y_incr = match (base, calc.y_buckets) {
            (Some(_), Buckets::Size(split)) if split > 0. => 1. / split,
            (Some(_), _) => 1.,
            (None, buckets) => increment(buckets, Y_BUCKETS, min_y, max_y, false),
        };
        let to_bin = move |value: f64| {
            base.map_or_else(
                || round_down(value, y_incr),
                |base| round_down(value.log(base), y_incr),
            )
        };
        let (first_x, last_x) = (round_down(min_x, x_incr), round_down(max_x, x_incr));
        let (first_y, last_y) = (to_bin(min_y), to_bin(max_y));
        let columns = ((last_x - first_x) / x_incr).round() as usize + 1;
        let rows = ((last_y - first_y) / y_incr).round() as usize + 1;
        self.values.resize(columns * rows, 0.);
        for (x, y) in pairs() {
            let column = ((round_down(*x, x_incr) - first_x) / x_incr).round() as usize;
            let row = ((to_bin(*y) - first_y) / y_incr).round() as usize;
            if let Some(cell) = self
                .values
                .get_mut(column.min(columns - 1) * rows + row.min(rows - 1))
            {
                *cell += 1.;
            }
        }
        self.x_edges
            .extend((0..=columns).map(|index| (index as f64).mul_add(x_incr, first_x)));
        self.y_edges.extend((0..=rows).map(|index| {
            let edge = (index as f64).mul_add(y_incr, first_y);
            base.map_or(edge, |base| base.powf(edge))
        }));
        self.log = base;
    }

    pub(crate) fn cells(
        &mut self,
        xs: &[f64],
        ys: &[f64],
        values: &[f64],
        x_ends: Option<&[f64]>,
        y_ends: Option<&[f64]>,
    ) {
        self.clear();
        let mut starts: Vec<f64> = xs.iter().copied().filter(|x| x.is_finite()).collect();
        if starts.is_empty() {
            return;
        }
        edges(&mut starts, &mut self.x_edges, x_ends);
        starts.clear();
        starts.extend(ys.iter().copied().filter(|y| y.is_finite()));
        edges(&mut starts, &mut self.y_edges, y_ends);
        let rows = self.rows();
        self.values.resize(self.columns() * rows, f64::NAN);
        for ((x, y), value) in xs.iter().zip(ys).zip(values) {
            if let (Some(column), Some(row)) = (index(&self.x_edges, *x), index(&self.y_edges, *y))
            {
                self.values[column * rows + row] = *value;
            }
        }
    }

    pub(crate) fn buckets(
        &mut self,
        xs: &[f64],
        names: &[SharedString],
        read: impl Fn(usize, usize) -> f64,
        layout: BucketLayout,
        numeric: bool,
    ) {
        self.clear();
        let mut starts: Vec<f64> = xs.iter().copied().filter(|x| x.is_finite()).collect();
        if starts.is_empty() || names.is_empty() {
            return;
        }
        edges(&mut starts, &mut self.x_edges, None);
        let bounds: Option<Vec<f64>> = numeric
            .then(|| {
                names
                    .iter()
                    .map(|name| match name.trim() {
                        "+Inf" | "Inf" | "inf" => Some(f64::INFINITY),
                        text => text.parse::<f64>().ok(),
                    })
                    .collect()
            })
            .flatten();
        if let Some(bounds) = bounds {
            let finite: Vec<f64> = bounds
                .iter()
                .copied()
                .filter(|bound| bound.is_finite())
                .collect();
            let step = finite
                .windows(2)
                .map(|pair| pair[1] - pair[0])
                .next_back()
                .unwrap_or(1.);
            let clean = |bound: f64| {
                if bound.is_finite() {
                    bound
                } else {
                    finite.last().copied().unwrap_or(0.) + step
                }
            };
            if layout == BucketLayout::Ge {
                self.y_edges
                    .extend(bounds.iter().map(|bound| clean(*bound)));
                self.y_edges
                    .push(clean(*bounds.last().unwrap_or(&0.)) + step);
            } else {
                let first = clean(bounds[0]);
                self.y_edges.push(if first > 0. {
                    0f64.max(first - step)
                } else {
                    first - step
                });
                self.y_edges
                    .extend(bounds.iter().map(|bound| clean(*bound)));
            }
        } else {
            self.y_edges
                .extend((0..=names.len()).map(|index| index as f64));
            self.labels.extend(names.iter().cloned());
            self.layout = Some(layout);
        }
        let rows = names.len();
        self.values.resize(self.columns() * rows, f64::NAN);
        for (row, x) in xs.iter().enumerate() {
            if let Some(column) = index(&self.x_edges, *x) {
                for field in 0..rows {
                    self.values[column * rows + field] = read(row, field);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_the_largest_nice_increment_below_the_estimate() {
        assert!((nice_incr(7., false) - 5.).abs() < 1e-9);
        assert!((nice_incr(0.3, false) - 0.25).abs() < 1e-9);
        assert!((nice_incr(70_000., true) - 60_000.).abs() < 1e-9);
    }

    #[test]
    fn counts_samples_into_buckets() {
        let mut grid = Grid::default();
        let xs = [0., 1., 2., 9.];
        let ys = [1., 1., 5., 9.];
        let calc = Calculation {
            x_buckets: Buckets::Size(5.),
            y_buckets: Buckets::Size(5.),
            y_scale: BucketScale::Linear,
        };
        grid.samples(&xs, &ys, calc, false);
        assert_eq!(grid.x_edges, [0., 5., 10.]);
        assert_eq!(grid.y_edges, [0., 5., 10.]);
        assert_eq!(grid.values, [2., 1., 0., 1.]);
    }

    #[test]
    fn log_buckets_use_powers() {
        let mut grid = Grid::default();
        let calc = Calculation {
            x_buckets: Buckets::Size(10.),
            y_buckets: Buckets::Auto,
            y_scale: BucketScale::Log10,
        };
        grid.samples(&[0., 1., 2.], &[5., 50., 500.], calc, false);
        assert_eq!(grid.y_edges, [1., 10., 100., 1000.]);
        assert_eq!(grid.values, [1., 1., 1.]);
    }

    #[test]
    fn reads_bucket_fields_as_ordinal_rows() {
        let mut grid = Grid::default();
        let names: Vec<SharedString> = ["0.1", "0.5", "+Inf"].map(Into::into).into();
        grid.buckets(
            &[0., 10.],
            &names,
            |row, field| (row * 3 + field) as f64,
            BucketLayout::Le,
            false,
        );
        assert_eq!(grid.rows(), 3);
        assert_eq!(grid.labels.len(), 3);
        assert!((grid.value(1, 2) - 5.).abs() < f64::EPSILON);
    }

    #[test]
    fn places_cells_by_their_start() {
        let mut grid = Grid::default();
        grid.cells(&[0., 0., 2.], &[0., 1., 1.], &[3., 4., 5.], None, None);
        assert_eq!(grid.x_edges, [0., 2., 4.]);
        assert_eq!(grid.y_edges, [0., 1., 2.]);
        assert!(grid.value(1, 0).is_nan());
        assert!((grid.value(1, 1) - 5.).abs() < f64::EPSILON);
    }
}
