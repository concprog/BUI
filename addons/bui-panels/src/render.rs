//! The workspace's element tree: one absolutely placed layer per panel, divider, slot and overlay,
//! sorted by z. Element ids (`zopra-panel-<id>`, `zopra-tab-<view>`, `zopra-div-<split>-<i>` ...)
//! name the parts for apps and tests.

use super::{
    drag::{DROP_SLOT, ResizeDir, SOURCE_SLOT},
    features::BarMode,
    icon::{Icon, IconName},
    keymap::{self, Command},
    menu::KeyResult,
    model::{
        tree::{Kind, find_stage, panels_of, sum},
        types::{Axis, FloatingLayer, FloatingPanel, Id, PanelNode, Rect},
    },
    nav::DragMode,
    workspace::{
        Badge, GAP, Navigation, Palette, Slot, TABBAR_HEIGHT, ViewHandle, ViewInstance, Workspace,
    },
};
use blenderui::prelude::{
    AnyElement, Context, Div, FluentBuilder as _, FontWeight, Hsla, IntoElement, KeyDownEvent,
    KeyUpEvent, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, ParentElement as _,
    Render, ScrollWheelEvent, SharedString, Styled, Window, canvas, div, px, relative,
};

type Layer = (i32, AnyElement);

const Z_BACKDROP: i32 = 1;
const Z_STAGE_EMPTY: i32 = 2;
const Z_EMPTY: i32 = 3;
const Z_DOCKED: i32 = 10;
const Z_DIVIDER: i32 = 20;
const Z_STAGE_FLOAT: i32 = 30;
const Z_OVERLAY_FLOAT: i32 = 1000;
const Z_SETTLING: i32 = 2900;
const Z_LEAVING: i32 = 2950;
const Z_DROP_SLOT: i32 = 3000;
const Z_LIFTED: i32 = 3100;
const Z_MARQUEE: i32 = 3400;
const Z_SNAP: i32 = 3500;
const Z_CHROME: i32 = 4000;
const Z_TOAST: i32 = 4050;
const Z_SHORTCUTS: i32 = 4100;
const Z_CAPTURE: i32 = 4200;
const Z_MENU: i32 = 4300;

const RADIUS: f32 = 10.0;
const TAB_RADIUS: f32 = 7.0;
/// `border-radius` of a tab bar and a tab: the top corners only.
const BAR_CORNERS: &str = "10px 10px 0 0";
const TAB_CORNERS: &str = "7px 7px 0 0";

/// The workspace's root element.
pub const ROOT_ID: &str = "zopra";

fn tab_id(view: &str) -> SharedString {
    format!("zopra-tab-{view}").into()
}

fn cluster_id(panel: &str) -> SharedString {
    format!("zopra-cluster-{panel}").into()
}

fn window_rect(b: blenderui::prelude::Bounds<f32>) -> Rect {
    Rect::new(
        f64::from(b.origin.x),
        f64::from(b.origin.y),
        f64::from(b.size.width),
        f64::from(b.size.height),
    )
}

pub(crate) fn place<E: Styled>(el: E, r: Rect) -> E {
    el.absolute()
        .left(px(r.x as f32))
        .top(px(r.y as f32))
        .w(px(r.w.max(0.0) as f32))
        .h(px(r.h.max(0.0) as f32))
}

impl Render for Workspace {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        for view in std::mem::take(&mut self.released) {
            view.release(cx);
        }
        self.measure(window);
        self.apply_view_ops(cx);
        if self.animating {
            self.step(window, cx);
        }
        let palette = self.palette();
        let layers = self.build_layers(&palette, window, cx);
        let ws = cx.entity();
        let held = self.nav.held_mode;

        // it fills its nearest positioned ancestor: the window when it is the root view
        let root = div()
            .id(ROOT_ID)
            .absolute()
            .inset_0()
            .overflow_hidden()
            .bg(palette.bg)
            .text_color(palette.text)
            .text_size(px(12.5))
            .when(held == Some(DragMode::Pan), Styled::cursor_grab)
            .when(held == Some(DragMode::Rect), Styled::cursor_crosshair)
            // blenderui renders a window again after a press, not after a key, a move or a wheel
            // its listeners handle: those that change something refresh it
            .on_key_down(move |ev: &KeyDownEvent, window, cx| {
                match ws.update(cx, |this, cx| this.on_key_down(ev, window, cx)) {
                    KeyResult::Ignored => {}
                    KeyResult::Handled => window.refresh(),
                    KeyResult::Run(run) => {
                        window.refresh();
                        if let Some(run) = run {
                            run(window, cx);
                        }
                    }
                }
            })
            .on_key_up(cx.listener(|this, ev: &KeyUpEvent, window, cx| {
                if this.on_key_up(ev, cx) {
                    window.refresh();
                }
            }))
            .on_scroll_wheel(cx.listener(|this, ev: &ScrollWheelEvent, window, cx| {
                if this.nav_wheel(ev, cx) {
                    window.refresh();
                    cx.stop_propagation();
                }
            }))
            .on_any_mouse_down(cx.listener(|this, ev: &MouseDownEvent, _, cx| {
                this.in_content = std::mem::take(&mut this.content_press);
                if matches!(ev.button, MouseButton::Left | MouseButton::Middle)
                    && this.nav_gesture_begin(ev.position, ev.button, ev.modifiers, cx)
                {
                    cx.stop_propagation();
                }
            }));
        let capture = self.capturing().then(|| (Z_CAPTURE, self.capture_layer(cx)));
        let menu = self.menu_layer(&palette, cx).map(|m| (Z_MENU, m));
        root.children(layers.into_iter().chain(capture).chain(menu).map(|(_, el)| el))
    }
}

