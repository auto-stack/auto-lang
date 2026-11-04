//! PLAN-929 T-08：声明面环境清单 codec/装配（core 级）。
//!
//! 冷建 env → dump → encode/decode 回环全等 → stub 装配（无 parse/resolve，
//! 计量窗口零 whole）→ 签名/槽位面等价（env_fn_names/env_free_fn_index）；
//! 篡改矩阵：schema/VT token/池索引越界 → 使用前受控拒。

use ace_core::body_work::capture_stage_attempts;
use ace_core::source::cst::FRONT_CAPS_VERSION;
use ace_core::source::decl_env::{
    decl_env_dump, decl_env_restore, decode_decl_env, encode_decl_env, DeclEnvError,
};
use ace_core::source::{env_fn_names, parse_program_with, resolve_env, SourceProfile};

fn sha_hex(s: &str) -> String {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    format!("fnv:{h:016x}")
}

const SRC: &str = "fn base(n int) int { return n * 2 }\nfn main() int { return base(21) }\n";

#[test]
fn decl_env_roundtrip_and_stub_assembly_zero_parse() {
    let program = parse_program_with(SRC, SourceProfile::Scalar).expect("parse");
    let env = resolve_env(&program, &[], SourceProfile::Scalar).expect("resolve");
    let sha = sha_hex(SRC);
    let dump = decl_env_dump(&env, FRONT_CAPS_VERSION, &sha);
    let text = encode_decl_env(&dump);
    let back = decode_decl_env(&text).expect("decode");
    assert_eq!(encode_decl_env(&back), text, "回环字节全等（确定性）");
    // stub 装配：零 parse/resolve（观察窗口 whole 轴 0）。
    let (env2, attempts) = capture_stage_attempts(|| {
        decl_env_restore(&back, SourceProfile::Scalar, FRONT_CAPS_VERSION, &sha).expect("restore")
    });
    assert_eq!(attempts.whole_total(), 0, "装配零整源 parse");
    // 声明面等价：名序与自由函数槽位一致。
    assert_eq!(env_fn_names(&env2), env_fn_names(&env), "名序一致");
    assert_eq!(
        ace_core::source::resolve::env_sig_of(&env2, "base"),
        ace_core::source::resolve::env_sig_of(&env, "base"),
        "签名面一致（VT 级）"
    );
}

#[test]
fn decl_env_tamper_rejections() {
    let program = parse_program_with(SRC, SourceProfile::Scalar).expect("parse");
    let env = resolve_env(&program, &[], SourceProfile::Scalar).expect("resolve");
    let sha = sha_hex(SRC);
    let text = encode_decl_env(&decl_env_dump(&env, FRONT_CAPS_VERSION, &sha));
    // schema 漂移。
    let bad = text.replacen("core-decl-env/1", "core-decl-env/2", 1);
    assert!(matches!(
        decode_decl_env(&bad),
        Err(DeclEnvError::Schema(_))
    ));
    // VT token 伪造。
    let bad = text
        .replace("fn base 3", "fn base 3x")
        .replacen("i32", "i32x", 1);
    assert!(
        matches!(decode_decl_env(&bad), Err(DeclEnvError::Malformed(_))),
        "{:?}",
        decode_decl_env(&bad)
    );
    // 池索引越界（rec:99）：改写 main fn 行的 result token。
    let bad = {
        let mut lines: Vec<String> = text.lines().map(|l| l.to_string()).collect();
        let idx = lines
            .iter()
            .position(|l| l.starts_with("fn main "))
            .expect("main fn line");
        let parts: Vec<&str> = lines[idx].split(' ').collect();
        let mut p = parts.clone();
        p[9] = "rec:99";
        lines[idx] = p.join(" ");
        lines.join(
            "
",
        ) + "
"
    };
    let d = decode_decl_env(&bad).expect("decode (token 合法)");
    match decl_env_restore(&d, SourceProfile::Scalar, FRONT_CAPS_VERSION, &sha) {
        Err(DeclEnvError::IndexOutOfRange { .. }) => {}
        other => panic!("expected IndexOutOfRange, got ok={}", other.is_ok()),
    }
    // source 绑定漂移。
    let d2 = decode_decl_env(&text).expect("decode");
    assert!(matches!(
        decl_env_restore(
            &d2,
            SourceProfile::Scalar,
            FRONT_CAPS_VERSION,
            "fnv:deadbeef"
        ),
        Err(DeclEnvError::SourceHash { .. })
    ));
    // profile 漂移。
    assert!(matches!(
        decl_env_restore(&d2, SourceProfile::Core, FRONT_CAPS_VERSION, &sha),
        Err(DeclEnvError::Profile { .. })
    ));
}
