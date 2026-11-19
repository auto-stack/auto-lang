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

// ---------------------------------------------------------------------------
// PLAN-975 B1 — the char->Str materialization primitive receipt
// (`c.to(Str)`): fresh owned 1-scalar buffers, multi-byte UTF-8 shapes,
// cross-function owned returns, and composition with views (slice) and
// concat. Hand oracles: scalar counts, byte lengths, codepoints.
// ---------------------------------------------------------------------------

#[test]
fn json975_char_to_str_primitive() {
    let src = r#"
fn mk(cp int) Str {
    return char.from_codepoint(cp).to(Str)
}
fn main() int {
    let a Str = mk(65)
    if a.len() != 1 { return 1 }
    if a.char_at(0).to_codepoint() != 65 { return 2 }
    // concat of materialized chars equals the literal (byte equality)
    let hi Str = mk(104) + mk(105)
    let lit Str = "hi"
    if hi != lit { return 3 }
    // multi-byte: U+4E2D = 20013 (3 UTF-8 bytes, 1 scalar)
    let zh Str = mk(20013)
    if zh.len() != 1 { return 4 }
    if zh.len_bytes() != 3 { return 5 }
    if zh.char_at(0).to_codepoint() != 20013 { return 6 }
    let mixed Str = mk(20013) + mk(25991) + mk(111) + mk(107)
    if mixed.len() != 4 { return 7 }
    if mixed.len_bytes() != 8 { return 8 }
    if mixed.char_at(2).to_codepoint() != 111 { return 9 }
    // 2-byte scalar U+00E9 = 233
    let e Str = mk(233)
    if e.len_bytes() != 2 { return 10 }
    // 4-byte scalar U+1F600 = 128512
    let em Str = mk(128512)
    if em.len() != 1 { return 11 }
    if em.len_bytes() != 4 { return 12 }
    // materialized char + view slice + materialized char
    let s Str = "hello"
    let combo Str = mk(33) + s.slice(1, 3) + mk(63)
    let want Str = "!el?"
    if combo != want { return 13 }
    return 0
}
"#;
    let mut backend = Win32PlatformIo::default();
    let grants: Vec<String> = vec!["platform.mem".into()];
    let program = ace_av::console_host::compile(src, "d_main").expect("compile primitive source");
    let (out, _t, _s) = program
        .run_platform(
            &grants,
            &ace_av_core::av::ExecLimits::default(),
            &ace_av_core::av::CancelFlag::new(false),
            None,
            &mut backend,
        )
        .expect("run platform primitive");
    assert!(
        matches!(
            &out,
            ace_av_core::contract::ExecOutcome::Success {
                value: ace_av_core::contract::Value::I32(0),
                ..
            }
        ),
        "char.to(Str) materialization oracles, got {out:?}"
    );
}

// ---------------------------------------------------------------------------
// PLAN-975 T-02 decode family — json.at's parse face inline: dotted-path
// kind routing, typed extraction (str with EVERY escape incl. \uXXXX via
// B1, int incl. negative, bool), full recursive validity, and the named
// negatives (malformed forms, trailing garbage, wrong-kind reads).
// ---------------------------------------------------------------------------

