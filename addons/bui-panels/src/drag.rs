use std::collections::HashMap;

use blenderui::prelude::{Context, Modifiers, Pixels, Point};

use super::{
    model::{
        document::{
            DockTarget, detach_view, float_panel, insert_panel, raise_float, remove_panel,
            reorder_tabs, select_view, uid,
        },
        resize::{BoundaryDrag, drag_boundary},
        spatial::{
            DockTargetSpec, apply_dock_target, drop_edge, drop_rects, frame_drop_target,
            seam_target, tile_drop_target,
        },
        tree::{
            Entries, Kind, contains_node, find_node, find_stage, leaf_ids, replace_node,
        },
        types::{
            Axis, Edge, FloatingLayer, FloatingPanel, Id, LayoutDocument, Node, PanelNode, Rect,
            SplitNode,
        },
    },
    motion::PICKUP_MS,
    workspace::{CommitOptions, Region, TABBAR_HEIGHT, Workspace},
};

pub(crate) const DROP_SLOT: &str = "__zopra-drop-slot";
pub(crate) const SOURCE_SLOT: &str = "__zopra-source-slot";
const THRESHOLD: f64 = 6.0;
const TAB_DETACH: f64 = 32.0;
const DIVIDER_SNAPS: [f64; 5] = [0.25, 1.0 / 3.0, 0.5, 2.0 / 3.0, 0.75];
const SETTLE_MS: f64 = 150.0;
const COMPACT: (f64, f64) = (380.0, 260.0);
const DESKTOP_SPLIT_MAX_PX: f64 = 64.0;
const MIN_SEAM_HIT_WIDTH: f64 = 8.0;
const OVERLAY_EDGE_BAND: f64 = 0.14;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DropTarget {
    Dock(DockTargetSpec),
    Tab { panel: Id, index: Option<usize> },
    Stage(Id),
    Float,
}

impl DropTarget {
    const fn kind(&self) -> &'static str {
        match self {
            Self::Dock(_) => "dock",
            Self::Tab { .. } => "tab",
            Self::Stage(_) => "stage",
            Self::Float => "float",
        }
    }
}

pub(crate) struct TabSort {
    pub row: Rect,
    pub items: Vec<(Id, f64, f64)>,
    pub order: Vec<Id>,
    pub grab: f64,
    pub started: bool,
    pub offsets: HashMap<Id, f64>,
}

#[allow(clippy::struct_excessive_bools)] // a drag's independent states
pub(crate) struct DragSession {
    pub start: (f64, f64),
    pub active: bool,
    pub origin: LayoutDocument,
    pub lifted: PanelNode,
    pub in_doc: bool,
    pub pending_tab: Option<Id>,
    pub press_view: Option<Id>,
    pub tab_sort: Option<TabSort>,
    pub excluded_host: Option<Id>,
    pub tab_origin: Option<Rect>,
    pub tab_rollback: bool,
    pub from_float: Option<FloatingLayer>,
    pub from_frame: bool,
    pub base: HashMap<Id, Rect>,
    pub source_screen: Rect,
    pub pdoc: LayoutDocument,
    pub drop_base: Entries,
    pub collapsed: bool,
    pub layout_targets: HashMap<Id, Rect>,
    pub floating: Rect,
    pickup_at: f64,
    compact_from: f64,
    compact_to: f64,
    compact_amount: f64,
    desktop_amount: f64,
    size_tick: f64,
    pub pointer: (f64, f64),
    grab_x: f64,
    grab_y: f64,
    pub target: Option<DropTarget>,
    pub candidate: Option<DropTarget>,
    settle_at: Option<f64>,
    animating: bool,
    pub drop_label: &'static str,
}

pub(crate) struct DividerDrag {
    pub split: Id,
    pub index: usize,
    pub grab: f64,
    pub origin: LayoutDocument,
    pub push: BoundaryDrag,
}

#[derive(Clone, Copy, Default)]
#[allow(clippy::struct_excessive_bools)] // one flag per edge
pub(crate) struct ResizeDir {
    pub n: bool,
    pub s: bool,
    pub e: bool,
    pub w: bool,
}

pub(crate) struct FloatResize {
    pub panel: Id,
    pub dir: ResizeDir,
    pub start: Rect,
    pub start_point: (f64, f64),
    pub origin: LayoutDocument,
    pub layer: FloatingLayer,
}

fn pf(p: Pixels) -> f64 {
    f64::from(f32::from(p))
}

fn overlay_edge(p: (f64, f64), r: Rect) -> Option<Edge> {
    let (x, y) = ((p.0 - r.x) / r.w, (p.1 - r.y) / r.h);
    let mut edges = [
        (Edge::Left, x),
        (Edge::Right, 1.0 - x),
        (Edge::Top, y),
        (Edge::Bottom, 1.0 - y),
    ];
    edges.sort_by(|a, b| a.1.total_cmp(&b.1));
    (edges[0].1 <= OVERLAY_EDGE_BAND).then_some(edges[0].0)
}

impl Workspace {
    pub(crate) fn local(&self, pos: Point<Pixels>) -> (f64, f64) {
        (pf(pos.x) - self.bounds.x, pf(pos.y) - self.bounds.y)
    }

    pub(crate) const fn capturing(&self) -> bool {
        self.drag.is_some()
            || self.divider.is_some()
            || self.resize.is_some()
            || self.nav.gesture_drag.is_some()
    }

    pub(crate) fn panel_pointer_down(&mut self, panel_id: &Id, cx: &mut Context<Self>) {
        if let Some(panel) = self.find_panel(panel_id) {
            self.focus_view(&panel.selected, cx);
        }
        if self.doc.floating.iter().any(|f| &f.panel.id == panel_id) {
            let raised = raise_float(&self.doc, panel_id);
            if raised != self.doc {
                self.commit(
                    raised,
                    CommitOptions {
                        animate: Some(false),
                        silent: true,
                        from: None,
                    },
                    cx,
                );
            }
        }
    }

    /// A press on a tab or a tab bar: a double click frames the panel, else a drag may start.
    /// False (the press goes on to the workspace) when a modifier is held.
    pub(crate) fn tab_pointer_down(
        &mut self,
        panel_id: &Id,
        view: Option<Id>,
        pos: Point<Pixels>,
        click_count: usize,
        mods: Modifiers,
        cx: &mut Context<Self>,
    ) -> bool {
        if mods.alt || mods.shift || mods.control {
            return false;
        }
        if click_count >= 2 {
            self.toggle_frame(panel_id, cx);
            return true;
        }
        if let Some(panel) = self.find_panel(panel_id) {
            self.drag_begin(panel, view, pos, cx);
        }
        true
    }

