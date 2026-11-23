use blenderui::prelude::{Context, Modifiers, MouseButton, Pixels, Point, ScrollWheelEvent};

use super::{
    keymap::any_chord,
    model::{
        spatial::{
            MaximizeSession, Step, Visit, best_fit, focus_layout, frame_leaves, hierarchy_step,
            maximize_transition, nav_node, nav_parent, neighbor, record_visit, rectangle_camera,
            saved_frame_destination,
        },
        tree::{Entries, Kind, Metrics, find_node, find_stage, leaf_ids},
        types::{Axis, FloatingLayer, Framing, GestureOwner, Id, NavigationState, Node, Rect},
    },
    motion::rubber_band,
    workspace::{Navigation, Workspace, WorkspaceEvent},
};

const MAX_HISTORY: usize = 100;
pub(crate) const TOUCH_IDLE_MS: f64 = 180.0;
const SWIPE_LOCK: f64 = 16.0;
const SWIPE_HISTORY_MS: f64 = 150.0;
const SWIPE_DECELERATION: f64 = 0.997;
const SWIPE_BAND: f64 = 0.05;
const ZOOM_BAND: f64 = 0.2;
const EDGE_ZONE: f64 = 30.0;
const EDGE_DELAY_MS: f64 = 100.0;
const EDGE_SPEED: f64 = 1500.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Touch {
    Step,
    Swipe,
    Scale,
}

#[derive(Default)]
struct AxisLock {
    total: (f64, f64),
    axis: Option<Axis>,
}

impl AxisLock {
    fn feed(&mut self, dx: f64, dy: f64) -> Option<(Axis, f64, bool)> {
        if let Some(axis) = self.axis {
            return Some(match axis {
                Axis::X => (axis, dx, false),
                Axis::Y => (axis, dy, false),
            });
        }
        self.total = (self.total.0 + dx, self.total.1 + dy);
        if self.total.0.hypot(self.total.1) < SWIPE_LOCK {
            return None;
        }
        let (axis, delta) = if self.total.0.abs() > self.total.1.abs() {
            (Axis::X, self.total.0)
        } else {
            (Axis::Y, self.total.1)
        };
        self.axis = Some(axis);
        Some((axis, delta, true))
    }
}

#[derive(Default)]
pub(crate) struct Swipe {
    lock: AxisLock,
    raw: f64,
    history: Vec<(f64, f64)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DragMode {
    Pan,
    Scale,
    Rect,
}

#[derive(Clone, Debug)]
pub enum FrameTarget {
    All,
    Stage,
    Ids(Vec<Id>),
}

pub(crate) struct GestureDrag {
    pub mode: DragMode,
    pub start: (f64, f64),
    pub last: (f64, f64),
    pub view: Rect,
    pub candidate: Option<Id>,
}

#[derive(Default)]
struct StepWheel {
    last: f64,
    total: f64,
    stepped_at: f64,
    direction: f64,
}

pub(crate) struct NavState {
    pub entries: Entries,
    pub framed: Option<Id>,
    framed_leaves: Vec<Id>,
    visits: Vec<Visit>,
    visit_index: usize,
    session: Option<MaximizeSession>,
    overview_return: Option<Vec<Id>>,
    pub framings: Vec<Framing>,
    pub gesture: bool,
    pub held_mode: Option<DragMode>,
    pub gesture_drag: Option<GestureDrag>,
    pub snap_preview: Option<Rect>,
    pub marquee: Option<Rect>,
    pub marquee_target: Option<(Rect, String)>,
    wheel: StepWheel,
    pub wheel_deadline: Option<f64>,
    pub swipe: Option<Swipe>,
    zoom_raw: Option<(f64, f64, f64)>,
    edge_since: Option<f64>,
    edge_scrolled: bool,
    pub shortcuts: bool,
}

impl NavState {
    pub fn new() -> Self {
        Self {
            entries: Entries::default(),
            framed: None,
            framed_leaves: Vec::new(),
            visits: vec![Visit {
                id: "".into(),
                leaves: Vec::new(),
            }],
            visit_index: 0,
            session: None,
            overview_return: None,
            framings: Vec::new(),
            gesture: false,
            held_mode: None,
            gesture_drag: None,
            snap_preview: None,
            marquee: None,
            marquee_target: None,
            wheel: StepWheel::default(),
            wheel_deadline: None,
            swipe: None,
            zoom_raw: None,
            edge_since: None,
            edge_scrolled: false,
            shortcuts: false,
        }
    }
}

impl Workspace {
    fn root_id(&self) -> Id {
        self.doc
            .root
            .as_ref()
            .map_or_else(|| "".into(), |r| r.id().clone())
    }
    fn nav_on(&self) -> bool {
        self.nav_mode() != Navigation::Off
    }
    fn nav_free(&self) -> bool {
        self.nav_mode() == Navigation::Free
    }
    fn nav_rect_of(&self, id: Option<&Id>) -> Rect {
        id.and_then(|i| self.nav.entries.rect(i))
            .unwrap_or(Rect::UNIT)
    }