#[test]
fn json975_decode_family_inline() {
    let stdlib_json = include_str!("../../../../../../stdlib/ace/std/json.at");
    let src = format!(
        r#"{stdlib_json}
fn main() int {{
    let doc str = "{{\"user\":{{\"name\":\"Michi\",\"age\":7,\"tags\":[\"a\"],\"active\":true,\"note\":null}},\"msg\":\"h\\u0069\",\"esc\":\"a\\\"b\\\\c\\nd\",\"pos\":42,\"neg\":-7,\"f\":1.5,\"zero\":0}}"
    // kind routing (-1/0/1/2/3/4/5/6/7)
    if json_kind_at(doc, "user") != 6 {{ return 1 }}
    if json_kind_at(doc, "user.name") != 4 {{ return 2 }}
    if json_kind_at(doc, "user.age") != 3 {{ return 3 }}
    if json_kind_at(doc, "user.tags") != 5 {{ return 4 }}
    if json_kind_at(doc, "user.active") != 2 {{ return 5 }}
    if json_kind_at(doc, "user.note") != 1 {{ return 6 }}
    if json_kind_at(doc, "nope") != 0 {{ return 9 }}
    if json_kind_at(doc, "msg") != 4 {{ return 7 }}
    if json_kind_at(doc, "f") != 7 {{ return 8 }}
    // nested string decode (V-DETAIL-01)
    let name Str = json_parse_str(doc, "user.name")
    let want_name Str = "Michi"
    if name != want_name {{ return 10 }}
    // \uXXXX through the B1 primitive: "h\u0069" -> "hi"
    let msg Str = json_parse_str(doc, "msg")
    let want_msg Str = "hi"
    if msg != want_msg {{ return 11 }}
    // full fixed-escape set: value a " b \ c LF d (7 scalars)
    let esc Str = json_parse_str(doc, "esc")
    if esc.len() != 7 {{ return 12 }}
    if esc.char_at(0).to_codepoint() != 97 {{ return 13 }}
    if esc.char_at(1).to_codepoint() != 34 {{ return 14 }}
    if esc.char_at(2).to_codepoint() != 98 {{ return 15 }}
    if esc.char_at(3).to_codepoint() != 92 {{ return 16 }}
    if esc.char_at(4).to_codepoint() != 99 {{ return 17 }}
    if esc.char_at(5).to_codepoint() != 10 {{ return 18 }}
    if esc.char_at(6).to_codepoint() != 100 {{ return 19 }}
    // typed extraction
    if json_parse_int(doc, "user.age") != 7 {{ return 20 }}
    if json_parse_int(doc, "pos") != 42 {{ return 21 }}
    if json_parse_int(doc, "neg") != -7 {{ return 22 }}
    if json_parse_int(doc, "zero") != 0 {{ return 23 }}
    if json_parse_int(doc, "f") != 0 {{ return 24 }}
    if json_parse_int(doc, "user.name") != 0 {{ return 25 }}
    if json_parse_bool(doc, "user.active") != 1 {{ return 26 }}
    if json_parse_bool(doc, "user.age") != -1 {{ return 27 }}
    // status pairing
    if json_parse_str_ok(doc, "user.name") != 1 {{ return 28 }}
    if json_parse_str_ok(doc, "nope") != 0 {{ return 29 }}
    if json_parse_str_ok(doc, "user.age") != -1 {{ return 30 }}
    // full recursive validity
    if json_is_valid(doc) != true {{ return 31 }}
    let v2 str = "[1, -2.5e3]"
    if json_is_valid(v2) != true {{ return 32 }}
    let v3 str = "{{\"a\":{{\"b\":[1,{{\"c\":null}}]}}}}"
    if json_is_valid(v3) != true {{ return 33 }}
    let v4 str = "\"\""
    if json_is_valid(v4) != true {{ return 34 }}
    let v5 str = "0"
    if json_is_valid(v5) != true {{ return 35 }}
    let v6 str = "  {{\"a\": 1}}  "
    if json_is_valid(v6) != true {{ return 36 }}
    // negatives: malformed forms and trailing garbage
    let b1 str = "{{\"a\":}}"
    if json_is_valid(b1) != false {{ return 37 }}
    let b2 str = "{{\"a\" 1}}"
    if json_is_valid(b2) != false {{ return 38 }}
    let b3 str = "[1,]"
    if json_is_valid(b3) != false {{ return 39 }}
    let b4 str = "{{\"a\":1"
    if json_is_valid(b4) != false {{ return 40 }}
    let b5 str = "\"ab"
    if json_is_valid(b5) != false {{ return 41 }}
    let b6 str = "tru"
    if json_is_valid(b6) != false {{ return 42 }}
    let b7 str = "01"
    if json_is_valid(b7) != false {{ return 43 }}
    let b8 str = "{{\"a\":1}} x"
    if json_is_valid(b8) != false {{ return 44 }}
    let b9 str = "1.5.5"
    if json_is_valid(b9) != false {{ return 45 }}
    // malformed document -> kind_at reports -1 (not 0/other)
    if json_kind_at(b1, "a") != -1 {{ return 46 }}
    // missing string -> "" pairs with kind 0 (owned literal context)
    let empty Str = ""
    if json_parse_str(doc, "nope") != empty {{ return 47 }}
    // UTF-16 surrogate pair decode + lone-surrogate non-trap + invalid escapes + i32::MIN
    let surr str = "{{\"emoji\":\"\\uD83D\\uDE00\",\"lone\":\"\\uD800\",\"min\":-2147483648}}"
    let emoji Str = json_parse_str(surr, "emoji")
    let want_emoji Str = "😀"
    if emoji != want_emoji {{ return 48 }}
    if json_parse_str(surr, "lone") != empty {{ return 49 }}
    if json_parse_int(surr, "min") != -2147483648 {{ return 50 }}
    let b10 str = "{{\"bad\":\"\\q\"}}"
    if json_is_valid(b10) != false {{ return 51 }}
    let b11 str = "{{\"bad\":\"\\uZZZZ\"}}"
    if json_is_valid(b11) != false {{ return 52 }}
    return 0
}}
"#
    );
    let mut backend = Win32PlatformIo::default();
    let grants: Vec<String> = vec!["platform.mem".into()];
    let program = ace_av::console_host::compile(&src, "d_main").expect("compile decode source");
    let (out, _t, _s) = program
        .run_platform(
            &grants,
            &ace_av_core::av::ExecLimits::default(),
            &ace_av_core::av::CancelFlag::new(false),
            None,
            &mut backend,
        )
        .expect("run platform decode");
    assert!(
        matches!(
            &out,
            ace_av_core::contract::ExecOutcome::Success {
                value: ace_av_core::contract::Value::I32(0),
                ..
            }
        ),
        "json decode family oracles, got {out:?}"
    );
}

