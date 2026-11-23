use std::{
    collections::{BTreeMap, HashSet},
    hash::BuildHasher,
    sync::atomic::{AtomicU64, Ordering},
};

use super::{
    tree::{
        find_node, find_stage, insert_beside, normalize, panels_of, remove_node,
        replace_node,
    },
    types::{
        Axis, Edge, FloatingLayer, FloatingPanel, HiddenPanel, Id, LayoutDocument, Node, PanelNode,
        Rect, RestoreTarget, SplitNode, StageNode, ViewRecord,
    },
};

static COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn uid(prefix: &str) -> Id {
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let salt = std::collections::hash_map::RandomState::new().hash_one(n);
    format!("{prefix}-{:05x}{:x}", salt & 0xf_ffff, n)
        .as_str()
        .into()
}

#[must_use]
pub const fn empty_document() -> LayoutDocument {
    LayoutDocument {
        schema: 1,
        version: None,
        root: None,
        floating: Vec::new(),
        hidden: Vec::new(),
        views: BTreeMap::new(),
        navigation: None,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Where {
    Docked,
    Floating,
    Hidden,
}

pub enum PanelLocation<'a> {
    Docked(&'a PanelNode),
    Floating(&'a PanelNode, &'a FloatingPanel),
    Hidden(&'a PanelNode, &'a RestoreTarget),
}

impl<'a> PanelLocation<'a> {
    #[must_use]
    pub const fn panel(&self) -> &'a PanelNode {
        match self {
            Self::Docked(p) | Self::Floating(p, _) | Self::Hidden(p, _) => p,
        }
    }
    #[must_use]
    pub const fn place(&self) -> Where {
        match self {
            Self::Docked(_) => Where::Docked,
            Self::Floating(..) => Where::Floating,
            Self::Hidden(..) => Where::Hidden,
        }
    }
}

#[must_use]
pub fn all_panels(doc: &LayoutDocument) -> Vec<&PanelNode> {
    let mut out = panels_of(doc.root.as_ref());
    out.extend(doc.floating.iter().map(|f| &f.panel));
    out.extend(doc.hidden.iter().map(|h| &h.panel));
    out
}

#[must_use]
pub fn locate_panel<'a>(doc: &'a LayoutDocument, panel_id: &str) -> Option<PanelLocation<'a>> {
    if let Some(Node::Panel(p)) = find_node(doc.root.as_ref(), panel_id) {
        return Some(PanelLocation::Docked(p));
    }
    if let Some(f) = doc
        .floating
        .iter()
        .find(|f| f.panel.id.as_ref() == panel_id)
    {
        return Some(PanelLocation::Floating(&f.panel, f));
    }
    doc.hidden
        .iter()
        .find(|h| h.panel.id.as_ref() == panel_id)
        .map(|h| PanelLocation::Hidden(&h.panel, &h.restore))
}

#[must_use]
pub fn panel_of_view<'a>(doc: &'a LayoutDocument, view_id: &str) -> Option<&'a PanelNode> {
    all_panels(doc)
        .into_iter()
        .find(|p| p.views.iter().any(|v| v.as_ref() == view_id))
}

#[must_use]
pub fn view_ids(doc: &LayoutDocument) -> Vec<Id> {
    all_panels(doc)
        .into_iter()
        .flat_map(|p| p.views.iter().cloned())
        .collect()
}

#[must_use]
pub fn update_panel(doc: &LayoutDocument, mut panel: PanelNode) -> LayoutDocument {
    if panel.views.is_empty() {
        return remove_panel(doc, &panel.id);
    }
    if !panel.views.contains(&panel.selected) {
        panel.selected = panel.views[0].clone();
    }
    let mut next = doc.clone();
    match locate_panel(doc, &panel.id).map(|l| l.place()) {
        None => return next,
        Some(Where::Docked) => {
            let id = panel.id.clone();
            next.root = replace_node(next.root.take(), &id, Node::Panel(panel));
        }
        Some(Where::Floating) => {
            if let Some(f) = next.floating.iter_mut().find(|f| f.panel.id == panel.id) {
                f.panel = panel;
            }
        }
        Some(Where::Hidden) => {
            if let Some(h) = next.hidden.iter_mut().find(|h| h.panel.id == panel.id) {
                h.panel = panel;
            }
        }
    }
    next
}