    fn drag_begin(
        &mut self,
        panel: PanelNode,
        view: Option<Id>,
        pos: Point<Pixels>,
        cx: &mut Context<Self>,
    ) {
        if self.capturing() {
            return;
        }
        let float = self
            .doc
            .floating
            .iter()
            .find(|f| f.panel.id == panel.id)
            .map(|f| f.layer);
        let tab = view.clone().filter(|_| panel.views.len() > 1);
        let from_frame = self.fx.frame_only.contains(&panel.id);
        let screen = self
            .last_rects
            .get(&panel.id)
            .copied()
            .unwrap_or(Rect::ZERO);
        let p = self.local(pos);
        let window_pos = (pf(pos.x), pf(pos.y));

        let tab_sort = tab.as_ref().map(|tab_id| {
            let items: Vec<(Id, f64, f64)> = panel
                .views
                .iter()
                .filter_map(|id| self.tab_rects.get(id).map(|r| (id.clone(), r.x, r.w)))
                .collect();
            let grab = items
                .iter()
                .find(|(id, ..)| id == tab_id)
                .map_or(0.0, |(_, x, _)| window_pos.0 - x);
            TabSort {
                row: Rect::new(
                    screen.x + self.bounds.x,
                    screen.y + self.bounds.y,
                    screen.w,
                    TABBAR_HEIGHT,
                ),
                order: items.iter().map(|(id, ..)| id.clone()).collect(),
                items,
                grab,
                started: false,
                offsets: HashMap::new(),
            }
        });

        self.drag = Some(DragSession {
            start: window_pos,
            active: false,
            origin: self.doc.clone(),
            lifted: panel,
            in_doc: true,
            pending_tab: tab,
            press_view: view,
            tab_sort,
            excluded_host: None,
            tab_origin: None,
            tab_rollback: false,
            from_float: float,
            from_frame,
            base: HashMap::new(),
            source_screen: screen,
            pdoc: self.doc.clone(),
            drop_base: Entries::default(),
            collapsed: false,
            layout_targets: HashMap::new(),
            floating: screen,
            pickup_at: 0.0,
            compact_from: 0.0,
            compact_to: 0.0,
            compact_amount: 0.0,
            desktop_amount: 0.0,
            size_tick: self.now(),
            pointer: p,
            grab_x: if screen.w > 0.0 {
                (p.0 - screen.x) / screen.w
            } else {
                0.5
            },
            grab_y: p.1 - screen.y,
            target: None,
            candidate: None,
            settle_at: None,
            animating: true,
            drop_label: "",
        });
        cx.notify();
    }

    pub(crate) fn on_pointer_move(
        &mut self,
        pos: Point<Pixels>,
        mods: Modifiers,
        cx: &mut Context<Self>,
    ) {
        if self.nav.gesture_drag.is_some() {
            return self.nav_gesture_move(pos, cx);
        }
        if self.divider.is_some() {
            return self.divider_move(pos, mods.shift, cx);
        }
        if self.resize.is_some() {
            return self.resize_move(pos, cx);
        }
        if self.drag.is_some() {
            self.drag_move(pos, cx);
        }
    }

    pub(crate) fn on_pointer_up(&mut self, cx: &mut Context<Self>) {
        if self.nav.gesture_drag.is_some() {
            return self.nav_gesture_end(true, cx);
        }
        if self.divider.is_some() {
            return self.divider_end(cx);
        }
        if self.resize.is_some() {
            return self.resize_end(cx);
        }
        self.nav_edge_end(cx);
        self.drag_end(true, cx);
    }

    pub(crate) fn on_escape(&mut self, cx: &mut Context<Self>) {
        if self.drag.is_some() {
            self.nav_edge_end(cx);
            self.drag_end(false, cx);
        } else if self.nav.gesture_drag.is_some() {
            self.nav_gesture_end(false, cx);
        }
    }

    fn sort_tab_at(d: &mut DragSession, client_x: f64) {
        let (Some(sort), Some(dragged_id)) = (d.tab_sort.as_mut(), d.pending_tab.clone()) else {
            return;
        };
        sort.started = true;
        let Some(&(_, _, dragged_w)) = sort.items.iter().find(|(id, ..)| *id == dragged_id) else {
            return;
        };
        let left = (client_x - sort.grab)
            .min(sort.row.right() - dragged_w)
            .max(sort.row.x);
        let find = |items: &[(Id, f64, f64)], id: &Id| {
            items
                .iter()
                .find(|(i, ..)| i == id)
                .map(|&(_, x, w)| (x, w))
        };

        let first_x = sort.items[0].1;
        let mut x = first_x;
        let mut index = 0;
        for id in &sort.order {
            let Some((_, w)) = find(&sort.items, id) else {
                continue;
            };
            if *id != dragged_id && client_x > x + w / 2.0 {
                index += 1;
            }
            x += w;
        }
        sort.order.retain(|id| *id != dragged_id);
        sort.order
            .insert(index.min(sort.order.len()), dragged_id.clone());

        x = first_x;
        sort.offsets.clear();
        for id in &sort.order {
            let Some((orig_x, w)) = find(&sort.items, id) else {
                continue;
            };
            let target = if *id == dragged_id { left } else { x };
            sort.offsets.insert(id.clone(), target - orig_x);
            x += w;
        }
    }

    fn drag_move(&mut self, pos: Point<Pixels>, cx: &mut Context<Self>) {
        let Some(mut d) = self.drag.take() else {
            return;
        };
        let wp = (pf(pos.x), pf(pos.y));
        if !d.active {
            if (wp.0 - d.start.0).hypot(wp.1 - d.start.1) < THRESHOLD {
                self.drag = Some(d);
                return;
            }
            if d.pending_tab.is_some() && d.tab_sort.is_some() {
                let inside_row = d
                    .tab_sort
                    .as_ref()
                    .is_some_and(|s| s.row.inset(-TAB_DETACH).contains(wp.0, wp.1));
                if inside_row {
                    Self::sort_tab_at(&mut d, wp.0);
                    if let (Some(sort), Some(dragged)) =
                        (d.tab_sort.as_ref(), d.pending_tab.as_ref())
                    {
                        let now = self.now();
                        self.fx.tab_motion.sort_to(&sort.offsets, dragged, now);
                    }
                    self.drag = Some(d);
                    self.wake(cx);
                    return;
                }
                if let Some(sort) = d.tab_sort.as_mut() {
                    sort.offsets.clear();
                }
                self.fx.tab_motion.clear_sort();
            }
            self.activate(&mut d);
        }
        d.pointer = self.local(pos);
        let next = self.resolve_target(&d);
        self.settle(&mut d, next);
        self.drag = Some(d);
        self.wake(cx);
    }

