// Plan 212 Phase 3C-v2: sig_code encoding/decoding
//
// Encodes FFI function signatures in exported function names.
// Format: auto_{func_name}_{param_types}_{return_type}
// Example: auto_from_str_s_s = from_str(String) -> String
//          auto_random__l   = random() -> i64
//          auto_year_s_i    = year(String) -> i32

use super::sandbox::{FunctionShim, ShimType};

/// Encode a function's signature as sig_code string.
///
/// Returns "{params}_{ret}" where each char represents a ShimType:
/// - v=Void, i=i32, l=i64, f=f64, b=bool, s=CString, p=Pointer
///
/// Examples:
/// - `encode_sig(&[], I64)` → `"_l"`
/// - `encode_sig(&[CString], CString)` → `"s_s"`
/// - `encode_sig(&[I64, I64], I64)` → `"ll_l"`
pub fn encode_sig(params: &[ShimType], ret: ShimType) -> String {
    let param_chars: String = params.iter().map(type_to_char).collect();
    let ret_char = type_to_char(&ret);
    format!("{}_{}", param_chars, ret_char)
}

/// Decode a sig_code string back to (param_types, return_type).
///
/// Input format: "{params}_{ret}" (e.g., "s_s", "_l", "ll_l")
/// Returns (Vec<ShimType>, ShimType).
pub fn decode_sig(code: &str) -> (Vec<ShimType>, ShimType) {
    let (params_str, ret_str) = code.split_once('_').unwrap_or(("", "s"));
    let param_types: Vec<ShimType> = params_str.chars().map(char_to_type).collect();
    let return_type = char_to_type(ret_str.chars().next().unwrap_or('s'));
    (param_types, return_type)
}

/// Parse an exported function name to extract (func_name, sig_code).
///
/// "auto_from_str_s_s" → Some(("from_str", "s_s"))
/// "auto_random__l"    → Some(("random", "_l"))
/// "auto__sig_manifest" → None (reserved)
/// "other_func"        → None (not an auto_ export)
pub fn parse_exported_name(name: &str) -> Option<(String, String)> {
    let rest = name.strip_prefix("auto_")?;

    // Skip reserved names
    if rest.starts_with("_sig_manifest") {
        return None;
    }

    // Find the last two underscore-separated segments that form sig_code: "{params}_{ret}"
    // Strategy: scan from the right, the last "_" separates params from ret.
    // The second-to-last "_" separates func_name from sig_code.
    //
    // "from_str_s_s" → func="from_str", sig="s_s"
    // "random__l"    → func="random",   sig="_l"
    // "year_s_i"     → func="year",     sig="s_i"
    //
    // Algorithm: find the rightmost '_', that's the ret type char.
    // Then find the next '_' to the left, that's the start of sig_code.

    let last_underscore = rest.rfind('_')?;
    let ret_char = &rest[last_underscore + 1..];

    // ret_char must be a single valid sig_code char
    if ret_char.len() != 1 || !is_valid_sig_char(ret_char.chars().next()?) {
        return None;
    }

    let before_ret = &rest[..last_underscore];
    let sep = before_ret.rfind('_')?;

    let func_name = &before_ret[..sep];
    let param_chars = &before_ret[sep + 1..];

    // Validate param_chars
    if !param_chars.chars().all(is_valid_sig_char) {
        return None;
    }

    Some((
        func_name.to_string(),
        format!("{}_{}", param_chars, ret_char),
    ))
}

/// Build the exported function name from func_name and sig_code.
///
/// ("from_str", "s_s") → "auto_from_str_s_s"
/// ("random", "_l")    → "auto_random__l"
pub fn build_exported_name(func_name: &str, sig_code: &str) -> String {
    format!("auto_{}_{}", func_name, sig_code)
}

/// Generate sig_code for a FunctionShim.
pub fn shim_to_sig_code(shim: &FunctionShim) -> String {
    encode_sig(&shim.param_types, shim.return_type)
}