#[must_use]
pub fn remove_panel(doc: &LayoutDocument, panel_id: &str) -> LayoutDocument {
    let mut next = doc.clone();
    match locate_panel(doc, panel_id).map(|l| l.place()) {
        None => {}
        Some(Where::Docked) => next.root = remove_node(next.root.take(), panel_id),
        Some(Where::Floating) => next.floating.retain(|f| f.panel.id.as_ref() != panel_id),
        Some(Where::Hidden) => next.hidden.retain(|h| h.panel.id.as_ref() != panel_id),
    }
    next
}

#[must_use]
pub fn detach_view(doc: &LayoutDocument, view_id: &str) -> LayoutDocument {
    let Some(panel) = panel_of_view(doc, view_id) else {
        return doc.clone();
    };
    let views: Vec<Id> = panel
        .views
        .iter()
        .filter(|v| v.as_ref() != view_id)
        .cloned()
        .collect();
    let mut selected = panel.selected.clone();
    if selected.as_ref() == view_id {
        let index = panel
            .views
            .iter()
            .position(|v| v.as_ref() == view_id)
            .unwrap_or(0);
        selected = views
            .get(index.min(views.len().saturating_sub(1)))
            .cloned()
            .unwrap_or_else(|| "".into());
    }
    update_panel(
        doc,
        PanelNode {
            id: panel.id.clone(),
            views,
            selected,
        },
    )
}

#[must_use]
pub fn close_view(doc: &LayoutDocument, view_id: &str) -> LayoutDocument {
    let mut next = detach_view(doc, view_id);
    next.views.remove(view_id);
    next
}

#[must_use]
pub fn select_view(doc: &LayoutDocument, view_id: &str) -> LayoutDocument {
    match panel_of_view(doc, view_id) {
        Some(p) if p.selected.as_ref() != view_id => update_panel(
            doc,
            PanelNode {
                selected: view_id.into(),
                ..p.clone()
            },
        ),
        _ => doc.clone(),
    }
}

#[must_use]
pub fn add_tab(
    doc: &LayoutDocument,
    panel_id: &str,
    view_id: &str,
    index: Option<usize>,
    select: bool,
) -> LayoutDocument {
    let Some(loc) = locate_panel(doc, panel_id) else {
        return doc.clone();
    };
    let panel = loc.panel();
    let mut views: Vec<Id> = panel
        .views
        .iter()
        .filter(|v| v.as_ref() != view_id)
        .cloned()
        .collect();
    let at = index.map_or(views.len(), |i| i.min(views.len()));
    views.insert(at, view_id.into());
    let selected = if select || panel.selected.is_empty() {
        view_id.into()
    } else {
        panel.selected.clone()
    };
    update_panel(
        doc,
        PanelNode {
            id: panel.id.clone(),
            views,
            selected,
        },
    )
}

#[must_use]
pub fn reorder_tabs(doc: &LayoutDocument, panel_id: &str, order: &[Id]) -> LayoutDocument {
    let Some(loc) = locate_panel(doc, panel_id) else {
        return doc.clone();
    };
    let panel = loc.panel();
    if order.len() != panel.views.len() || order.iter().any(|id| !panel.views.contains(id)) {
        return doc.clone();
    }
    update_panel(
        doc,
        PanelNode {
            views: order.to_vec(),
            ..panel.clone()
        },
    )
}

#[derive(Clone, Debug, PartialEq)]
pub enum DockTarget {
    Beside {
        beside: Id,
        edge: Edge,
        share: Option<f64>,
    },
    Into {
        panel: Id,
        index: Option<usize>,
    },
}

