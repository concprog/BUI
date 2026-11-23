//! Headless workspaces driven through blenderui's simulated input: the layout from
//! `layout.json`, a divider drag, a tab dragged to another panel, a panel menu action (mouse and
//! keyboard) and keyboard navigation.

use std::{
    cell::Cell,
    collections::HashMap,
    rc::Rc,
    time::Duration,
};

use blenderui::prelude::*;
use bui_panels::{
    LayoutDocument, MenuItem, Node, ViewType, Workspace, WorkspaceOptions, menu::row_id,
};

struct Label(String);

impl Render for Label {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().id(format!("body-{}", self.0)).size_full().p(px(8.0)).child(self.0.clone())
    }
}

fn layout_json() -> LayoutDocument {
    serde_json::from_str(include_str!("../layout.json")).expect("layout.json is a layout document")
}

/// A view type for every type the document uses.
fn types_for(doc: &LayoutDocument) -> HashMap<String, ViewType> {
    doc.views
        .values()
        .map(|record| {
            let ty = ViewType::new(record.ty.clone(), |handle, _, cx| {
                let id = handle.id.to_string();
                cx.new(|_| Label(id)).into()
            });
            (record.ty.clone(), ty)
        })
        .collect()
}

fn options(width: f32, height: f32) -> WindowOptions {
    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(Bounds {
            origin: point(px(0.0), px(0.0)),
            size: size(px(width), px(height)),
        })),
        ..Default::default()
    }
}

fn open(cx: &mut App, options: WorkspaceOptions, types: HashMap<String, ViewType>) -> WindowHandle<Workspace> {
    let win = cx
        .open_window(self::options(1200.0, 800.0), |window, cx| {
            cx.new(|cx| Workspace::new(options, types, window, cx))
        })
        .unwrap();
    settle(cx, win.id);
    win
}

/// Renders until the layout and its animations are done.
fn settle(cx: &mut App, id: WindowId) {
    for _ in 0..4 {
        cx.draw(id);
        cx.advance_clock(Duration::from_millis(100));
    }
    cx.draw(id);
}

fn bounds(cx: &mut App, id: WindowId, element: &str) -> Bounds<f32> {
    cx.element_bounds(id, element)
        .unwrap_or_else(|| panic!("no element #{element}"))
}

fn center(b: Bounds<f32>) -> (f32, f32) {
    (b.origin.x + b.size.width / 2.0, b.origin.y + b.size.height / 2.0)
}

fn layout_options() -> (WorkspaceOptions, HashMap<String, ViewType>) {
    let doc = layout_json();
    let types = types_for(&doc);
    (
        WorkspaceOptions {
            document: Some(doc),
            reduced_motion: true,
            ..Default::default()
        },
        types,
    )
}

fn panel_views(doc: &LayoutDocument, panel: &str) -> Vec<String> {
    fn find<'a>(node: &'a Node, id: &str) -> Option<&'a bui_panels::PanelNode> {
        match node {
            Node::Panel(p) if &*p.id == id => Some(p),
            Node::Split(s) => s.children.iter().find_map(|c| find(c, id)),
            Node::Stage(s) => s.child.as_deref().and_then(|c| find(c, id)),
            Node::Panel(_) => None,
        }
    }
    doc.root
        .as_ref()
        .and_then(|r| find(r, panel))
        .map(|p| p.views.iter().map(ToString::to_string).collect())
        .unwrap_or_default()
}

#[test]
fn layout_builds_from_layout_json() {
    Application::headless().run(|cx| {
        let (opts, types) = layout_options();
        let expected = layout_json();
        let win = open(cx, opts, types);
        for panel in ["left", "console-panel", "panel-e8ae13", "docs", "right", "tools"] {
            let b = bounds(cx, win.id, &format!("zopra-panel-{panel}"));
            assert!(b.size.width > 20.0 && b.size.height > 20.0, "{panel}: {b:?}");
        }
        // two tabs: a tab bar; the selected one shows its content
        bounds(cx, win.id, "zopra-tab-canvas");
        bounds(cx, win.id, "zopra-tab-search");
        bounds(cx, win.id, "body-search");
        assert!(cx.element_bounds(win.id, "body-canvas").is_none(), "only the selected tab renders");
        // the hidden panel stays hidden
        assert!(cx.element_bounds(win.id, "zopra-panel-panel-6e4e50").is_none());
        // left | stage | tools, in the document's proportions
        let (left, tools) = (bounds(cx, win.id, "zopra-panel-left"), bounds(cx, win.id, "zopra-panel-tools"));
        assert!(left.origin.x < tools.origin.x);
        assert!((left.size.width - tools.size.width).abs() < 4.0, "{left:?} {tools:?}");
        let doc = win.root().read(cx).document();
        assert_eq!(doc.views.keys().collect::<Vec<_>>(), expected.views.keys().collect::<Vec<_>>());
        assert_eq!(panel_views(&doc, "tools"), ["canvas", "search"]);
        assert_eq!(doc.hidden.len(), 1);
    });
}

