fn linear_to_db(linear: f64) -> f64 {
    20.0 * linear.log10()
}

/// Converts linear amplitude values to discrete levels based on a threshold and maximum level.
pub fn levels(input: &mut [f64], max: u32, th: f64) -> Vec<u32> {
    let mut output = Vec::with_capacity(input.len());
    let (l, r) = input.split_at_mut(input.len() / 2);
    let input = l.iter().zip(r.iter()).map(|(r, l)| (r + l) / 2.0);
    for i in input {
        let db = linear_to_db(i);
        let index = if db <= th {
            0
        } else if db >= 0.0 {
            max
        } else {
            ((db + th.abs()) / th.abs() * max as f64).round() as u32
        };
        output.push(index);
    }
    output
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_levels() {
        let mut input = vec![2.0, 0.32, 0.1, 0.032, 0.01, 0.001];
        let max = 10;
        let result = levels(&mut input, max, -60.0);
        assert_eq!(result, vec![10, 8, 7, 5, 3, 0]);
    }
}
