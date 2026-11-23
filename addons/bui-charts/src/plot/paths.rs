use crate::geom::Point;
use crate::path::Path;

/// Paths built for one layout, kept to be drawn again (moved) while their key holds: a hover
/// repaints the marks without building the series' paths again.
pub(crate) struct PathLayer<S> {
    key: Option<u64>,
    origin: Point,
    items: Vec<(Path, S)>,
}

impl<S> Default for PathLayer<S> {
    fn default() -> Self {
        Self { key: None, origin: Point::default(), items: Vec::new() }
    }
}

impl<S> PathLayer<S> {
    pub(crate) fn fresh(&self, key: u64) -> bool {
        self.key == Some(key)
    }

    /// The kept paths, moved to `origin` (in place: a paint that moved nothing moves nothing).
    pub(crate) fn replay(&mut self, origin: Point) -> &[(Path, S)] {
        let offset = origin - self.origin;
        if offset != Point::default() {
            for (path, _) in &mut self.items {
                path.translate(offset);
            }
            self.origin = origin;
        }
        &self.items
    }

    pub(crate) fn store(&mut self, key: u64, origin: Point, items: Vec<(Path, S)>) {
        self.key = Some(key);
        self.origin = origin;
        self.items = items;
    }
}
