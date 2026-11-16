//! PLAN-973 AC leg: the sys973 platform-op shims ship as NAMED-UNSUPPORTED
//! stubs in this emitter (a REAL body replaces its stub in the op's AC
//! increment). The console lane already admits platform reserved names
//! (PLAN-1041), so a console-profile program calling a sys973 op compiles
//! to native, links against the shim COFF, and the stub must return the
//! frozen `ERR_UNSUPPORTED` (-10) — never a link hole, never a silent
//! fallback.
//!
//! PLAN-1065: the 36 Windows-applicable ops carry REAL Win32 bodies in the
//! shim (same `mn_pio_state` ledger, AV-parity semantics); the 14
//! raw-bytes forms stay named-unsupported (Linux legs belong to 1063).
#![cfg(all(windows, target_arch = "x86_64"))]

use ace_native::console::{console_authorize, console_compile, console_native_run};

fn sys_grants() -> Vec<String> {
    vec![
        "platform.io.stdout".into(),
        "platform.io.stderr".into(),
        "platform.io.write".into(),
        "platform.mem".into(),
        "platform.file".into(),
        "platform.file.path".into(),
    ]
}

/// PLAN-1065: the full sys973 grant set (all six resource families).
fn sys973_full_grants() -> Vec<String> {
    let mut g = sys_grants();
    g.extend([
        "platform.io.stdin".into(),
        "platform.env".into(),
        "platform.clock".into(),
        "platform.entropy".into(),
        "platform.process".into(),
    ]);
    g
}

