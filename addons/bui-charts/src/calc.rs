#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Calc {
    First,
    Last,
    Min,
    Max,
    Mean,
    Median,
    Sum,
    Count,
    Range,
    Diff,
    Variance,
    StdDev,
}

impl Calc {
    pub const COUNT: usize = 12;
    pub const ALL: [Self; Self::COUNT] = [
        Self::First,
        Self::Last,
        Self::Min,
        Self::Max,
        Self::Mean,
        Self::Median,
        Self::Sum,
        Self::Count,
        Self::Range,
        Self::Diff,
        Self::Variance,
        Self::StdDev,
    ];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::First => "First",
            Self::Last => "Last",
            Self::Min => "Min",
            Self::Max => "Max",
            Self::Mean => "Mean",
            Self::Median => "Median",
            Self::Sum => "Total",
            Self::Count => "Count",
            Self::Range => "Range",
            Self::Diff => "Difference",
            Self::Variance => "Variance",
            Self::StdDev => "StdDev",
        }
    }

    pub(crate) fn apply(self, values: &[f64]) -> Option<f64> {
        let mut finite = values.iter().copied().filter(|value| value.is_finite());
        match self {
            Self::First => finite.next(),
            Self::Last => finite.next_back(),
            Self::Min => finite.reduce(f64::min),
            Self::Max => finite.reduce(f64::max),
            Self::Sum => Some(finite.sum()),
            Self::Count => Some(finite.count() as f64),
            Self::Mean => {
                let (sum, count) =
                    finite.fold((0., 0.), |(sum, count), value| (sum + value, count + 1.));
                (count > 0.).then(|| sum / count)
            }
            Self::Median => {
                let mut sorted: Vec<f64> = finite.collect();
                let len = sorted.len();
                if len == 0 {
                    return None;
                }
                let (below, median, _) = sorted.select_nth_unstable_by(len / 2, f64::total_cmp);
                let median = *median;
                Some(match len % 2 {
                    0 => f64::midpoint(below.iter().copied().fold(f64::MIN, f64::max), median),
                    _ => median,
                })
            }
            Self::Range => Some(Self::Max.apply(values)? - Self::Min.apply(values)?),
            Self::Diff => Some(Self::Last.apply(values)? - Self::First.apply(values)?),
            Self::Variance => {
                let mean = Self::Mean.apply(values)?;
                let (sum, count) = finite.fold((0., 0.), |(sum, count), value| {
                    ((value - mean).mul_add(value - mean, sum), count + 1.)
                });
                Some(sum / count)
            }
            Self::StdDev => Self::Variance.apply(values).map(f64::sqrt),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct CalcSet(u16);

impl CalcSet {
    #[must_use]
    pub const fn contains(self, calc: Calc) -> bool {
        self.0 & (1 << calc as u16) != 0
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub fn iter(self) -> impl Iterator<Item = Calc> {
        Calc::ALL
            .into_iter()
            .filter(move |calc| self.contains(*calc))
    }
}

impl FromIterator<Calc> for CalcSet {
    fn from_iter<I: IntoIterator<Item = Calc>>(calcs: I) -> Self {
        Self(
            calcs
                .into_iter()
                .fold(0, |set, calc| set | 1 << calc as u16),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{Calc, CalcSet};

    #[test]
    fn calcs_skip_missing_values() {
        let values = [f64::NAN, 2., 4., f64::NAN, 9.];
        assert_eq!(Calc::First.apply(&values), Some(2.));
        assert_eq!(Calc::Last.apply(&values), Some(9.));
        assert_eq!(Calc::Mean.apply(&values), Some(5.));
        assert_eq!(Calc::Median.apply(&values), Some(4.));
        assert_eq!(Calc::Median.apply(&[4., 1., 3., 2.]), Some(2.5));
        assert_eq!(Calc::Count.apply(&values), Some(3.));
        assert_eq!(Calc::Range.apply(&values), Some(7.));
        assert_eq!(Calc::Diff.apply(&values), Some(7.));
        assert_eq!(Calc::Max.apply(&[f64::NAN]), None);
    }

    #[test]
    fn sets_iterate_in_display_order() {
        let set: CalcSet = [Calc::Max, Calc::First].into_iter().collect();
        assert_eq!(set.iter().collect::<Vec<_>>(), [Calc::First, Calc::Max]);
        assert!(CalcSet::default().is_empty());
    }
}
