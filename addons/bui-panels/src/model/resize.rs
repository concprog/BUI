use std::collections::HashMap;

use super::{
    tree::{Entries, Kind, Metrics, PixelMetrics, layout_rects, sum},
    types::{Axis, Id, Node, Rect, SplitNode, StageNode},
};

struct Group {
    node_id: Id,
    ids: Vec<usize>,
    cramped: bool,
}

pub struct BoundaryDrag {
    pub axis: Axis,
    pub start: f64,
    pub min: f64,
    pub max: f64,
    root: Node,
    metrics: PixelMetrics,
    groups: Vec<Group>,
    pos: Vec<f64>,
    ahead: Vec<f64>,
    behind: Vec<f64>,
    handle: usize,
    reach_min: f64,
    reach_max: f64,
    beside: Id,
}

struct Graph<'a> {
    entries: &'a Entries,
    metrics: &'a PixelMetrics,
    axis: Axis,
    pixels: f64,
    pos: Vec<f64>,
    next: Vec<Vec<(usize, f64)>>,
    prev: Vec<Vec<(usize, f64)>>,
    groups: Vec<Group>,
}

impl Graph<'_> {
    fn position(&mut self, at: f64) -> usize {
        self.pos.push(at);
        self.next.push(Vec::new());
        self.prev.push(Vec::new());
        self.pos.len() - 1
    }

    fn at_least(&mut self, from: usize, to: usize, gap: f64) {
        self.next[from].push((to, gap));
        self.prev[to].push((from, gap));
    }

    fn scale_of(&self, id: &str) -> f64 {
        self.entries.get(id).map_or(1.0, |e| e.scale)
    }

    fn min_of(&self, node: &Node) -> f64 {
        self.metrics.min(node, self.axis) * self.scale_of(node.id()) / self.pixels
    }

    fn cramped_along(&self, node: &SplitNode) -> bool {
        let own = self.scale_of(&node.id);
        let factor = self.scale_of(node.children[0].id()) / own;
        if factor >= 1.0 - 1e-9 {
            return false;
        }
        let rect = self.entries.rect(&node.id).unwrap_or(Rect::UNIT);
        let size = rect.len(self.axis) * self.pixels / own;
        let need: f64 = node
            .children
            .iter()
            .map(|c| self.metrics.min(c, self.axis))
            .sum();
        size / need <= factor + 1e-9
    }

    fn walk(&mut self, node: &Node, a: usize, b: usize) {
        match node {
            Node::Panel(_) => {
                let gap = self.min_of(node);
                self.at_least(a, b, gap);
            }
            Node::Stage(s) => {
                if let Some(child) = &s.child {
                    self.walk(child, a, b);
                } else {
                    let gap = self.min_of(node);
                    self.at_least(a, b, gap);
                }
            }
            Node::Split(s) => {
                if s.axis != self.axis {
                    for child in &s.children {
                        self.walk(child, a, b);
                    }
                    return;
                }
                let mut ids = vec![a];
                for child in s.children.iter().skip(1) {
                    let lo = self
                        .entries
                        .rect(child.id())
                        .map_or(0.0, |r| r.lo(self.axis));
                    ids.push(self.position(lo));
                }
                ids.push(b);
                let cramped = self.cramped_along(s);
                self.groups.push(Group {
                    node_id: s.id.clone(),
                    ids: ids.clone(),
                    cramped,
                });
                for (i, child) in s.children.iter().enumerate() {
                    self.walk(child, ids[i], ids[i + 1]);
                }
            }
        }
    }
}

fn topological(next: &[Vec<(usize, f64)>]) -> Vec<usize> {
    let mut incoming = vec![0usize; next.len()];
    for edges in next {
        for &(to, _) in edges {
            incoming[to] += 1;
        }
    }
    let mut queue: std::collections::VecDeque<usize> = incoming
        .iter()
        .enumerate()
        .filter(|(_, n)| **n == 0)
        .map(|(v, _)| v)
        .collect();
    let mut order = Vec::with_capacity(next.len());
    while let Some(u) = queue.pop_front() {
        order.push(u);
        for &(to, _) in &next[u] {
            incoming[to] -= 1;
            if incoming[to] == 0 {
                queue.push_back(to);
            }
        }
    }
    order
}