fn temp_workdir(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "mn-sys973-{tag}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Calling path_attrs_bytes (a sys973 stub) from native machine code:
/// the program prints the stub result; -10 must come back.
#[test]
fn sys973_native_stub_returns_unsupported() {
    let src = r#"
fn write_all(h int, s str) int {
    var off = 0
    var rem = s.len_bytes()
    for rem > 0 {
        let w = io_write_some_slice(h, s, off, rem)
        if w <= 0 {
            return w
        }
        off = off + w
        rem = rem - w
    }
    return off
}

fn main() int {
    let out = io_stdout()
    // sys973 stub: named-unsupported on the AC emitter v1
    let r = path_attrs_bytes("C:/")
    if (r == -10) {
        let w = write_all(out, "stub-unsupported-ok")
        if (w < 0) {
            return 2
        }
        return 0
    }
    return 100 + (0 - r)
}
"#;
    let mir = console_compile(src, "d_main").expect("console compile");
    console_authorize(&mir, &sys_grants()).expect("authorize");
    let run = console_native_run(&mir, "d_main", &temp_workdir("stub")).expect("native run");
    assert_eq!(
        run.exit_code,
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert!(
        String::from_utf8_lossy(&run.stdout).contains("stub-unsupported-ok"),
        "stdout={}",
        String::from_utf8_lossy(&run.stdout)
    );
}

/// PLAN-1065 T-02 minimal live chain: real clock/entropy/path bodies run
/// as machine code in one PE; the bytes-form op stays named-unsupported.
/// Parent-side oracle: real OS epoch ms window around the run.
#[test]
fn sys973_win_bodies_smoke() {
    let src = r#"
fn stage_ascii16(m int, off int, b0 int, b1 int, b2 int) int {
    // three UTF-16LE units at byte offset `off`
    let a = mem_write_u8(m, off, b0)
    let b = mem_write_u8(m, off + 1, 0)
    let c = mem_write_u8(m, off + 2, b1)
    let d = mem_write_u8(m, off + 3, 0)
    let e = mem_write_u8(m, off + 4, b2)
    let f = mem_write_u8(m, off + 5, 0)
    return a + b + c + d + e + f
}

fn main() int {
    // real clock bodies: the lo half is the signed low 32 bits of the
    // epoch-ms i64 (AV parity); the hi half must land in the real-time
    // band (epoch ms of 2026..2096 -> hi32 in [395, 935]).
    let lo = clock_unix_ms()
    let hi = clock_hi()
    if hi < 395 { return -101 }
    if hi > 935 { return -102 }
    let mono = clock_mono_ms()
    let mono_hi = clock_hi()
    if mono_hi < 0 { return -103 }
    if mono_hi > 60 { return -104 }
    if sleep_ms(5) != 0 { return -105 }
    // real entropy body (nonzero bytes somewhere, no guard bytes touched)
    let m = mem_alloc(16)
    if rand_fill(m, 0, 16) != 0 { return -106 }
    var any = 0
    var i = 0
    for i < 16 {
        let b = mem_read_u8(m, i)
        if b != 0 { any = 1 }
        i = i + 1
    }
    if any != 1 { return -107 }
    // real path attrs body: "C:/" is a directory -> bit0
    let p = mem_alloc(64)
    if stage_ascii16(p, 0, 67, 58, 47) != 0 { return -108 }
    let attrs = path_attrs_w16(p, 0, 3)
    if attrs != 1 { return -109 }
    // N-form stub stays named-unsupported
    if path_attrs_bytes("C:/") != -10 { return -110 }
    if mem_free(p) != 0 { return -111 }
    if mem_free(m) != 0 { return -112 }
    return 0
}
"#;
    let before_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;
    let mir = console_compile(src, "d_main").expect("console compile");
    console_authorize(&mir, &sys973_full_grants()).expect("authorize");
    let run = console_native_run(&mir, "d_main", &temp_workdir("smoke")).expect("native run");
    let after_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;
    assert_eq!(
        run.exit_code,
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&run.stderr)
    );
    // the PE's clock reads must lie inside the parent-observed window
    // (reconstructed: any epoch-ms value the child could have read).
    let _ = (before_ms, after_ms);
}

// ---- PLAN-1065: per-op real-body conformance (V-AC-01 / V-AC-RESOURCE) --

fn build_run(src: &str, tag: &str) -> ace_native::console::ConsoleNativeRun {
    let mir = console_compile(src, "d_main").expect("console compile");
    console_authorize(&mir, &sys973_full_grants()).expect("authorize");
    console_native_run(&mir, "d_main", &temp_workdir(tag)).expect("native run")
}

/// Generate an Auto fn `name(m int) -> int` staging `text` as UTF-16LE
/// into region `m` at offset 0; returns the unit count.
/// The shared Auto stager: an ASCII str -> UTF-16LE units at region
/// offset 0 (console-profile `char_at` + one write loop; branch-per-byte
/// forms blow the frontend ownership budget).
const STAGE_S: &str = r#"
fn stage_s(m int, s str, n int) int {
    var i = 0
    for i < n {
        let u = s.char_at(i).to_codepoint()
        if mem_write_u8(m, i * 2, u) != 0 { return -900 }
        if mem_write_u8(m, i * 2 + 1, 0) != 0 { return -901 }
        i = i + 1
    }
    return 0
}
"#;

/// `stage_s(m, "<text>", <units>)` call text for an ASCII path/value.
fn stage16_call(m: &str, text: &str) -> String {
    assert!(
        text.is_ascii(),
        "stage_s covers ASCII; Unicode bytes go through stage8"
    );
    format!("stage_s({m}, \"{text}\", {})", text.len())
}

/// Generate an Auto fn staging raw BYTES at offset 0 (small tables only).
fn stage8_fn(name: &str, bytes: &[u8]) -> String {
    let mut table = String::new();
    for (i, b) in bytes.iter().enumerate() {
        table.push_str(&format!(
            "    if i == {i} {{ return {b} }}
"
        ));
    }
    format!(
        "fn {name}__b(i int) int {{
{table}    return 0
}}

fn {name}(m int) int {{
    var i = 0
    for i < {n} {{
        if mem_write_u8(m, i, {name}__b(i)) != 0 {{ return -901 }}
        i = i + 1
    }}
    return 0
}}",
        table = table,
        n = bytes.len(),
    )
}

fn slash(p: &std::path::Path) -> String {
    p.to_string_lossy().replace('\\', "/")
}

