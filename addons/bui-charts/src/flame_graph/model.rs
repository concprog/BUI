use std::cell::OnceCell;
use std::collections::BTreeSet;
use std::hash::{DefaultHasher, Hash, Hasher};

use blenderui::{Hsla, SharedString};

use crate::geom::Point;
use blenderui::hooks::Signal;

use super::colors::package_color;
use super::options::{FlameColors, FrameAction, LabelAlign, PaneLayout, SampleUnit};
use super::profile::Profile;
use super::search::{Finder, mark};
use super::tree::{NONE, Tree};
use crate::data::Revision;
use crate::format::{fixed, trimmed};

pub(crate) const ROW_HEIGHT: f32 = 22.;
pub(crate) const MAX_MATCHES: usize = 50;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Which {
    Main,
    Callers,
    Callees,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Pointer {
    pub which: Which,
    pub node: u32,
    pub at: Point,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Focus {
    pub which: Which,
    pub node: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Column {
    Symbol,
    Own,
    Total,
    Baseline,
    Comparison,
    Diff,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct Sort {
    pub column: Column,
    pub desc: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) enum Root {
    #[default]
    All,
    Callers(u32),
    Callees(u32),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct CallState {
    pub root: Root,
    pub expanded: BTreeSet<u32>,
    pub sort: Sort,
}

impl Default for CallState {
    fn default() -> Self {
        Self {
            root: Root::All,
            expanded: BTreeSet::from([0]),
            sort: Sort {
                column: Column::Total,
                desc: true,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Totals {
    pub own: f64,
    pub total: f64,
    pub own_right: f64,
    pub total_right: f64,
}

pub(crate) struct Format(pub Option<Box<dyn Fn(f64) -> SharedString>>);

#[derive(Clone, Copy)]
pub(crate) struct Units {
    pub unit: SampleUnit,
    pub decimals: Option<usize>,
    pub format: Signal<Format>,
}

impl Units {
    pub(crate) fn text(self, value: f64) -> SharedString {
        self.format.with(|format| match (&format.0, self.decimals) {
            (Some(format), _) => format(value),
            (None, Some(decimals)) => fixed(value, decimals),
            (None, None) => self.unit.text(value),
        })
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Flame {
    pub profile: Signal<Profile>,
    pub cache: Signal<Cache>,
    pub units: Units,
    pub colors: Signal<FlameColors>,
    pub align: Signal<LabelAlign>,
    pub layout: Signal<PaneLayout>,
    pub search: Signal<SharedString>,
    pub highlight: Signal<Option<u32>>,
    pub focus: Signal<Option<Focus>>,
    pub sandwich: Signal<Option<u32>>,
    pub groups: Signal<BTreeSet<u32>>,
    pub hover: Signal<Option<Pointer>>,
    pub menu: Signal<Option<Pointer>>,
    pub table_sort: Signal<Sort>,
    pub calls: Signal<CallState>,
    pub call_query: Signal<SharedString>,
    pub call_match: Signal<usize>,
    /// the call tree opens to its current match on its next render
    pub call_reveal: Signal<bool>,
    /// the call tree's node to scroll into view
    pub scroll_to: Signal<Option<u32>>,
    pub width: Signal<f32>,
    pub collapsing: bool,
    pub vertical: bool,
    pub header: bool,
    pub actions: &'static [FrameAction],
}

pub(crate) struct Labeled {
    pub tree: Tree,
    texts: Vec<OnceCell<SharedString>>,
}

impl Labeled {
    fn new(tree: Tree) -> Self {
        let texts = (0..tree.nodes.len()).map(|_| OnceCell::new()).collect();
        Self { tree, texts }
    }

    pub(crate) fn text(
        &self,
        node: u32,
        profile: &Profile,
        units: Units,
    ) -> &SharedString {
        self.texts[node as usize].get_or_init(|| {
            let label = profile.label(self.tree.node(node).label);
            format!("{label} ({})", units.text(self.tree.node(node).value)).into()
        })
    }
}

impl Default for Labeled {
    fn default() -> Self {
        Self::new(Tree::default())
    }
}

#[derive(Default)]
pub(crate) struct Sandwich {
    label: Option<u32>,
    pub callers: Labeled,
    pub callees: Labeled,
}

impl Sandwich {
    fn refresh(&mut self, main: &Tree, label: Option<u32>) {
        if self.label == label {
            return;
        }
        self.label = label;
        let build =
            |callers| label.map_or_else(Tree::default, |label| main.sandwich(label, callers));
        self.callers = Labeled::new(build(true));
        self.callees = Labeled::new(build(false));
    }
}

#[derive(Default)]
pub(crate) struct Cache {
    revision: Option<Revision>,
    pub main: Labeled,
    pub heads: Vec<u32>,
    pub rows: Vec<u32>,
    rows_key: Option<u64>,
    pub depth: u32,
    pub package: Vec<Hsla>,
    package_light: Option<bool>,
    pub matches: Vec<bool>,
    query: Option<SharedString>,
    pub flame: Sandwich,
    pub call: Sandwich,
    pub totals: Vec<Totals>,
    pub table: Vec<u32>,
    table_key: Option<u64>,
    pub call_rows: Vec<(u32, u32)>,
    call_rows_key: Option<u64>,
    pub call_matches: Vec<u32>,
    call_match_key: Option<u64>,
    pub call_invalid: bool,
}

pub(crate) struct Inputs<'a> {
    pub profile: &'a Profile,
    pub revision: Option<Revision>,
    pub collapsing: bool,
    pub groups: &'a BTreeSet<u32>,
    pub query: &'a SharedString,
    pub sandwich: Option<u32>,
    pub light: bool,
    pub table_sort: Sort,
    pub calls: &'a CallState,
    pub call_query: &'a SharedString,
}

fn hash(value: impl Hash) -> u64 {
    let mut state = DefaultHasher::new();
    value.hash(&mut state);
    state.finish()
}

impl Cache {
    pub(crate) fn is_stale(&self, revision: Option<Revision>) -> bool {
        revision.is_none() || self.revision != revision
    }

    pub(crate) fn refresh(&mut self, inputs: &Inputs<'_>) {
        if self.is_stale(inputs.revision) {
            let fresh = Self {
                revision: inputs.revision,
                ..Self::default()
            };
            *self = fresh;
            self.main = Labeled::new(Tree::from_profile(inputs.profile));
            self.heads = self.main.tree.groups();
            self.totals = totals(inputs.profile);
        }
        self.refresh_rows(inputs);
        self.refresh_package(inputs);
        self.refresh_matches(inputs);
        self.flame.refresh(&self.main.tree, inputs.sandwich);
        let call_label = match inputs.calls.root {
            Root::All => None,
            Root::Callers(label) | Root::Callees(label) => Some(label),
        };
        self.call.refresh(&self.main.tree, call_label);
        self.refresh_table(inputs);
        self.refresh_call_rows(inputs);
        self.refresh_call_matches(inputs);
    }

    pub(crate) fn group_size(&self, head: u32) -> usize {
        1 + self
            .heads
            .iter()
            .skip(head as usize + 1)
            .take_while(|member| **member == head)
            .count()
    }

    pub(crate) fn hidden(&self, node: u32, collapsing: bool, groups: &BTreeSet<u32>) -> bool {
        let head = self.heads.get(node as usize).copied().unwrap_or(NONE);
        collapsing && head != NONE && head != node && !groups.contains(&head)
    }

    fn refresh_rows(&mut self, inputs: &Inputs<'_>) {
        let key = Some(hash((inputs.collapsing, inputs.groups)));
        if self.rows_key == key && self.rows.len() == self.main.tree.nodes.len() {
            return;
        }
        self.rows_key = key;
        let mut rows = std::mem::take(&mut self.rows);
        rows.clear();
        self.depth = 0;
        for (id, node) in self.main.tree.nodes.iter().enumerate() {
            let shift = self
                .main
                .tree
                .nodes
                .get(node.parent as usize)
                .map_or(0, |parent| {
                    let hidden =
                        u32::from(self.hidden(node.parent, inputs.collapsing, inputs.groups));
                    parent.level - rows[node.parent as usize] + hidden
                });
            let row = node.level - shift;
            rows.push(row);
            if !self.hidden(id as u32, inputs.collapsing, inputs.groups) {
                self.depth = self.depth.max(row + 1);
            }
        }
        self.rows = rows;
    }

    fn refresh_package(&mut self, inputs: &Inputs<'_>) {
        if self.package_light == Some(inputs.light)
            && self.package.len() == inputs.profile.labels().len()
        {
            return;
        }
        self.package_light = Some(inputs.light);
        self.package.clear();
        self.package.extend(
            inputs
                .profile
                .labels()
                .iter()
                .map(|label| package_color(label, inputs.light)),
        );
    }

    fn refresh_matches(&mut self, inputs: &Inputs<'_>) {
        if self.query.as_ref() == Some(inputs.query)
            && self.matches.len() == inputs.profile.labels().len()
        {
            return;
        }
        self.query = Some(inputs.query.clone());
        mark(inputs.query, inputs.profile.labels(), &mut self.matches);
    }

    pub(crate) fn searching(&self) -> bool {
        self.query.as_ref().is_some_and(|query| !query.is_empty())
    }

    pub(crate) fn matched(&self, label: u32) -> bool {
        self.matches.get(label as usize).copied().unwrap_or(false)
    }

    fn refresh_table(&mut self, inputs: &Inputs<'_>) {
        let key = Some(hash((inputs.table_sort, inputs.query)));
        if self.table_key == key && !self.table.is_empty() {
            return;
        }
        self.table_key = key;
        let searching = self.searching();
        let mut table = std::mem::take(&mut self.table);
        table.clear();
        table.extend(
            (0..self.totals.len() as u32)
                .filter(|label| {
                    self.totals[*label as usize].total + self.totals[*label as usize].total_right
                        > 0.
                })
                .filter(|label| !searching || self.matched(*label)),
        );
        let (totals, profile, sort) = (&self.totals, inputs.profile, inputs.table_sort);
        let (left, right) = self
            .main
            .tree
            .nodes
            .first()
            .map_or((0., 0.), |root| (root.value - root.right, root.right));
        table.sort_by(|a, b| {
            let order = match sort.column {
                Column::Symbol => profile.label(*a).cmp(profile.label(*b)),
                column => {
                    let value =
                        |label: u32| column_value(column, &totals[label as usize], (left, right));
                    value(*a).total_cmp(&value(*b))
                }
            };
            if sort.desc { order.reverse() } else { order }
        });
        self.table = table;
    }

    pub(crate) const fn call_tree(&self, root: Root) -> &Labeled {
        match root {
            Root::All => &self.main,
            Root::Callers(_) => &self.call.callers,
            Root::Callees(_) => &self.call.callees,
        }
    }

    fn refresh_call_rows(&mut self, inputs: &Inputs<'_>) {
        let key = Some(hash(inputs.calls));
        if self.call_rows_key == key && !self.call_rows.is_empty() {
            return;
        }
        self.call_rows_key = key;
        let mut rows = std::mem::take(&mut self.call_rows);
        let tree = &self.call_tree(inputs.calls.root).tree;
        rows.clear();
        let mut stack: Vec<(u32, u32)> = Vec::new();
        let mut children: Vec<u32> = Vec::new();
        if !tree.nodes.is_empty() {
            stack.push((0, 0));
        }
        let profile = inputs.profile;
        let sort = inputs.calls.sort;
        while let Some((node, depth)) = stack.pop() {
            rows.push((node, depth));
            if !inputs.calls.expanded.contains(&node) {
                continue;
            }
            children.clear();
            children.extend(tree.children(node));
            children.sort_by(|a, b| {
                let order = match sort.column {
                    Column::Symbol => profile
                        .label(tree.node(*a).label)
                        .cmp(profile.label(tree.node(*b).label)),
                    Column::Own => tree
                        .self_values(*a, profile)
                        .0
                        .total_cmp(&tree.self_values(*b, profile).0),
                    _ => tree.node(*a).value.total_cmp(&tree.node(*b).value),
                };
                if sort.desc { order.reverse() } else { order }
            });
            stack.extend(children.iter().rev().map(|child| (*child, depth + 1)));
        }
        self.call_rows = rows;
    }

    fn refresh_call_matches(&mut self, inputs: &Inputs<'_>) {
        let query = inputs.call_query;
        let key = Some(hash((query, inputs.calls.root)));
        if self.call_match_key == key {
            return;
        }
        self.call_match_key = key;
        let mut matches = std::mem::take(&mut self.call_matches);
        matches.clear();
        self.call_invalid = false;
        if query.is_empty() {
            self.call_matches = matches;
            return;
        }
        let finder = Finder::new(query);
        self.call_invalid = finder.is_invalid();
        let tree = &self.call_tree(inputs.calls.root).tree;
        matches.extend(
            (0..tree.nodes.len() as u32)
                .filter(|node| finder.is_match(inputs.profile.label(tree.node(*node).label))),
        );
        matches.sort_by(|a, b| tree.node(*b).value.total_cmp(&tree.node(*a).value));
        matches.truncate(MAX_MATCHES);
        self.call_matches = matches;
    }
}

pub(crate) fn column_value(column: Column, totals: &Totals, (left, right): (f64, f64)) -> f64 {
    match column {
        Column::Symbol => 0.,
        Column::Own => totals.own,
        Column::Total => totals.total,
        Column::Baseline => percent(totals.total, left),
        Column::Comparison => percent(totals.total_right, right),
        Column::Diff => change(
            percent(totals.total, left),
            percent(totals.total_right, right),
        ),
    }
}

pub(crate) fn percent(ticks: f64, total: f64) -> f64 {
    if total == 0. {
        0.
    } else {
        (10_000. * ticks / total).round() / 100.
    }
}

pub(crate) fn change(before: f64, after: f64) -> f64 {
    if before == 0. {
        if after == 0. { 0. } else { f64::INFINITY }
    } else {
        (after - before) / before * 100.
    }
}

pub(crate) fn percent_text(value: f64) -> SharedString {
    let mut text = trimmed(value).to_string();
    text.push('%');
    text.into()
}

fn totals(profile: &Profile) -> Vec<Totals> {
    let mut totals = vec![Totals::default(); profile.labels().len()];
    let mut stack: Vec<u32> = Vec::new();
    for sample in profile.samples() {
        stack.truncate(sample.level as usize);
        let recursive = stack.contains(&sample.label);
        let entry = &mut totals[sample.label as usize];
        entry.own += sample.self_value;
        entry.own_right += sample.self_right;
        if !recursive {
            entry.total += sample.value;
            entry.total_right += sample.value_right;
        }
        stack.push(sample.label);
    }
    totals
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn totals_skip_recursive_frames() {
        let profile = Profile::from_folded("main;f;f 4\nmain;f 2");
        let totals = totals(&profile);
        let f = profile
            .labels()
            .iter()
            .position(|label| label == "f")
            .unwrap();
        assert!((totals[f].total - 6.).abs() < f64::EPSILON);
        assert!((totals[f].own - 6.).abs() < f64::EPSILON);
    }

    #[test]
    fn changes_follow_grafana() {
        assert!((percent(25., 200.) - 12.5).abs() < f64::EPSILON);
        assert!((change(10., 15.) - 50.).abs() < f64::EPSILON);
        assert!(change(0., 3.).is_infinite());
    }
}