/// Generate the manifest JSON string for a set of shims.
///
/// Returns JSON like: {"from_str":"s_s","to_string":"s_s","random":"_l"}
pub fn build_manifest_json(shims: &[FunctionShim]) -> String {
    let entries: Vec<String> = shims
        .iter()
        .map(|s| {
            let sig = shim_to_sig_code(s);
            format!(r#""{}":"{}""#, s.name, sig)
        })
        .collect();
    format!("{{{}}}", entries.join(","))
}

/// Parse manifest JSON back to a map of func_name → sig_code.
///
/// Input: {"from_str":"s_s","to_string":"s_s"}
/// Output: {"from_str": "s_s", "to_string": "s_s"}
pub fn parse_manifest_json(json: &str) -> Vec<(String, String)> {
    // Simple JSON parser for flat object of string:string pairs.
    // Avoids pulling in a full JSON parser dependency.
    let trimmed = json.trim().trim_start_matches('{').trim_end_matches('}');
    if trimmed.is_empty() {
        return vec![];
    }

    let mut result = vec![];
    // Split by "," but handle the fact that values don't contain commas
    for entry in trimmed.split(',') {
        let entry = entry.trim();
        if let Some((key, value)) = entry.split_once(':') {
            let key = key.trim().trim_matches('"').to_string();
            let value = value.trim().trim_matches('"').to_string();
            if !key.is_empty() {
                result.push((key, value));
            }
        }
    }
    result
}

// -- Internal helpers --

fn type_to_char(t: &ShimType) -> char {
    match t {
        ShimType::Void => 'v',
        ShimType::I32 => 'i',
        ShimType::I64 => 'l',
        ShimType::F64 => 'f',
        ShimType::Bool => 'b',
        ShimType::CString => 's',
    }
}

fn char_to_type(c: char) -> ShimType {
    match c {
        'v' => ShimType::Void,
        'i' => ShimType::I32,
        'l' => ShimType::I64,
        'f' => ShimType::F64,
        'b' => ShimType::Bool,
        's' => ShimType::CString,
        'p' => ShimType::CString, // pointer fallback to CString for now
        _ => ShimType::CString,
    }
}

fn is_valid_sig_char(c: char) -> bool {
    matches!(c, 'v' | 'i' | 'l' | 'f' | 'b' | 's' | 'p')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_roundtrip() {
        let cases: Vec<(Vec<ShimType>, ShimType)> = vec![
            (vec![], ShimType::I64),                          // () → i64
            (vec![ShimType::CString], ShimType::CString),     // String → String
            (vec![ShimType::I64, ShimType::I64], ShimType::I64), // (i64,i64) → i64
            (vec![ShimType::CString], ShimType::I32),         // String → i32
            (vec![], ShimType::Void),                          // () → void
            (vec![ShimType::Bool], ShimType::Bool),           // bool → bool
        ];

        for (params, ret) in cases {
            let sig_code = encode_sig(&params, ret);
            let (decoded_params, decoded_ret) = decode_sig(&sig_code);
            assert_eq!(decoded_params, params, "params mismatch for sig_code: {}", sig_code);
            assert_eq!(decoded_ret, ret, "ret mismatch for sig_code: {}", sig_code);
        }
    }

    #[test]
    fn test_parse_exported_name() {
        assert_eq!(
            parse_exported_name("auto_from_str_s_s"),
            Some(("from_str".to_string(), "s_s".to_string()))
        );
        assert_eq!(
            parse_exported_name("auto_random__l"),
            Some(("random".to_string(), "_l".to_string()))
        );
        assert_eq!(
            parse_exported_name("auto_year_s_i"),
            Some(("year".to_string(), "s_i".to_string()))
        );
        assert_eq!(
            parse_exported_name("auto_gen_range_ll_l"),
            Some(("gen_range".to_string(), "ll_l".to_string()))
        );
        // Reserved name
        assert_eq!(parse_exported_name("auto__sig_manifest"), None);
        // Non-auto name
        assert_eq!(parse_exported_name("other_func"), None);
    }

    #[test]
    fn test_build_exported_name() {
        assert_eq!(build_exported_name("from_str", "s_s"), "auto_from_str_s_s");
        assert_eq!(build_exported_name("random", "_l"), "auto_random__l");
    }

    #[test]
    fn test_manifest_roundtrip() {
        let shims = vec![
            FunctionShim::string_to_string("from_str"),
            FunctionShim::string_to_string("to_string"),
            FunctionShim::from_sig_str("random", ":l"),
        ];

        let json = build_manifest_json(&shims);
        let parsed = parse_manifest_json(&json);

        assert_eq!(parsed.len(), 3);
        assert_eq!(parsed[0], ("from_str".to_string(), "s_s".to_string()));
        assert_eq!(parsed[1], ("to_string".to_string(), "s_s".to_string()));
        assert_eq!(parsed[2], ("random".to_string(), "_l".to_string()));
    }
}