#[derive(Clone, Copy)]
enum Side {
    Left,
    Right,
}

/// A press on a tile, a tab bar or a tab: a double click frames the panel, else a drag may start.
/// The workspace takes the keyboard (Escape cancels the drag) unless a modifier sends the press on
/// to the workspace's own gestures.
fn tab_press(
    cx: &Context<Workspace>,
    panel: Id,
    view: Option<Id>,
    focus_panel: bool,
) -> impl Fn(&MouseDownEvent, &mut Window, &mut blenderui::prelude::App) + 'static {
    cx.listener(move |this, ev: &MouseDownEvent, window, cx| {
        if focus_panel {
            this.panel_pointer_down(&panel, cx);
        }
        if this.tab_pointer_down(&panel, view.clone(), ev.position, ev.click_count, ev.modifiers, cx) {
            this.in_content = false;
            window.focus_id(ROOT_ID);
            cx.stop_propagation();
        }
    })
}

/// The concave corner that joins a selected tab to the tab bar's floor.
fn flare(side: Side, color: Hsla) -> Div {
    use std::f32::consts::{FRAC_PI_2, PI};
    const STEPS: usize = 8;
    let r = TAB_RADIUS;
    let el = canvas(move |b, painter| {
        let (ox, oy) = (b.origin.x, b.origin.y);
        let mut points = [(0.0_f32, 0.0_f32); STEPS + 2];
        let (first, last) = match side {
            Side::Left => ((r, 0.0), (r, r)),
            Side::Right => ((0.0, 0.0), (0.0, r)),
        };
        points[0] = (ox + first.0, oy + first.1);
        for (i, point) in points[1..=STEPS].iter_mut().enumerate() {
            let t = FRAC_PI_2 * (i + 1) as f32 / STEPS as f32;
            *point = match side {
                Side::Left => (ox + r * t.cos(), oy + r * t.sin()),
                Side::Right => {
                    let t = PI - t;
                    (ox + r + r * t.cos(), oy + r * t.sin())
                }
            };
        }
        points[STEPS + 1] = (ox + last.0, oy + last.1);
        painter.polygon(&points, color);
    })
    .absolute()
    .bottom_0()
    .size(px(r));
    match side {
        Side::Left => el.right(relative(1.0)),
        Side::Right => el.left(relative(1.0)),
    }
}

impl Workspace {
    /// Reads back where the last frame put the workspace and its tabs (blenderui answers
    /// `bounds_of` while rendering), and follows a resize.
    fn measure(&mut self, window: &mut Window) {
        self.bounds = window.bounds_of(ROOT_ID).map_or_else(
            || {
                let s = window.viewport_size();
                Rect::new(0.0, 0.0, f64::from(s.width.0), f64::from(s.height.0))
            },
            window_rect,
        );
        let size = (self.bounds.w, self.bounds.h);
        if size.0 > 0.0 && size.1 > 0.0 && size != self.viewport {
            self.viewport = size;
            self.tween.stop();
            self.sync();
            self.wake_frame(window);
        }
        // the tabs this frame renders put their rects back (`render_tabbar`)
        self.tab_rects.clear();
    }

    /// One more frame: this one was laid out for the old size.
    fn wake_frame(&mut self, window: &mut Window) {
        self.animating = true;
        self.last_frame = None;
        window.request_animation_frame();
    }

