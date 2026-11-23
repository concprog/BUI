use std::collections::HashMap;

use blenderui::SharedString;
use blenderui::hooks::Signal;

use crate::data::Revision;

const NONE: u32 = u32::MAX;
const ROOT: &str = "total";

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Sample {
    pub label: u32,
    pub level: u32,
    pub value: f64,
    pub self_value: f64,
    pub value_right: f64,
    pub self_right: f64,
}

#[derive(Debug, Default)]
pub struct Profile {
    labels: Vec<SharedString>,
    samples: Vec<Sample>,
    diff: bool,
}

#[derive(Default)]
struct Labels {
    names: Vec<SharedString>,
    ids: HashMap<SharedString, u32>,
}

impl Labels {
    fn id(&mut self, label: &str) -> u32 {
        if let Some(id) = self.ids.get(label) {
            return *id;
        }
        let name = SharedString::from(label.to_owned());
        let id = self.names.len() as u32;
        self.ids.insert(name.clone(), id);
        self.names.push(name);
        id
    }
}

impl Profile {
    pub fn from_nested_set<S: AsRef<str>>(
        rows: impl IntoIterator<Item = (u32, S, f64, f64)>,
    ) -> Self {
        Self::collect(
            rows.into_iter()
                .map(|(level, label, value, self_value)| (level, label, value, self_value, 0., 0.)),
            false,
        )
    }

    pub fn diff_from_nested_set<S: AsRef<str>>(
        rows: impl IntoIterator<Item = (u32, S, f64, f64, f64, f64)>,
    ) -> Self {
        Self::collect(rows, true)
    }

    pub fn from_folded(folded: &str) -> Self {
        let mut trie = Trie::default();
        for line in folded.lines() {
            let Some((stack, count)) = line.trim().rsplit_once(char::is_whitespace) else {
                continue;
            };
            let Ok(count) = count.parse::<f64>() else {
                continue;
            };
            trie.add(
                stack.split(';').filter(|frame| !frame.is_empty()),
                count,
                0.,
            );
        }
        trie.into_profile(false)
    }

    #[must_use]
    pub fn diff(baseline: &Self, comparison: &Self) -> Self {
        let mut trie = Trie::default();
        trie.merge(baseline, Side::Left);
        trie.merge(comparison, Side::Right);
        trie.into_profile(true)
    }

    fn collect<S: AsRef<str>>(
        rows: impl IntoIterator<Item = (u32, S, f64, f64, f64, f64)>,
        diff: bool,
    ) -> Self {
        let mut labels = Labels::default();
        let samples = rows
            .into_iter()
            .map(
                |(level, label, value, self_value, value_right, self_right)| Sample {
                    label: labels.id(label.as_ref()),
                    level,
                    value,
                    self_value,
                    value_right,
                    self_right,
                },
            )
            .collect();
        Self {
            labels: labels.names,
            samples,
            diff,
        }
    }

    #[must_use]
    pub const fn is_diff(&self) -> bool {
        self.diff
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.samples.len()
    }

    pub(crate) fn labels(&self) -> &[SharedString] {
        &self.labels
    }

    pub(crate) fn samples(&self) -> &[Sample] {
        &self.samples
    }

