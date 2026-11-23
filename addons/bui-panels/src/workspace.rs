use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    rc::Rc,
    time::{Duration, Instant},
};

use blenderui::prelude::{
    AnyElement, App, Context, Entity, Hsla, KeyDownEvent, KeyUpEvent, SharedString, UiSender,
    WeakEntity, Window,
};
use serde_json::Value;

use super::{
    drag::{DividerDrag, DragSession, FloatResize},
    features::{BarMode, Detail, Fx},
    icon::IconName,
    keymap::{self, Command, GestureKeys, Keymap, merged_keymap},
    menu::{
        KeyResult, MenuEntry, MenuItem, MenuRegion, PanelMenu, PanelMenuContext, Popup,
        builtin, tidy_menu,
    },
    model::{
        document::{
            DockTarget, Where, clamp_float, close_view, detach_view, empty_document, float_panel,
            hide_panel, insert_panel, locate_panel, panel_of_view, raise_float, remove_panel,
            restore_panel, sanitize, select_view, uid, view_ids,
        },
        tree::{Entries, Metrics, PixelMetrics, find_node, find_stage, layout_rects, panels_of},
        types::{
            Allow, Edge, FloatingLayer, GestureOwner, HiddenPanel, Id, LayoutDocument, Node,
            PanelNode, Placement, Rect, RestoreTarget, Scaling, TabbarMode, ViewRecord, ViewRules,
        },
    },
    motion::{DOCK_HIDE_MS, DOCK_RESTORE_MS, LAYOUT_MS, LayoutTween, RectSpring},
    nav::NavState,
    view::AnyView,
};

#[derive(Clone)]
pub struct ViewHandle {
    pub id: Id,
    pub(crate) workspace: WeakEntity<Workspace>,
}