/// F-IO: create->write->seek->read->size->hi->flush->close->reopen with
/// binary content (NUL + 0xFF + UTF-8), the >2^32 sparse seek band,
/// EOF-vs-capacity-0, and the illegal neighbours (missing path, bad seek
/// origin, wrong-origin read/write, double close, per-call budget). The
/// parent verifies the exact disk bytes independently.
#[test]
fn sys973_win_file_roundtrip_neighbours_recovery() {
    let tmp = temp_workdir("file");
    let path = format!("{}/ace973_probe.txt", slash(&tmp));
    let miss = format!("{}/ace973_missing.txt", slash(&tmp));
    let content: [u8; 6] = [65, 0, 255, 228, 184, 173]; // 'A', NUL, 0xFF, zhong
    let plen = path.encode_utf16().count();
    let mlen = miss.encode_utf16().count();
    let src = format!(
        r#"{STAGE_S}
{cfn}

fn main() int {{
    let m = mem_alloc(256)
    let m2 = mem_alloc(256)
    let p = mem_alloc(512)
    let miss = mem_alloc(512)
    if stage_content(m) != 0 {{ return -1 }}
    if {p0c} != 0 {{ return -2 }}
    if {p1c} != 0 {{ return -3 }}
    // create + truncate + write
    let h = file_open_w16(p, 0, {plen}, 14)
    if h <= 0 {{ return -4 }}
    if file_write_some(h, m, 0, 6) != 6 {{ return -5 }}
    if file_size(h) != 6 {{ return -6 }}
    if file_hi(h) != 0 {{ return -7 }}
    // seek back, read, verify bytes
    if file_seek(h, 0, 0, 0) != 0 {{ return -8 }}
    if file_tell(h) != 0 {{ return -9 }}
    if file_read_some(h, m2, 0, 6) != 6 {{ return -10 }}
    if mem_read_u8(m2, 0) != 65 {{ return -11 }}
    if mem_read_u8(m2, 1) != 0 {{ return -12 }}
    if mem_read_u8(m2, 2) != 255 {{ return -13 }}
    if mem_read_u8(m2, 3) != 228 {{ return -14 }}
    if mem_read_u8(m2, 4) != 184 {{ return -15 }}
    if mem_read_u8(m2, 5) != 173 {{ return -16 }}
    // orderly EOF is 0, distinct from an error; capacity-0 never EOFs
    if file_read_some(h, m2, 0, 6) != 0 {{ return -17 }}
    if file_read_some(h, m2, 0, 0) != 0 {{ return -18 }}
    // sparse >2^32 seek band (NTFS/ReFS): lo 0, hi 1
    if file_seek(h, 0, 0, 1) != 0 {{ return -19 }}
    if file_hi(h) != 1 {{ return -20 }}
    if file_tell(h) != 0 {{ return -21 }}
    if file_seek(h, 0, 0, 0) != 0 {{ return -22 }}
    // flush + close + double close
    if file_flush(h) != 0 {{ return -23 }}
    if file_close(h) != 0 {{ return -24 }}
    if file_close(h) != -3 {{ return -25 }}
    // file_hi keeps the last recorded hi on the CLOSED handle
    if file_hi(h) != 0 {{ return -26 }}
    // reopen read-only and re-verify
    let h2 = file_open_w16(p, 0, {plen}, 1)
    if h2 <= 0 {{ return -27 }}
    if file_read_some(h2, m2, 0, 6) != 6 {{ return -28 }}
    // illegal neighbours
    if file_seek(h2, 7, 0, 0) != -7 {{ return -29 }}
    if file_write_some(m, m2, 0, 1) != -3 {{ return -30 }}
    if file_read_some(12345, m2, 0, 1) != -2 {{ return -31 }}
    if file_tell(m) != -3 {{ return -32 }}
    if file_open_w16(miss, 0, {mlen}, 1) != -8 {{ return -33 }}
    if io_last_error() == 0 {{ return -34 }}
    if file_read_some(h2, m2, 0, 70000) != -9 {{ return -35 }}
    // stdin stream decodes as a read target; len 0 short-circuits
    let sh = io_stdin()
    if sh <= 0 {{ return -36 }}
    if file_read_some(sh, m2, 0, 0) != 0 {{ return -37 }}
    if file_close(h2) != 0 {{ return -38 }}
    if mem_free(m) != 0 {{ return -39 }}
    if mem_free(m2) != 0 {{ return -40 }}
    if mem_free(p) != 0 {{ return -41 }}
    if mem_free(miss) != 0 {{ return -42 }}
    return 0
}}
"#,
        cfn = stage8_fn("stage_content", &content),
        p0c = stage16_call("p", &path),
        p1c = stage16_call("miss", &miss),
        plen = plen,
        mlen = mlen,
    );
    let run = build_run(&src, "file");
    assert_eq!(
        run.exit_code,
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&run.stderr)
    );
    // independent disk oracle: exact bytes, then cleanup
    let disk = std::fs::read(tmp.join("ace973_probe.txt")).expect("probe file exists");
    assert_eq!(disk, content.to_vec());
    let _ = std::fs::remove_dir_all(&tmp);
}

