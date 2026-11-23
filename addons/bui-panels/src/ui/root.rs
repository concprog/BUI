//! `<Workspace>`: a workspace declared in markup. Its children declare the layout and the views;
//! it makes the engine ([`crate::Workspace`]) on its first render, then keeps it in step with the
//! markup: views added or removed, titles, badges, rules and content.

use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    rc::Rc,
};

use blenderui::{
    hooks::Complete,
    prelude::{
        AnyElement, App, Entity, IntoElement, Pixels, SharedString, Size, Window, provide_context,
        use_signal,
    },
};

use super::{
    content::Blank,
    handle::WorkspaceRef,
    layout::{CloseFn, LayoutNode, TemplateNode, ViewNode, layout_children},
    scope::with_host,
};
use crate::{
    features::Detail,
    keymap::{GestureKeys, Keymap},
    menu::{MenuEntry, PanelMenu, PanelMenuContext},
    model::{
        builder::{FloatSpec, LayoutSpec, PanelSpec, SplitSpec, StageSpec, ViewSpec, create_document},
        document::locate_panel,
        types::{Axis, FloatingLayer, Id, LayoutDocument, Placement, Rect},
    },
    workspace::{
        Badge, Navigation, OpenOptions, Persist, ThemeMode, ViewBuilder, ViewInfo,
        ViewType, Workspace as Engine, WorkspaceEvent, WorkspaceOptions,
    },
};

/// The type of a declared view (`<View id="files">` is of type `zopra-view:files`).
const VIEW_TYPE: &str = "zopra-view:";

fn view_type(id: &str) -> String {
    format!("{VIEW_TYPE}{id}")
}

type DocFn = Box<dyn Fn(&LayoutDocument, &mut Window, &mut App)>;
type InfoFn = Box<dyn Fn(&ViewInfo, &mut Window, &mut App)>;
type IdFn = Box<dyn Fn(Option<&Id>, &mut Window, &mut App)>;
#[allow(clippy::option_option)]
type Applied = (Option<SharedString>, Option<Option<Badge>>);

#[derive(Default)]
struct Callbacks {
    change: Option<DocFn>,
    open: Option<InfoFn>,
    close: Option<InfoFn>,
    focus: Option<IdFn>,
    navigate: Option<IdFn>,
    views: HashMap<Id, CloseFn>,
}

impl Callbacks {
    fn deliver(&self, event: &WorkspaceEvent, window: &mut Window, cx: &mut App) {
        match event {
            WorkspaceEvent::Change(doc) => {
                if let Some(f) = &self.change {
                    f(doc, window, cx);
                }
            }
            WorkspaceEvent::Open(info) => {
                if let Some(f) = &self.open {
                    f(info, window, cx);
                }
            }
            WorkspaceEvent::Close(info) => {
                if let Some(f) = self.views.get(&info.id) {
                    f(window, cx);
                }
                if let Some(f) = &self.close {
                    f(info, window, cx);
                }
            }
            WorkspaceEvent::Focus(id) => {
                if let Some(f) = &self.focus {
                    f(id.as_ref(), window, cx);
                }
            }
            WorkspaceEvent::Navigate(id) => {
                if let Some(f) = &self.navigate {
                    f(id.as_ref(), window, cx);
                }
            }
        }
    }
}

struct Declared {
    node: ViewNode,
    panel: Id,
    index: usize,
}

struct FloatDecl {
    rect: Rect,
    layer: Option<FloatingLayer>,
    panel: PanelSpec,
}

#[derive(Default)]
struct Declaration {
    roots: Vec<LayoutSpec>,
    floats: Vec<FloatDecl>,
    views: Vec<Declared>,
    templates: Vec<TemplateNode>,
}

impl Declaration {
    fn new(nodes: Vec<LayoutNode>) -> Self {
        let mut decl = Self::default();
        for (i, node) in nodes.into_iter().enumerate() {
            if let Some(spec) = decl.walk(node, &format!("-{i}")) {
                decl.roots.push(spec);
            }
        }
        decl
    }