    pub(crate) fn nav_sync(&mut self) {
        let metrics = self.metrics();
        self.nav.entries = focus_layout(self.doc.root.as_ref(), Some(&metrics as &dyn Metrics));
        if let Some(framed) = self.nav.framed.clone() {
            if self.nav.entries.contains(&framed) {
                self.nav.framed_leaves = self.nav.entries.leaf_ids(&framed);
            } else {
                let surviving: Vec<Id> = self
                    .nav
                    .framed_leaves
                    .iter()
                    .filter(|id| self.nav.entries.contains(id))
                    .cloned()
                    .collect();
                let next = if surviving.is_empty() {
                    None
                } else {
                    frame_leaves(self.doc.root.as_ref(), &surviving)
                };
                let root = self.root_id();
                self.nav.framed = next.filter(|n| *n != root);
                self.nav.framed_leaves = self
                    .nav
                    .framed
                    .as_ref()
                    .map(|f| self.nav.entries.leaf_ids(f))
                    .unwrap_or_default();
            }
        }
        if self
            .nav
            .session
            .as_ref()
            .is_some_and(|s| !self.nav.entries.contains(&s.id))
        {
            self.nav.session = None;
        }
        self.nav_retarget();
    }

    fn nav_retarget(&mut self) {
        let target = self.nav_rect_of(self.nav.framed.as_ref());
        if !self.camera.target.approx_eq(&target, 1e-9) {
            self.camera.target = target;
            if self.reduced() {
                self.camera.finish();
            }
            self.animating = true;
            self.last_frame = None;
        }
    }

    pub(crate) fn nav_focus(&mut self, id: Option<Id>, record: bool, cx: &mut Context<Self>) {
        let mut id = if self.nav_on() { id } else { None };
        if let Some(i) = &id
            && self.doc.floating.iter().any(|f| &f.panel.id == i)
        {
            id = find_stage(self.doc.root.as_ref()).map(|s| s.id.clone());
        }
        id = id
            .filter(|i| self.nav.entries.contains(i))
            .map(|i| nav_node(&self.nav.entries, &i));
        if id.as_ref() == Some(&self.root_id()) {
            id = None;
        }
        if self
            .nav
            .session
            .as_ref()
            .is_some_and(|s| Some(&s.id) != id.as_ref())
        {
            self.nav.session = None;
        }
        if record {
            let leaves = match &id {
                Some(i) => self.nav.entries.leaf_ids(i),
                None => leaf_ids(self.doc.root.as_ref()),
            };
            let visit = Visit {
                id: id.clone().unwrap_or_else(|| self.root_id()),
                leaves,
            };
            let (visits, index) = record_visit(
                std::mem::take(&mut self.nav.visits),
                self.nav.visit_index,
                visit,
            );
            self.nav.visits = visits;
            self.nav.visit_index = index;
            if self.nav.visits.len() > MAX_HISTORY {
                let excess = self.nav.visits.len() - MAX_HISTORY;
                self.nav.visits.drain(..excess);
                self.nav.visit_index = self.nav.visits.len() - 1;
            }
        }
        self.nav_end_gesture();
        let changed = self.nav.framed != id;
        self.nav.framed_leaves = id
            .as_ref()
            .map(|i| self.nav.entries.leaf_ids(i))
            .unwrap_or_default();
        self.nav.framed = id;
        self.nav_retarget();
        if changed {
            self.emit(WorkspaceEvent::Navigate(self.nav.framed.clone()), cx);
            self.emit_change(cx);
        }
        self.wake(cx);
    }

    fn nav_end_gesture(&mut self) {
        self.nav.wheel_deadline = None;
        self.nav.snap_preview = None;
        self.nav.zoom_raw = None;
        self.nav.edge_since = None;
        self.nav.edge_scrolled = false;
        if self.nav.gesture {
            self.nav.gesture = false;
            self.tween.stop();
        }
    }