    fn activate(&mut self, d: &mut DragSession) {
        d.active = true;
        self.lifted = Some(d.lifted.clone());
        let mut doc = self.doc.clone();

        if let Some(tab) = d.pending_tab.take() {
            let origin = self
                .last_rects
                .get(&d.lifted.id)
                .copied()
                .unwrap_or(d.source_screen);
            let host_panel = d.lifted.clone();
            let order = d
                .tab_sort
                .as_ref()
                .map_or_else(|| host_panel.views.clone(), |s| s.order.clone());
            doc = reorder_tabs(&doc, &host_panel.id, &order);
            doc = detach_view(&doc, &tab);
            d.lifted = PanelNode {
                id: uid("panel"),
                views: vec![tab.clone()],
                selected: tab,
            };
            d.in_doc = false;
            d.excluded_host = Some(host_panel.id);
            d.tab_origin = Some(origin);
            d.tab_rollback = true;
            d.from_float = d.from_float.or(Some(FloatingLayer::Stage));
            d.from_frame = false;
            d.source_screen = origin;
            d.tab_sort = None;
            self.lifted = Some(d.lifted.clone());
            self.set_doc(doc.clone());
        }
        d.pdoc = doc;
        d.drop_base = self.layout(d.pdoc.root.as_ref());
        d.base = d.drop_base.iter().map(|e| (e.id.clone(), e.rect)).collect();
        d.layout_targets = self.resting_layout(d);
        d.pickup_at = self.now();
        d.floating = d.source_screen;
    }

    fn tab_target(&self, panel: &PanelNode, p: (f64, f64), screen: Rect) -> Option<DropTarget> {
        let bar = self.tabbar_height_of(panel);
        if bar == 0.0
            || !(p.0 >= screen.x
                && p.0 <= screen.right()
                && p.1 >= screen.y
                && p.1 <= screen.y + bar)
        {
            return None;
        }
        let mut index = panel.views.len();
        for (i, view) in panel.views.iter().enumerate() {
            let Some(r) = self.tab_rects.get(view) else { continue };
            if p.0 < r.x - self.bounds.x + r.w / 2.0 {
                index = i;
                break;
            }
        }
        Some(DropTarget::Tab {
            panel: panel.id.clone(),
            index: Some(index),
        })
    }

