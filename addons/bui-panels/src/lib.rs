//! Panel layouts for BUI's Rust binding (`blenderui`): docked splits around a stage, tabs you
//! drag between panels, floating panels, resizable dividers, hiding and restoring, keyboard and
//! gesture navigation, and a layout document (JSON) that saves and restores it all.
//!
//! Two ways in: the [`Workspace`] entity (view types registered in Rust, a window's root view or
//! part of one), or [`ui::Workspace`] markup in `rsx!` (see [`ui`]).

pub mod keymap;
pub mod menu;
pub mod model;
pub mod motion;
pub mod nav;
pub mod ui;

mod drag;
mod features;
mod icon;
mod render;
mod view;
mod workspace;

pub use features::{Detail, Titlebar};
pub use icon::{Icon, IconName};
pub use keymap::{Command, GestureKeys, Keymap, default_gesture_keys};
pub use menu::{MenuEntry, MenuItem, MenuRegion, PanelMenu, PanelMenuContext, builtin};
pub use model::builder::{FloatSpec, create_document, layout};
pub use model::document::{DockTarget, empty_document};
pub use model::types::*;
pub use nav::FrameTarget;
pub use render::ROOT_ID;
pub use view::AnyView;
pub use workspace::{
    Accessory, Badge, Navigation, OpenOptions, Palette, Persist, Reuse, SlotFn, Slots, Subscriber,
    TabOptions, ThemeMode, ViewBuilder, ViewHandle, ViewInfo, ViewPlacement, ViewType, Workspace,
    WorkspaceEvent, WorkspaceOptions,
};

pub mod prelude {
    pub use crate::ui::*;
    pub use crate::{
        Allow, Axis, Badge, Command, Detail, DockTarget, Edge, FloatingLayer, FrameTarget,
        Framing, GestureKeys, GestureOwner, IconName, Id, Keymap, LayoutDocument, MenuEntry,
        MenuItem, MenuRegion, Navigation, PanelMenuContext, Palette, Placement, Rect, Reuse,
        Scaling, TabbarMode, ThemeMode, Titlebar, ViewInfo, ViewPlacement, default_gesture_keys,
    };
}