    pub(crate) fn nav_toggle(&mut self, id: &Id, cx: &mut Context<Self>) -> bool {
        if !self.nav_on()
            || self.doc.floating.iter().any(|f| &f.panel.id == id)
            || !self.nav.entries.contains(id)
        {
            return false;
        }
        let root = self.root_id();
        let focused = self.nav.framed.clone().unwrap_or_else(|| root.clone());
        let node = nav_node(&self.nav.entries, id);
        let (destination, session) = maximize_transition(
            &self.nav.entries,
            &root,
            &focused,
            &node,
            self.nav.session.as_ref(),
        );
        self.nav_focus(Some(destination), true, cx);
        self.nav.session = session;
        true
    }

    pub(crate) fn nav_step_out(&mut self, cx: &mut Context<Self>) {
        if let Some(framed) = self.nav.framed.clone() {
            let parent = nav_parent(&self.nav.entries, &framed);
            self.nav_focus(parent, true, cx);
        }
    }

    fn nav_history_go(&mut self, direction: isize, cx: &mut Context<Self>) {
        let next = self.nav.visit_index.cast_signed() + direction;
        if next < 0 || next as usize >= self.nav.visits.len() {
            return;
        }
        self.nav.visit_index = next as usize;
        self.nav.session = None;
        let destination = saved_frame_destination(
            self.doc.root.as_ref(),
            &self.nav.visits[next as usize].leaves,
        );
        let fallback =
            destination.or_else(|| find_stage(self.doc.root.as_ref()).map(|s| s.id.clone()));
        self.nav_focus(fallback, false, cx);
    }
    pub(crate) fn nav_back(&mut self, cx: &mut Context<Self>) {
        self.nav_history_go(-1, cx);
    }
    pub(crate) fn nav_forward(&mut self, cx: &mut Context<Self>) {
        self.nav_history_go(1, cx);
    }

    pub(crate) fn nav_toggle_overview(&mut self, cx: &mut Context<Self>) {
        if self.nav.framed.is_none() {
            if let Some(leaves) = self.nav.overview_return.take() {
                let destination = saved_frame_destination(self.doc.root.as_ref(), &leaves);
                self.nav_focus(destination, true, cx);
            }
        } else {
            self.nav.overview_return = Some(self.nav.framed_leaves.clone());
            self.nav_focus(None, true, cx);
        }
    }

    pub(crate) fn nav_frame_stage(&mut self, cx: &mut Context<Self>) {
        let stage = find_stage(self.doc.root.as_ref()).map(|s| s.id.clone());
        self.nav_focus(stage, true, cx);
    }

    pub(crate) fn nav_ensure_visible(&mut self, panel_id: &Id, cx: &mut Context<Self>) {
        let Some(framed) = self.nav.framed.clone() else {
            return;
        };
        if !self.nav.entries.contains(&framed) {
            return;
        }
        if let Some(float) = self.doc.floating.iter().find(|f| &f.panel.id == panel_id) {
            if float.layer == FloatingLayer::Overlay {
                return;
            }
            let stage = find_stage(self.doc.root.as_ref()).map(|s| s.id.clone());
            if let Some(stage) = &stage
                && (self.nav.framed_leaves.contains(stage)
                    || self.nav.entries.contains_node(&framed, stage))
            {
                return;
            }
            self.nav_focus(stage, true, cx);
            return;
        }
        if self.nav.entries.contains_node(&framed, panel_id) {
            return;
        }
        let mut ids = self.nav.framed_leaves.clone();
        ids.push(panel_id.clone());
        let next = frame_leaves(self.doc.root.as_ref(), &ids);
        self.nav_focus(next, true, cx);
    }

    pub(crate) fn nav_include(&mut self, ids: &[Id], cx: &mut Context<Self>) {
        if self.nav.framed.is_none() {
            return;
        }
        let mut all: Vec<Id> = self
            .nav
            .framed_leaves
            .iter()
            .filter(|id| self.nav.entries.contains(id))
            .cloned()
            .collect();
        all.extend_from_slice(ids);
        let next = frame_leaves(self.doc.root.as_ref(), &all);
        self.nav_focus(next, true, cx);
    }

    pub(crate) fn nav_serialize(&self) -> NavigationState {
        NavigationState {
            frame: if self.nav.framed.is_some() {
                self.nav.framed_leaves.clone()
            } else {
                Vec::new()
            },
            framings: self.nav.framings.clone(),
        }
    }

    pub(crate) fn nav_restore(&mut self, nav: Option<NavigationState>, cx: &mut Context<Self>) {
        let nav = nav.unwrap_or_default();
        self.nav.framings = nav.framings;
        let destination = if nav.frame.is_empty() {
            None
        } else {
            saved_frame_destination(self.doc.root.as_ref(), &nav.frame)
        };
        self.nav.visits = vec![Visit {
            id: self.root_id(),
            leaves: leaf_ids(self.doc.root.as_ref()),
        }];
        self.nav.visit_index = 0;
        self.nav.session = None;
        self.nav.overview_return = None;
        self.nav.framed = None;
        self.nav.framed_leaves.clear();
        match destination {
            Some(d) if self.nav_on() => self.nav_focus(Some(d), true, cx),
            _ => self.nav_retarget(),
        }
    }

