use super::axis::AxisConfig;
use crate::time;

const NICE_STEPS: [f64; 5] = [1., 2., 2.5, 5., 10.];

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ScaleKind {
    #[default]
    Linear,
    Log {
        base: f64,
    },
    Symlog {
        base: f64,
        linear_threshold: f64,
    },
}

impl ScaleKind {
    fn backward(self, value: f64) -> f64 {
        match self {
            Self::Linear => value,
            Self::Log { base } => base.powf(value),
            Self::Symlog {
                base,
                linear_threshold,
            } => (value * base.ln()).sinh() * linear_threshold,
        }
    }

    fn forward(self, value: f64) -> f64 {
        match self {
            Self::Linear => value,
            Self::Log { base } => value.log(base),
            Self::Symlog {
                base,
                linear_threshold,
            } => (value / linear_threshold).asinh() / base.ln(),
        }
    }
}

pub(crate) const MAX_TICKS: usize = 32;

#[derive(Clone, Copy)]
pub(crate) struct Ticks {
    values: [f64; MAX_TICKS],
    len: usize,
    pub step: Option<f64>,
}

impl Ticks {
    const fn new(step: Option<f64>) -> Self {
        Self {
            values: [0.; MAX_TICKS],
            len: 0,
            step,
        }
    }

    const fn push(&mut self, value: f64) -> bool {
        let free = self.len < MAX_TICKS;
        if free {
            self.values[self.len] = value;
            self.len += 1;
        }
        free
    }

