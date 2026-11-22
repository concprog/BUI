use std::collections::HashMap;

use super::types::{Axis, Edge, Id, Node, PanelNode, Rect, SplitNode, StageNode};

#[must_use]
pub fn sum(values: &[f64]) -> f64 {
    values.iter().sum()
}

#[derive(Clone, Debug, PartialEq)]
pub enum Kind {
    Panel,
    Stage {
        child: Option<Id>,
    },
    Split {
        axis: Axis,
        weights: Vec<f64>,
        children: Vec<Id>,
    },
}

#[derive(Clone, Debug)]
pub struct Entry {
    pub id: Id,
    pub kind: Kind,
    pub rect: Rect,
    pub parent: Option<Id>,
    pub scale: f64,
    pub range: bool,
}

#[derive(Clone, Debug, Default)]
pub struct Entries {
    map: HashMap<Id, Entry>,
    order: Vec<Id>,
}

impl Entries {
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&Entry> {
        self.map.get(id)
    }
    #[must_use]
    pub fn contains(&self, id: &str) -> bool {
        self.map.contains_key(id)
    }
    #[must_use]
    pub fn rect(&self, id: &str) -> Option<Rect> {
        self.map.get(id).map(|e| e.rect)
    }
    #[must_use]
    pub fn len(&self) -> usize {
        self.map.len()
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
    pub fn insert(&mut self, entry: Entry) {
        if !self.map.contains_key(&entry.id) {
            self.order.push(entry.id.clone());
        }
        self.map.insert(entry.id.clone(), entry);
    }
    pub fn iter(&self) -> impl Iterator<Item = &Entry> {
        self.order.iter().filter_map(|id| self.map.get(id))
    }
    #[must_use]
    pub fn ids(&self) -> &[Id] {
        &self.order
    }

    #[must_use]
    pub fn leaf_ids(&self, id: &str) -> Vec<Id> {
        let mut out = Vec::new();
        self.collect_leaves(id, &mut out);
        out
    }
    fn collect_leaves(&self, id: &str, out: &mut Vec<Id>) {
        let Some(entry) = self.map.get(id) else {
            return;
        };
        match &entry.kind {
            Kind::Panel | Kind::Stage { child: None } => out.push(entry.id.clone()),
            Kind::Stage { child: Some(child) } => self.collect_leaves(child, out),
            Kind::Split { children, .. } => {
                for c in children {
                    self.collect_leaves(c, out);
                }
            }
        }
    }
    #[must_use]
    pub fn contains_node(&self, ancestor: &str, id: &str) -> bool {
        if ancestor == id {
            return true;
        }
        match self.map.get(ancestor).map(|e| &e.kind) {
            Some(Kind::Stage { child: Some(c) }) => self.contains_node(c, id),
            Some(Kind::Split { children, .. }) => {
                children.iter().any(|c| self.contains_node(c, id))
            }
            _ => false,
        }
    }
}

pub trait Metrics {
    fn width(&self) -> f64;
    fn height(&self) -> f64;
    fn min(&self, node: &Node, axis: Axis) -> f64;
}

#[derive(Clone, Copy, Debug)]
pub struct PixelMetrics {
    pub width: f64,
    pub height: f64,
    pub gap: f64,
    pub tabbar_height: f64,
    pub panel_min_width: f64,
}

impl Metrics for PixelMetrics {
    fn width(&self) -> f64 {
        self.width
    }
    fn height(&self) -> f64 {
        self.height
    }
    fn min(&self, node: &Node, axis: Axis) -> f64 {
        let base = match node {
            Node::Stage(StageNode { child: None, .. }) => 120.0,
            _ if axis == Axis::X => self.panel_min_width,
            _ => self.tabbar_height + 28.0,
        };
        base + self.gap
    }
}

fn kind_of(node: &Node, weights: Option<Vec<f64>>) -> Kind {
    match node {
        Node::Panel(_) => Kind::Panel,
        Node::Stage(s) => Kind::Stage {
            child: s.child.as_ref().map(|c| c.id().clone()),
        },
        Node::Split(s) => Kind::Split {
            axis: s.axis,
            weights: weights.unwrap_or_else(|| s.weights.clone()),
            children: s.children.iter().map(|c| c.id().clone()).collect(),
        },
    }
}

#[must_use]
pub fn layout_rects(root: Option<&Node>, metrics: Option<&dyn Metrics>) -> Entries {
    let mut out = Entries::default();
    if let Some(node) = root {
        match metrics {
            None => by_weights(node, Rect::UNIT, None, &mut out),
            Some(m) => with_minimums(
                node,
                Rect::UNIT,
                (m.width(), m.height()),
                1.0,
                None,
                m,
                &mut out,
            ),
        }
    }
    out
}

fn slice(rect: Rect, axis: Axis, offset: f64, share: f64) -> Rect {
    match axis {
        Axis::X => Rect {
            x: rect.x + offset * rect.w,
            w: rect.w * share,
            ..rect
        },
        Axis::Y => Rect {
            y: rect.y + offset * rect.h,
            h: rect.h * share,
            ..rect
        },
    }
}

fn by_weights(node: &Node, rect: Rect, parent: Option<&Id>, out: &mut Entries) {
    out.insert(Entry {
        id: node.id().clone(),
        kind: kind_of(node, None),
        rect,
        parent: parent.cloned(),
        scale: 1.0,
        range: false,
    });
    match node {
        Node::Stage(s) => {
            if let Some(child) = &s.child {
                by_weights(child, rect, Some(&s.id), out);
            }
        }
        Node::Split(s) => {
            let total = sum(&s.weights);
            let mut offset = 0.0;
            for (child, weight) in s.children.iter().zip(&s.weights) {
                let share = weight / total;
                by_weights(child, slice(rect, s.axis, offset, share), Some(&s.id), out);
                offset += share;
            }
        }
        Node::Panel(_) => {}
    }
}

fn with_minimums(
    node: &Node,
    rect: Rect,
    size: (f64, f64),
    scale: f64,
    parent: Option<&Id>,
    metrics: &dyn Metrics,
    out: &mut Entries,
) {
    let leaf = |out: &mut Entries| {
        out.insert(Entry {
            id: node.id().clone(),
            kind: kind_of(node, None),
            rect,
            parent: parent.cloned(),
            scale,
            range: false,
        });
    };
    match node {
        Node::Panel(_) => leaf(out),
        Node::Stage(s) => {
            leaf(out);
            if let Some(child) = &s.child {
                with_minimums(child, rect, size, scale, Some(&s.id), metrics, out);
            }
        }
        Node::Split(s) => {
            let along_x = s.axis == Axis::X;
            let pick = |(w, h): (f64, f64)| if along_x { w } else { h };
            let cross = s.axis.cross();
            let mins: Vec<f64> = s.children.iter().map(|c| metrics.min(c, s.axis)).collect();
            let need_along = sum(&mins);
            let need_across = s
                .children
                .iter()
                .map(|c| metrics.min(c, cross))
                .fold(0.0, f64::max);
            let size_along = pick(size);
            let size_across = if along_x { size.1 } else { size.0 };
            let s_factor = 1.0_f64
                .min(if need_along > 0.0 {
                    size_along / need_along
                } else {
                    1.0
                })
                .min(if need_across > 0.0 {
                    size_across / need_across
                } else {
                    1.0
                });
            let natural = (size.0 / s_factor, size.1 / s_factor);
            let natural_along = pick(natural);
            let extents = distribute(natural_along, &s.weights, &mins);
            let fractions: Vec<f64> = extents.iter().map(|e| e / natural_along).collect();

            out.insert(Entry {
                id: s.id.clone(),
                kind: kind_of(node, Some(fractions.clone())),
                rect,
                parent: parent.cloned(),
                scale,
                range: false,
            });
            let mut offset = 0.0;
            for (i, child) in s.children.iter().enumerate() {
                let child_size = if along_x {
                    (extents[i], natural.1)
                } else {
                    (natural.0, extents[i])
                };
                with_minimums(
                    child,
                    slice(rect, s.axis, offset, fractions[i]),
                    child_size,
                    scale * s_factor,
                    Some(&s.id),
                    metrics,
                    out,
                );
                offset += fractions[i];
            }
        }
    }
}

#[must_use]
pub fn distribute(total: f64, weights: &[f64], mins: &[f64]) -> Vec<f64> {
    let mut fixed = vec![false; weights.len()];
    loop {
        let free: Vec<usize> = (0..weights.len()).filter(|&i| !fixed[i]).collect();
        let left = total
            - (0..weights.len())
                .filter(|&i| fixed[i])
                .map(|i| mins[i])
                .sum::<f64>();
        let weight: f64 = free.iter().map(|&i| weights[i]).sum();
        let extents: Vec<f64> = (0..weights.len())
            .map(|i| {
                if fixed[i] {
                    mins[i]
                } else if weight > 0.0 {
                    left * weights[i] / weight
                } else {
                    left / free.len() as f64
                }
            })
            .collect();
        let under: Vec<usize> = free
            .into_iter()
            .filter(|&i| extents[i] < mins[i] - 1e-9)
            .collect();
        if under.is_empty() {
            return extents;
        }
        for i in under {
            fixed[i] = true;
        }
    }
}

#[must_use]
pub fn find_node<'a>(node: Option<&'a Node>, id: &str) -> Option<&'a Node> {
    let node = node?;
    if node.id().as_ref() == id {
        return Some(node);
    }
    match node {
        Node::Panel(_) => None,
        Node::Stage(s) => find_node(s.child.as_deref(), id),
        Node::Split(s) => s.children.iter().find_map(|c| find_node(Some(c), id)),
    }
}

