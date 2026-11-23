use std::collections::BTreeSet;
use std::fmt::Write as _;

use blenderui::element::{Div, ElementId, FluentBuilder, IntoElement, ParentElement};
use blenderui::hooks::{Signal, use_signal};
use blenderui::html::div;
use blenderui::style::TextAlign;
use blenderui::{App, ClickEvent, Hsla, MouseButton, MouseDownEvent, MouseMoveEvent, Painter, SharedString, Styled, Window, canvas, px, rgb};

use super::colors::{COLOR_BLIND, diff_color, lighten, value_color};
use super::model::{Cache, Flame, Focus, Labeled, Pointer, ROW_HEIGHT, Which, change, percent, percent_text};
use super::options::{FlameColors, LabelAlign, Selected};
use super::profile::Profile;
use super::tree::NONE;
use crate::color::mix;
use crate::format::{grouped, short};
use crate::geom::{Bounds, Point, point, size};
use crate::icon::{IconName, icon};
use crate::widgets::{Ui, button, pill, scroll_area, ui};
use crate::{ChartTheme, text};

const MUTE_THRESHOLD: f32 = 10.;
const HIDE_THRESHOLD: f32 = 0.5;
const LABEL_THRESHOLD: f32 = 20.;
const BORDER: f32 = 0.5;
const TEXT_PADDING: f32 = 4.;
const TEXT_SIZE: f32 = 12.;
const STRIP_WIDTH: f32 = 3.;
const STRIP_MARGIN: f32 = 4.;
const STRIP_TEXT_OFFSET: f32 = 2.;
const TEXT_COLOR: u32 = 0x22_2222;
const STRIP_COLOR: u32 = 0x66_6666;
const ANCESTOR_LIGHTEN: f32 = 0.15;
/// Steps the diff legend's two ramps are drawn in.
const LEGEND_STEPS: usize = 12;

impl Which {
    pub(crate) const fn labeled(self, cache: &Cache) -> &Labeled {
        match self {
            Self::Main => &cache.main,
            Self::Callers => &cache.flame.callers,
            Self::Callees => &cache.flame.callees,
        }
    }

    const fn slot(self) -> usize {
        match self {
            Self::Main => 0,
            Self::Callers => 1,
            Self::Callees => 2,
        }
    }
}

pub(crate) const fn effective(scheme: FlameColors, diff: bool) -> FlameColors {
    if scheme.is_diff() == diff { scheme } else { FlameColors::default_for(diff) }
}

/// Where each canvas of the pane last painted (`Which::slot`).
type Areas = Signal<[Bounds; 3]>;

#[derive(Clone, Copy)]
struct Viewport {
    which: Which,
    bounds: Bounds,
    range: (f64, f64),
    total: f64,
    per_tick: f64,
    focus_level: u32,
}

impl Viewport {
    fn new(which: Which, bounds: Bounds, flame: Flame, cache: &Cache) -> Self {
        let tree = &which.labeled(cache).tree;
        let total = tree.total();
        let focused = flame
            .focus
            .get()
            .filter(|focus| focus.which == which)
            .and_then(|focus| tree.nodes.get(focus.node as usize));
        let range = focused.map_or((0., 1.), |node| (node.start / total, (node.start + node.value) / total));
        let width = f64::from(bounds.size.width);
        let per_tick = width / total / (range.1 - range.0);
        Self { which, bounds, range, total, per_tick, focus_level: focused.map_or(0, |node| node.level) }
    }

    fn x(&self, start: f64) -> f32 {
        self.bounds.left() + ((start - self.total * self.range.0) * self.per_tick) as f32
    }

    fn hit(&self, position: Point, flame: Flame, cache: &Cache) -> Option<u32> {
        let offset = position - self.bounds.origin;
        if offset.y < 0. || offset.x < 0. || offset.x > self.bounds.size.width || !self.total.is_finite() || self.total <= 0.
        {
            return None;
        }
        let row = (offset.y / ROW_HEIGHT) as u32;
        let ticks = self.total.mul_add(self.range.0, f64::from(offset.x) / self.per_tick);
        let tree = &self.which.labeled(cache).tree;
        flame.groups.with(|groups| {
            tree.hit(row, ticks, |id| match self.which {
                Which::Main => (!cache.hidden(id, flame.collapsing, groups)).then(|| cache.rows[id as usize]),
                Which::Callers | Which::Callees => Some(tree.node(id).level),
            })
        })
    }
}