    fn capture_layer(&self, cx: &mut Context<Self>) -> AnyElement {
        let cursor = match (&self.divider, &self.nav.gesture_drag) {
            (Some(d), _) if d.push.axis == Axis::X => "col-resize",
            (Some(_), _) => "row-resize",
            (_, Some(g)) if g.mode == DragMode::Rect => "crosshair",
            (_, Some(_)) => "grabbing",
            _ if self.drag.as_ref().is_some_and(|d| d.active) => "grabbing",
            _ => "default",
        };
        div()
            .id("zopra-capture")
            .fixed_position()
            .inset_0()
            .css("cursor", cursor)
            .on_mouse_move(cx.listener(|this, ev: &MouseMoveEvent, window, cx| {
                if matches!(ev.pressed_button, Some(MouseButton::Left | MouseButton::Middle)) {
                    this.on_pointer_move(ev.position, ev.modifiers, cx);
                } else {
                    this.on_pointer_up(cx);
                }
                window.refresh();
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _: &MouseUpEvent, _, cx| this.on_pointer_up(cx)),
            )
            .on_mouse_up(
                MouseButton::Middle,
                cx.listener(|this, _: &MouseUpEvent, _, cx| this.on_pointer_up(cx)),
            )
            .into_any_element()
    }

    fn ensure_view(&mut self, view_id: &Id, window: &mut Window, cx: &mut Context<Self>) {
        if self.views.contains_key(view_id) {
            return;
        }
        let Some(def) = self
            .doc
            .views
            .get(view_id)
            .and_then(|r| self.types.get(&r.ty))
            .cloned()
        else {
            return;
        };
        let handle = ViewHandle {
            id: view_id.clone(),
            workspace: self.me,
        };
        let content = (def.build)(&handle, window, cx);
        self.views.insert(view_id.clone(), ViewInstance { content });
    }

    fn build_layers(
        &mut self,
        p: &Palette,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Vec<Layer> {
        let mut layers: Vec<Layer> = Vec::new();
        if self.viewport.0 <= 0.0 {
            return layers;
        }
        self.compute_collapsed();
        self.stage_layers(&mut layers, p, window, cx);
        self.panel_layers(&mut layers, p, window, cx);
        self.collapsed_layers(&mut layers, p, cx);
        self.divider_layers(&mut layers, p, window, cx);
        self.slot_layers(&mut layers, p);
        self.overlay_layers(&mut layers, p, window, cx);
        layers.sort_by_key(|(z, _)| *z);
        layers
    }

    fn stage_layers(
        &mut self,
        layers: &mut Vec<Layer>,
        p: &Palette,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let stage = find_stage(self.doc.root.as_ref()).cloned();
        if let Some(stage) = &stage {
            let preview = self
                .drag
                .as_ref()
                .filter(|d| d.active)
                .and_then(|d| d.layout_targets.get(&stage.id).copied());
            let screen = preview
                .map(|w| self.panel_screen(w))
                .or_else(|| self.stage_screen());
            if let Some(s) = screen {
                let r = self.tween.apply(&stage.id, s);
                self.last_rects.insert(stage.id.clone(), r);
                let mut backdrop = place(div(), r)
                    .rounded(px(RADIUS))
                    .overflow_hidden()
                    .bg(p.stage);
                if let Some(slot) = self.slot(Slot::Backdrop, window, cx) {
                    backdrop = backdrop.child(slot);
                }
                layers.push((Z_BACKDROP, backdrop.into_any_element()));
                if stage.child.is_none()
                    && let Some(slot) = self.slot(Slot::StageEmpty, window, cx)
                {
                    let el = place(div(), r)
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(slot);
                    layers.push((Z_STAGE_EMPTY, el.into_any_element()));
                }
            }
        }
        if self.doc.root.is_none()
            && self.doc.floating.is_empty()
            && self.lifted.is_none()
            && let Some(slot) = self.slot(Slot::Empty, window, cx)
        {
            let el = div()
                .absolute()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .child(slot);
            layers.push((Z_EMPTY, el.into_any_element()));
        }
    }

    fn slot_layers(&mut self, layers: &mut Vec<Layer>, p: &Palette) {
        let Some(d) = self.drag.as_ref().filter(|d| d.active) else {
            self.last_rects.remove(DROP_SLOT);
            self.last_rects.remove(SOURCE_SLOT);
            return;
        };
        let label = d.drop_label;
        let (source, drop) = (
            d.layout_targets.get(SOURCE_SLOT).copied(),
            d.layout_targets.get(DROP_SLOT).copied(),
        );
        for (id, world) in [(SOURCE_SLOT, source), (DROP_SLOT, drop)] {
            let Some(world) = world else {
                self.last_rects.remove(id);
                continue;
            };
            let r = self.tween.apply(id, self.panel_screen(world));
            self.last_rects.insert(id.into(), r);
            if id == DROP_SLOT && r.w > 8.0 && r.h > 8.0 {
                let slot = place(div().id(DROP_SLOT), r)
                    .rounded(px(RADIUS))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(p.text_muted);
                let slot = if label.is_empty() {
                    slot.bg(p.slot)
                } else {
                    slot.bg(p.slot.opacity(0.7))
                        .border_2()
                        .border_color(p.accent)
                };
                layers.push((Z_DROP_SLOT, slot.child(label).into_any_element()));
            }
        }
    }

    fn shortcuts_layer(&self, p: &Palette, cx: &mut Context<Self>) -> AnyElement {
        let keymap = self.keymap();
        let keys = &self.opts.gesture_keys;
        let chord = |combos: &[String]| {
            combos.first().map_or_else(String::new, |c| {
                c.split('+')
                    .map(|part| {
                        let mac = keymap::is_mac();
                        match part.trim().to_ascii_lowercase().as_str() {
                            "mod" => if mac { "⌘" } else { "Ctrl" }.to_string(),
                            "ctrl" | "control" => if mac { "⌃" } else { "Ctrl" }.to_string(),
                            "alt" | "option" => if mac { "⌥" } else { "Alt" }.to_string(),
                            "shift" => if mac { "⇧" } else { "Shift" }.to_string(),
                            "space" => "Space".to_string(),
                            other => keymap::format_combo(other),
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("+")
            })
        };
        let with = |c: String, rest: &str| {
            if c.is_empty() {
                rest.to_string()
            } else {
                format!("{c} + {rest}")
            }
        };
        let two = |combos: &[String]| with(chord(combos), "two fingers");
        let mod_key = ["Mod".to_string()];
        let keyboard: Vec<(String, String)> = Command::ALL
            .iter()
            .filter_map(|cmd| {
                let combo = keymap.get(cmd)?.as_ref()?;
                Some((cmd.label().to_string(), keymap::format_combo(combo)))
            })
            .collect();
        let mut gestures = Vec::new();
        if self.nav_mode() == Navigation::Free {
            gestures.extend([
                ("Swipe between panels".into(), "Two fingers".into()),
                ("Zoom".into(), "Pinch".into()),
                ("Scale".into(), two(&keys.touch_scale)),
                ("Step in / out".into(), two(&keys.step)),
                ("Step (mouse)".into(), with(chord(&mod_key), "wheel")),
                ("Pan (mouse)".into(), "Middle drag".into()),
                ("Scale (mouse)".into(), with(chord(&keys.scale), "drag")),
                ("Focus area (mouse)".into(), with(chord(&keys.rect), "drag")),
            ]);
        }
        let section = |title: &'static str, rows: Vec<(String, String)>| {
            div()
                .flex()
                .flex_col()
                .gap(px(6.0))
                .min_w(px(240.0))
                .child(
                    div()
                        .text_size(px(11.0))
                        .text_color(p.text_muted)
                        .child(title),
                )
                .children(rows.into_iter().map(|(label, combo)| {
                    div()
                        .flex()
                        .justify_between()
                        .gap(px(16.0))
                        .child(label)
                        .child(
                            div()
                                .px(px(6.0))
                                .rounded(px(4.0))
                                .bg(p.tab_active)
                                .border_1()
                                .border_color(p.border)
                                .text_color(p.text_muted)
                                .child(combo),
                        )
                }))
        };
        let mut card = div()
            .id("zopra-shortcuts")
            .flex()
            .flex_wrap()
            .gap(px(28.0))
            .p(px(20.0))
            .max_w(px(600.0))
            .rounded(px(RADIUS + 3.0))
            .bg(p.panel)
            .border_1()
            .border_color(p.border)
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .child(section("Keyboard", keyboard));
        if !gestures.is_empty() {
            card = card.child(section("Touchpad & mouse", gestures));
        }
        div()
            .id("zopra-shortcuts-backdrop")
            .absolute()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(blenderui::prelude::black().opacity(0.35))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _: &MouseDownEvent, _, cx| {
                    this.nav.shortcuts = false;
                    cx.notify();
                }),
            )
            .child(card)
            .into_any_element()
    }

    fn overlay_layers(
        &mut self,
        layers: &mut Vec<Layer>,
        p: &Palette,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let white = blenderui::prelude::white();
        let frame = |r: Rect, label: Option<String>| {
            place(div(), r)
                .pointer_events_none()
                .rounded(px(RADIUS + 3.0))
                .border_2()
                .border_color(white.opacity(0.6))
                .children(label.map(|label| {
                    div()
                        .absolute()
                        .top(px(12.0))
                        .w_full()
                        .flex()
                        .justify_center()
                        .child(
                            div()
                                .px(px(11.0))
                                .py(px(7.0))
                                .rounded_full()
                                .bg(blenderui::prelude::black().opacity(0.75))
                                .text_color(white)
                                .text_size(px(10.0))
                                .child(label),
                        )
                }))
                .into_any_element()
        };
        if let Some(r) = self.nav.snap_preview {
            layers.push((Z_SNAP, frame(r, None)));
        }
        if let Some((r, label)) = self.nav.marquee_target.clone() {
            layers.push((Z_SNAP, frame(r, Some(label))));
        }
        if let Some(r) = self.nav.marquee {
            layers.push((
                Z_MARQUEE,
                place(div(), r)
                    .rounded(px(4.0))
                    .border_1()
                    .border_color(p.accent)
                    .bg(p.accent.opacity(0.12))
                    .into_any_element(),
            ));
        }
        if self.nav.shortcuts {
            layers.push((Z_SHORTCUTS, self.shortcuts_layer(p, cx)));
        }
        if let Some(title) = self.toast().map(|c| c.title.clone()) {
            let hint = self
                .keymap()
                .get(&Command::ViewReopen)
                .cloned()
                .flatten()
                .map(|c| keymap::format_combo(&c));
            layers.push((
                Z_TOAST,
                div()
                    .absolute()
                    .bottom(px(20.0))
                    .left_0()
                    .right_0()
                    .flex()
                    .justify_center()
                    .pointer_events_none()
                    .child(
                        div()
                            .id("zopra-undo-toast")
                            .css("pointer-events", "auto")
                            .flex()
                            .items_center()
                            .gap(px(10.0))
                            .pl(px(14.0))
                            .pr(px(6.0))
                            .py(px(5.0))
                            .rounded_full()
                            .bg(blenderui::prelude::black().opacity(0.8))
                            .text_color(white)
                            .child(format!("Closed “{title}”"))
                            .child(
                                div()
                                    .id("zopra-undo")
                                    .flex()
                                    .gap(px(6.0))
                                    .px(px(10.0))
                                    .py(px(3.0))
                                    .rounded_full()
                                    .bg(white.opacity(0.14))
                                    .hover(|s| s.bg(white.opacity(0.24)))
                                    .cursor_pointer()
                                    .child("Undo")
                                    .children(hint.map(|h| div().opacity(0.6).child(h)))
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _: &MouseDownEvent, _, cx| {
                                            this.reopen_closed(cx);
                                            cx.stop_propagation();
                                        }),
                                    ),
                            ),
                    )
                    .into_any_element(),
            ));
        }
        if let Some(slot) = self.slot(Slot::Chrome, window, cx) {
            layers.push((
                Z_CHROME,
                div()
                    .absolute()
                    .size_full()
                    .pointer_events_none()
                    .child(div().size_full().css("pointer-events", "auto").child(slot))
                    .into_any_element(),
            ));
        }
    }

