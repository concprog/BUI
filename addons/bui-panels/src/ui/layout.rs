//! The layout markup: `<Row>`, `<Column>`, `<Stage>`, `<Panel>`, `<Float>`, `<Template>` and
//! `<View>`, inside a `<Workspace>`.
//!
//! zopra handed a component its typed children; blenderui's `rsx!` makes every child an element,
//! children first. So each of these components declares its node on a per-thread stack and
//! stands in the tree as a marker: a parent takes as many nodes off the stack as it has markers
//! among its children (they were declared last, in order).

use std::{borrow::Cow, cell::Cell, rc::Rc};

use blenderui::{
    hooks::Complete,
    prelude::{AnyElement, App, IntoElement, ParentElement as _, SharedString, Styled, Window, div},
};
use serde::de::DeserializeOwned;
use serde_json::Value;

use super::content::{Content, TemplateFn};
use crate::{
    icon::IconName,
    menu::MenuEntry,
    model::types::{
        Axis, FloatingLayer, GestureOwner, Id, Placement, Rect, Scaling, TabbarMode,
        ViewRules,
    },
    workspace::{Badge, Guard, ViewInfo, ViewMenu},
};

pub enum LayoutNode {
    Split {
        id: Option<Id>,
        axis: Axis,
        sizes: Option<Vec<f64>>,
        children: Vec<Self>,
    },
    Stage {
        id: Option<Id>,
        child: Option<Box<Self>>,
    },
    Panel {
        id: Option<Id>,
        views: Vec<ViewNode>,
    },
    Float {
        rect: Rect,
        layer: Option<FloatingLayer>,
        panels: Vec<Self>,
    },
    Template(TemplateNode),
}

pub(crate) type CloseFn = Rc<dyn Fn(&mut Window, &mut App)>;

pub struct TemplateNode {
    pub(crate) name: String,
    pub(crate) render: TemplateFn,
}

pub struct ViewNode {
    pub(crate) id: Id,
    pub(crate) title: Option<SharedString>,
    pub(crate) icon: Option<IconName>,
    #[allow(clippy::option_option)]
    pub(crate) badge: Option<Option<Badge>>,
    pub(crate) rules: ViewRules,
    pub(crate) accessory: Option<AnyElement>,
    pub(crate) menu: Option<ViewMenu>,
    pub(crate) content: Option<Content>,
    pub(crate) inline: Option<AnyElement>,
    pub(crate) open_from: Option<Rect>,
    pub(crate) focus_on_open: bool,
    pub(crate) on_close: Option<CloseFn>,
    pub(crate) can_close: Option<Guard>,
}

enum Declared {
    Layout(LayoutNode),
    View(Box<ViewNode>),
}

thread_local! {
    static DECLARED: Cell<Vec<Declared>> = const { Cell::new(Vec::new()) };
}

/// What a declared node leaves in the element tree (a word joiner: nothing shows if it ends up
/// in one).
static MARKER: &str = "\u{2060}";

fn is_marker(el: &AnyElement) -> bool {
    matches!(el, AnyElement::Text(Cow::Borrowed(s)) if std::ptr::eq(s.as_ptr(), MARKER.as_ptr()))
}

fn declare(node: Declared) -> AnyElement {
    DECLARED.with(|cell| {
        let mut stack = cell.take();
        stack.push(node);
        cell.set(stack);
    });
    AnyElement::Text(Cow::Borrowed(MARKER))
}

/// The nodes `children` declared, in order.
fn take(children: &[AnyElement]) -> Vec<Declared> {
    let n = children.iter().filter(|c| is_marker(c)).count();
    DECLARED.with(|cell| {
        let mut stack = cell.take();
        let nodes = stack.split_off(stack.len().checked_sub(n).expect("bui-panels: layout markup out of order"));
        cell.set(stack);
        nodes
    })
}

pub(crate) fn layout_children(children: &[AnyElement], parent: &str) -> Vec<LayoutNode> {
    take(children)
        .into_iter()
        .map(|node| match node {
            Declared::Layout(node) => node,
            Declared::View(view) => panic!("a <View> ({}) goes in a <Panel>, not in a <{parent}>", view.id),
        })
        .collect()
}

