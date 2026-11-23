#[rustfmt::skip]
const SIZES: [f64; 94] = [
    1e-9, 2e-9, 2.5e-9, 4e-9, 5e-9,
    1e-8, 2e-8, 2.5e-8, 4e-8, 5e-8,
    1e-7, 2e-7, 2.5e-7, 4e-7, 5e-7,
    1e-6, 2e-6, 2.5e-6, 4e-6, 5e-6,
    1e-5, 2e-5, 2.5e-5, 4e-5, 5e-5,
    1e-4, 2e-4, 2.5e-4, 4e-4, 5e-4,
    1e-3, 2e-3, 2.5e-3, 4e-3, 5e-3,
    1e-2, 2e-2, 2.5e-2, 4e-2, 5e-2,
    1e-1, 2e-1, 2.5e-1, 4e-1, 5e-1,
    1., 2., 4., 5.,
    1e1, 2e1, 2.5e1, 4e1, 5e1,
    1e2, 2e2, 2.5e2, 4e2, 5e2,
    1e3, 2e3, 2.5e3, 4e3, 5e3,
    1e4, 2e4, 2.5e4, 4e4, 5e4,
    1e5, 2e5, 2.5e5, 4e5, 5e5,
    1e6, 2e6, 2.5e6, 4e6, 5e6,
    1e7, 2e7, 2.5e7, 4e7, 5e7,
    1e8, 2e8, 2.5e8, 4e8, 5e8,
    1e9, 2e9, 2.5e9, 4e9, 5e9,
];

pub(crate) type Row = Box<[f64]>;

pub(crate) fn round(value: f64, decimals: i32) -> f64 {
    let power = 10f64.powi(decimals);
    (value * power).round() / power
}

pub(crate) fn auto_size(sorted: &[f64], count: usize) -> f64 {
    let (Some(first), Some(last)) = (sorted.first(), sorted.last()) else {
        return 1.;
    };
    let smallest = if sorted.len() == 1 {
        1.
    } else {
        sorted
            .windows(2)
            .map(|pair| pair[1] - pair[0])
            .filter(|delta| *delta != 0.)
            .fold(f64::INFINITY, f64::min)
    };
    let target = (last - first) / count.max(1) as f64;
    SIZES
        .iter()
        .copied()
        .find(|size| target < *size && *size >= smallest)
        .unwrap_or(1.)
}

pub(crate) fn start(value: f64, size: f64, offset: f64) -> f64 {
    round(((value - offset) / size).floor() * size + offset, 9)
}

pub(crate) fn decimals(size: f64) -> usize {
    let text = size.to_string();
    text.split_once('.')
        .map_or(0, |(_, fraction)| fraction.len())
}

pub(crate) fn count(series: &[Vec<f64>], size: f64, offset: f64, combine: bool) -> Vec<Row> {
    let mut keyed: Vec<(f64, usize)> = series
        .iter()
        .enumerate()
        .flat_map(|(index, values)| {
            values
                .iter()
                .map(move |value| (start(*value, size, offset), index))
        })
        .collect();
    keyed.sort_by(|a, b| a.0.total_cmp(&b.0));
    let width = if combine { 1 } else { series.len() };
    let mut rows: Vec<Row> = Vec::new();
    for (bucket, index) in keyed {
        if rows
            .last()
            .is_none_or(|row| row[0].to_bits() != bucket.to_bits())
        {
            let mut row = vec![0.; width + 2].into_boxed_slice();
            row[0] = bucket;
            row[1] = bucket + size;
            rows.push(row);
        }
        if let Some(row) = rows.last_mut() {
            row[2 + if combine { 0 } else { index }] += 1.;
        }
    }
    rows
}

pub(crate) fn counted(
    starts: &[f64],
    ends: &[f64],
    counts: &[Vec<f64>],
    combine: bool,
) -> Vec<Row> {
    let guess = |edges: &[f64]| match edges {
        [first, second, ..] => round(second - first, 6),
        _ => 1.,
    };
    let width = if combine { 1 } else { counts.len() };
    (0..starts.len().max(ends.len()))
        .filter_map(|index| {
            let (low, high) = match (starts.get(index), ends.get(index)) {
                (Some(low), Some(high)) => (*low, *high),
                (Some(low), None) => (*low, low + guess(starts)),
                (None, Some(high)) => (high - guess(ends), *high),
                (None, None) => return None,
            };
            if !low.is_finite() || !high.is_finite() {
                return None;
            }
            let mut row = vec![0.; width + 2].into_boxed_slice();
            row[0] = low;
            row[1] = high;
            for (series, values) in counts.iter().enumerate() {
                let value = values.get(index).copied().unwrap_or(0.);
                let value = if value.is_finite() { value } else { 0. };
                row[2 + if combine { 0 } else { series }] += value;
            }
            Some(row)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_the_first_size_above_the_target() {
        let values: Vec<f64> = (0..=100).map(f64::from).collect();
        assert!((auto_size(&values, 30) - 4.).abs() < 1e-12);
        assert!((auto_size(&values, 10) - 20.).abs() < 1e-12);
        assert!((auto_size(&[0., 0.5, 1.], 30) - 0.5).abs() < 1e-12);
    }

    #[test]
    fn buckets_with_an_offset() {
        assert!((start(7.3, 5., 0.) - 5.).abs() < 1e-12);
        assert!((start(7.3, 5., 1.) - 6.).abs() < 1e-12);
        assert!((start(-0.1, 1., 0.) + 1.).abs() < 1e-12);
    }

    #[test]
    fn counts_each_series_and_combines() {
        let series = [vec![1., 2., 11.], vec![3., 12., 13.]];
        let rows = count(&series, 10., 0., false);
        assert_eq!(&*rows[0], &[0., 10., 2., 1.]);
        assert_eq!(&*rows[1], &[10., 20., 1., 2.]);
        let combined = count(&series, 10., 0., true);
        assert_eq!(&*combined[1], &[10., 20., 3.]);
    }

    #[test]
    fn guesses_missing_edges() {
        let rows = counted(&[0., 5., 10.], &[], &[vec![1., 2., 3.]], false);
        assert_eq!(&*rows[2], &[10., 15., 3.]);
        assert_eq!(decimals(0.25), 2);
        assert_eq!(decimals(5.), 0);
    }
}
