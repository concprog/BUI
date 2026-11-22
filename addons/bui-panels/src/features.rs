use std::collections::{HashMap, HashSet};

use blenderui::prelude::{
    AnyElement, Context, IntoElement, KeyDownEvent, MouseButton, MouseDownEvent,
    ParentElement as _, Styled, div, px, relative,
};

use super::{
    model::{
        resize::drag_boundary,
        tree::{Kind, panels_of},
        types::{
            Axis, FloatingLayer, FloatingPanel, GestureOwner, Id, LayoutDocument, PanelNode, Rect,
            Scaling, TabbarMode,
        },
    },
    motion::cubic_bezier,
    render::place,
    workspace::{CommitOptions, Navigation, Palette, TABBAR_HEIGHT, Workspace},
};

#[derive(Clone, Copy, Debug)]
pub struct Detail {
    pub size: f64,
    pub outline: usize,
}

impl Default for Detail {
    fn default() -> Self {
        Self {
            size: 48.0,
            outline: 2,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BarMode {
    Normal,
    Overlay,
    Hidden,
}

pub(crate) const FRAME_ONLY: (f64, f64) = (160.0, 64.0);
const FREE_MIN: (f64, f64) = (480.0, 320.0);
const TAB_MS: f64 = 180.0;
const SWITCH_SLIDE: f64 = 14.0;
const Z_COLLAPSED: i32 = 10;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Titlebar {
    pub height: f64,
    pub inset_end: f64,
}

#[derive(Clone, Copy)]
struct Shift {
    from: f64,
    to: f64,
    start: f64,
}

fn tab_ease(t: f64) -> f64 {
    cubic_bezier(0.2, 0.8, 0.2, 1.0, t.clamp(0.0, 1.0))
}

impl Shift {
    fn value(&self, now: f64) -> f64 {
        self.from + (self.to - self.from) * tab_ease((now - self.start) / TAB_MS)
    }
    fn done(&self, now: f64) -> bool {
        now - self.start >= TAB_MS
    }
}

#[derive(Default)]
pub(crate) struct TabMotion {
    sort: HashMap<Id, Shift>,
    flip: HashMap<Id, Shift>,
}

impl TabMotion {
    pub fn sort_to(&mut self, targets: &HashMap<Id, f64>, dragged: &Id, now: f64) {
        for (id, &to) in targets {
            if id == dragged {
                self.sort.remove(id);
                continue;
            }
            let next = match self.sort.get(id).copied() {
                Some(s) if (s.to - to).abs() < 0.5 => continue,
                Some(s) => Shift {
                    from: s.value(now),
                    to,
                    start: now,
                },
                None => Shift {
                    from: 0.0,
                    to,
                    start: now,
                },
            };
            self.sort.insert(id.clone(), next);
        }
    }

    pub fn sort_value(&self, id: &Id, now: f64) -> f64 {
        self.sort.get(id).map_or(0.0, |s| s.value(now))
    }

    pub fn clear_sort(&mut self) {
        self.sort.clear();
    }

    pub fn flip(&mut self, deltas: impl Iterator<Item = (Id, f64)>, now: f64) {
        self.flip.clear();
        for (id, d) in deltas {
            if d.abs() >= 0.5 {
                self.flip.insert(
                    id,
                    Shift {
                        from: d,
                        to: 0.0,
                        start: now,
                    },
                );
            }
        }
    }

    pub fn active(&self, now: f64) -> bool {
        self.sort
            .values()
            .chain(self.flip.values())
            .any(|s| !s.done(now))
    }

    pub fn prune(&mut self, now: f64) {
        self.flip.retain(|_, s| !s.done(now));
    }
}

#[derive(Default)]
pub(crate) struct Fx {
    pub collapsed_of: HashMap<Id, Id>,
    pub collapsed_groups: Vec<Id>,
    pub hidden_splits: HashSet<Id>,
    pub frame_only: HashSet<Id>,
    /// The width of the cluster (accessory and menu button) at the end of each overlaid tab bar.
    pub inset_widths: HashMap<Id, f64>,
    pub tab_motion: TabMotion,
    pub scales: HashMap<Id, f64>,
    pub frozen_tabs: Option<(Id, f64)>,
    switch: HashMap<Id, Shift>,
}

impl Workspace {
    pub(crate) fn interacting(&self) -> bool {
        self.drag.as_ref().is_some_and(|d| d.active)
            || self.divider.is_some()
            || self.resize.is_some()
    }

    pub(crate) fn bar_mode(&self, panel: &PanelNode) -> BarMode {
        let modes: Vec<TabbarMode> = panel
            .views
            .iter()
            .map(|v| self.rules_of(v).tabbar)
            .collect();
        if modes.iter().all(|m| *m == TabbarMode::Never) {
            return BarMode::Hidden;
        }
        if panel.views.len() == 1 {
            match modes[0] {
                TabbarMode::Auto if self.lifted.as_ref().map(|l| &l.id) == Some(&panel.id) => {
                    return BarMode::Normal;
                }
                TabbarMode::Auto => return BarMode::Hidden,
                TabbarMode::Overlay => return BarMode::Overlay,
                _ => {}
            }
        }
        BarMode::Normal
    }

    pub(crate) fn is_frame_only(&self, panel: &PanelNode, r: &Rect) -> bool {
        self.lifted.as_ref().map(|l| &l.id) != Some(&panel.id)
            && (r.w < FRAME_ONLY.0
                || r.h
                    < FRAME_ONLY.1
                        + if panel.views.len() > 1 {
                            TABBAR_HEIGHT
                        } else {
                            0.0
                        })
    }

    fn is_small(&self, id: &Id, size: f64) -> bool {
        self.entries.rect(id).is_some_and(|r| {
            let s = self.to_screen(r);
            s.w < size || s.h < size
        })
    }

    fn is_nested_split(&self, id: &Id) -> bool {
        match self.entries.get(id).map(|e| &e.kind) {
            Some(Kind::Split { .. }) => true,
            Some(Kind::Stage { child: Some(c) }) => matches!(
                self.entries.get(c).map(|e| &e.kind),
                Some(Kind::Split { .. })
            ),
            _ => false,
        }
    }

    fn hide_splits(&self, id: &Id, out: &mut HashSet<Id>) {
        match self.entries.get(id).map(|e| &e.kind) {
            Some(Kind::Split { children, .. }) => {
                out.insert(id.clone());
                for c in children {
                    self.hide_splits(c, out);
                }
            }
            Some(Kind::Stage { child: Some(c) }) => self.hide_splits(c, out),
            _ => {}
        }
    }

    fn collapse_visit(
        &self,
        id: &Id,
        size: f64,
        framed: Option<&Id>,
        groups: &mut Vec<Id>,
        hidden: &mut HashSet<Id>,
    ) {
        match self.entries.get(id).map(|e| &e.kind) {
            Some(Kind::Stage { child: Some(c) }) => {
                self.collapse_visit(c, size, framed, groups, hidden);
            }
            Some(Kind::Split { children, .. }) => {
                let nested = children.iter().any(|c| self.is_nested_split(c));
                if framed != Some(id)
                    && nested
                    && (self.is_small(id, size) || children.iter().all(|c| self.is_small(c, size)))
                {
                    groups.push(id.clone());
                    self.hide_splits(id, hidden);
                    return;
                }
                for c in children {
                    self.collapse_visit(c, size, framed, groups, hidden);
                }
            }
            _ => {}
        }
    }

    pub(crate) fn compute_collapsed(&mut self) {
        let mut groups = Vec::new();
        let mut hidden = HashSet::new();
        if let (Some(detail), true, Some(root)) = (
            self.opts.detail,
            self.nav_mode() != Navigation::Off,
            self.doc.root.as_ref().map(|r| r.id().clone()),
        ) {
            self.collapse_visit(
                &root,
                detail.size,
                self.nav.framed.as_ref(),
                &mut groups,
                &mut hidden,
            );
        }
        let mut of = HashMap::new();
        for g in &groups {
            for leaf in self.entries.leaf_ids(g) {
                of.insert(leaf, g.clone());
            }
        }
        self.fx.collapsed_of = of;
        self.fx.collapsed_groups = groups;
        self.fx.hidden_splits = hidden;
    }

    fn outline(
        &self,
        id: &Id,
        bounds: Rect,
        level: usize,
        depth: usize,
        out: &mut Vec<(Axis, f32, f32, f32)>,
    ) {
        let Some(e) = self.entries.get(id) else {
            return;
        };
        match &e.kind {
            Kind::Stage { child: Some(c) } => self.outline(c, bounds, level, depth, out),
            Kind::Split { axis, children, .. } if level < depth => {
                let r = e.rect;
                for child in children.iter().skip(1) {
                    let Some(c) = self.entries.rect(child) else {
                        continue;
                    };
                    out.push(match axis {
                        Axis::X => (
                            Axis::X,
                            ((c.x - bounds.x) / bounds.w) as f32,
                            ((r.y - bounds.y) / bounds.h) as f32,
                            (r.h / bounds.h) as f32,
                        ),
                        Axis::Y => (
                            Axis::Y,
                            ((c.y - bounds.y) / bounds.h) as f32,
                            ((r.x - bounds.x) / bounds.w) as f32,
                            (r.w / bounds.w) as f32,
                        ),
                    });
                }
                for c in children {
                    self.outline(c, bounds, level + 1, depth, out);
                }
            }
            _ => {}
        }
    }

    pub(crate) fn group_segment_at(&self, group: &Id, at: (f64, f64)) -> Id {
        let p = self.to_world(at);
        let depth = self.opts.detail.map_or(1, |d| d.outline.max(1));
        let mut target = group.clone();
        let mut current = group.clone();
        for _ in 0..depth {
            if let Some(Kind::Stage { child: Some(c) }) =
                self.entries.get(&current).map(|e| &e.kind)
            {
                current = c.clone();
            }
            let Some(Kind::Split { children, .. }) = self.entries.get(&current).map(|e| &e.kind)
            else {
                break;
            };
            let Some(child) = children
                .iter()
                .find(|c| self.entries.rect(c).is_some_and(|r| r.contains(p.0, p.1)))
            else {
                break;
            };
            target = child.clone();
            current = child.clone();
        }
        target
    }

    pub(crate) fn collapsed_layers(
        &mut self,
        layers: &mut Vec<(i32, AnyElement)>,
        p: &Palette,
        cx: &mut Context<Self>,
    ) {
        let depth = self.opts.detail.map_or(0, |d| d.outline);
        let cx = &*cx;
        for gid in self.fx.collapsed_groups.clone() {
            let Some(world) = self.entries.rect(&gid) else {
                continue;
            };
            let r = self.panel_screen(world);
            if !self.in_view(&r) {
                continue;
            }
            let mut lines = Vec::new();
            self.outline(&gid, world, 0, depth, &mut lines);
            let border = p.border;
            let line_els: Vec<AnyElement> = lines
                .into_iter()
                .map(|(axis, main, start, len)| match axis {
                    Axis::X => div()
                        .absolute()
                        .left(relative(main))
                        .top(relative(start))
                        .h(relative(len))
                        .w(px(1.0))
                        .bg(border)
                        .into_any_element(),
                    Axis::Y => div()
                        .absolute()
                        .top(relative(main))
                        .left(relative(start))
                        .w(relative(len))
                        .h(px(1.0))
                        .bg(border)
                        .into_any_element(),
                })
                .collect();
            let target = gid.clone();
            let el = place(div().id(format!("zopra-group-{gid}")), r)
            .rounded(px(10.0))
            .bg(p.panel)
            .border_1()
            .border_color(p.border)
            .overflow_hidden()
            .cursor_pointer()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, ev: &MouseDownEvent, _, cx| {
                    if ev.click_count >= 2 {
                        let segment = this.group_segment_at(&target, this.local(ev.position));
                        this.nav_focus(Some(segment), true, cx);
                    }
                }),
            )
            .children(line_els);
            layers.push((Z_COLLAPSED, el.into_any_element()));
        }
    }

    pub(crate) fn panel_at(&self, at: (f64, f64)) -> Option<(&PanelNode, Rect)> {
        let mut floats: Vec<&FloatingPanel> = self.doc.floating.iter().collect();
        floats.sort_by(|a, b| {
            (b.layer == FloatingLayer::Overlay)
                .cmp(&(a.layer == FloatingLayer::Overlay))
                .then(b.z.cmp(&a.z))
        });
        let docked = panels_of(self.doc.root.as_ref());
        floats
            .into_iter()
            .map(|f| &f.panel)
            .chain(docked)
            .find_map(|panel| {
                let r = *self.last_rects.get(&panel.id)?;
                r.contains(at.0, at.1).then_some((panel, r))
            })
    }

    pub(crate) fn gesture_owner_at(&self, at: (f64, f64)) -> Option<GestureOwner> {
        let (panel, r) = self.panel_at(at)?;
        (at.1 >= r.y + self.tabbar_height_of(panel))
            .then(|| self.rules_of(&panel.selected).gestures)
    }

    pub(crate) fn min_size_of(&self, view: &Id) -> Option<(f64, f64)> {
        let rules = self.rules_of(view);
        if rules.scaling == Scaling::Off {
            return None;
        }
        rules.min_size.or_else(|| {
            if self.nav_mode() == Navigation::Free {
                Some(FREE_MIN)
            } else {
                None
            }
        })
    }

    pub(crate) fn scaled_body(
        &mut self,
        view: &Id,
        size: (f64, f64),
        content: AnyElement,
    ) -> AnyElement {
        let scale = match self.min_size_of(view) {
            Some((mw, mh)) => 1.0_f64
                .min(size.0 / mw.max(1.0))
                .min(size.1 / mh.max(1.0))
                .max(0.05),
            None => 1.0,
        };
        self.fx
            .scales
            .insert(view.clone(), (scale * 1000.0).round() / 1000.0);
        if scale >= 0.999 || self.rules_of(view).scaling != Scaling::Inert {
            return content;
        }
        div()
            .size_full()
            .relative()
            .child(content)
            .child(div().absolute().size_full())
            .into_any_element()
    }

    #[must_use]
    pub fn view_scale(&self, view: &Id) -> f64 {
        self.fx.scales.get(view).copied().unwrap_or(1.0)
    }

    #[must_use]
    pub fn titlebar(&self, view: &Id) -> Titlebar {
        let overlaid =
            panel_of_view_id(self, view).is_some_and(|p| self.bar_mode(&p) == BarMode::Overlay);
        if !overlaid {
            return Titlebar::default();
        }
        let measured = self.fx.inset_widths.get(view).copied().unwrap_or(24.0);
        Titlebar {
            height: TABBAR_HEIGHT,
            inset_end: measured + 24.0,
        }
    }

    pub(crate) fn tab_shift(&self, view: &Id) -> f64 {
        let now = self.now();
        let sorting = self.drag.as_ref().and_then(|d| {
            d.tab_sort
                .as_ref()
                .filter(|s| s.started)
                .map(|s| (s, d.pending_tab.as_ref()))
        });
        match sorting {
            Some((sort, dragged)) if dragged == Some(view) => {
                sort.offsets.get(view).copied().unwrap_or(0.0)
            }
            Some(_) => self
                .fx
                .tab_motion
                .sort
                .get(view)
                .map_or(0.0, |s| s.value(now)),
            None => self
                .fx
                .tab_motion
                .flip
                .get(view)
                .map_or(0.0, |s| s.value(now)),
        }
    }

    pub(crate) fn step_fx(&mut self, now: f64) -> bool {
        self.fx.tab_motion.prune(now);
        self.fx.switch.retain(|_, s| !s.done(now));
        self.fx.tab_motion.active(now) || !self.fx.switch.is_empty()
    }

    pub(crate) fn note_tab_switch(&mut self, previous: &LayoutDocument) -> bool {
        if self.reduced() || self.lifted.is_some() {
            return false;
        }
        let panels = |doc: &LayoutDocument| -> Vec<PanelNode> {
            panels_of(doc.root.as_ref())
                .into_iter()
                .chain(doc.floating.iter().map(|f| &f.panel))
                .cloned()
                .collect()
        };
        let before: HashMap<Id, PanelNode> = panels(previous)
            .into_iter()
            .map(|p| (p.id.clone(), p))
            .collect();
        let now = self.now();
        let mut noted = false;
        for panel in panels(&self.doc) {
            let Some(old) = before.get(&panel.id) else {
                continue;
            };
            if old.selected == panel.selected || !panel.views.contains(&old.selected) {
                continue;
            }
            let index = |id: &Id| panel.views.iter().position(|v| v == id).unwrap_or(0);
            let dir = if index(&panel.selected) > index(&old.selected) {
                1.0
            } else {
                -1.0
            };
            self.fx.switch.insert(
                panel.id.clone(),
                Shift {
                    from: dir * SWITCH_SLIDE,
                    to: 0.0,
                    start: now,
                },
            );
            noted = true;
        }
        noted
    }

    pub(crate) fn switch_offset(&self, panel: &Id) -> (f64, f64) {
        self.fx.switch.get(panel).map_or((0.0, 1.0), |s| {
            let x = s.value(self.now());
            (x, 1.0 - 0.6 * (x / SWITCH_SLIDE).abs())
        })
    }

    pub(crate) fn divider_key(
        &mut self,
        split: &Id,
        index: usize,
        ev: &KeyDownEvent,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(Kind::Split { axis, .. }) = self.entries.get(split).map(|e| e.kind.clone()) else {
            return false;
        };
        let (decrease, increase) = if axis == Axis::X {
            ("left", "right")
        } else {
            ("up", "down")
        };
        let key = super::keymap::normalize_key(&ev.keystroke.key);
        if key != decrease && key != increase {
            return false;
        }
        let Some(root) = self.doc.root.clone() else {
            return false;
        };
        let Some(push) = drag_boundary(&root, self.metrics(), split, index) else {
            return false;
        };
        let size = self.entries.rect(split).map_or(1.0, |r| r.len(axis));
        let step = if ev.keystroke.modifiers.shift {
            0.1
        } else {
            0.02
        } * size;
        let next = push.to(push.start + if key == increase { step } else { -step });
        if self.doc.root.as_ref() != Some(&next) {
            let mut doc = self.doc.clone();
            doc.root = Some(next);
            self.commit(
                doc,
                CommitOptions {
                    animate: Some(false),
                    ..Default::default()
                },
                cx,
            );
        }
        true
    }

    /// A pinch (BUI reports a trackpad pinch as Ctrl + a touch scroll, as browsers do): zooms the
    /// camera about the pointer; the gesture settles when the trackpad goes idle. False when the
    /// view under the pointer owns its gestures.
    #[cfg(feature = "pinch")]
    pub(crate) fn nav_pinch(
        &mut self,
        at: blenderui::prelude::Point<blenderui::prelude::Pixels>,
        delta: f32,
        ended: bool,
        cx: &mut Context<Self>,
    ) -> bool {
        if ended {
            self.nav_finish_gesture(cx);
            return true;
        }
        if self.nav_mode() != Navigation::Free || self.interacting() || delta == 0.0 {
            return false;
        }
        let local = self.local_point(at);
        if self.gesture_owner_at(local) == Some(GestureOwner::Exclusive) {
            return false;
        }
        self.nav.swipe = None;
        let factor = 1.0 / (1.0 + f64::from(delta)).max(0.05);
        self.nav_zoom_banded(factor, local.0, local.1, cx);
        self.nav.wheel_deadline = Some(self.now() + super::nav::TOUCH_IDLE_MS);
        true
    }
}

fn panel_of_view_id(ws: &Workspace, view: &Id) -> Option<PanelNode> {
    super::model::document::panel_of_view(&ws.doc, view).cloned()
}
