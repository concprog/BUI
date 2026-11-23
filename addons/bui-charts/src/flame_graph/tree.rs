use std::collections::HashMap;

use super::profile::Profile;

pub(crate) const NONE: u32 = u32::MAX;
const COLLAPSE_THRESHOLD: f64 = 0.99;

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Node {
    pub start: f64,
    pub value: f64,
    pub right: f64,
    pub level: u32,
    pub label: u32,
    pub parent: u32,
    pub end: u32,
    items: (u32, u32),
}

impl Node {
    pub(crate) fn contains(&self, ticks: f64) -> bool {
        self.start <= ticks && ticks < self.start + self.value
    }
}

#[derive(Default)]
pub(crate) struct Tree {
    pub nodes: Vec<Node>,
    items: Vec<u32>,
    pub depth: u32,
}

impl Tree {
    pub(crate) fn from_profile(profile: &Profile) -> Self {
        let samples = profile.samples();
        let mut nodes: Vec<Node> = Vec::with_capacity(samples.len());
        let mut depth = 0;
        for (index, sample) in samples.iter().enumerate() {
            let id = index as u32;
            let mut parent = id.wrapping_sub(1);
            let mut sibling = NONE;
            while parent != NONE && nodes[parent as usize].level >= sample.level {
                let node = &nodes[parent as usize];
                if node.level == sample.level {
                    sibling = parent;
                }
                parent = node.parent;
            }
            let start = match (sibling, parent) {
                (NONE, NONE) => 0.,
                (NONE, parent) => nodes[parent as usize].start,
                (sibling, _) => nodes[sibling as usize].start + nodes[sibling as usize].value,
            };
            depth = depth.max(sample.level + 1);
            nodes.push(Node {
                start,
                value: sample.value + sample.value_right,
                right: sample.value_right,
                level: sample.level,
                label: sample.label,
                parent,
                end: NONE,
                items: (id, 1),
            });
        }
        close_subtrees(&mut nodes);
        Self {
            nodes,
            items: Vec::new(),
            depth,
        }
    }

    pub(crate) fn total(&self) -> f64 {
        self.nodes.first().map_or(0., |root| root.value)
    }

    pub(crate) fn node(&self, id: u32) -> &Node {
        &self.nodes[id as usize]
    }

    pub(crate) fn items(&self, id: u32) -> impl Iterator<Item = u32> + '_ {
        let (from, len) = self.node(id).items;
        let shared =
            (!self.items.is_empty()).then(|| &self.items[from as usize..(from + len) as usize]);
        let own = shared.is_none().then_some(from);
        shared.into_iter().flatten().copied().chain(own)
    }

    pub(crate) fn children(&self, id: u32) -> impl Iterator<Item = u32> + '_ {
        let end = self.node(id).end;
        let mut child = id + 1;
        std::iter::from_fn(move || {
            (child < end).then(|| {
                let current = child;
                child = self.nodes[current as usize].end;
                current
            })
        })
    }

    pub(crate) fn self_values(&self, id: u32, profile: &Profile) -> (f64, f64) {
        self.items(id).fold((0., 0.), |(own, right), item| {
            let sample = &profile.samples()[item as usize];
            (own + sample.self_value, right + sample.self_right)
        })
    }

    pub(crate) fn hit(
        &self,
        row: u32,
        ticks: f64,
        rows: impl Fn(u32) -> Option<u32>,
    ) -> Option<u32> {
        let mut node = 0;
        loop {
            let current = self.nodes.get(node as usize)?;
            if !current.contains(ticks) {
                return None;
            }
            if rows(node) == Some(row) {
                return Some(node);
            }
            node = self
                .children(node)
                .find(|child| self.node(*child).contains(ticks))?;
        }
    }

    pub(crate) fn groups(&self) -> Vec<u32> {
        let mut heads = vec![NONE; self.nodes.len()];
        for (index, node) in self.nodes.iter().enumerate() {
            let Some(parent) = self.nodes.get(node.parent as usize) else {
                continue;
            };
            let only_child = index as u32 == node.parent + 1 && node.end == parent.end;
            if only_child && node.value > parent.value * COLLAPSE_THRESHOLD {
                let head = match heads[node.parent as usize] {
                    NONE => node.parent,
                    head => head,
                };
                heads[node.parent as usize] = head;
                heads[index] = head;
            }
        }
        heads
    }

    pub(crate) fn sandwich(&self, label: u32, callers: bool) -> Self {
        let roots = self
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, node)| node.label == label)
            .map(|(id, node)| (id as u32, node.value, node.right))
            .collect();
        Merge {
            main: self,
            callers,
        }
        .run(roots)
    }
}

struct Merge<'a> {
    main: &'a Tree,
    callers: bool,
}

