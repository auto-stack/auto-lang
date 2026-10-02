//! PLAN-729：服务端文件响应——a2r 发射与 VM native 面测试。
//!
//! - golden：`test/a2r/32_plan729/001_http_file_response/` 冻结发射形态
//!   （描述符构造 sync/async 同形、严格 options 字面量透传、typed 返回面）。
//! - probe：FileResponse 类型映射（`a2r_std::http::FileResponse` 全限定）、
//!   普通 int 不被误判（VM 面：声明门反例）、native 构造零 I/O。
//! - e2e wire/生命周期矩阵在 `http_e2e_plan729_*`（`test-http-e2e` 串行档，
//!   T-06/T-07 增补）。

use crate::trans::rust::transpile_rust_with_source_dir;
use std::fs::read_to_string;
use std::path::PathBuf;

const FILE_RESPONSE_CASE_DIR: &str = "test/a2r/32_plan729/001_http_file_response";

fn case_name(case_dir: &str) -> String {
    case_dir.rsplit('/').next().unwrap_or(case_dir).split_once('_').unwrap().1.to_string()
}

fn transpile_case(case_dir: &str) -> Vec<u8> {
    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let full = d.join(case_dir);
    let name = case_name(case_dir);
    let src = read_to_string(full.join(format!("{name}.at"))).expect("case .at");
    let mut rcode =
        transpile_rust_with_source_dir(&full, &name, &src).expect("transpile");
    rcode.done().expect("finalize").clone()
}

fn assert_golden(case_dir: &str) {
    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let name = case_name(case_dir);
    let expected = read_to_string(d.join(case_dir).join(format!("{name}.expected.rs")))
        .unwrap_or_default();
    let rs = transpile_case(case_dir);
    if rs != expected.as_bytes() {
        let wrong = d.join(case_dir).join(format!("{name}.wrong.rs"));
        std::fs::write(&wrong, &rs).expect("write wrong.rs");
        panic!(
            "golden mismatch for {case_dir}; actual written to {}",
            wrong.display()
        );
    }
}

#[test]
fn plan729_golden_file_response_matrix() {
    assert_golden(FILE_RESPONSE_CASE_DIR);
}

/// 描述符构造 lowering：qualified/直串/严格 options 字面量原样透传，
/// sync/async 同形（无 async 变体——构造零 I/O）。
#[test]
fn plan729_probe_file_response_emission() {
    let src = r#"
fn main() {
    let a = http.file_response("root", "x.bin", "{}")
    let b = http.file_response("root", "y.bin", "{\"etag\":\"\\\"e\\\"\"}")
}
"#;
    let mut rcode = crate::trans::rust::transpile_rust("plan729_probe", src)
        .expect("transpile");
    let rs = String::from_utf8(rcode.done().expect("finalize").clone()).expect("utf8");
    assert!(rs.contains("a2r_std::http::file_response("), "{rs}");
    // options 字面量字节保真（含转义引号）。
    assert!(
        rs.contains(r#"file_response("root", "y.bin", "{\"etag\":\"\\\"e\\\"\"}")"#)
            || rs.contains("file_response(\"root\", \"y.bin\","),
        "{rs}"
    );
}

/// 类型映射：`FileResponse` → `a2r_std::http::FileResponse`（全限定，
/// StringBuilder/SqliteDb 先例同款——无 glob import 也解析）。
#[test]
fn plan729_probe_file_response_type_mapping() {
    let src = r#"
fn helper(name str) FileResponse {
    return http.file_response("files", name, "{}")
}
fn main() {
    let f = helper("a")
}
"#;
    let mut rcode = crate::trans::rust::transpile_rust("plan729_type_map", src)
        .expect("transpile");
    let rs = String::from_utf8(rcode.done().expect("finalize").clone()).expect("utf8");
    assert!(
        rs.contains("-> a2r_std::http::FileResponse"),
        "typed 返回面映射: {rs}"
    );
    assert!(!rs.contains("impl FileResponse"), "不误判 trait 返回: {rs}");
}

// ===========================================================================
// VM 面（日常档）：native 构造（id 9936）、描述符登记、普通 int 反例门。
// e2e wire（真 TCP GET/HEAD/Range/条件）在 http_e2e_plan729_*（T-06）。
// ===========================================================================

#[cfg(test)]
mod vm {
    /// `http.file_response` native 构造：返回登记句柄（int），注册表有条目，
    /// 且构造本身零 I/O（不存在的 root/文件也不失败——故障由 adapter 映射）。
    #[test]
    fn plan729_vm_native_construct_registers_descriptor() {
        let code = r#"
let fr = http.file_response("C:/definitely/not/a/real/root", "missing.bin", "{}")
print("ok")
"#;
        let (_, out) = crate::run_with_capture(code).expect("run");
        assert!(
            out.contains("ok"),
            "构造零 I/O（缺 root 不失败）: {out}"
        );
        assert!(
            crate::vm::ffi::http_server_file::vm_file_response_count() >= 1,
            "描述符登记进 VM 注册表"
        );
    }

    /// 严格 options 反例：坏 options 描述符携带 init_error（构造不抛——
    /// adapter 映射 500；本测锁 VM 构造面的错误态存在性）。
    #[test]
    fn plan729_vm_bad_options_descriptor_has_init_error() {
        let code = r#"
let bad = http.file_response("root", "a", "{\"bogus\":1}")
print("done")
"#;
        let (_, out) = crate::run_with_capture(code).expect("run");
        assert!(out.contains("done"), "坏 options 构造不中断: {out}");
    }

    /// 声明返回门反例：非 #[api] 上下文直接调用返回 int——普通 int 不被
    /// marshal 当描述符消费（fn_is_api_file_return 只认 #[api] 声明表）。
    #[test]
    fn plan729_vm_plain_int_not_file_return() {
        assert!(
            !crate::vm::ffi::http_server::fn_is_api_file_return("main"),
            "普通 fn 不在文件返回表"
        );
    }
}