    fn resolve_target(&self, d: &DragSession) -> Option<DropTarget> {
        let (vw, vh) = self.viewport;
        let p = d.pointer;
        let point = self.to_world(p);
        let within = p.0 >= 0.0 && p.1 >= 0.0 && p.0 <= vw && p.1 <= vh;
        let layer = self.floating_layer();
        let doc = &d.pdoc;
        let stage = find_stage(doc.root.as_ref());
        let stage_is_desktop =
            stage.is_some_and(|s| s.child.is_none()) && layer == Some(FloatingLayer::Stage);
        let overlay_move = d.from_float == Some(FloatingLayer::Overlay);
        let stage_world = stage
            .filter(|s| s.child.is_some())
            .and_then(|s| d.drop_base.rect(&s.id));
        let stage_move = d.from_float == Some(FloatingLayer::Stage)
            && !d.tab_rollback
            && stage_world.is_some_and(|w| w.contains(point.0, point.1));
        let float_move = overlay_move || stage_move;
        let mut next: Option<DropTarget> = None;

        if within {
            let mut overlays: Vec<&FloatingPanel> = doc
                .floating
                .iter()
                .filter(|f| f.layer == FloatingLayer::Overlay && f.panel.id != d.lifted.id)
                .collect();
            overlays.sort_by_key(|a| std::cmp::Reverse(a.z));
            for f in overlays {
                let Some(r) = self.last_rects.get(&f.panel.id).copied() else {
                    continue;
                };
                if !r.contains(p.0, p.1) {
                    continue;
                }
                let target = self.tab_target(&f.panel, p, r).or_else(|| {
                    if drop_edge(p, r).is_some() {
                        None
                    } else {
                        Some(DropTarget::Tab {
                            panel: f.panel.id.clone(),
                            index: None,
                        })
                    }
                });
                return self.filter_target(d, target);
            }
        }

        let hovered: Option<Id> = within
            .then(|| {
                leaf_ids(doc.root.as_ref())
                    .into_iter()
                    .filter(|id| *id != d.lifted.id)
                    .find(|id| {
                        d.drop_base
                            .rect(id)
                            .is_some_and(|r| r.contains(point.0, point.1))
                    })
            })
            .flatten();
        let collapsed = hovered
            .as_ref()
            .and_then(|h| self.fx.collapsed_of.get(h).cloned());
        if let Some(group) = &collapsed {
            let edge = d.drop_base.rect(group).and_then(|r| drop_edge(point, r));
            next = edge.map(|e| DropTarget::Dock(tile_drop_target(&d.drop_base, group, e)));
        } else if let Some(hovered) = &hovered
            && let Some(Node::Panel(panel)) = find_node(doc.root.as_ref(), hovered)
        {
            let world = d.drop_base.rect(hovered).unwrap_or(Rect::ZERO);
            let screen = self.panel_screen(world);
            next = self.tab_target(panel, p, screen).or_else(|| {
                let edge = if float_move {
                    overlay_edge(p, screen)
                } else {
                    drop_edge(point, world)
                };
                match edge {
                    Some(e) => Some(DropTarget::Dock(tile_drop_target(&d.drop_base, hovered, e))),
                    None if float_move => Some(DropTarget::Float),
                    None => Some(DropTarget::Tab {
                        panel: hovered.clone(),
                        index: None,
                    }),
                }
            });
        }

        let radius = self.pad().max(MIN_SEAM_HIT_WIDTH / 2.0);
        let mut closest = radius;
        if within {
            for entry in d.drop_base.iter() {
                let Kind::Split { axis, children, .. } = &entry.kind else {
                    continue;
                };
                if self.fx.hidden_splits.contains(&entry.id)
                    || !self.to_screen(entry.rect).contains(p.0, p.1)
                {
                    continue;
                }
                if !d
                    .drop_base
                    .leaf_ids(&entry.id)
                    .iter()
                    .any(|l| *l != d.lifted.id)
                {
                    continue;
                }
                for (index, child) in children.iter().enumerate().skip(1) {
                    let Some(r) = d.drop_base.rect(child).map(|r| self.to_screen(r)) else {
                        continue;
                    };
                    let distance = if *axis == Axis::X {
                        (p.0 - r.x).abs()
                    } else {
                        (p.1 - r.y).abs()
                    };
                    if distance >= closest {
                        continue;
                    }
                    closest = distance;
                    next = seam_target(&d.drop_base, &entry.id, index).map(DropTarget::Dock);
                }
            }
        }

        if let (true, Some(stage)) = (
            within && closest == radius,
            stage.filter(|s| s.child.is_none()),
        ) && let Some(world) = d
            .drop_base
            .rect(&stage.id)
            .filter(|w| w.contains(point.0, point.1))
        {
            let s = self.panel_screen(world);
            let mut distances = [
                (Edge::Left, p.0 - s.x),
                (Edge::Right, s.right() - p.0),
                (Edge::Top, p.1 - s.y),
                (Edge::Bottom, s.bottom() - p.1),
            ];
            distances.sort_by(|a, b| a.1.total_cmp(&b.1));
            let dimension = if distances[0].0.axis() == Axis::X {
                s.w
            } else {
                s.h
            };
            let near_edge = distances[0].1 <= DESKTOP_SPLIT_MAX_PX.min(dimension * 0.28);
            next = Some(if near_edge {
                DropTarget::Dock(tile_drop_target(&d.drop_base, &stage.id, distances[0].0))
            } else if stage_is_desktop {
                DropTarget::Float
            } else {
                DropTarget::Stage(stage.id.clone())
            });
        }

        if matches!(next, Some(DropTarget::Float)) && stage_is_desktop {
            let mut floats: Vec<&FloatingPanel> = doc
                .floating
                .iter()
                .filter(|f| f.layer == FloatingLayer::Stage)
                .collect();
            floats.sort_by_key(|a| std::cmp::Reverse(a.z));
            for f in floats.into_iter().filter(|f| f.panel.id != d.lifted.id) {
                let Some(r) = self
                    .float_world(&f.panel.id)
                    .filter(|r| r.contains(point.0, point.1))
                else {
                    continue;
                };
                next = self
                    .last_rects
                    .get(&f.panel.id)
                    .and_then(|s| self.tab_target(&f.panel, p, *s))
                    .or_else(|| {
                        if drop_edge(point, r).is_some() {
                            next.clone()
                        } else {
                            Some(DropTarget::Tab {
                                panel: f.panel.id.clone(),
                                index: None,
                            })
                        }
                    });
                break;
            }
        }

        let group = self
            .nav
            .framed
            .clone()
            .or_else(|| doc.root.as_ref().map(|r| r.id().clone()));
        if let Some(group) = group.filter(|g| self.nav.entries.contains(g)) {
            let frame = frame_drop_target(
                &self.nav.entries,
                &group,
                p,
                vw,
                vh,
                (self.pad() * 2.0).max(MIN_SEAM_HIT_WIDTH),
            );
            if let Some(frame) = frame
                && self
                    .nav
                    .entries
                    .leaf_ids(&group)
                    .iter()
                    .any(|id| *id != d.lifted.id)
            {
                next = Some(DropTarget::Dock(frame));
            }
        }

        if let Some(DropTarget::Tab { panel, .. }) = &next
            && Some(panel) == d.excluded_host.as_ref()
        {
            let host_is_stage_float = doc.floating.iter().any(|f| {
                Some(&f.panel.id) == d.excluded_host.as_ref() && f.layer == FloatingLayer::Stage
            });
            next = host_is_stage_float.then_some(DropTarget::Float);
        }
        if next.is_none() && within && float_move {
            next = Some(DropTarget::Float);
        }
        self.filter_target(d, next)
    }

    fn filter_target(&self, d: &DragSession, next: Option<DropTarget>) -> Option<DropTarget> {
        let next = next?;
        let views = &d.lifted.views;
        match &next {
            DropTarget::Float => {
                return (self.floating_layer().is_some() && self.allowed(views, Region::Floating))
                    .then_some(next);
            }
            DropTarget::Stage(_) => return self.allowed(views, Region::Stage).then_some(next),
            _ => {}
        }
        let doc = &d.pdoc;
        let stage = find_stage(doc.root.as_ref());
        let in_stage = |id: &Id| {
            stage.is_some_and(|s| {
                s.id != *id && s.child.as_deref().is_some_and(|c| contains_node(c, id))
            })
        };
        let region = match &next {
            DropTarget::Tab { panel, .. } => {
                if doc.floating.iter().any(|f| f.panel.id == *panel) {
                    Region::Floating
                } else if in_stage(panel) {
                    Region::Stage
                } else {
                    Region::Side
                }
            }
            DropTarget::Dock(spec) => {
                let stage_side = match &spec.seam {
                    Some(seam) => {
                        !seam.leaves.is_empty()
                            && seam.leaves.iter().all(&in_stage)
                            && in_stage(&spec.id)
                    }
                    None => in_stage(&spec.id),
                };
                if stage_side {
                    Region::Stage
                } else {
                    Region::Side
                }
            }
            _ => Region::Side,
        };
        self.allowed(views, region).then_some(next)
    }

    fn settle(&mut self, d: &mut DragSession, next: Option<DropTarget>) {
        if next == d.candidate {
            return;
        }
        let settled = next == d.target;
        d.candidate = next;
        d.settle_at = None;
        if settled {
            return;
        }
        d.settle_at = Some(self.now() + SETTLE_MS);
    }

    fn world_of(&self, id: &str) -> Option<Rect> {
        self.rect_to_world(self.last_rects.get(id).copied())
    }

