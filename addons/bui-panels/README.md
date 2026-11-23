# bui-panels

Panel layouts for [BUI](https://github.com/ppmpreetham/BUI)'s Rust binding, `blenderui`: move
things around. A workspace is a tree of splits around a **stage** (the main area), with **panels**
of tabbed **views**, **floating** panels, and a layout document (JSON) that saves and restores it.

- Drag a tab to reorder it, onto another panel to join its tabs, to a panel's edge to split it,
  or out to float it. Drag dividers to resize (Shift snaps to quarters and thirds), double-click
  one to even it out; drag a floating panel's edges to resize it.
- Panel menus (the `⋯` button or a right-click on the tab bar): maximize, float / dock, move to
  another panel, split, hide, close, close others, and the app's own entries.
- Navigation: frame (maximize) a panel or a group, back / forward, overview, step out, move to
  the panel left / right / above / below; in free navigation, pan, zoom and swipe the camera.
- Keyboard shortcuts (`Keymap`, rebindable), close guards, badges, an undo toast for closed tabs,
  hiding and restoring panels, persistence to a file.
- Blender's icons (`IconName`), drawn from the kit's own SVGs.

It is its own layout model: it does not use BUI's screens and areas (`blenderui::screen`).

This is the BUI port of `zopra-panels` (GPUI-PANEL): the same model, layout JSON, keymap, view
types and markup, on blenderui instead of gpui / gpui-kit.

## Cargo

```toml
[dependencies]
bui-panels = { path = "../addons/bui-panels" } # inside a BUI checkout
blenderui = { path = "../bindings/rust/blenderui" }

[features]
pinch = [] # (default) a trackpad pinch zooms in free navigation
```

Building needs what `blenderui` needs (CMake, a C compiler). The build script reads Blender's
icons from the BUI checkout (`kit/intern/kit_icons_svg.c`); set `BUI_SOURCE_DIR` to point it at
another one.

## A workspace entity

View types are registered in Rust; the workspace is any view's content, or a window's root:

```rust
use std::collections::HashMap;
use blenderui::prelude::*;
use bui_panels::{Allow, ViewType, Workspace, WorkspaceOptions, create_document, layout::*};

struct Note(SharedString);
impl Render for Note {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().p(px(12.0)).child(self.0.clone())
    }
}

fn main() {
    Application::new().run(|cx| {
        cx.open_window(WindowOptions::default(), |window, cx| {
            let note = |title: &'static str| ViewType::new(title, move |_, _, cx| cx.new(|_| Note(title.into())).into());
            let types = HashMap::from([
                ("files".into(), note("Files").allow(Allow { stage: false, ..Default::default() })),
                ("editor".into(), note("Editor")),
            ]);
            let doc = create_document(
                Some(&row([panel([view("files")]).into(), stage(Some(panel([view("editor")]).into())).into()],
                          Some(vec![1.0, 3.0])).into()),
                vec![],
                None,
            ).unwrap();
            let options = WorkspaceOptions { default_layout: Some(doc), ..Default::default() };
            cx.new(|cx| Workspace::new(options, types, window, cx))
        }).unwrap();
    });
}
```

`workspace.update(cx, |ws, cx| ws.open("editor", OpenOptions::default(), cx))` opens views;
`ws.subscribe(|_, event, cx| .., cx)` hears `WorkspaceEvent`s (open, close, focus, navigate, change).

## A workspace in markup

```rust
use blenderui::prelude::*;
use bui_panels::prelude::*;

#[component]
fn Editor(path: SharedString) {
    let view = use_view();                       // this view: badge, title, close guard ...
    let (dirty, set_dirty) = use_state(false);
    view.guard_close(move |_| !dirty());
    rsx! { <div class="p-3" onClick={move || set_dirty(true)}>{path}</div> }
}

#[component]
fn Ide() {
    let ws = use_workspace();
    rsx! {
        <Workspace handle={ws} persist="layout.json" navigation={Navigation::Free}
                   on_change={|doc, _, _| println!("{} views", doc.views.len())}>
            <Row sizes={[1.0, 3.0]}>
                <Panel id="left">
                    <View id="files" title="Files" icon={IconName::Folder} keep_out_of_stage>"files"</View>
                </Panel>
                <Stage>
                    <Panel id="docs">
                        <View id="a.ts" title="a.ts" content={|| rsx! { <Editor path="a.ts" /> }} />
                    </Panel>
                </Stage>
            </Row>
        </Workspace>
    }
}

struct Root;
impl Render for Root {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        scope(window, cx, || app().child(rsx! { <Ide /> }))  // lends the window and app to <Workspace>
    }
}
```

The workspace fills its nearest positioned ancestor (the window when it is the root).

## Examples and tests

```sh
cargo run --example basic          # the entity, view types in Rust
cargo run --example ide            # markup, kept in ./layout.json
cargo run --example ide -- --screenshot ide.ppm   # headless, to a PPM
cargo test                         # model unit tests + headless workspaces (tests/)
```

The headless tests drive real windows through blenderui's simulated input: the layout from
`layout.json`, a divider drag, tabs dragged between and within panels, a panel menu (mouse and
keyboard), keyboard navigation, and the markup workspace with persistence. Parts carry element
ids: `zopra` (the workspace), `zopra-panel-<panel>`, `zopra-tab-<view>`, `zopra-close-<view>`,
`zopra-menu-<panel>` (the `⋯` button), `zopra-menu-<level>-<row>` (popup rows),
`zopra-div-<split>-<index>` (dividers).

## Coming from zopra-panels

Change the imports (`zopra_panels` → `bui_panels`, `gpui_kit::gpui` → `blenderui::prelude`) and
the app setup (`Application::new()`; markup renders inside `bui_panels::ui::scope`). Otherwise:

- `cx.subscribe(&workspace, ..)` is `workspace.update(cx, |ws, cx| ws.subscribe(.., cx))`.
- `ViewRef` methods take no `cx` (`view.set_badge(b)`, `view.close()`, `view.titlebar()`): changes
  are made when the workspace next renders, so they work in render code and in `use_effect`.
- A `<View>` needs an `id` (blenderui keeps `key` for hook state); `content` is a closure
  (`content={move || rsx! { <Editor/> }}`); `use_view()` works in `content` and `<Template>`s,
  not in a `<View>`'s children (blenderui builds children before their parent).
- `IconName` is Blender's icons (`IconName::Folder` is `FILE_FOLDER`, `Close` is `PANEL_CLOSE` ...).
- `Mod` is Ctrl (BUI reports Cmd as Ctrl on macOS); keys are BUI's keysyms (`Return`, `Left` ...),
  and a keymap's DOM names (`Enter`, `ArrowLeft`) still match them.

## License

MIT (see `LICENSE`). The icons are Blender's (GPL-2.0-or-later), and `blenderui` is
GPL-3.0-or-later.
