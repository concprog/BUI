//! What declared views show, held by the workspace: `<View>` children (this frame's elements),
//! `<View content>` (made again each render) and `<Template>`s (made from a view's params). Their
//! content renders with its [`ViewRef`] as context, so `use_view()` finds it.

use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use blenderui::prelude::{
    AnyElement, App, Context, IntoElement, Render, Window, div, provide_context,
};
use serde_json::Value;

use super::{
    handle::{ViewOp, ViewRef},
    scope::{scope, with_host},
};
use crate::{model::types::Id, workspace::Workspace};

/// A view's content: markup made again each render (`content={move || rsx! { <Editor/> }}`).
#[derive(Clone)]
pub struct Content(Rc<dyn Fn() -> AnyElement>);

impl<F: Fn() -> E + 'static, E: IntoElement> From<F> for Content {
    fn from(f: F) -> Self {
        Self(Rc::new(move || f().into_any_element()))
    }
}

pub(crate) type TemplateFn = Rc<dyn Fn(&Value, &mut Window, &mut App) -> Option<AnyElement>>;

/// The declared views' content, held by the workspace that renders it.
#[derive(Default)]
pub(crate) struct Hosting {
    /// `<View>` children and accessories: elements of this frame, used once.
    pub(crate) inline: HashMap<Id, AnyElement>,
    pub(crate) accessories: HashMap<Id, AnyElement>,
    /// `<Workspace backdrop stage_empty empty chrome>`, by [`crate::workspace::Slot`].
    pub(crate) slots: [Option<AnyElement>; 4],
    pub(crate) contents: HashMap<Id, Content>,
    /// by view type name
    pub(crate) templates: HashMap<String, TemplateFn>,
    refs: HashMap<Id, ViewRef>,
}

impl Hosting {
    /// Forgets the views that are gone (their [`ViewRef`]s with them).
    pub(crate) fn retain(&mut self, live: &HashSet<Id>) {
        self.refs.retain(|id, view| {
            let keep = live.contains(id);
            if !keep {
                view.0.dispose();
            }
            keep
        });
    }

    /// The close guard a view's content set with `use_view().guard_close(..)`.
    pub(crate) fn allows_close(&self, view: &Id, cx: &App) -> bool {
        let guard = self
            .refs
            .get(view)
            .and_then(|r| r.0.with(|target| target.guard.clone()));
        guard.is_none_or(|g| g(cx))
    }

    pub(crate) fn clear_frame(&mut self) {
        self.inline.clear();
        self.accessories.clear();
        self.slots = Default::default();
    }
}

impl Workspace {
    /// Declared content for `view`, rendered with its [`ViewRef`] (and the window and app lent
    /// to `<Workspace>`s inside it).
    pub(crate) fn hosted_content(
        &mut self,
        view: &Id,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        if let Some(element) = self.hosted.inline.remove(view) {
            return Some(element);
        }
        let record = self.doc.views.get(view)?;
        let content = self.hosted.contents.get(view).cloned();
        let template = content
            .is_none()
            .then(|| self.hosted.templates.get(&record.ty).cloned())
            .flatten();
        if content.is_none() && template.is_none() {
            return None;
        }
        let params = record.params.as_ref().unwrap_or(&Value::Null);
        let (titlebar, scale) = (self.titlebar(view), self.view_scale(view));
        let target = if let Some(target) = self.hosted.refs.get(view) {
            *target
        } else {
            let target = ViewRef::new(view.clone());
            self.hosted.refs.insert(view.clone(), target);
            target
        };
        target.0.update_untracked(|t| {
            t.titlebar = titlebar;
            t.scale = scale;
            if t.params != *params {
                t.params.clone_from(params);
            }
        });
        Some(blenderui::hooks::component(|| {
            provide_context(target);
            scope(window, cx, || match (&content, &template) {
                (Some(content), _) => Some((content.0)()),
                (None, Some(template)) => with_host(|window, cx| template(params, window, cx)),
                (None, None) => None,
            })
        }))
    }

    pub(crate) fn hosted_accessory(&mut self, view: &Id) -> Option<AnyElement> {
        self.hosted.accessories.remove(view)
    }

    /// Makes the changes views asked for through their [`ViewRef`]s.
    pub(crate) fn apply_view_ops(&mut self, cx: &mut Context<Self>) {
        let pending: Vec<(Id, Vec<ViewOp>)> = self
            .hosted
            .refs
            .values()
            .filter_map(|r| {
                r.0.update_untracked(|t| {
                    (!t.ops.is_empty()).then(|| (t.id.clone(), std::mem::take(&mut t.ops)))
                })
            })
            .collect();
        for (id, ops) in pending {
            for op in ops {
                match op {
                    ViewOp::Close => {
                        self.close(&id, false, cx);
                    }
                    ViewOp::Hide => self.hide(&id, None, cx),
                    ViewOp::Focus => self.focus(&id, cx),
                    ViewOp::Title(title) => self.set_title(&id, title, cx),
                    ViewOp::Params(patch) => self.set_params(&id, patch, cx),
                    ViewOp::Badge(badge) => self.set_badge(&id, badge, cx),
                }
            }
        }
    }
}

/// A view type whose content is declared: nothing of its own to show.
pub(crate) struct Blank;

impl Render for Blank {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}