    fn current_world(&self, d: &DragSession, id: &str) -> Option<Rect> {
        if let Some(own) = self.world_of(id) {
            return Some(own);
        }
        let rects: Vec<Rect> = find_node(d.pdoc.root.as_ref(), id)
            .map(|n| {
                leaf_ids(Some(n))
                    .iter()
                    .filter_map(|l| self.world_of(l))
                    .collect()
            })
            .unwrap_or_default();
        if rects.is_empty() {
            return d.drop_base.rect(id);
        }
        let x = rects.iter().map(|r| r.x).fold(f64::INFINITY, f64::min);
        let y = rects.iter().map(|r| r.y).fold(f64::INFINITY, f64::min);
        Some(Rect {
            x,
            y,
            w: rects
                .iter()
                .map(super::model::types::Rect::right)
                .fold(f64::NEG_INFINITY, f64::max)
                - x,
            h: rects
                .iter()
                .map(super::model::types::Rect::bottom)
                .fold(f64::NEG_INFINITY, f64::max)
                - y,
        })
    }

    fn seam_origin(&self, d: &DragSession, preview: &Entries, slot: Rect) -> Rect {
        let parent = preview
            .get(DROP_SLOT)
            .and_then(|e| e.parent.clone())
            .and_then(|id| preview.get(&id));
        let collapsed_at_center = match parent.map(|p| &p.kind) {
            Some(Kind::Split { axis: Axis::Y, .. }) => Rect {
                y: slot.y + slot.h / 2.0,
                h: 0.0,
                ..slot
            },
            _ => Rect {
                x: slot.x + slot.w / 2.0,
                w: 0.0,
                ..slot
            },
        };
        let Some(Kind::Split { axis, children, .. }) = parent.map(|p| &p.kind) else {
            return collapsed_at_center;
        };
        let Some(index) = children.iter().position(|c| c.as_ref() == DROP_SLOT) else {
            return collapsed_at_center;
        };
        let after = children.get(index + 1);
        let before = index.checked_sub(1).and_then(|i| children.get(i));
        let neighbour = after.or(before).and_then(|c| self.current_world(d, c));
        let Some(n) = neighbour else {
            return collapsed_at_center;
        };
        match axis {
            Axis::X => Rect {
                x: if after.is_some() { n.x } else { n.right() },
                w: 0.0,
                ..slot
            },
            Axis::Y => Rect {
                y: if after.is_some() { n.y } else { n.bottom() },
                h: 0.0,
                ..slot
            },
        }
    }

    fn resting_layout(&self, d: &DragSession) -> HashMap<Id, Rect> {
        let mut rects = d.base.clone();
        for e in d.drop_base.iter() {
            rects.insert(e.id.clone(), e.rect);
        }
        if d.collapsed {
            rects.insert(SOURCE_SLOT.into(), self.collapsed_source(d));
        } else if d.in_doc
            && d.from_float.is_none()
            && let Some(r) = d.base.get(&d.lifted.id)
        {
            rects.insert(SOURCE_SLOT.into(), *r);
        }
        rects
    }

    fn collapsed_source(&self, d: &DragSession) -> Rect {
        let entries = self.layout(d.origin.root.as_ref());
        let entry = entries.get(&d.lifted.id);
        let mut rect = d
            .base
            .get(&d.lifted.id)
            .copied()
            .or_else(|| entry.map(|e| e.rect))
            .unwrap_or(Rect::ZERO);
        let parent = entry
            .and_then(|e| e.parent.as_ref())
            .and_then(|p| entries.get(p));
        let Some(Kind::Split { axis, children, .. }) = parent.map(|p| &p.kind) else {
            return Rect { w: 0.0, ..rect };
        };
        let has_next = children
            .iter()
            .position(|c| *c == d.lifted.id)
            .is_some_and(|i| i + 1 < children.len());
        match axis {
            Axis::X => {
                if !has_next {
                    rect.x += rect.w;
                }
                rect.w = 0.0;
            }
            Axis::Y => {
                if !has_next {
                    rect.y += rect.h;
                }
                rect.h = 0.0;
            }
        }
        rect
    }