    pub(crate) fn label(&self, id: u32) -> &SharedString {
        &self.labels[id as usize]
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Side {
    Left,
    Right,
}

struct Node {
    label: u32,
    first: u32,
    last: u32,
    next: u32,
    value: [f64; 2],
    self_value: [f64; 2],
}

struct Trie {
    labels: Labels,
    nodes: Vec<Node>,
    children: HashMap<(u32, u32), u32>,
}

impl Default for Trie {
    fn default() -> Self {
        let mut labels = Labels::default();
        let root = labels.id(ROOT);
        let node = Node {
            label: root,
            first: NONE,
            last: NONE,
            next: NONE,
            value: [0.; 2],
            self_value: [0.; 2],
        };
        Self {
            labels,
            nodes: vec![node],
            children: HashMap::new(),
        }
    }
}

impl Trie {
    fn child(&mut self, parent: u32, label: &str) -> u32 {
        let label = self.labels.id(label);
        if let Some(child) = self.children.get(&(parent, label)) {
            return *child;
        }
        let id = self.nodes.len() as u32;
        self.nodes.push(Node {
            label,
            first: NONE,
            last: NONE,
            next: NONE,
            value: [0.; 2],
            self_value: [0.; 2],
        });
        let parent_node = &mut self.nodes[parent as usize];
        match parent_node.last {
            NONE => parent_node.first = id,
            last => self.nodes[last as usize].next = id,
        }
        self.nodes[parent as usize].last = id;
        self.children.insert((parent, label), id);
        id
    }

    fn add<'a>(&mut self, stack: impl Iterator<Item = &'a str>, left: f64, right: f64) {
        let mut node = 0;
        self.nodes[0].value[0] += left;
        self.nodes[0].value[1] += right;
        for frame in stack {
            node = self.child(node, frame);
            let entry = &mut self.nodes[node as usize];
            entry.value[0] += left;
            entry.value[1] += right;
        }
        let leaf = &mut self.nodes[node as usize];
        leaf.self_value[0] += left;
        leaf.self_value[1] += right;
    }

    fn merge(&mut self, profile: &Profile, side: Side) {
        let slot = usize::from(side == Side::Right);
        let mut path: Vec<u32> = Vec::new();
        for sample in &profile.samples {
            path.truncate(sample.level as usize);
            let parent = path.last().copied().unwrap_or(0);
            let label = profile.label(sample.label);
            let node = if sample.level == 0 && label == ROOT {
                0
            } else {
                self.child(parent, label)
            };
            let entry = &mut self.nodes[node as usize];
            entry.value[slot] += sample.value;
            entry.self_value[slot] += sample.self_value;
            path.push(node);
        }
    }

    fn into_profile(self, diff: bool) -> Profile {
        let mut samples = Vec::with_capacity(self.nodes.len());
        let mut stack = vec![(0u32, 0u32)];
        while let Some((node, level)) = stack.pop() {
            let entry = &self.nodes[node as usize];
            samples.push(Sample {
                label: entry.label,
                level,
                value: entry.value[0],
                self_value: entry.self_value[0],
                value_right: entry.value[1],
                self_right: entry.self_value[1],
            });
            let start = stack.len();
            let mut child = entry.first;
            while child != NONE {
                stack.push((child, level + 1));
                child = self.nodes[child as usize].next;
            }
            stack[start..].reverse();
        }
        Profile {
            labels: self.labels.names,
            samples,
            diff,
        }
    }
}

/// What a flame graph draws: a [`Profile`] (owned: derived again every render) or a signal of
/// one (derived again when it changes).
pub trait ProfileData: 'static {
    /// The profile's signal (`slot` holds an owned one) and its revision.
    #[track_caller]
    fn into_signal(self, slot: Signal<Profile>) -> (Signal<Profile>, Option<Revision>);
}

impl ProfileData for Profile {
    fn into_signal(self, slot: Signal<Profile>) -> (Signal<Profile>, Option<Revision>) {
        slot.update_untracked(|profile| *profile = self);
        (slot, None)
    }
}

impl ProfileData for Signal<Profile> {
    #[track_caller]
    fn into_signal(self, _slot: Signal<Profile>) -> (Signal<Profile>, Option<Revision>) {
        (self, Some(Revision::of(self)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows(profile: &Profile) -> Vec<(u32, &str, f64, f64)> {
        profile
            .samples
            .iter()
            .map(|sample| {
                (
                    sample.level,
                    profile.label(sample.label).as_ref(),
                    sample.value,
                    sample.self_value,
                )
            })
            .collect()
    }

    #[test]
    fn folded_stacks_become_a_nested_set() {
        let profile = Profile::from_folded("main;parse;lex 4\nmain;parse 2\nmain;render 6\n");
        assert_eq!(
            rows(&profile),
            [
                (0, "total", 12., 0.),
                (1, "main", 12., 0.),
                (2, "parse", 6., 2.),
                (3, "lex", 4., 4.),
                (2, "render", 6., 6.),
            ]
        );
    }

    #[test]
    fn diffs_line_up_matching_stacks() {
        let baseline = Profile::from_folded("main;a 4\nmain;b 6");
        let comparison = Profile::from_folded("main;a 8\nmain;c 2");
        let diff = Profile::diff(&baseline, &comparison);
        assert!(diff.is_diff());
        let a = diff
            .samples
            .iter()
            .find(|sample| diff.label(sample.label) == "a")
            .unwrap();
        assert_eq!((a.value, a.value_right), (4., 8.));
        let c = diff
            .samples
            .iter()
            .find(|sample| diff.label(sample.label) == "c")
            .unwrap();
        assert_eq!((c.value, c.value_right), (0., 2.));
    }
}
