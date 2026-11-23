mod grid;
mod model;
mod sparkline;

pub(crate) use grid::cells;
pub(crate) use model::{Item, Times, Values};
pub(crate) use sparkline::Sparkline;

pub(crate) fn percent_text(change: f64) -> String {
    let magnitude = change.abs();
    let digits = if magnitude == 0. {
        0
    } else {
        (2 - magnitude.log10().floor() as i32).max(0) as usize
    };
    let text = format!("{change:.digits$}");
    let text = if text.contains('.') {
        text.trim_end_matches('0').trim_end_matches('.')
    } else {
        &text
    };
    format!("{text}%")
}

#[cfg(test)]
mod tests {
    use super::percent_text;

    #[test]
    fn three_significant_digits() {
        assert_eq!(percent_text(12.345), "12.3%");
        assert_eq!(percent_text(-0.5), "-0.5%");
        assert_eq!(percent_text(150.), "150%");
        assert_eq!(percent_text(0.), "0%");
        assert_eq!(percent_text(1234.), "1234%");
    }
}