struct Label {
    text: SharedString,
    origin: Point,
    align: TextAlign,
    clip: Bounds,
}

struct Step {
    next: u32,
    label: Option<Label>,
}

/// What a bar is drawn from: the derived trees, the profile and the pane's settings.
struct Scene<'a> {
    view: Viewport,
    flame: Flame,
    ui: Ui,
    cache: &'a Cache,
    profile: &'a Profile,
    groups: &'a BTreeSet<u32>,
    scheme: FlameColors,
    align: LabelAlign,
}

fn paint(bounds: Bounds, which: Which, flame: Flame, areas: Areas, ui: Ui, painter: &mut Painter) {
    areas.update_untracked(|areas| areas[which.slot()] = bounds);
    let scheme = flame.colors.get();
    let align = flame.align.get();
    flame.profile.with(|profile| {
        flame.cache.with(|cache| {
            flame.groups.with(|groups| {
                let view = Viewport::new(which, bounds, flame, cache);
                if view.total <= 0. || !view.per_tick.is_finite() {
                    return;
                }
                let scene = Scene { view, flame, ui, cache, profile, groups, scheme, align };
                let len = which.labeled(cache).tree.nodes.len() as u32;
                let mut id = 0;
                while id < len {
                    let step = scene.bar(id, painter);
                    if let Some(label) = step.label {
                        painter.with_clip(label.clip.into(), |painter| {
                            text::draw(painter, &label.text, label.origin, label.align, TEXT_SIZE, rgb(TEXT_COLOR));
                        });
                    }
                    id = step.next;
                }
            });
        });
    });
}

impl Scene<'_> {
    fn bar(&self, id: u32, painter: &mut Painter) -> Step {
        let Self { view, flame, cache, profile, groups, .. } = *self;
        let labeled = view.which.labeled(cache);
        let node = *labeled.tree.node(id);
        let main = view.which == Which::Main;
        if main && cache.hidden(id, flame.collapsing, groups) {
            return Step { next: id + 1, label: None };
        }
        let width = (node.value * view.per_tick) as f32;
        let x = view.x(node.start);
        if width < HIDE_THRESHOLD || x > view.bounds.right() || x + width < view.bounds.left() {
            return Step { next: node.end, label: None };
        }
        let muted = width <= MUTE_THRESHOLD;
        let inner = width - if muted { 0. } else { BORDER * 2. };
        let row = if main { cache.rows[id as usize] } else { node.level };
        let y = view.bounds.top() + row as f32 * ROW_HEIGHT;
        let rect = Bounds::new(point(x + BORDER, y), size(inner, ROW_HEIGHT - 1.));
        painter.fill(rect.into(), self.color(&node, muted));

        let head = if main && flame.collapsing { cache.heads[id as usize] } else { NONE };
        let collapsed = head == id && !groups.contains(&head);
        if head != NONE {
            let tail = labeled.tree.children(id).next().is_none_or(|child| cache.heads[child as usize] != head);
            strip(rect, id == head, collapsed, tail, painter);
        }
        if muted || inner < LABEL_THRESHOLD {
            return Step { next: id + 1, label: None };
        }
        let full = labeled.text(id, profile, flame.units);
        let text = if collapsed { format!("({}) {full}", cache.group_size(head)).into() } else { full.clone() };
        let strip_offset = if head == NONE { 0. } else { STRIP_MARGIN + STRIP_TEXT_OFFSET + STRIP_WIDTH };
        let space = inner - TEXT_PADDING - strip_offset;
        let text_y = y + (ROW_HEIGHT - TEXT_SIZE) / 2.;
        let fits = text::measure(&text, TEXT_SIZE) <= space;
        let label = if !fits && self.align == LabelAlign::Right {
            Label {
                text: profile.label(node.label).clone(),
                origin: point(rect.right() - TEXT_PADDING, text_y),
                align: TextAlign::Right,
                clip: rect,
            }
        } else {
            let left = rect.left().max(view.bounds.left()) + TEXT_PADDING + strip_offset;
            Label { text, origin: point(left, text_y), align: TextAlign::Left, clip: rect }
        };
        Step { next: id + 1, label: Some(label) }
    }

    fn color(&self, node: &super::tree::Node, muted: bool) -> Hsla {
        let Self { view, cache, ui, .. } = *self;
        let searching = cache.searching();
        if muted && !searching {
            return ui.mute_bar;
        }
        let scheme = effective(self.scheme, self.profile.is_diff());
        let base = match scheme {
            FlameColors::ByValue => value_color(node.value, view.total, view.range),
            FlameColors::ByPackage => cache.package[node.label as usize],
            FlameColors::Diff | FlameColors::DiffColorBlind => {
                let root = cache.main.tree.nodes.first().copied().unwrap_or_default();
                diff_color(node.value - node.right, node.right, (root.value - root.right, root.right), diff_scale(scheme))
            }
        };
        if searching {
            return if cache.matched(node.label) { base } else { ui.mute_bar };
        }
        if node.level < view.focus_level && view.which == Which::Main { lighten(base, ANCESTOR_LIGHTEN) } else { base }
    }
}