impl Merge<'_> {
    fn run(&self, roots: Vec<(u32, f64, f64)>) -> Tree {
        let mut pool = roots;
        let mut queue = vec![(NONE, 0, pool.len(), 0)];
        let mut built: Vec<Node> = Vec::new();
        let mut items = Vec::new();
        let mut ends: Vec<f64> = Vec::new();
        let mut next: Vec<(u32, f64, f64)> = Vec::new();
        let mut order: HashMap<u32, usize> = HashMap::new();
        let mut cursor = 0;
        while let Some(&(parent, from, to, level)) = queue.get(cursor) {
            cursor += 1;
            if from == to {
                continue;
            }
            let id = built.len() as u32;
            let group = &pool[from..to];
            let (value, right) = group.iter().fold((0., 0.), |(value, right), item| {
                (value + item.1, right + item.2)
            });
            let first = items.len() as u32;
            for (node, _, _) in group {
                items.extend(self.main.items(*node));
            }
            let start = match parent {
                NONE => 0.,
                parent => {
                    let start = ends[parent as usize];
                    ends[parent as usize] += value;
                    start
                }
            };
            built.push(Node {
                start,
                value,
                right,
                level,
                label: self.main.node(group[0].0).label,
                parent,
                end: NONE,
                items: (first, items.len() as u32 - first),
            });
            ends.push(start);
            next.clear();
            for &(node, value, right) in group {
                if self.callers {
                    let parent = self.main.node(node).parent;
                    if parent != NONE {
                        next.push((parent, value, right));
                    }
                } else {
                    next.extend(self.main.children(node).map(|child| {
                        let child_node = self.main.node(child);
                        (child, child_node.value, child_node.right)
                    }));
                }
            }
            order.clear();
            for (position, (node, _, _)) in next.iter().enumerate() {
                order.entry(self.main.node(*node).label).or_insert(position);
            }
            next.sort_by_key(|(node, _, _)| order[&self.main.node(*node).label]);
            let mut start = 0;
            while start < next.len() {
                let label = self.main.node(next[start].0).label;
                let len = next[start..]
                    .iter()
                    .take_while(|(node, _, _)| self.main.node(*node).label == label)
                    .count();
                let offset = pool.len();
                pool.extend_from_slice(&next[start..start + len]);
                queue.push((id, offset, pool.len(), level + 1));
                start += len;
            }
        }
        depth_first(&built, items, self.callers)
    }
}

fn close_subtrees(nodes: &mut [Node]) {
    let len = nodes.len() as u32;
    for id in 0..len {
        let mut open = id.wrapping_sub(1);
        while open != NONE && nodes[open as usize].level >= nodes[id as usize].level {
            nodes[open as usize].end = id;
            open = nodes[open as usize].parent;
        }
    }
    let mut open = len.wrapping_sub(1);
    while open != NONE {
        nodes[open as usize].end = len;
        open = nodes[open as usize].parent;
    }
}

fn depth_first(built: &[Node], items: Vec<u32>, reverse_levels: bool) -> Tree {
    let len = built.len();
    let mut first_child = vec![NONE; len];
    let mut last_child = vec![NONE; len];
    for (id, node) in built.iter().enumerate().rev() {
        if node.parent != NONE {
            let parent = node.parent as usize;
            first_child[parent] = id as u32;
            if last_child[parent] == NONE {
                last_child[parent] = id as u32;
            }
        }
    }
    let mut mapped = vec![NONE; len];
    let mut nodes = Vec::with_capacity(len);
    let mut stack = Vec::new();
    if len > 0 {
        stack.push(0u32);
    }
    while let Some(old) = stack.pop() {
        mapped[old as usize] = nodes.len() as u32;
        let mut node = built[old as usize];
        node.parent = if node.parent == NONE {
            NONE
        } else {
            mapped[node.parent as usize]
        };
        nodes.push(node);
        let (first, last) = (first_child[old as usize], last_child[old as usize]);
        if first != NONE {
            stack.extend(
                (first..=last)
                    .rev()
                    .filter(|child| built[*child as usize].parent == old),
            );
        }
    }
    close_subtrees(&mut nodes);
    let depth = nodes.iter().map(|node| node.level + 1).max().unwrap_or(0);
    if reverse_levels {
        for node in &mut nodes {
            node.level = depth - 1 - node.level;
        }
    }
    Tree {
        nodes,
        items,
        depth,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile() -> Profile {
        Profile::from_folded("main;a;x 4\nmain;b;x 6\nmain;b;y 2\n")
    }

    #[test]
    fn nested_sets_lay_out_siblings_side_by_side() {
        let profile = profile();
        let tree = Tree::from_profile(&profile);
        let label = |id: u32| profile.label(tree.node(id).label).to_string();
        let children: Vec<String> = tree.children(1).map(label).collect();
        assert_eq!(children, ["a", "b"]);
        let b = tree.children(1).nth(1).unwrap();
        assert!((tree.node(b).start - 4.).abs() < f64::EPSILON);
        assert_eq!(tree.depth, 4);
        assert_eq!(
            tree.hit(3, 5., |id| Some(tree.node(id).level)).map(label),
            Some("x".into())
        );
    }

    #[test]
    fn sandwich_merges_every_occurrence() {
        let profile = profile();
        let tree = Tree::from_profile(&profile);
        let x = tree.node(3).label;
        let callees = tree.sandwich(x, false);
        assert!((callees.total() - 10.).abs() < f64::EPSILON);
        let callers = tree.sandwich(x, true);
        assert!((callers.total() - 10.).abs() < f64::EPSILON);
        assert_eq!(callers.node(0).level, callers.depth - 1);
        let parents: Vec<&str> = callers
            .children(0)
            .map(|id| profile.label(callers.node(id).label).as_ref())
            .collect();
        assert_eq!(parents, ["a", "b"]);
    }
}
