//! Workspaces in markup (zopra's components, on blenderui's `rsx!`): `<Workspace>` with
//! `<Row>`, `<Column>`, `<Stage>`, `<Panel>`, `<Float>`, `<Template>` and `<View>`, rendered
//! inside [`scope`]; [`use_workspace`] and [`use_view`] reach them from components.

mod content;
mod handle;
mod layout;
mod root;
mod scope;

pub(crate) use content::Hosting;
pub use content::Content;
pub use handle::{Opening, ViewRef, WorkspaceRef, use_view, use_workspace};
pub use layout::{Column, Float, LayoutNode, Panel, Row, Split, Stage, Template, TemplateNode, View, ViewNode};
pub use root::Workspace;
pub use scope::scope;
