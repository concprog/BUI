//! A workspace in markup, kept in `layout.json`: `cargo run --example ide`.

use blenderui::prelude::*;
use bui_panels::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq)]
struct File {
    path: SharedString,
    name: SharedString,
    dirty: bool,
}

impl File {
    fn new(name: &'static str) -> Self {
        Self {
            path: format!("src/{name}").into(),
            name: name.into(),
            dirty: false,
        }
    }
}

type Action = fn(WorkspaceRef, &mut App);

#[derive(Serialize, Deserialize)]
struct EditorParams {
    path: String,
}

#[component]
fn Editor(path: SharedString) {
    let view = use_view();
    let (dirty, set_dirty) = use_state(false);
    view.guard_close(move |_| !dirty());
    use_effect!(move || view.set_badge(dirty().then_some(Badge::Dot)), [dirty()]);
    rsx! {
        <div class="size-full flex flex-col gap-2 p-3">
            <div class="font-mono">{path}</div>
            <div id="edit" class="cursor-pointer" onClick={move || set_dirty(true)}>"Type something"</div>
            <div id="save" class="cursor-pointer" onClick={move || set_dirty(false)}>"Save"</div>
        </div>
    }
}

#[component]
fn Note(text: SharedString) {
    rsx! { <div class="p-3">{text}</div> }
}

#[component]
fn Toolbar(ws: WorkspaceRef) {
    let actions: [(&'static str, Action); 10] = [
        ("Pop out console", |ws, cx| ws.float("console", cx)),
        ("Overview", |ws, cx| ws.frame(FrameTarget::All, cx)),
        ("Reopen tab", |ws, cx| ws.reopen_closed(cx)),
        ("Back", |ws, cx| ws.back(cx)),
        ("Forward", |ws, cx| ws.forward(cx)),
        ("Save framing", |ws, cx| {
            ws.save_framing("review", cx);
        }),
        ("Next tab", |ws, cx| ws.run(Command::TabNext, cx)),
        ("Hide outline", |ws, cx| ws.hide("outline", cx)),
        ("Restore right", |ws, cx| ws.restore("right", cx)),
        ("Open c.ts", |ws, cx| {
            ws.open("editor", EditorParams { path: "src/c.ts".into() }, cx)
                .placement(Placement::Stage)
                .reuse(Reuse::Params)
                .title("c.ts");
        }),
    ];
    rsx! {
        <div class="flex flex-wrap gap-2 p-2 text-xs" style="background: #141518; color: #e7e8eb">
            { for (ix, (label, action)) in actions.into_iter().enumerate() {
                <div key={ix} id={format!("action-{ix}")} class="px-2 py-1 rounded border cursor-pointer"
                    onClick={move |_, _, cx| action(ws, cx)}>
                    {label}
                </div>
            } }
        </div>
    }
}

#[component]
fn Ide() {
    let ws = use_workspace();
    let files = use_signal(vec![File::new("a.ts"), File::new("b.ts")]);
    let docs = files.get().into_iter().map(move |file| {
        let (dirty, path) = (file.dirty, file.path.clone());
        let closed = file.path.clone();
        rsx! {
            <View
                key={file.path.clone()}
                id={file.path}
                title={file.name}
                badge={dirty.then_some(Badge::Dot)}
                keep_out_of_sides
                can_close={move |_| !dirty}
                on_close={move |_, _| files.update(|files| files.retain(|f| f.path != closed))}
                content={move || rsx! { <Editor path={path.clone()} /> }}
            />
        }
    });

    rsx! {
        <div class="size-full flex flex-col">
            <Toolbar ws={ws} />
            <div class="flex-1 relative">
                <Workspace
                    handle={ws}
                    persist="layout.json"
                    persist_version="2"
                    navigation={Navigation::Free}
                    theme={ThemeMode::Dark}
                    floating={FloatingLayer::Overlay}
                    keep_pushed={false}
                    reduced_motion={false}
                    detail={Detail { size: 48.0, outline: 2 }}
                    tab_fill
                    tab_inset={6.0}
                    tab_undo_seconds={5.0}
                    keymap={Keymap::new()}
                    gesture_keys={default_gesture_keys()}
                    panel_menu={|entries, _ctx| entries}
                    hide_toward={|_id| None}
                    backdrop={rsx! { <div class="size-full" /> }}
                    stage_empty={rsx! { <Note text="Open a file" /> }}
                    empty={rsx! { <Note text="Nothing open" /> }}
                    chrome={rsx! { <div /> }}
                    on_change={|doc, _, _| println!("layout changed: {} views", doc.views.len())}
                    on_open={|view, _, _| println!("open {}", view.id)}
                    on_close={|view, _, _| println!("close {}", view.id)}
                    on_focus={|id, _, _| println!("focus {id:?}")}
                    on_navigate={|id, _, _| println!("navigate {id:?}")}
                >
                    <Template
                        name="editor"
                        content={|params: EditorParams, _, _| rsx! { <Editor path={params.path} /> }}
                    />

                    <Row sizes={[1.0, 3.0, 1.0]}>
                        <Panel id="left">
                            <View id="files" title="Files" icon={IconName::Folder} keep_out_of_stage>
                                <Note text="files" />
                            </View>
                            <View id="search" title="Search" icon={IconName::Search} keep_out_of_stage>
                                <Note text="search" />
                            </View>
                        </Panel>

                        <Stage>
                            <Panel id="docs">
                                {...docs}
                            </Panel>
                        </Stage>

                        <Column sizes={[2.0, 1.0]}>
                            <Panel id="right">
                                <View
                                    id="outline"
                                    title="Outline"
                                    icon={IconName::Outliner}
                                    singleton
                                    accessory={rsx! { <span class="text-[11px]">"3 symbols"</span> }}
                                    menu={|_info| vec![MenuItem::new("Refresh").run(|_, _| println!("refresh")).into()]}
                                >
                                    <Note text="outline" />
                                </View>
                                <View id="locked" title="Locked" closable={false}>
                                    <Note text="locked" />
                                </View>
                            </Panel>
                            <Panel id="tools">
                                <View
                                    id="map"
                                    title="Map"
                                    icon={IconName::Grid}
                                    gestures={GestureOwner::Exclusive}
                                    scaling={Scaling::Inert}
                                    min_size={(480.0, 320.0)}
                                >
                                    <Note text="map" />
                                </View>
                                <View id="canvas" title="Canvas" icon={IconName::Image} tabbar={TabbarMode::Overlay}>
                                    <Note text="canvas" />
                                </View>
                            </Panel>
                        </Column>
                    </Row>

                    <Float x={120.0} y={80.0} w={360.0} h={240.0} layer={FloatingLayer::Stage}>
                        <Panel id="console-panel">
                            <View id="console" title="Console" icon={IconName::Console} keep_out_of_floating={false} focus_on_open={false} open_from={Rect::UNIT}>
                                <Note text="console" />
                            </View>
                        </Panel>
                    </Float>
                </Workspace>
            </div>
        </div>
    }
}

struct IdeView;

impl Render for IdeView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // `app()`: a body as tall as the window, without the browser's margin
        scope(window, cx, || app().child(rsx! { <Ide /> }))
    }
}

