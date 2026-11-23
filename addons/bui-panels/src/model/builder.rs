use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fmt,
};

use serde_json::Value;

use super::{
    document::uid,
    tree::normalize,
    types::{
        Axis, FloatingLayer, FloatingPanel, Id, LayoutDocument, Node, PanelNode, Rect, SplitNode,
        StageNode, ViewRecord,
    },
};

#[derive(Clone, Debug)]
pub struct ViewSpec {
    pub ty: String,
    pub id: Option<String>,
    pub params: Option<Value>,
    pub title: Option<String>,
}

impl ViewSpec {
    #[must_use]
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }
    #[must_use]
    pub fn params(mut self, params: Value) -> Self {
        self.params = Some(params);
        self
    }
    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }
}

#[derive(Clone, Debug)]
pub struct PanelSpec {
    pub id: Option<String>,
    pub views: Vec<ViewSpec>,
    pub selected: Option<usize>,
}

impl PanelSpec {
    #[must_use]
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }
    #[must_use]
    pub const fn selected(mut self, index: usize) -> Self {
        self.selected = Some(index);
        self
    }
}

#[derive(Clone, Debug)]
pub struct SplitSpec {
    pub id: Option<String>,
    pub axis: Axis,
    pub children: Vec<LayoutSpec>,
    pub weights: Option<Vec<f64>>,
}

impl SplitSpec {
    #[must_use]
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }
}

#[derive(Clone, Debug)]
pub struct StageSpec {
    pub id: Option<String>,
    pub child: Option<Box<LayoutSpec>>,
}

#[derive(Clone, Debug)]
pub enum LayoutSpec {
    View(ViewSpec),
    Panel(PanelSpec),
    Split(SplitSpec),
    Stage(StageSpec),
}

impl From<ViewSpec> for LayoutSpec {
    fn from(v: ViewSpec) -> Self {
        Self::View(v)
    }
}
impl From<PanelSpec> for LayoutSpec {
    fn from(v: PanelSpec) -> Self {
        Self::Panel(v)
    }
}
impl From<SplitSpec> for LayoutSpec {
    fn from(v: SplitSpec) -> Self {
        Self::Split(v)
    }
}
impl From<StageSpec> for LayoutSpec {
    fn from(v: StageSpec) -> Self {
        Self::Stage(v)
    }
}

#[derive(Clone, Debug)]
pub struct FloatSpec {
    pub panel: PanelSpec,
    pub rect: Rect,
    pub layer: Option<FloatingLayer>,
}

pub mod layout {
    use super::{Axis, LayoutSpec, PanelSpec, SplitSpec, StageSpec, ViewSpec};

    pub fn view(ty: impl Into<String>) -> ViewSpec {
        ViewSpec {
            ty: ty.into(),
            id: None,
            params: None,
            title: None,
        }
    }
    pub fn panel(views: impl IntoIterator<Item = ViewSpec>) -> PanelSpec {
        PanelSpec {
            id: None,
            views: views.into_iter().collect(),
            selected: None,
        }
    }
    pub fn split(
        axis: Axis,
        children: impl IntoIterator<Item = LayoutSpec>,
        weights: Option<Vec<f64>>,
    ) -> SplitSpec {
        SplitSpec {
            id: None,
            axis,
            children: children.into_iter().collect(),
            weights,
        }
    }
    pub fn row(
        children: impl IntoIterator<Item = LayoutSpec>,
        weights: Option<Vec<f64>>,
    ) -> SplitSpec {
        split(Axis::X, children, weights)
    }
    pub fn column(
        children: impl IntoIterator<Item = LayoutSpec>,
        weights: Option<Vec<f64>>,
    ) -> SplitSpec {
        split(Axis::Y, children, weights)
    }
    pub fn stage(child: Option<LayoutSpec>) -> StageSpec {
        StageSpec {
            id: None,
            child: child.map(Box::new),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum LayoutError {
    DuplicateView(String),
    MultipleStages,
    NestedStage,
}

impl fmt::Display for LayoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateView(id) => write!(f, "Zopra: duplicate view id \"{id}\""),
            Self::MultipleStages => write!(f, "Zopra: a layout may contain one stage"),
            Self::NestedStage => write!(f, "Zopra: a stage cannot contain a stage"),
        }
    }
}
impl std::error::Error for LayoutError {}

struct Compiler {
    views: BTreeMap<Id, ViewRecord>,
    explicit: HashSet<String>,
    counters: HashMap<String, usize>,
    stages: usize,
}

impl Compiler {
    fn collect(&mut self, spec: &LayoutSpec) {
        match spec {
            LayoutSpec::View(v) => self.collect_view(v),
            LayoutSpec::Panel(p) => p.views.iter().for_each(|v| self.collect_view(v)),
            LayoutSpec::Split(s) => s.children.iter().for_each(|c| self.collect(c)),
            LayoutSpec::Stage(s) => {
                if let Some(c) = &s.child {
                    self.collect(c);
                }
            }
        }
    }
    fn collect_view(&mut self, v: &ViewSpec) {
        if let Some(id) = &v.id {
            self.explicit.insert(id.clone());
        }
    }

