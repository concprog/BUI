//! The gallery's window: a sidebar of every chart and a page of its examples. Shared by `main.rs`
//! (desktop and headless screenshots).

use blenderui::prelude::*;

use crate::registry::{CHARTS, Chart, Example, GROUPS, Layout, page};

fn card(example: &Example) -> Div {
    let (_, _, muted) = page();
    div()
        .class("flex flex-col rounded-lg border p-4 gap-1")
        .border_color(Rgba { a: 0.15, ..page().1 })
        .child(div().class("text-sm font-semibold").child(example.title))
        .child(div().class("text-xs mb-2").text_color(muted).child(example.description))
        .child((example.render)())
}

fn body(chart: &Chart) -> AnyElement {
    match chart.layout {
        Layout::Page => div().class("absolute inset-0").child((chart.examples[0].render)()).into_any_element(),
        Layout::Grid(columns) => div()
            .id("examples")
            .class("absolute inset-0 flex flex-col p-6 gap-4 overflow-y-scroll")
            .child(div().class("text-xl font-semibold").child(chart.name))
            .child(div().class("grid gap-4").grid_cols(columns).children(chart.examples.iter().map(card)))
            .into_any_element(),
    }
}

#[component]
fn gallery(start: usize, alone: bool) {
    let selected = use_signal(start);
    let chart = &CHARTS[selected.get()];
    let (background, foreground, muted) = page();
    if alone {
        return div().class("absolute inset-0").bg(background).text_color(foreground).child(body(chart)).into_any_element();
    }
    let groups = GROUPS.iter().map(|(group, label)| {
        div().class("flex flex-col gap-1 pb-3").child(div().class("text-xs px-2 py-1").text_color(muted).child(*label)).children(
            CHARTS.iter().enumerate().filter(|(_, entry)| entry.group == *group).map(|(index, entry)| {
                div()
                    .id(format!("chart-{index}"))
                    .class("px-2 py-1 rounded-md text-sm cursor-pointer")
                    .when(selected.get() == index, |el| el.bg(Rgba { a: 0.12, ..foreground }))
                    .on_click(move |_, _, _| selected.set(index))
                    .child(entry.name)
            }),
        )
    });
    div()
        .class("absolute inset-0 flex")
        .bg(background)
        .text_color(foreground)
        .child(
            div()
                .id("sidebar")
                .class("flex flex-col w-[220px] flex-none p-3 overflow-y-scroll border-r")
                .border_color(Rgba { a: 0.12, ..foreground })
                .child(div().class("font-semibold pb-3").child("bui-charts"))
                .children(groups),
        )
        .child(div().class("relative flex-1 min-w-0").child(body(chart)))
        .into_any_element()
}

pub struct GalleryApp {
    /// the chart shown first
    pub start: usize,
    /// only that chart's page, no sidebar (screenshots)
    pub alone: bool,
}

impl Render for GalleryApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (start, alone) = (self.start, self.alone);
        // the dashboards hold bui-panels workspaces, which take the window and app from here
        bui_panels::ui::scope(window, cx, || rsx! { <gallery start={start} alone={alone} /> })
    }
}
