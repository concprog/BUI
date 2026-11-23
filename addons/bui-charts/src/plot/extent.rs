use super::scale::ScaleKind;

pub(crate) type Range = Option<(f64, f64)>;

#[derive(Clone, Copy, Default)]
pub(crate) struct Extent {
    finite: Range,
    positive: Range,
}

impl Extent {
    pub(crate) fn of(values: &[f64]) -> Self {
        values
            .iter()
            .fold(Self::default(), |extent, &value| extent.with(value))
    }

    pub(crate) const fn with(self, value: f64) -> Self {
        if !value.is_finite() {
            return self;
        }
        Self {
            finite: Some(widen(self.finite, value)),
            positive: if value > 0. {
                Some(widen(self.positive, value))
            } else {
                self.positive
            },
        }
    }

    pub(crate) const fn merge(self, other: Self) -> Self {
        Self {
            finite: union(self.finite, other.finite),
            positive: union(self.positive, other.positive),
        }
    }

    pub(crate) const fn for_scale(self, kind: ScaleKind) -> Range {
        match kind {
            ScaleKind::Log { .. } => self.positive,
            _ => self.finite,
        }
    }
}

pub(crate) const fn widen(range: Range, value: f64) -> (f64, f64) {
    match range {
        None => (value, value),
        Some((min, max)) => (value.min(min), value.max(max)),
    }
}

pub(crate) const fn finite(range: Range, value: f64) -> Range {
    if value.is_finite() {
        Some(widen(range, value))
    } else {
        range
    }
}

const fn union(a: Range, b: Range) -> Range {
    match (a, b) {
        (Some(a), Some(b)) => Some((a.0.min(b.0), a.1.max(b.1))),
        (Some(range), None) | (None, Some(range)) => Some(range),
        (None, None) => None,
    }
}
