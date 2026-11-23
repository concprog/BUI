use std::collections::{HashMap, VecDeque};

use super::data::{GraphEdge, GraphNode, NodeLayout};
use super::layout::{Position, force, grid, layered};

const MARKER_COUNT_LIMIT: usize = 101;

#[derive(Default)]
pub(crate) struct Graph {
    key: Option<u64>,
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub links: Vec<Option<(usize, usize)>>,
    pub around: Vec<Vec<usize>>,
    pub incoming: Vec<usize>,
    positions: Vec<Position>,
    shape: Option<Shape>,
    pub view: Visible,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Shape {
    pub layout: NodeLayout,
    pub width: u32,
    pub sort: Option<(usize, bool)>,
    pub limit: usize,
    pub root: Option<usize>,
}

impl Graph {
    pub(crate) fn fresh(&self, key: Option<u64>) -> bool {
        key.is_some() && self.key == key
    }

    pub(crate) fn load(&mut self, key: Option<u64>, nodes: &[GraphNode], edges: &[GraphEdge]) {
        self.key = key;
        self.nodes.clear();
        self.nodes.extend_from_slice(nodes);
        self.edges.clear();
        self.edges.extend_from_slice(edges);
        let index: HashMap<&str, usize> = self
            .nodes
            .iter()
            .enumerate()
            .map(|(at, node)| (node.id.as_ref(), at))
            .collect();
        self.links = self
            .edges
            .iter()
            .map(|edge| {
                Some((
                    *index.get(edge.source.as_ref())?,
                    *index.get(edge.target.as_ref())?,
                ))
            })
            .collect();
        self.around = vec![Vec::new(); self.nodes.len()];
        self.incoming = vec![0; self.nodes.len()];
        for (edge, link) in self.links.iter().enumerate() {
            if let Some((source, target)) = link {
                self.around[*source].push(edge);
                self.around[*target].push(edge);
                self.incoming[*target] += 1;
            }
        }
        self.shape = None;
    }

    pub(crate) fn ensure(&mut self, shape: Shape) {
        if self.shape == Some(shape) {
            return;
        }
        self.shape = Some(shape);
        let count = self.nodes.len();
        self.positions = vec![[0.; 2]; count];
        if shape.layout == NodeLayout::Grid {
            let mut order: Vec<usize> = (0..count).collect();
            if let Some((arc, ascending)) = shape.sort {
                let value = |node: usize| self.nodes[node].arcs.get(arc).copied().unwrap_or(0.);
                order.sort_by(|a, b| {
                    let ordering = value(*a).total_cmp(&value(*b));
                    if ascending {
                        ordering
                    } else {
                        ordering.reverse()
                    }
                });
            }
            let center = shape
                .root
                .and_then(|root| order.iter().position(|node| *node == root))
                .unwrap_or(0);
            let start = if count > shape.limit {
                center
                    .saturating_sub(shape.limit / 2)
                    .min(count - shape.limit)
            } else {
                0
            };
            let window = &order[start..(start + shape.limit).min(count)];
            let mut nodes = vec![false; count];
            for (node, at) in window.iter().zip(grid(window.len(), shape.width as f32)) {
                nodes[*node] = true;
                self.positions[*node] = at;
            }
            self.view = Visible {
                nodes,
                markers: Vec::new(),
            };
            return;
        }
        self.view = visible(self, shape.limit, shape.root);
        let members: Vec<usize> = (0..count)
            .filter(|node| self.view.shows(*node) || self.view.marked(*node))
            .collect();
        let mut local = vec![usize::MAX; count];
        for (slot, node) in members.iter().enumerate() {
            local[*node] = slot;
        }
        let pairs: Vec<(usize, usize)> = self
            .links
            .iter()
            .flatten()
            .filter(|(source, target)| local[*source] != usize::MAX && local[*target] != usize::MAX)
            .map(|(source, target)| (local[*source], local[*target]))
            .collect();
        let placed = match shape.layout {
            NodeLayout::Force => force(members.len(), &pairs),
            _ => layered(members.len(), &pairs),
        };
        for (node, at) in members.iter().zip(placed) {
            self.positions[*node] = at;
        }
    }