// ---------------------------------------------------------------------------
// PLAN-975 T-02 coverage increment — the B1 `c.to(Str)` primitive on the
// A2X ends (AC-COVERAGE follow-up): the SAME public source through the
// packages producer chain, observed on AV (value), native/mn (exit
// code), Rust (rustc exe stdout) and C (cl exe stdout). Hand oracle:
// materialized 'A'(1B) + U+4E2D(3B) + U+1F600(4B) give lens 134, the
// concatenation equals the literal "A中😀" (+7) -> 134*10+7 = 1347.
// ---------------------------------------------------------------------------

#[cfg(feature = "test-toolchain")]
#[test]
fn json975_char_to_str_a2x_ends() {
    use ace_a2x_c::CTargetProvider;
    use ace_a2x_rust::RustTargetProvider;
    use ace_a2x_store::modules::packages::PackageA2xTarget;
    use ace_av::modules::packages::PackageAvTarget;
    use ace_core::source::cst::FrontBudget;
    use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};
    use ace_source_query::modules::packages::{compile_package_product, stdlib_inputs};

    let src = r#"use std.text.byte_len
use std.text.equal
fn mk(cp int) Str {
    return char.from_codepoint(cp).to(Str)
}
fn sel(c bool, t int, f int) int {
    if (c) { return t }
    return f
}
fn main() int {
    let a Str = mk(65)
    let zh Str = mk(20013)
    let em Str = mk(128512)
    let lens int = byte_len(a.to(str)) * 100 + byte_len(zh.to(str)) * 10 + byte_len(em.to(str))
    let all Str = a + zh + em
    let eq int = sel(equal(all.to(str), "A中😀"), 7, 0)
    return lens * 10 + eq
}"#;

    let mut inputs = stdlib_inputs();
    inputs.push(PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id: "main".into(),
        source: src.into(),
        provider: None,
    });
    let product = compile_package_product(
        &inputs,
        "app::main",
        &[PACKAGES_CAPABILITY.into()],
        FrontBudget::WORK_OWNED_64MIB,
    )
    .expect("compiles on the packages chain");

    // ---- AV
    let av_dir = std::env::temp_dir().join(format!("p975-c2s-av-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&av_dir);
    std::fs::create_dir_all(&av_dir).unwrap();
    assert_eq!(
        PackageAvTarget::cold(&av_dir, &product)
            .unwrap()
            .run_value(),
        Ok(ace_av_core::contract::Value::I32(1347))
    );
    let _ = std::fs::remove_dir_all(&av_dir);

    // ---- Rust (A2R): rustc-built exe stdout
    let store = std::env::temp_dir().join(format!("p975-c2s-rs-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&store);
    let rust = PackageA2xTarget::cold(
        &store,
        "pkg975c2s",
        "app",
        16 << 20,
        RustTargetProvider::v1(),
        &product,
    )
    .unwrap();
    let rust_dir = store.join("build");
    std::fs::create_dir_all(&rust_dir).unwrap();
    for (path, content) in &rust.current.files {
        std::fs::write(rust_dir.join(path), content).unwrap();
    }
    let rust_exe = rust_dir.join("pkg.exe");
    let mut rust_cmd =
        std::process::Command::new(std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()));
    rust_cmd
        .args(["--edition", "2021", "main.rs", "-o"])
        .arg(&rust_exe)
        .current_dir(&rust_dir);
    let rust_build = rust_cmd.output().expect("rustc spawn");
    assert!(
        rust_build.status.success(),
        "{}",
        String::from_utf8_lossy(&rust_build.stderr)
    );
    let rust_run = std::process::Command::new(&rust_exe)
        .current_dir(&rust_dir)
        .output()
        .expect("run rust exe");
    assert!(
        rust_run.status.success(),
        "{}",
        String::from_utf8_lossy(&rust_run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&rust_run.stdout).trim(),
        "value:1347",
        "the A2R end materializes the same bytes"
    );
    let _ = std::fs::remove_dir_all(&store);

    // ---- C (AC): cl-built exe stdout
    let c_store = std::env::temp_dir().join(format!("p975-c2s-c-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&c_store);
    let c = PackageA2xTarget::cold(
        &c_store,
        "pkg975c2s",
        "app",
        16 << 20,
        CTargetProvider::v1(),
        &product,
    )
    .unwrap();
    let c_dir = c_store.join("build");
    std::fs::create_dir_all(&c_dir).unwrap();
    for (path, content) in &c.current.files {
        std::fs::write(c_dir.join(path), content).unwrap();
    }
    std::fs::write(c_dir.join("manifest.json"), &c.current.manifest).unwrap();
    let c_exe = c_dir.join("pkg_c.exe");
    ace_a2x_worker::crun::compile_c_fileset(&c_dir, &c_exe)
        .unwrap_or_else(|e| panic!("cl failed: {e}"));
    let c_run = std::process::Command::new(&c_exe)
        .current_dir(&c_dir)
        .output()
        .expect("run c exe");
    assert!(
        c_run.status.success(),
        "{}",
        String::from_utf8_lossy(&c_run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&c_run.stdout).trim(),
        "value:1347",
        "the AC end materializes the same bytes"
    );
    let _ = std::fs::remove_dir_all(&c_store);

    // ---- native/mn exit code
    let mn_dir = std::env::temp_dir().join(format!("p975-c2s-mn-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&mn_dir);
    std::fs::create_dir_all(&mn_dir).unwrap();
    let mut driver = ace_native::driver::MnDriver::ephemeral();
    let built = driver
        .build_checked(
            &product.compiled.checked,
            "d_main",
            &mn_dir,
            None,
            None,
            975,
        )
        .expect("mn build");
    let code = std::process::Command::new(&built.exe_path)
        .current_dir(&mn_dir)
        .status()
        .expect("run mn exe")
        .code()
        .unwrap_or(-1);
    assert_eq!(code, 1347, "the native/mn end materializes the same bytes");
    let _ = std::fs::remove_dir_all(&mn_dir);
}

// ---------------------------------------------------------------------------
// PLAN-975 T-03/T-08 — array segments in dotted paths (the legacy
// JsonValue.get_at face mapped onto the readers) + the V-08 JSON
// fidelity legs. The legacy 007_json fixture shape (nested object with
// an array of public records, field "ClearDiagnosticInformation") is
// read through the tracked module verbatim; independent hand oracle.
// ---------------------------------------------------------------------------
#[test]
fn json975_array_paths_and_legacy_shape() {
    let stdlib_json = include_str!("../../../../../../stdlib/ace/std/json.at");
    let src = format!(
        r#"{stdlib_json}
fn main() int {{
    // the legacy 007_json fixture shape: services[0].name
    let doc str = "{{\"services\": [{{\"name\": \"ClearDiagnosticInformation\", \"port\": 8090}}, {{\"name\": \"UpdateData\", \"port\": 8091}}]}}"
    if !json_is_valid(doc) {{ return 1 }}
    if json_kind_at(doc, "services") != 5 {{ return 2 }}
    if json_kind_at(doc, "services.0") != 6 {{ return 3 }}
    if json_kind_at(doc, "services.0.name") != 4 {{ return 4 }}
    let n Str = json_parse_str(doc, "services.0.name")
    let want_n Str = "ClearDiagnosticInformation"
    if n != want_n {{ return 5 }}
    let n2 Str = json_parse_str(doc, "services.1.name")
    let want_n2 Str = "UpdateData"
    if n2 != want_n2 {{ return 6 }}
    if json_parse_int(doc, "services.0.port") != 8090 {{ return 7 }}
    if json_parse_int(doc, "services.1.port") != 8091 {{ return 8 }}
    // nested arrays: index chains
    let nested str = "{{\"grid\": [[1, 2], [3, 4]]}}"
    if json_kind_at(nested, "grid.0") != 5 {{ return 9 }}
    if json_parse_int(nested, "grid.1.0") != 3 {{ return 10 }}
    if json_parse_int(nested, "grid.0.1") != 2 {{ return 11 }}
    // negatives: out-of-range index and non-digit segment are MISSING
    if json_kind_at(doc, "services.2.name") != 0 {{ return 12 }}
    if json_kind_at(nested, "grid.0.9") != 0 {{ return 13 }}
    if json_kind_at(doc, "services.x.name") != 0 {{ return 14 }}
    if json_kind_at(doc, "services..name") != 0 {{ return 15 }}
    // descending through a SCALAR is missing (0), malformed doc is -1
    if json_kind_at(doc, "services.0.name.deeper") != 0 {{ return 16 }}
    if json_kind_at("{{\"a\": [", "a.0") != -1 {{ return 17 }}
    // ---- V-08 fidelity legs: null vs missing vs wide-int no loss
    let d2 str = "{{\"a\": null, \"b\": 2147483647, \"c\": true}}"
    if json_kind_at(d2, "a") != 1 {{ return 18 }}
    if json_kind_at(d2, "zz") != 0 {{ return 19 }}
    if json_parse_int(d2, "b") != 2147483647 {{ return 20 }}
    if json_parse_bool(d2, "c") != 1 {{ return 21 }}
    return 0
}}"#
    );
    let mut backend = Win32PlatformIo::default();
    let grants: Vec<String> = vec!["platform.mem".into()];
    let program = ace_av::console_host::compile(&src, "d_main").expect("compile array-path source");
    let (out, _t, _s) = program
        .run_platform(
            &grants,
            &ace_av_core::av::ExecLimits::default(),
            &ace_av_core::av::CancelFlag::new(false),
            None,
            &mut backend,
        )
        .expect("run platform array-path");
    assert!(
        matches!(
            &out,
            ace_av_core::contract::ExecOutcome::Success {
                value: ace_av_core::contract::Value::I32(0),
                ..
            }
        ),
        "array paths + V-08 fidelity oracles, got {out:?}"
    );
}