    pub(crate) fn as_slice(&self) -> &[f64] {
        &self.values[..self.len]
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Scale {
    kind: ScaleKind,
    min: f64,
    max: f64,
    from: f64,
    span: f64,
    time: Option<i32>,
    reverse: bool,
}

impl Scale {
    pub(crate) fn new(axis: &AxisConfig, extent: Option<(f64, f64)>) -> Self {
        if axis.time {
            return Self::exact(axis, extent);
        }
        let (lo, hi) = extent.unwrap_or((0., 1.));
        let (lo, hi) = nice(axis.scale, lo, hi);
        Self::within(axis, lo, hi)
    }

    pub(crate) fn exact(axis: &AxisConfig, extent: Option<(f64, f64)>) -> Self {
        match extent {
            Some((lo, hi)) if hi > lo => Self::within(axis, lo, hi),
            Some((lo, _)) if axis.time => Self::within(axis, lo - 30_000., lo + 30_000.),
            None if axis.time => Self::within(axis, 0., 60_000.),
            _ => {
                let (lo, hi) = nice(
                    axis.scale,
                    extent.map_or(0., |e| e.0),
                    extent.map_or(1., |e| e.1),
                );
                Self::within(axis, lo, hi)
            }
        }
    }

    fn within(axis: &AxisConfig, mut lo: f64, mut hi: f64) -> Self {
        lo = axis.soft_min.map_or(lo, |soft| lo.min(soft));
        hi = axis.soft_max.map_or(hi, |soft| hi.max(soft));
        if axis.centered_zero {
            let reach = lo.abs().max(hi.abs());
            (lo, hi) = (-reach, reach);
        }
        let (min, max) = (axis.min.unwrap_or(lo), axis.max.unwrap_or(hi));
        let from = axis.scale.forward(min);
        Self {
            kind: axis.scale,
            min,
            max,
            from,
            span: axis.scale.forward(max) - from,
            time: axis.time.then_some(axis.utc_offset),
            reverse: axis.reverse,
        }
    }

    pub(crate) const fn baseline(&self) -> f64 {
        match self.kind {
            ScaleKind::Log { .. } => self.min,
            _ => 0f64.clamp(self.min, self.max),
        }
    }

    pub(crate) fn fraction(&self, value: f64) -> f32 {
        let fraction = ((self.kind.forward(value) - self.from) / self.span) as f32;
        if self.reverse {
            1. - fraction
        } else {
            fraction
        }
    }

    pub(crate) const fn range(&self) -> (f64, f64) {
        (self.min, self.max)
    }

    pub(crate) fn value(&self, fraction: f32) -> f64 {
        let fraction = f64::from(if self.reverse {
            1. - fraction
        } else {
            fraction
        });
        self.kind.backward(self.span.mul_add(fraction, self.from))
    }

    pub(crate) fn ticks(&self, length: f32, spacing: f32) -> Ticks {
        if let Some(offset) = self.time {
            let count = (length / spacing).clamp(1., (MAX_TICKS - 1) as f32);
            let step = time::step(self.max - self.min, count);
            let first = time::first(self.min, step, offset);
            let mut ticks = Ticks::new(Some(step));
            (0..MAX_TICKS as u32)
                .map(|i| f64::from(i).mul_add(step, first))
                .take_while(|value| *value <= self.max)
                .all(|value| ticks.push(value));
            return ticks;
        }
        match self.kind {
            ScaleKind::Linear => {
                let count = (length / spacing).clamp(1., (MAX_TICKS - 1) as f32);
                let step = nice_step((self.max - self.min) / f64::from(count));
                let first = (self.min / step).ceil() * step;
                let mut ticks = Ticks::new(Some(step));
                (0..MAX_TICKS as u32)
                    .map(|i| f64::from(i).mul_add(step, first))
                    .take_while(|value| *value <= self.max + step * 1e-9)
                    .all(|value| ticks.push(value));
                ticks
            }
            ScaleKind::Log { base } => self.thin(powers(base, self.min, self.max), length, spacing),
            ScaleKind::Symlog { base, .. } => {
                let reach = self.min.abs().max(self.max.abs());
                let candidates = std::iter::once(0.)
                    .chain(powers(base, 1., reach).flat_map(|power| [-power, power]))
                    .filter(|value| (self.min..=self.max).contains(value));
                self.thin(candidates, length, spacing)
            }
        }
    }

    fn thin(&self, candidates: impl Iterator<Item = f64>, length: f32, spacing: f32) -> Ticks {
        let mut kept = [(0f32, 0f64); MAX_TICKS];
        let mut len = 0;
        for value in candidates {
            let at = self.fraction(value) * length;
            if len < MAX_TICKS
                && kept[..len]
                    .iter()
                    .all(|(other, _)| (at - other).abs() >= spacing)
            {
                kept[len] = (at, value);
                len += 1;
            }
        }
        kept[..len].sort_unstable_by(|a, b| a.0.total_cmp(&b.0));
        let mut ticks = Ticks::new(None);
        kept[..len].iter().all(|(_, value)| ticks.push(*value));
        ticks
    }
}

fn powers(base: f64, from: f64, to: f64) -> impl Iterator<Item = f64> {
    let (first, last) = (
        exponent(from, base).ceil() as i32,
        exponent(to, base).floor() as i32,
    );
    (first..=last)
        .take(256)
        .map(move |exponent| base.powi(exponent))
}

fn nice(kind: ScaleKind, lo: f64, hi: f64) -> (f64, f64) {
    if let ScaleKind::Log { base } = kind {
        let floor = exponent(lo, base).floor();
        let ceil = exponent(hi, base).ceil().max(floor + 1.);
        (base.powf(floor), base.powf(ceil))
    } else {
        let (lo, hi) = if hi <= lo {
            let pad = if lo == 0. { 1. } else { lo.abs() * 0.1 };
            (lo - pad, hi + pad)
        } else {
            (lo, hi)
        };
        let step = nice_step((hi - lo) / 5.);
        ((lo / step).floor() * step, (hi / step).ceil() * step)
    }
}

fn exponent(value: f64, base: f64) -> f64 {
    let exponent = value.log(base);
    let rounded = exponent.round();
    if (exponent - rounded).abs() < 1e-9 {
        rounded
    } else {
        exponent
    }
}

fn nice_step(raw: f64) -> f64 {
    let magnitude = 10f64.powf(raw.log10().floor());
    let normalized = raw / magnitude;
    NICE_STEPS
        .into_iter()
        .find(|step| normalized <= *step)
        .unwrap_or(10.)
        * magnitude
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_extents_round_to_nice_steps() {
        let scale = Scale::new(&AxisConfig::new(), Some((0.3, 9.6)));
        assert_eq!((scale.min, scale.max), (0., 10.));
        assert_eq!(scale.ticks(100., 25.).as_slice(), [0., 2.5, 5., 7.5, 10.]);
    }

    #[test]
    fn soft_limits_widen_and_hard_limits_pin() {
        let axis = AxisConfig::new().soft_min(-20.).max(50.);
        let scale = Scale::new(&axis, Some((0., 100.)));
        assert_eq!((scale.min, scale.max), (-20., 50.));
    }

    #[test]
    fn log_extents_snap_to_powers() {
        let axis = AxisConfig::new().scale(ScaleKind::Log { base: 10. });
        let scale = Scale::new(&axis, Some((3., 420.)));
        assert_eq!((scale.min, scale.max), (1., 1000.));
        assert_eq!(scale.ticks(300., 20.).as_slice(), [1., 10., 100., 1000.]);
        assert!((scale.fraction(100.) - 2. / 3.).abs() < 1e-6);
    }

    #[test]
    fn symlog_ticks_keep_zero_and_stay_symmetric() {
        let axis = AxisConfig::new().scale(ScaleKind::Symlog {
            base: 10.,
            linear_threshold: 10.,
        });
        let scale = Scale::new(&axis, Some((-27000., 27000.)));
        let ticks = scale.ticks(240., 40.);
        let ticks = ticks.as_slice();
        assert!(ticks.contains(&0.));
        assert!(ticks.iter().all(|tick| ticks.contains(&-tick)));
    }

    #[test]
    fn centered_zero_is_symmetric() {
        let scale = Scale::new(&AxisConfig::new().centered_zero(true), Some((-3., 8.)));
        assert!((scale.min + scale.max).abs() < f64::EPSILON);
    }
}
