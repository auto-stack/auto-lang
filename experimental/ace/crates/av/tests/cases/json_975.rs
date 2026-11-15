// ---------------------------------------------------------------------------
// PLAN-975 T-02 — the tracked `stdlib/ace/std/json.at` ENCODE family
// (PLAN-1041 inline convention) verbatim into a console single source.
// Independent hand oracles: escaping (quote/backslash/newline/tab),
// int (incl. negative), bool/null leaves, kv composition, and the
// structural-balance scanner (nested object positive + broken
// negatives: unterminated string / extra close / leftover open).
// ---------------------------------------------------------------------------

use ace_av_core::platform_io::Win32PlatformIo;

#[test]
fn json975_encode_family_inline() {
    let stdlib_json = include_str!("../../../../../../stdlib/ace/std/json.at");
    let src = format!(
        r#"{stdlib_json}
fn main() int {{
    // escaping: quote, backslash, newline, tab survive; plain text kept
    let esc Str = json_str("a\"b\\c\nd\te")
    let want_esc Str = "\"a\\\"b\\\\c\\nd\\te\""
    if esc != want_esc {{ return 1 }}
    // ints: zero, positive, negative
    let z Str = json_int(0)
    let p Str = json_int(42)
    let m Str = json_int(-7)
    let wz Str = "0"
    let wp Str = "42"
    let wm Str = "-7"
    if z != wz {{ return 2 }}
    if p != wp {{ return 3 }}
    if m != wm {{ return 4 }}
    // bool / null
    let t Str = json_bool(true)
    let f Str = json_bool(false)
    let nl Str = json_null()
    let wt Str = "true"
    let wf Str = "false"
    let wn Str = "null"
    if t != wt {{ return 5 }}
    if f != wf {{ return 6 }}
    if nl != wn {{ return 7 }}
    // kv + object/array composition: {{"id":42,"ok":true,"note":"hi"}}
    let kv1 Str = json_kv("id", json_int(42))
    let kv2 Str = json_kv("ok", json_bool(true))
    let kv3 Str = json_kv("note", json_str("hi"))
    let comma Str = ","
    let doc Str = json_object_open() + kv1 + comma + kv2 + comma + kv3 + json_object_close()
    let want_doc Str = "{{\"id\":42,\"ok\":true,\"note\":\"hi\"}}"
    if doc != want_doc {{ return 8 }}
    // array: [1,-2,"x"]
    let arr Str = json_array_open() + json_int(1) + comma + json_int(-2) + comma + json_str("x") + json_array_close()
    let want_arr Str = "[1,-2,\"x\"]"
    if arr != want_arr {{ return 9 }}
    // structural balance: valid nested doc true
    if json_structurally_balanced(doc) != true {{ return 10 }}
    if json_structurally_balanced(arr) != true {{ return 11 }}
    // negatives: unterminated string
    let bad1 Str = "[\"abc]"
    if json_structurally_balanced(bad1) != false {{ return 12 }}
    // extra close
    let bad2 Str = "[1]]"
    if json_structurally_balanced(bad2) != false {{ return 13 }}
    // leftover open
    let bad3 Str = "{{\"a\":1"
    if json_structurally_balanced(bad3) != false {{ return 14 }}
    return 0
}}
"#
    );
    let mut backend = Win32PlatformIo::default();
    let grants: Vec<String> = vec!["platform.mem".into()];
    let program = ace_av::console_host::compile(&src, "d_main").expect("compile json source");
    let (out, _t, _s) = program
        .run_platform(
            &grants,
            &ace_av_core::av::ExecLimits::default(),
            &ace_av_core::av::CancelFlag::new(false),
            None,
            &mut backend,
        )
        .expect("run platform json");
    assert!(
        matches!(
            &out,
            ace_av_core::contract::ExecOutcome::Success {
                value: ace_av_core::contract::Value::I32(0),
                ..
            }
        ),
        "json encode family oracles, got {out:?}"
    );
}