fn strip(rect: Bounds, is_head: bool, collapsed: bool, tail: bool, painter: &mut Painter) {
    let height = rect.size.height;
    let (top, span) = match (collapsed, is_head, tail) {
        (true, _, _) => (height / 4., height / 2.),
        (false, true, _) => (height / 2., height / 2.),
        (false, false, true) => (0., height / 2.),
        (false, false, false) => (0., height),
    };
    let origin = point(rect.left() + STRIP_MARGIN, rect.top() + top);
    painter.fill(Bounds::new(origin, size(STRIP_WIDTH, span)).into(), rgb(STRIP_COLOR));
}

pub(crate) fn diff_scale(scheme: FlameColors) -> [Hsla; 3] {
    if scheme == FlameColors::DiffColorBlind {
        COLOR_BLIND.map(rgb)
    } else {
        let colors = ChartTheme::colors();
        [colors.good, colors.neutral, colors.bad]
    }
}

/// The bar under the pointer, if any.
fn hit_at(which: Which, at: Point, flame: Flame, areas: Areas) -> Option<u32> {
    let bounds = areas.get()[which.slot()];
    flame.cache.with(|cache| Viewport::new(which, bounds, flame, cache).hit(at, flame, cache))
}

fn on_move(which: Which, event: &MouseMoveEvent, flame: Flame, areas: Areas) {
    let at = Point::from(event.position);
    let current = flame.hover.get();
    match hit_at(which, at, flame, areas) {
        Some(node) if current.is_none_or(|hover| hover.which != which || hover.node != node) => {
            flame.hover.set(Some(Pointer { which, node, at }));
        }
        None if current.is_some_and(|hover| hover.which == which) => flame.hover.set(None),
        _ => {}
    }
}

fn on_down(which: Which, event: &MouseDownEvent, flame: Flame, areas: Areas) {
    let at = Point::from(event.position);
    if let Some(node) = hit_at(which, at, flame, areas) {
        flame.hover.set(None);
        flame.menu.set(Some(Pointer { which, node, at }));
    }
}

fn graph_canvas(which: Which, rows: u32, flame: Flame, areas: Areas, ui: Ui) -> Div {
    canvas(move |bounds, painter| paint(bounds.into(), which, flame, areas, ui, painter))
        .id(match which {
            Which::Main => "flame-canvas",
            Which::Callers => "flame-callers",
            Which::Callees => "flame-callees",
        })
        .class("w-full flex-none")
        .h(px(rows as f32 * ROW_HEIGHT))
        .on_mouse_move(move |event, _, _| on_move(which, event, flame, areas))
        .on_mouse_down(MouseButton::Left, move |event, _, _| on_down(which, event, flame, areas))
        .on_hover(move |inside, _, _| {
            if !inside && flame.hover.get().is_some_and(|hover| hover.which == which) {
                flame.hover.set(None);
            }
        })
}

