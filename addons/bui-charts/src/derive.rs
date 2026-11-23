use std::hash::{DefaultHasher, Hash, Hasher};

use blenderui::SharedString;
use blenderui::hooks::Signal;

use crate::data::{Accessor, Revision};
use crate::{Dataset, Schema};

pub(crate) type Row = Box<[f64]>;

#[derive(Default)]
pub(crate) struct Table {
    key: Option<u64>,
    pub rows: Vec<Row>,
    pub names: Vec<SharedString>,
    pub fields: Vec<SharedString>,
}

/// A derived chart's own rows, built from its data (see [`sync`]): a dataset for the base charts.
#[derive(Clone, Copy)]
pub(crate) struct Rows(pub Signal<Table>);

fn read(row: &Row, field: usize) -> f64 {
    row.get(field).copied().unwrap_or(f64::NAN)
}

pub(crate) const fn column(field: usize) -> Accessor<Row> {
    Accessor::Field(read, field)
}

impl Dataset<Row> for Rows {
    fn read<R>(&self, f: impl FnOnce(&[Row], Option<Schema<'_, Row>>) -> R) -> R {
        self.0.with(|table| f(&table.rows, Some(Schema { names: &table.fields, read, size: None, color: None })))
    }

    /// The key the table was built for (written untracked, so the signal's own writes don't count).
    #[track_caller]
    fn revision(&self) -> Option<Revision> {
        self.0.with(|table| table.key).map(|key| Revision { source: key, version: 0 })
    }
}

#[derive(Default)]
pub(crate) struct Build {
    pub rows: Vec<Row>,
    pub names: Vec<SharedString>,
    pub fields: Vec<SharedString>,
}

fn store(table: Signal<Table>, key: Option<u64>, built: Build) {
    table.update_untracked(|table| {
        table.key = key;
        table.rows = built.rows;
        table.names = built.names;
        table.fields = built.fields;
    });
}

#[track_caller]
pub(crate) fn data_key<T, D: Dataset<T>>(data: &D, settings: impl Hash) -> Option<u64> {
    data.revision().map(|revision| {
        let mut state = DefaultHasher::new();
        (revision, settings).hash(&mut state);
        state.finish()
    })
}

/// Builds the table from `data` when its revision or `settings` changed (always for data without
/// a revision). Called while rendering.
#[track_caller]
pub(crate) fn sync<T, D: Dataset<T>>(
    table: Signal<Table>,
    data: &D,
    settings: impl Hash,
    build: impl FnOnce(&[T]) -> Build,
) {
    let key = data_key(data, settings);
    if key.is_some() && table.with(|table| table.key) == key {
        return;
    }
    let built = data.rows(build);
    store(table, key, built);
}

pub(crate) fn sync_with(table: Signal<Table>, settings: impl Hash, build: impl FnOnce() -> Build) {
    let mut state = DefaultHasher::new();
    settings.hash(&mut state);
    let key = Some(state.finish());
    if table.with(|table| table.key) == key {
        return;
    }
    store(table, key, build());
}

pub(crate) fn row<const N: usize>(cells: [f64; N]) -> Row {
    Box::new(cells)
}

pub(crate) fn bits(values: &[f64]) -> Vec<u64> {
    values.iter().map(|value| value.to_bits()).collect()
}