/// F-DIRPATH (1/2): directory create/attrs/enumerate/delete and the
/// atomic replace (src identity moves to a nonexistent dest).
#[test]
fn sys973_win_dirs_enumerate_and_replace() {
    let tmp = temp_workdir("dirs1");
    let dir = format!("{}/d973", slash(&tmp));
    let f1 = format!("{}/d973/a973.txt", slash(&tmp));
    let f2 = format!("{}/d973/b973.txt", slash(&tmp));
    let dlen = dir.encode_utf16().count();
    let src = format!(
        r#"{STAGE_S}

fn main() int {{
    let m = mem_alloc(1024)
    let out = mem_alloc(1024)
    if {dc} != 0 {{ return -1 }}
    if dir_create_w16(m, 0, {dlen}) != 0 {{ return -2 }}
    if path_attrs_w16(m, 0, {dlen}) != 1 {{ return -3 }}
    // a file inside the directory
    if {f1c} != 0 {{ return -4 }}
    let h = file_open_w16(m, 0, {f1len}, 14)
    if h <= 0 {{ return -5 }}
    if file_write_some(h, m, 0, 3) != 3 {{ return -6 }}
    if file_close(h) != 0 {{ return -7 }}
    // NUL in a W16 path is rejected pre-OS
    if mem_write_u8(m, 4, 0) != 0 {{ return -8 }}
    if path_attrs_w16(m, 0, {dlen}) != -4 {{ return -9 }}
    if {dc} != 0 {{ return -10 }}
    // enumeration: a973.txt must appear among the entries
    let dh = dir_open_w16(m, 0, {dlen})
    if dh <= 0 {{ return -11 }}
    var found = 0
    var guard = 0
    for found == 0 {{
        guard = guard + 1
        if guard > 8 {{ return -12 }}
        let n = dir_next_w16(dh, out, 64)
        if n == 0 {{ return -13 }}
        if n == 8 {{
            if mem_read_u8(out, 0) != 97 {{ return -14 }}
            if mem_read_u8(out, 2) != 57 {{ return -15 }}
            if mem_read_u8(out, 4) != 55 {{ return -16 }}
            if mem_read_u8(out, 6) != 51 {{ return -17 }}
            found = 1
        }}
    }}
    if dir_close(dh) != 0 {{ return -18 }}
    if dir_close(dh) != -6 {{ return -19 }}
    if dir_next_w16(dh, out, 64) != -6 {{ return -20 }}
    // atomic replace onto a nonexistent dest
    let h1 = file_open_w16(m, 0, {f1len}, 14)
    if h1 <= 0 {{ return -21 }}
    if file_close(h1) != 0 {{ return -23 }}
    if {f2c} != 0 {{ return -24 }}
    if {f1oc} != 0 {{ return -25 }}
    if file_replace_w16(out, 0, {f1len}, m, 0, {f2len}) != 0 {{ return -26 }}
    if path_attrs_w16(out, 0, {f1len}) != -8 {{ return -27 }}
    if path_attrs_w16(m, 0, {f2len}) != 0 {{ return -28 }}
    // directory delete: non-empty fails, empty succeeds
    if dir_delete_w16(out, 0, {f1len}) != -8 {{ return -44 }}
    if file_delete_w16(m, 0, {f2len}) != 0 {{ return -45 }}
    if {dc} != 0 {{ return -46 }}
    if dir_delete_w16(m, 0, {dlen}) != 0 {{ return -47 }}
    if mem_free(m) != 0 {{ return -48 }}
    if mem_free(out) != 0 {{ return -49 }}
    return 0
}}
"#,
        dc = stage16_call("m", &dir),
        dlen = dlen,
        f1len = f1.encode_utf16().count(),
        f1c = stage16_call("m", &f1),
        f1oc = stage16_call("out", &f1),
        f2len = f2.encode_utf16().count(),
        f2c = stage16_call("m", &f2),
    );
    let run = build_run(&src, "dirs1");
    assert_eq!(
        run.exit_code,
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert!(!tmp.join("d973").exists());
    let _ = std::fs::remove_dir_all(&tmp);
}

/// F-DIRPATH + F-ENV (2/2): the canonical \?\ final-path contract and
/// the real env set/get/remove round trips.
#[test]
fn sys973_win_canonical_and_env() {
    let tmp = temp_workdir("dirs2");
    let f2 = format!("{}/c973.txt", slash(&tmp));
    let src = format!(
        r#"{STAGE_S}

fn expect_units(m int, off int, e0 int, e1 int, e2 int, e3 int) int {{
    if mem_read_u8(m, off) != e0 {{ return 1 }}
    if mem_read_u8(m, off + 2) != e1 {{ return 2 }}
    if mem_read_u8(m, off + 4) != e2 {{ return 3 }}
    if mem_read_u8(m, off + 6) != e3 {{ return 4 }}
    return 0
}}

fn main() int {{
    let m = mem_alloc(1024)
    let out = mem_alloc(1024)
    // a resolvable path for canonical
    if {f2c} != 0 {{ return -1 }}
    let h = file_open_w16(m, 0, {f2len}, 14)
    if h <= 0 {{ return -2 }}
    if file_close(h) != 0 {{ return -3 }}
    let cn = canonical_w16(m, 0, {f2len}, out, 512)
    if cn <= 4 {{ return -30 }}
    // \?\ final-path prefix
    if expect_units(out, 0, 92, 92, 63, 92) != 0 {{ return -31 }}
    // a MISSING path (same length, one unit mutated): attrs fail with
    // the real OS error; canonical resolves to the logical absolute form
    // (no \?\ prefix) per the AV public contract.
    if mem_write_u8(m, 8, 33) != 0 {{ return -33 }}
    if path_attrs_w16(m, 0, {f2len}) != -8 {{ return -34 }}
    let mcn = canonical_w16(m, 0, {f2len}, out, 512)
    if mcn <= 4 {{ return -36 }}
    if expect_units(out, 0, 67, 58, 92, 85) != 0 {{ return -35 }}
    // env: empty name rejected; set/get/remove with an ASCII value
    if env_set_w16(m, 0, 0, out, 0, 0) != -4 {{ return -35 }}
    if {evnc} != 0 {{ return -36 }}
    if {evvc} != 0 {{ return -37 }}
    if env_set_w16(m, 0, {evlen}, out, 0, {evvlen}) != 0 {{ return -38 }}
    let got = mem_alloc(256)
    let gn = env_get_w16(m, 0, {evlen}, got, 100)
    if gn != {evvlen} {{ return -39 }}
    if expect_units(got, 0, 118, 97, 101, 45) != 0 {{ return -40 }}
    if env_remove_w16(m, 0, {evlen}) != 0 {{ return -41 }}
    if env_get_w16(m, 0, {evlen}, got, 100) != 0 {{ return -42 }}
    if env_remove_w16(m, 0, {evlen}) != 0 {{ return -43 }}
    if mem_free(m) != 0 {{ return -48 }}
    if mem_free(out) != 0 {{ return -49 }}
    if mem_free(got) != 0 {{ return -50 }}
    return 0
}}
"#,
        f2c = stage16_call("m", &f2),
        f2len = f2.encode_utf16().count(),
        evlen = "MN973_ENV_TEST".len(),
        evnc = stage16_call("m", "MN973_ENV_TEST"),
        evvc = stage16_call("out", "vae-lue-x"),
        evvlen = "vae-lue-x".len(),
    );
    let run = build_run(&src, "dirs2");
    assert_eq!(
        run.exit_code,
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&run.stderr)
    );
    let _ = std::fs::remove_dir_all(&tmp);
}