    fn apply_target(&mut self, d: &mut DragSession, next: Option<DropTarget>) {
        if next == d.target {
            return;
        }
        let mut previous = self.last_rects.clone();
        let has_parent = self
            .layout(d.origin.root.as_ref())
            .get(&d.lifted.id)
            .is_some_and(|e| e.parent.is_some());
        if next.is_some() && d.in_doc && d.from_float.is_none() && !d.collapsed && has_parent {
            d.collapsed = true;
            d.pdoc = remove_panel(&d.pdoc, &d.lifted.id);
            d.drop_base = self.layout(d.pdoc.root.as_ref());
        }
        d.layout_targets = self.resting_layout(d);
        d.drop_label = "";

        match &next {
            Some(DropTarget::Tab { panel, .. }) => {
                let world = d
                    .drop_base
                    .rect(panel)
                    .or_else(|| self.float_world(panel))
                    .or_else(|| self.world_of(panel));
                if let Some(world) = world {
                    d.layout_targets.insert(DROP_SLOT.into(), world);
                    previous.insert(DROP_SLOT.into(), self.panel_screen(world));
                } else if let Some(screen) = self.last_rects.get(panel.as_ref()).copied() {
                    if let Some(w) = self.rect_to_world(Some(screen)) {
                        d.layout_targets.insert(DROP_SLOT.into(), w);
                    }
                    previous.insert(DROP_SLOT.into(), screen);
                }
                d.drop_label = "Add as tab";
            }
            Some(DropTarget::Stage(stage)) => {
                if let Some(world) = d.drop_base.rect(stage) {
                    d.layout_targets.insert(DROP_SLOT.into(), world);
                    let (cx_, cy_) = world.center();
                    previous.insert(
                        DROP_SLOT.into(),
                        self.panel_screen(Rect::new(cx_, cy_, 0.0, 0.0)),
                    );
                }
            }
            Some(DropTarget::Dock(spec)) => {
                let placeholder = PanelNode {
                    id: DROP_SLOT.into(),
                    views: vec![DROP_SLOT.into()],
                    selected: DROP_SLOT.into(),
                };
                if let (None, Some(rect)) = (&spec.seam, d.drop_base.rect(&spec.id)) {
                    let r = drop_rects(rect, spec.edge);
                    d.layout_targets.insert(spec.id.clone(), r.remaining);
                    d.layout_targets.insert(DROP_SLOT.into(), r.slot);
                    let from = self
                        .current_world(d, &spec.id)
                        .map_or(r.collapsed, |current| {
                            drop_rects(current, spec.edge).collapsed
                        });
                    previous.insert(DROP_SLOT.into(), self.panel_screen(from));
                } else {
                    let root = apply_dock_target(
                        d.pdoc.root.clone(),
                        spec,
                        Node::Panel(placeholder),
                        "__zopra-seam-preview".into(),
                    );
                    let preview = self.layout(Some(&root));
                    for e in preview.iter() {
                        d.layout_targets.insert(e.id.clone(), e.rect);
                    }
                    if let Some(slot) = preview.rect(DROP_SLOT) {
                        let origin = self.seam_origin(d, &preview, slot);
                        previous.insert(DROP_SLOT.into(), self.panel_screen(origin));
                    }
                }
            }
            None => {
                if let Some(old) = d.target.as_ref().filter(|t| t.kind() != "float") {
                    let world = previous
                        .get(DROP_SLOT)
                        .and_then(|s| self.rect_to_world(Some(*s)));
                    if let Some(world) = world {
                        let edge = if let DropTarget::Dock(spec) = old {
                            spec.edge
                        } else {
                            Edge::Left
                        };
                        let x_axis = edge.axis() == Axis::X;
                        let home = match old {
                            DropTarget::Dock(spec) => match spec.boundary {
                                Some(b) => match d.drop_base.get(&spec.id).map(|e| &e.kind) {
                                    Some(Kind::Split { children, .. }) => {
                                        let rect = |c| d.layout_targets.get(c).copied();
                                        match children.get(b) {
                                            Some(next) => rect(next)
                                                .map(|r| if x_axis { r.x } else { r.y }),
                                            None => b
                                                .checked_sub(1)
                                                .and_then(|i| children.get(i))
                                                .and_then(rect)
                                                .map(|r| {
                                                    if x_axis { r.right() } else { r.bottom() }
                                                }),
                                        }
                                    }
                                    _ => None,
                                },
                                None => Some(match spec.edge {
                                    Edge::Left => world.x,
                                    Edge::Top => world.y,
                                    Edge::Right => world.right(),
                                    Edge::Bottom => world.bottom(),
                                }),
                            },
                            _ => None,
                        };
                        let collapsed = if x_axis {
                            Rect {
                                x: home.unwrap_or(world.x + world.w / 2.0),
                                w: 0.0,
                                ..world
                            }
                        } else {
                            Rect {
                                y: home.unwrap_or(world.y + world.h / 2.0),
                                h: 0.0,
                                ..world
                            }
                        };
                        d.layout_targets.insert(DROP_SLOT.into(), collapsed);
                    }
                }
            }
            Some(DropTarget::Float) => {}
        }
        if d.collapsed
            && !previous.contains_key(SOURCE_SLOT)
            && let Some(base) = d.base.get(&d.lifted.id)
        {
            previous.insert(SOURCE_SLOT.into(), self.panel_screen(*base));
        }
        d.target = next;
        if !self.reduced() {
            let now = self.now();
            self.tween.begin(previous, now);
        }
    }

    pub(crate) fn drag_needs_frames(&self) -> bool {
        self.drag
            .as_ref()
            .is_some_and(|d| d.active && (d.animating || d.settle_at.is_some()))
            || self.nav_edge_pending()
    }

    pub(crate) fn step_drag(&mut self, now: f64, dt: f64, cx: &mut Context<Self>) {
        let Some(mut d) = self.drag.take() else {
            return;
        };
        if d.active {
            if self.nav_edge_scroll(d.pointer, now, dt) {
                let next = self.resolve_target(&d);
                self.settle(&mut d, next);
                cx.notify();
            }
            self.update_lifted(&mut d, now);
            if d.settle_at.is_some_and(|at| now >= at) {
                d.settle_at = None;
                let candidate = d.candidate.clone();
                self.apply_target(&mut d, candidate);
                cx.notify();
            }
        }
        self.drag = Some(d);
    }

    fn update_lifted(&self, d: &mut DragSession, now: f64) {
        let original = d.source_screen;
        let stage = find_stage(d.pdoc.root.as_ref());
        let desktop = stage
            .filter(|s| s.child.is_none() && self.floating_layer() == Some(FloatingLayer::Stage))
            .and_then(|s| d.drop_base.rect(&s.id))
            .map(|r| self.panel_screen(r));
        let viewport = Rect::new(0.0, 0.0, self.viewport.0, self.viewport.1);
        let source = if d.collapsed {
            None
        } else {
            d.tab_origin
                .or_else(|| match (d.from_float.filter(|_| !d.from_frame), desktop) {
                    (Some(_), Some(desk)) => Some(desk),
                    (Some(FloatingLayer::Overlay), None) => Some(viewport),
                    _ => Some(original),
                })
        };
        let amount = if self.reduced() {
            1.0
        } else {
            1.0 - (-(now - d.size_tick) / 60.0).exp()
        };
        d.size_tick = now;
        let over = |r: Option<Rect>| r.is_some_and(|r| r.contains(d.pointer.0, d.pointer.1));
        d.desktop_amount += ((if over(desktop) { 1.0 } else { 0.0 }) - d.desktop_amount) * amount;
        let compact_to = if over(source) || (!d.from_frame && over(desktop)) {
            0.0
        } else {
            1.0
        };
        if compact_to != d.compact_to {
            d.compact_from = d.compact_amount;
            d.compact_to = compact_to;
            d.pickup_at = now;
        }
        let progress = if self.reduced() {
            1.0
        } else {
            ((now - d.pickup_at) / PICKUP_MS).min(1.0)
        };
        let desktop_target = if over(desktop) { 1.0 } else { 0.0 };
        d.animating = progress < 1.0 || (desktop_target - d.desktop_amount).abs() > 1e-3;
        let eased = 1.0 - (1.0 - progress).powi(3);
        d.compact_amount = d.compact_from + (d.compact_to - d.compact_from) * eased;
        let target_scale = 1.0_f64
            .min(COMPACT.0 / original.w.max(1.0))
            .min(COMPACT.1 / original.h.max(1.0));
        let scale = 1.0 + (target_scale - 1.0) * d.compact_amount;
        let (w, h) = if d.from_frame {
            (
                original.w + (COMPACT.0 - original.w) * d.compact_amount,
                original.h + (COMPACT.1 - original.h) * d.compact_amount,
            )
        } else {
            (original.w * scale, original.h * scale)
        };
        let grab_y = d.grab_y + (d.grab_y.min(18.0) - d.grab_y) * d.compact_amount;
        d.floating = Rect {
            x: d.pointer.0 - w * d.grab_x,
            y: d.pointer.1 - grab_y,
            w,
            h,
        };
    }