struct Summary {
    total: f64,
    depth: u32,
    callers: u32,
    callees: u32,
    sandwich: Option<SharedString>,
    focus: Option<f64>,
    groups: bool,
    diff: bool,
}

fn summary(flame: Flame) -> Summary {
    let (focus, sandwich) = (flame.focus.get(), flame.sandwich.get());
    flame.profile.with(|profile| {
        flame.cache.with(|cache| {
            let total = cache.main.tree.total();
            let focus = focus.and_then(|focus| {
                let tree = &focus.which.labeled(cache).tree;
                tree.nodes.get(focus.node as usize).map(|node| percent(node.value, tree.total()))
            });
            Summary {
                total,
                depth: cache.depth,
                callers: cache.flame.callers.tree.depth,
                callees: cache.flame.callees.tree.depth,
                sandwich: sandwich.map(|label| {
                    let name = profile.label(label);
                    SharedString::from(name.rsplit('/').next().unwrap_or(name).to_owned())
                }),
                focus,
                groups: flame.collapsing && cache.heads.iter().any(|head| *head != NONE),
                diff: profile.is_diff(),
            }
        })
    })
}

/// The flame graph pane: the summary line, the color scheme, and the bars (or the sandwich of
/// callers over callees), with a tooltip and a menu per bar.
pub(crate) fn flame_pane(flame: Flame) -> Div {
    let ui = ui();
    let areas: Areas = use_signal([Bounds::default(); 3]);
    let info = summary(flame);
    let units = flame.units;
    let meta: SharedString =
        format!("{} | {} samples ({})", units.text(info.total), short(info.total), units.unit.title()).into();
    let scheme = effective(flame.colors.get(), info.diff);
    let legend = scheme.is_diff().then(|| diff_legend(diff_scale(scheme), ui));
    let choices = FlameColors::choices(info.diff);
    let focus = info
        .focus
        .map(|share| pill("clear-focus", format!("{share}% of total"), ui, move |_, _, _| flame.focus.set(None)));
    let sandwiched = info.sandwich.is_some();
    let sandwich = info.sandwich.map(|label| pill("clear-sandwich", label, ui, move |_, _, _| flame.sandwich.set(None)));
    let grouping = info.groups.then(|| {
        div()
            .class("flex gap-1")
            .child(button("expand-groups", "Expand all", false, ui, move |_, _, _| expand_all(flame)))
            .child(button("collapse-groups", "Collapse all", false, ui, move |_, _, _| flame.groups.set(BTreeSet::new())))
    });
    let body = if sandwiched {
        let caption = |text: &'static str, arrow: IconName| {
            div().class("flex items-center gap-1 text-xs py-1").text_color(ui.muted).child(text).child(icon(arrow, ui.muted).size(12.))
        };
        div()
            .class("flex flex-col")
            .child(caption("Callers", IconName::ArrowDown))
            .child(graph_canvas(Which::Callers, info.callers, flame, areas, ui))
            .child(caption("Callees", IconName::ArrowUp))
            .child(graph_canvas(Which::Callees, info.callees, flame, areas, ui))
    } else {
        graph_canvas(Which::Main, info.depth, flame, areas, ui)
    };
    let menu_open = flame.menu.get();
    let tooltip = menu_open.is_none().then(|| flame.hover.get()).flatten().and_then(|hover| tooltip(hover, flame, ui));
    let menu = menu_open.and_then(|target| menu(target, flame, ui));

    div()
        .class("flex flex-col size-full min-h-0 gap-2 p-2")
        .text_color(ui.text)
        .child(
            div()
                .class("flex flex-wrap items-center gap-2 text-xs")
                .child(div().text_color(ui.muted).child(meta))
                .child(focus)
                .child(sandwich)
                .child(div().class("flex-1"))
                .child(grouping)
                .children(choices.into_iter().enumerate().map(|(index, choice)| {
                    button(format!("scheme-{index}"), choice.label(), choice == scheme, ui, move |_, _, _| {
                        flame.colors.set(choice);
                    })
                })),
        )
        .child(legend)
        .child(scroll_area("flame-scroll", body).on_scroll_wheel(move |_, _, _| {
            if flame.hover.get().is_some() {
                flame.hover.set(None);
            }
        }))
        .child(tooltip)
        .child(menu)
}

