// Plan 386 Stage 1 —— AutoUI 桌面协议（Desktop Protocol）v1：进程外 App
// 与桌面的五通道接缝，先在同进程 loopback 内按协议编码走通（Design 25
// §7 = 施工图；协议正身 = Stage 2"两进程"的设计，本模块即其消息/编解码/
// 状态机单源）。
//
// 五通道（`docs/design/autoui/autoshell.md` §7 表）：
// | 通道   | 方向      | 内容                                                        |
// |--------|-----------|-------------------------------------------------------------|
// | 孵化/握手 | app→host→app | Hello(标题/图标/尺寸/字体注册) → Welcome(Wid+surface 句柄) → Ready |
// | 帧     | app→host  | FrameReady(共享缓冲槽 + DrawList 载荷 + damage) / FrameAck 归还槽 |
// | 输入   | host→app  | (Wid, event) 编码注入（指针/键盘/滚轮/IME 三变体，413 §7.1）   |
// | 控制   | 双向      | Close/Focus/Resize ↓；TitleChanged/Notify/ExitRequest/DesktopBus ↑ |
// | 观测   | 双向      | Attach/Detach ↓；Log/Metric ↑（MCP per-app 代理的最小底座）    |
//
// 分层（Stage 2 换 transport 时只动 loopback 层）：
// - `codec`   ：二进制信封（magic "APDL" + 版本 + 通道 + 长度）与 LE 原语。
// - `message` ：五通道消息结构 + 逐消息 encode/decode（后端中立，无 iced）。
// - `endpoint`: 双端状态机（App: Detached→Handshaking→Active→Closing；
//   Host: Listening→Active），非法迁移返回 [`ProtocolError`]。
// - `loopback`: 同进程字节管道（send 编码过线 / recv 解码）——Stage 1 的
//   "共享纹理模拟"；Stage 2 换命名管道/共享内存时签名不变。
// - `host`    ：宿主侧绑定——把端点动作落到真实 462 会话对象
//   （`DesktopSession`/`WmState`）与 [`SurfaceStore`] 双缓冲表面。
//
// 不变量（本 Stage 验收）：同一消息 encode→decode 恒等；状态机拒绝一切
// 非法迁移；loopback demo 的 App 行为与直挂（in-process 直调）无差。

#[cfg(feature = "ui-iced")]
pub mod codec;
#[cfg(feature = "ui-iced")]
pub mod broker;
#[cfg(feature = "ui-iced")]
pub mod client_runtime;
#[cfg(feature = "ui-iced")]
pub mod coverage;
#[cfg(feature = "ui-iced")]
pub mod demo;
#[cfg(feature = "ui-iced")]
pub mod dual_mode;
#[cfg(feature = "ui-iced")]
pub mod editor_frame;
#[cfg(feature = "ui-iced")]
pub mod endpoint;
#[cfg(feature = "ui-iced")]
pub mod host;
#[cfg(feature = "ui-iced")]
pub mod loopback;
#[cfg(feature = "ui-iced")]
pub mod message;
#[cfg(feature = "ui-iced")]
pub mod pixels;
#[cfg(feature = "ui-iced")]
pub mod remote;
#[cfg(feature = "ui-iced")]
pub mod shm;
#[cfg(feature = "ui-iced")]
pub mod stage3;
#[cfg(feature = "ui-iced")]
pub mod transport;

/// 协议版本（信封头携带；不一致拒收——`CodecError::UnsupportedVersion`）。
/// v1 = 本计划 Stage 1 定稿（见 `docs/design/autoui/desktop-protocol-v1.md`）。
pub const PROTOCOL_VERSION: u16 = 1;

pub use codec::{CodecError, Channel};
pub use endpoint::{AppEndpoint, FrameSource, HostEndpoint, HostAction, HostState, ProtocolError};
pub use host::{ProtocolHost, SurfaceStore};
pub use loopback::{loopback_pair, LoopbackEnd};
pub use transport::{Transport, TransportError};
pub use message::{
    ControlMsg, DrawList, DrawOp, FontBlob, FrameMsg, HandshakeMsg, InputMsg, ObserveMsg,
    ProtocolMsg, Rgba8, WRect,
};

/// Plan 515 G4 C 族（P500-1）—— e2e 用真 `auto` 二进制的陈旧防护。
/// `auto_exe()` 优先取现存二进制，陈旧产物会伪装成回归：mtime 对账
/// `crates/` 树最新源文件，陈旧则 eprintln 警告（默认不阻断——警告即
/// 留痕，"回归"结论前先看陈旧位）；`AUTO_FRESH_EXE=1` 强制重建。
#[cfg(all(test, feature = "ui-iced"))]
pub(crate) mod e2e_exe {
    use std::path::{Path, PathBuf};