    fn panel_layers(
        &mut self,
        layers: &mut Vec<Layer>,
        p: &Palette,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let time = self.now();
        let mut panels: Vec<PanelNode> = panels_of(self.doc.root.as_ref())
            .into_iter()
            .cloned()
            .collect();
        panels.extend(self.doc.floating.iter().map(|f| f.panel.clone()));
        if let Some(l) = &self.lifted {
            panels.push(l.clone());
        }
        panels.extend(self.leaving.values().map(|l| l.panel.clone()));
        let mut seen = std::collections::HashSet::new();
        panels.retain(|panel| seen.insert(panel.id.clone()));

        let float_z: std::collections::HashMap<Id, i32> = {
            let mut order: Vec<&FloatingPanel> = self.doc.floating.iter().collect();
            order.sort_by_key(|f| f.z);
            let mut seen = std::collections::HashMap::<FloatingLayer, i32>::new();
            order
                .into_iter()
                .map(|f| {
                    let n = seen.entry(f.layer).or_default();
                    *n += 1;
                    (f.panel.id.clone(), *n - 1)
                })
                .collect()
        };
        let lifted_id = self.lifted.as_ref().map(|l| l.id.clone());

        for panel in panels {
            let id = panel.id.clone();
            let leaving = self
                .leaving
                .get(&id)
                .map(|l| (l.from, l.to, l.start, l.duration));
            let entering = self.entering.get(&id).map(|e| (e.from, e.start));
            let (r, opacity) = if let Some((from, to, start, duration)) = leaving {
                let e = super::motion::dock_ease(((time - start) / duration).min(1.0));
                (from.lerp(&to, e), 1.0 - 0.9 * e)
            } else if let Some((from, start)) = entering {
                let Some(target) = self.target_rect(&id) else {
                    continue;
                };
                let e = super::motion::dock_ease(
                    ((time - start) / super::motion::DOCK_RESTORE_MS).min(1.0),
                );
                (from.lerp(&target, e), 0.1 + 0.9 * e)
            } else {
                let Some(target) = self.target_rect(&id) else {
                    continue;
                };
                if lifted_id.as_ref() == Some(&id) {
                    (target, 1.0)
                } else {
                    (self.tween.apply(&id, target), 1.0)
                }
            };
            self.last_rects.insert(id.clone(), r);
            if self.fx.collapsed_of.contains_key(&id) && leaving.is_none() {
                continue;
            }
            if !(self.in_view(&r) && r.w > 2.0 && r.h > 2.0) {
                continue;
            }

            let float = self
                .doc
                .floating
                .iter()
                .find(|f| f.panel.id == id)
                .map(|f| f.layer);
            let settling = self.settling.contains(&id);
            let z = if lifted_id.as_ref() == Some(&id) {
                Z_LIFTED
            } else if leaving.is_some() {
                Z_LEAVING
            } else if settling {
                Z_SETTLING
            } else {
                match float {
                    Some(FloatingLayer::Stage) => {
                        Z_STAGE_FLOAT + 3 * float_z.get(&id).copied().unwrap_or(0)
                    }
                    Some(FloatingLayer::Overlay) => {
                        Z_OVERLAY_FLOAT + 3 * float_z.get(&id).copied().unwrap_or(0)
                    }
                    None => Z_DOCKED,
                }
            };
            let is_lifted = lifted_id.as_ref() == Some(&id);
            let mode = self.bar_mode(&panel);
            let frame_only = self.is_frame_only(&panel, &r);
            if frame_only {
                self.fx.frame_only.insert(id.clone());
            } else {
                self.fx.frame_only.remove(&id);
            }
            let bar = if mode == BarMode::Normal {
                TABBAR_HEIGHT
            } else {
                0.0
            };
            let compact = r.w < 140.0 || r.h < bar + 24.0;
            let el = if frame_only {
                self.render_frame_tile(&panel, r, opacity as f32, p, cx)
            } else {
                self.render_panel(
                    &panel,
                    r,
                    opacity as f32,
                    float.is_some() || is_lifted,
                    mode,
                    compact,
                    p,
                    window,
                    cx,
                )
            };
            layers.push((z, el));
            if float.is_some() && !is_lifted && leaving.is_none() && !frame_only {
                layers.push((z + 2, self.render_handles(&id, r, cx)));
            }
        }
    }