#[must_use]
pub fn panels_of(node: Option<&Node>) -> Vec<&PanelNode> {
    let mut out = Vec::new();
    collect_panels(node, &mut out);
    out
}
fn collect_panels<'a>(node: Option<&'a Node>, out: &mut Vec<&'a PanelNode>) {
    match node {
        None => {}
        Some(Node::Panel(p)) => out.push(p),
        Some(Node::Stage(s)) => collect_panels(s.child.as_deref(), out),
        Some(Node::Split(s)) => s.children.iter().for_each(|c| collect_panels(Some(c), out)),
    }
}

#[must_use]
pub fn find_stage(node: Option<&Node>) -> Option<&StageNode> {
    match node? {
        Node::Stage(s) => Some(s),
        Node::Split(s) => s.children.iter().find_map(|c| find_stage(Some(c))),
        Node::Panel(_) => None,
    }
}

#[must_use]
pub fn leaf_ids(node: Option<&Node>) -> Vec<Id> {
    match node {
        None => Vec::new(),
        Some(Node::Panel(p)) => vec![p.id.clone()],
        Some(Node::Stage(s)) => match &s.child {
            Some(c) => leaf_ids(Some(c)),
            None => vec![s.id.clone()],
        },
        Some(Node::Split(s)) => s.children.iter().flat_map(|c| leaf_ids(Some(c))).collect(),
    }
}