    pub(crate) fn drag_end(&mut self, commit: bool, cx: &mut Context<Self>) {
        let Some(mut d) = self.drag.take() else {
            return;
        };
        if commit && d.active {
            let candidate = d.candidate.clone();
            self.apply_target(&mut d, candidate);
        }
        if !d.active {
            return self.end_pending(d, commit, cx);
        }
        self.lifted = None;
        self.suppress_clicks_until = self.now() + 350.0;
        let lifted = d.lifted.clone();
        let mut from = self.last_rects.clone();
        from.insert(lifted.id.clone(), d.floating);
        let target = if commit { d.target.clone() } else { None };

        let Some(target) = target else {
            self.restore_after_drag(d.origin.clone(), from, Some(lifted.id.clone()), cx);
            return;
        };
        let base = self.doc.clone();
        let without = if d.in_doc {
            remove_panel(&base, &lifted.id)
        } else {
            base
        };
        let layer = self.floating_layer().unwrap_or(FloatingLayer::Overlay);
        let next = match &target {
            DropTarget::Tab { panel, index } => insert_panel(
                &without,
                lifted.clone(),
                &DockTarget::Into {
                    panel: panel.clone(),
                    index: *index,
                },
            ),
            DropTarget::Stage(stage) => insert_panel(
                &without,
                lifted.clone(),
                &DockTarget::Into {
                    panel: stage.clone(),
                    index: None,
                },
            ),
            DropTarget::Dock(spec) => {
                let mut next = without.clone();
                next.root = Some(apply_dock_target(
                    without.root,
                    spec,
                    Node::Panel(lifted.clone()),
                    uid("split"),
                ));
                next
            }
            DropTarget::Float => {
                let float_layer = if d.from_float == Some(FloatingLayer::Overlay) {
                    FloatingLayer::Overlay
                } else {
                    layer
                };
                let rect = self.float_rect(&d, &without, float_layer);
                float_panel(&without, lifted.clone(), rect, float_layer)
            }
        };
        let repositioning =
            matches!(target, DropTarget::Float) && d.from_float.is_some() && !d.tab_rollback;
        let settle = if repositioning || matches!(target, DropTarget::Tab { .. }) {
            None
        } else {
            Some(lifted.id.clone())
        };
        self.restore_after_drag(d.origin.clone(), from.clone(), None, cx);
        if let Some(id) = settle {
            self.settling.insert(id);
        }
        self.commit(
            next,
            CommitOptions {
                from: Some(from),
                ..Default::default()
            },
            cx,
        );
        self.after_move(
            &lifted,
            d.from_float.is_some() && !d.tab_rollback,
            target.kind(),
            cx,
        );
    }

    fn restore_after_drag(
        &mut self,
        origin: LayoutDocument,
        from: HashMap<Id, Rect>,
        settle: Option<Id>,
        cx: &mut Context<Self>,
    ) {
        self.set_doc(origin);
        if let Some(id) = settle {
            self.settling.insert(id);
            if !self.reduced() {
                let now = self.now();
                self.tween.begin(from, now);
            }
            self.sync();
        }
        self.wake(cx);
    }

    fn after_move(
        &mut self,
        panel: &PanelNode,
        from_floating: bool,
        to: &str,
        cx: &mut Context<Self>,
    ) {
        let stage = find_stage(self.doc.root.as_ref()).map(|s| s.id.clone());
        match (to, from_floating, stage) {
            ("float", false, Some(stage)) => self.nav_include(&[stage], cx),
            (kind, true, _) if kind != "float" && kind != "tab" => {
                self.nav_include(std::slice::from_ref(&panel.id), cx);
            }
            _ => self.ensure_framed_visible(&panel.selected, cx),
        }
        self.focus_view(&panel.selected, cx);
    }

    #[allow(clippy::many_single_char_names)] // a rect's own names
    fn float_rect(&self, d: &DragSession, next: &LayoutDocument, layer: FloatingLayer) -> Rect {
        let container = match layer {
            FloatingLayer::Overlay => Rect::new(0.0, 0.0, self.viewport.0, self.viewport.1),
            FloatingLayer::Stage => {
                let world = find_stage(next.root.as_ref())
                    .and_then(|s| self.layout(next.root.as_ref()).rect(&s.id));
                self.to_screen(world.unwrap_or(Rect::UNIT))
            }
        };
        let current = if d.from_float.is_some() && !d.tab_rollback {
            self.last_rects.get(&d.lifted.id).copied()
        } else {
            None
        };
        let original = if d.from_frame {
            d.floating
        } else {
            current.unwrap_or(d.source_screen)
        };
        let (w, h) = if d.from_float.is_some() && !d.from_frame && !d.tab_rollback {
            (original.w, original.h)
        } else {
            (
                (container.w * 2.0 / 3.0).min(original.w),
                (container.h * 2.0 / 3.0).min(original.h),
            )
        };
        let grab_y = d
            .grab_y
            .min(18.0 + (d.grab_y - 18.0) * (1.0 - d.compact_amount));
        let (x, y) = (d.pointer.0 - w * d.grab_x, d.pointer.1 - grab_y);
        Rect {
            x: (x - container.x) / container.w,
            y: (y - container.y) / container.h,
            w: w / container.w,
            h: h / container.h,
        }
    }

