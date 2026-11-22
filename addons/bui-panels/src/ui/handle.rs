//! Handles a component holds: [`WorkspaceRef`] (from [`use_workspace`]) drives a workspace from
//! event handlers; [`ViewRef`] (from [`use_view`], inside a view's content) is the view's own.

use std::rc::Rc;

use blenderui::prelude::{App, Context, Entity, Signal, WeakEntity, try_use_context, use_signal};
use serde::Serialize;
use serde_json::Value;

use crate::{
    features::Titlebar,
    keymap::Command,
    model::{
        document::{DockTarget, empty_document},
        types::{Framing, Id, LayoutDocument, Placement, Rect},
    },
    nav::FrameTarget,
    workspace::{Badge, Guard, OpenOptions, Reuse, ViewInfo, Workspace},
};

/// A workspace, from event handlers: `ws.float("console", cx)`, `ws.open(..)`. Made by
/// [`use_workspace`] and bound by the `<Workspace handle={ws}>` it is given to.
#[derive(Clone, Copy)]
pub struct WorkspaceRef(pub(crate) Signal<Option<WeakEntity<Workspace>>>);

impl WorkspaceRef {
    pub(crate) fn engine(self, cx: &App) -> Option<Entity<Workspace>> {
        self.0.with(|weak| *weak)?.upgrade_in(cx)
    }

    fn update<R>(
        self,
        cx: &mut App,
        f: impl FnOnce(&mut Workspace, &mut Context<Workspace>) -> R,
    ) -> Option<R> {
        let engine = self.engine(cx)?;
        Some(engine.update(cx, f))
    }

    fn read<R>(self, cx: &App, f: impl FnOnce(&Workspace) -> R) -> Option<R> {
        Some(f(self.engine(cx)?.read(cx)))
    }