fn reweigh(node: &Node, weights: &HashMap<Id, Vec<f64>>) -> Node {
    match node {
        Node::Panel(_) => node.clone(),
        Node::Stage(s) => Node::Stage(StageNode {
            id: s.id.clone(),
            child: s.child.as_ref().map(|c| Box::new(reweigh(c, weights))),
        }),
        Node::Split(s) => Node::Split(SplitNode {
            id: s.id.clone(),
            axis: s.axis,
            weights: weights
                .get(&s.id)
                .cloned()
                .unwrap_or_else(|| s.weights.clone()),
            children: s.children.iter().map(|c| reweigh(c, weights)).collect(),
        }),
    }
}

#[must_use]
pub fn drag_boundary(
    root: &Node,
    metrics: PixelMetrics,
    split_id: &str,
    index: usize,
) -> Option<BoundaryDrag> {
    let entries = layout_rects(Some(root), Some(&metrics));
    let dragged = entries.get(split_id)?;
    let (axis, child_ids) = match &dragged.kind {
        Kind::Split { axis, children, .. } if index + 1 < children.len() => {
            (*axis, children.clone())
        }
        _ => return None,
    };
    let pixels = match axis {
        Axis::X => metrics.width,
        Axis::Y => metrics.height,
    };

    let mut graph = Graph {
        entries: &entries,
        metrics: &metrics,
        axis,
        pixels,
        pos: Vec::new(),
        next: Vec::new(),
        prev: Vec::new(),
        groups: Vec::new(),
    };
    let root_rect = entries.rect(root.id())?;
    let first = graph.position(root_rect.lo(axis));
    let last = graph.position(root_rect.lo(axis) + root_rect.len(axis));
    graph.walk(root, first, last);
    let handle = graph
        .groups
        .iter()
        .find(|g| g.node_id.as_ref() == split_id)?
        .ids[index + 1];
    let Graph {
        pos,
        next,
        prev,
        groups,
        ..
    } = graph;

    let order = topological(&next);
    let mut ahead = vec![f64::NEG_INFINITY; pos.len()];
    ahead[handle] = 0.0;
    for &u in &order {
        if ahead[u] > f64::NEG_INFINITY {
            for &(to, gap) in &next[u] {
                ahead[to] = ahead[to].max(ahead[u] + gap);
            }
        }
    }
    let mut behind = vec![f64::NEG_INFINITY; pos.len()];
    behind[handle] = 0.0;
    for &u in order.iter().rev() {
        if behind[u] > f64::NEG_INFINITY {
            for &(from, gap) in &prev[u] {
                behind[from] = behind[from].max(behind[u] + gap);
            }
        }
    }

    let start = pos[handle];
    let reach_max = start.max(pos[last] - ahead[last]);
    let reach_min = start.min(pos[first] + behind[first]);

    let mut drag = BoundaryDrag {
        axis,
        start,
        min: start,
        max: start,
        root: root.clone(),
        metrics,
        groups,
        pos,
        ahead,
        behind,
        handle,
        reach_min,
        reach_max,
        beside: child_ids[index + 1].clone(),
    };
    drag.min = if reach_min == start {
        start
    } else {
        start.min(drag.landed(drag.solve(reach_min).as_ref()))
    };
    drag.max = if reach_max == start {
        start
    } else {
        start.max(drag.landed(drag.solve(reach_max).as_ref()))
    };
    Some(drag)
}

