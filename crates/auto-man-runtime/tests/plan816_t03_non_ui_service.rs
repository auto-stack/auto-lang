//! PLAN-816 T-03 F-02：非 UI 服务不得依赖 UI provider。
//!
//! runtime-only 消费（不安装任何 UiOps）下，语言执行与共享 workspace
//! 必须仍走真实非 UI owner。

use std::net::TcpListener;

/// 有效 API 的 serve_project("vm") 未安装 UI 时须越过 UI seam，
/// 到达真实语言执行/HTTP bind（本测用占用端口的负控：bind 失败合法）。
#[test]
fn vm_service_without_ui_ops_reaches_language_not_ui_missing() {
    let root = std::env::temp_dir().join(format!("p816-t03-f02-vm-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("src/back")).unwrap();
    std::fs::write(
        root.join("src/back/api.at"),
        "#[api(method = \"GET\", path = \"/api/ping\")]\npub fn ping() str { return \"ok\" }\n",
    )
    .unwrap();

    // 占用端口 → 预期 bind 失败，而不是 UI ops not installed。
    let reserved = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = reserved.local_addr().unwrap().port();
    let cfg = auto_man_runtime::http_service::resolve_config(None, Some(port)).unwrap();
    let result = auto_man_runtime::http_service::serve_project(&root, "vm", cfg);

    let err = result.expect_err("占用端口下应 bind 失败");
    assert!(
        !err.contains("UI ops not installed"),
        "非 UI 服务不得被 UI seam 截断：{err}"
    );
    assert!(
        err.to_lowercase().contains("bind")
            || err.to_lowercase().contains("address")
            || err.to_lowercase().contains("10048"),
        "应到达真实 bind/address 失败路径：{err}"
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// Rust 轨 workspace/member 命名不经 UI provider。
#[test]
fn rust_workspace_helpers_work_without_ui_ops() {
    let root = std::env::temp_dir().join(format!("p816-t03-f02-ws-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("src/back")).unwrap();
    std::fs::write(
        root.join("pac.at"),
        "name: \"f02ws\"\nrender: \"vue\"\napi: \"rust\"\n",
    )
    .unwrap();

    let ws = auto_api_gen::rust_workspace::ensure_shared_workspace(&root);
    let back = auto_api_gen::rust_workspace::back_member_name(&root);
    assert!(!ws.as_os_str().is_empty(), "workspace 路径不得为空");
    assert!(
        ws.is_absolute() || ws.exists(),
        "workspace 应为真实路径：{ws:?}"
    );
    assert!(!back.is_empty(), "back member 名不得为空");
    assert!(
        back.contains("back") || back.contains("f02ws"),
        "back member 应为真实命名：{back}"
    );
    let _ = std::fs::remove_dir_all(&root);
}
