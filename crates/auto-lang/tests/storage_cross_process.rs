//! PLAN-044 AC-01/AC-02：storage 多进程并发写安全集成测试。
//!
//! 真实并发面 = 桌面宿主 × outproc 子进程（env 全继承，`AUTO_VM_STORAGE_FILE`
//! 共享同一份 desktop-storage.json）。本测试用 `current_exe()` 重入本测试
//! 二进制模拟两个独立进程（libtest 过滤器跑对应 worker 测试；无角色 env
//! 时 worker no-op 直接过，不影响常规 `cargo t` 全量跑）：
//!
//! - AC-01（双进程交错写不同键）：修复前全量覆盖 persist 必丢一键（红），
//!   键级合并写后两键均存活且为各自最后值（绿）。
//! - AC-02（撕裂免疫）：tear worker 循环写 256KB 大值，父进程随机时刻
//!   SIGKILL——修复前 `fs::write` 半截即坏库（红），原子替换后任意时刻
//!   文件均为合法 JSON（绿）。

use std::process::Command;

const ROLE_ENV: &str = "AUTO_VM_STORAGE_XPROC_ROLE";
const KEY_A: &str = "xproc.a";
const KEY_B: &str = "xproc.b";

fn storage_file(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "auto-xproc-{}-{}-{}",
        tag,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0)
    ));
    let _ = std::fs::create_dir_all(&dir);
    dir.join("storage.json")
}

fn spawn_worker(role: &str, storage: &std::path::Path) -> Command {
    let mut cmd = Command::new(std::env::current_exe().unwrap());
    cmd.arg(format!("storage_child_worker_{}", role))
        .env(ROLE_ENV, role)
        .env("AUTO_VM_STORAGE_FILE", storage);
    cmd
}

/// AC-01：两进程各写各的键各 200 轮 → 终态两键均存活且为各自最后值。
#[test]
fn cross_process_writes_both_keys_survive() {
    let storage = storage_file("ac01");
    let mut a = spawn_worker("a", &storage).spawn().expect("spawn worker a");
    let mut b = spawn_worker("b", &storage).spawn().expect("spawn worker b");
    let ra = a.wait().expect("wait a");
    let rb = b.wait().expect("wait b");
    assert!(ra.success() && rb.success(), "workers must exit clean");

    let raw = std::fs::read_to_string(&storage).expect("storage file must exist");
    let map: std::collections::HashMap<String, String> =
        serde_json::from_str(&raw).expect("storage must be valid JSON");
    assert_eq!(
        map.get(KEY_A).map(String::as_str),
        Some("v199"),
        "key A lost or stale — merge write broken; file: {raw}"
    );
    assert_eq!(
        map.get(KEY_B).map(String::as_str),
        Some("v199"),
        "key B lost or stale — merge write broken; file: {raw}"
    );
    let _ = std::fs::remove_dir_all(storage.parent().unwrap());
}

/// AC-02：写循环中 SIGKILL，读回恒为合法 JSON（撕裂不可能落盘）。
#[test]
fn kill_mid_write_leaves_valid_json() {
    for round in 0..5 {
        let storage = storage_file("ac02");
        // 预置一份合法基线（模拟既有库）
        std::fs::write(&storage, r#"{"seed":"base"}"#).unwrap();
        let mut child = spawn_worker("tear", &storage).spawn().expect("spawn tear");
        std::thread::sleep(std::time::Duration::from_millis(
            20 + (round as u64 * 37) % 120,
        ));
        let _ = child.kill();
        let _ = child.wait();
        let raw = std::fs::read_to_string(&storage)
            .unwrap_or_else(|e| panic!("round {round}: file readable: {e}"));
        let parsed: std::collections::HashMap<String, String> = serde_json::from_str(&raw)
            .unwrap_or_else(|e| panic!("round {round}: valid JSON after kill: {e} — raw head: {}", &raw[..raw.len().min(120)]));
        // 撕裂不可能：要么基线要么某次完整新值
        assert!(
            parsed.contains_key("seed") || parsed.contains_key("xproc.tear"),
            "round {round}: unexpected content: {}",
            &raw[..raw.len().min(120)]
        );
        let _ = std::fs::remove_dir_all(storage.parent().unwrap());
    }
}

// ── worker 测试（子进程重入点；无角色 env 时 no-op）──────────────────────

#[test]
fn storage_child_worker_a() {
    if std::env::var(ROLE_ENV).as_deref() != Ok("a") {
        return;
    }
    for i in 0..200 {
        auto_lang::vm::ffi::stdlib::shim_storage_set(KEY_A.to_string(), format!("v{i}"));
    }
}

#[test]
fn storage_child_worker_b() {
    if std::env::var(ROLE_ENV).as_deref() != Ok("b") {
        return;
    }
    for i in 0..200 {
        auto_lang::vm::ffi::stdlib::shim_storage_set(KEY_B.to_string(), format!("v{i}"));
    }
}

#[test]
fn storage_child_worker_tear() {
    if std::env::var(ROLE_ENV).as_deref() != Ok("tear") {
        return;
    }
    let big = "x".repeat(256 * 1024);
    let mut i = 0u64;
    loop {
        auto_lang::vm::ffi::stdlib::shim_storage_set(
            "xproc.tear".to_string(),
            format!("{:08}{}", i, big),
        );
        i += 1;
    }
}