    fn render_frame_tile(
        &mut self,
        panel: &PanelNode,
        r: Rect,
        opacity: f32,
        p: &Palette,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let size = (r.w - 12.0).min(r.h - 12.0).clamp(0.0, 40.0) as f32;
        let icon = match self.type_of(&panel.selected).and_then(|t| t.icon) {
            Some(i) => Icon::new(i)
                .size(px(size * 0.6))
                .color(p.text_muted)
                .into_any_element(),
            None => div()
                .text_size(px(size * 0.6))
                .font_weight(FontWeight::BOLD)
                .child(
                    self.title_shared(&panel.selected)
                        .chars()
                        .next()
                        .map(|c| c.to_uppercase().to_string())
                        .unwrap_or_default(),
                )
                .into_any_element(),
        };
        let pid = panel.id.clone();
        place(div().id(format!("zopra-tile-{}", panel.id)), r)
            .css("user-select", "none")
            .rounded(px(RADIUS))
            .bg(p.panel)
            .border_1()
            .border_color(p.border)
            .opacity(opacity)
            .cursor_grab()
            .flex()
            .items_center()
            .justify_center()
            .text_color(p.text_muted)
            .on_mouse_down(MouseButton::Left, tab_press(cx, pid, None, true))
            .child(icon)
            .into_any_element()
    }