    fn end_pending(&mut self, d: DragSession, commit: bool, cx: &mut Context<Self>) {
        let sorted = d.tab_sort.as_ref().is_some_and(|s| s.started);
        if !commit {
            self.fx.tab_motion.clear_sort();
            cx.notify();
            return;
        }
        if !sorted {
            self.fx.tab_motion.clear_sort();
            if let Some(view) = d.press_view
                && self.now() >= self.suppress_clicks_until
            {
                self.select_and_focus(&view, cx);
            }
            cx.notify();
            return;
        }
        let (Some(sort), Some(view)) = (d.tab_sort, d.pending_tab) else {
            return;
        };
        let now = self.now();
        self.suppress_clicks_until = now + 350.0;
        let first_x = sort.items.first().map_or(0.0, |i| i.1);
        let mut slot_x = first_x;
        let mut deltas = Vec::with_capacity(sort.order.len());
        for id in &sort.order {
            let Some(&(_, orig_x, width)) = sort.items.iter().find(|(i, ..)| i == id) else {
                continue;
            };
            let shift = if *id == view {
                sort.offsets.get(id).copied().unwrap_or(0.0)
            } else {
                self.fx.tab_motion.sort_value(id, now)
            };
            deltas.push((id.clone(), orig_x + shift - slot_x));
            slot_x += width;
        }
        self.fx.tab_motion.clear_sort();
        self.fx.tab_motion.flip(deltas.into_iter(), now);
        self.wake(cx);
        let mut next = reorder_tabs(&self.doc, &d.lifted.id, &sort.order);
        next = select_view(&next, &view);
        self.commit(
            next,
            CommitOptions {
                from: Some(HashMap::new()),
                ..Default::default()
            },
            cx,
        );
    }

    fn along(&self, axis: Axis, pos: Point<Pixels>) -> f64 {
        let p = self.to_world(self.local(pos));
        if axis == Axis::X { p.0 } else { p.1 }
    }

    pub(crate) fn divider_begin(
        &mut self,
        split: Id,
        index: usize,
        pos: Point<Pixels>,
        click_count: usize,
        cx: &mut Context<Self>,
    ) {
        if self.capturing() {
            return;
        }
        if click_count >= 2 {
            return self.equalize(&split, index, cx);
        }
        let Some(root) = self.doc.root.clone() else {
            return;
        };
        let Some(push) = drag_boundary(&root, self.metrics(), &split, index) else {
            return;
        };
        let grab = self.along(push.axis, pos);
        self.divider = Some(DividerDrag {
            split,
            index,
            grab,
            origin: self.doc.clone(),
            push,
        });
        self.wake(cx);
    }

    fn divider_move(&mut self, pos: Point<Pixels>, snap: bool, cx: &mut Context<Self>) {
        let Some(d) = &self.divider else { return };
        let (split, index, grab, axis, start) =
            (d.split.clone(), d.index, d.grab, d.push.axis, d.push.start);
        let mut target = start + self.along(axis, pos) - grab;
        if snap && let Some(r) = self.entries.rect(&split) {
            let from = if axis == Axis::X { r.x } else { r.y };
            let len = r.len(axis);
            target = DIVIDER_SNAPS
                .iter()
                .map(|f| from + f * len)
                .min_by(|a, b| (a - target).abs().total_cmp(&(b - target).abs()))
                .unwrap_or(target);
        }
        let current = if self.opts.keep_pushed {
            self.doc
                .root
                .as_ref()
                .and_then(|r| drag_boundary(r, self.metrics(), &split, index))
        } else {
            None
        };
        let new_root = match (&current, &self.divider) {
            (Some(c), _) => c.to(target),
            (None, Some(d)) => d.push.to(target),
            (None, None) => return,
        };
        self.doc.root = Some(new_root);
        self.sync();
        cx.notify();
    }

    fn divider_end(&mut self, cx: &mut Context<Self>) {
        let Some(d) = self.divider.take() else { return };
        let next = std::mem::replace(&mut self.doc, d.origin);
        self.commit(
            next,
            CommitOptions {
                animate: Some(false),
                ..Default::default()
            },
            cx,
        );
        self.sync();
        cx.notify();
    }

    fn equalize(&mut self, split_id: &Id, index: usize, cx: &mut Context<Self>) {
        let Some(Node::Split(split)) = find_node(self.doc.root.as_ref(), split_id).cloned() else {
            return;
        };
        let mut weights = split.weights.clone();
        let pair = weights[index] + weights[index + 1];
        weights[index] = pair / 2.0;
        weights[index + 1] = pair / 2.0;
        let mut next = self.doc.clone();
        next.root = replace_node(
            next.root.take(),
            split_id,
            Node::Split(SplitNode { weights, ..split }),
        );
        self.commit(next, CommitOptions::default(), cx);
    }

    pub(crate) fn resize_begin(
        &mut self,
        panel: Id,
        dir: ResizeDir,
        pos: Point<Pixels>,
        cx: &mut Context<Self>,
    ) {
        if self.capturing() {
            return;
        }
        let Some(float) = self
            .doc
            .floating
            .iter()
            .find(|f| f.panel.id == panel)
            .cloned()
        else {
            return;
        };
        self.resize = Some(FloatResize {
            panel,
            dir,
            start: self.float_screen(float.rect, float.layer),
            start_point: self.local(pos),
            origin: self.doc.clone(),
            layer: float.layer,
        });
        self.wake(cx);
    }

    #[allow(clippy::many_single_char_names)] // a rect's own names
    fn resize_move(&mut self, pos: Point<Pixels>, cx: &mut Context<Self>) {
        let Some(r) = &self.resize else { return };
        let p = self.local(pos);
        let (dx, dy) = (p.0 - r.start_point.0, p.1 - r.start_point.1);
        let min_w: f64 = 160.0;
        let min_h = TABBAR_HEIGHT + 60.0;
        let Rect {
            mut x,
            mut y,
            mut w,
            mut h,
        } = r.start;
        if r.dir.e {
            w = min_w.max(r.start.w + dx);
        }
        if r.dir.s {
            h = min_h.max(r.start.h + dy);
        }
        if r.dir.w {
            w = min_w.max(r.start.w - dx);
            x = r.start.x + r.start.w - w;
        }
        if r.dir.n {
            h = min_h.max(r.start.h - dy);
            y = r.start.y + r.start.h - h;
        }
        let (panel, layer) = (r.panel.clone(), r.layer);
        let rect = self.screen_to_float(Rect::new(x, y, w, h), layer);
        if let Some(f) = self.doc.floating.iter_mut().find(|f| f.panel.id == panel) {
            f.rect = rect;
        }
        cx.notify();
    }

    fn resize_end(&mut self, cx: &mut Context<Self>) {
        let Some(r) = self.resize.take() else { return };
        let next = std::mem::replace(&mut self.doc, r.origin);
        self.commit(
            next,
            CommitOptions {
                animate: Some(false),
                ..Default::default()
            },
            cx,
        );
        cx.notify();
    }
}
