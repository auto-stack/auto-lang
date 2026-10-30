//! Shared ACE development-stage latency percentile for integration tests.

/// Nearest-rank P90. All raw windows must be finite and non-negative; callers retain their path-specific zero rules.
pub(super) fn p90_ms(windows: &[f64]) -> Result<f64, &'static str> {
    if windows.is_empty() || windows.iter().any(|w| !w.is_finite() || *w < 0.0) {
        return Err("missing or invalid latency sample");
    }
    let mut sorted = windows.to_vec();
    sorted.sort_by(f64::total_cmp);
    // ceil(0.9 * n) == n - floor(n / 10), one-based.
    Ok(sorted[sorted.len() - sorted.len() / 10 - 1])
}

#[test]
fn nearest_rank_boundaries() {
    let mut ten = vec![2000.0; 10];
    ten[9] = 2001.0;
    assert!(p90_ms(&ten).unwrap() <= 2000.0);
    ten[8] = 2001.0;
    assert!(p90_ms(&ten).unwrap() > 2000.0);
    let mut thirty = vec![2000.0; 30];
    thirty[27..].fill(2001.0);
    assert!(p90_ms(&thirty).unwrap() <= 2000.0);
    thirty[26] = 2001.0;
    assert!(p90_ms(&thirty).unwrap() > 2000.0);
    assert!(p90_ms(&[]).is_err());
    assert!(p90_ms(&[1.0, f64::NAN]).is_err());
    assert!(p90_ms(&[1.0, f64::INFINITY]).is_err());
    assert_eq!(p90_ms(&[0.0]), Ok(0.0));
    assert!(p90_ms(&[-1.0]).is_err());
}
