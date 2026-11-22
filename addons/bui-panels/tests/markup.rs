//! A workspace in `rsx!` markup (the `ide` example's shape), headless: it restores `layout.json`
//! through `persist`, renders `<View>` children and `content` (with `use_view`), hands its events
//! to the callbacks, follows drags, and writes the layout back.

use std::{cell::Cell, path::PathBuf, time::Duration};

use blenderui::prelude::*;
use bui_panels::{LayoutDocument, Node, prelude::*};

thread_local! {
    static CHANGES: Cell<usize> = const { Cell::new(0) };
    static OPENS: Cell<usize> = const { Cell::new(0) };
}

#[component]
fn Doc(name: SharedString) {
    let view = use_view();
    let (dirty, set_dirty) = use_state(false);
    view.guard_close(move |_| !dirty());
    use_effect!(move || view.set_badge(dirty().then_some(Badge::Dot)), [dirty()]);
    rsx! {
        <div id={format!("doc-{}", view.id())} class="p-2">
            {name}
            <div id={format!("edit-{}", view.id())} onClick={move || set_dirty(true)}>"edit"</div>
        </div>
    }
}

#[component]
fn Shell(persist: String) {
    let ws = use_workspace();
    let docs = ["src/a.ts", "src/b.ts"].map(|path| {
        rsx! {
            <View key={path} id={path} title={path.trim_start_matches("src/")} keep_out_of_sides
                content={move || rsx! { <Doc name={path} /> }} />
        }
    });
    rsx! {
        <Workspace handle={ws} persist={persist} persist_version="2" reduced_motion
            on_change={|_, _, _| CHANGES.set(CHANGES.get() + 1)}
            on_open={|_, _, _| OPENS.set(OPENS.get() + 1)}>
            <Row sizes={[1.0, 3.0, 1.0]}>
                <Panel id="left">
                    <View id="files" title="Files" keep_out_of_stage><div id="inline-files">"files"</div></View>
                </Panel>
                <Stage><Panel id="docs">{...docs}</Panel></Stage>
                <Column sizes={[2.0, 1.0]}>
                    <Panel id="right">
                        <View id="outline" title="Outline">"outline"</View>
                        <View id="locked" title="Locked" closable={false}>"locked"</View>
                    </Panel>
                    <Panel id="tools">
                        <View id="search" title="Search"><div id="inline-search">"search"</div></View>
                        <View id="canvas" title="Canvas">"canvas"</View>
                    </Panel>
                </Column>
            </Row>
            <Float x={100.0} y={80.0} w={300.0} h={200.0} layer={FloatingLayer::Stage}>
                <Panel id="console-panel"><View id="console" title="Console">"console"</View></Panel>
            </Float>
        </Workspace>
    }
}

struct Host {
    persist: String,
}

impl Render for Host {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let persist = self.persist.clone();
        scope(window, cx, || rsx! { <Shell persist={persist} /> })
    }
}

fn settle(cx: &mut App, id: WindowId) {
    for _ in 0..4 {
        cx.draw(id);
        cx.advance_clock(Duration::from_millis(100));
    }
    cx.draw(id);
}

fn center(cx: &mut App, id: WindowId, element: &str) -> (f32, f32) {
    let b = cx.element_bounds(id, element).unwrap_or_else(|| panic!("no element #{element}"));
    (b.origin.x + b.size.width / 2.0, b.origin.y + b.size.height / 2.0)
}

fn views_of(doc: &LayoutDocument, panel: &str) -> Vec<String> {
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

fn persisted_copy(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("bui-panels-{name}-{}.json", std::process::id()));
    std::fs::copy(concat!(env!("CARGO_MANIFEST_DIR"), "/layout.json"), &path).expect("copying layout.json");
    path
}

fn open_host(cx: &mut App, persist: &std::path::Path) -> WindowId {
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(Bounds {
            origin: point(px(0.0), px(0.0)),
            size: size(px(1200.0), px(800.0)),
        })),
        ..Default::default()
    };
    let persist = persist.to_string_lossy().into_owned();
    let win = cx.open_window(options, |_, cx| cx.new(|_| Host { persist })).unwrap();
    settle(cx, win.id);
    win.id
}

#[test]
fn markup_restores_layout_json_and_follows_drags() {
    let path = persisted_copy("restore");
    Application::headless().run(|cx| {
        let win = open_host(cx, &path);
        // layout.json's arrangement, not the markup's: the stage is split into four panels
        for panel in ["left", "console-panel", "panel-e8ae13", "docs", "right", "tools"] {
            assert!(cx.element_bounds(win, &format!("zopra-panel-{panel}")).is_some(), "{panel}");
        }
        // children and content render; content finds its view with use_view()
        assert!(cx.element_bounds(win, "inline-search").is_some());
        assert!(cx.element_bounds(win, "doc-src/a.ts").is_some());
        assert!(cx.element_bounds(win, "doc-src/b.ts").is_some());

        // a view's content guards its close and badges its tab through its ViewRef
        let edit = center(cx, win, "edit-src/a.ts");
        cx.simulate_click(win, edit.0, edit.1);
        settle(cx, win);
        let close = center(cx, win, "zopra-close-src/a.ts");
        cx.simulate_click(win, close.0, close.1);
        settle(cx, win);
        assert!(cx.element_bounds(win, "zopra-tab-src/a.ts").is_some(), "the guard kept it open");

        // a drag: the callbacks hear the change, and the file follows
        let before = CHANGES.get();
        let tab = center(cx, win, "zopra-tab-search");
        let target = center(cx, win, "zopra-panel-left");
        cx.simulate_drag(win, tab, target);
        settle(cx, win);
        assert!(CHANGES.get() > before, "on_change ran");
        std::thread::sleep(Duration::from_millis(300));
        cx.run_pending();
        let saved: LayoutDocument = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        assert_eq!(views_of(&saved, "left"), ["files", "search"]);
        assert_eq!(saved.version.as_deref(), Some("2"));
    });
    std::fs::remove_file(path).ok();
}

#[test]
fn markup_builds_its_own_layout_without_a_file() {
    let path = std::env::temp_dir().join(format!("bui-panels-fresh-{}.json", std::process::id()));
    std::fs::remove_file(&path).ok();
    Application::headless().run(|cx| {
        let win = open_host(cx, &path);
        for panel in ["left", "docs", "right", "tools", "console-panel"] {
            assert!(cx.element_bounds(win, &format!("zopra-panel-{panel}")).is_some(), "{panel}");
        }
        assert!(cx.element_bounds(win, "zopra-panel-panel-e8ae13").is_none());
        assert!(cx.element_bounds(win, "inline-files").is_some());
        assert!(OPENS.get() == 0, "the markup's own views are there from the start, not opened");
    });
    std::fs::remove_file(path).ok();
}