    fn next_id(&mut self, ty: &str) -> String {
        loop {
            let n = self.counters.entry(ty.to_string()).or_insert(0);
            *n += 1;
            let id = format!("{ty}-{n}");
            if !self.explicit.contains(&id) && !self.views.contains_key(id.as_str()) {
                return id;
            }
        }
    }

    fn add_view(&mut self, spec: &ViewSpec) -> Result<Id, LayoutError> {
        let id = spec.id.clone().unwrap_or_else(|| self.next_id(&spec.ty));
        if self.views.contains_key(id.as_str()) {
            return Err(LayoutError::DuplicateView(id));
        }
        let id: Id = id.as_str().into();
        self.views.insert(
            id.clone(),
            ViewRecord {
                ty: spec.ty.clone(),
                params: spec.params.clone(),
                title: spec.title.clone(),
            },
        );
        Ok(id)
    }

    fn panel(
        &mut self,
        panel_id: Option<&String>,
        views: &[ViewSpec],
        selected: usize,
    ) -> Result<PanelNode, LayoutError> {
        let ids = views
            .iter()
            .map(|v| self.add_view(v))
            .collect::<Result<Vec<_>, _>>()?;
        let selected = ids
            .get(selected.min(ids.len().saturating_sub(1)))
            .cloned()
            .unwrap_or_else(|| "".into());
        Ok(PanelNode {
            id: panel_id.map_or_else(|| uid("panel"), |s| s.as_str().into()),
            views: ids,
            selected,
        })
    }

    fn build(&mut self, spec: &LayoutSpec) -> Result<Node, LayoutError> {
        match spec {
            LayoutSpec::View(v) => Ok(Node::Panel(self.panel(None, std::slice::from_ref(v), 0)?)),
            LayoutSpec::Panel(p) => Ok(Node::Panel(self.panel(
                p.id.as_ref(),
                &p.views,
                p.selected.unwrap_or(0),
            )?)),
            LayoutSpec::Stage(s) => {
                self.stages += 1;
                if self.stages > 1 {
                    return Err(LayoutError::MultipleStages);
                }
                let child = s.child.as_ref().map(|c| self.build(c)).transpose()?;
                if matches!(child, Some(Node::Stage(_))) {
                    return Err(LayoutError::NestedStage);
                }
                Ok(Node::Stage(StageNode {
                    id: s.id.as_deref().unwrap_or("stage").into(),
                    child: child.map(Box::new),
                }))
            }
            LayoutSpec::Split(s) => {
                let children = s
                    .children
                    .iter()
                    .map(|c| self.build(c))
                    .collect::<Result<Vec<_>, _>>()?;
                let weights = match &s.weights {
                    Some(w) if w.len() == children.len() => w.clone(),
                    _ => vec![1.0; children.len()],
                };
                Ok(Node::Split(SplitNode {
                    id: s.id.as_deref().map_or_else(|| uid("split"), Id::from),
                    axis: s.axis,
                    weights,
                    children,
                }))
            }
        }
    }
}

#[allow(clippy::needless_pass_by_value)] // zopra-panels' signature
pub fn create_document(
    root: Option<&LayoutSpec>,
    floating: Vec<FloatSpec>,
    version: Option<String>,
) -> Result<LayoutDocument, LayoutError> {
    let mut c = Compiler {
        views: BTreeMap::new(),
        explicit: HashSet::new(),
        counters: HashMap::default(),
        stages: 0,
    };
    if let Some(root) = &root {
        c.collect(root);
    }
    for f in &floating {
        f.panel.views.iter().for_each(|v| c.collect_view(v));
    }

    let tree = match &root {
        Some(spec) => normalize(c.build(spec)?),
        None => None,
    };
    let mut floats = Vec::with_capacity(floating.len());
    for (i, f) in floating.iter().enumerate() {
        floats.push(FloatingPanel {
            panel: c.panel(
                f.panel.id.as_ref(),
                &f.panel.views,
                f.panel.selected.unwrap_or(0),
            )?,
            rect: f.rect,
            z: i64::try_from(i + 1).unwrap_or(i64::MAX),
            layer: f.layer.unwrap_or(FloatingLayer::Overlay),
        });
    }
    Ok(LayoutDocument {
        schema: 1,
        version,
        root: tree,
        floating: floats,
        hidden: Vec::new(),
        views: c.views,
        navigation: None,
    })
}

#[cfg(test)]
mod tests {
    use super::{layout::*, *};

    #[test]
    fn deterministic_view_ids() {
        let doc = create_document(
            Some(&row([panel([view("editor"), view("editor")]).into()], None).into()),
            vec![],
            None,
        )
        .unwrap();
        let ids: Vec<&str> = doc.views.keys().map(std::convert::AsRef::as_ref).collect();
        assert_eq!(ids, ["editor-1", "editor-2"]);
    }

    #[test]
    fn rejects_two_stages() {
        let spec = row([stage(None).into(), stage(None).into()], None);
        assert_eq!(
            create_document(Some(&spec.into()), vec![], None).unwrap_err(),
            LayoutError::MultipleStages
        );
    }

    #[test]
    fn rejects_duplicate_ids() {
        let spec = panel([view("a").id("x"), view("b").id("x")]);
        assert_eq!(
            create_document(Some(&spec.into()), vec![], None).unwrap_err(),
            LayoutError::DuplicateView("x".into())
        );
    }
}
