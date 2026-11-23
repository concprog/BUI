use std::hash::{DefaultHasher, Hash, Hasher};
use std::ops::Range;

use blenderui::SharedString;

use crate::bar_gauge::{Source, Text};
use crate::data::Revision;

pub(crate) struct Item {
    pub name: SharedString,
    pub value: f64,
    pub text: SharedString,
    pub change: Option<f64>,
    pub series: Option<Range<usize>>,
}

#[derive(Default)]
pub(crate) struct Values {
    key: Option<u64>,
    pub items: Vec<Item>,
    pub xs: Vec<f64>,
    pub ys: Vec<f64>,
    pub extent: (f64, f64),
}

pub(crate) enum Times<'a, T> {
    Index,
    Row(&'a dyn Fn(&T) -> f64),
    Field(fn(&T, usize) -> f64, usize),
}

pub(crate) fn percent_change(column: &[f64]) -> Option<f64> {
    let mut finite = column.iter().copied().filter(|value| value.is_finite());
    let first = finite.next()?;
    let last = finite.next_back().unwrap_or(first);
    let change = (last - first) / first * 100.;
    change.is_finite().then_some(change)
}

impl Values {
    pub(crate) fn fresh(&self, key: Option<u64>) -> bool {
        key.is_some() && self.key == key
    }

    pub(crate) fn key<T>(
        source: &Source<'_, T>,
        revision: Option<Revision>,
        text: Text<'_>,
        flags: (bool, bool, bool),
    ) -> Option<u64> {
        let base = source.key(revision, text)?;
        let mut state = DefaultHasher::new();
        (base, flags).hash(&mut state);
        Some(state.finish())
    }

    pub(crate) fn refresh<T>(
        &mut self,
        key: Option<u64>,
        source: &Source<'_, T>,
        times: &Times<'_, T>,
        text: Text<'_>,
        sparkline: bool,
        change: bool,
    ) {
        self.key = key;
        self.items.clear();
        self.ys.clear();
        self.xs.clear();
        match times {
            Times::Index => self
                .xs
                .extend((0..source.rows.len()).map(|index| index as f64)),
            Times::Row(read) => self.xs.extend(source.rows.iter().map(read)),
            Times::Field(read, field) => {
                self.xs
                    .extend(source.rows.iter().map(|row| read(row, *field)));
            }
        }
        let (mut low, mut high) = (f64::INFINITY, f64::NEG_INFINITY);
        let (items, ys) = (&mut self.items, &mut self.ys);
        source.each(|name, value, column| {
            for number in column.unwrap_or_else(|| std::slice::from_ref(&value)) {
                if number.is_finite() {
                    low = low.min(*number);
                    high = high.max(*number);
                }
            }
            let series = column
                .filter(|column| sparkline && column.len() > 1)
                .map(|column| {
                    let start = ys.len();
                    ys.extend_from_slice(column);
                    start..ys.len()
                });
            items.push(Item {
                name,
                value,
                text: text.of(value),
                change: column.filter(|_| change).and_then(percent_change),
                series,
            });
        });
        self.extent = if low <= high { (low, high) } else { (0., 1.) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_change_uses_the_first_and_last_numbers() {
        assert_eq!(
            percent_change(&[f64::NAN, 50., 60., 75., f64::NAN]),
            Some(50.)
        );
        assert_eq!(percent_change(&[-10., -5.]), Some(-50.));
        assert_eq!(percent_change(&[0., 5.]), None);
        assert_eq!(percent_change(&[]), None);
    }
}