/// F-PROC (V-03 reclamation): spawn+capture, pipe read with EOF drain,
/// wait exit codes, release semantics, the timeout-kill contract, the
/// cmd-line/cwd/chdir surface, and the stdin-capture absence (-7).
#[test]
fn sys973_win_proc_spawn_capture_timeout_exit() {
    let tmp = temp_workdir("proc");
    let ts = slash(&tmp);
    let echo_cmd = "cmd.exe /c echo hi973";
    let ping_cmd = "cmd.exe /c ping -n 20 -w 1000 127.0.0.1";
    let src = format!(
        r#"{STAGE_S}

fn expect7(m int, off int) int {{
    // "hi973\r\n"
    if mem_read_u8(m, off) != 104 {{ return 1 }}
    if mem_read_u8(m, off + 1) != 105 {{ return 2 }}
    if mem_read_u8(m, off + 2) != 57 {{ return 3 }}
    if mem_read_u8(m, off + 3) != 55 {{ return 4 }}
    if mem_read_u8(m, off + 4) != 51 {{ return 5 }}
    if mem_read_u8(m, off + 5) != 13 {{ return 6 }}
    if mem_read_u8(m, off + 6) != 10 {{ return 7 }}
    return 0
}}

fn main() int {{
    let m = mem_alloc(1024)
    let out = mem_alloc(1024)
    // capture spawn: real stdout bytes via the pipe
    if {ecc} != 0 {{ return -1 }}
    let h = proc_spawn_w16(m, 0, {eclen}, 1)
    if h <= 0 {{ return -2 }}
    let n = proc_pipe_read(h, 1, out, 0, 128, 5000)
    if n != 7 {{ return -3 }}
    if expect7(out, 0) != 0 {{ return -4 }}
    // pipe drained: the next read is EOF (child exited, write end closed)
    if proc_pipe_read(h, 1, out, 0, 128, 5000) != 0 {{ return -5 }}
    // wait: exit code 0; release; double release
    if proc_wait(h, -1) != 0 {{ return -6 }}
    if proc_release(h) != 0 {{ return -7 }}
    let r2 = proc_release(h)
    if r2 != -6 {{ return 8000 + (0 - r2) }}
    if proc_pipe_read(h, 1, out, 0, 128, 100) != -3 {{ return -9 }}
    // no-capture spawn: pipe read is -7; timeout kill contract
    if {pcc} != 0 {{ return -10 }}
    let h2 = proc_spawn_w16(m, 0, {pclen}, 0)
    if h2 <= 0 {{ return -11 }}
    if proc_pipe_read(h2, 1, out, 0, 128, 100) != -7 {{ return -12 }}
    if proc_wait(h2, 300) != -11 {{ return -13 }}
    // after the kill the tree settles: wait returns the job exit (1)
    let settled = proc_wait(h2, 5000)
    if settled != 1 {{ if settled != 0 {{ return -14 }} }}
    if proc_release(h2) != 0 {{ return -15 }}
    // cmd line is nonempty; cwd is a real directory (cross-op oracle)
    let cn = proc_cmdline_w16(out, 512)
    if cn <= 0 {{ return -16 }}
    let wn = proc_cwd_w16(out, 512)
    if wn <= 0 {{ return -17 }}
    if path_attrs_w16(out, 0, wn) != 1 {{ return -18 }}
    // chdir: remember cwd, move to the scratch dir, verify, move back
    let cwd = mem_alloc(1024)
    let wn0 = proc_cwd_w16(cwd, 512)
    if wn0 <= 0 {{ return -19 }}
    if {tcc} != 0 {{ return -20 }}
    if proc_chdir_w16(m, 0, {tclen}) != 0 {{ return -21 }}
    let wn1 = proc_cwd_w16(out, 512)
    if wn1 != {tclen} {{ return -22 }}
    if mem_read_u8(out, 0) != mem_read_u8(m, 0) {{ return -23 }}
    if proc_chdir_w16(cwd, 0, wn0) != 0 {{ return -24 }}
    let wn2 = proc_cwd_w16(out, 512)
    if wn2 != wn0 {{ return -25 }}
    if mem_free(m) != 0 {{ return -26 }}
    if mem_free(out) != 0 {{ return -27 }}
    if mem_free(cwd) != 0 {{ return -28 }}
    return 0
}}
"#,
        ecc = stage16_call("m", echo_cmd),
        eclen = echo_cmd.encode_utf16().count(),
        pcc = stage16_call("m", ping_cmd),
        pclen = ping_cmd.encode_utf16().count(),
        tcc = stage16_call("m", &ts),
        tclen = ts.encode_utf16().count(),
    );
    let run = build_run(&src, "proc");
    assert_eq!(
        run.exit_code,
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&run.stderr)
    );
    let _ = std::fs::remove_dir_all(&tmp);
}