macro_rules! props {
    ($ty:ident) => {
        impl Complete for $ty {
            type Props = Self;
            fn finish(self) -> Self {
                self
            }
        }
    };
}

/// `<Row>` / `<Column>`: a split along x / y; `sizes` are the children's weights.
pub struct Split {
    axis: Axis,
    id: Option<Id>,
    sizes: Option<Vec<f64>>,
    children: Vec<AnyElement>,
}

impl Split {
    const fn new(axis: Axis) -> Self {
        Self {
            axis,
            id: None,
            sizes: None,
            children: Vec::new(),
        }
    }
    #[must_use]
    pub fn id(mut self, id: impl Into<Id>) -> Self {
        self.id = Some(id.into());
        self
    }
    #[must_use]
    pub fn sizes(mut self, sizes: impl Into<Vec<f64>>) -> Self {
        self.sizes = Some(sizes.into());
        self
    }
    #[must_use]
    pub fn children(mut self, children: Vec<AnyElement>) -> Self {
        self.children = children;
        self
    }
}
props!(Split);

impl IntoElement for Split {
    fn into_any_element(self) -> AnyElement {
        let parent = if self.axis == Axis::X { "Row" } else { "Column" };
        let children = layout_children(&self.children, parent);
        declare(Declared::Layout(LayoutNode::Split {
            id: self.id,
            axis: self.axis,
            sizes: self.sizes,
            children,
        }))
    }
}

/// `<Row>`: children side by side.
pub struct Row;
impl Row {
    #[doc(hidden)]
    #[must_use]
    pub const fn __builder() -> Split {
        Split::new(Axis::X)
    }
}

/// `<Column>`: children stacked.
pub struct Column;
impl Column {
    #[doc(hidden)]
    #[must_use]
    pub const fn __builder() -> Split {
        Split::new(Axis::Y)
    }
}

/// `<Stage>`: the workspace's main area (at most one child).
#[derive(Default)]
pub struct Stage {
    id: Option<Id>,
    children: Vec<AnyElement>,
}

impl Stage {
    #[doc(hidden)]
    #[must_use]
    pub fn __builder() -> Self {
        Self::default()
    }
    #[must_use]
    pub fn id(mut self, id: impl Into<Id>) -> Self {
        self.id = Some(id.into());
        self
    }
    #[must_use]
    pub fn children(mut self, children: Vec<AnyElement>) -> Self {
        self.children = children;
        self
    }
}
props!(Stage);

impl IntoElement for Stage {
    fn into_any_element(self) -> AnyElement {
        let child = layout_children(&self.children, "Stage").into_iter().next();
        declare(Declared::Layout(LayoutNode::Stage {
            id: self.id,
            child: child.map(Box::new),
        }))
    }
}

/// `<Panel>`: tabs, one per `<View>`.
#[derive(Default)]
pub struct Panel {
    id: Option<Id>,
    children: Vec<AnyElement>,
}

impl Panel {
    #[doc(hidden)]
    #[must_use]
    pub fn __builder() -> Self {
        Self::default()
    }
    #[must_use]
    pub fn id(mut self, id: impl Into<Id>) -> Self {
        self.id = Some(id.into());
        self
    }
    #[must_use]
    pub fn children(mut self, children: Vec<AnyElement>) -> Self {
        self.children = children;
        self
    }
}
props!(Panel);

impl IntoElement for Panel {
    fn into_any_element(self) -> AnyElement {
        let views = take(&self.children)
            .into_iter()
            .map(|node| match node {
                Declared::View(view) => *view,
                Declared::Layout(_) => panic!("a <Panel> holds <View>s only"),
            })
            .collect();
        declare(Declared::Layout(LayoutNode::Panel { id: self.id, views }))
    }
}

/// `<Float x y w h>`: floating panels (pixels of the workspace).
#[derive(Default)]
pub struct Float {
    rect: Rect,
    layer: Option<FloatingLayer>,
    children: Vec<AnyElement>,
}