    /// `crates/` 树下最新 `.rs` 的 (mtime, 路径)（跳过 target/node_modules）。
    fn newest_source(root: &Path) -> Option<(std::time::SystemTime, PathBuf)> {
        fn walk(dir: &Path, best: &mut Option<(std::time::SystemTime, PathBuf)>) {
            let Ok(entries) = std::fs::read_dir(dir) else { return };
            for e in entries.flatten() {
                let p = e.path();
                let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if p.is_dir() {
                    if name == "target" || name == "node_modules" {
                        continue;
                    }
                    walk(&p, best);
                } else if name.ends_with(".rs") {
                    if let Ok(meta) = e.metadata() {
                        if let Ok(m) = meta.modified() {
                            if best.as_ref().is_none_or(|(bm, _)| m > *bm) {
                                *best = Some((m, p.clone()));
                            }
                        }
                    }
                }
            }
        }
        let mut best = None;
        walk(root, &mut best);
        best
    }

    /// 陈旧判定（纯函数，单测面）：exe mtime < newest 源 mtime → Some(最新源路径)。
    pub fn stale_against(exe: &Path, src_root: &Path) -> Option<PathBuf> {
        let exe_m = exe.metadata().ok()?.modified().ok()?;
        let (src_m, src_p) = newest_source(src_root)?;
        (exe_m < src_m).then_some(src_p)
    }

    /// 定位 + 陈旧防护（stage3/remote 两处 `auto_exe()` 共用体）。
    pub fn locate_with_stale_guard() -> PathBuf {
        let target = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target");
        let build = || {
            let status = std::process::Command::new("cargo")
                .args(["build", "-p", "auto", "--bin", "auto"])
                .current_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))
                .status()
                .expect("spawn cargo build -p auto");
            assert!(status.success(), "cargo build -p auto 失败");
            target.join("debug").join("auto.exe")
        };
        // AUTO_FRESH_EXE=1：无条件重建（e2e 结论前的确定性档）。
        if std::env::var("AUTO_FRESH_EXE").as_deref() == Ok("1") {
            eprintln!("[auto_exe] AUTO_FRESH_EXE=1 → 强制重建");
            return build();
        }
        for profile in ["debug", "release"] {
            let p = target.join(profile).join("auto.exe");
            if p.exists() {
                let src_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
                if let Some(newer) = stale_against(&p, &src_root.join("crates")) {
                    eprintln!(
                        "[auto_exe] 警告: {} 陈旧于最新源码（{}）——先重建再下\"回归\"结论；AUTO_FRESH_EXE=1 强制重建",
                        p.display(),
                        newer.display()
                    );
                }
                return p;
            }
        }
        build()
    }
}

#[cfg(all(test, feature = "ui-iced"))]
mod e2e_exe_tests {
    use super::e2e_exe::stale_against;

    /// Plan 515 G4 C2 单测：陈旧/新鲜两档判定（临时树手写 mtime）。
    #[test]
    fn stale_guard_detects_newer_source() {
        let dir = std::env::temp_dir().join(format!("auto-exe-stale-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("crates")).expect("mkdir");
        let exe = dir.join("auto.exe");
        let src = dir.join("crates").join("lib.rs");
        std::fs::write(&exe, b"exe").expect("exe");
        std::fs::write(&src, b"src").expect("src");
        // 同拍写盘（mtime 粒度）：exe 触回一次使其确定更旧。
        let older = std::time::SystemTime::now() - std::time::Duration::from_secs(3600);
        let _ = std::fs::File::options().write(true).open(&exe).and_then(|f| {
            f.set_modified(older)
        });
        assert!(
            stale_against(&exe, &dir.join("crates")).is_some(),
            "源码新于 exe → 陈旧检出"
        );
        // 源码更旧 → None。
        let _ = std::fs::File::options().write(true).open(&src).and_then(|f| {
            f.set_modified(older - std::time::Duration::from_secs(3600))
        });
        assert!(
            stale_against(&exe, &dir.join("crates")).is_none(),
            "源码旧于 exe → 不陈旧"
        );
        // exe 不存在 → None（调用方走构建臂）。
        std::fs::remove_file(&exe).expect("rm");
        assert!(stale_against(&exe, &dir.join("crates")).is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