#[must_use]
pub fn contains_node(node: &Node, id: &str) -> bool {
    find_node(Some(node), id).is_some()
}

pub fn normalize(node: Node) -> Option<Node> {
    match node {
        Node::Panel(p) => (!p.views.is_empty()).then_some(Node::Panel(p)),
        Node::Stage(mut s) => {
            s.child = s.child.and_then(|c| normalize(*c)).map(Box::new);
            Some(Node::Stage(s))
        }
        Node::Split(s) => {
            let SplitNode {
                id,
                axis,
                weights,
                children,
            } = s;
            let mut out_children: Vec<Node> = Vec::with_capacity(children.len());
            let mut out_weights: Vec<f64> = Vec::with_capacity(children.len());
            for (i, child) in children.into_iter().enumerate() {
                let raw = weights.get(i).copied().unwrap_or(f64::NAN);
                let weight = if raw.is_finite() && raw > 0.0 {
                    raw
                } else {
                    1e-3
                };
                let Some(next) = normalize(child) else {
                    continue;
                };
                match next {
                    Node::Split(inner) if inner.axis == axis => {
                        let total = sum(&inner.weights);
                        for (grandchild, w) in inner.children.into_iter().zip(inner.weights) {
                            out_children.push(grandchild);
                            out_weights.push(weight * w / total);
                        }
                    }
                    other => {
                        out_children.push(other);
                        out_weights.push(weight);
                    }
                }
            }
            match out_children.len() {
                0 => None,
                1 => out_children.pop(),
                _ => {
                    let total = sum(&out_weights);
                    let weights = out_weights.iter().map(|w| w / total).collect();
                    Some(Node::Split(SplitNode {
                        id,
                        axis,
                        weights,
                        children: out_children,
                    }))
                }
            }
        }
    }
}