#[must_use]
pub fn insert_panel(doc: &LayoutDocument, panel: PanelNode, target: &DockTarget) -> LayoutDocument {
    match target {
        DockTarget::Into { panel: into, index } => match find_node(doc.root.as_ref(), into) {
            Some(Node::Stage(stage)) => match &stage.child {
                None => {
                    let mut next = doc.clone();
                    next.root = replace_node(
                        next.root.take(),
                        &stage.id,
                        Node::Stage(StageNode {
                            id: stage.id.clone(),
                            child: Some(Box::new(Node::Panel(panel))),
                        }),
                    );
                    next
                }
                Some(child) => {
                    let first = panels_of(Some(child))[0].id.clone();
                    merge_into(doc, panel, &first, *index)
                }
            },
            _ => merge_into(doc, panel, into, *index),
        },
        DockTarget::Beside {
            beside,
            edge,
            share,
        } => {
            let Some(root) = &doc.root else {
                let mut next = doc.clone();
                next.root = Some(Node::Panel(panel));
                return next;
            };
            if find_node(Some(root), beside).is_none() {
                let fallback = DockTarget::Beside {
                    beside: root.id().clone(),
                    edge: *edge,
                    share: None,
                };
                return insert_panel(doc, panel, &fallback);
            }
            let mut next = doc.clone();
            next.root = Some(insert_beside(
                next.root.take(),
                beside,
                Node::Panel(panel),
                *edge,
                uid("split"),
                share.unwrap_or(0.5),
            ));
            next
        }
    }
}

fn merge_into(
    doc: &LayoutDocument,
    panel: PanelNode,
    into: &str,
    index: Option<usize>,
) -> LayoutDocument {
    let Some(loc) = locate_panel(doc, into) else {
        return doc.clone();
    };
    let target = loc.panel();
    let mut views = target.views.clone();
    let at = index.unwrap_or(views.len()).min(views.len());
    for (offset, v) in panel.views.iter().enumerate() {
        views.insert(at + offset, v.clone());
    }
    update_panel(
        doc,
        PanelNode {
            id: target.id.clone(),
            views,
            selected: panel.selected,
        },
    )
}

#[must_use]
pub fn float_panel(
    doc: &LayoutDocument,
    panel: PanelNode,
    rect: Rect,
    layer: FloatingLayer,
) -> LayoutDocument {
    let mut next = remove_panel(doc, &panel.id);
    let z = next.floating.iter().map(|f| f.z).max().unwrap_or(0) + 1;
    next.floating.push(FloatingPanel {
        panel,
        rect: clamp_float(rect),
        z,
        layer,
    });
    next
}

#[must_use]
pub fn clamp_float(rect: Rect) -> Rect {
    let w = rect.w.clamp(0.02, 1.0);
    let h = rect.h.clamp(0.02, 1.0);
    Rect {
        w,
        h,
        x: rect.x.clamp(-w + 0.05, 1.0 - w.min(0.05)),
        y: rect.y.clamp(0.0, 1.0 - h.min(0.05)),
    }
}

#[must_use]
pub fn raise_float(doc: &LayoutDocument, panel_id: &str) -> LayoutDocument {
    let top = doc.floating.iter().map(|f| f.z).max().unwrap_or(0);
    match doc
        .floating
        .iter()
        .find(|f| f.panel.id.as_ref() == panel_id)
    {
        Some(f) if f.z != top => {
            let mut next = doc.clone();
            if let Some(f) = next
                .floating
                .iter_mut()
                .find(|f| f.panel.id.as_ref() == panel_id)
            {
                f.z = top + 1;
            }
            next
        }
        _ => doc.clone(),
    }
}

#[must_use]
pub fn parent_split<'a>(root: Option<&'a Node>, id: &str) -> Option<&'a SplitNode> {
    match root? {
        Node::Panel(_) => None,
        Node::Stage(s) => parent_split(s.child.as_deref(), id),
        Node::Split(s) => {
            if s.children.iter().any(|c| c.id().as_ref() == id) {
                return Some(s);
            }
            s.children.iter().find_map(|c| parent_split(Some(c), id))
        }
    }
}