impl ViewHandle {
    fn with(&self, cx: &mut App, f: impl FnOnce(&mut Workspace, &mut Context<Workspace>)) {
        self.workspace.update(cx, f);
    }
    pub fn set_title(&self, title: impl Into<String>, cx: &mut App) {
        let (id, title) = (self.id.clone(), title.into());
        self.with(cx, move |ws, cx| ws.set_title(&id, title, cx));
    }
    pub fn set_params(&self, patch: Value, cx: &mut App) {
        let id = self.id.clone();
        self.with(cx, move |ws, cx| ws.set_params(&id, patch, cx));
    }
    pub fn set_badge(&self, badge: Option<Badge>, cx: &mut App) {
        let id = self.id.clone();
        self.with(cx, move |ws, cx| ws.set_badge(&id, badge, cx));
    }
    pub fn focus(&self, cx: &mut App) {
        let id = self.id.clone();
        self.with(cx, move |ws, cx| ws.focus(&id, cx));
    }
    pub fn close(&self, cx: &mut App) {
        let id = self.id.clone();
        self.with(cx, move |ws, cx| {
            ws.close(&id, false, cx);
        });
    }
    pub fn hide(&self, cx: &mut App) {
        let id = self.id.clone();
        self.with(cx, move |ws, cx| ws.hide(&id, None, cx));
    }
    #[must_use]
    pub fn titlebar(&self, cx: &App) -> super::features::Titlebar {
        self.workspace
            .upgrade_in(cx)
            .map(|ws| ws.read(cx).titlebar(&self.id))
            .unwrap_or_default()
    }
    #[must_use]
    pub fn scale(&self, cx: &App) -> f64 {
        self.workspace
            .upgrade_in(cx)
            .map_or(1.0, |ws| ws.read(cx).view_scale(&self.id))
    }
    pub fn guard_close(&self, guard: impl Fn() -> bool + 'static, cx: &mut App) {
        let (id, guard): (Id, Guard) = (self.id.clone(), Rc::new(move |_: &App| guard()));
        self.with(cx, move |ws, _| {
            ws.guards.entry(id).or_default().push(guard);
        });
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Badge {
    Dot,
    Text(String),
}

pub type ViewBuilder = Rc<dyn Fn(&ViewHandle, &mut Window, &mut App) -> AnyView>;
pub type Accessory = Rc<dyn Fn(&ViewHandle, &mut Window, &mut App) -> AnyElement>;
/// A view type's own panel-menu entries ([`ViewType::menu`]).
pub type ViewMenu = Rc<dyn Fn(&ViewInfo) -> Vec<MenuEntry>>;
/// Where a hidden panel flies to ([`WorkspaceOptions::hide_toward`]).
pub type HideToward = Rc<dyn Fn(&Id) -> Option<Rect>>;

pub(crate) type Guard = Rc<dyn Fn(&App) -> bool>;

#[derive(Clone)]
pub struct ViewType {
    pub title: Rc<dyn Fn(&Value) -> String>,
    pub icon: Option<IconName>,
    pub rules: ViewRules,
    pub build: ViewBuilder,
    pub menu: Option<ViewMenu>,
    pub accessory: Option<Accessory>,
}

impl ViewType {
    pub fn new(
        title: impl Into<String>,
        build: impl Fn(&ViewHandle, &mut Window, &mut App) -> AnyView + 'static,
    ) -> Self {
        let title = title.into();
        Self {
            title: Rc::new(move |_| title.clone()),
            icon: None,
            rules: ViewRules::default(),
            build: Rc::new(build),
            menu: None,
            accessory: None,
        }
    }
    #[must_use]
    pub fn title_fn(mut self, f: impl Fn(&Value) -> String + 'static) -> Self {
        self.title = Rc::new(f);
        self
    }
    #[must_use]
    pub const fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }
    #[must_use]
    pub fn placement(mut self, p: Placement) -> Self {
        self.rules.placement = Some(p);
        self
    }
    #[must_use]
    pub const fn allow(mut self, allow: Allow) -> Self {
        self.rules.allow = allow;
        self
    }
    #[must_use]
    pub const fn singleton(mut self) -> Self {
        self.rules.singleton = true;
        self
    }
    #[must_use]
    pub const fn closable(mut self, closable: bool) -> Self {
        self.rules.closable = Some(closable);
        self
    }
    #[must_use]
    pub const fn tabbar(mut self, mode: TabbarMode) -> Self {
        self.rules.tabbar = mode;
        self
    }
    #[must_use]
    pub const fn min_size(mut self, width: f64, height: f64) -> Self {
        self.rules.min_size = Some((width, height));
        self
    }
    #[must_use]
    pub const fn scaling(mut self, scaling: Scaling) -> Self {
        self.rules.scaling = scaling;
        self
    }
    #[must_use]
    pub const fn gestures(mut self, owner: GestureOwner) -> Self {
        self.rules.gestures = owner;
        self
    }
    #[must_use]
    pub fn accessory(
        mut self,
        f: impl Fn(&ViewHandle, &mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        self.accessory = Some(Rc::new(f));
        self
    }
    #[must_use]
    pub fn menu(mut self, f: impl Fn(&ViewInfo) -> Vec<MenuEntry> + 'static) -> Self {
        self.menu = Some(Rc::new(f));
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewPlacement {
    Docked,
    Stage,
    Floating,
    Hidden,
}

pub(crate) struct Closed {
    pub view: Id,
    pub title: String,
    record: Option<ViewRecord>,
    panel: Option<(Id, usize)>,
    exact: Option<(u64, LayoutDocument)>,
}

const MAX_CLOSED: usize = 20;

#[derive(Clone, Debug)]
pub struct ViewInfo {
    pub id: Id,
    pub ty: String,
    pub params: Value,
    pub title: String,
    pub panel_id: Option<Id>,
    pub placement: ViewPlacement,
    pub selected: bool,
}

#[derive(Clone, Debug)]
pub enum WorkspaceEvent {
    Change(LayoutDocument),
    Open(ViewInfo),
    Close(ViewInfo),
    Focus(Option<Id>),
    Navigate(Option<Id>),
}

/// A listener of a workspace's events ([`Workspace::subscribe`], gpui's `cx.subscribe`).
pub type Subscriber = Rc<dyn Fn(Entity<Workspace>, &WorkspaceEvent, &mut App)>;

/// Who hears a workspace's events. Subscribers get them right after the change, once the
/// workspace is no longer borrowed (a job on blenderui's UI queue); the `<Workspace>` component
/// takes its own while it renders, where it has the window its callbacks get.
#[derive(Default)]
struct Events {
    subscribers: Vec<Subscriber>,
    queued: Vec<WorkspaceEvent>,
    scheduled: bool,
    hosted: Option<Vec<WorkspaceEvent>>,
}

impl Events {
    const fn listened(&self) -> bool {
        !self.subscribers.is_empty() || self.hosted.is_some()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Navigation {
    Off,
    #[default]
    Focus,
    Free,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ThemeMode {
    Light,
    Medium,
    Dark,
    Darker,
    #[default]
    System,
}

#[derive(Clone, Copy, Debug)]
pub struct TabOptions {
    pub fill: bool,
    pub inset: f32,
    pub undo_seconds: f64,
}

impl Default for TabOptions {
    fn default() -> Self {
        Self {
            fill: false,
            inset: 4.0,
            undo_seconds: 0.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Persist {
    pub path: PathBuf,
    pub version: Option<String>,
}

pub type SlotFn = Rc<dyn Fn(&mut Window, &mut App) -> AnyElement>;

#[derive(Clone, Default)]
pub struct Slots {
    pub backdrop: Option<SlotFn>,
    pub stage_empty: Option<SlotFn>,
    pub empty: Option<SlotFn>,
    pub chrome: Option<SlotFn>,
}

/// One of the [`Slots`].
#[derive(Clone, Copy)]
pub(crate) enum Slot {
    Backdrop,
    StageEmpty,
    Empty,
    Chrome,
}

#[derive(Clone)]
pub struct WorkspaceOptions {
    pub floating: Option<FloatingLayer>,
    pub navigation: Navigation,
    pub reduced_motion: bool,
    pub theme: ThemeMode,
    pub keymap: Keymap,
    pub gesture_keys: GestureKeys,
    pub tabs: TabOptions,
    pub panel_menu: PanelMenu,
    pub keep_pushed: bool,
    pub detail: Option<Detail>,
    pub persist: Option<Persist>,
    pub default_layout: Option<LayoutDocument>,
    pub document: Option<LayoutDocument>,
    pub hide_toward: Option<HideToward>,
    pub slots: Slots,
}

impl Default for WorkspaceOptions {
    fn default() -> Self {
        Self {
            floating: Some(FloatingLayer::Overlay),
            navigation: Navigation::Focus,
            reduced_motion: false,
            theme: ThemeMode::System,
            keymap: Keymap::new(),
            gesture_keys: keymap::default_gesture_keys(),
            tabs: TabOptions::default(),
            panel_menu: PanelMenu::Default,
            keep_pushed: true,
            detail: Some(Detail::default()),
            persist: None,
            default_layout: None,
            document: None,
            hide_toward: None,
            slots: Slots::default(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct OpenOptions {
    pub params: Option<Value>,
    pub id: Option<Id>,
    pub title: Option<String>,
    pub placement: Option<Placement>,
    pub reuse: Reuse,
    pub focus: Option<bool>,
    pub from: Option<Rect>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Reuse {
    #[default]
    Auto,
    None,
    Type,
    Params,
}

#[derive(Clone, Copy)]
pub struct Palette {
    pub bg: Hsla,
    pub panel: Hsla,
    pub tabbar: Hsla,
    pub tab_hover: Hsla,
    pub tab_active: Hsla,
    pub text: Hsla,
    pub text_muted: Hsla,
    pub border: Hsla,
    pub accent: Hsla,
    pub accent_contrast: Hsla,
    pub stage: Hsla,
    pub slot: Hsla,
}

fn c(hex: u32) -> Hsla {
    blenderui::prelude::rgb(hex)
}

impl Palette {
    #[must_use]
    pub fn for_theme(theme: ThemeMode, system_dark: bool) -> Self {
        let theme = match theme {
            ThemeMode::System if system_dark => ThemeMode::Dark,
            ThemeMode::System => ThemeMode::Light,
            other => other,
        };
        match theme {
            ThemeMode::Light | ThemeMode::System => Self {
                bg: c(0xdfe0e4),
                panel: c(0xffffff),
                tabbar: c(0xf1f1f4),
                tab_hover: c(0x0f1119).opacity(0.05),
                tab_active: c(0xffffff),
                text: c(0x1c1d22),
                text_muted: c(0x636670),
                border: c(0x0f1119).opacity(0.1),
                accent: c(0x3a6ff7),
                accent_contrast: c(0xffffff),
                stage: c(0xeceef1),
                slot: c(0xcdd0d6),
            },
            ThemeMode::Medium => Self {
                bg: c(0x1f1f22),
                panel: c(0x45454a),
                tabbar: c(0x38383c),
                tab_hover: c(0xffffff).opacity(0.06),
                tab_active: c(0x45454a),
                text: c(0xf1f1f3),
                text_muted: c(0xbcbcc3),
                border: c(0x000000).opacity(0.28),
                accent: c(0x4d8bff),
                accent_contrast: c(0xffffff),
                stage: c(0x2a2a2e),
                slot: c(0x2c2c30),
            },
            ThemeMode::Dark => Self {
                bg: c(0x0c0d0f),
                panel: c(0x1a1b1f),
                tabbar: c(0x141518),
                tab_hover: c(0xffffff).opacity(0.05),
                tab_active: c(0x1a1b1f),
                text: c(0xe7e8eb),
                text_muted: c(0x9699a3),
                border: c(0xffffff).opacity(0.07),
                accent: c(0x5b8cff),
                accent_contrast: c(0xffffff),
                stage: c(0x111215),
                slot: c(0x1f2024),
            },
            ThemeMode::Darker => Self {
                bg: c(0x050506),
                panel: c(0x111215),
                tabbar: c(0x0b0c0e),
                tab_hover: c(0xffffff).opacity(0.045),
                tab_active: c(0x111215),
                text: c(0xd9dbe0),
                text_muted: c(0x8a8e98),
                border: c(0xffffff).opacity(0.06),
                accent: c(0x5b8cff),
                accent_contrast: c(0xffffff),
                stage: c(0x09090b),
                slot: c(0x15161a),
            },
        }
    }
}

pub(crate) const GAP: f64 = 6.0;
pub(crate) const TABBAR_HEIGHT: f64 = 34.0;
const PANEL_MIN_WIDTH: f64 = 80.0;
const DEFAULT_FLOAT_SIZE: (f64, f64) = (560.0, 400.0);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Region {
    Stage,
    Side,
    Floating,
}

pub(crate) struct ViewInstance {
    pub content: AnyView,
}

pub(crate) struct Leaving {
    pub panel: PanelNode,
    pub from: Rect,
    pub to: Rect,
    pub start: f64,
    pub duration: f64,
}

pub(crate) struct Entering {
    pub from: Rect,
    pub start: f64,
}

#[derive(Default)]
pub(crate) struct CommitOptions {
    pub animate: Option<bool>,
    pub silent: bool,
    pub from: Option<HashMap<Id, Rect>>,
}

pub struct Workspace {
    pub(crate) me: WeakEntity<Self>,
    pub(crate) opts: WorkspaceOptions,
    bindings: keymap::Bindings,
    pub(crate) chords: keymap::Chords,
    pub(crate) types: HashMap<String, ViewType>,
    pub(crate) doc: LayoutDocument,
    pub(crate) entries: Entries,
    pub(crate) focused_panel: Option<Id>,
    pub(crate) focused_view: Option<Id>,
    pub(crate) last_stage_panel: Option<Id>,
    pub(crate) badges: HashMap<Id, Badge>,
    pub(crate) guards: HashMap<Id, Vec<Guard>>,
    pub(crate) prop_guards: HashMap<Id, Guard>,
    pub(crate) views: HashMap<Id, ViewInstance>,
    /// Views whose tab closed: their entities are released at the next render.
    pub(crate) released: Vec<AnyView>,
    /// Content declared with `<View>` / `<Template>` markup.
    pub(crate) hosted: crate::ui::Hosting,
    pub(crate) last_rects: HashMap<Id, Rect>,
    /// Each tab's rect (window pixels) as last laid out, read back while rendering.
    pub(crate) tab_rects: HashMap<Id, Rect>,
    pub(crate) leaving: HashMap<Id, Leaving>,
    pub(crate) entering: HashMap<Id, Entering>,
    pub(crate) settling: HashSet<Id>,
    pub(crate) tween: LayoutTween,
    pub(crate) camera: RectSpring,
    pub(crate) nav: NavState,
    pub(crate) drag: Option<DragSession>,
    pub(crate) lifted: Option<PanelNode>,
    pub(crate) divider: Option<DividerDrag>,
    pub(crate) resize: Option<FloatResize>,
    pub(crate) suppress_clicks_until: f64,
    pub(crate) viewport: (f64, f64),
    /// The workspace's own rect in the window (window pixels), as last laid out.
    pub(crate) bounds: Rect,
    pub(crate) epoch: Instant,
    pub(crate) animating: bool,
    pub(crate) last_frame: Option<f64>,
    /// The last press landed in a panel's content (not on the workspace's chrome).
    pub(crate) in_content: bool,
    /// A press reached a panel's content on its way up (read by the workspace's own handler).
    pub(crate) content_press: bool,
    pub(crate) held: HashSet<String>,
    persist_gen: u64,
    pub(crate) closed: Vec<Closed>,
    doc_gen: u64,
    titles: HashMap<Id, SharedString>,
    hovered_panel: Option<Id>,
    pub(crate) toast_until: f64,
    pub(crate) menu: Option<Popup>,
    events: Events,
    sender: Option<UiSender>,
    pub(crate) open_counter: usize,
    pub(crate) remembered_floats: HashMap<Id, Rect>,
    pub(crate) fx: Fx,
}

impl Workspace {
    pub fn new(
        options: WorkspaceOptions,
        types: HashMap<String, ViewType>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let mut ws = Self::blank(options, types, cx);
        ws.start(cx);
        ws
    }

    pub(crate) fn blank(
        options: WorkspaceOptions,
        types: HashMap<String, ViewType>,
        cx: &mut Context<Self>,
    ) -> Self {
        Self {
            me: cx.weak_entity(),
            types,
            doc: empty_document(),
            entries: Entries::default(),
            focused_panel: None,
            focused_view: None,
            last_stage_panel: None,
            badges: HashMap::new(),
            guards: HashMap::new(),
            prop_guards: HashMap::new(),
            views: HashMap::new(),
            released: Vec::new(),
            hosted: crate::ui::Hosting::default(),
            last_rects: HashMap::new(),
            tab_rects: HashMap::new(),
            leaving: HashMap::new(),
            entering: HashMap::new(),
            settling: HashSet::new(),
            tween: LayoutTween::default(),
            camera: RectSpring::new(Rect::UNIT),
            nav: NavState::new(),
            drag: None,
            lifted: None,
            divider: None,
            resize: None,
            suppress_clicks_until: 0.0,
            viewport: (0.0, 0.0),
            bounds: Rect::ZERO,
            epoch: Instant::now(),
            animating: false,
            last_frame: None,
            in_content: false,
            content_press: false,
            held: HashSet::new(),
            persist_gen: 0,
            closed: Vec::new(),
            doc_gen: 0,
            titles: HashMap::new(),
            hovered_panel: None,
            toast_until: 0.0,
            menu: None,
            events: Events::default(),
            sender: None,
            open_counter: 0,
            remembered_floats: HashMap::new(),
            fx: Fx::default(),
            bindings: keymap::Bindings::new(&merged_keymap(&options.keymap)),
            chords: keymap::Chords::new(&options.gesture_keys),
            opts: options,
        }
    }

    pub(crate) fn start(&mut self, cx: &mut Context<Self>) -> bool {
        let persisted = self.load_persisted();
        let restored = self.opts.document.is_none() && persisted.is_some();
        let initial = self
            .opts
            .document
            .clone()
            .or(persisted)
            .or_else(|| self.opts.default_layout.clone())
            .unwrap_or_else(empty_document);
        self.doc = sanitize(&initial, &|_| true);
        self.sync();
        let nav = self.doc.navigation.clone();
        self.nav_restore(nav, cx);
        self.camera.finish();
        if let Some(first) = panels_of(self.doc.root.as_ref())
            .first()
            .map(|p| p.selected.clone())
            .or_else(|| self.doc.floating.first().map(|f| f.panel.selected.clone()))
        {
            self.set_focus(Some(first), false, cx);
        }
        restored
    }

    pub(crate) fn set_options(&mut self, options: WorkspaceOptions) {
        if options.keymap != self.opts.keymap {
            self.bindings = keymap::Bindings::new(&merged_keymap(&options.keymap));
        }
        if options.gesture_keys != self.opts.gesture_keys {
            self.chords = keymap::Chords::new(&options.gesture_keys);
        }
        let (default_layout, document) = (self.opts.default_layout.take(), self.opts.document.take());
        self.opts = WorkspaceOptions {
            default_layout,
            document,
            ..options
        };
    }

    pub(crate) fn element(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let element = blenderui::prelude::IntoElement::into_any_element(blenderui::prelude::Render::render(self, window, cx));
        self.hosted.clear_frame();
        element
    }

    /// A slot's content: given in `<Workspace>` markup this frame, else made by its [`SlotFn`].
    pub(crate) fn slot(&mut self, slot: Slot, window: &mut Window, cx: &mut App) -> Option<AnyElement> {
        if let Some(el) = self.hosted.slots[slot as usize].take() {
            return Some(el);
        }
        let s = &self.opts.slots;
        let f = match slot {
            Slot::Backdrop => &s.backdrop,
            Slot::StageEmpty => &s.stage_empty,
            Slot::Empty => &s.empty,
            Slot::Chrome => &s.chrome,
        };
        f.clone().map(|f| f(window, cx))
    }

    pub(crate) fn now(&self) -> f64 {
        self.epoch.elapsed().as_secs_f64() * 1000.0
    }

    pub(crate) const fn reduced(&self) -> bool {
        self.opts.reduced_motion
    }

    pub(crate) const fn floating_layer(&self) -> Option<FloatingLayer> {
        self.opts.floating
    }

    pub(crate) const fn nav_mode(&self) -> Navigation {
        self.opts.navigation
    }

    pub(crate) fn keymap(&self) -> Keymap {
        merged_keymap(&self.opts.keymap)
    }

    pub(crate) fn wake(&mut self, cx: &mut Context<Self>) {
        if !self.animating {
            self.animating = true;
            self.last_frame = None;
        }
        cx.notify();
    }

    pub(crate) fn pad(&self) -> f64 {
        GAP / 2.0
    }

    pub(crate) fn metrics(&self) -> PixelMetrics {
        PixelMetrics {
            width: self.viewport.0 - self.pad() * 2.0,
            height: self.viewport.1 - self.pad() * 2.0,
            gap: GAP,
            tabbar_height: TABBAR_HEIGHT,
            panel_min_width: PANEL_MIN_WIDTH,
        }
    }

    pub(crate) fn layout(&self, root: Option<&Node>) -> Entries {
        let m = self.metrics();
        layout_rects(root, Some(&m as &dyn Metrics))
    }

    /// `ThemeMode::System` follows BUI's theme (`blenderui::set_theme`).
    pub(crate) fn palette(&self) -> Palette {
        Palette::for_theme(self.opts.theme, blenderui::theme_name() == "dark")
    }

    pub(crate) fn type_of(&self, view_id: &str) -> Option<&ViewType> {
        self.doc
            .views
            .get(view_id)
            .and_then(|r| self.types.get(&r.ty))
    }

    pub(crate) fn rules_of(&self, view_id: &str) -> &ViewRules {
        static DEFAULT: std::sync::LazyLock<ViewRules> =
            std::sync::LazyLock::new(ViewRules::default);
        self.type_of(view_id).map_or(&DEFAULT, |t| &t.rules)
    }

    pub(crate) fn title_of(&self, view_id: &str) -> String {
        let Some(record) = self.doc.views.get(view_id) else {
            return String::new();
        };
        if let Some(title) = &record.title {
            return title.clone();
        }
        match self.types.get(&record.ty) {
            Some(t) => (t.title)(record.params.as_ref().unwrap_or(&Value::Null)),
            None => record.ty.clone(),
        }
    }

    pub(crate) fn title_shared(&mut self, view_id: &Id) -> SharedString {
        if let Some(t) = self.titles.get(view_id) {
            return t.clone();
        }
        let title = SharedString::from(self.title_of(view_id));
        self.titles.insert(view_id.clone(), title.clone());
        title
    }

    pub(crate) fn placement_of(&self, view_id: &str) -> ViewPlacement {
        let panel = panel_of_view(&self.doc, view_id).or_else(|| {
            self.lifted
                .as_ref()
                .filter(|l| l.views.iter().any(|v| v.as_ref() == view_id))
        });
        let Some(panel) = panel else {
            return ViewPlacement::Docked;
        };
        if self.doc.hidden.iter().any(|h| h.panel.id == panel.id) {
            return ViewPlacement::Hidden;
        }
        match self.region_of(&panel.id) {
            Region::Floating => ViewPlacement::Floating,
            Region::Stage => ViewPlacement::Stage,
            Region::Side => ViewPlacement::Docked,
        }
    }

    pub(crate) fn info_of(&self, view_id: &str) -> Option<ViewInfo> {
        let record = self.doc.views.get(view_id)?;
        let panel = panel_of_view(&self.doc, view_id).or_else(|| {
            self.lifted
                .as_ref()
                .filter(|l| l.views.iter().any(|v| v.as_ref() == view_id))
        });
        Some(ViewInfo {
            id: view_id.into(),
            ty: record.ty.clone(),
            params: record.params.clone().unwrap_or(Value::Null),
            title: self.title_of(view_id),
            panel_id: panel.map(|p| p.id.clone()),
            placement: self.placement_of(view_id),
            selected: panel.is_some_and(|p| p.selected.as_ref() == view_id),
        })
    }

    pub(crate) fn allowed(&self, views: &[Id], region: Region) -> bool {
        views.iter().all(|id| {
            let allow = self.rules_of(id).allow;
            match region {
                Region::Stage => allow.stage,
                Region::Side => allow.side,
                Region::Floating => allow.floating && self.floating_layer().is_some(),
            }
        })
    }

    pub(crate) fn region_of(&self, panel_id: &str) -> Region {
        if self
            .doc
            .floating
            .iter()
            .any(|f| f.panel.id.as_ref() == panel_id)
        {
            return Region::Floating;
        }
        match find_stage(self.doc.root.as_ref()) {
            Some(stage) if find_node(Some(&Node::Stage(stage.clone())), panel_id).is_some() => {
                Region::Stage
            }
            _ => Region::Side,
        }
    }

    pub(crate) fn find_panel(&self, panel_id: &str) -> Option<PanelNode> {
        if let Some(l) = self.lifted.as_ref().filter(|l| l.id.as_ref() == panel_id) {
            return Some(l.clone());
        }
        locate_panel(&self.doc, panel_id)
            .map(|l| l.panel().clone())
            .or_else(|| self.leaving.get(panel_id).map(|l| l.panel.clone()))
    }

    pub(crate) fn resolve_panel(&self, id: &str) -> Option<PanelNode> {
        locate_panel(&self.doc, id)
            .map(|l| l.panel().clone())
            .or_else(|| panel_of_view(&self.doc, id).cloned())
    }

    pub(crate) fn to_screen_with(&self, camera: Rect, r: Rect) -> Rect {
        let p = self.pad();
        let (w, h) = (self.viewport.0 - p * 2.0, self.viewport.1 - p * 2.0);
        Rect {
            x: p + (r.x - camera.x) / camera.w * w,
            y: p + (r.y - camera.y) / camera.h * h,
            w: r.w / camera.w * w,
            h: r.h / camera.h * h,
        }
    }

    pub(crate) fn to_screen(&self, r: Rect) -> Rect {
        self.to_screen_with(self.camera.value, r)
    }

    pub(crate) fn to_world(&self, p: (f64, f64)) -> (f64, f64) {
        let c = self.camera.value;
        let q = self.pad();
        (
            c.x + (p.0 - q) / (self.viewport.0 - q * 2.0) * c.w,
            c.y + (p.1 - q) / (self.viewport.1 - q * 2.0) * c.h,
        )
    }

    pub(crate) fn rect_to_world(&self, screen: Option<Rect>) -> Option<Rect> {
        let s = screen?;
        let p = self.pad();
        let a = self.to_world((s.x - p, s.y - p));
        let b = self.to_world((s.x + s.w + p, s.y + s.h + p));
        Some(Rect {
            x: a.0,
            y: a.1,
            w: b.0 - a.0,
            h: b.1 - a.1,
        })
    }

    pub(crate) fn panel_screen(&self, world: Rect) -> Rect {
        self.to_screen(world).inset(self.pad())
    }

    pub(crate) fn stage_world(&self) -> Rect {
        find_stage(self.doc.root.as_ref())
            .and_then(|s| self.entries.rect(&s.id))
            .unwrap_or(Rect::UNIT)
    }

    pub(crate) fn stage_screen(&self) -> Option<Rect> {
        let stage = find_stage(self.doc.root.as_ref())?;
        self.entries.rect(&stage.id).map(|r| self.panel_screen(r))
    }

    pub(crate) fn float_container(&self, layer: FloatingLayer) -> Rect {
        match layer {
            FloatingLayer::Overlay => Rect::new(0.0, 0.0, self.viewport.0, self.viewport.1),
            FloatingLayer::Stage => self.to_screen(self.stage_world()),
        }
    }

    pub(crate) fn float_screen(&self, rect: Rect, layer: FloatingLayer) -> Rect {
        let c = self.float_container(layer);
        Rect {
            x: c.x + rect.x * c.w,
            y: c.y + rect.y * c.h,
            w: rect.w * c.w,
            h: rect.h * c.h,
        }
    }

    pub(crate) fn screen_to_float(&self, r: Rect, layer: FloatingLayer) -> Rect {
        let c = self.float_container(layer);
        Rect {
            x: (r.x - c.x) / c.w,
            y: (r.y - c.y) / c.h,
            w: r.w / c.w,
            h: r.h / c.h,
        }
    }

    pub(crate) fn float_world(&self, panel_id: &str) -> Option<Rect> {
        let f = self
            .doc
            .floating
            .iter()
            .find(|f| f.panel.id.as_ref() == panel_id)?;
        if f.layer != FloatingLayer::Stage {
            return None;
        }
        let s = self.stage_world();
        Some(Rect {
            x: s.x + f.rect.x * s.w,
            y: s.y + f.rect.y * s.h,
            w: f.rect.w * s.w,
            h: f.rect.h * s.h,
        })
    }

    pub(crate) fn tabbar_height_of(&self, panel: &PanelNode) -> f64 {
        if self.bar_mode(panel) == BarMode::Normal {
            TABBAR_HEIGHT
        } else {
            0.0
        }
    }

    pub(crate) fn target_rect_with(&self, camera: Rect, panel_id: &str) -> Option<Rect> {
        if let Some(d) = self.drag.as_ref().filter(|d| d.active) {
            if d.lifted.id.as_ref() == panel_id {
                return Some(d.floating);
            }
            let is_float = self
                .doc
                .floating
                .iter()
                .any(|f| f.panel.id.as_ref() == panel_id);
            if !is_float && let Some(preview) = d.layout_targets.get(panel_id) {
                return Some(self.to_screen_with(camera, *preview).inset(self.pad()));
            }
        }
        if let Some(f) = self
            .doc
            .floating
            .iter()
            .find(|f| f.panel.id.as_ref() == panel_id)
        {
            return Some(match f.layer {
                FloatingLayer::Overlay => self.float_screen(f.rect, f.layer),
                FloatingLayer::Stage => {
                    let c = self.to_screen_with(camera, self.stage_world());
                    Rect {
                        x: c.x + f.rect.x * c.w,
                        y: c.y + f.rect.y * c.h,
                        w: f.rect.w * c.w,
                        h: f.rect.h * c.h,
                    }
                }
            });
        }
        self.entries
            .rect(panel_id)
            .map(|r| self.to_screen_with(camera, r).inset(self.pad()))
    }

    pub(crate) fn target_rect(&self, panel_id: &str) -> Option<Rect> {
        self.target_rect_with(self.camera.value, panel_id)
    }

    pub(crate) fn in_view(&self, r: &Rect) -> bool {
        r.x < self.viewport.0 - 1.0
            && r.y < self.viewport.1 - 1.0
            && r.right() > 1.0
            && r.bottom() > 1.0
    }

    pub(crate) fn sync(&mut self) {
        self.entries = self.layout(self.doc.root.as_ref());
        self.titles.clear();
        let mut live: HashSet<Id> = view_ids(&self.doc).into_iter().collect();
        if let Some(l) = &self.lifted {
            live.extend(l.views.iter().cloned());
        }
        for l in self.leaving.values() {
            live.extend(l.panel.views.iter().cloned());
        }
        for h in &self.doc.hidden {
            live.extend(h.panel.views.iter().cloned());
        }
        self.released.extend(
            self.views
                .extract_if(|id, _| !live.contains(id))
                .map(|(_, view)| view.content),
        );
        self.badges.retain(|id, _| live.contains(id));
        self.guards.retain(|id, _| live.contains(id));
        self.prop_guards.retain(|id, _| live.contains(id));
        self.tab_rects.retain(|id, _| live.contains(id));
        self.fx.inset_widths.retain(|id, _| live.contains(id));
        self.fx.scales.retain(|id, _| live.contains(id));
        self.hosted.retain(&live);
        let panels: HashSet<&Id> = self
            .doc
            .floating
            .iter()
            .map(|f| &f.panel.id)
            .chain(self.doc.hidden.iter().map(|h| &h.panel.id))
            .chain(self.leaving.keys())
            .chain(self.lifted.as_ref().map(|l| &l.id))
            .collect();
        self.last_rects.retain(|id, _| {
            self.entries.contains(id)
                || panels.contains(id)
                || id.as_ref() == super::drag::DROP_SLOT
                || id.as_ref() == super::drag::SOURCE_SLOT
        });
        self.nav_sync();
    }

    pub(crate) fn set_doc(&mut self, doc: LayoutDocument) {
        self.doc = doc;
        self.doc_gen += 1;
        self.sync();
    }

    pub(crate) fn commit(
        &mut self,
        next: LayoutDocument,
        o: CommitOptions,
        cx: &mut Context<Self>,
    ) {
        if next == self.doc {
            return;
        }
        self.doc_gen += 1;
        let animate = o.animate.unwrap_or(true) && !self.reduced();
        if animate {
            let mut from = self.last_rects.clone();
            if let Some(extra) = o.from {
                from.extend(extra);
            }
            let now = self.now();
            self.tween.begin(from, now);
        }
        let previous = std::mem::replace(&mut self.doc, next);
        let before = view_ids(&previous);
        let after = view_ids(&self.doc);
        let (opened, closed): (Vec<Id>, Vec<Id>) = if before == after {
            (Vec::new(), Vec::new())
        } else {
            let (b, a): (HashSet<&Id>, HashSet<&Id>) =
                (before.iter().collect(), after.iter().collect());
            (
                a.difference(&b).map(|id| (*id).clone()).collect(),
                b.difference(&a).map(|id| (*id).clone()).collect(),
            )
        };
        self.sync();
        if self.note_tab_switch(&previous) {
            self.wake(cx);
        }

        for id in &opened {
            if let Some(info) = self.info_of(id) {
                self.emit(WorkspaceEvent::Open(info), cx);
            }
        }
        for id in &closed {
            if self.lifted.as_ref().is_some_and(|l| l.views.contains(id)) {
                continue;
            }
            if let Some(record) = previous.views.get(id) {
                let info = ViewInfo {
                    id: id.clone(),
                    ty: record.ty.clone(),
                    params: record.params.clone().unwrap_or(Value::Null),
                    title: record.title.clone().unwrap_or_else(|| record.ty.clone()),
                    panel_id: None,
                    placement: ViewPlacement::Docked,
                    selected: false,
                };
                self.emit(WorkspaceEvent::Close(info), cx);
            }
        }
        if let Some(fv) = self.focused_view.clone() {
            if !after.contains(&fv) && !self.lifted.as_ref().is_some_and(|l| l.views.contains(&fv))
            {
                let fallback = self
                    .last_stage_panel
                    .as_ref()
                    .and_then(|p| locate_panel(&self.doc, p).map(|l| l.panel().selected.clone()))
                    .and_then(|v| panel_of_view(&self.doc, &v).map(|p| p.selected.clone()))
                    .or_else(|| {
                        panels_of(self.doc.root.as_ref())
                            .first()
                            .map(|p| p.selected.clone())
                    })
                    .or_else(|| self.doc.floating.first().map(|f| f.panel.selected.clone()));
                self.set_focus(fallback, false, cx);
            } else if let Some(panel) = panel_of_view(&self.doc, &fv) {
                self.focused_panel = Some(panel.id.clone());
            }
        }
        if !o.silent {
            self.emit_change(cx);
        }
        self.wake(cx);
    }

    pub(crate) fn emit_change(&mut self, cx: &mut Context<Self>) {
        if self.events.listened() {
            self.emit(WorkspaceEvent::Change(self.document()), cx);
        }
        self.persist(cx);
    }

    /// Queue `event` for whoever listens (nothing is kept when nobody does).
    pub(crate) fn emit(&mut self, event: WorkspaceEvent, cx: &mut Context<Self>) {
        let events = &mut self.events;
        match (events.subscribers.is_empty(), events.hosted.as_mut()) {
            (true, None) => return,
            (true, Some(hosted)) => return hosted.push(event),
            (false, Some(hosted)) => hosted.push(event.clone()),
            (false, None) => {}
        }
        events.queued.push(event);
        if !std::mem::replace(&mut events.scheduled, true) {
            let me = self.me;
            self.sender(cx).send(move |cx| Self::deliver(me, cx));
        }
    }

    fn deliver(me: WeakEntity<Self>, cx: &mut App) {
        let Some(entity) = me.upgrade_in(cx) else { return };
        let (queued, subscribers) = entity.update(cx, |ws, _| {
            ws.events.scheduled = false;
            (std::mem::take(&mut ws.events.queued), ws.events.subscribers.clone())
        });
        for event in &queued {
            for subscriber in &subscribers {
                subscriber(entity, event, cx);
            }
        }
    }

    /// Hear this workspace's events (gpui's `cx.subscribe(&workspace, ..)`): `f` runs right after
    /// each change, once the workspace can be updated again.
    pub fn subscribe(
        &mut self,
        f: impl Fn(Entity<Self>, &WorkspaceEvent, &mut App) + 'static,
        cx: &mut Context<Self>,
    ) {
        self.sender(cx);
        self.events.subscribers.push(Rc::new(f));
    }

    /// The `<Workspace>` component takes the events itself, while it renders.
    pub(crate) fn host_events(&mut self) {
        self.events.hosted.get_or_insert_with(Vec::new);
    }

    pub(crate) fn take_hosted_events(&mut self) -> Vec<WorkspaceEvent> {
        self.events.hosted.as_mut().map(std::mem::take).unwrap_or_default()
    }

    fn sender(&mut self, cx: &mut App) -> &UiSender {
        self.sender.get_or_insert_with(|| cx.ui_sender())
    }

    /// Run `f` on this workspace after `delay` (gpui's timer task): a thread waits, then
    /// blenderui's UI queue runs it.
    fn after(
        &mut self,
        delay: Duration,
        cx: &mut App,
        f: impl FnOnce(&mut Self, &mut Context<Self>) + Send + 'static,
    ) {
        let sender = self.sender(cx).clone();
        let me = self.me;
        std::thread::spawn(move || {
            std::thread::sleep(delay);
            sender.send(move |cx| {
                me.update(cx, f);
            });
        });
    }

    #[must_use]
    pub fn document(&self) -> LayoutDocument {
        let mut doc = self.doc.clone();
        doc.navigation = Some(self.nav_serialize());
        doc
    }

    #[allow(clippy::needless_pass_by_value)] // zopra-panels' signature
    pub fn set_document(&mut self, next: LayoutDocument, animate: bool, cx: &mut Context<Self>) {
        let prepared = sanitize(&next, &|_| true);
        let nav = prepared.navigation.clone();
        self.commit(
            prepared,
            CommitOptions {
                animate: Some(animate),
                ..Default::default()
            },
            cx,
        );
        self.nav_restore(nav, cx);
    }

    pub fn reset(&mut self, cx: &mut Context<Self>) {
        if let Some(p) = &self.opts.persist {
            std::fs::remove_file(&p.path).ok();
        }
        let doc = self
            .opts
            .default_layout
            .clone()
            .unwrap_or_else(empty_document);
        self.set_document(doc, true, cx);
    }

    fn load_persisted(&self) -> Option<LayoutDocument> {
        let p = self.opts.persist.as_ref()?;
        let doc: LayoutDocument = serde_json::from_slice(&std::fs::read(&p.path).ok()?).ok()?;
        (doc.schema == 1 && doc.version == p.version).then_some(doc)
    }

    /// Writes the layout 120 ms after the last change.
    fn persist(&mut self, cx: &mut Context<Self>) {
        if self.opts.persist.is_none() {
            return;
        }
        self.persist_gen += 1;
        let generation = self.persist_gen;
        self.after(Duration::from_millis(120), cx, move |this, _| {
            if this.persist_gen == generation {
                this.write_persisted();
            }
        });
    }

    fn write_persisted(&self) {
        let Some(p) = &self.opts.persist else { return };
        let mut doc = self.document();
        doc.version.clone_from(&p.version);
        if let Ok(bytes) = serde_json::to_vec(&doc) {
            std::fs::write(&p.path, bytes).ok();
        }
    }

    pub(crate) fn set_focus(&mut self, view: Option<Id>, emit: bool, cx: &mut Context<Self>) {
        let panel = view
            .as_ref()
            .and_then(|v| panel_of_view(&self.doc, v))
            .map(|p| p.id.clone());
        if self.focused_view == view && self.focused_panel == panel {
            return;
        }
        self.focused_view.clone_from(&view);
        self.focused_panel.clone_from(&panel);
        if let Some(p) = &panel
            && self.region_of(p) == Region::Stage
        {
            self.last_stage_panel = Some(p.clone());
        }
        if emit {
            self.emit(WorkspaceEvent::Focus(view), cx);
        }
        cx.notify();
    }

    pub(crate) fn focus_view(&mut self, view_id: &Id, cx: &mut Context<Self>) {
        if !self.doc.views.contains_key(view_id) {
            return;
        }
        self.set_focus(Some(view_id.clone()), true, cx);
        self.raise_if_floating(view_id, cx);
        let collapsed = panel_of_view(&self.doc, view_id)
            .map(|p| p.id.clone())
            .filter(|p| self.fx.collapsed_of.contains_key(p));
        if let Some(panel) = collapsed.filter(|_| !self.drag.as_ref().is_some_and(|d| d.active)) {
            self.nav_focus(Some(panel), true, cx);
        }
    }

    fn raise_if_floating(&mut self, view_id: &Id, cx: &mut Context<Self>) {
        let Some(panel) = panel_of_view(&self.doc, view_id) else {
            return;
        };
        if self.drag.as_ref().is_some_and(|d| d.active) {
            return;
        }
        let raised = raise_float(&self.doc, &panel.id.clone());
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

    pub(crate) fn select_and_focus(&mut self, view_id: &Id, cx: &mut Context<Self>) {
        let next = select_view(&self.doc, view_id);
        if next != self.doc {
            self.commit(
                next,
                CommitOptions {
                    animate: Some(false),
                    ..Default::default()
                },
                cx,
            );
        }
        self.focus_view(view_id, cx);
    }

    fn cascade_rect(&mut self, layer: FloatingLayer) -> Rect {
        let c = self.float_container(layer);
        let (w, h) = (
            (DEFAULT_FLOAT_SIZE.0).min(c.w * 0.6),
            (DEFAULT_FLOAT_SIZE.1).min(c.h * 0.6),
        );
        let step = (self.open_counter % 6) as f64 * 28.0;
        self.open_counter += 1;
        clamp_float(Rect {
            x: (c.w * 0.5 - w / 2.0 + step - 70.0) / c.w,
            y: (c.h * 0.45 - h / 2.0 + step - 50.0) / c.h,
            w: w / c.w,
            h: h / c.h,
        })
    }

    fn default_placement(&self, ty: &str) -> Placement {
        if let Some(p) = self.types.get(ty).and_then(|t| t.rules.placement.clone()) {
            return p;
        }
        if find_stage(self.doc.root.as_ref()).is_some() {
            Placement::Stage
        } else if self.focused_panel.is_some() {
            Placement::Tab
        } else {
            Placement::Side
        }
    }

    fn region_in(d: &LayoutDocument, id: &str) -> Region {
        if d.floating.iter().any(|f| f.panel.id.as_ref() == id) {
            return Region::Floating;
        }
        match find_stage(d.root.as_ref()) {
            Some(stage) if find_node(Some(&Node::Stage(stage.clone())), id).is_some() => {
                Region::Stage
            }
            _ => Region::Side,
        }
    }

    pub(crate) fn place_panel(
        &mut self,
        d: &LayoutDocument,
        panel: PanelNode,
        placement: Placement,
    ) -> LayoutDocument {
        let views = panel.views.clone();
        let stage = find_stage(d.root.as_ref()).cloned();
        let fl = self.floating_layer();
        let focused = self
            .focused_panel
            .clone()
            .filter(|p| locate_panel(d, p).is_some());

        let fallback = (placement != Placement::Side).then_some(Placement::Side);
        for p in std::iter::once(placement).chain(fallback) {
            match p {
                Placement::Float | Placement::FloatAt { .. } => {
                    let Some(default_layer) = fl else { continue };
                    if !self.allowed(&views, Region::Floating) {
                        continue;
                    }
                    let (layer, rect) = match p {
                        Placement::FloatAt { rect, layer } => {
                            (layer.unwrap_or(default_layer), rect)
                        }
                        _ => (default_layer, self.cascade_rect(default_layer)),
                    };
                    return float_panel(d, panel, rect, layer);
                }
                Placement::Beside {
                    beside,
                    edge,
                    share,
                } => {
                    let target = find_node(d.root.as_ref(), &beside).map(|_| beside.clone());
                    let region = match (&target, &stage) {
                        (Some(t), Some(s))
                            if t != &s.id
                                && find_node(Some(&Node::Stage(s.clone())), t).is_some() =>
                        {
                            Region::Stage
                        }
                        _ => Region::Side,
                    };
                    if !self.allowed(&views, region) {
                        continue;
                    }
                    let beside = target
                        .or_else(|| d.root.as_ref().map(|r| r.id().clone()))
                        .unwrap_or_else(|| "".into());
                    return insert_panel(
                        d,
                        panel,
                        &DockTarget::Beside {
                            beside,
                            edge,
                            share,
                        },
                    );
                }
                Placement::Into { panel: into, index } => {
                    let loc = locate_panel(d, &into).is_some();
                    let node_is_stage =
                        matches!(find_node(d.root.as_ref(), &into), Some(Node::Stage(_)));
                    let region = if node_is_stage {
                        Region::Stage
                    } else if loc {
                        Self::region_in(d, &into)
                    } else {
                        Region::Side
                    };
                    if !self.allowed(&views, region) || (!loc && !node_is_stage) {
                        continue;
                    }
                    return insert_panel(d, panel, &DockTarget::Into { panel: into, index });
                }
                Placement::Stage => {
                    let Some(stage) = &stage else {
                        if let Some(f) = &focused
                            && self.allowed(&views, Self::region_in(d, f))
                        {
                            return insert_panel(
                                d,
                                panel,
                                &DockTarget::Into {
                                    panel: f.clone(),
                                    index: None,
                                },
                            );
                        }
                        continue;
                    };
                    if !self.allowed(&views, Region::Stage) {
                        continue;
                    }
                    if stage.child.is_none() {
                        return insert_panel(
                            d,
                            panel,
                            &DockTarget::Into {
                                panel: stage.id.clone(),
                                index: None,
                            },
                        );
                    }
                    let in_stage = panels_of(stage.child.as_deref());
                    let target = focused
                        .filter(|f| in_stage.iter().any(|p| &p.id == f))
                        .or_else(|| {
                            self.last_stage_panel
                                .clone()
                                .filter(|l| in_stage.iter().any(|p| &p.id == l))
                        })
                        .unwrap_or_else(|| in_stage[0].id.clone());
                    return insert_panel(
                        d,
                        panel,
                        &DockTarget::Into {
                            panel: target,
                            index: None,
                        },
                    );
                }
                Placement::Tab => {
                    if let Some(f) = &focused
                        && self.allowed(&views, Self::region_in(d, f))
                    {
                        return insert_panel(
                            d,
                            panel,
                            &DockTarget::Into {
                                panel: f.clone(),
                                index: None,
                            },
                        );
                    }
                    if stage.is_some() {
                        return self.place_panel(d, panel, Placement::Stage);
                    }
                }
                Placement::Side => {
                    let Some(root) = &d.root else {
                        if self.allowed(&views, Region::Side) {
                            let mut next = d.clone();
                            next.root = Some(Node::Panel(panel));
                            return next;
                        }
                        continue;
                    };
                    if !self.allowed(&views, Region::Side) {
                        break;
                    }
                    if let Some(stage) = &stage
                        && &stage.id != root.id()
                    {
                        let t = DockTarget::Beside {
                            beside: stage.id.clone(),
                            edge: Edge::Right,
                            share: Some(0.25),
                        };
                        return insert_panel(d, panel, &t);
                    }
                    let t = DockTarget::Beside {
                        beside: root.id().clone(),
                        edge: Edge::Right,
                        share: Some(if stage.is_some() { 0.25 } else { 0.35 }),
                    };
                    return insert_panel(d, panel, &t);
                }
            }
        }
        if let Some(layer) = fl
            && d.root.is_some()
        {
            let rect = self.cascade_rect(layer);
            return float_panel(d, panel, rect, layer);
        }
        if let Some(root) = &d.root {
            insert_panel(
                d,
                panel,
                &DockTarget::Beside {
                    beside: root.id().clone(),
                    edge: Edge::Right,
                    share: None,
                },
            )
        } else {
            let mut next = d.clone();
            next.root = Some(Node::Panel(panel));
            next
        }
    }

    pub fn open(&mut self, ty: &str, o: OpenOptions, cx: &mut Context<Self>) -> Option<ViewInfo> {
        let def = self.types.get(ty)?;
        let reuse = match o.reuse {
            Reuse::Auto if def.rules.singleton => Reuse::Type,
            Reuse::Auto => Reuse::None,
            other => other,
        };
        let focus = o.focus.unwrap_or(true);
        if let Some(existing) = self.find_existing(ty, &o, reuse) {
            self.reveal(&existing, focus, cx);
            return self.info_of(&existing);
        }
        if let Some(id) = &o.id
            && self.doc.views.contains_key(id)
        {
            self.reveal(id, focus, cx);
            return self.info_of(id);
        }
        let OpenOptions {
            params,
            id,
            title,
            placement,
            from,
            ..
        } = o;
        let id: Id = id.unwrap_or_else(|| uid(ty));
        let mut with_record = self.doc.clone();
        with_record.views.insert(
            id.clone(),
            ViewRecord {
                ty: ty.to_string(),
                params,
                title,
            },
        );
        let panel_id = uid("panel");
        let panel = PanelNode {
            id: panel_id.clone(),
            views: vec![id.clone()],
            selected: id.clone(),
        };
        let placement = placement.unwrap_or_else(|| self.default_placement(ty));
        let next = self.place_panel(&with_record, panel, placement);
        let from = from.map(|r| HashMap::from([(panel_id, r)]));
        self.commit(
            next,
            CommitOptions {
                from,
                ..Default::default()
            },
            cx,
        );
        if focus {
            if let Some(target) = panel_of_view(&self.doc, &id).map(|p| p.id.clone())
                && self.doc.floating.iter().any(|f| f.panel.id == target)
            {
                let raised = raise_float(&self.doc, &target);
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
            self.focus_view(&id, cx);
            self.ensure_framed_visible(&id, cx);
        }
        self.info_of(&id)
    }

    fn find_existing(&self, ty: &str, o: &OpenOptions, reuse: Reuse) -> Option<Id> {
        if matches!(reuse, Reuse::None | Reuse::Auto) {
            return None;
        }
        view_ids(&self.doc).into_iter().find(|id| {
            let Some(record) = self.doc.views.get(id) else {
                return false;
            };
            if record.ty != ty {
                return false;
            }
            match reuse {
                Reuse::Type => true,
                Reuse::Params => record.params == o.params,
                _ => false,
            }
        })
    }

    fn reveal(&mut self, view_id: &Id, focus: bool, cx: &mut Context<Self>) {
        let Some(panel) = panel_of_view(&self.doc, view_id).cloned() else {
            return;
        };
        if self.doc.hidden.iter().any(|h| h.panel.id == panel.id) {
            self.restore(&panel.id, None, cx);
        }
        let mut next = select_view(&self.doc, view_id);
        if self.doc.floating.iter().any(|f| f.panel.id == panel.id) {
            next = raise_float(&next, &panel.id);
        }
        if next != self.doc {
            self.commit(
                next,
                CommitOptions {
                    animate: Some(false),
                    ..Default::default()
                },
                cx,
            );
        }
        if focus {
            self.focus_view(view_id, cx);
        }
        self.ensure_framed_visible(view_id, cx);
    }

    pub(crate) fn ensure_framed_visible(&mut self, view_id: &str, cx: &mut Context<Self>) {
        if let Some(panel) = panel_of_view(&self.doc, view_id).map(|p| p.id.clone()) {
            self.nav_ensure_visible(&panel, cx);
        }
    }

    fn can_close(&self, view_id: &Id, cx: &App) -> bool {
        self.guards
            .get(view_id)
            .is_none_or(|gs| gs.iter().all(|g| g(cx)))
            && self.prop_guards.get(view_id).is_none_or(|g| g(cx))
            && self.hosted.allows_close(view_id, cx)
    }

    pub fn close(&mut self, view_id: &Id, force: bool, cx: &mut Context<Self>) -> bool {
        if !self.doc.views.contains_key(view_id) || (!force && !self.can_close(view_id, cx)) {
            return false;
        }
        let next = close_view(&self.doc, view_id);
        self.commit(next, CommitOptions::default(), cx);
        true
    }

    pub(crate) fn close_by_user(&mut self, view_id: &Id, cx: &mut Context<Self>) -> bool {
        let title = self.title_of(view_id);
        let record = self.doc.views.get(view_id).cloned();
        let panel = panel_of_view(&self.doc, view_id).map(|p| {
            let index = p.views.iter().position(|v| v == view_id).unwrap_or(0);
            (p.id.clone(), index)
        });
        let before = self.doc.clone();
        if !self.close(view_id, false, cx) {
            return false;
        }
        if let Some(last) = self.closed.last_mut() {
            last.exact = None;
        }
        self.closed.push(Closed {
            view: view_id.clone(),
            title,
            record,
            panel,
            exact: Some((self.doc_gen, before)),
        });
        if self.closed.len() > MAX_CLOSED {
            self.closed.remove(0);
        }
        let secs = self.opts.tabs.undo_seconds;
        if secs > 0.0 {
            self.toast_until = self.now() + secs * 1000.0;
            self.after(Duration::from_secs_f64(secs), cx, |_, cx| cx.notify());
        }
        cx.notify();
        true
    }

    pub(crate) fn close_tab_frozen(&mut self, view_id: &Id, cx: &mut Context<Self>) {
        let panel = panel_of_view(&self.doc, view_id).map(|p| p.id.clone());
        let width = self.tab_rects.get(view_id).map(|r| r.w);
        if self.close_by_user(view_id, cx)
            && let (Some(panel), Some(width)) = (panel, width)
            && self.find_panel(&panel).is_some()
        {
            self.fx.frozen_tabs = Some((panel, width));
        }
    }

    pub(crate) fn toast(&self) -> Option<&Closed> {
        self.closed.last().filter(|_| self.now() < self.toast_until)
    }

    pub fn reopen_closed(&mut self, cx: &mut Context<Self>) {
        let Some(c) = self.closed.pop() else { return };
        self.toast_until = 0.0;
        if self.doc.views.contains_key(&c.view) {
            return self.reveal(&c.view, true, cx);
        }
        if let Some((_, before)) = c
            .exact
            .filter(|(generation, _)| *generation == self.doc_gen)
        {
            self.commit(before, CommitOptions::default(), cx);
        } else if let Some(record) = c.record {
            let placement = c
                .panel
                .filter(|(panel, _)| self.find_panel(panel).is_some())
                .map(|(panel, index)| Placement::Into {
                    panel,
                    index: Some(index),
                });
            self.open(
                &record.ty,
                OpenOptions {
                    id: Some(c.view.clone()),
                    params: record.params,
                    title: record.title,
                    placement,
                    ..Default::default()
                },
                cx,
            );
        }
        if self.doc.views.contains_key(&c.view) {
            self.reveal(&c.view, true, cx);
        }
    }

    pub fn focus(&mut self, id: &Id, cx: &mut Context<Self>) {
        let view =
            locate_panel(&self.doc, id).map_or_else(|| id.clone(), |l| l.panel().selected.clone());
        if self.doc.views.contains_key(&view) {
            self.reveal(&view, true, cx);
        }
    }

    pub fn select(&mut self, view_id: &Id, cx: &mut Context<Self>) {
        let next = select_view(&self.doc, view_id);
        self.commit(
            next,
            CommitOptions {
                animate: Some(false),
                ..Default::default()
            },
            cx,
        );
    }

    fn toward_rect(toward: Option<Rect>, from: Rect) -> Rect {
        toward.unwrap_or(Rect {
            x: from.x + from.w * 0.25,
            y: from.y + from.h,
            w: from.w * 0.5,
            h: from.h * 0.1,
        })
    }

    pub fn hide(&mut self, id: &Id, toward: Option<Rect>, cx: &mut Context<Self>) {
        let owner = if locate_panel(&self.doc, id).is_none() {
            panel_of_view(&self.doc, id).cloned()
        } else {
            None
        };
        if let Some(owner) = owner
            .filter(|o| o.views.len() > 1 && !self.doc.hidden.iter().any(|h| h.panel.id == o.id))
        {
            let alone = PanelNode {
                id: uid("panel"),
                views: vec![id.clone()],
                selected: id.clone(),
            };
            let mut detached = detach_view(&self.doc, id);
            if let Some(from) = self
                .last_rects
                .get(&owner.id)
                .copied()
                .filter(|_| !self.reduced())
            {
                let start = self.now();
                self.leaving.insert(
                    alone.id.clone(),
                    Leaving {
                        panel: alone.clone(),
                        from,
                        to: Self::toward_rect(toward, from),
                        start,
                        duration: DOCK_HIDE_MS,
                    },
                );
            }
            detached.hidden.push(HiddenPanel {
                panel: alone,
                restore: RestoreTarget::Tab { panel: owner.id },
            });
            self.commit(detached, CommitOptions::default(), cx);
            return;
        }
        let Some(panel) = self.resolve_panel(id) else {
            return;
        };
        if self.doc.hidden.iter().any(|h| h.panel.id == panel.id) {
            return;
        }
        let from = self
            .last_rects
            .get(&panel.id)
            .copied()
            .or_else(|| self.target_rect(&panel.id));
        let next = hide_panel(&self.doc, &panel.id);
        if let Some(from) = from.filter(|_| !self.reduced()) {
            let start = self.now();
            self.leaving.insert(
                panel.id.clone(),
                Leaving {
                    panel,
                    from,
                    to: Self::toward_rect(toward, from),
                    start,
                    duration: DOCK_HIDE_MS,
                },
            );
        }
        self.commit(next, CommitOptions::default(), cx);
    }

    pub fn restore(&mut self, panel_id: &Id, from: Option<Rect>, cx: &mut Context<Self>) {
        let Some(hidden) = self
            .doc
            .hidden
            .iter()
            .find(|h| &h.panel.id == panel_id)
            .cloned()
        else {
            return;
        };
        self.leaving.remove(panel_id);
        let layer = self.floating_layer().unwrap_or(FloatingLayer::Overlay);
        let next = restore_panel(&self.doc, panel_id, layer);
        self.last_rects.remove(panel_id);
        if let Some(from) = from.filter(|_| !self.reduced()) {
            let start = self.now();
            self.entering
                .insert(panel_id.clone(), Entering { from, start });
        }
        self.commit(next, CommitOptions::default(), cx);
        self.focus_view(&hidden.panel.selected, cx);
    }

    pub fn toggle_dock(&mut self, id: &Id, cx: &mut Context<Self>) {
        let Some(panel) = self.resolve_panel(id) else {
            return;
        };
        if self.doc.hidden.iter().any(|h| h.panel.id == panel.id) {
            return;
        }
        let stage = find_stage(self.doc.root.as_ref()).cloned();
        let float = self
            .doc
            .floating
            .iter()
            .find(|f| f.panel.id == panel.id)
            .cloned();
        let mut from = HashMap::new();
        if let Some(cur) = self.last_rects.get(&panel.id) {
            from.insert(panel.id.clone(), *cur);
        }
        let without = remove_panel(&self.doc, &panel.id);
        if let Some(float) = float {
            if !self.allowed(&panel.views, Region::Side) {
                return;
            }
            self.settling.insert(panel.id.clone());
            self.remembered_floats.insert(panel.id.clone(), float.rect);
            let s = self.stage_screen();
            let next = match &stage {
                Some(stage)
                    if without
                        .root
                        .as_ref()
                        .is_some_and(|r| find_node(Some(r), &stage.id).is_some()) =>
                {
                    let edge = if s.is_none_or(|s| s.w >= s.h) {
                        Edge::Right
                    } else {
                        Edge::Bottom
                    };
                    insert_panel(
                        &without,
                        panel.clone(),
                        &DockTarget::Beside {
                            beside: stage.id.clone(),
                            edge,
                            share: None,
                        },
                    )
                }
                _ => self.place_panel(&without, panel.clone(), Placement::Side),
            };
            self.commit(
                next,
                CommitOptions {
                    from: Some(from),
                    ..Default::default()
                },
                cx,
            );
            let ids: Vec<Id> = stage
                .iter()
                .map(|s| s.id.clone())
                .chain([panel.id.clone()])
                .collect();
            self.nav_include(&ids, cx);
        } else {
            let Some(layer) = self.floating_layer() else {
                return;
            };
            if !self.allowed(&panel.views, Region::Floating) {
                return;
            }
            self.settling.insert(panel.id.clone());
            let rect = self
                .remembered_floats
                .get(&panel.id)
                .copied()
                .unwrap_or_else(|| self.cascade_rect(layer));
            self.commit(
                float_panel(&without, panel.clone(), rect, layer),
                CommitOptions {
                    from: Some(from),
                    ..Default::default()
                },
                cx,
            );
            if let Some(stage) = &stage {
                self.nav_include(std::slice::from_ref(&stage.id), cx);
            }
        }
        self.focus_view(&panel.selected, cx);
    }

    pub fn float(&mut self, id: &Id, rect: Option<Rect>, cx: &mut Context<Self>) {
        let Some(layer) = self.floating_layer() else {
            return;
        };
        let loc_panel = locate_panel(&self.doc, id).map(|l| l.panel().clone());
        let mut d = self.doc.clone();
        let panel = if let Some(p) = &loc_panel {
            p.clone()
        } else {
            let Some(owner) = panel_of_view(&self.doc, id).cloned() else {
                return;
            };
            if owner.views.len() > 1 {
                d = detach_view(&self.doc, id);
                PanelNode {
                    id: uid("panel"),
                    views: vec![id.clone()],
                    selected: id.clone(),
                }
            } else {
                owner
            }
        };
        if !self.allowed(&panel.views, Region::Floating) {
            return;
        }
        if locate_panel(&d, &panel.id).is_some_and(|l| l.place() == Where::Floating)
            && rect.is_none()
        {
            return;
        }
        let current = self.last_rects.get(&panel.id).copied();
        let target = rect.unwrap_or_else(|| {
            let base = current.unwrap_or_else(|| {
                let r = self.cascade_rect(layer);
                self.float_screen(r, layer)
            });
            let (w, h) = (
                base.w.min(DEFAULT_FLOAT_SIZE.0),
                base.h.min(DEFAULT_FLOAT_SIZE.1),
            );
            self.screen_to_float(
                Rect::new(
                    base.x + (base.w - w) / 2.0 + 24.0,
                    base.y + (base.h - h) / 2.0 + 24.0,
                    w,
                    h,
                ),
                layer,
            )
        });
        let mut from = HashMap::new();
        if let Some(cur) = current {
            from.insert(panel.id.clone(), cur);
        }
        if locate_panel(&d, &panel.id).is_some() {
            d = remove_panel(&d, &panel.id);
        }
        self.commit(
            float_panel(&d, panel, target, layer),
            CommitOptions {
                from: Some(from),
                ..Default::default()
            },
            cx,
        );
    }

    #[allow(clippy::needless_pass_by_value)] // zopra-panels' signature
    pub fn dock(&mut self, id: &Id, target: DockTarget, cx: &mut Context<Self>) {
        let Some(mut panel) = self.resolve_panel(id) else {
            return;
        };
        let d = if locate_panel(&self.doc, id).is_none() && panel.views.len() > 1 {
            let d = detach_view(&self.doc, id);
            panel = PanelNode {
                id: uid("panel"),
                views: vec![id.clone()],
                selected: id.clone(),
            };
            d
        } else {
            remove_panel(&self.doc, &panel.id)
        };
        let mut from = HashMap::new();
        if let Some(cur) = self.last_rects.get(&panel.id) {
            from.insert(panel.id.clone(), *cur);
        }
        let next = match &target {
            DockTarget::Into { panel: into, .. }
                if find_node(d.root.as_ref(), into).is_none()
                    && locate_panel(&d, into).is_none() =>
            {
                self.place_panel(&d, panel, Placement::Side)
            }
            _ if d.root.is_some() => insert_panel(&d, panel, &target),
            _ => {
                let mut next = d.clone();
                next.root = Some(Node::Panel(panel));
                next
            }
        };
        self.commit(
            next,
            CommitOptions {
                from: Some(from),
                ..Default::default()
            },
            cx,
        );
    }

    pub fn set_title(&mut self, view_id: &Id, title: String, cx: &mut Context<Self>) {
        let Some(record) = self.doc.views.get_mut(view_id) else {
            return;
        };
        if record.title.as_deref() == Some(title.as_str()) {
            return;
        }
        record.title = Some(title);
        self.titles.remove(view_id);
        self.doc_gen += 1;
        self.emit_change(cx);
        cx.notify();
    }

    pub fn set_params(&mut self, view_id: &Id, patch: Value, cx: &mut Context<Self>) {
        let Some(record) = self.doc.views.get_mut(view_id) else {
            return;
        };
        let mut merged = match record.params.take() {
            Some(Value::Object(m)) => m,
            _ => serde_json::Map::new(),
        };
        if let Value::Object(p) = patch {
            merged.extend(p);
        }
        record.params = Some(Value::Object(merged));
        self.titles.remove(view_id);
        self.doc_gen += 1;
        self.emit_change(cx);
        cx.notify();
    }

    pub fn set_badge(&mut self, view_id: &Id, badge: Option<Badge>, cx: &mut Context<Self>) {
        match badge {
            Some(b) => self.badges.insert(view_id.clone(), b),
            None => self.badges.remove(view_id),
        };
        cx.notify();
    }

    #[must_use]
    pub fn views(&self) -> Vec<ViewInfo> {
        view_ids(&self.doc)
            .iter()
            .filter_map(|id| self.info_of(id))
            .collect()
    }

    #[must_use]
    pub const fn focused_view(&self) -> Option<&Id> {
        self.focused_view.as_ref()
    }

    fn act(
        &self,
        f: impl Fn(&mut Self, &mut Context<Self>) + 'static,
    ) -> impl Fn(&mut Window, &mut App) + 'static {
        let me = self.me;
        move |_, cx| {
            me.update(cx, |ws, cx| f(ws, cx));
        }
    }

    fn view_menu(&self, panel: &PanelNode) -> Vec<MenuEntry> {
        match (
            self.type_of(&panel.selected).and_then(|t| t.menu.clone()),
            self.info_of(&panel.selected),
        ) {
            (Some(menu), Some(info)) => menu(&info),
            _ => Vec::new(),
        }
    }

    fn built_in_menu(&self, panel: &PanelNode) -> Vec<MenuEntry> {
        let panel_id = panel.id.clone();
        let region = self.region_of(&panel_id);
        let keymap = self.keymap();
        let hint = |c: Command| {
            keymap
                .get(&c)
                .cloned()
                .flatten()
                .map(|k| keymap::format_combo(&k))
        };
        let mut items: Vec<MenuEntry> = Vec::new();

        if self.nav_mode() != Navigation::Off && region != Region::Floating {
            let pid = panel_id.clone();
            items.push(
                MenuItem::new(if self.nav.framed.as_ref() == Some(&panel_id) {
                    "Restore size"
                } else {
                    "Maximize"
                })
                .id(builtin::MAXIMIZE)
                .shortcut(hint(Command::FrameToggle))
                .run(self.act(move |ws, cx| {
                    ws.toggle_frame(&pid, cx);
                }))
                .into(),
            );
        }
        if region == Region::Floating {
            let pid = panel_id.clone();
            let stage_layer = self.floating_layer() == Some(FloatingLayer::Stage);
            let can_stage = find_stage(self.doc.root.as_ref()).is_some()
                && self.allowed(&panel.views, Region::Stage);
            items.push(
                MenuItem::new(if stage_layer {
                    "Dock beside stage"
                } else {
                    "Dock"
                })
                .id(builtin::DOCK)
                .run(self.act(move |ws, cx| {
                    if stage_layer {
                        ws.toggle_dock(&pid, cx);
                    } else {
                        let target = if can_stage {
                            let stage = find_stage(ws.doc.root.as_ref())
                                .map_or_else(|| "".into(), |s| s.id.clone());
                            DockTarget::Into {
                                panel: stage,
                                index: None,
                            }
                        } else {
                            DockTarget::Beside {
                                beside: ws
                                    .doc
                                    .root
                                    .as_ref()
                                    .map_or_else(|| "".into(), |r| r.id().clone()),
                                edge: Edge::Right,
                                share: Some(0.3),
                            }
                        };
                        ws.dock(&pid, target, cx);
                    }
                }))
                .into(),
            );
        } else if self.floating_layer().is_some() && self.allowed(&panel.views, Region::Floating) {
            let pid = panel_id.clone();
            let stage_layer = self.floating_layer() == Some(FloatingLayer::Stage);
            items.push(
                MenuItem::new("Float")
                    .id(builtin::FLOAT)
                    .run(self.act(move |ws, cx| {
                        if stage_layer {
                            ws.toggle_dock(&pid, cx);
                        } else {
                            ws.float(&pid, None, cx);
                        }
                    }))
                    .into(),
            );
        }

        let view_id = panel.selected.clone();
        let mut moves: Vec<MenuEntry> = panels_of(self.doc.root.as_ref())
            .into_iter()
            .cloned()
            .chain(self.doc.floating.iter().map(|f| f.panel.clone()))
            .filter(|p| {
                p.id != panel_id
                    && self.allowed(std::slice::from_ref(&view_id), self.region_of(&p.id))
            })
            .map(|p| {
                let (vid, pid) = (view_id.clone(), p.id.clone());
                let label = if p.views.len() > 1 {
                    format!("{} +{}", self.title_of(&p.selected), p.views.len() - 1)
                } else {
                    self.title_of(&p.selected)
                };
                MenuItem::new(label)
                    .id(format!("move:{}", p.id))
                    .run(self.act(move |ws, cx| {
                        ws.dock(
                            &vid,
                            DockTarget::Into {
                                panel: pid.clone(),
                                index: None,
                            },
                            cx,
                        );
                        ws.focus_view(&vid, cx);
                    }))
                    .into()
            })
            .collect();
        if panel.views.len() > 1 && region != Region::Floating {
            for (label, id, edge) in [
                ("New split right", builtin::SPLIT_RIGHT, Edge::Right),
                ("New split below", builtin::SPLIT_BELOW, Edge::Bottom),
            ] {
                let (vid, pid) = (view_id.clone(), panel_id.clone());
                moves.push(
                    MenuItem::new(label)
                        .id(id)
                        .run(self.act(move |ws, cx| {
                            ws.dock(
                                &vid,
                                DockTarget::Beside {
                                    beside: pid.clone(),
                                    edge,
                                    share: None,
                                },
                                cx,
                            );
                        }))
                        .into(),
                );
            }
        }
        if !moves.is_empty() {
            items.push(
                MenuItem::new(format!("Move {} to", self.title_of(&view_id)))
                    .id(builtin::MOVE)
                    .submenu(moves)
                    .into(),
            );
        }

        let pid = panel_id;
        let toward = self.opts.hide_toward.clone();
        items.push(
            MenuItem::new("Hide")
                .id(builtin::HIDE)
                .run(self.act(move |ws, cx| {
                    let to = toward.as_ref().and_then(|f| f(&pid));
                    ws.hide(&pid, to, cx);
                }))
                .into(),
        );

        let closable: Vec<&Id> = panel
            .views
            .iter()
            .filter(|v| self.rules_of(v).is_closable())
            .collect();
        if !closable.is_empty() {
            items.push(MenuEntry::Separator);
            if self.rules_of(&panel.selected).is_closable() {
                let vid = panel.selected.clone();
                items.push(
                    MenuItem::new(format!("Close {}", self.title_of(&panel.selected)))
                        .id(builtin::CLOSE)
                        .shortcut(hint(Command::ViewClose))
                        .run(self.act(move |ws, cx| {
                            ws.close_by_user(&vid, cx);
                        }))
                        .into(),
                );
            }
            if panel.views.len() > 1 {
                let (selected, others) = (panel.selected.clone(), panel.views.clone());
                items.push(
                    MenuItem::new("Close other tabs")
                        .id(builtin::CLOSE_OTHERS)
                        .run(self.act(move |ws, cx| {
                            for v in others.iter().filter(|v| **v != selected) {
                                ws.close_by_user(v, cx);
                            }
                        }))
                        .into(),
                );
            }
        }
        items
    }

    pub(crate) fn panel_menu_entries(&self, panel_id: &str) -> Vec<MenuEntry> {
        let Some(panel) = self.find_panel(panel_id) else {
            return Vec::new();
        };
        let custom = self.view_menu(&panel);
        if matches!(self.opts.panel_menu, PanelMenu::Off) {
            return tidy_menu(custom);
        }
        let built_ins = self.built_in_menu(&panel);
        let all: Vec<MenuEntry> = if custom.is_empty() {
            built_ins
        } else {
            custom
                .into_iter()
                .chain([MenuEntry::Separator])
                .chain(built_ins)
                .collect()
        };
        match &self.opts.panel_menu {
            PanelMenu::Custom(f) => {
                let Some(view) = self.info_of(&panel.selected) else {
                    return tidy_menu(all);
                };
                let region = match self.region_of(panel_id) {
                    Region::Stage => MenuRegion::Stage,
                    Region::Side => MenuRegion::Side,
                    Region::Floating => MenuRegion::Floating,
                };
                tidy_menu(f(
                    tidy_menu(all),
                    &PanelMenuContext {
                        panel_id: panel.id,
                        view,
                        region,
                    },
                ))
            }
            _ => tidy_menu(all),
        }
    }

    /// A key pressed anywhere in the workspace: `Handled` when it changed something (the window
    /// renders again), `Run` with a menu row chosen with the keyboard, to run once the workspace
    /// is no longer borrowed.
    pub(crate) fn on_key_down(
        &mut self,
        ev: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> KeyResult {
        let menu = self.menu_key(&ev.keystroke.key, cx);
        if !matches!(menu, KeyResult::Ignored) {
            cx.stop_propagation();
            return menu;
        }
        let key = keymap::normalize_key(&ev.keystroke.key);
        if key == "escape" && self.capturing() {
            self.on_escape(cx);
            cx.stop_propagation();
            return KeyResult::Handled;
        }
        let changed = self.nav_modifiers_changed(ev.keystroke.modifiers, cx);
        let idle = if changed { KeyResult::Handled } else { KeyResult::Ignored };
        if self.drag.as_ref().is_some_and(|d| d.active) {
            self.held.insert(key);
            return idle;
        }
        if self.nav.shortcuts && key == "escape" {
            self.nav.shortcuts = false;
            cx.notify();
            cx.stop_propagation();
            return KeyResult::Handled;
        }
        self.held.insert(key);
        let skip_step_out = self.nav.framed.is_none() || self.in_content;
        let Some(command) = self
            .bindings
            .matching(&ev.keystroke)
            .find(|c| !(*c == Command::NavigationStepOut && skip_step_out))
        else {
            return idle;
        };
        if matches!(command, Command::TabNext | Command::TabPrevious) {
            let at = self.local_point(window.mouse_position());
            self.hovered_panel = self.panel_at(at).map(|(p, _)| p.id.clone());
        }
        self.run(command, cx);
        self.hovered_panel = None;
        cx.stop_propagation();
        KeyResult::Handled
    }

    /// A key let go: true when the held gesture (and its cursor) changed.
    pub(crate) fn on_key_up(&mut self, ev: &KeyUpEvent, cx: &mut Context<Self>) -> bool {
        let key = keymap::normalize_key(&ev.keystroke.key);
        if key == "meta" || key == "control" {
            self.held.clear();
        } else {
            self.held.remove(&key);
        }
        self.nav_modifiers_changed(ev.keystroke.modifiers, cx)
    }

    pub fn run(&mut self, command: Command, cx: &mut Context<Self>) {
        let panel_id = self.focused_panel.clone();
        let panel = panel_id.as_ref().and_then(|p| self.find_panel(p));
        match command {
            Command::FrameToggle => {
                if let Some(p) = &panel_id {
                    self.toggle_frame(p, cx);
                }
            }
            Command::NavigationBack => self.nav_back(cx),
            Command::NavigationForward => self.nav_forward(cx),
            Command::NavigationOverview => self.nav_toggle_overview(cx),
            Command::NavigationStepOut => self.nav_step_out(cx),
            Command::NavigationLeft => self.nav_neighbor(-1.0, 0.0, cx),
            Command::NavigationRight => self.nav_neighbor(1.0, 0.0, cx),
            Command::NavigationUp => self.nav_neighbor(0.0, -1.0, cx),
            Command::NavigationDown => self.nav_neighbor(0.0, 1.0, cx),
            Command::ShortcutsToggle => {
                self.nav.shortcuts = !self.nav.shortcuts;
                cx.notify();
            }
            Command::PanelNext | Command::PanelPrevious => {
                let order: Vec<PanelNode> = panels_of(self.doc.root.as_ref())
                    .into_iter()
                    .cloned()
                    .chain(self.doc.floating.iter().map(|f| f.panel.clone()))
                    .collect();
                if order.is_empty() {
                    return;
                }
                let i = order
                    .iter()
                    .position(|p| Some(&p.id) == panel_id.as_ref())
                    .map_or(0, usize::cast_signed);
                let step = if command == Command::PanelNext { 1 } else { -1 };
                let next = &order[(i + step).rem_euclid(order.len().cast_signed()) as usize];
                let selected = next.selected.clone();
                self.focus_view(&selected, cx);
                self.ensure_framed_visible(&selected, cx);
            }
            Command::TabNext | Command::TabPrevious => {
                let hovered = self.hovered_panel.as_ref().and_then(|p| self.find_panel(p));
                let Some(panel) = hovered.or(panel) else {
                    return;
                };
                let i = panel
                    .views
                    .iter()
                    .position(|v| *v == panel.selected)
                    .unwrap_or(0)
                    .cast_signed();
                let step = if command == Command::TabNext { 1 } else { -1 };
                let view =
                    panel.views[(i + step).rem_euclid(panel.views.len().cast_signed()) as usize].clone();
                self.select_and_focus(&view, cx);
            }
            Command::ViewReopen => self.reopen_closed(cx),
            Command::ViewClose => {
                if let Some(v) = self.focused_view.clone()
                    && self.rules_of(&v).is_closable()
                {
                    self.close_by_user(&v, cx);
                }
            }
            Command::PanelFloat => {
                if let Some(p) = &panel_id {
                    self.float(p, None, cx);
                }
            }
            Command::PanelHide => {
                if let Some(p) = &panel_id {
                    self.hide(p, None, cx);
                }
            }
        }
    }

    pub(crate) fn toggle_frame(&mut self, panel_id: &Id, cx: &mut Context<Self>) -> bool {
        if let Some(f) = self.doc.floating.iter().find(|f| &f.panel.id == panel_id) {
            if f.layer != FloatingLayer::Stage {
                return false;
            }
            self.nav_frame_stage(cx);
            return true;
        }
        self.nav_toggle(panel_id, cx)
    }

    pub(crate) fn step(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let now = self.now();
        let dt = self
            .last_frame
            .replace(now)
            .map_or(0.016, |p| (now - p) / 1000.0);
        let mut moving = false;

        if self.camera.moving() && !self.nav.gesture {
            if self.reduced() {
                self.camera.finish();
            } else {
                moving |= self.camera.step(dt);
            }
        }
        if self.tween.active() {
            moving |= self.tween.step(now, LAYOUT_MS);
        }
        if !self.tween.active() {
            self.settling.clear();
        }
        self.leaving.retain(|_, l| now - l.start < l.duration);
        moving |= !self.leaving.is_empty();
        self.entering.retain(|_, e| now - e.start < DOCK_RESTORE_MS);
        moving |= !self.entering.is_empty();

        self.step_drag(now, dt, cx);
        moving |= self.step_nav(now, cx);
        moving |= self.step_fx(now);
        moving |= self.drag_needs_frames();

        if moving {
            window.request_animation_frame();
        } else {
            self.animating = false;
            self.last_frame = None;
        }
    }
}
