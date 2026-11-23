//! Panel menus: the entries (built-in and the app's) and the popup that shows them, opened from a
//! panel's `⋯` button or by right-clicking its tab bar. blenderui has no popup menu, so the popup
//! is built here from its elements: rows with a check, a shortcut and a submenu arrow (Blender's
//! `MENU_ITEM` layout), hover and keyboard navigation, and a backdrop that closes it.

use std::rc::Rc;

use blenderui::prelude::{
    AnyElement, App, Context, FluentBuilder as _, FontWeight,
    IntoElement, MouseButton, MouseDownEvent, ParentElement as _, SharedString, Styled, Window, div,
    px,
};

use super::{
    icon::{Icon, IconName},
    model::types::{Id, Rect},
    render::place,
    workspace::{Palette, ViewInfo, Workspace},
};

pub type MenuAction = Rc<dyn Fn(&mut Window, &mut App)>;

#[derive(Clone)]
pub struct MenuItem {
    pub id: Option<SharedString>,
    pub label: SharedString,
    pub shortcut: Option<SharedString>,
    pub disabled: bool,
    pub checked: Option<bool>,
    pub danger: bool,
    pub run: Option<MenuAction>,
    pub items: Vec<MenuEntry>,
}

#[derive(Clone)]
pub enum MenuEntry {
    Item(MenuItem),
    Separator,
}

impl MenuItem {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            id: None,
            label: label.into(),
            shortcut: None,
            disabled: false,
            checked: None,
            danger: false,
            run: None,
            items: Vec::new(),
        }
    }

    #[must_use]
    pub fn id(mut self, id: impl Into<SharedString>) -> Self {
        self.id = Some(id.into());
        self
    }
    #[must_use]
    pub fn shortcut(mut self, shortcut: Option<String>) -> Self {
        self.shortcut = shortcut.map(Into::into);
        self
    }
    #[must_use]
    pub const fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    #[must_use]
    pub const fn checked(mut self, checked: bool) -> Self {
        self.checked = Some(checked);
        self
    }
    #[must_use]
    pub const fn danger(mut self) -> Self {
        self.danger = true;
        self
    }
    #[must_use]
    pub fn run(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.run = Some(Rc::new(f));
        self
    }
    #[must_use]
    pub fn submenu(mut self, items: Vec<MenuEntry>) -> Self {
        self.items = items;
        self
    }
}

impl From<MenuItem> for MenuEntry {
    fn from(item: MenuItem) -> Self {
        Self::Item(item)
    }
}

pub mod builtin {
    pub const MAXIMIZE: &str = "maximize";
    pub const FLOAT: &str = "float";
    pub const DOCK: &str = "dock";
    pub const MOVE: &str = "move";
    pub const SPLIT_RIGHT: &str = "split-right";
    pub const SPLIT_BELOW: &str = "split-below";
    pub const HIDE: &str = "hide";
    pub const CLOSE: &str = "close";
    pub const CLOSE_OTHERS: &str = "close-others";
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuRegion {
    Stage,
    Side,
    Floating,
}

pub struct PanelMenuContext {
    pub panel_id: Id,
    pub view: ViewInfo,
    pub region: MenuRegion,
}

/// Rewrites a panel's menu (`<Workspace panel_menu>`): the entries and where the panel is.
pub type PanelMenuFn = Rc<dyn Fn(Vec<MenuEntry>, &PanelMenuContext) -> Vec<MenuEntry>>;

#[derive(Clone, Default)]
pub enum PanelMenu {
    #[default]
    Default,
    Off,
    Custom(PanelMenuFn),
}

#[must_use]
pub fn tidy_menu(entries: Vec<MenuEntry>) -> Vec<MenuEntry> {
    let mut out: Vec<MenuEntry> = Vec::with_capacity(entries.len());
    for entry in entries {
        let is_sep = matches!(entry, MenuEntry::Separator);
        if is_sep && matches!(out.last(), None | Some(MenuEntry::Separator)) {
            continue;
        }
        out.push(entry);
    }
    while matches!(out.last(), Some(MenuEntry::Separator)) {
        out.pop();
    }
    out
}

const ROW: f64 = 24.0;
const SEPARATOR: f64 = 9.0;
const PAD: f64 = 4.0;
const MIN_WIDTH: f64 = 168.0;
/// An estimate of the UI font's average advance at the menu's size, to size and place columns.
const CHAR: f64 = 7.0;

/// An open popup: its entries, where its first column sits (workspace pixels) and the
/// highlighted row of each open column (the last one is the keyboard's).
pub(crate) struct Popup {
    entries: Vec<MenuEntry>,
    at: (f64, f64),
    path: Vec<usize>,
}

impl Popup {
    pub(crate) const fn new(entries: Vec<MenuEntry>, at: (f64, f64)) -> Self {
        Self {
            entries,
            at,
            path: Vec::new(),
        }
    }