impl Float {
    #[doc(hidden)]
    #[must_use]
    pub fn __builder() -> Self {
        Self::default()
    }
    #[must_use]
    pub const fn x(mut self, x: f64) -> Self {
        self.rect.x = x;
        self
    }
    #[must_use]
    pub const fn y(mut self, y: f64) -> Self {
        self.rect.y = y;
        self
    }
    #[must_use]
    pub const fn w(mut self, w: f64) -> Self {
        self.rect.w = w;
        self
    }
    #[must_use]
    pub const fn h(mut self, h: f64) -> Self {
        self.rect.h = h;
        self
    }
    #[must_use]
    pub const fn layer(mut self, layer: FloatingLayer) -> Self {
        self.layer = Some(layer);
        self
    }
    #[must_use]
    pub fn children(mut self, children: Vec<AnyElement>) -> Self {
        self.children = children;
        self
    }
}
props!(Float);

impl IntoElement for Float {
    fn into_any_element(self) -> AnyElement {
        let panels = layout_children(&self.children, "Float");
        declare(Declared::Layout(LayoutNode::Float {
            rect: self.rect,
            layer: self.layer,
            panels,
        }))
    }
}

/// `<Template name content>`: a view type whose content is made from a view's params
/// (`ws.open(name, params, cx)` opens one).
#[derive(Default)]
pub struct Template {
    name: Option<String>,
    render: Option<TemplateFn>,
}