    pub fn frame(&mut self, target: FrameTarget, cx: &mut Context<Self>) {
        match target {
            FrameTarget::All => self.nav_focus(None, true, cx),
            FrameTarget::Stage => self.nav_frame_stage(cx),
            FrameTarget::Ids(ids) => {
                let mut leaves: Vec<Id> = Vec::new();
                for id in ids {
                    if self.nav.entries.contains(&id) {
                        leaves.extend(self.nav.entries.leaf_ids(&id));
                    } else if let Some(panel) = self.nav_panel_of(&id) {
                        leaves.push(panel);
                    }
                }
                if !leaves.is_empty() {
                    let next = frame_leaves(self.doc.root.as_ref(), &leaves);
                    self.nav_focus(next, true, cx);
                }
            }
        }
    }

    fn nav_panel_of(&self, view_id: &Id) -> Option<Id> {
        leaf_ids(self.doc.root.as_ref()).into_iter().find(|id| {
            matches!(find_node(self.doc.root.as_ref(), id), Some(Node::Panel(p)) if p.views.contains(view_id))
        })
    }

    pub fn toggle_maximize(&mut self, panel_or_view: &Id, cx: &mut Context<Self>) -> bool {
        let id = self
            .resolve_panel(panel_or_view)
            .map_or_else(|| panel_or_view.clone(), |p| p.id);
        self.nav_toggle(&id, cx)
    }
    pub fn navigate_back(&mut self, cx: &mut Context<Self>) {
        self.nav_back(cx);
    }
    pub fn navigate_forward(&mut self, cx: &mut Context<Self>) {
        self.nav_forward(cx);
    }
    pub fn overview(&mut self, cx: &mut Context<Self>) {
        if self.nav.framed.is_some() {
            self.nav.overview_return = Some(self.nav.framed_leaves.clone());
        }
        self.nav_focus(None, true, cx);
    }
    #[must_use]
    pub const fn framed(&self) -> Option<&Id> {
        self.nav.framed.as_ref()
    }
    #[must_use]
    pub const fn camera(&self) -> Rect {
        self.camera.value
    }
    #[must_use]
    pub fn framings(&self) -> &[Framing] {
        &self.nav.framings
    }
    pub fn save_framing(&mut self, name: &str, cx: &mut Context<Self>) -> Option<Framing> {
        if name.trim().is_empty() {
            return None;
        }
        let frame = if self.nav.framed.is_some() {
            self.nav.framed_leaves.clone()
        } else {
            leaf_ids(self.doc.root.as_ref())
        };
        let framing = Framing {
            id: super::model::document::uid("framing"),
            name: name.trim().to_string(),
            frame,
        };
        self.nav.framings.push(framing.clone());
        self.emit_change(cx);
        Some(framing)
    }
    pub fn go_framing(&mut self, id: &Id, cx: &mut Context<Self>) {
        if let Some(f) = self.nav.framings.iter().find(|f| &f.id == id).cloned() {
            let destination = saved_frame_destination(self.doc.root.as_ref(), &f.frame);
            self.nav_focus(destination, true, cx);
        }
    }
    pub fn remove_framing(&mut self, id: &Id, cx: &mut Context<Self>) {
        self.nav.framings.retain(|f| &f.id != id);
        self.emit_change(cx);
    }

    pub(crate) fn local_point(&self, pos: Point<Pixels>) -> (f64, f64) {
        self.local(pos)
    }

    fn nav_drag_mode(&self, mods: Modifiers) -> Option<DragMode> {
        let keys = &self.chords;
        [
            (DragMode::Scale, &keys.scale),
            (DragMode::Rect, &keys.rect),
            (DragMode::Pan, &keys.pan),
        ]
        .into_iter()
        .find(|(_, chords)| any_chord(mods, chords, &self.held))
        .map(|(mode, _)| mode)
    }

    /// The gesture the held keys select (its cursor shows): true when it changed.
    pub(crate) fn nav_modifiers_changed(&mut self, mods: Modifiers, cx: &mut Context<Self>) -> bool {
        let mode = if self.nav_free() && !self.interacting() {
            self.nav_drag_mode(mods)
        } else {
            None
        };
        let changed = mode != self.nav.held_mode;
        if changed {
            self.nav.held_mode = mode;
            cx.notify();
        }
        changed
    }