#[must_use]
pub fn restore_target_for(doc: &LayoutDocument, panel_id: &str) -> Option<RestoreTarget> {
    match locate_panel(doc, panel_id)? {
        PanelLocation::Floating(_, f) => Some(RestoreTarget::Floating {
            rect: f.rect,
            layer: f.layer,
        }),
        PanelLocation::Hidden(_, restore) => Some(restore.clone()),
        PanelLocation::Docked(_) => {
            let Some(parent) = parent_split(doc.root.as_ref(), panel_id) else {
                if let Some(stage) = find_stage(doc.root.as_ref())
                    && stage
                        .child
                        .as_ref()
                        .is_some_and(|c| c.id().as_ref() == panel_id)
                {
                    return Some(RestoreTarget::Tab {
                        panel: stage.id.clone(),
                    });
                }
                let beside = doc
                    .root
                    .as_ref()
                    .map_or_else(|| "".into(), |r| r.id().clone());
                return Some(RestoreTarget::Docked {
                    beside,
                    edge: Edge::Left,
                    share: 0.5,
                });
            };
            let index = parent
                .children
                .iter()
                .position(|c| c.id().as_ref() == panel_id)?;
            let after = parent.children.get(index + 1).is_some();
            let neighbour_index = if after { index + 1 } else { index - 1 };
            let neighbour = &parent.children[neighbour_index];
            let own = parent.weights[index];
            let share = own / (own + parent.weights[neighbour_index]);
            let edge = match (parent.axis, after) {
                (Axis::X, true) => Edge::Left,
                (Axis::X, false) => Edge::Right,
                (Axis::Y, true) => Edge::Top,
                (Axis::Y, false) => Edge::Bottom,
            };
            Some(RestoreTarget::Docked {
                beside: neighbour.id().clone(),
                edge,
                share,
            })
        }
    }
}

#[must_use]
pub fn hide_panel(doc: &LayoutDocument, panel_id: &str) -> LayoutDocument {
    let Some(loc) = locate_panel(doc, panel_id) else {
        return doc.clone();
    };
    if loc.place() == Where::Hidden {
        return doc.clone();
    }
    let panel = loc.panel().clone();
    let Some(restore) = restore_target_for(doc, panel_id) else {
        return doc.clone();
    };
    let mut next = remove_panel(doc, panel_id);
    next.hidden.push(HiddenPanel { panel, restore });
    next
}

#[must_use]
pub fn restore_panel(
    doc: &LayoutDocument,
    panel_id: &str,
    fallback_layer: FloatingLayer,
) -> LayoutDocument {
    let Some(hidden) = doc.hidden.iter().find(|h| h.panel.id.as_ref() == panel_id) else {
        return doc.clone();
    };
    let mut without = doc.clone();
    without.hidden.retain(|h| h.panel.id.as_ref() != panel_id);
    let (restore, panel) = (&hidden.restore, hidden.panel.clone());

    match restore {
        RestoreTarget::Floating { rect, layer } => {
            return float_panel(&without, panel, *rect, *layer);
        }
        RestoreTarget::Tab { panel: into } => {
            let live = locate_panel(&without, into).is_some_and(|l| l.place() != Where::Hidden);
            let is_stage = matches!(find_node(without.root.as_ref(), into), Some(Node::Stage(_)));
            if live || is_stage {
                return insert_panel(
                    &without,
                    panel,
                    &DockTarget::Into {
                        panel: into.clone(),
                        index: None,
                    },
                );
            }
        }
        RestoreTarget::Docked {
            beside,
            edge,
            share,
        } => {
            if find_node(without.root.as_ref(), beside).is_some() {
                let target = DockTarget::Beside {
                    beside: beside.clone(),
                    edge: *edge,
                    share: Some(*share),
                };
                return insert_panel(&without, panel, &target);
            }
        }
    }
    if without.root.is_some() {
        return float_panel(
            &without,
            panel,
            Rect::new(0.25, 0.2, 0.5, 0.6),
            fallback_layer,
        );
    }
    without.root = Some(Node::Panel(panel));
    without
}

struct Sanitizer<'a> {
    views: &'a BTreeMap<Id, ViewRecord>,
    is_known: &'a dyn Fn(&str) -> bool,
    seen_views: HashSet<Id>,
    seen_panels: HashSet<Id>,
    stages: usize,
}