fn expand_all(flame: Flame) {
    let heads: BTreeSet<u32> = flame.cache.with(|cache| cache.heads.iter().copied().filter(|head| *head != NONE).collect());
    flame.groups.set(heads);
}

/// The diff scale from removed to added: two ramps drawn in steps.
fn diff_legend(scale: [Hsla; 3], ui: Ui) -> Div {
    let ramp = |from: Hsla, to: Hsla| {
        div().class("flex flex-1 h-full").children((0..LEGEND_STEPS).map(move |step| {
            div().class("flex-1 h-full").bg(mix(from, to, (step as f32 + 0.5) / LEGEND_STEPS as f32))
        }))
    };
    div()
        .class("flex items-center gap-2 text-xs")
        .text_color(ui.muted)
        .child("-100% (removed)")
        .child(div().class("flex flex-1 h-2 rounded-full overflow-hidden").child(ramp(scale[0], scale[1])).child(ramp(scale[1], scale[2])))
        .child("+100% (added)")
}

/// A card at a window position (the pointer's), over everything.
fn floating(at: Point) -> Div {
    div().class("fixed rounded-md border shadow-md text-xs").left(px(at.x)).top(px(at.y))
}

fn tooltip(hover: Pointer, flame: Flame, ui: Ui) -> Option<Div> {
    let groups_hold = flame.groups.with(|groups| groups.contains(&hover.node));
    let (title, rows) = flame.profile.with(|profile| {
        flame.cache.with(|cache| {
            let tree = &hover.which.labeled(cache).tree;
            let node = *tree.nodes.get(hover.node as usize)?;
            let (own, _) = tree.self_values(hover.node, profile);
            let total = tree.total();
            let units = flame.units;
            let mut title = profile.label(node.label).to_string();
            if hover.which == Which::Main && flame.collapsing && cache.heads[hover.node as usize] == hover.node && !groups_hold {
                let members = cache.group_size(hover.node) - 1;
                let _ = write!(title, " and {members} similar items");
            }
            let rows: [[SharedString; 4]; 4] = if profile.is_diff() {
                let root = cache.main.tree.nodes.first().copied().unwrap_or_default();
                let (left, right) = (node.value - node.right, node.right);
                let (total_left, total_right) = (root.value - root.right, root.right);
                let (before, after) = (percent(left, total_left), percent(right, total_right));
                [
                    ["".into(), "Baseline".into(), "Comparison".into(), "Diff".into()],
                    ["% of total".into(), percent_text(before), percent_text(after), percent_text(change(before, after))],
                    ["Value".into(), units.text(left), units.text(right), units.text(right - left)],
                    ["Samples".into(), short(left), short(right), short(right - left)],
                ]
            } else {
                [
                    [units.unit.title().into(), "".into(), "".into(), "".into()],
                    ["Total".into(), units.text(node.value), percent_text(percent(node.value, total)), "".into()],
                    ["Self".into(), units.text(own), percent_text(percent(own, total)), "".into()],
                    ["Samples".into(), grouped(node.value), "".into(), "".into()],
                ]
            };
            Some((SharedString::from(title), rows))
        })
    })?;
    // muted text on the popover's own background (the theme's muted text is for the page)
    let muted = crate::ui::theme().popover_muted;
    Some(
        floating(hover.at + point(12., 12.))
            .class("flex flex-col gap-1 p-2 max-w-[420px] pointer-events-none")
            .bg(ui.popover)
            .text_color(ui.popover_text)
            .border_color(ui.border)
            .child(div().class("font-semibold").child(title))
            .children(rows.into_iter().map(|row| {
                div().class("flex gap-3").children(row.into_iter().enumerate().map(|(index, cell)| {
                    div().class("w-[96px] truncate").when(index == 0, |el| el.text_color(muted)).child(cell)
                }))
            })),
    )
}