    pub(crate) fn nav_zoom(
        &mut self,
        factor: f64,
        x: f64,
        y: f64,
        pan: (f64, f64),
        cx: &mut Context<Self>,
    ) {
        self.nav_zoom_with(factor, x, y, pan, false, cx);
    }

    pub(crate) fn nav_zoom_banded(&mut self, factor: f64, x: f64, y: f64, cx: &mut Context<Self>) {
        self.nav_zoom_with(factor, x, y, (0.0, 0.0), true, cx);
    }

    #[allow(clippy::many_single_char_names)] // a rect's own names
    fn nav_zoom_with(
        &mut self,
        factor: f64,
        x: f64,
        y: f64,
        pan: (f64, f64),
        band: bool,
        cx: &mut Context<Self>,
    ) {
        if self.nav.entries.is_empty() || self.viewport.0 <= 0.0 {
            return;
        }
        let (bw, bh) = self.viewport;
        let c = self.camera.value;
        let min = self
            .nav
            .entries
            .iter()
            .map(|e| e.rect.w.min(e.rect.h))
            .fold(f64::INFINITY, f64::min)
            * 0.65;
        if band {
            let (z, w0, h0) = self.nav.zoom_raw.unwrap_or((0.0, c.w, c.h));
            let z = z + factor.ln();
            self.nav.zoom_raw = Some((z, w0, h0));
            let z = rubber_band(z, (min / w0.min(h0)).ln(), f64::INFINITY, ZOOM_BAND);
            let size = |s0: f64| rubber_band(s0.ln() + z, f64::NEG_INFINITY, 0.0, ZOOM_BAND).exp();
            let place = |at: f64, old: f64, new: f64, anchor: f64| {
                if new <= 1.0 {
                    (at + anchor * (old - new)).clamp(0.0, 1.0 - new)
                } else {
                    (1.0 - new) / 2.0
                }
            };
            let (w, h) = (size(w0), size(h0));
            let next = Rect {
                x: place(c.x, c.w, w, x / bw),
                y: place(c.y, c.h, h, y / bh),
                w,
                h,
            };
            self.nav_set_camera(next, cx);
            return;
        }
        let factor = (min / c.w.min(c.h)).max(factor.min(1.35 / c.w.max(c.h)));
        let mut next = Rect {
            x: c.x + c.w * (x / bw) * (1.0 - factor) - pan.0 / bw * c.w,
            y: c.y + c.h * (y / bh) * (1.0 - factor) - pan.1 / bh * c.h,
            w: c.w * factor,
            h: c.h * factor,
        };
        next.x = (-next.w * 0.3).max((1.0 - next.w * 0.7).min(next.x));
        next.y = (-next.h * 0.3).max((1.0 - next.h * 0.7).min(next.y));
        self.nav_set_camera(next, cx);
    }

    fn nav_set_camera(&mut self, next: Rect, cx: &mut Context<Self>) {
        if !self.nav.gesture {
            self.nav.gesture = true;
            self.tween.stop();
        }
        self.camera.velocity = Rect::ZERO;
        self.camera.jump(next);
        self.nav.snap_preview = best_fit(self.camera.value, &self.nav.entries)
            .and_then(|id| self.nav.entries.rect(&id))
            .map(|r| self.panel_screen(r));
        self.wake(cx);
    }

    pub(crate) fn nav_finish_gesture(&mut self, cx: &mut Context<Self>) {
        if self.nav.gesture {
            let best = best_fit(self.camera.value, &self.nav.entries);
            self.nav_focus(best, true, cx);
        }
    }

    pub(crate) fn step_nav(&mut self, now: f64, cx: &mut Context<Self>) -> bool {
        match self.nav.wheel_deadline {
            Some(at) if now >= at => {
                self.nav.wheel_deadline = None;
                self.nav_touch_end(true, cx);
                false
            }
            Some(_) => true,
            None => false,
        }
    }

    fn nav_step_scroll(&mut self, dy: f64, at: (f64, f64), cx: &mut Context<Self>) {
        if self.interacting() {
            return;
        }
        let now = self.now();
        let direction = dy.signum();
        if dy == 0.0 {
            return;
        }
        let w = &mut self.nav.wheel;
        if now - w.last > 260.0 || direction != w.direction {
            *w = StepWheel {
                stepped_at: f64::NEG_INFINITY,
                ..Default::default()
            };
        }
        w.last = now;
        w.direction = direction;
        w.total += dy;
        if now - w.stepped_at >= 140.0 && w.total.abs() >= 12.0 {
            w.stepped_at = now;
            let step = if w.total > 0.0 { Step::Out } else { Step::In };
            w.total = 0.0;
            let point = self.to_world(at);
            let from = self.nav.framed.clone().unwrap_or_else(|| self.root_id());
            let next = hierarchy_step(&self.nav.entries, &from, step, point);
            self.nav_focus(Some(next), true, cx);
        }
    }

