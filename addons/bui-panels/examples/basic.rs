//! A workspace entity with view types registered in Rust: `cargo run --example basic`.

use std::collections::HashMap;

use blenderui::prelude::{
    App, Application, Context, IntoElement, ParentElement as _, Render, SharedString, Styled,
    Window, WindowOptions, div, px,
};
use bui_panels::{
    Allow, GestureOwner, Navigation, OpenOptions, Placement, Scaling, TabbarMode, ViewType,
    Workspace, WorkspaceEvent, WorkspaceOptions, create_document, layout::{row, panel, view, stage},
};
use serde_json::json;

struct Note {
    label: SharedString,
}

impl Render for Note {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().p(px(12.0)).child(self.label.clone())
    }
}

fn view_types() -> HashMap<String, ViewType> {
    let note = |title: &'static str| {
        ViewType::new(title, move |handle, _window, cx| {
            cx.new(|_| Note {
                label: format!("{title} · {}", handle.id).into(),
            })
            .into()
        })
    };
    HashMap::from([
        (
            "files".into(),
            note("Files").allow(Allow {
                stage: false,
                ..Default::default()
            }),
        ),
        (
            "search".into(),
            note("Search").allow(Allow {
                stage: false,
                ..Default::default()
            }),
        ),
        (
            "editor".into(),
            note("Editor")
                .title_fn(|params| params["name"].as_str().unwrap_or("Editor").to_string())
                .placement(Placement::Stage)
                .allow(Allow {
                    side: false,
                    ..Default::default()
                }),
        ),
        (
            "outline".into(),
            note("Outline")
                .singleton()
                .accessory(|_handle, _window, _cx| {
                    div()
                        .text_size(px(11.0))
                        .child("3 symbols")
                        .into_any_element()
                }),
        ),
        ("locked".into(), note("Locked").closable(false)),
        (
            "map".into(),
            note("Map")
                .gestures(GestureOwner::Exclusive)
                .min_size(480.0, 320.0)
                .scaling(Scaling::Inert),
        ),
        ("canvas".into(), note("Canvas").tabbar(TabbarMode::Overlay)),
    ])
}

pub fn main() {
    Application::new().run(|cx: &mut App| {
        cx.open_window(WindowOptions::default(), |window, cx| {
            let doc = create_document(
                Some(
                    &row(
                        [
                            panel([view("files").id("files"), view("search").id("search")])
                                .id("left")
                                .into(),
                            stage(Some(
                                panel([
                                    view("editor").id("a").params(json!({ "name": "a.ts" })),
                                    view("editor").id("b").params(json!({ "name": "b.ts" })),
                                ])
                                .id("docs")
                                .into(),
                            ))
                            .into(),
                            panel([view("outline").id("outline")]).id("right").into(),
                        ],
                        Some(vec![1.0, 3.0, 1.0]),
                    )
                    .into(),
                ),
                vec![],
                None,
            )
            .expect("valid layout");

            let workspace = cx.new(|cx| {
                Workspace::new(
                    WorkspaceOptions {
                        default_layout: Some(doc),
                        navigation: Navigation::Free,
                        ..Default::default()
                    },
                    view_types(),
                    window,
                    cx,
                )
            });
            workspace.update(cx, |ws, cx| {
                ws.subscribe(
                    |_, event, _| match event {
                        WorkspaceEvent::Open(v) => println!("open {}", v.id),
                        WorkspaceEvent::Close(v) => println!("close {}", v.id),
                        _ => {}
                    },
                    cx,
                );
                ws.open(
                    "editor",
                    OpenOptions {
                        params: Some(json!({ "name": "new.ts" })),
                        ..Default::default()
                    },
                    cx,
                );
            });
            workspace
        })
        .unwrap();
        cx.activate(true);
    });
}