fn menu_item(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    ui: Ui,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> Div {
    div()
        .id(id)
        .class("px-3 py-1 rounded-sm cursor-pointer")
        .hover(|style| style.bg(ui.accent))
        .on_click(on_click)
        .child(label.into())
}

fn menu(target: Pointer, flame: Flame, ui: Ui) -> Option<impl IntoElement> {
    let (label, node, own) = flame.profile.with(|profile| {
        flame.cache.with(|cache| {
            let tree = &target.which.labeled(cache).tree;
            let node = *tree.nodes.get(target.node as usize)?;
            let (own, _) = tree.self_values(target.node, profile);
            Some((profile.label(node.label).clone(), node, own))
        })
    })?;
    let head = if target.which == Which::Main && flame.collapsing {
        flame.cache.with(|cache| cache.heads[target.node as usize])
    } else {
        NONE
    };
    let (expanded, any_expanded, all_expanded) = flame.groups.with(|groups| {
        let all = flame.cache.with(|cache| cache.heads.iter().filter(|head| **head != NONE).all(|head| groups.contains(head)));
        (groups.contains(&head), !groups.is_empty(), all)
    });
    let close = move || flame.menu.set(None);
    let (copy, sandwich_label) = (label.clone(), node.label);
    let level = node.level;
    let card = floating(target.at + point(10., 0.))
        .id("flame-menu")
        .class("flex flex-col py-1 min-w-[180px]")
        .bg(ui.popover)
        .text_color(ui.popover_text)
        .border_color(ui.border)
        .child(menu_item("focus", "Focus block", ui, move |_, _, _| {
            flame.focus.set(Some(Focus { which: target.which, node: target.node }));
            close();
        }))
        .child(menu_item("copy", "Copy function name", ui, move |_, window, _| {
            window.write_clipboard(&copy);
            close();
        }))
        .child(menu_item("sandwich", "Sandwich view", ui, move |_, _, _| {
            flame.focus.set(None);
            flame.sandwich.set(Some(sandwich_label));
            close();
        }))
        .children(flame.actions.iter().copied().enumerate().map(|(index, action)| {
            menu_item(format!("action-{index}"), action.label, ui, run(action, label.clone(), (node.value, own, level), flame))
        }))
        .child((head != NONE).then(|| {
            menu_item("group", if expanded { "Collapse group" } else { "Expand group" }, ui, move |_, _, _| {
                flame.groups.update(|groups| {
                    if !groups.remove(&head) {
                        groups.insert(head);
                    }
                });
                close();
            })
        }))
        .child((head != NONE && !all_expanded).then(|| {
            menu_item("expand-all", "Expand all groups", ui, move |_, _, _| {
                expand_all(flame);
                close();
            })
        }))
        .child((head != NONE && any_expanded).then(|| {
            menu_item("collapse-all", "Collapse all groups", ui, move |_, _, _| {
                flame.groups.set(BTreeSet::new());
                close();
            })
        }));
    // a press anywhere else closes the menu
    let backdrop = div().id("flame-menu-backdrop").class("fixed inset-0").on_any_mouse_down(move |_, _, _| close());
    Some(div().class("contents").child(backdrop).child(card))
}

fn run(
    action: super::options::FrameAction,
    label: SharedString,
    (total, self_value, level): (f64, f64, u32),
    flame: Flame,
) -> impl Fn(&ClickEvent, &mut Window, &mut App) + 'static {
    move |_, window, cx| {
        let selected = Selected { label: &label, total, self_value, level };
        (action.run)(&selected, window, cx);
        flame.menu.set(None);
    }
}