    /// A wheel or touchpad scroll. BUI reports a wheel notch as lines (`delta`) and a touchpad's
    /// exact distance as `pixels`, vertical only: Shift makes it horizontal, as a wheel's is.
    pub(crate) fn nav_wheel(&mut self, ev: &ScrollWheelEvent, cx: &mut Context<Self>) -> bool {
        if !self.nav_free() {
            return false;
        }
        let at = self.local_point(ev.position);
        if let Some(pixels) = ev.pixels {
            let d = f64::from(pixels);
            #[cfg(feature = "pinch")]
            if ev.modifiers.control && !ev.modifiers.shift && !ev.modifiers.alt {
                return self.nav_pinch(ev.position, -pixels * 0.01, false, cx);
            }
            let mut mods = ev.modifiers;
            let (dx, dy) = if std::mem::take(&mut mods.shift) {
                (d, 0.0)
            } else {
                (0.0, d)
            };
            return self.nav_touchpad(mods, at, dx, dy, cx);
        }
        let dy = f64::from(ev.delta) * 16.0;
        if any_chord(ev.modifiers, &self.chords.step, &self.held) {
            self.nav_step_scroll(dy, at, cx);
            return true;
        }
        let owner = self.gesture_owner_at(at);
        if ev.modifiers.control {
            if owner == Some(GestureOwner::Exclusive) {
                return false;
            }
            self.nav_step_scroll(-dy, at, cx);
            return true;
        }
        let plain = !ev.modifiers.shift && !ev.modifiers.alt;
        if owner == Some(GestureOwner::Workspace) && plain {
            self.nav_step_scroll(dy, at, cx);
            return true;
        }
        false
    }

    fn nav_touch_mode(&self, mods: Modifiers, owner: Option<GestureOwner>) -> Option<Touch> {
        let keys = &self.chords;
        let chord = [(Touch::Step, &keys.step), (Touch::Scale, &keys.touch_scale)]
            .into_iter()
            .find(|(_, chords)| any_chord(mods, chords, &self.held))
            .map(|(mode, _)| mode);
        if chord.is_some() {
            return chord;
        }
        let plain = !mods.shift && !mods.control;
        let free = owner != Some(GestureOwner::Exclusive);
        (plain && (free || self.nav.swipe.is_some())).then_some(Touch::Swipe)
    }

    /// A touchpad scroll. BUI reports no touch phases: the gesture ends when the touchpad is idle
    /// for `TOUCH_IDLE_MS` (the `wheel_deadline`).
    fn nav_touchpad(
        &mut self,
        mods: Modifiers,
        at: (f64, f64),
        dx: f64,
        dy: f64,
        cx: &mut Context<Self>,
    ) -> bool {
        let owner = self.gesture_owner_at(at);
        let mode = self.nav_touch_mode(mods, owner);
        if mode != Some(Touch::Swipe) && self.nav.swipe.is_some() {
            self.nav_swipe_end(false, cx);
        }
        let Some(mode) = mode else {
            return false;
        };
        if mode == Touch::Step {
            self.nav_step_scroll(dy, at, cx);
            return true;
        }
        if owner == Some(GestureOwner::Exclusive) && self.nav.swipe.is_none() {
            return false;
        }
        if self.interacting() || (dx == 0.0 && dy == 0.0) {
            return true;
        }
        match mode {
            Touch::Swipe => self.nav_swipe(dx, dy, cx),
            _ => self.nav_zoom_banded((-dy * 0.009).clamp(-0.3, 0.3).exp(), at.0, at.1, cx),
        }
        self.nav.wheel_deadline = Some(self.now() + TOUCH_IDLE_MS);
        true
    }

    fn nav_touch_end(&mut self, idle: bool, cx: &mut Context<Self>) {
        if self.nav.swipe.is_some() {
            self.nav_swipe_end(idle, cx);
        } else {
            self.nav_finish_gesture(cx);
        }
    }

    fn swipe_bounds(&self, axis: Axis) -> (f64, f64) {
        let len = self.camera.value.len(axis);
        ((1.0 - len).min(0.0), (1.0 - len).max(0.0))
    }