impl Sanitizer<'_> {
    fn keep(&mut self, id: &Id) -> bool {
        let Some(record) = self.views.get(id) else {
            return false;
        };
        if self.seen_views.contains(id) || !(self.is_known)(&record.ty) {
            return false;
        }
        self.seen_views.insert(id.clone());
        true
    }

    fn panel(&mut self, panel: &PanelNode) -> Option<PanelNode> {
        if !self.seen_panels.insert(panel.id.clone()) {
            return None;
        }
        let views: Vec<Id> = panel
            .views
            .iter()
            .filter(|v| self.keep(v))
            .cloned()
            .collect();
        let selected = if views.contains(&panel.selected) {
            panel.selected.clone()
        } else {
            views.first()?.clone()
        };
        Some(PanelNode {
            id: panel.id.clone(),
            views,
            selected,
        })
    }

    fn node(&mut self, node: &Node) -> Option<Node> {
        match node {
            Node::Panel(p) => self.panel(p).map(Node::Panel),
            Node::Stage(s) => {
                self.stages += 1;
                if self.stages > 1 {
                    return s.child.as_ref().and_then(|c| self.node(c));
                }
                let child = s
                    .child
                    .as_ref()
                    .and_then(|c| self.node(c))
                    .filter(|c| !c.is_stage());
                Some(Node::Stage(StageNode {
                    id: s.id.clone(),
                    child: child.map(Box::new),
                }))
            }
            Node::Split(s) => {
                let mut children = Vec::new();
                let mut weights = Vec::new();
                for (i, child) in s.children.iter().enumerate() {
                    if let Some(fixed) = self.node(child) {
                        children.push(fixed);
                        let w = s.weights.get(i).copied().unwrap_or(1.0);
                        weights.push(if w.is_finite() && w > 0.0 { w } else { 1.0 });
                    }
                }
                if children.is_empty() {
                    return None;
                }
                normalize(Node::Split(SplitNode {
                    id: s.id.clone(),
                    axis: s.axis,
                    weights,
                    children,
                }))
            }
        }
    }
}

pub fn sanitize(input: &LayoutDocument, is_known_type: &dyn Fn(&str) -> bool) -> LayoutDocument {
    let mut s = Sanitizer {
        views: &input.views,
        is_known: is_known_type,
        seen_views: HashSet::new(),
        seen_panels: HashSet::new(),
        stages: 0,
    };
    let root = input
        .root
        .as_ref()
        .and_then(|r| s.node(r))
        .and_then(normalize);
    let floating: Vec<FloatingPanel> = input
        .floating
        .iter()
        .filter_map(|f| {
            s.panel(&f.panel).map(|panel| FloatingPanel {
                panel,
                rect: clamp_float(f.rect),
                z: f.z,
                layer: f.layer,
            })
        })
        .collect();
    let hidden: Vec<HiddenPanel> = input
        .hidden
        .iter()
        .filter_map(|h| {
            s.panel(&h.panel).map(|panel| HiddenPanel {
                panel,
                restore: h.restore.clone(),
            })
        })
        .collect();
    let seen = std::mem::take(&mut s.seen_views);
    let views = input
        .views
        .iter()
        .filter(|(id, _)| seen.contains(*id))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    LayoutDocument {
        schema: 1,
        version: input.version.clone(),
        root,
        floating,
        hidden,
        views,
        navigation: input.navigation.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::super::builder::{create_document, layout::*};
    use super::*;

    fn doc() -> LayoutDocument {
        create_document(
            Some(
                &row(
                    [
                        panel([view("a").id("a"), view("b").id("b")]).id("p").into(),
                        panel([view("c").id("c")]).id("q").into(),
                    ],
                    None,
                )
                .into(),
            ),
            vec![],
            None,
        )
        .unwrap()
    }

    #[test]
    fn detach_selects_a_neighbour() {
        let d = select_view(&doc(), "a");
        let d = detach_view(&d, "a");
        assert_eq!(panel_of_view(&d, "b").unwrap().selected.as_ref(), "b");
    }

    #[test]
    fn hide_then_restore_round_trips() {
        let d = doc();
        let hidden = hide_panel(&d, "q");
        assert!(locate_panel(&hidden, "q").is_some_and(|l| l.place() == Where::Hidden));
        let back = restore_panel(&hidden, "q", FloatingLayer::Overlay);
        assert!(locate_panel(&back, "q").is_some_and(|l| l.place() == Where::Docked));
    }

    #[test]
    fn sanitize_drops_unknown_types_and_empty_panels() {
        let d = doc();
        let clean = sanitize(&d, &|ty| ty != "c");
        assert!(locate_panel(&clean, "q").is_none());
        assert!(!clean.views.contains_key("c"));
    }

    #[test]
    fn document_round_trips_through_json() {
        let d = doc();
        let json = serde_json::to_string(&d).unwrap();
        let back: LayoutDocument = serde_json::from_str(&json).unwrap();
        assert_eq!(d, back);
    }
}
