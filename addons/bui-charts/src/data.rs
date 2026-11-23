use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};

use blenderui::SharedString;
use blenderui::hooks::{Memo, Signal};

use crate::Scheme;

/// Which rows a chart derived its marks from: a chart keeps what it computed until the revision of
/// its data changes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Revision {
    pub source: u64,
    pub version: u32,
}

static CHANGES: AtomicU64 = AtomicU64::new(1);

impl Revision {
    /// The revision of the value in `signal`, as the calling component sees it: a new one after
    /// every write to the signal (blenderui counts writes per signal; a memo at the caller's
    /// place notices them). Call it while rendering, from the same place each render (a hook).
    #[track_caller]
    #[must_use]
    pub fn of<V: 'static>(signal: Signal<V>) -> Self {
        let change = Memo::new(move || {
            signal.with(|_| ());
            CHANGES.fetch_add(1, Ordering::Relaxed)
        });
        Self { source: change.get(), version: 0 }
    }
}

pub struct Schema<'a, T> {
    pub names: &'a [SharedString],
    pub read: fn(&T, usize) -> f64,
    pub size: Option<usize>,
    pub color: Option<(usize, &'a Scheme)>,
}

impl<T> Clone for Schema<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Schema<'_, T> {}

/// What a chart draws: rows of `T` (and, for frames, named numeric fields).
///
/// The rows are lent for the time of a call (`read`), since a signal's value lives in blenderui's
/// signal store; do not read the same dataset again inside `f`.
pub trait Dataset<T>: 'static {
    /// The rows and, when the data names its fields, its schema.
    fn read<R>(&self, f: impl FnOnce(&[T], Option<Schema<'_, T>>) -> R) -> R;

    /// Changes whenever the rows do; `None` when it can't tell (charts then derive again on every
    /// render). Called while rendering.
    #[track_caller]
    fn revision(&self) -> Option<Revision> {
        None
    }

    fn rows<R>(&self, f: impl FnOnce(&[T]) -> R) -> R {
        self.read(|rows, _| f(rows))
    }

    fn schema<R>(&self, f: impl FnOnce(Option<Schema<'_, T>>) -> R) -> R {
        self.read(|_, schema| f(schema))
    }
}

impl<T: 'static> Dataset<T> for Vec<T> {
    fn read<R>(&self, f: impl FnOnce(&[T], Option<Schema<'_, T>>) -> R) -> R {
        f(self, None)
    }
}

impl<T: 'static> Dataset<T> for Signal<Vec<T>> {
    fn read<R>(&self, f: impl FnOnce(&[T], Option<Schema<'_, T>>) -> R) -> R {
        self.with(|rows| f(rows, None))
    }

    #[track_caller]
    fn revision(&self) -> Option<Revision> {
        Some(Revision::of(*self))
    }
}

pub struct Frame<const N: usize> {
    names: [SharedString; N],
    rows: Vec<[f64; N]>,
    size: Option<usize>,
    color: Option<(usize, Scheme)>,
}

impl<const N: usize> Frame<N> {
    pub fn new<S: Into<SharedString>>(names: [S; N]) -> Self {
        Self { names: names.map(Into::into), rows: Vec::new(), size: None, color: None }
    }

    pub fn from_columns<S: Into<SharedString>, C: AsRef<[f64]>>(columns: [(S, C); N]) -> Self {
        let len = columns.iter().map(|(_, values)| values.as_ref().len()).max().unwrap_or(0);
        let rows = (0..len)
            .map(|row| std::array::from_fn(|field| columns[field].1.as_ref().get(row).copied().unwrap_or(f64::NAN)))
            .collect();
        Self { rows, ..Self::new(columns.map(|(name, _)| name)) }
    }

    #[must_use]
    pub fn with_rows(mut self, rows: impl Into<Vec<[f64; N]>>) -> Self {
        self.rows = rows.into();
        self
    }

    pub fn push(&mut self, row: [f64; N]) {
        self.rows.push(row);
    }

    #[must_use]
    pub fn size(mut self, field: &str) -> Self {
        self.size = self.index(field);
        self
    }

    #[must_use]
    pub fn color(mut self, field: &str, scheme: Scheme) -> Self {
        self.color = self.index(field).map(|field| (field, scheme));
        self
    }

    #[must_use]
    pub fn index(&self, field: &str) -> Option<usize> {
        self.names.iter().position(|name| name == field)
    }

    pub fn get(&self, field: &str) -> impl Fn(&[f64; N]) -> f64 + Copy + 'static {
        let field = self.index(field);
        move |row| field.map_or(f64::NAN, |field| row[field])
    }

    fn schema(&self) -> Schema<'_, [f64; N]> {
        Schema {
            names: &self.names,
            read: read::<N>,
            size: self.size,
            color: self.color.as_ref().map(|(field, scheme)| (*field, scheme)),
        }
    }
}

const fn read<const N: usize>(row: &[f64; N], field: usize) -> f64 {
    row[field]
}

impl<const N: usize> Dataset<[f64; N]> for Frame<N> {
    fn read<R>(&self, f: impl FnOnce(&[[f64; N]], Option<Schema<'_, [f64; N]>>) -> R) -> R {
        f(&self.rows, Some(self.schema()))
    }
}

impl<const N: usize> Dataset<[f64; N]> for Signal<Frame<N>> {
    fn read<R>(&self, f: impl FnOnce(&[[f64; N]], Option<Schema<'_, [f64; N]>>) -> R) -> R {
        self.with(|frame| f(&frame.rows, Some(frame.schema())))
    }

    #[track_caller]
    fn revision(&self) -> Option<Revision> {
        Some(Revision::of(*self))
    }
}

pub(crate) enum Accessor<T> {
    Row(Box<dyn Fn(&T) -> f64>),
    Field(fn(&T, usize) -> f64, usize),
}

impl<T> Accessor<T> {
    pub(crate) fn row(read: impl Fn(&T) -> f64 + 'static) -> Self {
        Self::Row(Box::new(read))
    }

    pub(crate) fn fill(&self, rows: &[T], out: &mut Vec<f64>) {
        out.clear();
        match self {
            Self::Row(read) => out.extend(rows.iter().map(read)),
            Self::Field(read, field) => out.extend(rows.iter().map(|row| read(row, *field))),
        }
    }

    pub(crate) fn fingerprint<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Row(_) => usize::MAX.hash(state),
            Self::Field(_, field) => field.hash(state),
        }
    }
}
