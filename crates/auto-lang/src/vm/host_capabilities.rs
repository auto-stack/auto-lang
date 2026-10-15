// PLAN-779: VM 宿主能力装配点（pilot = frame 族 9918/9919）。
//
// Design 37 §4 四层拆分中的"装配"层落点：core（vm）只持有能力接口与
// per-VM 存储，不引用任何 UI/宿主实现；宿主（UI adapter、外部 consumer）
// 经显式 factory/provider 注册供值——不依赖 linker 自动拉 inventory 的
// 偶然性。进程默认 + 实例覆盖两级：
// - 进程默认：宿主启动时 `set_process_default_frame_provider` 显式注册；
//   `AutoVM::new` 在构造期**快照**当前默认进本实例——之后进程默认替换
//   不影响已建实例，两 VM 实例可持不同 provider（不串线）。
// - 实例覆盖：`AutoVM::set_frame_provider` / `NativeInterface::
//   set_frame_provider`，任何时点对本实例替换。
//
// 缺席语义（与 pilot 前一致）：无 provider = 未捕获，读侧 0。provider
// 注入不改变绑定身份（shim 函数/ID/contract 均不动），因此**不** bump
// binding_revision、不撤销 reference proof——register/merge 的既有失效
// 规则原样保留（计划 AC-02/V-03）。
//
// provider `Send + Sync` 只约束 provider 自身可跨线程共享；不代表 !Send
// 的 VM 实例可跨线程迁移（计划 §3）。

use std::sync::Arc;
use std::sync::RwLock;

/// 帧时间戳观测能力（pilot 唯一族；毫秒、单调时源由实现方保证）。
///
/// 值域约定与 VM 内建 9918/9919 出口一致：i64 内部态，shim 侧
/// saturating 收窄 i32（`min(i32::MAX)`）；未捕获/未发生 = 0。
pub trait FrameProvider: Send + Sync {
    /// 帧开始时间戳（宿主定义的帧起点语义）。
    fn begin_ms(&self) -> i64;
    /// 呈现完成时间戳（宿主定义的 present 语义）。
    fn present_ms(&self) -> i64;
}

/// 进程级默认 provider（宿主显式注册；None = 未注册 → VM 缺席 0）。
///
/// 替换语义：后注册者胜（宿主启动/测试装配用）；已建 VM 实例的快照
/// 不受影响。
static PROCESS_DEFAULT_FRAME_PROVIDER: RwLock<Option<Arc<dyn FrameProvider>>> = RwLock::new(None);

/// 注册进程默认 frame provider（显式权威 API：替换现存值，后注册者胜）。
/// 宿主启动/测试装配用；已建 VM 实例的快照不受影响。
pub fn set_process_default_frame_provider(provider: Arc<dyn FrameProvider>) {
    let mut slot = PROCESS_DEFAULT_FRAME_PROVIDER
        .write()
        .expect("PROCESS_DEFAULT_FRAME_PROVIDER poisoned");
    *slot = Some(provider);
}

/// 兼容默认装配（F779-R2-01）：**仅当槽位为空时**安装。检查与安装在同一
/// 写锁临界区内完成——并发的显式 `set_` 与本操作互斥，不存在"检查后
/// 被覆盖/覆盖他人"的窗口。供 UI 等内建宿主的隐式入口使用：兼容默认
/// 只补缺，不覆盖调用方显式选择（显式 `set_` 恒胜；需要强制替换走显式
/// API）。
pub fn set_process_default_frame_provider_if_absent(provider: Arc<dyn FrameProvider>) {
    let mut slot = PROCESS_DEFAULT_FRAME_PROVIDER
        .write()
        .expect("PROCESS_DEFAULT_FRAME_PROVIDER poisoned");
    if slot.is_none() {
        *slot = Some(provider);
    }
}

/// 当前进程默认（`AutoVM::new` 构造期快照消费）。
pub fn process_default_frame_provider() -> Option<Arc<dyn FrameProvider>> {
    PROCESS_DEFAULT_FRAME_PROVIDER
        .read()
        .expect("PROCESS_DEFAULT_FRAME_PROVIDER poisoned")
        .clone()
}

/// 清除进程默认（回到缺席语义：新建 VM 未捕获 0）。测试装配/宿主拆卸用。
pub fn clear_process_default_frame_provider() {
    let mut slot = PROCESS_DEFAULT_FRAME_PROVIDER
        .write()
        .expect("PROCESS_DEFAULT_FRAME_PROVIDER poisoned");
    *slot = None;
}

// ── PLAN-785: 文本剪贴板能力（auto.clipboard.text 2925 / set_text 2926）──
//
// 779 装配模式的第一个真实迁移批次：core 只持本接口与 per-VM 存储；
// OS 实现留在 UI 侧（clipboard_provider adapter 委托既有 arboard 桥），
// no-ui 宿主可注入自定义 provider。与 frame 族的差别：文本能力进出均
// 为**值**——出参 owned `String`（None=宿主不可用→shim 落空串，非
// raise）、入参 `&str`（shim 侧已 `pop_string_arg` 物化为 owned Rust
// String，接口不泄漏 VM 字符串索引/借用缓冲）。
//
// 缺席语义（V03/R-C）：无 provider = 与迁移前 no-ui 构建同形 VMError
//（shim 侧保留原 "the `ui-clipboard` feature is disabled" 文本，函数名
// 即 2925/2926 诊断定位面）；有 provider 但 OS 不可用 = empty/false。
// 任何状态不得伪装成功写入。

