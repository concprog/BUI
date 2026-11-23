use blenderui::Hsla;

use super::trace::Trace;
use crate::ChartColors;

const MIN_CONTRAST: f32 = 1.5;

fn java_hash(key: &str) -> i32 {
    key.to_lowercase().encode_utf16().fold(0i32, |hash, unit| {
        (hash << 5).wrapping_sub(hash).wrapping_add(i32::from(unit))
    })
}

fn contrast(a: Hsla, b: Hsla) -> f32 {
    let (a, b) = (crate::color::luminance(a), crate::color::luminance(b));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

pub(crate) fn assign(trace: &Trace, colors: &ChartColors, out: &mut Vec<Hsla>) {
    out.clear();
    let count = colors.palette.len();
    if count == 0 {
        out.resize(trace.services.len(), colors.fallback);
        return;
    }
    let mut previous: Option<usize> = None;
    for service in &trace.services {
        let mut index = java_hash(&service.name).unsigned_abs() as usize % count;
        if Some(index) == previous {
            index = (index + 1) % count;
        }
        if let Some(last) = previous {
            let mut tries = 0;
            while contrast(colors.palette[index], colors.palette[last]) < MIN_CONTRAST
                && tries < count
            {
                index = (index + 1) % count;
                tries += 1;
            }
        }
        out.push(colors.palette[index]);
        previous = Some(index);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashes_like_java_strings() {
        assert_eq!(java_hash("abc"), 96354);
        assert_eq!(java_hash("ABC"), java_hash("abc"));
    }
}