#[test]
fn split_resizes_by_drag() {
    Application::headless().run(|cx| {
        let (opts, types) = layout_options();
        let win = open(cx, opts, types);
        let before = bounds(cx, win.id, "zopra-panel-left");
        let divider = bounds(cx, win.id, "zopra-div-split-1-0");
        let (x, y) = center(divider);
        cx.simulate_drag(win.id, (x, y), (x + 120.0, y));
        settle(cx, win.id);
        let after = bounds(cx, win.id, "zopra-panel-left");
        let grown = after.size.width - before.size.width;
        assert!((grown - 120.0).abs() < 6.0, "the left panel grew by {grown}");
        let Some(Node::Split(split)) = win.root().read(cx).document().root else {
            panic!("a split root");
        };
        let total: f64 = split.weights.iter().sum();
        assert!(split.weights[0] / total > 0.2, "the weights follow: {:?}", split.weights);
    });
}

#[test]
fn tab_moves_to_another_panel_by_drag() {
    Application::headless().run(|cx| {
        let (opts, types) = layout_options();
        let win = open(cx, opts, types);
        let tab = center(bounds(cx, win.id, "zopra-tab-search"));
        let target = center(bounds(cx, win.id, "zopra-panel-left"));
        cx.simulate_drag(win.id, tab, target);
        settle(cx, win.id);
        let doc = win.root().read(cx).document();
        assert_eq!(panel_views(&doc, "left"), ["files", "search"]);
        assert_eq!(panel_views(&doc, "tools"), ["canvas"]);
        // the moved tab is selected and its content shows in its new panel
        bounds(cx, win.id, "zopra-tab-search");
        let body = bounds(cx, win.id, "body-search");
        let left = bounds(cx, win.id, "zopra-panel-left");
        assert!(body.origin.x >= left.origin.x && body.origin.x < left.origin.x + left.size.width);
    });
}

#[test]
fn tabs_reorder_by_drag_within_their_bar() {
    Application::headless().run(|cx| {
        let (opts, types) = layout_options();
        let win = open(cx, opts, types);
        let canvas = bounds(cx, win.id, "zopra-tab-canvas");
        let search = bounds(cx, win.id, "zopra-tab-search");
        let from = center(canvas);
        cx.simulate_drag(win.id, from, (search.origin.x + search.size.width - 4.0, from.1));
        settle(cx, win.id);
        let doc = win.root().read(cx).document();
        assert_eq!(panel_views(&doc, "tools"), ["search", "canvas"]);
    });
}

#[test]
fn panel_menu_runs_its_actions() {
    Application::headless().run(|cx| {
        let (opts, mut types) = layout_options();
        let pinged = Rc::new(Cell::new(0));
        let ping = pinged.clone();
        let search = types.remove("zopra-view:search").unwrap().menu(move |_| {
            let ping = ping.clone();
            vec![MenuItem::new("Ping").id("ping").run(move |_, _| ping.set(ping.get() + 1)).into()]
        });
        types.insert("zopra-view:search".into(), search);
        let win = open(cx, opts, types);

        // the ⋯ button opens the menu; a click on a row runs it and closes the menu
        let button = center(bounds(cx, win.id, "zopra-menu-tools"));
        cx.simulate_click(win.id, button.0, button.1);
        settle(cx, win.id);
        let row = center(bounds(cx, win.id, &row_id(0, 0)));
        cx.simulate_click(win.id, row.0, row.1);
        settle(cx, win.id);
        assert_eq!(pinged.get(), 1);
        assert!(cx.element_bounds(win.id, "zopra-menu-0").is_none(), "the menu closed");

        // keyboard: Down highlights the first row, Return runs it
        cx.simulate_click(win.id, button.0, button.1);
        settle(cx, win.id);
        cx.simulate_key(win.id, "", "Down");
        cx.simulate_key(win.id, "", "Return");
        settle(cx, win.id);
        assert_eq!(pinged.get(), 2);

        // a built-in entry: Hide (the row after the app's, a separator and the built-ins before it)
        cx.simulate_click(win.id, button.0, button.1);
        settle(cx, win.id);
        let hide = (0..12)
            .map(|i| row_id(0, i))
            .filter_map(|id| cx.element_bounds(win.id, &id).map(|b| (id, b)))
            .nth(4)
            .expect("Ping, Maximize, Float, Move to, Hide");
        let at = center(hide.1);
        cx.simulate_click(win.id, at.0, at.1);
        settle(cx, win.id);
        let doc = win.root().read(cx).document();
        assert!(doc.hidden.iter().any(|h| &*h.panel.id == "tools"), "tools is hidden");
        assert!(cx.element_bounds(win.id, "zopra-panel-tools").is_none());
    });
}