    #[allow(clippy::too_many_arguments)]
    fn render_panel(
        &mut self,
        panel: &PanelNode,
        r: Rect,
        opacity: f32,
        raised: bool,
        mode: BarMode,
        compact: bool,
        p: &Palette,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let bar = if mode == BarMode::Normal {
            TABBAR_HEIGHT
        } else {
            0.0
        };
        let focused = self.focused_panel.as_ref() == Some(&panel.id);
        let content = self.render_content(&panel.selected, p, window, cx);
        let body = self.scaled_body(&panel.selected, (r.w, (r.h - bar).max(0.0)), content);
        let (slide, fade) = self.switch_offset(&panel.id);
        let body = div()
            .size_full()
            .relative()
            .left(px(slide as f32))
            .opacity(fade as f32)
            .child(body);
        let tabbar = (mode != BarMode::Hidden).then(|| {
            self.render_tabbar(
                panel,
                focused,
                p,
                mode == BarMode::Overlay,
                compact,
                window,
                cx,
            )
        });
        let pid = panel.id.clone();
        place(div().id(format!("zopra-panel-{}", panel.id)), r)
            .rounded(px(RADIUS))
            .bg(p.panel)
            .border_1()
            .border_color(p.border)
            .opacity(opacity)
            .when(raised, Styled::shadow_lg)
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _: &MouseDownEvent, _, cx| this.panel_pointer_down(&pid, cx)),
            )
            .child(
                div()
                    .id(format!("zopra-body-{}", panel.id))
                    .absolute()
                    .top(px(bar as f32))
                    .left_0()
                    .right_0()
                    .bottom_0()
                    .overflow_hidden()
                    .on_any_mouse_down(cx.listener(|this, _: &MouseDownEvent, _, _| {
                        this.content_press = true;
                    }))
                    .child(body),
            )
            .children(tabbar)
            .into_any_element()
    }

    /// A view's content: declared in `<View>` / `<Template>` markup, or built by its view type.
    fn render_content(
        &mut self,
        view: &Id,
        p: &Palette,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        if let Some(element) = self.hosted_content(view, window, cx) {
            return div().size_full().child(element).into_any_element();
        }
        self.ensure_view(view, window, cx);
        if let Some(instance) = self.views.get(view) {
            return div()
                .size_full()
                .child(instance.content.render(window, cx))
                .into_any_element();
        }
        let ty = self
            .doc
            .views
            .get(view)
            .map(|r| r.ty.clone())
            .unwrap_or_default();
        div()
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(4.0))
            .text_color(p.text_muted)
            .child(
                div()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(p.text)
                    .child("Unavailable"),
            )
            .child(format!("No view type named “{ty}” is registered."))
            .into_any_element()
    }

    #[allow(clippy::too_many_arguments)]
    fn render_tabbar(
        &mut self,
        panel: &PanelNode,
        focused: bool,
        p: &Palette,
        overlay: bool,
        compact: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let inset = self.opts.tabs.inset;
        let mut tabs = Vec::with_capacity(panel.views.len());
        for view in &panel.views {
            let id = tab_id(view);
            if let Some(b) = window.bounds_of(&id) {
                self.tab_rects.insert(view.clone(), window_rect(b));
            }
            tabs.push(self.render_tab(panel, view, id, focused, p, overlay, compact, cx));
        }
        let (pid, menu_pid, ctx_pid) = (panel.id.clone(), panel.id.clone(), panel.id.clone());

        let menu_button = div()
            .id(format!("zopra-menu-{}", panel.id))
            .flex()
            .items_center()
            .justify_center()
            .size(px(22.0))
            .rounded(px(5.0))
            .cursor_pointer()
            .hover(|s| s.bg(p.tab_hover))
            .child(Icon::new(IconName::Ellipsis).size(px(14.0)).color(p.text_muted))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, ev: &MouseDownEvent, _, cx| {
                    let (x, y) = this.local(ev.position);
                    let below = this
                        .last_rects
                        .get(&menu_pid)
                        .map_or(y + 12.0, |r| r.y + TABBAR_HEIGHT);
                    this.open_menu(&menu_pid, (x - 150.0, below), cx);
                    cx.stop_propagation();
                }),
            );

        let accessory = (!compact)
            .then(|| {
                self.type_of(&panel.selected)
                    .and_then(|t| t.accessory.clone())
            })
            .flatten()
            .map(|build| {
                let handle = ViewHandle {
                    id: panel.selected.clone(),
                    workspace: self.me,
                };
                div()
                    .flex()
                    .items_center()
                    .gap(px(4.0))
                    .child(build(&handle, window, cx))
            })
            .or_else(|| {
                self.hosted_accessory(&panel.selected)
                    .map(|el| div().flex().items_center().gap(px(4.0)).child(el))
            });

        let cluster = cluster_id(&panel.id);
        if let Some(b) = window.bounds_of(&cluster) {
            self.fx
                .inset_widths
                .insert(panel.selected.clone(), f64::from(b.size.width));
        }
        let cluster = div()
            .id(cluster)
            .relative()
            .flex()
            .items_center()
            .gap(px(4.0))
            .children(accessory)
            .child(menu_button);

        div()
            .id(format!("zopra-bar-{}", panel.id))
            .css("user-select", "none")
            .on_hover(cx.listener(|this, hovered: &bool, window, cx| {
                if !*hovered && this.fx.frozen_tabs.take().is_some() {
                    cx.notify();
                    window.refresh();
                }
            }))
            .absolute()
            .top_0()
            .left_0()
            .right_0()
            .h(px(TABBAR_HEIGHT as f32))
            .flex()
            .items_stretch()
            .gap(px(2.0))
            .pr(px(2.0))
            .overflow_hidden()
            .when(!overlay, |d| {
                d.bg(p.tabbar).css("border-radius", BAR_CORNERS).child(
                    div()
                        .absolute()
                        .left_0()
                        .right_0()
                        .bottom_0()
                        .h(px(1.0))
                        .bg(p.border),
                )
            })
            .on_mouse_down(MouseButton::Left, tab_press(cx, pid, None, false))
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(move |this, ev: &MouseDownEvent, _, cx| {
                    let at = this.local(ev.position);
                    this.open_menu(&ctx_pid, at, cx);
                    cx.stop_propagation();
                }),
            )
            .child(
                div()
                    .flex()
                    .flex_1()
                    .min_w(px(0.0))
                    .items_stretch()
                    .gap(px(2.0))
                    .pt(px(inset))
                    .px(px(inset.max(TAB_RADIUS)))
                    .children(tabs),
            )
            .child(div().flex().items_center().child(cluster))
            .into_any_element()
    }

    #[allow(clippy::too_many_arguments)]
    fn render_tab(
        &self,
        panel: &PanelNode,
        view: &Id,
        id: SharedString,
        focused: bool,
        p: &Palette,
        overlay: bool,
        compact: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let selected = panel.selected == *view;
        let closable = self.rules_of(view).is_closable();
        let shift = self.tab_shift(view) as f32;
        let dragging = self.drag.as_ref().is_some_and(|d| {
            d.tab_sort.as_ref().is_some_and(|s| s.started) && d.pending_tab.as_ref() == Some(view)
        });
        let icon = self.type_of(view).and_then(|t| t.icon);
        let (text, muted, tab_hover, accent, panel_bg) =
            (p.text, p.text_muted, p.tab_hover, p.accent, p.tab_active);
        let accent_contrast = p.accent_contrast;

        let frozen = self
            .fx
            .frozen_tabs
            .as_ref()
            .filter(|(p, _)| *p == panel.id)
            .map(|(_, w)| *w);
        let (pid, vid, close_vid, mid_vid) =
            (panel.id.clone(), view.clone(), view.clone(), view.clone());

        let close = (closable && !compact).then(|| {
            div()
                .id(format!("zopra-close-{view}"))
                .flex()
                .items_center()
                .justify_center()
                .size(px(20.0))
                .rounded(px(5.0))
                .class(if selected {
                    "opacity-75"
                } else {
                    "opacity-0 group-hover:opacity-75"
                })
                .hover(move |s| s.opacity(1.0).bg(tab_hover))
                .child(Icon::new(IconName::Close).size(px(12.0)).color(text))
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _: &MouseDownEvent, _, cx| {
                        this.close_tab_frozen(&close_vid, cx);
                        cx.stop_propagation();
                    }),
                )
        });

        let badge_el = self.badges.get(view).filter(|_| !compact).map(|b| match b {
            Badge::Dot => div()
                .size(px(8.0))
                .flex_none()
                .rounded_full()
                .bg(muted)
                .into_any_element(),
            Badge::Text(t) => div()
                .px(px(5.0))
                .h(px(16.0))
                .flex_none()
                .rounded(px(8.0))
                .bg(accent)
                .text_color(accent_contrast)
                .text_size(px(10.0))
                .font_weight(FontWeight::SEMIBOLD)
                .child(t.clone())
                .into_any_element(),
        });

        div()
            .id(id)
            .class("group")
            .relative()
            .left(px(shift))
            .flex()
            .items_center()
            .gap(px(6.0))
            .min_w(px(if compact { 32.0 } else { 72.0 }))
            .max_w(px(220.0))
            .pl(px(if compact { 8.0 } else { 10.0 }))
            .pr(px(if compact { 8.0 } else { 6.0 }))
            .css("border-radius", TAB_CORNERS)
            .font_weight(FontWeight::MEDIUM)
            .text_color(if selected { text } else { muted })
            .when(overlay, |d| d.opacity(0.0))
            .when(self.opts.tabs.fill, Styled::flex_1)
            .when_some(frozen, |d, w| {
                let w = px(w as f32);
                d.flex_none().w(w).min_w(w).max_w(w)
            })
            .when(selected, |d| {
                d.bg(panel_bg)
                    .when(focused, |d| d.border_t_2().border_color(accent))
            })
            .when(!selected, |d| {
                d.hover(move |s| s.bg(tab_hover).text_color(text))
            })
            .when(dragging, Styled::shadow_md)
            .when(selected && !overlay, |d| {
                d.child(flare(Side::Left, panel_bg))
                    .child(flare(Side::Right, panel_bg))
            })
            .children(icon.map(|i| Icon::new(i).size(px(14.0)).color(if selected { text } else { muted })))
            .when(!compact, |d| {
                d.child(
                    div()
                        .flex_1()
                        .min_w(px(0.0))
                        .truncate()
                        .child(self.title_of(view)),
                )
            })
            .children(badge_el)
            .children(close)
            .on_mouse_down(MouseButton::Left, tab_press(cx, pid, Some(vid), false))
            .on_mouse_down(
                MouseButton::Middle,
                cx.listener(move |this, _: &MouseDownEvent, _, cx| {
                    if this.rules_of(&mid_vid).is_closable() {
                        this.close_tab_frozen(&mid_vid, cx);
                    }
                }),
            )
            .into_any_element()
    }

    fn render_handles(&self, panel_id: &Id, r: Rect, cx: &mut Context<Self>) -> AnyElement {
        const HANDLES: [(&str, ResizeDir, &str); 8] = [
            ("n", ResizeDir { n: true, s: false, e: false, w: false }, "ns-resize"),
            ("s", ResizeDir { n: false, s: true, e: false, w: false }, "ns-resize"),
            ("e", ResizeDir { n: false, s: false, e: true, w: false }, "ew-resize"),
            ("w", ResizeDir { n: false, s: false, e: false, w: true }, "ew-resize"),
            ("ne", ResizeDir { n: true, s: false, e: true, w: false }, "nesw-resize"),
            ("nw", ResizeDir { n: true, s: false, e: false, w: true }, "nwse-resize"),
            ("se", ResizeDir { n: false, s: true, e: true, w: false }, "nwse-resize"),
            ("sw", ResizeDir { n: false, s: true, e: false, w: true }, "nesw-resize"),
        ];
        let handles = HANDLES.iter().map(|&(name, dir, cursor)| {
            let pid = panel_id.clone();
            let edge = div()
                .id(format!("zopra-resize-{panel_id}-{name}"))
                .absolute()
                .css("cursor", cursor);
            let edge = match name {
                "n" => edge.top(px(-4.0)).left(px(8.0)).right(px(8.0)).h(px(8.0)),
                "s" => edge.bottom(px(-4.0)).left(px(8.0)).right(px(8.0)).h(px(8.0)),
                "e" => edge.right(px(-4.0)).top(px(8.0)).bottom(px(8.0)).w(px(8.0)),
                "w" => edge.left(px(-4.0)).top(px(8.0)).bottom(px(8.0)).w(px(8.0)),
                "ne" => edge.right(px(-5.0)).top(px(-5.0)).size(px(14.0)),
                "nw" => edge.left(px(-5.0)).top(px(-5.0)).size(px(14.0)),
                "se" => edge.right(px(-5.0)).bottom(px(-5.0)).size(px(14.0)),
                _ => edge.left(px(-5.0)).bottom(px(-5.0)).size(px(14.0)),
            };
            edge.on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, ev: &MouseDownEvent, window, cx| {
                    this.resize_begin(pid.clone(), dir, ev.position, cx);
                    window.focus_id(ROOT_ID);
                    cx.stop_propagation();
                }),
            )
        });
        place(div(), r)
            .pointer_events_none()
            .children(handles.map(|h| h.css("pointer-events", "auto")))
            .into_any_element()
    }

    fn divider_layers(
        &mut self,
        layers: &mut Vec<Layer>,
        p: &Palette,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        struct Item {
            split: Id,
            index: usize,
            axis: Axis,
            rect: Rect,
            size: f64,
        }
        if self.tween.active() || self.drag.as_ref().is_some_and(|d| d.active) || self.nav.gesture {
            return;
        }
        let (vw, vh) = self.viewport;
        let active = self.divider.as_ref().map(|d| (d.split.clone(), d.index));
        let accent = p.accent;

        let mut items = Vec::new();
        for entry in self.entries.iter() {
            let Kind::Split {
                axis,
                weights,
                children,
            } = &entry.kind
            else {
                continue;
            };
            if self.fx.hidden_splits.contains(&entry.id) {
                continue;
            }
            let r = self.to_screen(entry.rect);
            let total = sum(weights);
            for index in 0..children.len() - 1 {
                let boundary = weights[..=index].iter().sum::<f64>() / total;
                let size = 8.0_f64.max(GAP + 4.0);
                let rect = match axis {
                    Axis::X => Rect::new(
                        r.x + r.w * boundary - size / 2.0,
                        r.y + self.pad(),
                        size,
                        r.h - GAP,
                    ),
                    Axis::Y => Rect::new(
                        r.x + self.pad(),
                        r.y + r.h * boundary - size / 2.0,
                        r.w - GAP,
                        size,
                    ),
                };
                if rect.x < vw && rect.y < vh && rect.right() > 0.0 && rect.bottom() > 0.0 {
                    items.push(Item {
                        split: entry.id.clone(),
                        index,
                        axis: *axis,
                        rect,
                        size,
                    });
                }
            }
        }

        for item in items {
            let id: SharedString = format!("zopra-div-{}-{}", item.split, item.index).into();
            let focused = window.focused_id() == Some(&*id);
            let is_active = active
                .as_ref()
                .is_some_and(|(s, i)| *s == item.split && *i == item.index);
            let (begin_split, key_split, index) =
                (item.split.clone(), item.split.clone(), item.index);
            let line = match item.axis {
                Axis::X => div()
                    .absolute()
                    .top_0()
                    .bottom_0()
                    .left(px((item.size / 2.0 - 1.5) as f32))
                    .w(px(3.0)),
                Axis::Y => div()
                    .absolute()
                    .left_0()
                    .right_0()
                    .top(px((item.size / 2.0 - 1.5) as f32))
                    .h(px(3.0)),
            };
            let el = place(div().id(id).class("group"), item.rect)
                .when(item.axis == Axis::X, Styled::cursor_col_resize)
                .when(item.axis == Axis::Y, Styled::cursor_row_resize)
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, ev: &MouseDownEvent, _, cx| {
                        this.divider_begin(begin_split.clone(), index, ev.position, ev.click_count, cx);
                        cx.stop_propagation();
                    }),
                )
                .on_key_down(cx.listener(move |this, ev: &KeyDownEvent, window, cx| {
                    if this.divider_key(&key_split, index, ev, cx) {
                        window.refresh();
                        cx.stop_propagation();
                    }
                }))
                .child(
                    line.pointer_events_none()
                        .rounded(px(2.0))
                        .bg(accent)
                        .class(if is_active || focused {
                            "opacity-100"
                        } else {
                            "opacity-0 group-hover:opacity-80"
                        }),
                );
            layers.push((Z_DIVIDER, el.into_any_element()));
        }
    }
}