#[must_use]
pub fn remove_node(root: Option<Node>, id: &str) -> Option<Node> {
    let root = root?;
    if find_node(Some(&root), id).is_none() {
        return Some(root);
    }
    remove_inner(root, id)
}

fn remove_inner(root: Node, id: &str) -> Option<Node> {
    if root.id().as_ref() == id {
        return root.is_stage().then_some(root);
    }
    match root {
        Node::Panel(_) => Some(root),
        Node::Stage(mut s) => {
            s.child = match s.child.take() {
                Some(child) if child.id().as_ref() == id => None,
                Some(child) => remove_inner(*child, id).map(Box::new),
                None => None,
            };
            Some(Node::Stage(s))
        }
        Node::Split(mut s) => {
            if let Some(index) = s.children.iter().position(|c| c.id().as_ref() == id) {
                if s.children[index].is_stage() {
                    return Some(Node::Split(s));
                }
                s.children.remove(index);
                let removed = s.weights.remove(index);
                if s.children.is_empty() {
                    return None;
                }
                let at = index.min(s.weights.len() - 1);
                s.weights[at] += removed;
                return normalize(Node::Split(s));
            }
            let SplitNode {
                id: sid,
                axis,
                weights,
                children,
            } = s;
            let mut out_children = Vec::with_capacity(children.len());
            let mut out_weights: Vec<f64> = Vec::with_capacity(children.len());
            for (child, weight) in children.into_iter().zip(weights) {
                match remove_inner(child, id) {
                    Some(next) => {
                        out_children.push(next);
                        out_weights.push(weight);
                    }
                    None => {
                        if let Some(last) = out_weights.last_mut() {
                            *last += weight;
                        }
                    }
                }
            }
            if out_children.is_empty() {
                return None;
            }
            normalize(Node::Split(SplitNode {
                id: sid,
                axis,
                weights: out_weights,
                children: out_children,
            }))
        }
    }
}

#[must_use]
#[allow(clippy::needless_pass_by_value)] // zopra-panels' signature
pub fn insert_beside(
    root: Option<Node>,
    target: &str,
    incoming: Node,
    edge: Edge,
    split_id: Id,
    share: f64,
) -> Node {
    let Some(root) = root else { return incoming };
    insert_rec(
        root,
        target,
        &incoming,
        edge,
        &split_id,
        share.clamp(0.1, 0.9),
    )
}

fn insert_rec(
    root: Node,
    target: &str,
    incoming: &Node,
    edge: Edge,
    split_id: &Id,
    share: f64,
) -> Node {
    if root.id().as_ref() == target {
        let (weights, children) = if edge.before() {
            (vec![share, 1.0 - share], vec![incoming.clone(), root])
        } else {
            (vec![1.0 - share, share], vec![root, incoming.clone()])
        };
        let wrapped = Node::Split(SplitNode {
            id: split_id.clone(),
            axis: edge.axis(),
            weights,
            children,
        });
        return normalize(wrapped).expect("a split of two non-empty nodes never normalizes away");
    }
    match root {
        Node::Panel(_) => root,
        Node::Stage(mut s) => {
            s.child = s
                .child
                .take()
                .map(|c| Box::new(insert_rec(*c, target, incoming, edge, split_id, share)));
            Node::Stage(s)
        }
        Node::Split(mut s) => {
            s.children = s
                .children
                .into_iter()
                .map(|c| insert_rec(c, target, incoming, edge, split_id, share))
                .collect();
            normalize(Node::Split(s)).expect("a split keeps its children")
        }
    }
}