    fn walk(&mut self, node: LayoutNode, path: &str) -> Option<LayoutSpec> {
        match node {
            LayoutNode::Split {
                id,
                axis,
                sizes,
                children,
            } => {
                let children = children
                    .into_iter()
                    .enumerate()
                    .filter_map(|(i, child)| self.walk(child, &format!("{path}-{i}")))
                    .collect();
                Some(LayoutSpec::Split(SplitSpec {
                    id: Some(id.map_or_else(|| format!("split{path}"), |id| id.to_string())),
                    axis,
                    children,
                    weights: sizes,
                }))
            }
            LayoutNode::Stage { id, child } => Some(LayoutSpec::Stage(StageSpec {
                id: id.map(|id| id.to_string()),
                child: child
                    .and_then(|child| self.walk(*child, &format!("{path}-0")))
                    .map(Box::new),
            })),
            LayoutNode::Panel { id, views } => {
                Some(LayoutSpec::Panel(self.panel(id, views, &format!("panel{path}"))))
            }
            LayoutNode::Float {
                rect,
                layer,
                panels,
            } => {
                for (i, child) in panels.into_iter().enumerate() {
                    let fallback = format!("float{path}-{i}");
                    let panel = match child {
                        LayoutNode::Panel { id, views } => self.panel(id, views, &fallback),
                        other => match self.walk(other, &format!("{path}-{i}")) {
                            Some(LayoutSpec::Panel(panel)) => panel,
                            _ => continue,
                        },
                    };
                    self.floats.push(FloatDecl { rect, layer, panel });
                }
                None
            }
            LayoutNode::Template(template) => {
                self.templates.push(template);
                None
            }
        }
    }

    fn panel(&mut self, id: Option<Id>, views: Vec<ViewNode>, fallback: &str) -> PanelSpec {
        let id: Id = id.unwrap_or_else(|| fallback.into());
        let specs = views
            .into_iter()
            .enumerate()
            .map(|(index, node)| {
                let spec = ViewSpec {
                    ty: view_type(&node.id),
                    id: Some(node.id.to_string()),
                    params: None,
                    title: node.title.as_ref().map(ToString::to_string),
                };
                self.views.push(Declared {
                    node,
                    panel: id.clone(),
                    index,
                });
                spec
            })
            .collect();
        PanelSpec {
            id: Some(id.to_string()),
            views: specs,
            selected: None,
        }
    }

    fn document(&self, viewport: Size<Pixels>) -> LayoutDocument {
        let root = match self.roots.as_slice() {
            [] => None,
            [root] => Some(root.clone()),
            roots => Some(LayoutSpec::Split(SplitSpec {
                id: Some("split".into()),
                axis: Axis::X,
                children: roots.to_vec(),
                weights: None,
            })),
        };
        let (w, h) = (
            f64::from(viewport.width.0).max(1.0),
            f64::from(viewport.height.0).max(1.0),
        );
        let floats = self
            .floats
            .iter()
            .map(|f| FloatSpec {
                panel: f.panel.clone(),
                rect: Rect {
                    x: f.rect.x / w,
                    y: f.rect.y / h,
                    w: f.rect.w / w,
                    h: f.rect.h / h,
                },
                layer: f.layer,
            })
            .collect();
        create_document(root.as_ref(), floats, None).unwrap_or_else(|error| panic!("{error}"))
    }

    fn ids(&self) -> HashSet<Id> {
        self.views.iter().map(|d| d.node.id.clone()).collect()
    }
}

fn blank() -> ViewBuilder {
    Rc::new(|_, _, cx| cx.new(|_| Blank).into())
}

fn declared_type(node: &ViewNode) -> ViewType {
    let title = node
        .title
        .as_ref()
        .map_or_else(|| node.id.to_string(), ToString::to_string);
    ViewType {
        title: Rc::new(move |_| title.clone()),
        icon: node.icon,
        rules: node.rules.clone(),
        build: blank(),
        menu: node.menu.clone(),
        accessory: None,
    }
}

/// What a `<Workspace>` keeps between renders (in a hook): its engine and what it last applied.
#[derive(Default)]
struct Binding {
    engine: Option<Entity<Engine>>,
    declared: HashSet<Id>,
    types: HashSet<String>,
    last_layout: Option<LayoutDocument>,
    applied: HashMap<Id, Applied>,
}