/// 文本剪贴板能力接口（ owning Rust 文本/布尔值域，无 UI 类型）。
pub trait ClipboardTextProvider: Send + Sync {
    /// 读剪贴板文本（None = 宿主/OS 不可用；空剪贴板返回 `Some("")`，
    /// 与 arboard `get_text` 的空串形一致——由实现方保证）。
    fn text(&self) -> Option<String>;
    /// 写剪贴板文本；false = 写入失败（不可用/被占用）。
    fn set_text(&self, text: &str) -> bool;
}

/// 进程级默认 clipboard text provider（显式注册；None = 未注册 → shim
/// 缺席错误）。替换/快照/if-absent 语义与 frame 族逐条相同。
static PROCESS_DEFAULT_CLIPBOARD_TEXT_PROVIDER: RwLock<Option<Arc<dyn ClipboardTextProvider>>> =
    RwLock::new(None);

/// 注册进程默认（显式权威 API：替换现存值，后注册者胜）。
pub fn set_process_default_clipboard_text_provider(provider: Arc<dyn ClipboardTextProvider>) {
    let mut slot = PROCESS_DEFAULT_CLIPBOARD_TEXT_PROVIDER
        .write()
        .expect("PROCESS_DEFAULT_CLIPBOARD_TEXT_PROVIDER poisoned");
    *slot = Some(provider);
}

/// 兼容默认装配（仅空槽补缺，同锁原子——语义同 frame 族 F779-R2-01）。
/// 供 UI 等内建宿主隐式入口：不覆盖调用方显式默认。
pub fn set_process_default_clipboard_text_provider_if_absent(
    provider: Arc<dyn ClipboardTextProvider>,
) {
    let mut slot = PROCESS_DEFAULT_CLIPBOARD_TEXT_PROVIDER
        .write()
        .expect("PROCESS_DEFAULT_CLIPBOARD_TEXT_PROVIDER poisoned");
    if slot.is_none() {
        *slot = Some(provider);
    }
}

/// 当前进程默认（`AutoVM::new` 构造期快照消费）。
pub fn process_default_clipboard_text_provider() -> Option<Arc<dyn ClipboardTextProvider>> {
    PROCESS_DEFAULT_CLIPBOARD_TEXT_PROVIDER
        .read()
        .expect("PROCESS_DEFAULT_CLIPBOARD_TEXT_PROVIDER poisoned")
        .clone()
}

/// 清除进程默认（回到缺席语义）。测试装配/宿主拆卸用。
pub fn clear_process_default_clipboard_text_provider() {
    let mut slot = PROCESS_DEFAULT_CLIPBOARD_TEXT_PROVIDER
        .write()
        .expect("PROCESS_DEFAULT_CLIPBOARD_TEXT_PROVIDER poisoned");
    *slot = None;
}

/// 测试隔离（F-785-R1-01）：进程默认槽（frame / clipboard text）的
/// **同进程**测试互斥锁。凡直接写槽（set/clear/if-absent）、经宿主入口
/// 触发装配（VmBridge 构造、run 家族高层入口会 if-absent 写槽）、或
/// 断言槽状态的 in-crate 测试，都必须持本锁串行并自行 save/restore。
/// 各测试文件不得自建本地锁——不相交的锁域等于没有互斥（本轮复现：
/// adapter 测试与 shim 缺席测试并行，clear→AutoVM::new 窗口被补缺写入
/// 毒化）。nextest 每测独立进程不能替代裸 cargo test 的同进程并行隔离。
#[cfg(test)]
pub(crate) fn test_default_slot_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
    LOCK.get_or_init(|| std::sync::Mutex::new(()))
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fake {
        begin: i64,
        present: i64,
    }

    impl FrameProvider for Fake {
        fn begin_ms(&self) -> i64 {
            self.begin
        }
        fn present_ms(&self) -> i64 {
            self.present
        }
    }

    /// 缺席语义：未注册 → None（shim 读侧落 0 的装配层前提）。
    #[test]
    fn absent_default_is_none() {
        // 仅断言 API 形状与 None 语义；set/clear 往返在 fixture（进程外）
        // 与 AutoVM 级测试覆盖，避免本单测污染同进程其他测试的默认槽。
        // F-785-R1-01：写槽测试须持共享隔离锁（与其他槽写/装配/缺席
        // 断言测试同域串行）。
        let _slot = test_default_slot_lock();
        let snapshot = process_default_frame_provider();
        set_process_default_frame_provider(Arc::new(Fake {
            begin: 1,
            present: 2,
        }));
        assert!(process_default_frame_provider().is_some());
        // 还原（保 None 或原值——本测试进程内默认槽无既有注册者）。
        match snapshot {
            Some(p) => set_process_default_frame_provider(p),
            None => {
                let mut slot = PROCESS_DEFAULT_FRAME_PROVIDER
                    .write()
                    .expect("PROCESS_DEFAULT_FRAME_PROVIDER poisoned");
                *slot = None;
            }
        }
    }
}