    pub fn open<P: Serialize>(self, ty: impl Into<String>, params: P, cx: &mut App) -> Opening<'_> {
        Opening {
            ws: self,
            ty: ty.into(),
            options: OpenOptions {
                params: serde_json::to_value(params).ok().filter(|v| !v.is_null()),
                ..Default::default()
            },
            cx,
            done: false,
        }
    }

    pub fn close(self, id: impl Into<Id>, cx: &mut App) -> bool {
        let id = id.into();
        self.update(cx, |ws, cx| ws.close(&id, false, cx)).unwrap_or(false)
    }

    pub fn close_forced(self, id: impl Into<Id>, cx: &mut App) -> bool {
        let id = id.into();
        self.update(cx, |ws, cx| ws.close(&id, true, cx)).unwrap_or(false)
    }

    pub fn reopen_closed(self, cx: &mut App) {
        self.update(cx, Workspace::reopen_closed);
    }

    pub fn select(self, id: impl Into<Id>, cx: &mut App) {
        let id = id.into();
        self.update(cx, |ws, cx| ws.select(&id, cx));
    }

    pub fn focus(self, id: impl Into<Id>, cx: &mut App) {
        let id = id.into();
        self.update(cx, |ws, cx| ws.focus(&id, cx));
    }

    pub fn hide(self, id: impl Into<Id>, cx: &mut App) {
        let id = id.into();
        self.update(cx, |ws, cx| ws.hide(&id, None, cx));
    }

    pub fn hide_toward(self, id: impl Into<Id>, toward: Rect, cx: &mut App) {
        let id = id.into();
        self.update(cx, |ws, cx| ws.hide(&id, Some(toward), cx));
    }

    pub fn restore(self, panel_id: impl Into<Id>, cx: &mut App) {
        let id = panel_id.into();
        self.update(cx, |ws, cx| ws.restore(&id, None, cx));
    }

    pub fn restore_from(self, panel_id: impl Into<Id>, from: Rect, cx: &mut App) {
        let id = panel_id.into();
        self.update(cx, |ws, cx| ws.restore(&id, Some(from), cx));
    }

    pub fn float(self, id: impl Into<Id>, cx: &mut App) {
        let id = id.into();
        self.update(cx, |ws, cx| ws.float(&id, None, cx));
    }

    pub fn float_at(self, id: impl Into<Id>, rect: Rect, cx: &mut App) {
        let id = id.into();
        self.update(cx, |ws, cx| ws.float(&id, Some(rect), cx));
    }

    pub fn dock(self, id: impl Into<Id>, target: DockTarget, cx: &mut App) {
        let id = id.into();
        self.update(cx, |ws, cx| ws.dock(&id, target, cx));
    }

    pub fn toggle_dock(self, id: impl Into<Id>, cx: &mut App) {
        let id = id.into();
        self.update(cx, |ws, cx| ws.toggle_dock(&id, cx));
    }

    pub fn frame(self, target: FrameTarget, cx: &mut App) {
        self.update(cx, |ws, cx| ws.frame(target, cx));
    }

    pub fn toggle_maximize(self, id: impl Into<Id>, cx: &mut App) -> bool {
        let id = id.into();
        self.update(cx, |ws, cx| ws.toggle_maximize(&id, cx))
            .unwrap_or(false)
    }

    pub fn back(self, cx: &mut App) {
        self.update(cx, Workspace::navigate_back);
    }

    pub fn forward(self, cx: &mut App) {
        self.update(cx, Workspace::navigate_forward);
    }

    pub fn overview(self, cx: &mut App) {
        self.update(cx, Workspace::overview);
    }

    pub fn save_framing(self, name: impl AsRef<str>, cx: &mut App) -> Option<Framing> {
        self.update(cx, |ws, cx| ws.save_framing(name.as_ref(), cx))
            .flatten()
    }

    pub fn go_framing(self, id: impl Into<Id>, cx: &mut App) {
        let id = id.into();
        self.update(cx, |ws, cx| ws.go_framing(&id, cx));
    }

    pub fn remove_framing(self, id: impl Into<Id>, cx: &mut App) {
        let id = id.into();
        self.update(cx, |ws, cx| ws.remove_framing(&id, cx));
    }

    #[must_use]
    pub fn framings(self, cx: &App) -> Vec<Framing> {
        self.read(cx, |ws| ws.framings().to_vec()).unwrap_or_default()
    }

    pub fn run(self, command: Command, cx: &mut App) {
        self.update(cx, |ws, cx| ws.run(command, cx));
    }

    #[must_use]
    pub fn views(self, cx: &App) -> Vec<ViewInfo> {
        self.read(cx, Workspace::views).unwrap_or_default()
    }

    #[must_use]
    pub fn document(self, cx: &App) -> LayoutDocument {
        self.read(cx, Workspace::document)
            .unwrap_or_else(empty_document)
    }

    pub fn set_document(self, document: LayoutDocument, animate: bool, cx: &mut App) {
        self.update(cx, |ws, cx| ws.set_document(document, animate, cx));
    }

    pub fn reset(self, cx: &mut App) {
        self.update(cx, Workspace::reset);
    }

    pub fn set_title(self, id: impl Into<Id>, title: impl Into<String>, cx: &mut App) {
        let (id, title) = (id.into(), title.into());
        self.update(cx, |ws, cx| ws.set_title(&id, title, cx));
    }

    pub fn set_params<P: Serialize>(self, id: impl Into<Id>, patch: P, cx: &mut App) {
        let id = id.into();
        let patch = serde_json::to_value(patch).unwrap_or(Value::Null);
        self.update(cx, |ws, cx| ws.set_params(&id, patch, cx));
    }

    pub fn set_badge(self, id: impl Into<Id>, badge: impl Into<Option<Badge>>, cx: &mut App) {
        let (id, badge) = (id.into(), badge.into());
        self.update(cx, |ws, cx| ws.set_badge(&id, badge, cx));
    }

    #[must_use]
    pub fn focused_view(self, cx: &App) -> Option<Id> {
        self.read(cx, |ws| ws.focused_view().cloned()).flatten()
    }

    #[must_use]
    pub fn framed(self, cx: &App) -> Option<Id> {
        self.read(cx, |ws| ws.framed().cloned()).flatten()
    }

    #[must_use]
    pub fn camera(self, cx: &App) -> Rect {
        self.read(cx, Workspace::camera).unwrap_or(Rect::UNIT)
    }

    #[must_use]
    pub fn view_scale(self, id: impl Into<Id>, cx: &App) -> f64 {
        let id = id.into();
        self.read(cx, |ws| ws.view_scale(&id)).unwrap_or(1.0)
    }

    #[must_use]
    pub fn titlebar(self, id: impl Into<Id>, cx: &App) -> Titlebar {
        let id = id.into();
        self.read(cx, |ws| ws.titlebar(&id)).unwrap_or_default()
    }
}

/// `ws.open(ty, params, cx)`: options, then it opens when dropped (or at [`Opening::info`]).
pub struct Opening<'a> {
    ws: WorkspaceRef,
    ty: String,
    options: OpenOptions,
    cx: &'a mut App,
    done: bool,
}