impl Template {
    #[doc(hidden)]
    #[must_use]
    pub fn __builder() -> Self {
        Self::default()
    }
    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
    /// `|params: MyParams, window, cx| rsx! { .. }`: views whose params don't deserialize show
    /// nothing.
    #[must_use]
    pub fn content<P, E>(mut self, content: impl Fn(P, &mut Window, &mut App) -> E + 'static) -> Self
    where
        P: DeserializeOwned + 'static,
        E: IntoElement,
    {
        self.render = Some(Rc::new(move |params: &Value, window, cx| {
            let params = P::deserialize(params).ok()?;
            Some(content(params, window, cx).into_any_element())
        }));
        self
    }
}
props!(Template);

impl IntoElement for Template {
    fn into_any_element(self) -> AnyElement {
        let name = self.name.expect("a <Template> needs a `name`");
        let render = self.render.expect("a <Template> needs `content`");
        declare(Declared::Layout(LayoutNode::Template(TemplateNode { name, render })))
    }
}

/// `<View id title ..>`: a tab. Its children are its content, or `content={move || ..}` makes it
/// each render (where `use_view()` works).
pub struct View {
    id: Option<Id>,
    title: Option<SharedString>,
    icon: Option<IconName>,
    #[allow(clippy::option_option)]
    badge: Option<Option<Badge>>,
    rules: ViewRules,
    accessory: Option<AnyElement>,
    menu: Option<ViewMenu>,
    content: Option<Content>,
    open_from: Option<Rect>,
    focus_on_open: bool,
    on_close: Option<CloseFn>,
    can_close: Option<Guard>,
    children: Vec<AnyElement>,
}

impl View {
    #[doc(hidden)]
    #[must_use]
    pub fn __builder() -> Self {
        Self {
            id: None,
            title: None,
            icon: None,
            badge: None,
            rules: ViewRules::default(),
            accessory: None,
            menu: None,
            content: None,
            open_from: None,
            focus_on_open: true,
            on_close: None,
            can_close: None,
            children: Vec::new(),
        }
    }
    #[must_use]
    pub fn id(mut self, id: impl Into<Id>) -> Self {
        self.id = Some(id.into());
        self
    }
    #[must_use]
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }
    #[must_use]
    pub const fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }
    /// `None` takes the badge off; leaving `badge` out leaves it to `use_view().set_badge(..)`.
    #[must_use]
    pub fn badge(mut self, badge: impl Into<Option<Badge>>) -> Self {
        self.badge = Some(badge.into());
        self
    }
    #[must_use]
    pub fn placement(mut self, placement: Placement) -> Self {
        self.rules.placement = Some(placement);
        self
    }
    #[must_use]
    pub const fn keep_out_of_stage(mut self, keep_out: bool) -> Self {
        self.rules.allow.stage = !keep_out;
        self
    }
    #[must_use]
    pub const fn keep_out_of_sides(mut self, keep_out: bool) -> Self {
        self.rules.allow.side = !keep_out;
        self
    }
    #[must_use]
    pub const fn keep_out_of_floating(mut self, keep_out: bool) -> Self {
        self.rules.allow.floating = !keep_out;
        self
    }
    #[must_use]
    pub const fn singleton(mut self, singleton: bool) -> Self {
        self.rules.singleton = singleton;
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
    pub const fn min_size(mut self, size: (f64, f64)) -> Self {
        self.rules.min_size = Some(size);
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
    pub fn accessory(mut self, accessory: impl IntoElement) -> Self {
        self.accessory = Some(accessory.into_any_element());
        self
    }
    #[must_use]
    pub fn menu(mut self, menu: impl Fn(&ViewInfo) -> Vec<MenuEntry> + 'static) -> Self {
        self.menu = Some(Rc::new(menu));
        self
    }
    #[must_use]
    pub fn content(mut self, content: impl Into<Content>) -> Self {
        self.content = Some(content.into());
        self
    }
    #[must_use]
    pub const fn open_from(mut self, rect: Rect) -> Self {
        self.open_from = Some(rect);
        self
    }
    #[must_use]
    pub const fn focus_on_open(mut self, focus: bool) -> Self {
        self.focus_on_open = focus;
        self
    }
    #[must_use]
    pub fn on_close(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_close = Some(Rc::new(f));
        self
    }
    #[must_use]
    pub fn can_close(mut self, f: impl Fn(&App) -> bool + 'static) -> Self {
        self.can_close = Some(Rc::new(f));
        self
    }
    #[must_use]
    pub fn children(mut self, children: Vec<AnyElement>) -> Self {
        self.children = children;
        self
    }
}
props!(View);

impl IntoElement for View {
    fn into_any_element(self) -> AnyElement {
        let id = self.id.expect("a <View> needs an `id` (blenderui keeps `key` for hook state)");
        let inline = (!self.children.is_empty())
            .then(|| div().size_full().children(self.children).into_any_element());
        declare(Declared::View(Box::new(ViewNode {
            id,
            title: self.title,
            icon: self.icon,
            badge: self.badge,
            rules: self.rules,
            accessory: self.accessory,
            menu: self.menu,
            content: self.content,
            inline,
            open_from: self.open_from,
            focus_on_open: self.focus_on_open,
            on_close: self.on_close,
            can_close: self.can_close,
        })))
    }
}

#[cfg(test)]
mod tests {
    use blenderui::prelude::rsx;

    use super::*;

    #[test]
    fn markup_declares_nested_nodes_in_order() {
        let el = rsx! {
            <Row sizes={[1.0, 2.0]}>
                <Panel id="a"><View id="x" /><View id="y" /></Panel>
                <Column><Panel id="b"><View id="z" /></Panel></Column>
            </Row>
        };
        let nodes = layout_children(&[el.into_any_element()], "Workspace");
        let [LayoutNode::Split { axis: Axis::X, children, sizes, .. }] = nodes.as_slice() else {
            panic!("one row");
        };
        assert_eq!(sizes.as_deref(), Some(&[1.0, 2.0][..]));
        let LayoutNode::Panel { id, views } = &children[0] else { panic!("a panel") };
        assert_eq!(id.as_deref(), Some("a"));
        assert_eq!(views.iter().map(|v| &*v.id).collect::<Vec<_>>(), ["x", "y"]);
        assert!(matches!(&children[1], LayoutNode::Split { axis: Axis::Y, .. }));
        assert!(DECLARED.with(|c| {
            let s = c.take();
            let empty = s.is_empty();
            c.set(s);
            empty
        }));
    }
}
