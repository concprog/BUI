use std::collections::HashSet;

use super::{
    tree::{Entries, Entry, Kind, Metrics, insert_beside, layout_rects, leaf_ids, normalize, sum},
    types::{Axis, Edge, Id, Node, Rect},
};

#[must_use]
pub fn range_id(split_id: &str, children: &[Id]) -> Id {
    let mut parts: Vec<&str> = vec![split_id];
    parts.extend(children.iter().map(std::convert::AsRef::as_ref));
    format!(
        "range:{}",
        serde_json::to_string(&parts).unwrap_or_default()
    )
    .as_str()
    .into()
}

#[must_use]
pub fn focus_layout(root: Option<&Node>, metrics: Option<&dyn Metrics>) -> Entries {
    let mut entries = layout_rects(root, metrics);
    let splits: Vec<(Id, Axis, Vec<f64>, Vec<Id>)> = entries
        .iter()
        .filter_map(|e| match &e.kind {
            Kind::Split {
                axis,
                weights,
                children,
            } if children.len() >= 3 => {
                Some((e.id.clone(), *axis, weights.clone(), children.clone()))
            }
            _ => None,
        })
        .collect();

    for (split_id, axis, weights, children) in splits {
        for start in 0..children.len() - 1 {
            for end in start + 2..=children.len() {
                if start == 0 && end == children.len() {
                    continue;
                }
                let kids = &children[start..end];
                let (Some(first), Some(last)) =
                    (entries.rect(&kids[0]), entries.rect(&kids[kids.len() - 1]))
                else {
                    continue;
                };
                let ws = &weights[start..end];
                let total = sum(ws);
                entries.insert(Entry {
                    id: range_id(&split_id, kids),
                    kind: Kind::Split {
                        axis,
                        weights: ws.iter().map(|w| w / total).collect(),
                        children: kids.to_vec(),
                    },
                    rect: Rect {
                        x: first.x,
                        y: first.y,
                        w: last.right() - first.x,
                        h: last.bottom() - first.y,
                    },
                    parent: Some(split_id.clone()),
                    scale: 1.0,
                    range: true,
                });
            }
        }
    }
    entries
}

#[must_use]
pub fn nav_parent(entries: &Entries, id: &str) -> Option<Id> {
    let entry = entries.get(id)?;
    let mut at = entry.parent.clone();
    while let Some(ref parent_id) = at {
        let parent = entries.get(parent_id)?;
        if !parent.rect.approx_eq(&entry.rect, 1e-9) {
            return at;
        }
        at.clone_from(&parent.parent);
    }
    None
}

#[must_use]
pub fn nav_node(entries: &Entries, id: &Id) -> Id {
    let mut current = id.clone();
    while let Some(Kind::Stage { child: Some(child) }) = entries.get(&current).map(|e| &e.kind) {
        current = child.clone();
    }
    current
}

#[must_use]
pub fn best_fit(camera: Rect, entries: &Entries) -> Option<Id> {
    fit_among(camera, entries, |_| true)
}

fn fit_among(camera: Rect, entries: &Entries, keep: impl Fn(Rect) -> bool) -> Option<Id> {
    let mut best: Option<Id> = None;
    let mut score = f64::INFINITY;
    for e in entries.iter() {
        if matches!(e.kind, Kind::Stage { child: Some(_) }) || !keep(e.rect) {
            continue;
        }
        let r = e.rect;
        let size = (r.w / camera.w).ln().abs() + (r.h / camera.h).ln().abs();
        let distance = ((r.x + r.w / 2.0 - camera.x - camera.w / 2.0) / camera.w)
            .hypot((r.y + r.h / 2.0 - camera.y - camera.h / 2.0) / camera.h);
        let next = size + distance * 2.5;
        if next < score {
            best = Some(e.id.clone());
            score = next;
        }
    }
    best
}