impl BoundaryDrag {
    fn solve(&self, p: f64) -> Option<Node> {
        let mut moved: Vec<f64> = (0..self.pos.len())
            .map(|v| {
                let at = self.pos[v];
                if p >= self.start {
                    if self.ahead[v] > f64::NEG_INFINITY {
                        at.max(p + self.ahead[v])
                    } else {
                        at
                    }
                } else if self.behind[v] > f64::NEG_INFINITY {
                    at.min(p - self.behind[v])
                } else {
                    at
                }
            })
            .collect();
        moved[self.handle] = p;

        let mut weights = HashMap::new();
        for g in &self.groups {
            let before: Vec<f64> = g
                .ids
                .windows(2)
                .map(|w| self.pos[w[1]] - self.pos[w[0]])
                .collect();
            let after: Vec<f64> = g
                .ids
                .windows(2)
                .map(|w| moved[w[1]] - moved[w[0]])
                .collect();
            let (total, was) = (sum(&after), sum(&before));
            if after
                .iter()
                .zip(&before)
                .all(|(a, b)| (a / total - b / was).abs() < 1e-12)
            {
                continue;
            }
            if g.cramped
                && g.ids[1..g.ids.len() - 1]
                    .iter()
                    .all(|&id| moved[id] == self.pos[id])
            {
                continue;
            }
            weights.insert(g.node_id.clone(), after.iter().map(|a| a / total).collect());
        }
        (!weights.is_empty()).then(|| reweigh(&self.root, &weights))
    }

    fn landed(&self, layout: Option<&Node>) -> f64 {
        match layout {
            None => self.start,
            Some(node) => layout_rects(Some(node), Some(&self.metrics))
                .rect(&self.beside)
                .map_or(self.start, |r| r.lo(self.axis)),
        }
    }

    #[must_use]
    pub fn to(&self, target: f64) -> Node {
        let p = target.clamp(self.min, self.max);
        let guess = self.solve(p.clamp(self.reach_min, self.reach_max));
        if (self.landed(guess.as_ref()) - p).abs() < 1e-9 {
            return guess.unwrap_or_else(|| self.root.clone());
        }
        let (mut below, mut above) = if p >= self.start {
            (self.start, self.reach_max)
        } else {
            (self.reach_min, self.start)
        };
        for _ in 0..50 {
            if above - below <= 1e-12 {
                break;
            }
            let mid = f64::midpoint(below, above);
            if self.landed(self.solve(mid).as_ref()) < p {
                below = mid;
            } else {
                above = mid;
            }
        }
        self.solve(if p >= self.start { above } else { below })
            .unwrap_or_else(|| self.root.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::types::PanelNode;
    use std::sync::Arc;

    fn id(s: &str) -> Id {
        Arc::from(s)
    }
    fn panel(name: &str) -> Node {
        Node::Panel(PanelNode {
            id: id(name),
            views: vec![id("v")],
            selected: id("v"),
        })
    }
    fn metrics() -> PixelMetrics {
        PixelMetrics {
            width: 1000.0,
            height: 600.0,
            gap: 6.0,
            tabbar_height: 34.0,
            panel_min_width: 80.0,
        }
    }

    #[test]
    fn dragging_moves_the_divider() {
        let root = Node::Split(SplitNode {
            id: id("r"),
            axis: Axis::X,
            weights: vec![0.5, 0.5],
            children: vec![panel("a"), panel("b")],
        });
        let drag = drag_boundary(&root, metrics(), "r", 0).unwrap();
        let moved = drag.to(0.7);
        let rects = layout_rects(Some(&moved), None);
        assert!((rects.rect("a").unwrap().w - 0.7).abs() < 1e-6);
    }

    #[test]
    fn the_divider_stops_at_the_neighbours_minimum() {
        let root = Node::Split(SplitNode {
            id: id("r"),
            axis: Axis::X,
            weights: vec![0.5, 0.5],
            children: vec![panel("a"), panel("b")],
        });
        let drag = drag_boundary(&root, metrics(), "r", 0).unwrap();
        assert!(drag.max < 1.0 && drag.max > 0.9);
    }
}
