use std::hash::{DefaultHasher, Hash, Hasher};

use blenderui::SharedString;

use super::options::Reduce;
use crate::Schema;
use crate::data::Revision;
use crate::format::{fixed, trimmed};

pub(crate) struct Bar {
    pub name: SharedString,
    pub value: f64,
    pub text: SharedString,
}

#[derive(Default)]
pub(crate) struct Bars {
    key: Option<u64>,
    pub items: Vec<Bar>,
}

#[derive(Clone, Copy)]
pub(crate) struct Text<'a> {
    pub format: Option<&'a dyn Fn(f64) -> SharedString>,
    pub decimals: Option<usize>,
}

impl Text<'_> {
    pub(crate) fn of(self, value: f64) -> SharedString {
        match (self.format, self.decimals) {
            (Some(format), _) => format(value),
            (None, Some(decimals)) => fixed(value, decimals),
            (None, None) => trimmed(value),
        }
    }
}

pub(crate) struct Source<'a, T> {
    pub rows: &'a [T],
    pub name: Option<&'a dyn Fn(&T) -> &str>,
    pub value: Option<&'a dyn Fn(&T) -> f64>,
    pub schema: Option<Schema<'a, T>>,
    pub fields: Option<&'static [&'static str]>,
    pub skip: Option<usize>,
    pub reduce: Reduce,
}

impl<T> Source<'_, T> {
    fn fields<'s>(&'s self, names: &'s [SharedString]) -> impl Iterator<Item = usize> + 's {
        (0..names.len()).filter(move |index| {
            self.fields
                .is_none_or(|fields| fields.contains(&names[*index].as_ref()))
                && Some(*index) != self.skip
        })
    }

    pub(crate) fn fill(&self, bars: &mut Vec<Bar>, text: Text<'_>) {
        bars.clear();
        self.each(|name, value, _| {
            bars.push(Bar {
                name,
                value,
                text: text.of(value),
            });
        });
    }

    pub(crate) fn each(&self, mut push: impl FnMut(SharedString, f64, Option<&[f64]>)) {
        match (self.value, &self.schema, self.reduce) {
            (Some(value), _, Reduce::All { limit }) => {
                for row in self.rows.iter().take(limit.unwrap_or(usize::MAX)) {
                    let name = self
                        .name
                        .map_or_else(SharedString::default, |name| name(row).to_owned().into());
                    push(name, value(row), None);
                }
            }
            (Some(value), _, Reduce::Calc(calc)) => {
                let values: Vec<f64> = self.rows.iter().map(value).collect();
                push(
                    SharedString::default(),
                    calc.apply(&values).unwrap_or(f64::NAN),
                    Some(&values),
                );
            }
            (None, Some(schema), Reduce::Calc(calc)) => {
                let mut column = Vec::with_capacity(self.rows.len());
                for field in self.fields(schema.names) {
                    column.clear();
                    column.extend(self.rows.iter().map(|row| (schema.read)(row, field)));
                    push(
                        schema.names[field].clone(),
                        calc.apply(&column).unwrap_or(f64::NAN),
                        Some(&column),
                    );
                }
            }
            (None, Some(schema), Reduce::All { limit }) => {
                let cells = self
                    .rows
                    .iter()
                    .flat_map(|row| self.fields(schema.names).map(move |field| (row, field)));
                for (row, field) in cells.take(limit.unwrap_or(usize::MAX)) {
                    push(schema.names[field].clone(), (schema.read)(row, field), None);
                }
            }
            (None, None, _) => {}
        }
    }

    pub(crate) fn key(&self, revision: Option<Revision>, text: Text<'_>) -> Option<u64> {
        let revision = revision?;
        let mut state = DefaultHasher::new();
        (
            revision,
            self.reduce,
            self.fields,
            self.skip,
            self.name.is_some(),
            self.value.is_some(),
        )
            .hash(&mut state);
        (text.format.is_some(), text.decimals).hash(&mut state);
        Some(state.finish())
    }
}

impl Bars {
    pub(crate) fn fresh(&self, key: Option<u64>) -> bool {
        key.is_some() && self.key == key
    }

    pub(crate) fn store(&mut self, key: Option<u64>, items: Vec<Bar>) {
        self.key = key;
        self.items = items;
    }
}
