//! Blender's icons as elements. blenderui has no call that draws one of the kit's icons, so an
//! [`Icon`] decodes the icon's SVG (embedded from the kit by `build.rs`) with blenderui's image
//! decoder, recolored, once per color and size.

use std::{cell::Cell, collections::HashMap, fmt::Write as _, sync::Arc};

use blenderui::{
    Image,
    prelude::{AnyElement, IntoElement, Pixels, Rgba, Styled, img},
};

include!(concat!(env!("OUT_DIR"), "/icons.rs"));

/// Decoded at twice the drawn size: sharp on 2x displays, a clean 2:1 shrink on 1x ones.
const OVERSAMPLE: f32 = 2.0;

type Key = (IconName, u32, u16);

thread_local! {
    static DECODED: Cell<HashMap<Key, Arc<Image>>> = Cell::new(HashMap::new());
}

fn hex(c: Rgba) -> u32 {
    let ch = |v: f32| (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u32;
    (ch(c.r) << 16) | (ch(c.g) << 8) | ch(c.b)
}

/// Blender's icons are white: every white paint becomes `rgb`.
fn recolor(svg: &str, rgb: u32) -> String {
    let mut color = String::with_capacity(7);
    let _ = write!(color, "#{rgb:06x}");
    svg.replace("#ffffff", &color)
        .replace("#fff", &color)
        .replace("\"white\"", &format!("\"{color}\""))
}

fn decoded(name: IconName, rgb: u32, size: u16) -> Option<Arc<Image>> {
    let key = (name, rgb, size);
    DECODED.with(|cell| {
        let mut map = cell.take();
        let image = if let Some(image) = map.get(&key) { Some(image.clone()) } else {
            let px = u32::from(size);
            let image = Image::decode(recolor(name.svg(), rgb).as_bytes(), px, px).map(Arc::new);
            if let Some(image) = &image {
                map.insert(key, image.clone());
            }
            image
        };
        cell.set(map);
        image
    })
}

/// An icon element (gpui-kit's `Icon`): `Icon::new(IconName::Close).size(px(12.0)).color(c)`.
#[derive(Clone, Copy, Debug)]
pub struct Icon {
    name: IconName,
    size: f32,
    color: Rgba,
}

impl Icon {
    #[must_use]
    pub const fn new(name: IconName) -> Self {
        Self {
            name,
            size: 16.0,
            color: Rgba {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
        }
    }

    #[must_use]
    pub const fn size(mut self, size: Pixels) -> Self {
        self.size = size.0;
        self
    }

    #[must_use]
    pub const fn color(mut self, color: Rgba) -> Self {
        self.color = color;
        self
    }
}

impl IntoElement for Icon {
    fn into_any_element(self) -> AnyElement {
        let side = Pixels(self.size);
        let device = (self.size * OVERSAMPLE).ceil().clamp(1.0, f32::from(u16::MAX)) as u16;
        let el = img().flex_none().w(side).h(side).opacity(self.color.a);
        match decoded(self.name, hex(self.color), device) {
            Some(image) => el.image(image).into_any_element(),
            None => el.into_any_element(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_icon_is_blender_svg() {
        for icon in IconName::ALL {
            assert!(icon.svg().starts_with("<svg") || icon.svg().starts_with("<?xml"), "{icon:?}");
            assert!(!icon.blender_name().is_empty());
        }
    }

    #[test]
    fn recolor_replaces_white() {
        let svg = recolor(r##"<g fill="#fff"><path stroke="#ffffff"/></g>"##, 0x3a6ff7);
        assert_eq!(svg, r##"<g fill="#3a6ff7"><path stroke="#3a6ff7"/></g>"##);
    }
}
