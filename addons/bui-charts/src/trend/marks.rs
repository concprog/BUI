use blenderui::{Hsla, SharedString};

#[derive(Clone, Debug, PartialEq)]
pub struct Annotation {
    pub(crate) time: f64,
    pub(crate) end: Option<f64>,
    pub(crate) title: SharedString,
    pub(crate) text: SharedString,
    pub(crate) color: Option<Hsla>,
}

impl Annotation {
    #[must_use]
    pub fn at(time: f64, title: impl Into<SharedString>) -> Self {
        Self {
            time,
            end: None,
            title: title.into(),
            text: SharedString::default(),
            color: None,
        }
    }

    #[must_use]
    pub fn region(from: f64, to: f64, title: impl Into<SharedString>) -> Self {
        Self {
            end: Some(to),
            ..Self::at(from, title)
        }
    }

    #[must_use]
    pub fn text(mut self, text: impl Into<SharedString>) -> Self {
        self.text = text.into();
        self
    }

    #[must_use]
    pub fn color(mut self, color: impl Into<Hsla>) -> Self {
        self.color = Some(color.into());
        self
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum VizOrientation {
    #[default]
    Auto,
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Mark {
    Note(usize),
    Exemplar(usize),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MarkHover {
    pub mark: Mark,
    pub at: crate::geom::Point,
    pub area: crate::geom::Size,
}

pub enum Annotations {
    Owned(Vec<Annotation>),
    Shared(blenderui::hooks::Signal<Vec<Annotation>>),
    Static(&'static [Annotation]),
}

impl Annotations {
    /// The notes, owned (they are drawn after the signal is let go).
    pub(crate) fn into_notes(self) -> Vec<Annotation> {
        match self {
            Self::Owned(notes) => notes,
            Self::Shared(signal) => signal.get(),
            Self::Static(notes) => notes.to_vec(),
        }
    }
}

impl From<Vec<Annotation>> for Annotations {
    fn from(notes: Vec<Annotation>) -> Self {
        Self::Owned(notes)
    }
}

impl From<blenderui::hooks::Signal<Vec<Annotation>>> for Annotations {
    fn from(signal: blenderui::hooks::Signal<Vec<Annotation>>) -> Self {
        Self::Shared(signal)
    }
}

impl From<&'static [Annotation]> for Annotations {
    fn from(notes: &'static [Annotation]) -> Self {
        Self::Static(notes)
    }
}
