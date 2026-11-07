//! Lossy display of captured tool bytes without an uncharged whole-output copy.
use ace_source_query::resources::{Charge, Reservation, Resources};
use std::borrow::Cow;

pub(crate) struct Text<'a> {
    pub(crate) value: Cow<'a, str>,
    // The owned replacement string drops before its admission ticket.
    pub(crate) _allocation: Option<Reservation>,
}

pub(crate) fn display<'a>(
    bytes: &'a [u8],
    resources: Option<&Resources>,
) -> Result<Text<'a>, String> {
    if let Ok(valid) = std::str::from_utf8(bytes) {
        return Ok(Text {
            value: Cow::Borrowed(valid),
            _allocation: None,
        });
    }
    // Use the standard library's exact invalid-subsequence boundaries, just
    // as String::from_utf8_lossy does. Count first so no growth or old/new
    // buffer coexistence is hidden in its infallible allocation path.
    let mut size = 0usize;
    for chunk in bytes.utf8_chunks() {
        size = size
            .checked_add(chunk.valid().len())
            .and_then(|n| n.checked_add(if chunk.invalid().is_empty() { 0 } else { 3 }))
            .ok_or_else(|| "C output text capacity overflow".to_owned())?;
    }
    let allocation = resources
        .map(|r| r.reserve(Charge::io(size)).map_err(|e| e.to_string()))
        .transpose()?;
    let mut value = String::new();
    value
        .try_reserve_exact(size)
        .map_err(|e| format!("C output text allocation: {e}"))?;
    for chunk in bytes.utf8_chunks() {
        value.push_str(chunk.valid());
        if !chunk.invalid().is_empty() {
            value.push('\u{fffd}');
        }
    }
    debug_assert_eq!(value.len(), size);
    Ok(Text {
        value: Cow::Owned(value),
        _allocation: allocation,
    })
}