    fn nav_swipe(&mut self, dx: f64, dy: f64, cx: &mut Context<Self>) {
        if self.nav.entries.is_empty() || self.viewport.0 <= 0.0 {
            return;
        }
        let now = self.now();
        let c = self.camera.value;
        let s = self.nav.swipe.get_or_insert_with(Swipe::default);
        let Some((axis, delta, first)) = s.lock.feed(dx, dy) else {
            return;
        };
        if first {
            s.raw = match axis {
                Axis::X => c.x,
                Axis::Y => c.y,
            };
        }
        let screen = match axis {
            Axis::X => self.viewport.0,
            Axis::Y => self.viewport.1,
        };
        let world = -delta / screen.max(1.0) * c.len(axis);
        s.raw += world;
        s.history.push((now, world));
        s.history.retain(|(t, _)| now - t <= SWIPE_HISTORY_MS);
        let raw = s.raw;
        let (lo, hi) = self.swipe_bounds(axis);
        let banded = rubber_band(raw, lo, hi, SWIPE_BAND * c.len(axis));
        let mut next = c;
        match axis {
            Axis::X => next.x = banded,
            Axis::Y => next.y = banded,
        }
        self.nav_set_camera(next, cx);
    }

    fn nav_swipe_end(&mut self, idle: bool, cx: &mut Context<Self>) {
        let Some(s) = self.nav.swipe.take() else {
            return;
        };
        let Some(axis) = s.lock.axis else {
            return;
        };
        let now = match s.history.last() {
            Some(&(t, _)) if idle => t,
            _ => self.now(),
        };
        let (lo, hi) = self.swipe_bounds(axis);
        let velocity = match s.history.iter().find(|(t, _)| now - t <= SWIPE_HISTORY_MS) {
            Some(&(first, _)) if (lo..=hi).contains(&s.raw) => {
                let moved: f64 = s
                    .history
                    .iter()
                    .filter(|(t, _)| *t >= first)
                    .map(|(_, d)| d)
                    .sum();
                moved / (now - first).max(16.0) * 1000.0
            }
            _ => 0.0,
        };
        let projected =
            (s.raw.clamp(lo, hi) - velocity / (1000.0 * SWIPE_DECELERATION.ln())).clamp(lo, hi);
        let mut target = self.camera.value;
        match axis {
            Axis::X => target.x = projected,
            Axis::Y => target.y = projected,
        }
        let best = best_fit(target, &self.nav.entries);
        self.nav_focus(best, true, cx);
        match axis {
            Axis::X => self.camera.velocity.x = velocity,
            Axis::Y => self.camera.velocity.y = velocity,
        }
    }

    pub(crate) fn nav_neighbor(&mut self, dx: f64, dy: f64, cx: &mut Context<Self>) {
        let Some(framed) = self.nav.framed.as_ref() else {
            return;
        };
        let from = self.nav_rect_of(Some(framed));
        if let Some(id) = neighbor(&self.nav.entries, from, dx, dy) {
            self.nav_focus(Some(id), true, cx);
        }
    }

    pub(crate) fn nav_edge_scroll(&mut self, pointer: (f64, f64), now: f64, dt: f64) -> bool {
        let (bw, bh) = self.viewport;
        if !self.nav_on() || bw <= 0.0 {
            return false;
        }
        let push = |p: f64, len: f64| {
            if p < EDGE_ZONE {
                -(EDGE_ZONE - p.max(0.0)) / EDGE_ZONE
            } else if p > len - EDGE_ZONE {
                (p.min(len) - (len - EDGE_ZONE)) / EDGE_ZONE
            } else {
                0.0
            }
        };
        let c = self.camera.value;
        let vx = if c.w < 1.0 { push(pointer.0, bw) } else { 0.0 };
        let vy = if c.h < 1.0 { push(pointer.1, bh) } else { 0.0 };
        if vx == 0.0 && vy == 0.0 {
            self.nav.edge_since = None;
            return false;
        }
        let since = *self.nav.edge_since.get_or_insert(now);
        if now - since < EDGE_DELAY_MS {
            return false;
        }
        let slide = |at: f64, v: f64, len: f64, screen: f64| {
            let next = at + v * EDGE_SPEED * dt / screen * len;
            if v > 0.0 {
                next.min((1.0 - len).max(at))
            } else {
                next.max(at.min(0.0))
            }
        };
        let mut next = c;
        if vx != 0.0 {
            next.x = slide(c.x, vx, c.w, bw);
        }
        if vy != 0.0 {
            next.y = slide(c.y, vy, c.h, bh);
        }
        if next.approx_eq(&c, 1e-12) {
            return false;
        }
        if !self.nav.gesture {
            self.nav.gesture = true;
            self.tween.stop();
        }
        self.camera.velocity = Rect::ZERO;
        self.camera.jump(next);
        self.nav.edge_scrolled = true;
        true
    }