#[test]
fn keyboard_moves_focus_between_panels_and_tabs() {
    Application::headless().run(|cx| {
        let (opts, types) = layout_options();
        let win = open(cx, opts, types);
        // focus a panel with the pointer: key events go to the focused element
        let files = center(bounds(cx, win.id, "body-files"));
        cx.simulate_click(win.id, files.0, files.1);
        settle(cx, win.id);
        assert_eq!(win.root().read(cx).focused_view().map(std::string::ToString::to_string).as_deref(), Some("files"));

        cx.simulate_key(win.id, "", "F6");
        settle(cx, win.id);
        let next = win.root().read(cx).focused_view().map(std::string::ToString::to_string);
        assert_eq!(next.as_deref(), Some("console"), "F6: the next panel");

        cx.simulate_key_with(win.id, "", "F6", false, true);
        settle(cx, win.id);
        let back = win.root().read(cx).focused_view().map(std::string::ToString::to_string);
        assert_eq!(back.as_deref(), Some("files"), "Shift+F6: the previous panel");

        // Ctrl+Tab: the next tab of the focused panel
        let search = center(bounds(cx, win.id, "body-search"));
        cx.simulate_click(win.id, search.0, search.1);
        settle(cx, win.id);
        cx.simulate_key_with(win.id, "", "Tab", true, false);
        settle(cx, win.id);
        let ws = win.root().read(cx);
        assert_eq!(ws.focused_view().map(std::string::ToString::to_string).as_deref(), Some("canvas"));
        assert_eq!(panel_views(&ws.document(), "tools"), ["canvas", "search"]);
    });
}

#[test]
fn panels_follow_the_pointer_while_dragging() {
    Application::headless().run(|cx| {
        let (opts, types) = layout_options();
        let win = open(cx, opts, types);
        let before = bounds(cx, win.id, "zopra-panel-left");
        let (x, y) = center(bounds(cx, win.id, "zopra-div-split-1-0"));
        cx.simulate_press(win.id, x, y);
        cx.simulate_mouse_move(win.id, x + 40.0, y);
        cx.simulate_mouse_move(win.id, x + 80.0, y);
        cx.draw(win.id);
        let mid = bounds(cx, win.id, "zopra-panel-left");
        assert!((mid.size.width - before.size.width - 80.0).abs() < 6.0, "mid-drag: {mid:?}");
        cx.simulate_release(win.id, x + 80.0, y);
        settle(cx, win.id);

        // a tab lifted out of its bar: a drop slot shows where it would land
        let tab = center(bounds(cx, win.id, "zopra-tab-search"));
        let target = center(bounds(cx, win.id, "zopra-panel-left"));
        cx.simulate_press(win.id, tab.0, tab.1);
        cx.simulate_mouse_move(win.id, tab.0 - 60.0, tab.1 + 80.0);
        cx.simulate_mouse_move(win.id, target.0, target.1);
        // a drop target takes once the pointer rests on it (150 ms of the workspace's own clock)
        std::thread::sleep(Duration::from_millis(200));
        settle(cx, win.id);
        assert!(cx.element_bounds(win.id, "__zopra-drop-slot").is_some(), "the drop slot shows");
        // Escape puts it back
        cx.simulate_key(win.id, "", "Escape");
        cx.simulate_release(win.id, target.0, target.1);
        settle(cx, win.id);
        let doc = win.root().read(cx).document();
        assert_eq!(panel_views(&doc, "tools"), ["canvas", "search"]);
        assert_eq!(panel_views(&doc, "left"), ["files"]);
    });
}