impl Binding {
    fn mount(
        &mut self,
        decl: &Declaration,
        mut options: WorkspaceOptions,
        layout: Option<&LayoutDocument>,
        window: &Window,
        cx: &mut App,
    ) -> Entity<Engine> {
        options.default_layout = Some(decl.document(window.viewport_size()));
        options.document = layout.cloned();
        let mut types = HashMap::new();
        for declared in &decl.views {
            let ty = view_type(&declared.node.id);
            types.insert(ty.clone(), declared_type(&declared.node));
            self.types.insert(ty);
        }
        for template in &decl.templates {
            types.insert(template.name.clone(), ViewType::new(template.name.clone(), |_, _, cx| cx.new(|_| Blank).into()));
            self.types.insert(template.name.clone());
        }
        let engine = cx.new(|cx| Engine::blank(options, types, cx));
        let restored = engine.update(cx, |engine, cx| {
            engine.host_events();
            engine.start(cx)
        });
        let declared = decl.ids();
        if restored {
            engine.update(cx, |engine, cx| {
                let stale: Vec<Id> = engine
                    .doc
                    .views
                    .iter()
                    .filter(|(id, record)| record.ty.starts_with(VIEW_TYPE) && !declared.contains(*id))
                    .map(|(id, _)| id.clone())
                    .collect();
                for id in stale {
                    engine.close(&id, true, cx);
                }
            });
            self.declared = HashSet::new();
        } else {
            self.declared = declared;
        }
        self.last_layout = layout.cloned();
        self.engine = Some(engine);
        engine
    }

    /// Hands the engine's events since the last call to the callbacks.
    fn deliver(&mut self, engine: Entity<Engine>, callbacks: &Callbacks, window: &mut Window, cx: &mut App) {
        let events = engine.update(cx, |engine, _| engine.take_hosted_events());
        for event in &events {
            if let WorkspaceEvent::Change(doc) = event {
                self.last_layout = Some(doc.clone());
            }
            callbacks.deliver(event, window, cx);
        }
    }

    #[allow(clippy::too_many_arguments, clippy::too_many_lines)]
    fn render(
        &mut self,
        ws: WorkspaceRef,
        nodes: Vec<LayoutNode>,
        options: WorkspaceOptions,
        layout: Option<LayoutDocument>,
        slots: [Option<AnyElement>; 4],
        mut callbacks: Callbacks,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let decl = Declaration::new(nodes);
        let mounting = self.engine.is_none();
        let engine = match self.engine {
            Some(engine) => engine,
            None => self.mount(&decl, options.clone(), layout.as_ref(), window, cx),
        };
        if ws.engine(cx).is_none() {
            ws.0.update_untracked(|slot| *slot = Some(engine.downgrade()));
        }
        for declared in &decl.views {
            if let Some(f) = &declared.node.on_close {
                callbacks.views.insert(declared.node.id.clone(), f.clone());
            }
        }
        self.deliver(engine, &callbacks, window, cx);

        let current = decl.ids();
        let removed: Vec<Id> = self.declared.difference(&current).cloned().collect();
        let added: HashSet<Id> = current.difference(&self.declared).cloned().collect();
        let wanted: HashSet<String> = current
            .iter()
            .map(|id| view_type(id))
            .chain(decl.templates.iter().map(|t| t.name.clone()))
            .collect();
        let stale_types: Vec<String> = self.types.difference(&wanted).cloned().collect();
        let layout = layout.filter(|doc| self.last_layout.as_ref() != Some(doc));
        if let Some(doc) = &layout {
            self.last_layout = Some(doc.clone());
        }
        let mut applied = std::mem::take(&mut self.applied);
        let known = &self.types;

        let element = engine.update(cx, |engine, cx| {
            engine.set_options(options);
            for id in &removed {
                engine.close(id, true, cx);
            }
            for ty in &stale_types {
                engine.types.remove(ty);
                engine.hosted.templates.remove(ty);
            }
            for template in decl.templates {
                if !known.contains(&template.name) {
                    let ty = ViewType::new(template.name.clone(), |_, _, cx| cx.new(|_| Blank).into());
                    engine.types.insert(template.name.clone(), ty);
                }
                engine.hosted.templates.insert(template.name, template.render);
            }
            engine.hosted.slots = slots;
            engine.hosted.contents.retain(|id, _| current.contains(id));
            for Declared { node, panel, index } in decl.views {
                let ty = view_type(&node.id);
                match engine.types.get_mut(&ty) {
                    Some(t) => {
                        t.icon = node.icon;
                        t.rules = node.rules.clone();
                        t.menu.clone_from(&node.menu);
                    }
                    None => {
                        engine.types.insert(ty.clone(), declared_type(&node));
                    }
                }
                if added.contains(&node.id) && !engine.doc.views.contains_key(&node.id) {
                    let placement = node.rules.placement.clone().or_else(|| {
                        locate_panel(&engine.doc, &panel).map(|_| Placement::Into {
                            panel: panel.clone(),
                            index: Some(index),
                        })
                    });
                    engine.open(
                        &ty,
                        OpenOptions {
                            id: Some(node.id.clone()),
                            title: node.title.as_ref().map(ToString::to_string),
                            placement,
                            from: node.open_from,
                            focus: Some(node.focus_on_open && !mounting),
                            ..Default::default()
                        },
                        cx,
                    );
                }
                let previous = applied.get(&node.id);
                if let Some(title) = &node.title
                    && previous.is_none_or(|(applied, _)| applied.as_ref() != Some(title))
                    && engine
                        .doc
                        .views
                        .get(&node.id)
                        .is_some_and(|record| record.title.as_deref() != Some(title.as_ref()))
                {
                    engine.set_title(&node.id, title.to_string(), cx);
                }
                if let Some(badge) = &node.badge
                    && previous.is_none_or(|(_, applied)| applied.as_ref() != Some(badge))
                    && engine.badges.get(&node.id) != badge.as_ref()
                {
                    engine.set_badge(&node.id, badge.clone(), cx);
                }
                applied.insert(node.id.clone(), (node.title, node.badge));
                match node.can_close {
                    Some(guard) => {
                        engine.prop_guards.insert(node.id.clone(), guard);
                    }
                    None => {
                        engine.prop_guards.remove(&node.id);
                    }
                }
                match node.content {
                    Some(content) => {
                        engine.hosted.contents.insert(node.id.clone(), content);
                    }
                    None => {
                        engine.hosted.contents.remove(&node.id);
                    }
                }
                if let Some(el) = node.inline {
                    engine.hosted.inline.insert(node.id.clone(), el);
                }
                if let Some(el) = node.accessory {
                    engine.hosted.accessories.insert(node.id, el);
                }
            }
            if let Some(doc) = layout {
                engine.set_document(doc, true, cx);
            }
            engine.element(window, cx)
        });
        self.deliver(engine, &callbacks, window, cx);
        applied.retain(|id, _| current.contains(id));
        self.applied = applied;
        self.declared = current;
        self.types = wanted;
        element
    }
}