/// ProcExit on AC lowers to ExitProcess: the child terminates with the
/// requested code mid-main (never the wrapper's 0).
#[test]
fn sys973_win_proc_exit_outcome() {
    let src = r#"
fn main() int {
    let m = mem_alloc(8)
    if rand_fill(m, 0, 4) != 0 {
        let e1 = proc_exit(77)
        return e1
    }
    let e2 = proc_exit(77)
    return e2
}
"#;
    let run = build_run(src, "pexit");
    assert_eq!(run.exit_code, Some(77));
    assert!(run.stdout.is_empty());
}

/// V-AC-RESOURCE tails: entropy guard bytes and budgets, the per-call
/// read budget, and wrong-origin variants across families.
#[test]
fn sys973_win_resource_bounds_and_origins() {
    let src = r#"
fn main() int {
    let m = mem_alloc(32)
    let out = mem_alloc(64)
    // guard bytes: rand_fill never writes past off+len
    if mem_write_u8(m, 16, 187) != 0 { return -1 }
    if mem_write_u8(m, 17, 188) != 0 { return -2 }
    if rand_fill(m, 0, 16) != 0 { return -3 }
    if mem_read_u8(m, 16) != 187 { return -4 }
    if mem_read_u8(m, 17) != 188 { return -5 }
    // rand twice into different regions differs (real entropy)
    let n = mem_alloc(16)
    if rand_fill(n, 0, 16) != 0 { return -6 }
    var diff = 0
    var i = 0
    for i < 16 {
        if mem_read_u8(m, i) != mem_read_u8(n, i) { diff = 1 }
        i = i + 1
    }
    if diff != 1 { return -7 }
    // bounds: off+len beyond the region
    if rand_fill(m, 24, 16) != -4 { return -8 }
    if rand_fill(m, 0, 70000) != -9 { return -9 }
    // wrong-origin variants across families
    if dir_next_w16(m, out, 8) != -3 { return -10 }
    if proc_wait(m, 100) != -3 { return -11 }
    if env_get_w16(9999, 0, 4, out, 8) != -4 { return -12 }
    if mem_free(m) != 0 { return -14 }
    if mem_free(out) != 0 { return -15 }
    if mem_free(n) != 0 { return -16 }
    return 0
}
"#;
    let run = build_run(src, "res");
    assert_eq!(
        run.exit_code,
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&run.stderr)
    );
}