    pub(crate) const fn nav_edge_pending(&self) -> bool {
        self.nav.edge_since.is_some()
    }

    pub(crate) fn nav_edge_end(&mut self, cx: &mut Context<Self>) {
        self.nav.edge_since = None;
        if std::mem::take(&mut self.nav.edge_scrolled) && self.nav.gesture {
            self.nav_finish_gesture(cx);
        }
    }

    pub(crate) fn nav_gesture_begin(
        &mut self,
        pos: Point<Pixels>,
        button: MouseButton,
        mods: Modifiers,
        cx: &mut Context<Self>,
    ) -> bool {
        if !self.nav_free() || self.interacting() || self.nav.gesture_drag.is_some() {
            return false;
        }
        let mode = if button == MouseButton::Middle {
            let (x, y) = (f64::from(pos.x.0), f64::from(pos.y.0));
            let on_tab = self.tab_rects.values().any(|r| r.contains(x, y));
            if on_tab {
                return false;
            }
            Some(DragMode::Pan)
        } else {
            self.nav_drag_mode(mods)
        };
        let Some(mode) = mode else {
            return false;
        };
        let at = self.local_point(pos);
        self.nav.gesture_drag = Some(GestureDrag {
            mode,
            start: at,
            last: at,
            view: self.camera.value,
            candidate: None,
        });
        if mode == DragMode::Rect {
            self.nav_end_gesture();
            self.nav_move_rect(at);
        } else {
            self.nav_zoom(1.0, at.0, at.1, (0.0, 0.0), cx);
        }
        cx.notify();
        true
    }

    fn nav_move_rect(&mut self, at: (f64, f64)) {
        let Some(d) = self.nav.gesture_drag.as_ref() else {
            return;
        };
        let viewport = Rect::new(0.0, 0.0, self.viewport.0, self.viewport.1);
        let world = rectangle_camera(d.start, at, viewport, d.view);
        let screen = self.to_screen(world);
        let candidate = if screen.w >= 8.0 && screen.h >= 8.0 {
            best_fit(world, &self.nav.entries)
        } else {
            None
        };
        self.nav.marquee = Some(screen);
        self.nav.marquee_target = candidate.as_ref().and_then(|id| {
            let rect = self.panel_screen(self.nav.entries.rect(id)?);
            Some((
                rect,
                format!("Release to focus {} · Esc to cancel", self.node_title(id)),
            ))
        });
        if let Some(d) = self.nav.gesture_drag.as_mut() {
            d.candidate = candidate;
        }
    }

    fn node_title(&self, id: &Id) -> String {
        match self.nav.entries.get(id).map(|e| &e.kind) {
            Some(Kind::Panel) => self
                .find_panel(id)
                .map(|p| self.title_of(&p.selected))
                .unwrap_or_default(),
            Some(Kind::Stage { .. }) => "the stage".into(),
            _ => "this group".into(),
        }
    }

    pub(crate) fn nav_gesture_move(&mut self, pos: Point<Pixels>, cx: &mut Context<Self>) {
        let at = self.local_point(pos);
        let Some(d) = self.nav.gesture_drag.as_mut() else {
            return;
        };
        let (dx, dy) = (at.0 - d.last.0, at.1 - d.last.1);
        d.last = at;
        let (mode, start) = (d.mode, d.start);
        match mode {
            DragMode::Rect => self.nav_move_rect(at),
            DragMode::Pan => self.nav_zoom(1.0, at.0, at.1, (dx, dy), cx),
            DragMode::Scale => {
                self.nav_zoom(((dy - dx) * 0.006).exp(), start.0, start.1, (0.0, 0.0), cx);
            }
        }
        cx.notify();
    }

    pub(crate) fn nav_gesture_end(&mut self, commit: bool, cx: &mut Context<Self>) {
        let Some(d) = self.nav.gesture_drag.take() else {
            return;
        };
        self.suppress_clicks_until = self.now() + 350.0;
        self.nav.marquee = None;
        self.nav.marquee_target = None;
        match d.mode {
            DragMode::Rect => {
                let target = if commit {
                    d.candidate.or_else(|| self.nav.framed.clone())
                } else {
                    self.nav.framed.clone()
                };
                self.nav_focus(target, true, cx);
            }
            _ if commit => self.nav_finish_gesture(cx),
            _ => {
                let framed = self.nav.framed.clone();
                self.nav_focus(framed, true, cx);
            }
        }
    }
}