fn main() {
    let screenshot = std::env::args().skip_while(|a| a != "--screenshot").nth(1);
    let app = if screenshot.is_some() { Application::headless() } else { Application::new() };
    app.run(move |cx: &mut App| {
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds {
                origin: point(px(0.0), px(0.0)),
                size: size(px(1280.0), px(800.0)),
            })),
            titlebar: Some(TitlebarOptions { title: Some("bui-panels: ide".into()) }),
            ..Default::default()
        };
        let window = cx.open_window(options, |_, cx| cx.new(|_| IdeView)).unwrap();
        if let Some(path) = screenshot {
            for _ in 0..3 {
                cx.draw(window.id);
                cx.advance_clock(std::time::Duration::from_millis(500));
            }
            let (w, h, pixels) = cx.draw(window.id);
            write_ppm(&path, w, h, &pixels);
            println!("wrote {path} ({w}x{h})");
        }
    });
}

/// A binary PPM of the `0x00RRGGBB` pixels `App::draw` returns.
fn write_ppm(path: &str, w: usize, h: usize, pixels: &[u32]) {
    let mut out = format!("P6\n{w} {h}\n255\n").into_bytes();
    for p in pixels {
        out.extend_from_slice(&[(p >> 16) as u8, (p >> 8) as u8, *p as u8]);
    }
    std::fs::write(path, out).expect("writing the screenshot");
}