#[must_use]
pub fn neighbor(entries: &Entries, from: Rect, dx: f64, dy: f64) -> Option<Id> {
    const EPS: f64 = 1e-6;
    let target = Rect {
        x: from.x + dx * from.w,
        y: from.y + dy * from.h,
        ..from
    };
    fit_among(target, entries, |r| {
        let beyond = (dx <= 0.0 || r.x >= from.right() - EPS)
            && (dx >= 0.0 || r.right() <= from.x + EPS)
            && (dy <= 0.0 || r.y >= from.bottom() - EPS)
            && (dy >= 0.0 || r.bottom() <= from.y + EPS);
        let overlaps = if dx == 0.0 {
            r.x < from.right() - EPS && r.right() > from.x + EPS
        } else {
            r.y < from.bottom() - EPS && r.bottom() > from.y + EPS
        };
        beyond && overlaps
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    In,
    Out,
}

#[must_use]
pub fn hierarchy_step(entries: &Entries, focused: &Id, direction: Step, point: (f64, f64)) -> Id {
    let id = nav_node(entries, focused);
    let Some(entry) = entries.get(&id) else {
        return focused.clone();
    };
    if direction == Step::Out {
        return nav_parent(entries, &id).unwrap_or(id);
    }
    let Kind::Split { axis, children, .. } = &entry.kind else {
        return id;
    };
    let coordinate = if *axis == Axis::X { point.0 } else { point.1 };
    for child in &children[..children.len() - 1] {
        if let Some(r) = entries.rect(child) {
            let end = if *axis == Axis::X {
                r.right()
            } else {
                r.bottom()
            };
            if coordinate < end {
                return nav_node(entries, child);
            }
        }
    }
    nav_node(entries, &children[children.len() - 1])
}

#[must_use]
pub fn frame_leaves(root: Option<&Node>, ids: &[Id]) -> Option<Id> {
    let root = root?;
    let entries = focus_layout(Some(root), None);
    let wanted: Vec<&Id> = ids.iter().filter(|id| entries.contains(id)).collect();
    let mut best = entries.get(root.id())?;
    for entry in entries.iter() {
        if matches!(entry.kind, Kind::Stage { child: Some(_) }) {
            continue;
        }
        if entry.rect.w * entry.rect.h < best.rect.w * best.rect.h
            && wanted.iter().all(|id| entries.contains_node(&entry.id, id))
        {
            best = entry;
        }
    }
    Some(best.id.clone())
}

#[must_use]
pub fn rectangle_camera(a: (f64, f64), b: (f64, f64), viewport: Rect, camera: Rect) -> Rect {
    let clamp = |v: f64| v.clamp(0.0, 1.0);
    let (ax, ay) = (
        clamp((a.0 - viewport.x) / viewport.w),
        clamp((a.1 - viewport.y) / viewport.h),
    );
    let (bx, by) = (
        clamp((b.0 - viewport.x) / viewport.w),
        clamp((b.1 - viewport.y) / viewport.h),
    );
    Rect {
        x: camera.x + ax.min(bx) * camera.w,
        y: camera.y + ay.min(by) * camera.h,
        w: (bx - ax).abs() * camera.w,
        h: (by - ay).abs() * camera.h,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Visit {
    pub id: Id,
    pub leaves: Vec<Id>,
}

#[must_use]
pub fn record_visit(mut visits: Vec<Visit>, index: usize, visit: Visit) -> (Vec<Visit>, usize) {
    if visits.get(index).is_some_and(|prev| *prev == visit) {
        return (visits, index);
    }
    visits.truncate(index + 1);
    visits.push(visit);
    let last = visits.len() - 1;
    (visits, last)
}

#[must_use]
pub fn saved_frame_destination(root: Option<&Node>, leaves: &[Id]) -> Option<Id> {
    let entries = layout_rects(root, None);
    let surviving: Vec<Id> = leaves
        .iter()
        .filter(|id| entries.contains(id))
        .cloned()
        .collect();
    if surviving.is_empty() {
        None
    } else {
        frame_leaves(root, &surviving)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaximizeSession {
    pub id: Id,
    pub return_path: Vec<Id>,
}

#[must_use]
pub fn maximize_transition(
    entries: &Entries,
    root_id: &Id,
    focused: &Id,
    id: &Id,
    session: Option<&MaximizeSession>,
) -> (Id, Option<MaximizeSession>) {
    if focused == id {
        let destination = match session {
            Some(s) if &s.id == id => s
                .return_path
                .iter()
                .find(|key| *key != id && entries.contains(key))
                .cloned()
                .unwrap_or_else(|| root_id.clone()),
            _ => nav_parent(entries, id).unwrap_or_else(|| root_id.clone()),
        };
        return (destination, None);
    }
    let mut return_path = Vec::new();
    let mut at = Some(focused.clone());
    while let Some(current) = at {
        at = entries.get(&current).and_then(|e| e.parent.clone());
        return_path.push(current);
    }
    (
        id.clone(),
        Some(MaximizeSession {
            id: id.clone(),
            return_path,
        }),
    )
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SplitSeam {
    pub axis: Axis,
    pub leaves: Vec<Id>,
    pub before: Vec<Id>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DockTargetSpec {
    pub id: Id,
    pub edge: Edge,
    pub seam: Option<SplitSeam>,
    pub boundary: Option<usize>,
}

impl DockTargetSpec {
    const fn plain(id: Id, edge: Edge) -> Self {
        Self {
            id,
            edge,
            seam: None,
            boundary: None,
        }
    }
}

#[must_use]
pub fn drop_edge(point: (f64, f64), rect: Rect) -> Option<Edge> {
    if rect.w <= 0.0 || rect.h <= 0.0 {
        return None;
    }
    let x = (point.0 - rect.x) / rect.w;
    let y = (point.1 - rect.y) / rect.h;
    if !(0.0..=1.0).contains(&x) || !(0.0..=1.0).contains(&y) {
        return None;
    }
    let mut edges = [
        (Edge::Left, x),
        (Edge::Right, 1.0 - x),
        (Edge::Top, y),
        (Edge::Bottom, 1.0 - y),
    ];
    edges.sort_by(|a, b| a.1.total_cmp(&b.1));
    (edges[0].1 <= 0.28).then_some(edges[0].0)
}

pub struct DropRects {
    pub slot: Rect,
    pub remaining: Rect,
    pub collapsed: Rect,
}

#[must_use]
pub fn drop_rects(rect: Rect, edge: Edge) -> DropRects {
    let (mut slot, mut remaining, mut collapsed) = (rect, rect, rect);
    if edge.axis() == Axis::X {
        slot.w = rect.w / 2.0;
        remaining.w = slot.w;
        collapsed.w = 0.0;
        if edge == Edge::Left {
            remaining.x += slot.w;
        } else {
            slot.x += slot.w;
            collapsed.x += rect.w;
        }
    } else {
        slot.h = rect.h / 2.0;
        remaining.h = slot.h;
        collapsed.h = 0.0;
        if edge == Edge::Top {
            remaining.y += slot.h;
        } else {
            slot.y += slot.h;
            collapsed.y += rect.h;
        }
    }
    DropRects {
        slot,
        remaining,
        collapsed,
    }
}

#[must_use]
pub fn seam_target(entries: &Entries, split_id: &Id, boundary: usize) -> Option<DockTargetSpec> {
    let Kind::Split { axis, children, .. } = &entries.get(split_id)?.kind else {
        return None;
    };
    let before = children[..boundary.min(children.len())]
        .iter()
        .flat_map(|c| entries.leaf_ids(c))
        .collect();
    Some(DockTargetSpec {
        id: split_id.clone(),
        edge: if *axis == Axis::X {
            Edge::Left
        } else {
            Edge::Top
        },
        boundary: Some(boundary),
        seam: Some(SplitSeam {
            axis: *axis,
            leaves: entries.leaf_ids(split_id),
            before,
        }),
    })
}

#[must_use]
pub fn tile_drop_target(entries: &Entries, id: &Id, edge: Edge) -> DockTargetSpec {
    let seam = entries
        .get(id)
        .and_then(|entry| entry.parent.as_ref())
        .and_then(|parent_id| {
            let Kind::Split { axis, children, .. } = &entries.get(parent_id)?.kind else {
                return None;
            };
            if *axis != edge.axis() {
                return None;
            }
            let index = children.iter().position(|c| c == id)?;
            seam_target(
                entries,
                parent_id,
                index + usize::from(matches!(edge, Edge::Right | Edge::Bottom)),
            )
        });
    seam.unwrap_or_else(|| DockTargetSpec::plain(id.clone(), edge))
}

#[must_use]
pub fn frame_drop_target(
    entries: &Entries,
    group_id: &Id,
    point: (f64, f64),
    width: f64,
    height: f64,
    radius: f64,
) -> Option<DockTargetSpec> {
    if point.0 < 0.0 || point.1 < 0.0 || point.0 > width || point.1 > height {
        return None;
    }
    let mut distances = [
        (Edge::Left, point.0),
        (Edge::Right, width - point.0),
        (Edge::Top, point.1),
        (Edge::Bottom, height - point.1),
    ];
    distances.sort_by(|a, b| a.1.total_cmp(&b.1));
    let (edge, distance) = distances[0];
    if distance > radius {
        return None;
    }
    let axis = edge.axis();
    let after = matches!(edge, Edge::Right | Edge::Bottom);
    let inner = match entries.get(group_id).map(|e| &e.kind) {
        Some(Kind::Stage { child: Some(child) }) => child.clone(),
        _ => group_id.clone(),
    };
    if let Some(Kind::Split {
        axis: inner_axis,
        children,
        ..
    }) = entries.get(&inner).map(|e| &e.kind)
        && *inner_axis == axis
    {
        return seam_target(entries, &inner, if after { children.len() } else { 0 });
    }
    let leaves = entries.leaf_ids(group_id);
    Some(DockTargetSpec {
        id: group_id.clone(),
        edge,
        boundary: None,
        seam: Some(SplitSeam {
            axis,
            before: if after { leaves.clone() } else { Vec::new() },
            leaves,
        }),
    })
}

enum SeamAction {
    IntoStageChild,
    IntoChild(usize),
    Aligned(usize),
    Beside,
}

fn holds_all(node: &Node, surviving: &HashSet<Id>) -> bool {
    let ids: HashSet<Id> = leaf_ids(Some(node)).into_iter().collect();
    surviving.iter().all(|id| ids.contains(id))
}

fn insert_seam_rec(
    node: Node,
    surviving: &HashSet<Id>,
    seam: &SplitSeam,
    incoming: &Node,
    split_id: &Id,
) -> Node {
    let action = match &node {
        Node::Stage(s)
            if s.child.as_deref().is_some_and(|c| holds_all(c, surviving))
                && !surviving.contains(&s.id) =>
        {
            SeamAction::IntoStageChild
        }
        Node::Split(s) => {
            if let Some(i) = s.children.iter().position(|c| holds_all(c, surviving)) {
                SeamAction::IntoChild(i)
            } else if s.axis == seam.axis {
                let index = s
                    .children
                    .iter()
                    .position(|c| leaf_ids(Some(c)).iter().any(|id| !seam.before.contains(id)))
                    .unwrap_or(s.children.len());
                SeamAction::Aligned(index)
            } else {
                SeamAction::Beside
            }
        }
        _ => SeamAction::Beside,
    };

    match (action, node) {
        (SeamAction::IntoStageChild, Node::Stage(mut s)) => {
            s.child = s
                .child
                .take()
                .map(|c| Box::new(insert_seam_rec(*c, surviving, seam, incoming, split_id)));
            Node::Stage(s)
        }
        (SeamAction::IntoChild(i), Node::Split(mut s)) => {
            let child = s.children.remove(i);
            s.children.insert(
                i,
                insert_seam_rec(child, surviving, seam, incoming, split_id),
            );
            Node::Split(s)
        }
        (SeamAction::Aligned(index), Node::Split(mut s)) => {
            let total = sum(&s.weights);
            let share = 1.0 / (s.children.len() as f64 + 1.0);
            let mut weights: Vec<f64> = s
                .weights
                .iter()
                .map(|w| w / total * (1.0 - share))
                .collect();
            weights.insert(index, share);
            s.children.insert(index, incoming.clone());
            s.weights = weights;
            Node::Split(s)
        }
        (_, node) => {
            let before = !surviving.iter().any(|id| seam.before.contains(id));
            let edge = match (seam.axis, before) {
                (Axis::X, true) => Edge::Left,
                (Axis::X, false) => Edge::Right,
                (Axis::Y, true) => Edge::Top,
                (Axis::Y, false) => Edge::Bottom,
            };
            let target = node.id().clone();
            insert_beside(
                Some(node),
                &target,
                incoming.clone(),
                edge,
                split_id.clone(),
                0.5,
            )
        }
    }
}

#[must_use]
pub fn insert_at_seam(root: Node, seam: &SplitSeam, incoming: &Node, split_id: &Id) -> Node {
    let surviving: HashSet<Id> = leaf_ids(Some(&root))
        .into_iter()
        .filter(|id| seam.leaves.contains(id))
        .collect();
    if surviving.is_empty() {
        return root;
    }
    let inserted = insert_seam_rec(root.clone(), &surviving, seam, incoming, split_id);
    normalize(inserted).unwrap_or(root)
}

#[must_use]
pub fn apply_dock_target(
    root: Option<Node>,
    target: &DockTargetSpec,
    incoming: Node,
    split_id: Id,
) -> Node {
    let Some(root) = root else { return incoming };
    match &target.seam {
        Some(seam) => insert_at_seam(root, seam, &incoming, &split_id),
        None => insert_beside(Some(root), &target.id, incoming, target.edge, split_id, 0.5),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::types::{PanelNode, SplitNode};
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
    fn row3() -> Node {
        Node::Split(SplitNode {
            id: id("r"),
            axis: Axis::X,
            weights: vec![1.0 / 3.0; 3],
            children: vec![panel("a"), panel("b"), panel("c")],
        })
    }

    #[test]
    fn edge_band_is_28_percent() {
        let r = Rect::new(0.0, 0.0, 100.0, 100.0);
        assert_eq!(drop_edge((10.0, 50.0), r), Some(Edge::Left));
        assert_eq!(drop_edge((50.0, 50.0), r), None);
    }

    #[test]
    fn focus_layout_adds_sibling_ranges() {
        let root = row3();
        let entries = focus_layout(Some(&root), None);
        let ranges = entries.iter().filter(|e| e.range).count();
        assert_eq!(ranges, 2);
    }

    #[test]
    fn frame_leaves_prefers_the_smallest_container() {
        let root = row3();
        assert_eq!(frame_leaves(Some(&root), &[id("a")]).unwrap().as_ref(), "a");
        let both = frame_leaves(Some(&root), &[id("a"), id("b")]).unwrap();
        assert!(both.starts_with("range:"));
    }

    #[test]
    fn seam_insert_takes_an_equal_share() {
        let root = row3();
        let entries = layout_rects(Some(&root), None);
        let target = seam_target(&entries, &id("r"), 1).unwrap();
        let out = apply_dock_target(Some(root), &target, panel("n"), id("s"));
        let Node::Split(s) = out else {
            panic!("expected a split")
        };
        assert_eq!(s.children.len(), 4);
        assert_eq!(s.children[1].id().as_ref(), "n");
        assert!((s.weights[1] - 0.25).abs() < 1e-9);
    }
}