    /// The entries of the column at `level` (the root is level 0).
    fn column(&self, level: usize) -> &[MenuEntry] {
        let mut entries = self.entries.as_slice();
        for &i in &self.path[..level] {
            match entries.get(i) {
                Some(MenuEntry::Item(item)) if !item.items.is_empty() => entries = &item.items,
                _ => return &[],
            }
        }
        entries
    }

    fn item(&self, level: usize, index: usize) -> Option<&MenuItem> {
        match self.column(level).get(index)? {
            MenuEntry::Item(item) => Some(item),
            MenuEntry::Separator => None,
        }
    }

    /// The open columns: the root, then one per highlighted submenu row.
    fn depth(&self) -> usize {
        let mut depth = 1;
        while depth <= self.path.len() && self.item(depth - 1, self.path[depth - 1]).is_some_and(|i| !i.items.is_empty() && !i.disabled) {
            depth += 1;
        }
        depth
    }

    fn highlight(&mut self, level: usize, index: usize) -> bool {
        if self.path.len() == level + 1 && self.path[level] == index {
            return false;
        }
        self.path.truncate(level);
        self.path.push(index);
        true
    }

    /// The next enabled row of the keyboard's column after (`step` 1) or before (-1) the highlight.
    fn step(&mut self, step: isize) {
        let level = self.path.len().saturating_sub(1);
        let column = self.column(level);
        let n = column.len().cast_signed();
        if n == 0 {
            return;
        }
        let start = self.path.get(level).map_or(if step > 0 { -1 } else { n }, |&i| i.cast_signed());
        for k in 1..=n {
            let i = (start + step * k).rem_euclid(n) as usize;
            if matches!(&column[i], MenuEntry::Item(item) if !item.disabled) {
                self.path.truncate(level);
                self.path.push(i);
                return;
            }
        }
    }

    /// Opens the highlighted row's submenu at its first enabled row.
    fn enter_submenu(&mut self) -> bool {
        let level = self.path.len();
        if level == 0 || self.depth() <= level {
            return false;
        }
        let first = (0..self.column(level).len())
            .find(|&i| self.item(level, i).is_some_and(|item| !item.disabled));
        first.is_some_and(|i| {
            self.path.push(i);
            true
        })
    }

    fn width(entries: &[MenuEntry]) -> f64 {
        entries
            .iter()
            .filter_map(|e| match e {
                MenuEntry::Item(item) => Some(
                    (item.label.chars().count() + item.shortcut.as_ref().map_or(0, |s| s.chars().count() + 4)) as f64
                        * CHAR
                        + 64.0,
                ),
                MenuEntry::Separator => None,
            })
            .fold(MIN_WIDTH, f64::max)
    }

    fn height(entries: &[MenuEntry]) -> f64 {
        entries
            .iter()
            .map(|e| match e {
                MenuEntry::Item(_) => ROW,
                MenuEntry::Separator => SEPARATOR,
            })
            .sum::<f64>()
            + PAD * 2.0
    }

    fn row_top(entries: &[MenuEntry], index: usize) -> f64 {
        PAD + Self::height(&entries[..index]) - PAD * 2.0
    }

