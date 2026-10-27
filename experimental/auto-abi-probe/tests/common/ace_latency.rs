pub fn p90_ms(windows: &[f64]) -> Option<f64> {
    if windows.is_empty() || windows.iter().any(|w| !w.is_finite() || *w < 0.0) {
        return None;
    }
    let mut v = windows.to_vec();
    v.sort_by(f64::total_cmp);
    Some(v[v.len() - v.len() / 10 - 1])
}

#[test]
fn p90_nearest_rank_boundaries() {
    let mut ten = vec![2000.0; 10];
    ten[9] = 2001.0;
    assert_eq!(p90_ms(&ten), Some(2000.0));
    ten[8] = 2001.0;
    assert_eq!(p90_ms(&ten), Some(2001.0));
    let mut thirty = vec![2000.0; 30];
    thirty[27..].fill(2001.0);
    assert_eq!(p90_ms(&thirty), Some(2000.0));
    thirty[26] = 2001.0;
    assert_eq!(p90_ms(&thirty), Some(2001.0));
    assert!(p90_ms(&[]).is_none());
    assert!(p90_ms(&[f64::NAN]).is_none());
    assert!(p90_ms(&[f64::INFINITY]).is_none());
    assert_eq!(p90_ms(&[0.0]), Some(0.0));
    assert!(p90_ms(&[-1.0]).is_none());
}