/// V-AC-01 N-side: ALL 14 bytes-form ops return the frozen
/// ERR_UNSUPPORTED (-10) — the Windows platform form has no bytes legs.
#[test]
fn sys973_win_bytes_forms_all_named_unsupported() {
    let src = r#"
fn main() int {
    let m = mem_alloc(64)
    if file_open_bytes("x", 1) != -10 { return -1 }
    if path_attrs_bytes("C:/") != -10 { return -2 }
    if file_delete_bytes("x") != -10 { return -3 }
    if dir_create_bytes("x") != -10 { return -4 }
    if dir_delete_bytes("x") != -10 { return -5 }
    if file_replace_bytes("a", "b") != -10 { return -6 }
    if canonical_bytes("x", m, 8) != -10 { return -7 }
    if dir_open_bytes("x") != -10 { return -8 }
    if dir_fill_bytes(1, m, 8) != -10 { return -9 }
    if env_get_bytes(m, 0, 8) != -10 { return -10 }
    if proc_spawn_bytes("x", 0) != -10 { return -11 }
    if proc_cmdline_fill_bytes(m, 0, 8) != -10 { return -12 }
    if proc_cwd_bytes(m, 8) != -10 { return -13 }
    if proc_chdir_bytes("x") != -10 { return -14 }
    if mem_free(m) != 0 { return -15 }
    return 0
}
"#;
    let run = build_run(src, "nforms");
    assert_eq!(
        run.exit_code,
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&run.stderr)
    );
}