#[must_use]
#[allow(clippy::needless_pass_by_value)] // zopra-panels' signature
pub fn replace_node(root: Option<Node>, id: &str, replacement: Node) -> Option<Node> {
    let root = root?;
    if find_node(Some(&root), id).is_none() {
        return Some(root);
    }
    Some(replace_rec(root, id, &replacement))
}

fn replace_rec(root: Node, id: &str, replacement: &Node) -> Node {
    if root.id().as_ref() == id {
        return replacement.clone();
    }
    match root {
        Node::Panel(_) => root,
        Node::Stage(mut s) => {
            s.child = s
                .child
                .take()
                .map(|c| Box::new(replace_rec(*c, id, replacement)));
            Node::Stage(s)
        }
        Node::Split(mut s) => {
            s.children = s
                .children
                .into_iter()
                .map(|c| replace_rec(c, id, replacement))
                .collect();
            Node::Split(s)
        }
    }
}

pub fn resize_boundary(
    split: &SplitNode,
    index: usize,
    position: f64,
    min: impl Fn(usize) -> f64,
) -> SplitNode {
    let total = sum(&split.weights);
    let mut normalized: Vec<f64> = split.weights.iter().map(|w| w / total).collect();
    let start = sum(&normalized[..index]);
    let pair = normalized[index] + normalized[index + 1];
    let lo = min(index).min(pair / 2.0);
    let hi = pair - min(index + 1).min(pair / 2.0);
    let left = (position - start).clamp(lo, hi);
    normalized[index] = left;
    normalized[index + 1] = pair - left;
    SplitNode {
        weights: normalized,
        ..split.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn id(s: &str) -> Id {
        Arc::from(s)
    }
    use std::sync::Arc;

    pub fn panel(name: &str) -> Node {
        Node::Panel(PanelNode {
            id: id(name),
            views: vec![id(&format!("{name}-v"))],
            selected: id(&format!("{name}-v")),
        })
    }
    pub fn row(name: &str, children: Vec<Node>, weights: Vec<f64>) -> Node {
        Node::Split(SplitNode {
            id: id(name),
            axis: Axis::X,
            weights,
            children,
        })
    }

    #[test]
    fn layout_follows_weights() {
        let root = row("r", vec![panel("a"), panel("b")], vec![1.0, 3.0]);
        let e = layout_rects(Some(&root), None);
        assert!((e.rect("a").unwrap().w - 0.25).abs() < 1e-12);
        assert!((e.rect("b").unwrap().x - 0.25).abs() < 1e-12);
    }

    #[test]
    fn normalize_flattens_aligned_splits() {
        let inner = row("inner", vec![panel("b"), panel("c")], vec![1.0, 1.0]);
        let root = row("r", vec![panel("a"), inner], vec![1.0, 1.0]);
        let Some(Node::Split(s)) = normalize(root) else {
            panic!("expected a split")
        };
        assert_eq!(s.children.len(), 3);
        assert!((s.weights[1] - 0.25).abs() < 1e-12);
    }

    #[test]
    fn removing_a_child_gives_its_share_to_the_next_sibling() {
        let root = row(
            "r",
            vec![panel("a"), panel("b"), panel("c")],
            vec![0.25, 0.25, 0.5],
        );
        let Some(Node::Split(s)) = remove_node(Some(root), "a") else {
            panic!("expected a split")
        };
        assert_eq!(s.children.len(), 2);
        assert!((s.weights[0] - 0.5).abs() < 1e-12);
    }

    #[test]
    fn minimums_are_respected() {
        struct Fixed;
        impl Metrics for Fixed {
            fn width(&self) -> f64 {
                400.0
            }
            fn height(&self) -> f64 {
                300.0
            }
            fn min(&self, _: &Node, _: Axis) -> f64 {
                150.0
            }
        }
        let root = row("r", vec![panel("a"), panel("b")], vec![1.0, 9.0]);
        let e = layout_rects(Some(&root), Some(&Fixed));
        assert!(e.rect("a").unwrap().w * 400.0 >= 150.0 - 1e-6);
    }
}