/// `<Workspace ..>`: a workspace from markup (see the module docs); render it inside
/// [`super::scope`]. The props are [`crate::WorkspaceOptions`]' and the event callbacks.
#[derive(Default)]
pub struct Workspace {
    handle: Option<WorkspaceRef>,
    layout: Option<LayoutDocument>,
    options: WorkspaceOptions,
    detail: Option<Detail>,
    no_detail: bool,
    floating: Option<FloatingLayer>,
    no_floating: bool,
    no_panel_menu: bool,
    persist_path: Option<PathBuf>,
    persist_version: Option<String>,
    slots: [Option<AnyElement>; 4],
    callbacks: Callbacks,
    children: Vec<AnyElement>,
}

impl Complete for Workspace {
    type Props = Self;
    fn finish(self) -> Self {
        self
    }
}

#[allow(clippy::return_self_not_must_use, clippy::must_use_candidate)]
impl Workspace {
    #[doc(hidden)]
    pub fn __builder() -> Self {
        Self::default()
    }
    /// The [`WorkspaceRef`] from [`super::use_workspace`] that drives it.
    pub const fn handle(mut self, handle: WorkspaceRef) -> Self {
        self.handle = Some(handle);
        self
    }
    /// A layout to show (a change animates to it); without one, the markup's or the persisted.
    pub fn layout(mut self, layout: impl Into<Option<LayoutDocument>>) -> Self {
        self.layout = layout.into();
        self
    }
    /// Keep the layout in this JSON file.
    pub fn persist(mut self, path: impl Into<PathBuf>) -> Self {
        self.persist_path = Some(path.into());
        self
    }
    pub fn persist_version(mut self, version: impl Into<String>) -> Self {
        self.persist_version = Some(version.into());
        self
    }
    pub const fn navigation(mut self, navigation: Navigation) -> Self {
        self.options.navigation = navigation;
        self
    }
    pub const fn theme(mut self, theme: ThemeMode) -> Self {
        self.options.theme = theme;
        self
    }
    pub const fn floating(mut self, layer: FloatingLayer) -> Self {
        self.floating = Some(layer);
        self
    }
    pub const fn no_floating(mut self, off: bool) -> Self {
        self.no_floating = off;
        self
    }
    pub const fn keep_pushed(mut self, keep: bool) -> Self {
        self.options.keep_pushed = keep;
        self
    }
    pub const fn reduced_motion(mut self, reduced: bool) -> Self {
        self.options.reduced_motion = reduced;
        self
    }
    pub const fn detail(mut self, detail: Detail) -> Self {
        self.detail = Some(detail);
        self
    }
    pub const fn no_detail(mut self, off: bool) -> Self {
        self.no_detail = off;
        self
    }
    pub const fn tab_fill(mut self, fill: bool) -> Self {
        self.options.tabs.fill = fill;
        self
    }
    pub const fn tab_inset(mut self, inset: f32) -> Self {
        self.options.tabs.inset = inset;
        self
    }
    pub const fn tab_undo_seconds(mut self, seconds: f64) -> Self {
        self.options.tabs.undo_seconds = seconds;
        self
    }
    pub fn keymap(mut self, keymap: Keymap) -> Self {
        self.options.keymap = keymap;
        self
    }
    pub fn gesture_keys(mut self, keys: GestureKeys) -> Self {
        self.options.gesture_keys = keys;
        self
    }
    pub fn panel_menu(
        mut self,
        menu: impl Fn(Vec<MenuEntry>, &PanelMenuContext) -> Vec<MenuEntry> + 'static,
    ) -> Self {
        self.options.panel_menu = PanelMenu::Custom(Rc::new(menu));
        self
    }
    pub const fn no_panel_menu(mut self, off: bool) -> Self {
        self.no_panel_menu = off;
        self
    }
    /// Where a hidden panel flies to (a dock icon's rect), by panel id.
    pub fn hide_toward(mut self, f: impl Fn(&Id) -> Option<Rect> + 'static) -> Self {
        self.options.hide_toward = Some(Rc::new(f));
        self
    }
    /// Shown behind the stage.
    pub fn backdrop(mut self, el: impl IntoElement) -> Self {
        self.slots[0] = Some(el.into_any_element());
        self
    }
    /// Shown in an empty stage.
    pub fn stage_empty(mut self, el: impl IntoElement) -> Self {
        self.slots[1] = Some(el.into_any_element());
        self
    }
    /// Shown when nothing is open.
    pub fn empty(mut self, el: impl IntoElement) -> Self {
        self.slots[2] = Some(el.into_any_element());
        self
    }
    /// Drawn over everything (pointer events pass through its empty parts).
    pub fn chrome(mut self, el: impl IntoElement) -> Self {
        self.slots[3] = Some(el.into_any_element());
        self
    }
    pub fn on_change(mut self, f: impl Fn(&LayoutDocument, &mut Window, &mut App) + 'static) -> Self {
        self.callbacks.change = Some(Box::new(f));
        self
    }
    pub fn on_open(mut self, f: impl Fn(&ViewInfo, &mut Window, &mut App) + 'static) -> Self {
        self.callbacks.open = Some(Box::new(f));
        self
    }
    pub fn on_close(mut self, f: impl Fn(&ViewInfo, &mut Window, &mut App) + 'static) -> Self {
        self.callbacks.close = Some(Box::new(f));
        self
    }
    pub fn on_focus(mut self, f: impl Fn(Option<&Id>, &mut Window, &mut App) + 'static) -> Self {
        self.callbacks.focus = Some(Box::new(f));
        self
    }
    pub fn on_navigate(mut self, f: impl Fn(Option<&Id>, &mut Window, &mut App) + 'static) -> Self {
        self.callbacks.navigate = Some(Box::new(f));
        self
    }
    pub fn children(mut self, children: Vec<AnyElement>) -> Self {
        self.children = children;
        self
    }
}

impl IntoElement for Workspace {
    fn into_any_element(self) -> AnyElement {
        let Self {
            handle,
            layout,
            mut options,
            detail,
            no_detail,
            floating,
            no_floating,
            no_panel_menu,
            persist_path,
            persist_version,
            slots,
            callbacks,
            children,
        } = self;
        let nodes = layout_children(&children, "Workspace");
        let binding = use_signal(None::<Binding>);
        let own = WorkspaceRef(use_signal(None));
        let ws = handle.unwrap_or(own);
        options.floating = (!no_floating).then(|| floating.unwrap_or(FloatingLayer::Overlay));
        options.detail = (!no_detail).then(|| detail.unwrap_or_default());
        if no_panel_menu {
            options.panel_menu = PanelMenu::Off;
        }
        options.persist = persist_path.map(|path| Persist {
            path,
            version: persist_version,
        });
        blenderui::hooks::component(|| {
            provide_context(ws);
            let mut state = binding.update_untracked(Option::take).unwrap_or_default();
            let element = with_host(|window, cx| {
                state.render(ws, nodes, options, layout, slots, callbacks, window, cx)
            });
            binding.update_untracked(|slot| *slot = Some(state));
            element
        })
    }
}
