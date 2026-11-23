use blenderui::SharedString;

const SHORT: [&str; 5] = ["", " K", " Mil", " Bil", " Tri"];
const BYTES: [&str; 6] = [" B", " KiB", " MiB", " GiB", " TiB", " PiB"];
const DURATIONS: [(f64, &str); 7] = [
    (1., " ns"),
    (1e3, " µs"),
    (1e6, " ms"),
    (1e9, " s"),
    (6e10, " min"),
    (3.6e12, " hour"),
    (8.64e13, " day"),
];

pub(crate) fn step_decimals(step: f64) -> usize {
    (0..10i32)
        .find(|&decimals| {
            let scaled = step * 10f64.powi(decimals);
            (scaled - scaled.round()).abs() < 1e-6
        })
        .map_or(10, |decimals| decimals.unsigned_abs() as usize)
}

pub(crate) fn fixed(value: f64, decimals: usize) -> SharedString {
    format!("{value:.decimals$}").into()
}

pub(crate) fn trimmed(value: f64) -> SharedString {
    with_suffix(value, 3, "")
}

pub(crate) fn short(value: f64) -> SharedString {
    scaled(value, 1000., &SHORT)
}

pub(crate) fn bytes(value: f64) -> SharedString {
    scaled(value, 1024., &BYTES)
}

pub(crate) fn nanoseconds(value: f64) -> SharedString {
    let (scale, suffix) = DURATIONS
        .iter()
        .rev()
        .find(|(scale, _)| value.abs() >= *scale)
        .copied()
        .unwrap_or(DURATIONS[0]);
    with_suffix(value / scale, 2, suffix)
}

pub(crate) fn grouped(value: f64) -> SharedString {
    let digits = format!("{:.0}", value.abs());
    let mut text = String::with_capacity(digits.len() + digits.len() / 3 + 1);
    if value < 0. {
        text.push('-');
    }
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index) % 3 == 0 {
            text.push(',');
        }
        text.push(digit);
    }
    text.into()
}

fn scaled(value: f64, base: f64, suffixes: &[&str]) -> SharedString {
    let mut value = value;
    let mut step = 0;
    while value.abs() >= base && step + 1 < suffixes.len() {
        value /= base;
        step += 1;
    }
    with_suffix(value, 2, suffixes[step])
}

fn with_suffix(value: f64, decimals: usize, suffix: &str) -> SharedString {
    let mut text = format!("{value:.decimals$}");
    let kept = text.trim_end_matches('0').trim_end_matches('.').len();
    text.truncate(kept);
    if text == "-0" {
        text.remove(0);
    }
    text.push_str(suffix);
    text.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_trim_and_scale() {
        assert_eq!(trimmed(1.250), "1.25");
        assert_eq!(trimmed(1500.), "1500");
        assert_eq!(short(1234.), "1.23 K");
        assert_eq!(short(5_600_000.), "5.6 Mil");
        assert_eq!(bytes(2048.), "2 KiB");
        assert_eq!(nanoseconds(1_500_000.), "1.5 ms");
        assert_eq!(nanoseconds(90e9), "1.5 min");
        assert_eq!(grouped(1_234_567.), "1,234,567");
        assert_eq!(step_decimals(2.5), 1);
    }
}
