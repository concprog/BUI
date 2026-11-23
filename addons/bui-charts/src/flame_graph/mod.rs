mod call_tree;
mod colors;
mod graph;
mod header;
mod model;
mod options;
mod profile;
mod search;
mod table;
mod tree;

use std::collections::BTreeSet;
use std::mem;

use blenderui::element::{AnyElement, Div, FluentBuilder, IntoElement, ParentElement};
use blenderui::hooks::use_signal;
use blenderui::html::div;
use blenderui::{SharedString, Styled, canvas, px};

pub use options::{FlameColors, FrameAction, LabelAlign, Pane, PaneLayout, SampleUnit, Selected};
pub use profile::{Profile, ProfileData};

use crate::component;
use crate::data::Revision;
use call_tree::call_tree_pane;
use graph::flame_pane;
use header::{SPLIT_WIDTH, flame_header};
use model::{Cache, CallState, Column, Flame, Format, Inputs, Sort, Units};
use table::top_table;

const STACKED_MIN: f32 = 160.;

#[component]
pub fn flame_graph<D: ProfileData, F: Fn(f64) -> SharedString + 'static>(
    data: D,
    unit: Option<SampleUnit>,
    format: Option<F>,
    decimals: Option<usize>,
    layout: Option<PaneLayout>,
    header: Option<bool>,
    color_scheme: Option<FlameColors>,
    text_align: Option<LabelAlign>,
    collapsing: Option<bool>,
    vertical: Option<bool>,
    keep_focus_on_data_change: Option<bool>,
    actions: Option<&'static [FrameAction]>,
    class: Option<SharedString>,
) -> AnyElement {
    let slot = use_signal(Profile::default());
    let (profile, revision) = data.into_signal(slot);
    let diff = profile.with(Profile::is_diff);
    let format_slot = use_signal(Format(None));
    if let Some(format) = format {
        format_slot.update_untracked(|slot| slot.0 = Some(Box::new(format)));
    }
    let flame = Flame {
        profile,
        cache: use_signal(Cache::default()),
        units: Units { unit: unit.unwrap_or_default(), decimals, format: format_slot },
        colors: use_signal(color_scheme.unwrap_or_else(|| FlameColors::default_for(diff))),
        align: use_signal(text_align.unwrap_or_default()),
        layout: use_signal(layout.unwrap_or_default()),
        search: use_signal(SharedString::default()),
        highlight: use_signal(None::<u32>),
        focus: use_signal(None::<model::Focus>),
        sandwich: use_signal(None::<u32>),
        groups: use_signal(BTreeSet::<u32>::new()),
        hover: use_signal(None::<model::Pointer>),
        menu: use_signal(None::<model::Pointer>),
        table_sort: use_signal(Sort { column: Column::Own, desc: true }),
        calls: use_signal(CallState::default()),
        call_query: use_signal(SharedString::default()),
        call_match: use_signal(0usize),
        call_reveal: use_signal(false),
        scroll_to: use_signal(None::<u32>),
        width: use_signal(0f32),
        collapsing: collapsing.unwrap_or(true),
        vertical: vertical.unwrap_or(false),
        header: header.unwrap_or(true),
        actions: actions.unwrap_or(&[]),
    };
    refresh(flame, revision, keep_focus_on_data_change.unwrap_or(false));

    let current = flame.layout.get();
    let narrow = !flame.vertical && flame.width.get() < SPLIT_WIDTH;
    let (primary, secondary) = current.panes();
    let secondary = secondary.filter(|_| !narrow);
    let first = pane(primary, flame);
    let second = secondary.map(|kind| pane(kind, flame));
    let header = flame.header.then(|| flame_header(flame));
    let theme = crate::ui::theme();
    let min = if flame.vertical { STACKED_MIN } else { SPLIT_WIDTH / 2. };
    let body = match second {
        None => first,
        Some(second) => {
            let half = |pane: Div| {
                div()
                    .class("flex flex-1 min-w-0 min-h-0")
                    .map(|el| if flame.vertical { el.min_h(px(min)) } else { el.min_w(px(min)) })
                    .child(pane)
            };
            div()
                .id("flame-split")
                .class("flex flex-1 min-w-0 min-h-0")
                .map(|el| if flame.vertical { el.flex_col() } else { el.flex_row() })
                .child(half(first))
                .child(div().class("flex-none").map(|el| if flame.vertical { el.h(px(1.)) } else { el.w(px(1.)) }).bg(theme.border))
                .child(half(second))
        }
    };
    let width = flame.width;
    div()
        .id("flame-graph")
        .class("relative flex flex-col size-full min-w-0 min-h-0")
        .bg(theme.background)
        .when_some(class, Div::class)
        .child(
            canvas(move |bounds, _| {
                if (width.get() - bounds.size.width).abs() > 0.5 {
                    width.set(bounds.size.width);
                }
            })
            .class("absolute top-0 left-0 w-full h-[1px] pointer-events-none"),
        )
        .child(header)
        .child(div().class("flex flex-1 min-h-0").child(body))
        .into_any_element()
}

fn pane(kind: Pane, flame: Flame) -> Div {
    match kind {
        Pane::FlameGraph => flame_pane(flame),
        Pane::TopTable => top_table(flame),
        Pane::CallTree => call_tree_pane(flame),
    }
}

fn refresh(flame: Flame, revision: Option<Revision>, keep_focus: bool) {
    if flame.cache.with(|cache| cache.is_stale(revision)) && !keep_focus {
        flame.focus.update_untracked(|focus| *focus = None);
        flame.sandwich.update_untracked(|sandwich| *sandwich = None);
        flame.groups.update_untracked(BTreeSet::clear);
        flame.hover.update_untracked(|hover| *hover = None);
        flame.menu.update_untracked(|menu| *menu = None);
        flame.calls.update_untracked(|calls| *calls = CallState::default());
    }
    let mut cache = Cache::default();
    flame.cache.update_untracked(|slot| mem::swap(slot, &mut cache));
    let light = !crate::ui::theme().dark;
    let (sandwich, table_sort) = (flame.sandwich.get(), flame.table_sort.get());
    flame.profile.with(|profile| {
        flame.calls.with(|calls| {
            flame.groups.with(|groups| {
                flame.search.with(|query| {
                    flame.call_query.with(|call_query| {
                        let inputs = Inputs {
                            profile,
                            revision,
                            collapsing: flame.collapsing,
                            groups,
                            query,
                            sandwich,
                            light,
                            table_sort,
                            calls,
                            call_query,
                        };
                        cache.refresh(&inputs);
                    });
                });
            });
        });
    });
    flame.cache.update_untracked(|slot| *slot = cache);
}