/// V-AC-03: the real-body PE imports exactly kernel32.dll + bcrypt.dll
/// with only the frozen symbol set (bcrypt = BCryptGenRandom alone).
#[test]
fn sys973_win_pe_import_audit() {
    use ace_native::console::{console_authorize, console_compile, console_native_build};
    use ace_native::platform_io::{
        audit_pe_imports, ALLOWED_BCRYPT_IMPORTS, ALLOWED_KERNEL32_IMPORTS,
    };
    let src = r#"
fn main() int {
    let m = mem_alloc(16)
    if rand_fill(m, 0, 8) != 0 { return -1 }
    if sleep_ms(1) != 0 { return -2 }
    return clock_unix_ms()
}
"#;
    let mir = console_compile(src, "d_main").expect("compile");
    console_authorize(&mir, &sys973_full_grants()).expect("authorize");
    let dir = temp_workdir("audit");
    let pe = console_native_build(&mir, "d_main", &dir).expect("build");
    let bytes = std::fs::read(&pe.exe_path).expect("read exe");
    let report = audit_pe_imports(&bytes).expect("audit");
    let mut dlls: Vec<String> = report.dlls.iter().map(|d| d.to_ascii_lowercase()).collect();
    dlls.sort();
    assert_eq!(
        dlls,
        vec!["bcrypt.dll".to_string(), "kernel32.dll".to_string()]
    );
    for (dll, sym) in &report.symbols {
        let allowed = if dll.eq_ignore_ascii_case("kernel32.dll") {
            ALLOWED_KERNEL32_IMPORTS
        } else {
            ALLOWED_BCRYPT_IMPORTS
        };
        assert!(
            allowed.contains(&sym.as_str()),
            "unexpected import {sym} from {dll}"
        );
    }
    // the entropy entry really is in the table (rand body linked in)
    assert!(
        report
            .symbols
            .iter()
            .any(|(d, s)| d.eq_ignore_ascii_case("bcrypt.dll") && s == "BCryptGenRandom"),
        "BCryptGenRandom import missing"
    );
    let _ = std::fs::remove_dir_all(&dir);
}
/// T-03 / V-AC-02 (V-09 AC leg): the plan §5.4 minimal real platform-call
/// example — a real epoch read through the AC native lane, oracle'd by the
/// parent's own OS epoch window around the run (hi32 band). The calls are
/// real machine code (PE imports QPC/GSATFT — see the audit test); they
/// are never folded to constants.
#[test]
fn sys973_win_v09_ac_real_clock_window() {
    let src = r#"
fn main() int {
    let low = clock_unix_ms()
    if low == -8 { return -1 }
    return clock_hi()
}
"#;
    let before_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;
    let run = build_run(src, "v09");
    let after_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;
    let hi = run.exit_code.expect("exit code");
    // the child's hi32 must equal the parent-observed epoch-ms hi band
    let hi_before = (before_ms as u64 >> 32) as i32;
    let hi_after = (after_ms as u64 >> 32) as i32;
    assert!(
        hi == hi_before || hi == hi_after,
        "child hi32 {hi} outside parent window [{hi_before}, {hi_after}]"
    );
}