#[allow(clippy::return_self_not_must_use, clippy::must_use_candidate)]
impl Opening<'_> {
    pub fn placement(mut self, placement: Placement) -> Self {
        self.options.placement = Some(placement);
        self
    }

    pub const fn reuse(mut self, reuse: Reuse) -> Self {
        self.options.reuse = reuse;
        self
    }

    #[allow(clippy::should_implement_trait, clippy::wrong_self_convention)]
    pub const fn from(mut self, rect: Rect) -> Self {
        self.options.from = Some(rect);
        self
    }

    pub const fn focus(mut self, focus: bool) -> Self {
        self.options.focus = Some(focus);
        self
    }

    pub fn id(mut self, id: impl Into<Id>) -> Self {
        self.options.id = Some(id.into());
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.options.title = Some(title.into());
        self
    }

    pub fn info(mut self) -> Option<ViewInfo> {
        self.done = true;
        self.run()
    }

    fn run(&mut self) -> Option<ViewInfo> {
        let ty = std::mem::take(&mut self.ty);
        let options = std::mem::take(&mut self.options);
        self.ws
            .update(self.cx, |ws, cx| ws.open(&ty, options, cx))
            .flatten()
    }
}

impl Drop for Opening<'_> {
    fn drop(&mut self) {
        if !self.done {
            self.run();
        }
    }
}

/// A change a view asked for while rendering or in an effect (no app there): the workspace
/// makes it when it next renders.
pub(crate) enum ViewOp {
    Close,
    Hide,
    Focus,
    Title(String),
    Params(Value),
    Badge(Option<Badge>),
}

/// What a [`ViewRef`] holds: its view, its close guard, the changes it asked for, and what the
/// workspace last told it (its title bar, scale and params).
pub(crate) struct ViewTarget {
    pub(crate) id: Id,
    pub(crate) guard: Option<Guard>,
    pub(crate) ops: Vec<ViewOp>,
    pub(crate) titlebar: Titlebar,
    pub(crate) scale: f64,
    pub(crate) params: Value,
}

/// The view whose content is rendering (from [`use_view`]). It needs no `cx`: reads answer from
/// the workspace's last render, and changes are made when the workspace next renders (so they
/// work in render code and in `use_effect`, which blenderui runs without an app).
#[derive(Clone, Copy)]
pub struct ViewRef(pub(crate) Signal<ViewTarget>);

impl ViewRef {
    pub(crate) fn new(id: Id) -> Self {
        Self(Signal::new(ViewTarget {
            id,
            guard: None,
            ops: Vec::new(),
            titlebar: Titlebar::default(),
            scale: 1.0,
            params: Value::Null,
        }))
    }

    fn push(self, op: ViewOp) {
        self.0.update(|target| target.ops.push(op));
    }

    #[must_use]
    pub fn id(self) -> Id {
        self.0.with(|target| target.id.clone())
    }

    /// Closes the view (unless a close guard says no).
    pub fn close(self) {
        self.push(ViewOp::Close);
    }

    pub fn hide(self) {
        self.push(ViewOp::Hide);
    }

    pub fn focus(self) {
        self.push(ViewOp::Focus);
    }

    #[must_use]
    pub fn titlebar(self) -> Titlebar {
        self.0.with(|target| target.titlebar)
    }

    #[must_use]
    pub fn scale(self) -> f64 {
        self.0.with(|target| target.scale)
    }

    /// Asked before the user closes the view: false keeps it open. Call it every render.
    pub fn guard_close(self, guard: impl Fn(&App) -> bool + 'static) {
        self.0.update_untracked(|target| target.guard = Some(Rc::new(guard)));
    }

    pub fn set_title(self, title: impl Into<String>) {
        self.push(ViewOp::Title(title.into()));
    }

    pub fn set_params<P: Serialize>(self, patch: P) {
        self.push(ViewOp::Params(serde_json::to_value(patch).unwrap_or(Value::Null)));
    }

    pub fn set_badge(self, badge: impl Into<Option<Badge>>) {
        self.push(ViewOp::Badge(badge.into()));
    }

    #[must_use]
    pub fn params(self) -> Value {
        self.0.with(|target| target.params.clone())
    }
}

/// A [`WorkspaceRef`] for this component: the one of the `<Workspace>` it renders in, else its
/// own, to give to the `<Workspace handle={..}>` it renders.
#[track_caller]
#[must_use]
pub fn use_workspace() -> WorkspaceRef {
    let own = WorkspaceRef(use_signal(None));
    try_use_context::<WorkspaceRef>().unwrap_or(own)
}

/// The view whose content (`<View content>` or a `<Template>`) is rendering.
#[must_use]
pub fn use_view() -> ViewRef {
    try_use_context::<ViewRef>()
        .expect("use_view() works inside a <View>'s `content` or a <Template>'s; render it from one")
}