    /// Each open column's entries and rect, kept inside `viewport`.
    fn columns(&self, viewport: (f64, f64)) -> Vec<(usize, Rect)> {
        let mut out: Vec<(usize, Rect)> = Vec::with_capacity(self.depth());
        for level in 0..self.depth() {
            let entries = self.column(level);
            let (w, h) = (Self::width(entries), Self::height(entries));
            let (x, y) = match out.last() {
                None => self.at,
                Some((_, parent)) => {
                    let top = parent.y + Self::row_top(self.column(level - 1), self.path[level - 1]) - PAD;
                    let right = parent.right() - 2.0;
                    let x = if right + w > viewport.0 { parent.x - w + 2.0 } else { right };
                    (x, top)
                }
            };
            let x = x.min(viewport.0 - w).max(0.0);
            let y = y.min(viewport.1 - h).max(0.0);
            out.push((level, Rect::new(x, y, w, h)));
        }
        out
    }
}

/// What a key did to the open popup.
pub(crate) enum KeyResult {
    Ignored,
    Handled,
    Run(Option<MenuAction>),
}

/// The id of a popup row (`zopra-menu-<level>-<index>`): tests and the app find rows with it.
#[must_use]
pub fn row_id(level: usize, index: usize) -> SharedString {
    format!("zopra-menu-{level}-{index}").into()
}

impl Workspace {
    pub(crate) fn open_menu(&mut self, panel_id: &str, at: (f64, f64), cx: &mut Context<Self>) {
        let entries = self.panel_menu_entries(panel_id);
        if entries.is_empty() {
            return;
        }
        self.menu = Some(Popup::new(entries, at));
        cx.notify();
    }

    pub(crate) fn close_menu(&mut self, cx: &mut Context<Self>) {
        if self.menu.take().is_some() {
            cx.notify();
        }
    }

    /// Up / Down move the highlight, Right / Enter open a submenu, Left / Escape close one, Enter
    /// runs a row: its action comes back to run once the workspace is no longer borrowed.
    pub(crate) fn menu_key(&mut self, key: &str, cx: &mut Context<Self>) -> KeyResult {
        let Some(popup) = self.menu.as_mut() else {
            return KeyResult::Ignored;
        };
        match super::keymap::normalize_key(key).as_str() {
            "down" => popup.step(1),
            "up" => popup.step(-1),
            "right" => {
                popup.enter_submenu();
            }
            "left" => {
                if popup.path.len() > 1 {
                    popup.path.pop();
                }
            }
            "escape" => {
                if popup.path.len() > 1 {
                    popup.path.pop();
                } else {
                    self.menu = None;
                }
            }
            "enter" | "space" => {
                let level = popup.path.len().saturating_sub(1);
                let Some(&index) = popup.path.last() else {
                    return KeyResult::Handled;
                };
                if popup.enter_submenu() {
                    cx.notify();
                    return KeyResult::Handled;
                }
                let run = popup
                    .item(level, index)
                    .filter(|item| !item.disabled)
                    .map(|item| item.run.clone());
                if let Some(run) = run {
                    self.menu = None;
                    cx.notify();
                    return KeyResult::Run(run);
                }
            }
            _ => return KeyResult::Handled,
        }
        cx.notify();
        KeyResult::Handled
    }

    pub(crate) fn menu_layer(&self, p: &Palette, cx: &mut Context<Self>) -> Option<AnyElement> {
        let popup = self.menu.as_ref()?;
        let ws = cx.entity();
        let close = move |_: &MouseDownEvent, _: &mut Window, cx: &mut App| {
            ws.update(cx, Self::close_menu);
            cx.stop_propagation();
        };
        let mut layer = div()
            .id("zopra-menu-backdrop")
            .absolute()
            .size_full()
            .on_mouse_down(MouseButton::Left, close)
            .on_mouse_down(MouseButton::Right, close)
            .on_mouse_down(MouseButton::Middle, close);
        for (level, rect) in popup.columns(self.viewport) {
            let rows = popup.column(level).iter().enumerate().map(|(index, entry)| match entry {
                MenuEntry::Separator => div()
                    .h(px(SEPARATOR as f32))
                    .flex()
                    .items_center()
                    .px(px(6.0))
                    .child(div().w_full().h(px(1.0)).bg(p.border))
                    .into_any_element(),
                MenuEntry::Item(item) => {
                    let highlighted = popup.path.get(level) == Some(&index);
                    self.menu_row(item, level, index, highlighted, p, cx)
                }
            });
            layer = layer.child(
                place(div().id(format!("zopra-menu-{level}")), rect)
                    .css("user-select", "none")
                    .flex()
                    .flex_col()
                    .p(px(PAD as f32))
                    .rounded(px(6.0))
                    .bg(p.panel)
                    .border_1()
                    .border_color(p.border)
                    .shadow_xl()
                    .text_color(p.text)
                    .text_size(px(12.0))
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .on_mouse_down(MouseButton::Right, |_, _, cx| cx.stop_propagation())
                    .children(rows),
            );
        }
        Some(layer.into_any_element())
    }

