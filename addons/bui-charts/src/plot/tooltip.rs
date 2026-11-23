use blenderui::{Pixels, Styled, px};

use crate::geom::{Point, Size};

const OFFSET: f32 = 12.;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TooltipMode {
    #[default]
    Single,
    Multi,
    Hidden,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SortOrder {
    #[default]
    None,
    Ascending,
    Descending,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Tooltip {
    pub(crate) mode: TooltipMode,
    pub(crate) sort: SortOrder,
    pub(crate) hide_zeros: bool,
    pub(crate) max_width: Option<Pixels>,
    pub(crate) max_height: Option<Pixels>,
}

impl Tooltip {
    #[must_use]
    pub fn single() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn multi() -> Self {
        Self { mode: TooltipMode::Multi, ..Self::default() }
    }

    #[must_use]
    pub fn hidden() -> Self {
        Self { mode: TooltipMode::Hidden, ..Self::default() }
    }

    with! {
        sort: SortOrder,
        hide_zeros: bool,
        max_width: Pixels,
        max_height: Pixels,
    }

    pub(crate) fn visible(self) -> bool {
        self.mode != TooltipMode::Hidden
    }
}

/// Places a card beside the pointer at `at` in an area of `area`, on the side with more room.
pub(crate) fn corner<E: Styled>(element: E, at: Point, area: Size) -> E {
    let element = if at.x > area.width / 2. {
        element.right(px(area.width - at.x + OFFSET))
    } else {
        element.left(px(at.x + OFFSET))
    };
    if at.y > area.height / 2. {
        element.bottom(px(area.height - at.y + OFFSET))
    } else {
        element.top(px(at.y + OFFSET))
    }
}