    pub(crate) fn placed(&self) -> &[Position] {
        &self.positions
    }

    pub(crate) fn other(&self, edge: usize, node: usize) -> Option<usize> {
        let (source, target) = self.links.get(edge).copied().flatten()?;
        Some(if source == node { target } else { source })
    }
}

#[derive(Default)]
pub(crate) struct Visible {
    pub nodes: Vec<bool>,
    pub markers: Vec<(usize, usize)>,
}

impl Visible {
    pub(crate) fn shows(&self, node: usize) -> bool {
        self.nodes.get(node).copied().unwrap_or(false)
    }

    pub(crate) fn marked(&self, node: usize) -> bool {
        self.markers.iter().any(|(marker, _)| *marker == node)
    }
}

pub(crate) fn visible(graph: &Graph, limit: usize, root: Option<usize>) -> Visible {
    let count = graph.nodes.len();
    if count <= limit {
        return Visible {
            nodes: vec![true; count],
            markers: Vec::new(),
        };
    }
    let roots: Vec<usize> = root.map_or_else(
        || {
            let found: Vec<usize> = (0..count)
                .filter(|node| graph.incoming[*node] == 0)
                .collect();
            if found.is_empty() { vec![0] } else { found }
        },
        |root| vec![root],
    );
    let mut nodes = vec![false; count];
    let mut shown = 0;
    let mut queue: VecDeque<usize> = roots.into_iter().collect();
    while shown < limit {
        let Some(current) = queue.pop_front() else {
            break;
        };
        if nodes[current] {
            continue;
        }
        nodes[current] = true;
        shown += 1;
        queue.extend(
            graph.around[current]
                .iter()
                .filter_map(|edge| graph.other(*edge, current)),
        );
    }
    let mut frontier: Vec<usize> = queue.into_iter().filter(|node| !nodes[*node]).collect();
    frontier.sort_unstable();
    frontier.dedup();
    let mut markers = Vec::new();
    for marker in frontier {
        let mut seen: Vec<bool> = vec![false; count];
        let mut stack = VecDeque::from([marker]);
        let mut hidden = 0;
        while hidden <= MARKER_COUNT_LIMIT {
            let Some(current) = stack.pop_front() else {
                break;
            };
            if nodes[current] || seen[current] {
                continue;
            }
            seen[current] = true;
            hidden += 1;
            stack.extend(
                graph.around[current]
                    .iter()
                    .filter_map(|edge| graph.other(*edge, current)),
            );
        }
        if hidden == 1 {
            nodes[marker] = true;
        } else {
            markers.push((marker, hidden));
        }
    }
    Visible { nodes, markers }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chain(count: usize) -> Graph {
        let nodes: Vec<GraphNode> = (0..count)
            .map(|index| GraphNode::new(index.to_string()))
            .collect();
        let edges: Vec<GraphEdge> = (1..count)
            .map(|index| GraphEdge::new((index - 1).to_string(), index.to_string()))
            .collect();
        let mut graph = Graph::default();
        graph.load(Some(1), &nodes, &edges);
        graph
    }

    #[test]
    fn edges_resolve_to_node_indices() {
        let graph = chain(3);
        assert_eq!(graph.links, [Some((0, 1)), Some((1, 2))]);
        assert_eq!(graph.incoming, [0, 1, 1]);
    }

    #[test]
    fn limit_collapses_the_rest_into_a_marker() {
        let graph = chain(10);
        let shown = visible(&graph, 4, None);
        assert_eq!(shown.nodes.iter().filter(|shown| **shown).count(), 4);
        assert_eq!(shown.markers, [(4, 6)]);
    }
}