    fn menu_row(
        &self,
        item: &MenuItem,
        level: usize,
        index: usize,
        highlighted: bool,
        p: &Palette,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let submenu = !item.items.is_empty();
        let ws = cx.entity();
        let run = item.run.clone();
        let (text, accent, contrast, muted) = (
            if item.danger { blenderui::prelude::rgb(0xe5_5b_5b) } else { p.text },
            p.accent,
            p.accent_contrast,
            p.text_muted,
        );
        let label_color = if highlighted && !item.disabled { contrast } else { text };
        div()
            .id(row_id(level, index))
            .h(px(ROW as f32))
            .flex_none()
            .flex()
            .items_center()
            .gap(px(6.0))
            .px(px(6.0))
            .rounded(px(4.0))
            .text_color(label_color)
            .when(item.disabled, |d| d.opacity(0.45))
            .when(highlighted && !item.disabled, |d| d.bg(accent))
            .on_hover(cx.listener(move |this, hovered: &bool, window, cx| {
                if *hovered && this.menu.as_mut().is_some_and(|m| m.highlight(level, index)) {
                    cx.notify();
                    window.refresh();
                }
            }))
            .when(!item.disabled, |d| {
                d.cursor_pointer().on_mouse_down(MouseButton::Left, move |_, window, cx| {
                    cx.stop_propagation();
                    if submenu {
                        ws.update(cx, |this, cx| {
                            if let Some(menu) = this.menu.as_mut() {
                                menu.highlight(level, index);
                                cx.notify();
                            }
                        });
                        return;
                    }
                    ws.update(cx, Self::close_menu);
                    if let Some(run) = &run {
                        run(window, cx);
                    }
                })
            })
            .child(
                div().w(px(14.0)).flex_none().flex().items_center().children(
                    item.checked
                        .filter(|c| *c)
                        .map(|_| Icon::new(IconName::Check).size(px(12.0)).color(label_color)),
                ),
            )
            .child(
                div()
                    .flex_1()
                    .min_w(px(0.0))
                    .truncate()
                    .when(highlighted, |d| d.font_weight(FontWeight::MEDIUM))
                    .child(item.label.clone()),
            )
            .children(item.shortcut.clone().map(|s| {
                div()
                    .flex_none()
                    .text_size(px(11.0))
                    .text_color(if highlighted { contrast } else { muted })
                    .child(s)
            }))
            .children(submenu.then(|| Icon::new(IconName::ChevronRight).size(px(10.0)).color(label_color)))
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entries() -> Vec<MenuEntry> {
        vec![
            MenuItem::new("One").into(),
            MenuEntry::Separator,
            MenuItem::new("Off").disabled(true).into(),
            MenuItem::new("More").submenu(vec![MenuItem::new("A").into(), MenuItem::new("B").into()]).into(),
        ]
    }

    #[test]
    fn keyboard_skips_separators_and_disabled_rows() {
        let mut popup = Popup::new(entries(), (0.0, 0.0));
        popup.step(1);
        assert_eq!(popup.path, [0]);
        popup.step(1);
        assert_eq!(popup.path, [3]);
        assert!(popup.enter_submenu());
        assert_eq!(popup.path, [3, 0]);
        assert_eq!(popup.depth(), 2);
        popup.step(-1);
        assert_eq!(popup.path, [3, 1]);
    }

    #[test]
    fn columns_stay_in_the_viewport() {
        let mut popup = Popup::new(entries(), (790.0, 590.0));
        popup.highlight(0, 3);
        let cols = popup.columns((800.0, 600.0));
        assert_eq!(cols.len(), 2);
        for (_, r) in cols {
            assert!(r.right() <= 800.0 && r.bottom() <= 600.0 && r.x >= 0.0 && r.y >= 0.0);
        }
    }

    #[test]
    fn tidy_drops_extra_separators() {
        let tidy = tidy_menu(vec![MenuEntry::Separator, MenuItem::new("a").into(), MenuEntry::Separator, MenuEntry::Separator]);
        assert_eq!(tidy.len(), 1);
    }
}
