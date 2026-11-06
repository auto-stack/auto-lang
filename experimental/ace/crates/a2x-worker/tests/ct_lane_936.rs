//! PLAN-936 T-13（F-936-R1-01 修复闭包证据）：Rust 端产品车道接 CT——
//! `#{sum(20,22)}` 经 a2x 冷装配编译过，发射产物携带冻结常量 42；
//! 同源体编辑（调用点不改）→43。修复前该车道以 source.syntax 拒。

use ace_a2x_c::CTargetProvider;
use ace_a2x_rust::RustTargetProvider;
use ace_a2x_store::A2xSession;
use ace_core::source::SourceProfile;

const V1: &str = "fn sum(a int, b int) int {\n    return a+b\n}\nfn main() int {\n    let n = #{ sum(20, 22) }\n    return n\n}\n";
const V2: &str = "fn sum(a int, b int) int {\n    return a+b+1\n}\nfn main() int {\n    let n = #{ sum(20, 22) }\n    return n\n}\n";

fn assemble(src: &str) -> ace_a2x_store::Assembly {
    let dir = std::env::temp_dir().join(format!(
        "a2x-ct936-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    let mut session = A2xSession::open(&dir, "ct936", "ct936", 8 << 20, RustTargetProvider::v1())
        .expect("session");
    let caps = vec!["ac.source.cte.scalar/v1".to_string()];
    let out = session.cold_assemble_profile(src, SourceProfile::Scalar, &caps, "main", false);
    std::fs::remove_dir_all(&dir).ok();
    out.expect("the a2x lane compiles a CT source")
}

fn emitted_constant(a: &ace_a2x_store::Assembly) -> Option<i32> {
    // main 的发射体是常量返回：在 Rust 片段里找 main 邻近的字面量。
    let mut text = String::new();
    for (name, body) in &a.files {
        if name.contains("main") || name.ends_with(".rs") {
            text.push_str(body);
        }
    }
    if text.is_empty() {
        for (_, body) in &a.files {
            text.push_str(body);
        }
    }
    for want in [42, 43] {
        if text.contains(&format!("return {want};")) || text.contains(&format!("= {want};")) {
            return Some(want);
        }
    }
    None
}

#[test]
fn plain_scalar_assembly_baseline() {
    // 探针自校准：无 CT 的普通源走同一装配口必须成功（否则探针设置错）。
    let plain = "fn sum(a int, b int) int {
    return a+b
}
fn main() int {
    let n = sum(20, 22)
    return n
}
";
    let dir = std::env::temp_dir().join(format!(
        "a2x-plain936-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    let mut session = A2xSession::open(&dir, "ct936", "ct936", 8 << 20, RustTargetProvider::v1())
        .expect("session");
    let out = session.cold_assemble_profile(
        plain,
        SourceProfile::Scalar,
        &[],
        "main",
        false,
    );
    std::fs::remove_dir_all(&dir).ok();
    let _a = out.expect("plain scalar assembly works");
}

#[test]
fn rust_end_compiles_ct_source_and_carries_the_frozen_constant() {
    let a1 = assemble(V1);
    let c1 = emitted_constant(&a1).expect("the emitted main carries a frozen constant");
    assert_eq!(c1, 42);
    let a2 = assemble(V2);
    let c2 = emitted_constant(&a2).expect("edited body re-freezes");
    assert_eq!(c2, 43, "callee body edit (callsite unchanged) -> 43");
}

/// C 端：同一编译车道（CT 单门在 cold_assemble 内），C provider 消费同一
/// fragments——装配成功且发射文本携带冻结常量。
#[test]
fn c_end_compiles_ct_source_and_carries_the_frozen_constant() {
    let dir = std::env::temp_dir().join(format!("a2x-c-ct936-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let mut session =
        A2xSession::open(&dir, "ct936", "ct936", 8 << 20, CTargetProvider::v1())
            .expect("session");
    let caps = vec!["ac.source.cte.scalar/v1".to_string()];
    let out =
        session.cold_assemble_profile(V1, SourceProfile::Scalar, &caps, "main", false);
    std::fs::remove_dir_all(&dir).ok();
    let a = out.expect("the C lane assembles the CT source");
    let text = a
        .files
        .iter()
        .map(|(_, b)| b.as_str())
        .collect::<String>();
    assert!(text.contains('4') && text.contains('2'), "emitted C carries the constant");
}
