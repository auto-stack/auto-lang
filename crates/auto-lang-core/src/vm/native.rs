use crate::vm::collections::{SpecializedHashMap, SpecializedHashSet};
use crate::vm::engine::{AutoVM, VMError};
use crate::vm::ffi::rust_stdlib::RustStdlibObject;
use crate::vm::task::AutoTask;
use auto_val::Value;
use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::RwLock;

use crate::{for_each_native, gen_native_constants};
// Plan 019 批次八: autodown natives（shim 常驻 —— feature off 时为桩）。
use crate::vm::autodown_natives::{
    shim_autodown_editor_text, shim_autodown_find_block, shim_autodown_insert_template,
    shim_autodown_insert_text, shim_autodown_parse, shim_autodown_serialize, shim_autodown_text,
};
// auto-os Plan 013 T2: AutoTerm 引擎桥 catalog shim(auto.term.*,ID 2943-2949)。
// 014 直键入:pump_input(2957)+ 几何随动/光标格(2958/2959/2976)。
// PLAN-018 D5:spawn_ex(2983)+ 定向泵三件 rows_for/pump_for/
// apply_resize_for(2984-2986)。
use crate::vm::ffi::term_engine::{
    shim_term_apply_resize,
    shim_term_apply_resize_for,
    shim_term_backlog_dropped,
    shim_term_backlog_paused,
    shim_term_backlog_pending_mb,
    shim_term_backlog_take_alerts,
    // PLAN-025:VM 臂配置面 stub + 滚动待排探针(泵节拍门)。
    shim_term_config_default_profile,
    shim_term_config_profiles,
    shim_term_config_profiles_full,
    shim_term_config_spawn_argv,
    shim_term_config_spawn_cwd,
    shim_term_config_spawn_program,
    shim_term_cursor_col,
    shim_term_cursor_row,
    shim_term_engine_history,
    // PLAN-028:分体轨 resize 断链桥(pend 写入 + history 查询)。
    shim_term_engine_pend_resize_geom,
    shim_term_free,
    shim_term_interrupt,
    shim_term_is_exited,
    shim_term_menu_take,
    shim_term_pump_for,
    shim_term_pump_input,
    shim_term_resize,
    shim_term_rows,
    shim_term_rows_for,
    shim_term_scroll_pending_for,
    shim_term_spawn,
    shim_term_spawn_ex,
    shim_term_viewport_cols,
    shim_term_viewport_rows,
    // PLAN-020 T-00b:窗口尺寸面 shim(auto.term.window_width/height)。
    shim_term_window_height,
    shim_term_window_width,
    shim_term_write_line,
    shim_term_write_raw,
};

/// Decode a tagged string index from a NanoValue popped from the stack.
#[inline]
fn decode_str_idx_nv(nv: auto_val::NanoValue) -> usize {
    auto_val::decode_string(nv) as usize
}

// Plan 094: ID ranges for hybrid FFI
/// Maximum ID for static FFI bindings
pub const STATIC_ID_MAX: u16 = 10000;
/// Starting ID for dynamic FFI bindings
pub const DYNAMIC_ID_START: u16 = 10000;

pub type ShimFunc = Arc<dyn Fn(&mut AutoTask, &AutoVM) -> Result<(), VMError> + Send + Sync>;

#[derive(Debug, Clone, serde::Serialize)]
pub struct NativeContract {
    pub parameters: Vec<String>,
    pub returns: String,
    pub producer: String,
    /// Independent target adapter facts; never populated by the .at scanner.
    pub receiver: Option<String>,
    pub is_static: bool,
    pub generics: Vec<String>,
    pub parameter_modes: Vec<String>,
    pub error_shape: String,
}

/// PLAN-808 R4: one remembered terminal provider plus its revival counter.
///
/// Every reinstallation (`set_terminal_provider`) and every session
/// publication through the spawn shims bumps `revivals` while the entry is
/// in the owners list. A prune verdict may remove the entry only if
/// `revivals` is unchanged since the verdict was taken: a provider that
/// revived after the liveness sample (reinstalled, or it gained a live
/// session through a real spawn) must stay **continuously** routable —
/// removing it first and re-adding it later leaves a window in which reads
/// and frees of already-exposed handles are lost (R4-F01). All bump and
/// verdict-check sites hold the owners write lock, so lock exclusion orders
/// the counter; a revival that has not acquired the lock yet re-inserts the
/// provider as a fresh entry instead (the spawn shims re-remember before
/// the new handle is returned to VM code).
struct TerminalOwnerEntry {
    provider: std::sync::Arc<dyn auto_terminal_core::TerminalHostProvider>,
    revivals: std::sync::atomic::AtomicU64,
}

/// Plan 094: NativeInterface with hybrid lookup support
///
/// Supports two types of native functions:
/// - **Static** (IDs 0-9999): Built into VM, registered at compile time
/// - **Dynamic** (IDs 10000+): Loaded via `use.rust`, registered at runtime
pub struct NativeInterface {
    /// Static shims: direct array lookup for maximum performance
    static_shims: Vec<Option<ShimFunc>>,
    /// Dynamic shims: HashMap for flexibility
    dynamic_shims: HashMap<u16, ShimFunc>,
    /// Next available dynamic ID
    next_dynamic_id: u16,
    /// Plan 200 Task 3.3: name -> ID mapping for CALL_SPEC fallback
    name_to_id: HashMap<String, u16>,
    contracts: HashMap<u16, NativeContract>,
    binding_revision: u64,
    pub binding_conflicts: Vec<(u16, String, String)>,
    reference_proofs: std::sync::Mutex<
        HashMap<u16, Result<Option<crate::stdlib_assembly::reference::ReferenceProof>, String>>,
    >,
    /// PLAN-779: 宿主能力装配槽（pilot=frame 族）。None=未捕获（读侧 0）。
    /// per-NativeInterface（即 per-VM）持有，两实例互不串线；Arc 共享同一
    /// provider 是宿主装配选择，非本结构的隐式全局。内 mut 供 VM 建后注入
    /// （`AutoVM::set_frame_provider`）。注入不 bump binding_revision/
    /// 不撤 proof（绑定身份不变，§host_capabilities 契约注释）。
    frame_provider:
        std::sync::RwLock<Option<std::sync::Arc<dyn crate::vm::host_capabilities::FrameProvider>>>,
    /// PLAN-785: 文本剪贴板能力槽（2925/2926）。语义与 frame_provider
    /// 槽逐条相同（per-NativeInterface 持有、Clone 保留、注入不动绑定
    /// 身份）；None = 缺席 → shim 报 feature-disabled 形 VMError。
    clipboard_text_provider: std::sync::RwLock<
        Option<std::sync::Arc<dyn crate::vm::host_capabilities::ClipboardTextProvider>>,
    >,
    /// PLAN-790: 文件/图像剪贴板能力槽（2934..2937）。装配语义与
    /// clipboard_text_provider 槽逐条相同（per-NativeInterface 持有、
    /// Clone 保留、注入不动绑定身份）；差别在缺席语义 = G3 **值降级**
    ///（空表/false/null，与迁移前无 OS 面同形），非 VMError。
    clipboard_media_provider: std::sync::RwLock<
        Option<std::sync::Arc<dyn crate::vm::host_capabilities::ClipboardMediaProvider>>,
    >,
    /// PLAN-794: scroll controller/子锚点能力槽（9900..9905/9922/9923）。
    /// 装配语义与 clipboard_media_provider 槽逐条相同（per-NativeInterface
    /// 持有、Clone 保留、注入不动绑定身份）；缺席语义 = T-01 冻结矩阵的
    /// 降级臂（Int 0 / Bool false / Float 0.0 / "error:unsupported" /
    /// "unknown"，与迁移前 no-ui 构建同形值语义，非 raise）。
    scroll_host_provider: std::sync::RwLock<
        Option<std::sync::Arc<dyn crate::vm::host_capabilities::ScrollHostProvider>>,
    >,
    /// PLAN-802: code editor 能力槽（22 native 全族：2910..2915、
    /// 2919..2924、2932/2933、2939、9906、9908..9913）。装配语义与
    /// scroll_host_provider 槽逐条相同（per-NativeInterface 持有、Clone
    /// 保留、注入不动绑定身份）；缺席语义 = 802 T-01 冻结矩阵的缺席臂
    /// （21 项 raise feature-disabled 形 VMError、9913 scroll_to 弹参
    /// 返 false——与迁移前 feature-off 构建逐字面值同形）。
    code_editor_provider: std::sync::RwLock<
        Option<std::sync::Arc<dyn crate::vm::host_capabilities::CodeEditorHostProvider>>,
    >,
    /// PLAN-804: OS 文件对话框能力槽（2927/2928）。装配语义与既有五族
    /// 逐条相同（per-NativeInterface 持有、Clone 保留、注入不动绑定身份）；
    /// 缺席语义 = 原 feature-off 形 VMError（文本逐字）。模态重入纪律：
    /// accessor 先 clone Arc 释放槽锁再返回——shim 调用 provider 时不持
    /// 本锁，真实模态阻塞/回调重入安全（V03）。
    dialog_host_provider: std::sync::RwLock<
        Option<std::sync::Arc<dyn crate::vm::host_capabilities::DialogHostProvider>>,
    >,
    /// PLAN-804: 拖出能力槽（2938）。装配语义与 dialog 槽逐条相同（两槽
    /// 独立装配，一种在位不掩盖另一种缺席）；缺席语义 = 迁移前 off 臂
    /// 同形 false（值降级，非 raise）。
    dnd_host_provider: std::sync::RwLock<
        Option<std::sync::Arc<dyn crate::vm::host_capabilities::DndHostProvider>>,
    >,
    /// PLAN-807: 编辑器快照能力槽（diff_snapshots 9916 供值）。装配语义
    /// 与既有七族逐条相同（per-NativeInterface 持有、Clone 保留、注入不
    /// 动绑定身份）；缺席语义 = "无法找到相应 editor" err envelope（值，
    /// 非 raise）。独立于 PLAN-802 的 code_editor_provider 槽——快照供值
    /// 是单一方法能力，不给 802 trait 加必实现方法（外部实现零破坏）。
    editor_snapshot_provider: std::sync::RwLock<
        Option<std::sync::Arc<dyn crate::vm::host_capabilities::EditorSnapshotProvider>>,
    >,
    /// PLAN-808: terminal 能力槽（34 native 全族：2943..2949、2957..2959、
    /// 2976..2982、2983..2987、2988..2993、2998..3003）。装配语义与既有
    /// provider 槽逐条相同（per-NativeInterface 持有、Clone 保留、注入不动
    /// 绑定身份）。**缺席语义不同于其他族**：无槽时 shim 回落到进程共享
    /// engine service（非 raise/非全空），保留迁移前"无 UI 也能驱动引擎"
    /// 的行为。
    terminal_provider:
        std::sync::RwLock<Option<std::sync::Arc<dyn auto_terminal_core::TerminalHostProvider>>>,
    /// PLAN-808 R1: per-VM history of terminal providers ever installed.
    /// Handles stay bound to the provider that owns them across
    /// `set_terminal_provider` replacement (owner routing); the strong Arcs
    /// also keep the original service alive until its last session is freed.
    /// Pruned when a replaced owner has no live sessions (R1-F04); each
    /// entry's revival counter keeps a provider that revived after the
    /// liveness sample continuously routable during prune (R4-F01).
    terminal_owners: std::sync::RwLock<Vec<std::sync::Arc<TerminalOwnerEntry>>>,
    /// Generation bumped on every owner-list mutation. Owner queries snapshot
    /// the list and query providers **outside** the lock; a concurrent
    /// set/prune is detected by the generation and retried so a live owner is
    /// never lost (R2-F01).
    terminal_owners_gen: std::sync::atomic::AtomicU64,
    /// Per-VM retired handle cache (cursor/viewport) captured when a handle
    /// is freed through the shims. It preserves the frozen post-free values
    /// without keeping the original service alive or touching a freed DLL
    /// pointer (R2-F02).
    terminal_retired:
        std::sync::Mutex<std::collections::HashMap<i64, auto_terminal_core::TerminalRetiredCache>>,
}

impl Clone for NativeInterface {
    fn clone(&self) -> Self {
        Self {
            static_shims: self.static_shims.clone(),
            dynamic_shims: self.dynamic_shims.clone(),
            next_dynamic_id: self.next_dynamic_id,
            name_to_id: self.name_to_id.clone(),
            contracts: self.contracts.clone(),
            binding_conflicts: self.binding_conflicts.clone(),
            binding_revision: self.binding_revision,
            reference_proofs: std::sync::Mutex::new(HashMap::new()),
            // PLAN-779: provider 随装配克隆保留（merge_native_interface 经
            // Clone+merge 路径不丢宿主能力；proof 按既有规则清空不变）。
            frame_provider: std::sync::RwLock::new(
                self.frame_provider
                    .read()
                    .expect("frame_provider poisoned")
                    .clone(),
            ),
            // PLAN-785: clipboard text provider 随装配克隆保留（同上，
            // merge_native_interface 经 Clone+merge 路径不丢宿主能力）。
            clipboard_text_provider: std::sync::RwLock::new(
                self.clipboard_text_provider
                    .read()
                    .expect("clipboard_text_provider poisoned")
                    .clone(),
            ),
            // PLAN-790: clipboard media provider 随装配克隆保留（同上）。
            clipboard_media_provider: std::sync::RwLock::new(
                self.clipboard_media_provider
                    .read()
                    .expect("clipboard_media_provider poisoned")
                    .clone(),
            ),
            // PLAN-794: scroll host provider 随装配克隆保留（同上）。
            scroll_host_provider: std::sync::RwLock::new(
                self.scroll_host_provider
                    .read()
                    .expect("scroll_host_provider poisoned")
                    .clone(),
            ),
            // PLAN-802: code editor provider 随装配克隆保留（同上——
            // merge_native_interface 经 Clone+merge 路径不丢宿主能力）。
            code_editor_provider: std::sync::RwLock::new(
                self.code_editor_provider
                    .read()
                    .expect("code_editor_provider poisoned")
                    .clone(),
            ),
            // PLAN-804: dialog/dnd host provider 随装配克隆保留（同上，
            // merge_native_interface 经 Clone+merge 路径不丢宿主能力）。
            dialog_host_provider: std::sync::RwLock::new(
                self.dialog_host_provider
                    .read()
                    .expect("dialog_host_provider poisoned")
                    .clone(),
            ),
            dnd_host_provider: std::sync::RwLock::new(
                self.dnd_host_provider
                    .read()
                    .expect("dnd_host_provider poisoned")
                    .clone(),
            ),
            editor_snapshot_provider: std::sync::RwLock::new(
                self.editor_snapshot_provider
                    .read()
                    .expect("editor_snapshot_provider poisoned")
                    .clone(),
            ),
            // PLAN-808: terminal provider 随装配克隆保留（同上）。
            terminal_provider: std::sync::RwLock::new(
                self.terminal_provider
                    .read()
                    .expect("terminal_provider poisoned")
                    .clone(),
            ),
            // PLAN-808 R1: owner history survives Clone/merge so a handle
            // exposed before the clone/replace still routes to its origin.
            terminal_owners: std::sync::RwLock::new(
                self.terminal_owners
                    .read()
                    .expect("terminal_owners poisoned")
                    .clone(),
            ),
            terminal_owners_gen: std::sync::atomic::AtomicU64::new(
                self.terminal_owners_gen
                    .load(std::sync::atomic::Ordering::Relaxed),
            ),
            terminal_retired: std::sync::Mutex::new(
                self.terminal_retired
                    .lock()
                    .expect("terminal_retired poisoned")
                    .clone(),
            ),
        }
    }
}

impl NativeInterface {
    pub fn new() -> Self {
        Self {
            static_shims: vec![None; STATIC_ID_MAX as usize],
            dynamic_shims: HashMap::new(),
            next_dynamic_id: DYNAMIC_ID_START,
            name_to_id: HashMap::new(),
            contracts: HashMap::new(),
            binding_revision: 0,
            binding_conflicts: Vec::new(),
            reference_proofs: std::sync::Mutex::new(HashMap::new()),
            frame_provider: std::sync::RwLock::new(None),
            clipboard_text_provider: std::sync::RwLock::new(None),
            clipboard_media_provider: std::sync::RwLock::new(None),
            scroll_host_provider: std::sync::RwLock::new(None),
            code_editor_provider: std::sync::RwLock::new(None),
            dialog_host_provider: std::sync::RwLock::new(None),
            dnd_host_provider: std::sync::RwLock::new(None),
            editor_snapshot_provider: std::sync::RwLock::new(None),
            terminal_provider: std::sync::RwLock::new(None),
            terminal_owners: std::sync::RwLock::new(Vec::new()),
            terminal_owners_gen: std::sync::atomic::AtomicU64::new(0),
            terminal_retired: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }

    /// PLAN-779: 注入/替换本接口（=本 VM）的 frame provider。
    /// 绑定身份不变（shim/ID/contract 均不动），不触发 revision/proof 失效。
    pub fn set_frame_provider(
        &self,
        provider: std::sync::Arc<dyn crate::vm::host_capabilities::FrameProvider>,
    ) {
        *self
            .frame_provider
            .write()
            .expect("frame_provider poisoned") = Some(provider);
    }

    /// PLAN-779: 当前 provider 快照（None=未捕获 0 语义）。
    pub fn frame_provider(
        &self,
    ) -> Option<std::sync::Arc<dyn crate::vm::host_capabilities::FrameProvider>> {
        self.frame_provider
            .read()
            .expect("frame_provider poisoned")
            .clone()
    }

    /// PLAN-785: 注入/替换本接口（=本 VM）的 clipboard text provider。
    /// 绑定身份不变（shim/ID/contract 均不动），不触发 revision/proof 失效。
    pub fn set_clipboard_text_provider(
        &self,
        provider: std::sync::Arc<dyn crate::vm::host_capabilities::ClipboardTextProvider>,
    ) {
        *self
            .clipboard_text_provider
            .write()
            .expect("clipboard_text_provider poisoned") = Some(provider);
    }

    /// PLAN-785: 当前 clipboard text provider（None=缺席错误语义）。
    pub fn clipboard_text_provider(
        &self,
    ) -> Option<std::sync::Arc<dyn crate::vm::host_capabilities::ClipboardTextProvider>> {
        self.clipboard_text_provider
            .read()
            .expect("clipboard_text_provider poisoned")
            .clone()
    }

    /// PLAN-790: 注入/替换本接口（=本 VM）的 clipboard media provider。
    /// 绑定身份不变（shim/ID/contract 均不动），不触发 revision/proof 失效。
    pub fn set_clipboard_media_provider(
        &self,
        provider: std::sync::Arc<dyn crate::vm::host_capabilities::ClipboardMediaProvider>,
    ) {
        *self
            .clipboard_media_provider
            .write()
            .expect("clipboard_media_provider poisoned") = Some(provider);
    }

    /// PLAN-790: 当前 clipboard media provider（None=G3 值降级语义）。
    pub fn clipboard_media_provider(
        &self,
    ) -> Option<std::sync::Arc<dyn crate::vm::host_capabilities::ClipboardMediaProvider>> {
        self.clipboard_media_provider
            .read()
            .expect("clipboard_media_provider poisoned")
            .clone()
    }

    /// PLAN-794: 注入/替换本接口（=本 VM）的 scroll host provider。
    /// 绑定身份不变（shim/ID/contract 均不动），不触发 revision/proof 失效。
    pub fn set_scroll_host_provider(
        &self,
        provider: std::sync::Arc<dyn crate::vm::host_capabilities::ScrollHostProvider>,
    ) {
        *self
            .scroll_host_provider
            .write()
            .expect("scroll_host_provider poisoned") = Some(provider);
    }

    /// PLAN-794: 当前 scroll host provider（None=T-01 冻结降级臂语义）。
    pub fn scroll_host_provider(
        &self,
    ) -> Option<std::sync::Arc<dyn crate::vm::host_capabilities::ScrollHostProvider>> {
        self.scroll_host_provider
            .read()
            .expect("scroll_host_provider poisoned")
            .clone()
    }

    /// PLAN-802: 注入/替换本接口（=本 VM）的 code editor provider。
    /// 绑定身份不变（shim/ID/contract 均不动），不触发 revision/proof 失效。
    pub fn set_code_editor_provider(
        &self,
        provider: std::sync::Arc<dyn crate::vm::host_capabilities::CodeEditorHostProvider>,
    ) {
        *self
            .code_editor_provider
            .write()
            .expect("code_editor_provider poisoned") = Some(provider);
    }

    /// PLAN-802: 当前 code editor provider（None=T-01 冻结缺席臂语义）。
    pub fn code_editor_provider(
        &self,
    ) -> Option<std::sync::Arc<dyn crate::vm::host_capabilities::CodeEditorHostProvider>> {
        self.code_editor_provider
            .read()
            .expect("code_editor_provider poisoned")
            .clone()
    }
    /// PLAN-804: 注入/替换本接口（=本 VM）的 dialog host provider。
    /// 绑定身份不变（shim/ID/contract 均不动），不触发 revision/proof 失效。
    pub fn set_dialog_host_provider(
        &self,
        provider: std::sync::Arc<dyn crate::vm::host_capabilities::DialogHostProvider>,
    ) {
        *self
            .dialog_host_provider
            .write()
            .expect("dialog_host_provider poisoned") = Some(provider);
    }

    /// PLAN-804: 当前 dialog host provider（None=缺席错误语义）。clone
    /// Arc 即释放槽锁——模态阻塞/回调重入不持锁（V03）。
    pub fn dialog_host_provider(
        &self,
    ) -> Option<std::sync::Arc<dyn crate::vm::host_capabilities::DialogHostProvider>> {
        self.dialog_host_provider
            .read()
            .expect("dialog_host_provider poisoned")
            .clone()
    }

    /// PLAN-804: 注入/替换本接口（=本 VM）的 dnd host provider。
    pub fn set_dnd_host_provider(
        &self,
        provider: std::sync::Arc<dyn crate::vm::host_capabilities::DndHostProvider>,
    ) {
        *self
            .dnd_host_provider
            .write()
            .expect("dnd_host_provider poisoned") = Some(provider);
    }

    /// PLAN-807: 注入/替换本接口（=本 VM）的 editor snapshot provider
    /// （diff_snapshots 9916 供值）。绑定身份不变，不触发 revision/proof
    /// 失效。
    pub fn set_editor_snapshot_provider(
        &self,
        provider: std::sync::Arc<dyn crate::vm::host_capabilities::EditorSnapshotProvider>,
    ) {
        *self
            .editor_snapshot_provider
            .write()
            .expect("editor_snapshot_provider poisoned") = Some(provider);
    }

    /// PLAN-807: 当前 editor snapshot provider（None="无法找到相应
    /// editor" err envelope 语义）。
    pub fn editor_snapshot_provider(
        &self,
    ) -> Option<std::sync::Arc<dyn crate::vm::host_capabilities::EditorSnapshotProvider>> {
        self.editor_snapshot_provider
            .read()
            .expect("editor_snapshot_provider poisoned")
            .clone()
    }

    /// PLAN-804: 当前 dnd host provider（None=false 降级语义）。
    pub fn dnd_host_provider(
        &self,
    ) -> Option<std::sync::Arc<dyn crate::vm::host_capabilities::DndHostProvider>> {
        self.dnd_host_provider
            .read()
            .expect("dnd_host_provider poisoned")
            .clone()
    }

    /// PLAN-808: 注入/替换本接口（=本 VM）的 terminal provider。
    /// 绑定身份不变（shim/ID/contract 均不动），不触发 revision/proof 失效。
    /// The provider is also remembered in `terminal_owners`: handles already
    /// exposed by it stay bound to it after later replacements. The old slot
    /// value is dropped **outside** the write lock (R2-F01: its Drop may
    /// reenter the interface).
    pub fn set_terminal_provider(
        &self,
        provider: std::sync::Arc<dyn auto_terminal_core::TerminalHostProvider>,
    ) {
        let old = {
            let mut slot = self
                .terminal_provider
                .write()
                .expect("terminal_provider poisoned");
            std::mem::replace(&mut *slot, Some(provider.clone()))
        };
        drop(old);
        self.remember_terminal_owner(provider);
    }

    /// Remember a provider for handle-owner routing (deduplicated by Arc
    /// identity). Called by `set_terminal_provider` and by the spawn shims
    /// after every successful session publication. An already-remembered
    /// provider has its entry's **revival counter bumped** instead of the
    /// call being a no-op: the bump invalidates a concurrently-running prune
    /// verdict that sampled `live_handle_count == 0` before this
    /// reinstallation/publication, so a revived provider is never removed
    /// from the visible list (R4-F01). No provider callback runs under the
    /// owners lock (R2-F01).
    pub fn remember_terminal_owner(
        &self,
        provider: std::sync::Arc<dyn auto_terminal_core::TerminalHostProvider>,
    ) {
        let mut owners = self
            .terminal_owners
            .write()
            .expect("terminal_owners poisoned");
        if let Some(entry) = owners
            .iter_mut()
            .find(|entry| std::sync::Arc::ptr_eq(&entry.provider, &provider))
        {
            entry
                .revivals
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        } else {
            owners.push(std::sync::Arc::new(TerminalOwnerEntry {
                provider,
                revivals: std::sync::atomic::AtomicU64::new(0),
            }));
            self.terminal_owners_gen
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
    }

    /// Find the provider that owns `handle` (most recently installed first).
    /// Returns `None` when no remembered provider owns it — callers then use
    /// the current provider / shared fallback.
    ///
    /// The provider list is snapshotted under the read lock and queried
    /// outside it; the generation retry prevents a concurrent set/prune from
    /// hiding a live owner (R2-F01).
    pub fn terminal_owner_for(
        &self,
        handle: i64,
    ) -> Option<std::sync::Arc<dyn auto_terminal_core::TerminalHostProvider>> {
        use std::sync::atomic::Ordering;
        for _ in 0..3 {
            let generation = self.terminal_owners_gen.load(Ordering::Relaxed);
            let snapshot: Vec<std::sync::Arc<TerminalOwnerEntry>> = self
                .terminal_owners
                .read()
                .expect("terminal_owners poisoned")
                .clone();
            if let Some(owner) = snapshot
                .iter()
                .rev()
                .find(|entry| entry.provider.owns_handle(handle))
                .map(|entry| entry.provider.clone())
            {
                return Some(owner);
            }
            if self.terminal_owners_gen.load(Ordering::Relaxed) == generation {
                return None;
            }
        }
        None
    }

    /// Find a remembered provider that still has the *retired* cache for
    /// `handle` (after free). Used only by the cursor/viewport cache shims;
    /// live operations continue to use [`terminal_owner_for`] (R2-F02).
    pub fn terminal_cache_owner_for(
        &self,
        handle: i64,
    ) -> Option<std::sync::Arc<dyn auto_terminal_core::TerminalHostProvider>> {
        use std::sync::atomic::Ordering;
        for _ in 0..3 {
            let generation = self.terminal_owners_gen.load(Ordering::Relaxed);
            let snapshot: Vec<std::sync::Arc<TerminalOwnerEntry>> = self
                .terminal_owners
                .read()
                .expect("terminal_owners poisoned")
                .clone();
            if let Some(owner) = snapshot
                .iter()
                .rev()
                .find(|entry| entry.provider.cached_cursor_viewport(handle).is_some())
                .map(|entry| entry.provider.clone())
            {
                return Some(owner);
            }
            if self.terminal_owners_gen.load(Ordering::Relaxed) == generation {
                return None;
            }
        }
        None
    }

    /// Drop remembered owners with no live sessions. The current slot is not
    /// touched (it stays installed even at zero sessions). This lets a
    /// replaced isolated service Drop and free any remaining sessions
    /// (F808-R1-04) without losing owners that still have handles.
    ///
    /// Provider methods and removed-provider destructors run **outside** the
    /// owners lock. The visible owner list is **never emptied** while those
    /// lock-free calls run: it is snapshotted intact and evaluated outside
    /// (R3-F01). Removal is **conditional on the entry's revival counter**
    /// (R4-F01): the counter is sampled before each liveness call and every
    /// reinstallation / session publication through the spawn shims bumps
    /// it, so a provider that revived after the (legitimately stale-zero)
    /// liveness sample is never removed from the visible list at all — it
    /// stays continuously routable instead of being removed and re-added.
    /// A publication that acquires the write lock only after the removal
    /// re-inserts the provider as a fresh entry before its handle is
    /// returned to VM code.
    pub fn prune_terminal_owners(&self) {
        // 1. Snapshot the live entries; no provider call under the lock.
        let snapshot: Vec<std::sync::Arc<TerminalOwnerEntry>> = self
            .terminal_owners
            .read()
            .expect("terminal_owners poisoned")
            .clone();

        // 2. Evaluate outside the lock; the visible list stays resolvable.
        //    The revival stamp is read BEFORE the liveness call: any bump
        //    after the stamp (reinstall or real spawn) makes the verdict
        //    stale, and a bump before the stamp implies the session table
        //    already held the session, so the count itself returns alive.
        let dead: Vec<(std::sync::Arc<TerminalOwnerEntry>, u64)> = snapshot
            .into_iter()
            .filter_map(|entry| {
                let stamp = entry.revivals.load(std::sync::atomic::Ordering::Relaxed);
                (entry.provider.live_handle_count() == 0).then_some((entry, stamp))
            })
            .collect();
        if dead.is_empty() {
            return;
        }

        // 3. Remove a dead-verdict entry only if its revival counter still
        //    equals the verdict stamp — a revival since the verdict keeps
        //    the provider visible. No provider method runs under this lock;
        //    only the entry-local atomic is read.
        let removed: Vec<std::sync::Arc<TerminalOwnerEntry>> = {
            let mut owners = self
                .terminal_owners
                .write()
                .expect("terminal_owners poisoned");
            let mut removed = Vec::new();
            owners.retain(|entry| {
                match dead
                    .iter()
                    .position(|(dead_entry, _)| std::sync::Arc::ptr_eq(dead_entry, entry))
                {
                    Some(index)
                        if entry.revivals.load(std::sync::atomic::Ordering::Relaxed)
                            == dead[index].1 =>
                    {
                        removed.push(entry.clone());
                        false
                    }
                    _ => true,
                }
            });
            if !removed.is_empty() {
                self.terminal_owners_gen
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
            removed
        };

        // 4. The final Drop of the removed providers (which may reenter the
        //    interface through their destructors) happens outside every
        //    lock. No post-hoc re-add is needed: providers that revived were
        //    never removed (step 3), and one that revives only after removal
        //    re-inserts itself through remember_terminal_owner.
        drop(removed);
    }

    /// Remember the frozen cursor/viewport cache of a handle freed through
    /// the shims. A copy survives provider replacement and owner pruning
    /// without keeping the old service alive (R2-F02).
    pub fn remember_terminal_retired_cache(
        &self,
        handle: i64,
        cache: auto_terminal_core::TerminalRetiredCache,
    ) {
        self.terminal_retired
            .lock()
            .expect("terminal_retired poisoned")
            .insert(handle, cache);
    }

    /// Retired cache for a freed handle (None = never observed freed here).
    pub fn terminal_retired_cache(
        &self,
        handle: i64,
    ) -> Option<auto_terminal_core::TerminalRetiredCache> {
        self.terminal_retired
            .lock()
            .expect("terminal_retired poisoned")
            .get(&handle)
            .copied()
    }

    /// PLAN-808: 当前 terminal provider（None → shim 回落进程共享 engine
    /// service；**不是**缺席错误/空值，见槽位注释）。
    pub fn terminal_provider(
        &self,
    ) -> Option<std::sync::Arc<dyn auto_terminal_core::TerminalHostProvider>> {
        self.terminal_provider
            .read()
            .expect("terminal_provider poisoned")
            .clone()
    }

    /// Common production binding order, including the VM overrides.
    /// Callers may merge external interfaces later; those merges invalidate proof.
    pub fn production() -> Self {
        let mut native_interface = NativeInterface::new();
        native_interface.register_std_shims();
        // Plan 094: Register manual FFI shims (cannot use #[rust_fn])
        crate::vm::ffi::register_stdlib_ffi(&mut native_interface);
        // Plan 555 T06: 分发组合子（interop.obj_* 家族，1860-1865）。
        crate::vm::interop::register_interop_natives(&mut native_interface);
        // Plan 198: Register #[rust_fn]-annotated shims via inventory
        native_interface.build_from_inventory();
        // The inventory-generated IO.read_line shim has no VM access and
        // reads process stdin. Rebind it after inventory so hosted requests
        // use their isolated input queue; CLI calls still fall back to stdin.
        crate::vm::ffi::stdlib::register_request_io_shims(&mut native_interface);
        // Override inventory shims with nanbox-aware versions for str methods
        // (inventory shims use String extraction which doesn't work with NanoValue stack)
        {
            native_interface.register(
                crate::vm::native::NATIVE_STR_CONTAINS,
                crate::vm::native::shim_str_contains,
            );
            native_interface.register(
                crate::vm::native::NATIVE_STR_STARTS_WITH,
                crate::vm::native::shim_str_starts_with,
            );
            native_interface.register(
                crate::vm::native::NATIVE_STR_ENDS_WITH,
                crate::vm::native::shim_str_ends_with,
            );
            native_interface.register(
                crate::vm::native::NATIVE_STR_TO_INT,
                crate::vm::native::shim_str_to_int_nv,
            );
            native_interface.register(
                crate::vm::native::NATIVE_STR_TO_UINT,
                crate::vm::native::shim_str_to_uint_nv,
            );
            // Plan 446 批三 D1: json.parse 占位 shim（rust_fn 版原样透传字符串）
            // 覆盖为 vm 感知版——经 Plan 340 转换器物化成 __json_object/
            // ListData 堆值，下游 GET_FIELD/for-in 直读。ID 取静态表
            // （auto.json.parse = 1902，native_catalog）。
            native_interface.register_typed_shim_by_name(
                "auto.json.parse",
                crate::vm::ffi::stdlib::shim_json_parse_vm,
                &["str"],
                "JsonValue?",
            );
            // PLAN-057 T6（等价性缺陷族③）：web 内建 natives 补齐——
            // Array.isArray/JSON.stringify/Math.trunc/Math.imul 此前未注册，
            // 调用点经未解析静态兜底（CALL_SPEC str 接收者臂）静默 None。
            // 名/ID 已入 native_catalog 静态表（NATIVE_ID_ENTRIES，codegen
            // 任意时刻可解析，沿 auto.json.parse 惯例）；此处绑定 shim 到
            // 固定 ID + 注册字面/canonical 双名（沿 build_from_inventory
            // 三名惯例）。注意 canonical 经 TYPE_CANONICAL_MAP：Array→
            // auto.list、Object→auto.obj。
            {
                const WEB_NATIVES: &[(
                    &str,
                    &str,
                    fn(&mut AutoTask, &AutoVM) -> Result<(), VMError>,
                )] = &[
                    (
                        "Array.isArray",
                        "auto.list.is_array",
                        crate::vm::ffi::stdlib::shim_web_is_array,
                    ),
                    (
                        "JSON.stringify",
                        "auto.json.stringify",
                        crate::vm::ffi::stdlib::shim_web_json_stringify,
                    ),
                    (
                        "Math.trunc",
                        "auto.math.trunc",
                        crate::vm::ffi::stdlib::shim_web_math_trunc,
                    ),
                    (
                        "Math.imul",
                        "auto.math.imul",
                        crate::vm::ffi::stdlib::shim_web_math_imul,
                    ),
                ];
                for (name, canonical, shim) in WEB_NATIVES {
                    let id = crate::vm::native_registry::BIGVM_NATIVES
                        .lock()
                        .unwrap()
                        .resolve_qualified(canonical)
                        .expect("PLAN-057 web native missing from NATIVE_ID_ENTRIES");
                    native_interface.register(id, *shim);
                    native_interface.register_name(name, id);
                    native_interface.register_name(canonical, id);
                }
            }
            // Plan 446 批三 D1(续): 文档类 json native（get/get_at/has_key/
            // len/keys/is_valid）双态化——堆文档首参序列化回文本走原路径，
            // json.get(json.parse(x), k) 文本工具链与点访问 idiom 互通。
            crate::vm::ffi::stdlib::override_json_doc_natives(&mut native_interface);
        }

        // Plan 011 (MS3-B): shell-host bridge natives.
        {
            native_interface.register(
                crate::vm::native::NATIVE_SHELL_SYSTEM,
                crate::vm::native::shim_shell_system,
            );
            native_interface.register(
                crate::vm::native::NATIVE_SHELL_SYSTEM_STATUS,
                crate::vm::native::shim_shell_system_status,
            );
            native_interface.register(
                crate::vm::native::NATIVE_SHELL_EXPORT,
                crate::vm::native::shim_shell_export,
            );
            native_interface.register(
                crate::vm::native::NATIVE_SHELL_EXIT,
                crate::vm::native::shim_shell_exit,
            );
            // PLAN-082: structured interop natives.
            native_interface.register(
                crate::vm::native::NATIVE_SHELL_QUERY,
                crate::vm::native::shim_shell_query,
            );
            native_interface.register(
                crate::vm::native::NATIVE_SHELL_RUN,
                crate::vm::native::shim_shell_run,
            );
        }

        // PLAN-738 §5.3：既有生产能力面的独立逻辑契约——在全部绑定与
        // 覆盖（inventory 推导、json 双态化、shell 桥）完成之后声明：
        // register_static/register/merge 都会撤销同 ID 契约，显式身份必
        // 须最后落位。声明按 stdlib/auto 公共签名；异步句柄面（Response/
        // HTTPStream/FileTransfer/RequestBuilder）经 push_handle 物化，
        // 返回按公共身份；wire 适配记入 error_shape，不改绑定/shim/次序。
        native_interface.declare_tcp_method_contracts();
        native_interface.declare_contract_identity("auto.http.get", &[], &["str"], "Response");
        native_interface.declare_contract_identity(
            "auto.http.get_stream",
            &[],
            &["str"],
            "HTTPStream",
        );
        native_interface.declare_contract_identity(
            "auto.http.request",
            &[],
            &["str", "str"],
            "RequestBuilder",
        );
        native_interface.declare_contract_identity(
            "auto.http.transfer_download",
            &[],
            &["str", "str", "str"],
            "FileTransfer",
        );
        native_interface.declare_contract_identity(
            "auto.http.file_response",
            &[],
            &["str", "str", "str"],
            "FileResponse",
        );
        native_interface.declare_contract_identity(
            "auto.http.upload_error",
            &[],
            &["int", "str"],
            "UploadReceipt",
        );
        native_interface.declare_contract_identity("auto.json.is_valid", &[], &["str"], "int");
        native_interface.declare_contract_identity(
            "auto.io.read_line_opt",
            &[],
            &[],
            "Option<str>",
        );
        native_interface.declare_contract_identity("auto.random.rand_int", &[], &["int"], "int");
        native_interface.declare_contract_identity("auto.random.rand_float", &[], &[], "float");
        native_interface.declare_contract_identity("auto.random.rand_bool", &[], &[], "bool");
        // 声明级泛型面（§5.3 明确 Generic 身份保留）：encode[T]/decode[T]
        // 的 producer 经 VMConvertible wire 动态取值，契约携带泛型身份。
        native_interface.declare_contract_identity("auto.json.encode", &["T"], &["T"], "str");
        native_interface.declare_contract_identity("auto.json.decode", &["T"], &["str"], "T");
        native_interface.declare_method_contract_identity(
            "auto.http.stream_next",
            "HTTPStream",
            &["HTTPStream"],
            "str",
        );
        native_interface.declare_method_contract_identity(
            "auto.http.stream_is_done",
            "HTTPStream",
            &["HTTPStream"],
            "int",
        );
        native_interface.declare_contract_identity(
            "auto.http.transfer_wait",
            &[],
            &["FileTransfer"],
            "str",
        );
        native_interface.declare_contract_identity(
            "auto.http.transfer_error",
            &[],
            &["FileTransfer"],
            "str",
        );
        native_interface.declare_contract_identity(
            "auto.http.transfer_cancel",
            &[],
            &["FileTransfer"],
            "void",
        );
        native_interface.declare_contract_identity(
            "auto.http.upload_metadata",
            &[],
            &["UploadSession"],
            "str",
        );
        // PLAN-755 T-03：HTTP 公共面契约覆盖补齐（SD-01 覆盖完整性规则）。
        // 以下家族此前 resolved+bound 但缺独立契约——真实程序引用即在编译期
        // 以 SIGNATURE_UNVERIFIED 拒绝（751-R2 实证 #9937/#3108；41 项全量
        // 缺口由 plan755_http_contract_tests 走查锁定）。签名逐字对齐
        // stdlib/auto/http.at 公共声明；句柄/Receipt 的 wire 适配语义沿用本
        // 节 error_shape 记法，不改绑定/shim/次序。
        //
        // Server 构造/路由链面：
        native_interface.declare_contract_identity("auto.http.server", &[], &[], "Server");
        native_interface.declare_method_contract_identity(
            "auto.http.server_get",
            "Server",
            &["Server", "str", "fn(Request) Response"],
            "Server",
        );
        native_interface.declare_method_contract_identity(
            "auto.http.server_post",
            "Server",
            &["Server", "str", "fn(Request) Response"],
            "Server",
        );
        native_interface.declare_method_contract_identity(
            "auto.http.server_put",
            "Server",
            &["Server", "str", "fn(Request) Response"],
            "Server",
        );
        native_interface.declare_method_contract_identity(
            "auto.http.server_delete",
            "Server",
            &["Server", "str", "fn(Request) Response"],
            "Server",
        );
        native_interface.declare_method_contract_identity(
            "auto.http.server_static",
            "Server",
            &["Server", "str", "str"],
            "Server",
        );
        native_interface.declare_method_contract_identity(
            "auto.http.server_listen",
            "Server",
            &["Server", "str"],
            "void",
        );
        native_interface.declare_contract_identity(
            "auto.http.listen",
            &[],
            &["str", "int", "fn(str, str, str) str"],
            "void",
        );
        // Response 构造面（Plan 346 redirect 在内）：
        native_interface.declare_contract_identity("auto.http.response", &[], &[], "Response");
        native_interface.declare_method_contract_identity(
            "auto.http.response_status",
            "Response",
            &["Response", "int"],
            "Response",
        );
        native_interface.declare_method_contract_identity(
            "auto.http.response_header",
            "Response",
            &["Response", "str", "str"],
            "Response",
        );
        native_interface.declare_method_contract_identity(
            "auto.http.response_text",
            "Response",
            &["Response", "str"],
            "Response",
        );
        native_interface.declare_method_contract_identity(
            "auto.http.response_html",
            "Response",
            &["Response", "str"],
            "Response",
        );
        native_interface.declare_method_contract_identity(
            "auto.http.response_bytes",
            "Response",
            &["Response", "[]byte"],
            "Response",
        );
        native_interface.declare_contract_identity(
            "auto.http.response_redirect",
            &[],
            &["str", "int"],
            "Response",
        );
        // Response sugar 构造族（ok/created/bad_request/not_found/internal_error）：
        // inventory 从 Rust 签名推导的契约返回 int（句柄 id 栈载荷），与公共层
        // Response 身份漂移——按 §5.3「返回按公共身份」在尾部覆盖重声明。
        native_interface.declare_contract_identity("auto.http.ok", &[], &["str"], "Response");
        native_interface.declare_contract_identity("auto.http.created", &[], &["str"], "Response");
        native_interface.declare_contract_identity(
            "auto.http.bad_request",
            &[],
            &["str"],
            "Response",
        );
        native_interface.declare_contract_identity(
            "auto.http.not_found",
            &[],
            &["str"],
            "Response",
        );
        native_interface.declare_contract_identity(
            "auto.http.internal_error",
            &[],
            &["str"],
            "Response",
        );
        // 客户端面（get/request 族已有；同步 post_sync/bearer 与状态查询补齐）：
        native_interface.declare_contract_identity(
            "auto.http.post",
            &[],
            &["str", "str"],
            "Response",
        );
        native_interface.declare_contract_identity(
            "auto.http.put",
            &[],
            &["str", "str"],
            "Response",
        );
        native_interface.declare_contract_identity("auto.http.delete", &[], &["str"], "Response");
        native_interface.declare_contract_identity(
            "auto.http.post_sync",
            &[],
            &["str", "str", "str"],
            "str",
        );
        native_interface.declare_contract_identity(
            "auto.http.post_bearer",
            &[],
            &["str", "str", "str"],
            "str",
        );
        native_interface.declare_contract_identity("auto.http.last_status", &[], &[], "int");
        // RequestBuilder 方法面（json[T] 为声明级泛型方法）：
        native_interface.declare_method_contract_identity(
            "auto.http.request_builder_header",
            "RequestBuilder",
            &["RequestBuilder", "str", "str"],
            "RequestBuilder",
        );
        native_interface.declare_method_contract_identity(
            "auto.http.request_builder_body",
            "RequestBuilder",
            &["RequestBuilder", "str"],
            "RequestBuilder",
        );
        native_interface.declare_generic_method_contract_identity(
            "auto.http.request_builder_json",
            "RequestBuilder",
            &["T"],
            &["RequestBuilder", "T"],
            "RequestBuilder",
        );
        native_interface.declare_method_contract_identity(
            "auto.http.request_builder_timeout",
            "RequestBuilder",
            &["RequestBuilder", "int"],
            "RequestBuilder",
        );
        native_interface.declare_method_contract_identity(
            "auto.http.request_builder_send",
            "RequestBuilder",
            &["RequestBuilder"],
            "Response",
        );
        // 流式面（post_stream 族 + close/iter 收尾；同 id 双名沿用
        // stream_next 的方法形声明——自由函数形不核 receiver，两序皆过）：
        native_interface.declare_contract_identity(
            "auto.http.post_stream",
            &[],
            &["str", "str"],
            "HTTPStream",
        );
        native_interface.declare_contract_identity(
            "auto.http.post_stream_with_headers",
            &[],
            &["str", "str", "str"],
            "HTTPStream",
        );
        native_interface.declare_method_contract_identity(
            "auto.http_stream.stream_close",
            "HTTPStream",
            &["HTTPStream"],
            "void",
        );
        native_interface.declare_method_contract_identity(
            "auto.http_stream.stream_iter",
            "HTTPStream",
            &["HTTPStream"],
            "int",
        );
        // SSE 逐步拉取面（PLAN-658/707）：
        native_interface.declare_contract_identity("auto.http.sse_open", &[], &["str"], "int");
        native_interface.declare_contract_identity("auto.http.sse_poll", &[], &["int"], "str");
        native_interface.declare_contract_identity("auto.http.sse_close", &[], &["int"], "void");
        native_interface.declare_contract_identity("auto.http.sse_error", &[], &["int"], "str");
        // 文件传输面（download/wait/error/cancel 已有；upload/progress 补齐）：
        native_interface.declare_contract_identity(
            "auto.http.transfer_upload",
            &[],
            &["str", "str", "str"],
            "FileTransfer",
        );
        native_interface.declare_contract_identity(
            "auto.http.transfer_next_progress",
            &[],
            &["FileTransfer"],
            "str",
        );
        // 服务端上传面（PLAN-730；metadata/error 已有，receive/commit/reject
        // 为 751-R2 实证缺口 #9937 家族）：
        native_interface.declare_contract_identity(
            "auto.http.upload_receive",
            &[],
            &["UploadRequest", "str", "str", "str"],
            "UploadSession",
        );
        native_interface.declare_contract_identity(
            "auto.http.upload_commit",
            &[],
            &["UploadSession", "str"],
            "UploadReceipt",
        );
        native_interface.declare_contract_identity(
            "auto.http.upload_reject",
            &[],
            &["UploadSession", "int", "str"],
            "UploadReceipt",
        );

        native_interface
    }

    /// Register a static shim (IDs 0-9999)
    ///
    /// Used by VM intrinsics and built-in stdlib functions.
    /// Panics if ID is out of range.
    pub fn register_static<F>(&mut self, id: u16, func: F)
    where
        F: Fn(&mut AutoTask, &AutoVM) -> Result<(), VMError> + Send + Sync + 'static,
    {
        assert!(id < STATIC_ID_MAX, "Static ID must be < {}", STATIC_ID_MAX);
        self.binding_revision = self.binding_revision.wrapping_add(1);
        self.contracts.remove(&id);
        self.reference_proofs.get_mut().unwrap().remove(&id);
        self.static_shims[id as usize] = Some(Arc::new(func));
    }

    /// Register a dynamic shim (IDs 10000+)
    ///
    /// Used by `use.rust` for runtime-loaded crates.
    /// Returns the assigned ID.
    pub fn register_dynamic<F>(&mut self, func: F) -> u16
    where
        F: Fn(&mut AutoTask, &AutoVM) -> Result<(), VMError> + Send + Sync + 'static,
    {
        let id = self.next_dynamic_id;
        self.binding_revision = self.binding_revision.wrapping_add(1);
        self.next_dynamic_id += 1;
        self.dynamic_shims.insert(id, Arc::new(func));
        id
    }

    /// Register a dynamic shim with a specific ID
    ///
    /// Used when the caller wants to control the ID assignment.
    pub fn register_dynamic_with_id<F>(&mut self, id: u16, func: F)
    where
        F: Fn(&mut AutoTask, &AutoVM) -> Result<(), VMError> + Send + Sync + 'static,
    {
        assert!(
            id >= DYNAMIC_ID_START,
            "Dynamic ID must be >= {}",
            DYNAMIC_ID_START
        );
        self.binding_revision = self.binding_revision.wrapping_add(1);
        self.contracts.remove(&id);
        self.reference_proofs.get_mut().unwrap().remove(&id);
        self.dynamic_shims.insert(id, Arc::new(func));
        if id >= self.next_dynamic_id {
            self.next_dynamic_id = id + 1;
        }
    }

    /// Unified lookup - used by CALL_NAT opcode
    ///
    /// Checks static array first (fast path), then dynamic HashMap.
    pub fn get(&self, id: u16) -> Option<&ShimFunc> {
        if id < STATIC_ID_MAX {
            self.static_shims.get(id as usize)?.as_ref()
        } else {
            self.dynamic_shims.get(&id)
        }
    }

    pub fn contract(&self, id: u16) -> Option<&NativeContract> {
        self.contracts.get(&id)
    }

    /// Restore the public receiver identity for the two TCP methods whose
    /// Rust inventory wrappers receive VM handles as integer arguments.
    pub fn declare_tcp_method_contracts(&mut self) {
        self.declare_method_contract_identity(
            "auto.net.tcp_listener_local_addr",
            "TcpListener",
            &["TcpListener"],
            "str",
        );
        self.declare_method_contract_identity(
            "auto.net.tcp_stream_read_line",
            "TcpStream",
            &["TcpStream"],
            "str",
        );
    }

    /// PLAN-738 §5.3：既有生产能力面的独立逻辑契约补充——与 shim 绑定
    /// 同址声明（`register_shim_by_name` 无类型元数据）。只补契约，不改
    /// 绑定、不改 shim、不改注册次序；wire 适配（VMConvertible 动态取值/
    /// push_handle 物化）记入 error_shape，不冒称同形。名字不可解析 =
    /// 无该面，静默跳过（引用核对只会查询可解析面）。
    #[track_caller]
    pub fn declare_contract_identity(
        &mut self,
        name: &str,
        generics: &[&str],
        parameters: &[&str],
        returns: &str,
    ) -> Option<u16> {
        let id = self.name_to_id.get(name).copied().or_else(|| {
            crate::vm::native_registry::BIGVM_NATIVES
                .lock()
                .unwrap()
                .resolve_qualified(name)
        })?;
        let error_shape =
            "VMError; wire adapter (VMConvertible value / push_handle materialization)".to_string();
        self.contracts.entry(id).or_insert_with(|| NativeContract {
            parameters: Vec::new(),
            returns: String::new(),
            producer: format!(
                "{}:{} (declared contract for {name})",
                std::panic::Location::caller().file().replace('\\', "/"),
                std::panic::Location::caller().line()
            ),
            receiver: None,
            is_static: false,
            generics: Vec::new(),
            parameter_modes: Vec::new(),
            error_shape,
        });
        let contract = self.contracts.get_mut(&id)?;
        contract.generics = generics.iter().map(|g| g.to_string()).collect();
        contract.parameters = parameters.iter().map(|p| p.to_string()).collect();
        contract.parameter_modes = parameters.iter().map(|_| "View".into()).collect();
        contract.returns = returns.into();
        Some(id)
    }

    /// PLAN-738 §5.3 方法面变体：receiver 身份（self 属主）随契约声明——
    /// 签名核对的 receiver/static 证据来自 producer 侧，与 shim 同址。
    pub fn declare_method_contract_identity(
        &mut self,
        name: &str,
        receiver: &str,
        parameters: &[&str],
        returns: &str,
    ) -> Option<u16> {
        let id = self.declare_contract_identity(name, &[], parameters, returns)?;
        if let Some(contract) = self.contracts.get_mut(&id) {
            contract.receiver = Some(receiver.into());
            contract.is_static = false;
        }
        Some(id)
    }

    /// PLAN-755 T-03：泛型方法面变体——`RequestBuilder.json[T]` 一类声明级
    /// 泛型方法需要 receiver 与 generics 同时随契约声明（§5.3 的方法变体
    /// 会把 generics 清空，无法表达该形态）。
    pub fn declare_generic_method_contract_identity(
        &mut self,
        name: &str,
        receiver: &str,
        generics: &[&str],
        parameters: &[&str],
        returns: &str,
    ) -> Option<u16> {
        let id = self.declare_contract_identity(name, generics, parameters, returns)?;
        if let Some(contract) = self.contracts.get_mut(&id) {
            contract.receiver = Some(receiver.into());
            contract.is_static = false;
        }
        Some(id)
    }

    pub fn binding_revision(&self) -> u64 {
        self.binding_revision
    }

    pub fn verify_core_reference(
        &self,
        id: u16,
    ) -> Result<Option<crate::stdlib_assembly::reference::ReferenceProof>, String> {
        if let Some(proof) = self.reference_proofs.lock().unwrap().get(&id).cloned() {
            return proof;
        }
        let proof = crate::stdlib_assembly::reference::verify_native_reference(id, self);
        self.reference_proofs
            .lock()
            .unwrap()
            .insert(id, proof.clone());
        proof
    }

    /// Logical contract belongs to the manually implemented producer, not to
    /// the .at scanner. Keep it next to the shim registration.
    #[track_caller]
    pub fn register_typed_shim_by_name(
        &mut self,
        name: &str,
        func: fn(&mut AutoTask, &AutoVM) -> Result<(), VMError>,
        parameters: &[&str],
        returns: &str,
    ) -> u16 {
        let id = self.register_shim_by_name(name, func);
        self.contracts.insert(
            id,
            NativeContract {
                parameters: parameters.iter().map(|p| p.to_string()).collect(),
                returns: returns.into(),
                producer: format!(
                    "{}:{} ({name})",
                    std::panic::Location::caller().file().replace('\\', "/"),
                    std::panic::Location::caller().line()
                ),
                receiver: None,
                is_static: false,
                generics: Vec::new(),
                parameter_modes: parameters.iter().map(|_| "View".into()).collect(),
                error_shape: "VMError".into(),
            },
        );
        id
    }

    #[track_caller]
    pub fn register_method_shim_by_name(
        &mut self,
        name: &str,
        func: fn(&mut AutoTask, &AutoVM) -> Result<(), VMError>,
        receiver: &str,
        parameters: &[&str],
        returns: &str,
    ) -> u16 {
        let id = self.register_typed_shim_by_name(name, func, parameters, returns);
        self.contracts.get_mut(&id).unwrap().receiver = Some(receiver.into());
        id
    }

    /// Check if an ID is static or dynamic
    pub fn is_static(&self, id: u16) -> bool {
        id < STATIC_ID_MAX
    }

    /// Register a name -> ID mapping for CALL_SPEC fallback (Plan 200 Task 3.3)
    pub fn register_name(&mut self, name: &str, id: u16) {
        self.binding_revision = self.binding_revision.wrapping_add(1);
        self.reference_proofs.get_mut().unwrap().clear();
        self.name_to_id.insert(name.to_string(), id);
    }

    /// Reverse map (ID -> first registered name) for disassembly tooltips
    pub fn id_to_name(&self) -> std::collections::HashMap<u16, String> {
        let mut map: std::collections::HashMap<u16, String> = std::collections::HashMap::new();
        for (name, &id) in &self.name_to_id {
            map.entry(id).or_insert_with(|| name.clone());
        }
        map
    }

    /// Look up a native ID by qualified name (e.g., "Result.Ok.map_err")
    ///
    /// Supports canonical normalization: "List.push" → checks "auto.list.push" etc.
    pub fn resolve(&self, name: &str) -> Option<u16> {
        // Direct lookup first
        if let Some(id) = self.name_to_id.get(name).copied() {
            return Some(id);
        }
        // Canonical normalization: "List.push" → "auto.list.push"
        if !name.starts_with("auto.") && !name.starts_with("rust.") && !name.starts_with("py.") {
            if let Some(canonical) = Self::to_canonical(name) {
                if let Some(id) = self.name_to_id.get(&canonical).copied() {
                    return Some(id);
                }
            }
        }
        None
    }

    /// Convert a short native name to its canonical "auto.X.Y" form.
    pub(crate) fn to_canonical(name: &str) -> Option<String> {
        let (prefix, rest) = name.split_once('.')?;
        use crate::vm::native_registry::TYPE_CANONICAL_MAP;
        for &(short, canonical_prefix) in TYPE_CANONICAL_MAP {
            if prefix == short {
                return Some(format!("{}.{}", canonical_prefix, rest));
            }
        }
        let lower = prefix.to_lowercase();
        Some(format!("auto.{}.{}", lower, rest))
    }

    /// Get the next available dynamic ID
    pub fn next_dynamic_id(&self) -> u16 {
        self.next_dynamic_id
    }

    /// Plan 212b Task 4: Merge shims from another NativeInterface into this one
    ///
    /// Used to merge Rust FFI bridge native shims into the main VM's
    /// NativeInterface after the bridge has loaded and registered functions.
    pub fn merge(&mut self, other: &NativeInterface) {
        self.binding_revision = self.binding_revision.wrapping_add(1);
        self.reference_proofs.get_mut().unwrap().clear();
        self.binding_conflicts
            .extend(other.binding_conflicts.clone());
        // Merge static shims
        for (id, shim) in other.static_shims.iter().enumerate() {
            if let Some(shim) = shim {
                if id < self.static_shims.len() {
                    self.contracts.remove(&(id as u16));
                    self.static_shims[id] = Some(shim.clone());
                }
            }
        }
        // Merge dynamic shims
        for (id, shim) in &other.dynamic_shims {
            self.contracts.remove(id);
            self.dynamic_shims.insert(*id, shim.clone());
        }
        self.contracts.extend(other.contracts.clone());
        self.name_to_id.extend(other.name_to_id.clone());
        // Advance next_dynamic_id if needed
        if other.next_dynamic_id > self.next_dynamic_id {
            self.next_dynamic_id = other.next_dynamic_id;
        }
    }

    /// Register a shim by name, looking up the ID from BIGVM_NATIVES.
    ///
    /// Used by inventory-based auto-registration (Plan 198).
    /// Returns the resolved ID.
    pub fn register_shim_by_name<F>(&mut self, name: &str, func: F) -> u16
    where
        F: Fn(&mut AutoTask, &AutoVM) -> Result<(), VMError> + Send + Sync + 'static,
    {
        use crate::vm::native_registry::BIGVM_NATIVES;
        let id = BIGVM_NATIVES
            .lock()
            .unwrap()
            .resolve_qualified(name)
            .unwrap_or_else(|| {
                panic!(
                    "register_shim_by_name: '{}' not found in BIGVM_NATIVES",
                    name
                )
            });
        self.register_shim_for_id(id, func);
        self.register_name(name, id);
        id
    }

    /// Route a name-resolved shim to the correct ID table.
    ///
    /// Kept separate so the dynamic-band path can be tested without mutating
    /// the process-global `BIGVM_NATIVES` registry.
    fn register_shim_for_id<F>(&mut self, id: u16, func: F)
    where
        F: Fn(&mut AutoTask, &AutoVM) -> Result<(), VMError> + Send + Sync + 'static,
    {
        // Names whose registry id landed in the dynamic band (>= 10000 —
        // e.g. after register_with_id bumped the shared next_id) must go to
        // the dynamic table: register_static asserts the static band and
        // would panic on a fresh-binary first VM run (Plan 768 T-07).
        if id < STATIC_ID_MAX {
            self.register_static(id, func);
        } else {
            self.register_dynamic_with_id(id, func);
        }
    }

    /// Collect all inventory-submitted FFI registrations and register them.
    ///
    /// Called during VM init after BIGVM_NATIVES is populated.
    pub fn build_from_inventory(&mut self) {
        use crate::vm::ffi::StaticFFIRegistration;
        let mut producers = HashMap::<u16, (&str, usize)>::new();
        // Preserve legacy registration order until the production conflict
        // gate is wired. Metadata describes the final selected callee; a
        // diagnostic must not silently choose a different first producer.
        for entry in inventory::iter::<StaticFFIRegistration> {
            use crate::vm::native_registry::BIGVM_NATIVES;
            let mut natives = BIGVM_NATIVES.lock().unwrap();
            let id = natives.resolve_qualified(entry.name).unwrap_or_else(|| {
                panic!(
                    "build_from_inventory: '{}' not found in BIGVM_NATIVES",
                    entry.name
                )
            });
            assert!(
                id < STATIC_ID_MAX,
                "Inventory shim '{}' resolved to dynamic ID {}",
                entry.name,
                id
            );
            if let Some((previous, pointer)) =
                producers.insert(id, (entry.producer, entry.shim as usize))
            {
                if pointer != entry.shim as usize {
                    self.binding_conflicts
                        .push((id, previous.into(), entry.producer.into()));
                }
            }
            self.static_shims[id as usize] = Some(Arc::new(entry.shim));
            self.contracts.insert(
                id,
                NativeContract {
                    parameters: entry.parameters.iter().map(|p| p.to_string()).collect(),
                    returns: entry.returns.into(),
                    producer: entry.producer.into(),
                    receiver: None,
                    is_static: false,
                    generics: Vec::new(),
                    parameter_modes: entry.parameters.iter().map(|_| "View".into()).collect(),
                    error_shape: "VMError".into(),
                },
            );
            // Register names for CALL_SPEC resolve
            // Register the inventory name itself (e.g., "Str.char_at")
            self.name_to_id.entry(entry.name.to_string()).or_insert(id);
            // Register the canonical form (e.g., "auto.str.char_at")
            if let Some(canonical) = natives.resolve_qualified_to_canonical(entry.name) {
                self.name_to_id.entry(canonical).or_insert(id);
            }
            // Also register lowercase variant for CALL_SPEC dispatch
            if let Some(lower) = Self::to_canonical(entry.name) {
                self.name_to_id.entry(lower).or_insert(id);
            }
        }
    }

    /// Legacy method for backwards compatibility
    /// Routes to register_static for IDs < 10000, register_dynamic otherwise
    pub fn register<F>(&mut self, id: u16, func: F)
    where
        F: Fn(&mut AutoTask, &AutoVM) -> Result<(), VMError> + Send + Sync + 'static,
    {
        if id < STATIC_ID_MAX {
            self.register_static(id, func);
        } else {
            self.register_dynamic_with_id(id, func);
        }
    }

    pub fn register_std_shims(&mut self) {
        // Plan 249 Phase 2: All shim bindings generated from unified catalog.
        // We define a local macro to consume for_each_native! because #[macro_export]
        // macros cannot use `self` (hygiene restriction). Local macros have no such issue.
        macro_rules! __register_shims {
            (($id:expr, $name:ident, $fn:ident, $canonical:expr) $(, $rest:tt)*) => {
                self.register($name, $fn);
                __register_shims!($($rest),*);
            };
            () => {};
        }
        for_each_native!(__register_shims);

        // Plan 249 Phase 5: Canonical name → ID mappings for CALL_SPEC fallback.
        // Generated from the same catalog as shim bindings (4th field = canonical name).
        {
            macro_rules! __register_names {
                (($id:expr, $name:ident, $fn:ident, $canonical:expr) $(, $rest:tt)*) => {
                    self.register_name($canonical, $name);
                    __register_names!($($rest),*);
                };
                () => {};
            }
            for_each_native!(__register_names);
        }

        // --- Extra aliases (1-to-N mappings, not in catalog) ---
        // PLAN-844 D-4：Str.contains/starts_with/ends_with 的纯 rust_fn shim
        // 退役后，其 inventory 注册附带的三种名字形态（"Str.X" 直名与
        // "auto.str.X" canonical/小写形态）失去唯一来源——PLAN-608 路由守护
        // 要求 CALL_SPEC 对这些名恒 registry 命中（nv shim 经 production()
        // 覆盖块绑定 id，函数面不变）；此处单点显式注册，取代偶发的
        // inventory 副作用。
        self.register_name("Str.contains", NATIVE_STR_CONTAINS);
        self.register_name("auto.str.contains", NATIVE_STR_CONTAINS);
        self.register_name("Str.starts_with", NATIVE_STR_STARTS_WITH);
        self.register_name("auto.str.starts_with", NATIVE_STR_STARTS_WITH);
        self.register_name("Str.ends_with", NATIVE_STR_ENDS_WITH);
        self.register_name("auto.str.ends_with", NATIVE_STR_ENDS_WITH);
        self.register_name("Result.map_err", NATIVE_RESULT_MAP_ERR);
        self.register_name("Result.Ok.map_err", NATIVE_RESULT_MAP_ERR);
        self.register_name("Result.Err.map_err", NATIVE_RESULT_MAP_ERR);
        self.register_name("auto.hashmap.insert", NATIVE_HASHMAP_INSERT_STR);
        self.register_name("auto.hashmap.set", NATIVE_HASHMAP_INSERT_STR);
        self.register_name("auto.hashmap.get", NATIVE_HASHMAP_GET_STR);
        self.register_name("auto.hashmap.contains_key", NATIVE_HASHMAP_CONTAINS);
        self.register_name("auto.hashmap.len", NATIVE_HASHMAP_SIZE);
    }
}

// Plan 249: Native constants generated from unified catalog
for_each_native!(gen_native_constants);

// ============================================================================
// Plan 178: Bit Operation Shims
// ============================================================================

// === Phase 4: Dynamic bitfield views ===

// ============================================================================
// Plan 240 VM-1: Math shims for f64 methods (sin/cos/tan/sqrt/pow etc.)
//
// PLAN-872 实测钉死：下列宏版 nv shim 经 `register_std_shims`（步骤 1）绑定
// 后，被 `build_from_inventory` 的 ffi/stdlib/math.rs 纯 shim 在同 ID 上
// **后写覆盖**（1700-1733 段）——稳态派发不经过本块（RETIRE 候选，须单独
// 签核；绑定序先序事实保留，见 attachments/872/binding-table.md 族二）。
// ============================================================================

// Plan 437: f64 算术 shim 的 tag 驱动操作数弹出——调用方常以 f32 字面量/
// 变量传参（lexer 默认 unsuffixed decimal 是 TAG_F32），盲 pop_f64 会把
// f32 nv 当 f64 位读成 NaN 且位模式保留，sin(pi/2) 呈"参数回显"假象。
// 与 engine::pop_f32_operand 同一 tag-first 哲学。
fn pop_f64_operand(task: &mut AutoTask) -> f64 {
    let nv = task.ram.pop_nv();
    if auto_val::is_f64(nv) {
        auto_val::decode_f64(nv)
    } else if auto_val::is_f32(nv) {
        auto_val::decode_f32(nv) as f64
    } else if auto_val::is_i32(nv) {
        auto_val::decode_i32(nv) as f64
    } else {
        auto_val::decode_f64(nv)
    }
}

macro_rules! math_unary_shim {
    ($name:ident, $method:ident) => {
        pub fn $name(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
            let n = pop_f64_operand(task);
            task.ram.push_f64(n.$method());
            Ok(())
        }
    };
}

macro_rules! math_binary_shim {
    ($name:ident, $method:ident) => {
        pub fn $name(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
            let exp = pop_f64_operand(task);
            let base = pop_f64_operand(task);
            task.ram.push_f64(base.$method(exp));
            Ok(())
        }
    };
}

math_unary_shim!(shim_math_sin, sin);
math_unary_shim!(shim_math_cos, cos);
math_unary_shim!(shim_math_tan, tan);
// sqrt — registered in native.rs but shim lives in ffi/stdlib.rs with #[rust_fn]
// (needed because #[rust_fn] uses VMConvertible which handles f64 encoding)
math_unary_shim!(shim_math_abs_f, abs);
math_unary_shim!(shim_math_floor, floor);
math_unary_shim!(shim_math_ceil, ceil);
math_unary_shim!(shim_math_round, round);
math_unary_shim!(shim_math_exp, exp);
math_unary_shim!(shim_math_ln, ln);
math_unary_shim!(shim_math_log2, log2);
math_unary_shim!(shim_math_log10, log10);
math_unary_shim!(shim_math_signum, signum);
math_unary_shim!(shim_math_asin, asin);
math_unary_shim!(shim_math_acos, acos);
math_unary_shim!(shim_math_atan, atan);
math_unary_shim!(shim_math_to_radians, to_radians);
math_unary_shim!(shim_math_to_degrees, to_degrees);
math_binary_shim!(shim_math_pow, powf);
math_binary_shim!(shim_math_powf, powf);

// === Manual constants (registered via register_shim_by_name, not in catalog) ===

// String methods (canonical IDs: auto.str.contains=1504, auto.str.starts_with=1505, auto.str.ends_with=1506)
pub const NATIVE_STR_CONTAINS: u16 = 1504;
pub const NATIVE_STR_STARTS_WITH: u16 = 1505;
pub const NATIVE_STR_ENDS_WITH: u16 = 1506;
pub const NATIVE_STR_TO_INT: u16 = 1516;
pub const NATIVE_STR_TO_UINT: u16 = 1523;

// Math functions registered in stdlib.rs via register_shim_by_name
pub const NATIVE_MATH_ABS: u16 = 1700;
pub const NATIVE_MATH_MIN: u16 = 1701;
pub const NATIVE_MATH_MAX: u16 = 1702;
pub const NATIVE_MATH_SQRT: u16 = 1750; // Changed from 1703 to avoid conflict
pub const NATIVE_MATH_MIN_F: u16 = 1714;
pub const NATIVE_MATH_MAX_F: u16 = 1715;
pub const NATIVE_MATH_CLAMP: u16 = 1725;

// Plan 011 (MS3-B): Shell-host bridge natives.
// system(cmd) -> String, system_status() -> Int, export(k,v), exit(code).
// These forward to `vm.host` (an Option<SharedHost>). When host is None
// (pure AutoLang), system returns "" and exit/export are no-ops.
pub const NATIVE_SHELL_SYSTEM: u16 = 1800;
pub const NATIVE_SHELL_SYSTEM_STATUS: u16 = 1801;
pub const NATIVE_SHELL_EXPORT: u16 = 1802;
pub const NATIVE_SHELL_EXIT: u16 = 1803;
pub const NATIVE_SHELL_QUERY: u16 = 1804;
pub const NATIVE_SHELL_RUN: u16 = 1805;

// === Standard Shims ===

// ============================================================================
// Plan 011 (MS3-B): Shell-host bridge natives
// ============================================================================

/// Read a string argument from the stack (tagged string index → text).
fn pop_string_arg(task: &mut AutoTask, vm: &AutoVM) -> String {
    let nv = crate::vm::native::pop_arg_nv(task);

    let _stake_nv = crate::vm::native::StakeGuard::nv(vm, nv);
    if auto_val::is_string(nv) {
        let idx = auto_val::decode_string(nv);
        vm.get_string(idx)
            .map(|b| String::from_utf8_lossy(&b).into_owned())
            .unwrap_or_default()
    } else {
        String::new()
    }
}

// ── Plan 413: code editor payload accessors (UI bridge) ─────────────────
// Read / jump live code_editor state by storage key from handler
// expressions. Text auto-binding (INPUT_TEXT) covers most oninput flows;
// these natives add cursor/selection/find access and explicit writes for
// both VM and script use.
//
// PLAN-802 依赖倒置：本族 22 项（2910..2915、2919..2924、2932/2933、2939、
// 9906、9908..9913）不再直引 `crate::ui::code_editor`（editor 注册表 /
// 寄宿 scroll controller / OS 剪贴板桥）——经本 VM NativeInterface 装配的
// [`crate::vm::host_capabilities::CodeEditorHostProvider`] 供值（UI 宿主
// 注册委托既有 registry 的 adapter；外部 no-ui 宿主可注入自定义实现）。
// shim 体因此 feature 中性：`code-editor` feature 仅决定 UI adapter 是否
// 编译/装配，不再决定本函数的编译形态。
//
// 缺席语义（T-01 冻结矩阵 = 迁移前 feature-off 分支逐字面值）：无
// provider → raise `…: the \`code-editor\` feature is disabled`（弹参前
// raise，镜像旧 cfg-off stub）；9913 scroll_to 唯一例外——弹参后 push
// Bool false（值缺席臂）。cut/copy/paste 为组合特征文案（
// `code-editor`+`ui-clipboard`），provider 在位但宿主无剪贴板面
// （`Option<bool>` → None）时同文案。

// ── PLAN-703 供⑤: diff engine consumption endpoints (9915/9916/9917) ──
// Envelope JSON field-identical to the downstream replacement-seam
// contracts (auto-edit diff-view.md SD-01/PLAN-011/012); errors are VALUES
// in the envelope err field, never raises (669/687 convention).

// ── PLAN-701 供④: time 族 VM 运行时 shim（catalog 1200/1201/1205 既有
// 登记位的实装；此前裸/use 两形态均返 0——m1-supply §9 观察清偿）。语义
// 与 a2r-std/src/time.rs 三方一致（GOAL-003）：now_ms/now_sec = Unix epoch
// （.at stdlib time.at/time.rs.at 文档注释口径，非单调钟），now = epoch
// 秒十进制字符串。下游 bench 毫秒差值在双轨语义下同形。

/// `auto.time.now_ms() -> i64` — Unix epoch milliseconds.
/// PLAN-872 实测钉死（族四）：本 nv 版经 catalog 绑定 id 1200 后，被
/// `build_from_inventory` 的 sys_env.rs 纯 shim（`.unwrap()`）**后写覆盖**，
/// 稳态派发不经过本函数（`unwrap_or(0)` 语义当前不在生效路径上；RETIRE
/// 候选，须单独签核，attachments/872/binding-table.md 族四）。
pub fn shim_time_now_ms(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);
    vm.push_i64_vm(task, ms);
    Ok(())
}

/// `auto.time.now_sec() -> i64` — Unix epoch seconds.
pub fn shim_time_now_sec(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let s = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    vm.push_i64_vm(task, s);
    Ok(())
}

/// `auto.time.now() -> str` — epoch seconds as decimal string
/// (a2r-std `time_now` alias同形).
pub fn shim_time_now(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let s = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_default();
    let idx = vm.add_string(s.into_bytes());
    vm.rc_push_str_idx(task, idx as usize);
    Ok(())
}

// ── PLAN-701 供⑥: `shell_add_recent(path) -> Bool` — Windows jump-list
// Recent 面 shim（auto-edit PLAN-014 T-01 裁定 (b) 用户确认改道件）。
// One-call form: `SHAddToRecentDocs(SHARD_PATHW, path)` — the opened file
// lands in the shell Recent items (taskbar jump list). Zero registry
// association surface (HKCR/Classes untouched); ICustomDestinationList
// custom-task categories are a non-goal (裁定成文). Non-Windows: no-op
// returning false (恒定可链接——a2r 轨同形).

/// Non-Windows + test-visible core: cfg(windows) FFI one-call; elsewhere a
/// false no-op. Exposed (not only a shim body) so the a2r/merged rust
/// emission (`vm_builtin_host_call` arm) links the same implementation.
pub fn shell_add_recent(path: &str) -> bool {
    #[cfg(windows)]
    {
        #[link(name = "shell32")]
        extern "system" {
            fn SHAddToRecentDocs(flags: u32, path: *const core::ffi::c_void);
        }
        const SHARD_PATHW: u32 = 0x0000_0002;
        if path.is_empty() {
            return false;
        }
        let wide: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
        unsafe {
            SHAddToRecentDocs(SHARD_PATHW, wide.as_ptr() as *const core::ffi::c_void);
        }
        true
    }
    #[cfg(not(windows))]
    {
        let _ = path;
        false
    }
}

// ── Plan 428 P1: code folding natives ─────────────────────────────────

// ── Plan 418: editor action natives (menu/toolbar handlers) ────────────
// Same semantics as the editor's internal Ctrl+Z/Y/A/C/X/V handling, but
// triggered programmatically from `.at` handlers (auto-edit menus).
// PLAN-802: provider 供值；cut/copy/paste 的 `Option<bool>` None 臂 =
// 宿主有编辑器面但无 OS 剪贴板面（= 迁移前 code-editor on +
// ui-clipboard off 组合的 raise 臂，组合特征文案逐字面值）。

// ── Plan 418: OS clipboard natives (arboard) ───────────────────────────
// PLAN-785 依赖倒置：core 不再直引 `crate::ui::clipboard`——经本 VM
// NativeInterface 装配的 [`crate::vm::host_capabilities::ClipboardTextProvider`]
// 供值（UI 宿主注册 arboard adapter；外部 no-ui 宿主可注入自定义实现）。
// 能力=注入：provider 缺席时保留迁移前 no-ui 构建的同形 VMError（函数名
// 即 2925/2926 诊断定位面）；provider 在而 OS 不可用 = empty/false（值
// 语义不变，非 raise）。shim 体因此 feature 中性——`ui-clipboard` 仅决定
// adapter 是否编译/装配，不再决定本函数的编译形态。

// ── Plan 485: native clipboard files/images (CF_HDROP / DIBV5+PNG) ─────
// PLAN-790 依赖倒置：core 不再直引 `ui::clipboard_native`——经本 VM
// NativeInterface 装配的
// [`crate::vm::host_capabilities::ClipboardMediaProvider`] 供值（UI 宿主
// 在 windows+native-clipboard 下注册 Win32 adapter；外部 no-ui 宿主可注入
// 自定义实现）。降级契约（G3）原样：provider 缺席或供值失败 = 空表 /
// false / null（**值语义**，非 raise——与文本族 2925/2926 的
// feature-disabled VMError 形不同，迁移前非 Windows / 未开
// native-clipboard 臂即此形），.at 代码零平台分支。shim 体因此
// feature/平台中性——`native-clipboard` 仅决定 UI adapter 是否编译/装配，
// 不再决定本函数的编译形态。

// ── Plan 488: OLE 拖出（auto.dnd.start；拖入路由在 renderer 侧） ────────
// 降级契约（G5）：无 provider（非 Windows / 未开 `native-dnd` / no-ui /
// 未装配）返 false，事件不触发，.at 代码零条件分支。PLAN-804 依赖倒置：
// shim 不再直引 `ui::native_dnd`——载荷 JSON 交
// [`crate::vm::host_capabilities::DndHostProvider`] 唯一解释（UI adapter
// 委托既有 parse_payload_json + win32::start_drag，windows ∧ native-dnd ∧
// ui 三门控安装），shim 体因此 feature 中性。

// ============================================================================
// PLAN-656 T-03: scroll-pane controller natives
// （controller 队列；renderer update 期排空消费。
//   语义：controller 是 logical handle——见 docs/plans/656 §5；
//   方法语法糖 scroll.to_end() 属后续语言面，v1 为函数族形态。）
// PLAN-794 依赖倒置：八 shim（9900..9905 + 9922/9923）不再直引 UI 的
// scroll 模块——经本 VM NativeInterface 装配的
// [`crate::vm::host_capabilities::ScrollHostProvider`] 供值（UI 宿主注册
// 委托既有 controller 注册表/锚点状态机的 adapter；no-ui 宿主可注入自定义
// 实现）。缺席降级 = 迁移前 no-ui 臂逐字面值（Int 0 / false / 0.0 /
// "error:unsupported" / "unknown"——值语义非 raise）；shim 体因此 feature
// 中性——`ui` 仅决定 UI adapter 是否编译/装配。ID/arity/返回形态与 781
// 状态机语义冻结于计划 T-01 矩阵。
// ============================================================================

/// 弹出 controller 句柄实参。句柄是不透明非空串（`scroll_controller()`
/// 分配的形态，或 .at 侧直接以字符串状态字段作柄——两者等价；绑定发生
/// 在 pane 的 `controller:` prop）。空串 = None（9901..9904 跳过入队）。
fn pop_controller_handle(task: &mut AutoTask, vm: &AutoVM) -> Option<String> {
    let s = pop_string_arg(task, vm);
    (!s.is_empty()).then_some(s)
}

/// 弹出可选轴实参："x"/"y" → [`ScrollAxisDto`]；缺省 Y（单轴 pane 简写）。
/// 轴词只认 "x"，其余（含非字符串实参）归 Y——旧臂解码规则逐条保持。
fn pop_axis_arg(task: &mut AutoTask, vm: &AutoVM) -> crate::vm::host_capabilities::ScrollAxisDto {
    use crate::vm::host_capabilities::ScrollAxisDto;
    let nv = crate::vm::native::pop_arg_nv(task);
    let _stake = crate::vm::native::StakeGuard::nv(vm, nv);
    if auto_val::is_string(nv) {
        let idx = auto_val::decode_string(nv);
        let s = vm
            .get_string(idx)
            .map(|b| String::from_utf8_lossy(&b).into_owned())
            .unwrap_or_default();
        // 池份额已由 StakeGuard 释放；轴词只认 "x"，其余归 Y。
        if s == "x" {
            ScrollAxisDto::X
        } else {
            ScrollAxisDto::Y
        }
    } else {
        ScrollAxisDto::Y
    }
}

// ── Plan 418: native file dialogs（rfd，同步 API）──────────────────────
// PLAN-804 依赖倒置：OS 对话框、父窗选择（原 dialog_parent，Win32
// EnumWindows 最大可见自有顶层窗 + rfd set_parent）迁 crate 上层
// `host_adapters::dialog`（ui-dialog 单门控，独立于 ui renderer 门）；
// shim 只编解码并调用 [`crate::vm::host_capabilities::DialogHostProvider`]
// ——过滤语义在单一 OS 实现维护，core 不复制第二套 parser。两者返回
// "" on cancel/unavailable（headless CI included）。
//
// 缺席语义（T-01 冻结矩阵 = 迁移前 feature-off 臂逐字面值）：无 provider
// → feature-disabled 形 VMError，**在弹实参前 raise**（旧 off 臂不弹实参；
// 错误中止下栈面不可观察，缺席矩阵按逐字面值冻结）。在位 provider 取消
// 或不可用返回空串。Plan 428 债修（父窗锚定）语义由 VM 默认 adapter
// （parented 策略）保持——见 host_adapters::dialog。

// ── Plan 541: System Monitor Natives (auto.sys.*) ───────────────────────────

// PLAN-802: code_editor 族 22 项已单态化（feature 中性，provider 装配）——
// 本处原 feature-off stub 群（text/delta/edit/cursor_line/cursor_col/
// selection_len/find/set_text/fold_toggle/fold_hidden_count/undo/redo/
// select_all/cut/copy/paste）并入各 shim 的缺席臂，逐字面值保留。

// ── Plan 413 follow-up: console natives (in-app Console panel) ──────────
// Backed by the process-wide ring buffer in vm::ui_console (also fed by
// vm_print, so plain `print()` lines show up in the panel too).

/// Generic print that handles any value type.
/// If the value is a tagged string index (negative), prints the string.
/// Otherwise prints as an integer.

/// Format a RustStdlibObject for display.
pub fn format_rust_stdlib_obj(obj: &RustStdlibObject) -> String {
    match obj.type_name.as_str() {
        "Instant" => "<Instant>".to_string(),
        "Duration" => {
            if let Some(dur) = obj.downcast_ref::<std::time::Duration>() {
                format!("{}ms", dur.as_millis())
            } else {
                "<Duration>".to_string()
            }
        }
        "PathBuf" => {
            if let Some(p) = obj.downcast_ref::<std::path::PathBuf>() {
                format!("{}", p.display())
            } else {
                "<PathBuf>".to_string()
            }
        }
        "Arc" => "<Arc>".to_string(),
        "Mutex" => "<Mutex>".to_string(),
        "Box" => "<Box>".to_string(),
        "RefCell" => "<RefCell>".to_string(),
        "std::cell::OnceCell::Value" => {
            if let Some(s) = obj.downcast_ref::<String>() {
                s.clone()
            } else {
                "<OnceCell::Value>".to_string()
            }
        }
        "semver::Version" => {
            if let Some(mutex) = obj.downcast_ref::<std::sync::Mutex<semver::Version>>() {
                format!("{}", mutex.lock().unwrap())
            } else {
                "<semver::Version>".to_string()
            }
        }
        // PLAN-596 T-07 (DIV-DEP-8 VM 半边收口): url::Url Display 路由——
        // print 与 TYPE_TO_STR(.to(str)) 共用本表,镜像 semver::Version 臂。
        "url::Url" => {
            if let Some(mutex) = obj.downcast_ref::<std::sync::Mutex<url::Url>>() {
                format!("{}", mutex.lock().unwrap())
            } else {
                "<url::Url>".to_string()
            }
        }
        "semver::VersionReq" => {
            if let Some(mutex) = obj.downcast_ref::<std::sync::Mutex<semver::VersionReq>>() {
                format!("{}", mutex.lock().unwrap())
            } else {
                "<semver::VersionReq>".to_string()
            }
        }
        other => format!("<{}>", other),
    }
}

pub fn shim_assert(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    {
        let nv = crate::vm::native::pop_arg_nv(task);

        let _stake_nv = crate::vm::native::StakeGuard::nv(vm, nv);
        let is_true = if auto_val::is_bool(nv) {
            auto_val::decode_bool(nv)
        } else if auto_val::is_i32(nv) {
            let v = auto_val::decode_i32(nv);
            v != 0 && v != (-2147483647i32)
        } else {
            true
        };
        if !is_true {
            return Err(VMError::RuntimeError("Assertion failed".to_string()));
        }
    }
    Ok(())
}

// ============================================================================
// List Native Shims
// ============================================================================

/// Get element at index.
/// Stack: list_id, index -> elem
// Plan 077 Phase 5: Updated to use unified registry
/// Plan 432 D26 修复:ListData<i32> 元素的容器侧引用记账。
/// 编码契约(nano_value.rs encode_string 注释):字符串以负哨兵 -(idx+1)
/// 落在 i32 空间(decode_i32(encode_string(idx)) 即该负值);≥HEAP_ID_BASE
/// 为裸堆 id;其余为纯 int。
/// 缺陷:push/set/insert/clear 此前只对堆 id retain/release——负哨兵从不
/// 获得池份额,调用方栈份额被 native 死区结算释放后,列表持裸哨兵指向
/// 已回收池槽;读回 push_tagged_value_rc 的 +1 复活墓碑 → canary UAF
/// (242:S2-432,复现 repro_242_string_pool_uaf:循环内 push 运行期串后读回)。
#[inline]
fn list_i32_elem_retain(vm: &AutoVM, val: i32) {
    if (val as i64) >= crate::vm::rc::HEAP_ID_BASE as i64 {
        vm.rc_retain_id(val as u64);
    } else if val < 0 {
        vm.pool_retain((-val - 1) as usize);
    }
}

#[inline]
fn list_i32_elem_release(vm: &AutoVM, val: i32) {
    if (val as i64) >= crate::vm::rc::HEAP_ID_BASE as i64 {
        vm.rc_release_id(val as u64);
    } else if val < 0 {
        vm.pool_release((-val - 1) as usize);
    }
}

/// Plan 419: ListData<i32> 元素回栈 —— ≥HEAP_ID_BASE 的元素是裸堆 id,+1。
/// (调用方持有 &AutoVM 时用本函数替代 push_tagged_value。)
fn push_tagged_value_rc(vm: &AutoVM, task: &mut AutoTask, val: i32) {
    if val < 0 {
        let str_idx = (-(val) - 1) as u32;
        // Plan 432 D30: 池界外的负值是裸 int 而非字符串哨兵 ——
        // ListData<i32> 的负 int 元素经 .at get 需原样往返
        // (此前越界哨兵产生悬垂 string tag,读回即坏)
        if (str_idx as usize) < vm.strings.read().unwrap().len() {
            vm.rc_push_str_idx(task, str_idx as usize);
        } else {
            task.ram.push_i32(val);
        }
    } else {
        vm.rc_push_id(task, val as u64);
    }
}

/// Plan 335: 把 ListData<Value> 里取出的 `Value` 推回 VM 栈（nanbox 编码）。
/// 用于 get/pop/remove 等 struct List 元素访问的返回值。
/// VmRef → encode_object；Str → 新增 string pool 条目；Int → i32；
/// Bool → encode_bool（Plan 406：裸 1/0 会让 is_bool 消费者失效）。
/// Plan 419: VmRef/裸 id(≥4M 的 Int)入栈 +1 —— get(拷贝语义)与
/// pop/remove(容器侧另行 release,转移语义)统一 +1。
fn push_value(task: &mut AutoTask, vm: &AutoVM, val: &auto_val::Value) {
    match val {
        auto_val::Value::Int(i) => {
            if (*i as i64) >= crate::vm::rc::HEAP_ID_BASE as i64 {
                vm.rc_push_id(task, *i as u64);
            } else {
                task.ram.push_i32(*i);
            }
        }
        auto_val::Value::Bool(b) => task.ram.push_nv(auto_val::encode_bool(*b)),
        // Plan 437: float 元素此前落进 `_ => push_i32(0)` 兜底——List<float>.get()
        // 恒返回 0。镜像 GET_ELEM 的 ListData<Value> 臂（Double→f64/Float→f32）。
        auto_val::Value::Double(d) => task.ram.push_f64(*d),
        auto_val::Value::Float(f) => task.ram.push_f32(*f as f32),
        auto_val::Value::VmRef(r) => {
            vm.rc_push(task, auto_val::encode_object(r.id as u32));
        }
        auto_val::Value::Str(s) => {
            let str_idx = vm.add_string(s.as_bytes().to_vec());
            vm.rc_push_str_idx(task, str_idx as usize);
        }
        _ => task.ram.push_i32(0), // Nil 等兜底为 0
    }
}

/// PLAN-082: 把 host 侧返回的整棵 `Value` 树物化为 VM 堆对象并把根压栈。
/// * `Obj` → 堆 `ObjectData`（字段递归转换）——Dot/字段访问与原生 struct 同路径；
/// * `Array` → 堆 `ListData<Value>`——for-in 逐元素迭代、`.len()`/`.get` 可用，
///   容器元素是 `Value::VmRef`（嵌套 Obj/Array）或内联标量（Str/Int/Bool）；
/// * 标量走既有 [`push_value`]（Str 经 string pool）。
/// 根引用按**裸堆 id** 入栈（`rc_push_id`，clipboard natives 同款）——
/// `.len()` 等 `pop_arg_i32` 消费者只认裸 id；TAG_OBJECT 编码仅在部分
/// 消费者有兼容臂（Plan 437 判例）。T-01 spike 实测锚点。
fn push_value_tree(task: &mut AutoTask, vm: &AutoVM, val: &auto_val::Value) {
    match val {
        auto_val::Value::Obj(_) | auto_val::Value::Array(_) => {
            if let auto_val::Value::VmRef(r) = heap_value(vm, val) {
                vm.rc_push_id(task, r.id as u64);
            }
        }
        _ => push_value(task, vm, val),
    }
}

/// PLAN-082: [`push_value_tree`] 的递归臂——容器（Obj/Array）返回指向新堆对象的
/// `Value::VmRef`（insert 的初始计数即容器所有权），标量原样克隆。
fn heap_value(vm: &AutoVM, val: &auto_val::Value) -> auto_val::Value {
    match val {
        auto_val::Value::Obj(o) => {
            let mut rec = crate::vm::types::ObjectData::new();
            for (k, fv) in o.iter() {
                rec.set(k.clone(), heap_value(vm, fv));
            }
            let id = vm.insert_heap_object(rec);
            auto_val::Value::VmRef(auto_val::VmRef { id: id as usize })
        }
        auto_val::Value::Array(a) => {
            let mut list = crate::vm::types::ListData::<auto_val::Value>::new();
            for ev in &a.values {
                list.push(heap_value(vm, ev));
            }
            let id = vm.insert_heap_object(list);
            auto_val::Value::VmRef(auto_val::VmRef { id: id as usize })
        }
        other => other.clone(),
    }
}

/// Plan 335: 栈上的 nanbox NanoValue 解码为 `Value`（set/insert/contains 等写入/比较用）。
/// VmRef / Int / String / Bool / Nil 五类，与 shim_list_push 的编码一致。
/// Plan 403: string values now decode to Value::Str (actual bytes), not
/// Value::Int(pool_index) — keeps List<Value> string elements round-tripping.
/// PLAN-872（族六，DISTINGUISH）：与 interop.rs 的 `nv_to_value(vm, nv)` 是
/// **双签名双语义**的并存版——本版无 vm 访问（string 退化为
/// Value::Int(pool_index) 尽力解码、无 f32/f64 臂），消费面=collections
/// 写入/比较；interop 版消费面=obj_* 字段写入（真 Str 字节+浮点臂）。
/// 双侧均有活消费者，勿合并（attachments/872/binding-table.md 族六）。
fn nv_to_value(nv: auto_val::NanoValue) -> auto_val::Value {
    if auto_val::is_future(nv) {
        // PLAN-776 R8-01：Future 句柄保真映射 Value::Future（句柄视图）。
        return Value::Future(Box::new(auto_val::FutureData {
            id: auto_val::decode_future(nv) as usize,
            state: auto_val::FutureState::Pending,
            result: None,
        }));
    }
    if auto_val::is_i32(nv) {
        Value::Int(auto_val::decode_i32(nv))
    } else if auto_val::is_object(nv) {
        Value::VmRef(auto_val::VmRef {
            id: auto_val::decode_object(nv) as usize,
        })
    } else if auto_val::is_string(nv) {
        // Plan 403: resolve to actual string bytes via the pool index.
        // NOTE: this is a best-effort decode; callers that need the string
        // should use the pool directly. We fall back to Int if unavailable.
        Value::Int(auto_val::decode_string(nv) as i32)
    } else if auto_val::is_null(nv) {
        Value::Nil
    } else if auto_val::is_bool(nv) {
        Value::Bool(auto_val::decode_bool(nv))
    } else {
        Value::Int(auto_val::decode_i32(nv))
    }
}

/// Plan 335: 两个 Value 是否相等（用于 contains 等）。
/// VmRef 按 id 比较（同一 struct 实例视为相等）；Int/Bool/Str 按值；跨类型不相等。
fn values_eq(a: &auto_val::Value, b: &auto_val::Value) -> bool {
    match (a, b) {
        (auto_val::Value::Int(x), auto_val::Value::Int(y)) => x == y,
        (auto_val::Value::Bool(x), auto_val::Value::Bool(y)) => x == y,
        (auto_val::Value::VmRef(x), auto_val::Value::VmRef(y)) => x.id == y.id,
        _ => false,
    }
}

/// JS Array.splice start normalization: negative counts from the end,
/// everything clamps into [0, len].
#[inline]
fn normalize_splice_start(start_raw: i32, len: usize) -> usize {
    if start_raw < 0 {
        len.saturating_sub((-start_raw) as usize)
    } else {
        (start_raw as usize).min(len)
    }
}

// ============================================================
// List Higher-Order Functions (Plan 206: Closure HOF)
// ============================================================

/// Helper: get list elements as Vec<i32> from a list heap object
fn get_list_i32_elements(vm: &AutoVM, list_id: u64) -> Result<Vec<i32>, VMError> {
    use crate::vm::types::ListData;

    // First check heap objects (List created via List.new or previous HOF)
    if let Some(obj) = vm.get_heap_object(list_id) {
        let guard = obj.read().unwrap();
        if let Some(list) = guard.as_any().downcast_ref::<ListData<i32>>() {
            return Ok(list.elems.clone());
        }
    }

    // Plan 446 批四 D4: 此前对 ListData<Value> 有"非 Int 元素 coerce 为 0"的
    // fallback——对象/字符串元素的 find/map/any 等全部 HOF 走 i32 快路径时
    // 谓词接到恒 0，永不命中（plan454 已在 obj.find 剔除同款 tag 位误读，
    // 本处补齐 list 族）。Value 列表一律交由各 shim 的 Value 路径处理。
    Err(VMError::RuntimeError(format!(
        "Invalid list ID: {}",
        list_id
    )))
}

/// Helper: create a new array from Vec<i32> elements, return array ID.
/// Plan 390 §15 H3b: stored as ListData<Value> in heap_objects (same storage
/// as CREATE_ARRAY) so results work with ARRAY_LEN etc.
pub(crate) fn create_list_from_i32(vm: &AutoVM, elems: Vec<i32>) -> u64 {
    let values: Vec<auto_val::Value> = elems.into_iter().map(|e| auto_val::Value::Int(e)).collect();
    vm.insert_heap_object(crate::vm::types::ListData {
        elems: values,
        storage: None,
    })
}

/// Plan 340: 从任意 ListData<T> 获取 Vec<Value>（i32 自动转 Value::Int）。
/// 查 heap_objects 的 ListData<Value> 和 ListData<i32>（array 字面量已并入，
/// Plan 390 §15 H3b）。用于 HOF shim（map/filter/find 等）的 Value fallback 路径。
fn get_list_elements_as_value(vm: &AutoVM, list_id: u64) -> Result<Vec<auto_val::Value>, VMError> {
    use crate::vm::types::ListData;

    if let Some(obj) = vm.get_heap_object(list_id) {
        let guard = obj.read().unwrap();
        if let Some(list) = guard.as_any().downcast_ref::<ListData<Value>>() {
            return Ok(list.elems.clone());
        }
        if let Some(list) = guard.as_any().downcast_ref::<ListData<i32>>() {
            return Ok(list
                .elems
                .iter()
                .map(|i| auto_val::Value::Int(*i))
                .collect());
        }
    }

    Err(VMError::RuntimeError(format!(
        "Invalid list ID: {}",
        list_id
    )))
}

/// Plan 340: 从 Vec<Value> 创建新列表，返回 registry ID（存入 heap_objects，
/// Plan 390 §15 H3b，同 CREATE_ARRAY 的 ListData<Value> 存储）。
fn create_list_from_value(vm: &AutoVM, elems: Vec<auto_val::Value>) -> u64 {
    vm.insert_heap_object(crate::vm::types::ListData {
        elems,
        storage: None,
    })
}

/// Helper: check if a VM value is truthy (handles both conventions)
/// True values: 1, i32::MIN (-2147483648), or any non-zero/non-false value
/// False values: 0, i32::MIN+1 (-2147483647)
#[inline]
fn vm_is_truthy(val: i32) -> bool {
    val != 0 && val != i32::MIN + 1
}

/// Helper: convert a VM value to a printable boolean (1 or 0)
#[inline]
#[allow(dead_code)]
fn vm_to_printable_bool(val: i32) -> i32 {
    if vm_is_truthy(val) {
        1
    } else {
        0
    }
}

// ============================================================================
// Result HOF Native Shims
// ============================================================================

/// Result.map_err(closure) — if Err, call closure with error value; if Ok, pass through.
/// Stack: closure_id, result_instance_id -> new_result_instance_id
/// Plan 200 Task 3.3
pub fn shim_result_map_err(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::generic_registry::GenericInstanceData;
    let closure_id = crate::vm::native::pop_arg_i32(task) as u32;

    let _stake_closure_id = crate::vm::native::StakeGuard::new(vm, closure_id as i64 as u64);
    let result_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_result_id = crate::vm::native::StakeGuard::new(vm, result_id as i64 as u64);

    // Look up the Result heap object
    let obj = vm.get_heap_object(result_id).ok_or_else(|| {
        VMError::RuntimeError(format!("map_err: invalid heap object {}", result_id))
    })?;
    let guard = obj.read().unwrap();
    let instance = guard
        .as_any()
        .downcast_ref::<GenericInstanceData>()
        .ok_or_else(|| VMError::RuntimeError("map_err: not a Result heap object".into()))?;

    if instance.mono_name == "Result.Err" {
        let err_val = match instance.fields.first() {
            Some(auto_val::Value::Int(v)) => *v,
            _ => {
                return Err(VMError::RuntimeError(
                    "map_err: Err field not an int".into(),
                ));
            }
        };
        // Release the read lock before calling closure (which may access heap)
        drop(guard);

        // Call closure with the error value as argument
        task.ram.push_i32(err_val);
        vm.call_closure(task, closure_id, 1)?;
        let new_err_val = crate::vm::native::pop_arg_i32(task);

        let _stake_new_err_val = crate::vm::native::StakeGuard::new(vm, new_err_val as i64 as u64);

        // Wrap back in Result.Err heap object
        let new_instance = GenericInstanceData::new(
            "Result.Err".to_string(),
            vec![auto_val::Value::Int(new_err_val)],
        );
        let new_id = vm.insert_heap_object(new_instance);
        vm.rc_push_id(task, new_id as u64); // Plan 419
    } else {
        // Ok — pass through unchanged
        drop(guard);
        task.ram.push_i32(result_id as i32);
    }
    Ok(())
}

// ============================================================================
// Iterator Native Shims
// ============================================================================

thread_local! {
    /// PLAN-696: only the top-level SSE iterator is budgeted. Nested iterator
    /// calls made by the generator body retain their ordinary synchronous
    /// semantics.
    static PLAN696_SSE_NEXT_BUDGET: std::cell::Cell<Option<(u32, u32)>> = const { std::cell::Cell::new(None) };
}

/// Run one SSE iterator step cooperatively. `false` means the generator used
/// its instruction budget or entered a cooperative sleep and should be polled
/// again after yielding to the LocalSet.
/// PLAN-707 T-05：generator next() 驱动体（自 [`shim_iterator_next`] 的
/// Generator 臂抽出）——调用方先 clone 状态再调本函数，**不持 iterators
/// 锁驱动**（内层嵌套 iterator_next 同 shard get_mut 重入会自死锁），
/// 返回后由调用方回写状态。
fn generator_next_drive(
    task: &mut AutoTask,
    vm: &AutoVM,
    iterator_id: u32,
    gen_state: &mut crate::vm::engine::GeneratorState,
) {
    //
    // Each next() runs the generator task forward from its saved
    // ip/bp/sp until the NEXT YIELD_VAL (or RET/Terminated). The
    // task stays alive across next() calls (vm.tasks is not cleaned
    // by run_task_loop, and we manage the task lifecycle manually).
    //
    // This replaces the previous eager approach (which ran the whole
    // body on first next(), collecting all yields). Lazy enables:
    //  - infinite generators (fib) that the consumer can break from
    //  - SSE incremental flush (each yield pushes one frame)
    //  - side effects in the generator interleaved with consumption
    //
    // Plan 326 §1 fix preserved: done pushes the -1 nil sentinel
    // so the for-loop's `DUP; CONST -1; EQ` exits cleanly.
    // cooperative 判定原在 shim 顶层（PLAN696 tl 目标比对）——随体迁入。
    let cooperative_budget = PLAN696_SSE_NEXT_BUDGET.with(|slot| {
        slot.get()
            .filter(|(target_iterator, _)| *target_iterator == iterator_id)
            .map(|(_, budget)| budget)
    });
    let cooperative = cooperative_budget.is_some();
    use crate::vm::engine::StepResult;
    use crate::vm::task::TaskStatus;

    if gen_state.done {
        task.ram.push_i32(-1);
        return;
    }

    // First next(): spawn the generator task and set up its frame
    // (mirrors the CALL opcode's frame setup).
    if !gen_state.started {
        gen_state.started = true;
        let tid = vm.spawn_task(gen_state.func_addr as usize, 65536);
        gen_state.task_id = Some(tid);

        if let Some(gen_task_arc) = vm.tasks.get(&tid).map(|entry| entry.clone()) {
            if let Ok(mut gt) = gen_task_arc.try_lock() {
                gt.cooperative_http_sleep = cooperative;
                // Plan 417-D2: seed the CALL-transferred args
                // BELOW the frame markers, mirroring the CALL
                // convention [args..., ret_addr, old_bp] so the
                // generator's FN_PROLOG and RET see a well-formed
                // frame (previously parameterized generators read
                // garbage params and underflowed at RET).
                let seed = std::mem::take(&mut gen_state.stack_snapshot);
                for nv in seed {
                    gt.ram.push_nv(nv);
                }
                gt.ram.push_i32(0); // return address (unused)
                gt.ram.push_i32(0); // old BP
                let new_bp = gt.ram.sp - 1;
                gt.bp = new_bp;
                gt.current_fn_n_args = gen_state.n_args as usize;
                gt.call_stack.push(crate::vm::task::CallFrame {
                    return_ip: 0,
                    old_bp: new_bp,
                    fn_name: None,
                    line: 0,
                    old_fn_n_args: 0,
                    old_fn_n_locals: 0,
                });
                gt.ip = gen_state.func_addr as usize;
            }
        }
    }

    let tid = match gen_state.task_id {
        Some(id) => id,
        None => {
            // Task was already cleaned up (generator finished).
            gen_state.done = true;
            task.ram.push_i32(-1);
            return;
        }
    };

    // A generator can be created before its first HTTP pull; set
    // the mode on every pull so sleep dispatch stays scoped to
    // this cooperative SSE consumer.
    if let Some(gen_task_arc) = vm.tasks.get(&tid).map(|entry| entry.clone()) {
        if let Ok(mut gt) = gen_task_arc.try_lock() {
            gt.cooperative_http_sleep = cooperative;
        }
    }

    // Run the generator task forward until the next YIELD_VAL or
    // completion. Budget caps a single next() (prevents runaway
    // loops with no yields); the task resumes from its saved ip.
    const NEXT_BUDGET: u32 = 1_000_000;
    let instruction_budget = cooperative_budget.unwrap_or(NEXT_BUDGET);
    let mut yielded_nv: Option<u64> = None;
    let mut finished = false;
    let mut pending = false;

    // Instructions can start nested generators/tasks. Keep task ownership,
    // but release the registry shard guard before executing any instruction.
    if let Some(gen_task_arc) = vm.tasks.get(&tid).map(|entry| entry.clone()) {
        if let Ok(mut gt) = gen_task_arc.try_lock() {
            for _ in 0..instruction_budget {
                match vm.run_one_instruction(&mut gt) {
                    Ok(StepResult::Continue) => continue,
                    Ok(StepResult::GeneratorYield) => {
                        // Value is on the generator task's stack.
                        // PEEK (don't pop): codegen emits a POP after
                        // YIELD_VAL (ExprStmt discards yield's nil
                        // return value), so we must leave the yielded
                        // value on the generator stack for that POP to
                        // consume. We copy it to the caller instead.
                        let nv = gt.ram.raw_nv[gt.ram.sp - 1];
                        yielded_nv = Some(nv);
                        // Park the task so run_task_loop won't pick
                        // it up between next() calls. Its ip/bp/sp
                        // are preserved on the task itself.
                        gt.status = TaskStatus::Waiting("generator_suspended".into());
                        break;
                    }
                    Ok(StepResult::Terminated) => {
                        finished = true;
                        break;
                    }
                    Ok(StepResult::Yield) => {
                        // PLAN-707 T-05（D-6）：cooperative sleep 与
                        // 外部流等待都必须立即停步——流等待无 wake_time，
                        // 旧形态落 1M 步预算内同 shim 反复重试（热循环）。
                        if cooperative
                            && (gt.wake_time.is_some()
                                || gt.waiting_sse_stream_id.is_some()
                                || gt.waiting_http_stream_id.is_some())
                        {
                            pending = true;
                            break;
                        }
                    }
                    Ok(StepResult::AwaitFuture {
                        future_id,
                        body_offset,
                    }) => {
                        // PLAN-707 T-05（§5.4）：外部 `~{}` await 在
                        // generator 体内同样停步（Pending）——镜像
                        // drive_handler_segment 的 AwaitFuture 处理；
                        // 旧形态直接 continue 跳过体执行并热转。
                        if let Err(e) = vm.handle_await_future(&mut gt, future_id, body_offset) {
                            eprintln!("[Generator] await body error: {:?}", e);
                            finished = true;
                            break;
                        }
                        if gt.waiting_future_id.is_some() {
                            pending = true;
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("[Generator] Error: {:?}", e);
                        finished = true;
                        break;
                    }
                }
            }
            // Budget exhausted without yield or termination: treat
            // as done to avoid an infinite external loop.
            if yielded_nv.is_none() && !finished && !pending {
                if cooperative {
                    pending = true;
                } else {
                    eprintln!("[Generator] next() budget exhausted without yield");
                    finished = true;
                }
            }
        }
    }

    if pending {
        return;
    }

    if finished {
        // Generator returned/terminated: clean up the task.
        vm.remove_task(tid);
        gen_state.task_id = None;
        gen_state.done = true;
        task.ram.push_i32(-1);
        return;
    }

    // Push the yielded value onto the caller's stack.
    match yielded_nv {
        Some(nv) => {
            if auto_val::is_i32(nv) {
                task.ram.push_i32(auto_val::decode_i32(nv));
            } else {
                task.ram.push_nv(nv);
            }
        }
        None => {
            // No yield and not finished (shouldn't happen, but be safe).
            gen_state.done = true;
            task.ram.push_i32(-1);
        }
    }
    return;
}

/// Get next element from iterator.
/// Stack: iterator_id -> element (or -1 for nil)
/// Returns: element value, or -1 if exhausted
// Plan 077 Phase 6: Updated to use unified registry
/// PLAN-776 R7-02：iterator 消费面**凭据感知**弹栈——TAG_OBJECT Sse 凭据
///（generator 生产点全部产此形态）解包底层 iterator id；裸 i32 保持原
/// 语义。返回 (iterator_id, 源是否凭据)。
pub fn pop_iterator_receiver(task: &mut AutoTask, vm: &AutoVM) -> (u32, bool) {
    // PLAN-776 R8-02/R9-01：消费协议——
    // ① 清零弹出槽（残留 TAG_OBJECT 会被 R2-04 任务栈全槽扫描误读成
    //    有效持有 → 双释放/泄漏两向失真）；
    // ② 对**原始凭据 nv**：**StakeGuard 即那一次消费**（其 Drop 执行
    //    rc_release_id——一个输入槽恰一份；R8 误加显式 rc_release 致
    //    RC2 共享引用 2→0 双释放——F776-R9-01 反例：全局份额被销）。
    //    读取期间份额未动（release 只发生在 Drop）；错误早退由 Drop
    //    覆盖同样恰一份。
    let nv = task.ram.pop_nv();
    task.ram.raw_nv[task.ram.sp] = 0;
    if auto_val::is_object(nv) {
        let stake = StakeGuard::nv(vm, nv);
        let resolved = crate::vm::ffi::http_return_resource::resolve_resource(vm, nv);
        drop(stake);
        if let Some(crate::vm::ffi::http_return_resource::ResourceClaim::Sse { iterator_id }) =
            resolved
        {
            return (iterator_id, true);
        }
    }
    (auto_val::decode_i32(nv) as u32, false)
}

/// PLAN-776 R7-02：派生 iterator 身份——源为凭据时派生 id 同样包装 Sse
/// 凭据（身份传播一致，下游 next/terminal 继续可解）；裸源保持裸 id。
pub fn push_derived_iterator(
    task: &mut AutoTask,
    vm: &AutoVM,
    id: u32,
    from_credential: bool,
) -> Result<(), VMError> {
    if from_credential {
        crate::vm::ffi::http_return_resource::push_sse_resource(task, vm, id)
    } else {
        task.ram.push_i32(id as i32);
        Ok(())
    }
}

// ============================================================================
// Terminal Operations
// ============================================================================

// ============================================================================
// HashMap Shims (Plan 086)
// ============================================================================

// ============================================================================
// Plan 046 (auto-musk T2): dynamic-receiver (`obj`) method family.
// musk front handlers historically could not link `.find(pred)` /
// `Object.values(x)` / corrected `Object.keys(x)` on dynamically-typed values,
// forcing hand-scanned loops (auto-musk PLAN-045 VM-clean workarounds, see
// PLAN-046 T4 inventory R1-R4). Runtime shapes for dynamic objects:
//   - object literals -> heap ObjectData (Plan 390 §15 H3b)
//   - typed instances -> GenericInstanceData
// Both are collected here; unsupported shapes error LOUDLY instead of the
// previous silent empty-value degradation observed in PLAN-046 T2 baseline.

fn dyn_object_entries(vm: &AutoVM, obj_id: u64) -> Result<Vec<(String, auto_val::Value)>, VMError> {
    let obj = vm.get_heap_object(obj_id).ok_or_else(|| {
        VMError::RuntimeError(format!(
            "obj method family: invalid heap object id {}",
            obj_id
        ))
    })?;
    let guard = obj.read().unwrap();
    if let Some(od) = guard
        .as_any()
        .downcast_ref::<crate::vm::types::ObjectData>()
    {
        let mut entries: Vec<(String, auto_val::Value)> = od
            .fields
            .iter()
            .filter_map(|(k, v)| k.name().map(|n| (n.to_string(), v.clone())))
            .collect();
        // Deterministic ordering (HashMap storage is unordered) so find/sum
        // consumers behave reproducibly run-to-run.
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        return Ok(entries);
    }
    if let Some(inst) = guard
        .as_any()
        .downcast_ref::<crate::vm::generic_registry::GenericInstanceData>()
    {
        return Ok(inst
            .field_names
            .iter()
            .cloned()
            .zip(inst.fields.iter().cloned())
            .collect());
    }
    Err(VMError::RuntimeError(format!(
        "obj method family: heap object {} is not a dynamic object",
        obj_id
    )))
}

fn push_value_list(task: &mut AutoTask, vm: &AutoVM, items: Vec<auto_val::Value>) {
    let mut list: crate::vm::types::ListData<auto_val::Value> = crate::vm::types::ListData::new();
    for item in items {
        list.push(item);
    }
    let list_id = vm.insert_heap_object(list);
    vm.rc_push_id(task, list_id as u64); // Plan 419
}

// ============================================================================
// HashSet Shims (Plan 118 Phase 3)
// ============================================================================

// ============================================================================
// StringBuilder Shims (Plan 118 Phase 3)
// ============================================================================

// ============================================================================
// VecDeque Shims (Plan 118 Phase 3)
// ============================================================================

// ============================================================================
// BTreeMap Shims (Plan 118 Phase 3)
// ============================================================================

/// Helper: decode a NanoValue to a String (from string pool)
fn nv_to_string(nv: auto_val::NanoValue, vm: &AutoVM) -> String {
    if auto_val::is_string(nv) {
        vm.get_string(auto_val::decode_string(nv))
            .map(|b| String::from_utf8_lossy(&b[..]).to_string())
            .unwrap_or_default()
    } else {
        String::new()
    }
}

// ============================================================================
// Mutable String Shims (177-186)
// ============================================================================

// ============================================================================
// Memory Allocation Shims
// ============================================================================

// ============================================================================
// Storage Shims (Heap, InlineInt64)
// ============================================================================

/// Heap.new() -> instance_id
/// Create a new Heap storage instance using a ListData<i32> internally.
pub fn shim_heap_new(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;

    // Create an empty list to represent the Heap storage
    let list: ListData<i32> = ListData::new();
    let id = vm.insert_heap_object(list);
    vm.rc_push_id(task, id as u64); // Plan 419
    Ok(())
}

/// heap.capacity() -> capacity
/// Get Heap storage capacity.
/// Stack: [instance_id] -> capacity
pub fn shim_heap_capacity(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;

    let inst_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_inst_id = crate::vm::native::StakeGuard::new(vm, inst_id as i64 as u64);
    if let Some(obj) = vm.get_heap_object(inst_id) {
        let guard = obj.read().unwrap();
        if let Some(list) = guard.as_any().downcast_ref::<ListData<i32>>() {
            task.ram.push_i32(list.elems.capacity() as i32);
            return Ok(());
        }
    }
    task.ram.push_i32(0);
    Ok(())
}

/// heap.try_grow(min_cap) -> bool
/// Try to grow Heap storage to at least min_cap elements.
/// Stack: [min_cap, instance_id] -> bool (1/0)
pub fn shim_heap_try_grow(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;

    let min_cap = crate::vm::native::pop_arg_i32(task) as usize;

    let _stake_min_cap = crate::vm::native::StakeGuard::new(vm, min_cap as i64 as u64);
    let inst_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_inst_id = crate::vm::native::StakeGuard::new(vm, inst_id as i64 as u64);

    if let Some(obj) = vm.get_heap_object(inst_id) {
        let mut guard = obj.write().unwrap();
        if let Some(list) = guard.as_any_mut().downcast_mut::<ListData<i32>>() {
            let cap = list.elems.capacity();
            let new_cap = if cap == 0 {
                std::cmp::max(8, min_cap)
            } else {
                std::cmp::max(cap * 2, min_cap)
            };
            list.elems.resize(new_cap, 0);
            drop(guard);
            task.ram.push_nv(auto_val::encode_bool(true)); // success
            return Ok(());
        }
    }
    task.ram.push_nv(auto_val::encode_bool(false));
    Ok(())
}

/// heap.drop() -> nil
/// Free Heap storage.
/// Stack: [instance_id] -> nil
pub fn shim_heap_drop(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // Plan 419: 语义更新 —— rc 释放(RC 归零才真回收;单持有时即手动提前释放)。
    let id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_id = crate::vm::native::StakeGuard::new(vm, id as i64 as u64);
    vm.rc_release_id(id);
    Ok(())
}

// ============================================================================
// List Extra Shims
// ============================================================================

// ============================================================================
// Plan 212 Phase 2: Rand Native Shims
// ============================================================================

/// Simple xorshift64 PRNG — no external rand crate needed
#[derive(Debug)]
struct Xorshift64 {
    state: u64,
}

impl Xorshift64 {
    fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 {
                0xDEAD_BEEF_CAFE_BABE
            } else {
                seed
            },
        }
    }

    fn next(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }
}

// ============================================================================
// Plan 212 Phase 2.2: Regex Opaque Struct Shims
// ============================================================================

/// Helper: pop a string from the VM stack
fn pop_vm_string(task: &mut AutoTask, vm: &AutoVM) -> String {
    {
        let nv = crate::vm::native::pop_arg_nv(task);

        let _stake_nv = crate::vm::native::StakeGuard::nv(vm, nv);
        if auto_val::is_string(nv) {
            let idx = auto_val::decode_string(nv);
            vm.get_string(idx)
                .map(|b| String::from_utf8_lossy(&b).into_owned())
                .unwrap_or_default()
        } else {
            let val = auto_val::decode_i32(nv);
            format!("{}", val)
        }
    }
}

/// Helper: push a string onto the VM stack via string pool
fn push_vm_string(task: &mut AutoTask, vm: &AutoVM, s: &str) {
    let idx = vm.add_string(s.as_bytes().to_vec()) as u32;
    vm.rc_push_str_idx(task, idx as usize);
}

// ============================================================================
// Plan 212 Phase 2.2: Url Opaque Struct Shims
// ============================================================================

// ============================================================================
// Plan 212 Phase 2.2: Semver Opaque Struct Shims
// ============================================================================

// ============================================================================
// Plan 212 Phase 2.3: Chrono Opaque Struct Shims
// ============================================================================

// ============================================================================
// Plan 212 Phase 2.3: Base64 Pure Function Shims
// ============================================================================

// ============================================================================
// Plan 212 Phase 2.3: Hex Pure Function Shims
// ============================================================================

// ============================================================================
// Plan 212 Phase 2.3: Sha2 Opaque Struct Shims
// ============================================================================

/// Sha256::new() → opaque handle
/// Stack: [] -> [handle_i32]
pub fn shim_sha2_sha256_new(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;
    use sha2::Digest;

    let hasher = sha2::Sha256::new();
    let obj = RustStdlibObject::new("sha2::Sha256", std::sync::Mutex::new(hasher));
    let id = vm.insert_heap_object(obj);
    vm.rc_push_id(task, id as u64); // Plan 419
    Ok(())
}

/// hasher.update(data) → void
/// Stack: [data_str, handle_i32] -> [0]
pub fn shim_sha2_update(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;
    use sha2::Digest;

    let data = pop_vm_string(task, vm);
    let hasher_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_hasher_id = crate::vm::native::StakeGuard::new(vm, hasher_id as i64 as u64);

    if let Some(obj) = vm.get_heap_object(hasher_id) {
        let mut guard = obj.write().unwrap();
        if let Some(rso) = guard.as_any_mut().downcast_mut::<RustStdlibObject>() {
            if let Some(hasher) = rso.downcast_mut::<std::sync::Mutex<sha2::Sha256>>() {
                hasher.get_mut().unwrap().update(data.as_bytes());
            }
        }
    }
    task.ram.push_i32(0);
    Ok(())
}

/// hasher.finalize() → hex string
/// Note: finalize() takes ownership of the hasher, so we clone the hasher state
/// to compute the hash while keeping the original available for further updates.
/// Stack: [handle_i32] -> [hex_str]
pub fn shim_sha2_finalize(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;
    use sha2::Digest;

    let hasher_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_hasher_id = crate::vm::native::StakeGuard::new(vm, hasher_id as i64 as u64);

    if let Some(obj) = vm.get_heap_object(hasher_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(hasher) = rso.downcast_ref::<std::sync::Mutex<sha2::Sha256>>() {
                // Clone the hasher to compute without consuming
                let cloned = hasher.lock().unwrap().clone();
                let result = cloned.finalize();
                let hex_str = hex::encode(result);
                drop(guard);
                push_vm_string(task, vm, &hex_str);
                return Ok(());
            }
        }
    }
    push_vm_string(task, vm, "");
    Ok(())
}

/// hasher.drop() — no-op, GC handles cleanup
pub fn shim_sha2_drop(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // Plan 419: 语义更新 —— rc 释放(RC 归零才真回收;单持有时即手动提前释放)。
    let id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_id = crate::vm::native::StakeGuard::new(vm, id as i64 as u64);
    vm.rc_release_id(id);
    Ok(())
}

// ============================================================================
// Plan 212 Phase 2.3: Mime Guess Pure Function Shim
// ============================================================================

// Plan 240: Instant opaque shims for std::time::Instant

// ============================================================================
// Plan 442 A5: one-shot scheduler primitives (set_timeout / clear_timeout)
// ============================================================================

// ============================================================================
// Plan 442 B-support: web-platform globals on the VM render target
// (musk corpus runs single-source in both render modes)
// ============================================================================

// Plan 442 Phase B: `dom.*` / `location.reload()` web globals for the VM
// target. The iced renderer is dark-fixed, so prefers_dark reports true and
// set_dark/set_css_var are recorded no-ops; focus/click/reload are desktop
// stubs; open_url really opens the OS browser. copy_text is routed by the
// codegen to auto.clipboard.set_text (Plan 418) and has no shim here.

/// PLAN-095 T-04: `ui.focus(target_key)` 请求槽（进程级，renderer update
/// 轮询消费）。native 无 bridge 状态写面——经全局槽递交，命中/缺失结果由
/// renderer 落 `__focus_result` 状态（"ok" / "miss:<key>"）可观察。
pub static UI_FOCUS_REQUEST: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

/// renderer 侧取走一次聚焦请求（take 语义，防重复消费）。
pub fn take_ui_focus_request() -> Option<String> {
    UI_FOCUS_REQUEST
        .lock()
        .ok()
        .and_then(|mut slot| slot.take())
}

// Plan 240: File I/O opaque shims

/// Helper to pop a string from the stack
fn pop_string(task: &mut AutoTask, vm: &AutoVM) -> String {
    {
        let nv = crate::vm::native::pop_arg_nv(task);

        let _stake_nv = crate::vm::native::StakeGuard::nv(vm, nv);
        if auto_val::is_string(nv) {
            let idx = auto_val::decode_string(nv) as usize;
            vm.strings
                .read()
                .unwrap()
                .get(idx)
                .cloned()
                .map(|b| String::from_utf8_lossy(&b).to_string())
                .unwrap_or_default()
        } else {
            auto_val::decode_i32(nv).to_string()
        }
    }
}

// Plan 240: OnceCell opaque shims using RustStdlibObject wrapping Option<String>

// ============================================================================
// Plan 250: Stdlib Enhancement — New Native Shims
// ============================================================================

// --- Result shims (2760-2766) ---
// Result<T,E> is represented as a tagged i32:
//   Ok(v)  => v >= 0 (the value itself)
//   Err(e) => e < 0  (negated error code)

/// Check if a Result is Ok.
/// Stack: result_val (i32) -> bool
pub fn shim_result_is_ok(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let val = crate::vm::native::pop_arg_i32(task);

    let _stake_val = crate::vm::native::StakeGuard::new(vm, val as i64 as u64);
    task.ram.push_nv(auto_val::encode_bool(val >= 0));
    Ok(())
}

/// Check if a Result is Err.
/// Stack: result_val (i32) -> bool
pub fn shim_result_is_err(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let val = crate::vm::native::pop_arg_i32(task);

    let _stake_val = crate::vm::native::StakeGuard::new(vm, val as i64 as u64);
    task.ram.push_nv(auto_val::encode_bool(val < 0));
    Ok(())
}

/// Unwrap Result — returns value if Ok, panics if Err.
/// Stack: result_val (i32) -> value (i32)
pub fn shim_result_unwrap(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let val = crate::vm::native::pop_arg_i32(task);

    let _stake_val = crate::vm::native::StakeGuard::new(vm, val as i64 as u64);
    if val < 0 {
        return Err(VMError::RuntimeError("unwrap on Err result".to_string()));
    }
    task.ram.push_i32(val);
    Ok(())
}

/// Unwrap Result with default.
/// Stack: default (i32), result_val (i32) -> value (i32)
pub fn shim_result_unwrap_or(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let default = crate::vm::native::pop_arg_i32(task);

    let _stake_default = crate::vm::native::StakeGuard::new(vm, default as i64 as u64);
    let val = crate::vm::native::pop_arg_i32(task);

    let _stake_val = crate::vm::native::StakeGuard::new(vm, val as i64 as u64);
    task.ram.push_i32(if val >= 0 { val } else { default });
    Ok(())
}

/// Unwrap the Err value from a Result.
/// Stack: result_val (i32) -> err_value (i32)
pub fn shim_result_unwrap_err(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let val = crate::vm::native::pop_arg_i32(task);

    let _stake_val = crate::vm::native::StakeGuard::new(vm, val as i64 as u64);
    if val >= 0 {
        return Err(VMError::RuntimeError("unwrap_err on Ok result".to_string()));
    }
    task.ram.push_i32(val);
    Ok(())
}

/// Create Ok result — passthrough.
/// Stack: value (i32) -> result (i32)
pub fn shim_result_ok(_task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    Ok(())
}

/// Create Err result — negate to mark as error.
/// Stack: value (i32) -> result (i32)
pub fn shim_result_err(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let val = crate::vm::native::pop_arg_i32(task);

    let _stake_val = crate::vm::native::StakeGuard::new(vm, val as i64 as u64);
    task.ram.push_i32(if val >= 0 { -val - 1 } else { val });
    Ok(())
}

// --- List Reverse (2770) ---

// --- Random convenience (2780-2783) ---

// --- DateTime extended (2790-2792) ---
// These use the existing chrono_opaque infrastructure from stdlib.rs

// --- CSV (2800-2803) ---
// CSV functions return JSON strings, matching the pattern used by file.walk etc.

// --- Hashing (2810-2813) ---

// --- Test assertions (2820-2825) ---

// --- Format (2830-2832) ---

// --- FS extended (2840-2847) ---

// ============================================================================
// Plan 419: 消费型 pop(死区结算的补盲)
//
// CALL_NAT 的死区结算只覆盖净减栈的 shim;"pop N → push N"(消费 receiver
// 返回新值)的 sp 中性形态不会进死区,receiver 的 stake 就此泄漏。所有
// 消费型(读后即弃)的 pop 走这里:引用值 -1 + 槽位清零(防死区二次释放)。
// 存储型 pop(值进容器/邮箱)不走这里 —— 容器侧 retain 配平。
// ============================================================================

/// 消费型参数 pop:弹值 + 槽位清零(不计数 —— 计数由 StakeGuard 在
/// shim 返回时统一释放,保证"先读取后释放"次序)。槽位清零令死区结算
/// 对这些槽为 no-op,避免双重释放。
pub fn pop_arg_i32(task: &mut AutoTask) -> i32 {
    let v = task.ram.pop_i32();
    task.ram.raw_nv[task.ram.sp] = 0;
    v
}

pub fn pop_arg_nv(task: &mut AutoTask) -> auto_val::NanoValue {
    let v = task.ram.pop_nv();
    task.ram.raw_nv[task.ram.sp] = 0;
    v
}

/// Plan 419: shim 参数的 stake 守卫 —— Drop 时释放(覆盖早退 return;
/// 读取对象必须发生在释放之前,故不在 pop 点释放)。id < HEAP_ID_BASE
/// 时堆侧 Drop 为 no-op(迭代器/闭包/任务 id 安全)。
/// Plan 510 G3:池串实参同守卫——此前只释放堆引用,池份额(pop_arg_nv
/// 弹出的 TAG_STRING)从不配平,每个池串实参泄漏 1 份(soak 实测
/// print(acc) 残 1)。Drop 时点在内容读取之后,安全。
pub struct StakeGuard<'a> {
    vm: &'a AutoVM,
    id: u64,
    pool_idx: Option<u32>,
}

impl<'a> StakeGuard<'a> {
    pub fn new(vm: &'a AutoVM, id: u64) -> Self {
        Self {
            vm,
            id,
            pool_idx: None,
        }
    }
    pub fn nv(vm: &'a AutoVM, nv: auto_val::NanoValue) -> Self {
        Self {
            vm,
            id: crate::vm::rc::heap_ref_id(nv).unwrap_or(0),
            pool_idx: crate::vm::rc::pool_idx_nv(nv),
        }
    }
}

impl Drop for StakeGuard<'_> {
    fn drop(&mut self) {
        if self.id >= crate::vm::rc::HEAP_ID_BASE {
            self.vm.rc_release_id(self.id);
        }
        if let Some(idx) = self.pool_idx {
            self.vm.pool_release(idx as usize);
        }
    }
}

// ============================================================================
// Plan 419: 引用计数诊断 natives(rc_stats 的脚本侧断言通道)
// ============================================================================

/// Simple recursive directory walker.
fn walkdir_recursive(dir: &str) -> Result<Vec<String>, std::io::Error> {
    let mut result = Vec::new();
    let entries = std::fs::read_dir(dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let sub = walkdir_recursive(&path.to_string_lossy())?;
            result.extend(sub);
        } else {
            result.push(path.to_string_lossy().to_string());
        }
    }
    Ok(result)
}

// --- FS more (2860-2865) ---

/// Recursive directory walker returning ALL paths (files + dirs).
fn walkdir_all(dir: &str) -> Result<Vec<String>, std::io::Error> {
    let mut result = Vec::new();
    let entries = std::fs::read_dir(dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let path_str = path.to_string_lossy().to_string();
        result.push(path_str.clone());
        if path.is_dir() {
            let sub = walkdir_all(&path.to_string_lossy())?;
            result.extend(sub);
        }
    }
    Ok(result)
}

// --- FS tree (2875) — Plan 626 T-04 ---

/// Skip-list for `fs.tree`: VCS/build/dependency noise + dotfiles.
fn fs_tree_skipped(name: &str) -> bool {
    matches!(
        name,
        ".git" | "target" | "build" | "node_modules" | "gen" | "dist" | "__pycache__"
    ) || name.starts_with('.')
}

/// Recursive tree builder emitting nested JSON (no trailing separators).
/// `depth` counts levels BELOW `dir` still to descend (0 = emit nothing).
pub fn fs_tree_walk(dir: &std::path::Path, root: &std::path::Path, depth: usize, out: &mut String) {
    let mut entries: Vec<std::fs::DirEntry> = match std::fs::read_dir(dir) {
        Ok(rd) => rd.filter_map(|e| e.ok()).collect(),
        Err(_) => return,
    };
    // Dirs first, then case-insensitive name order — stable tree shape.
    entries.sort_by_key(|e| {
        let is_dir = e.path().is_dir();
        (!is_dir, e.file_name().to_string_lossy().to_lowercase())
    });
    for entry in entries {
        let name = entry.file_name().to_string_lossy().to_string();
        if fs_tree_skipped(&name) {
            continue;
        }
        let path = entry.path();
        let is_dir = path.is_dir();
        // id = '/'-separated path relative to root — the TreeView node id
        // convention (apps rejoin it onto the workspace root to read files).
        let rel = path
            .strip_prefix(root)
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_else(|_| name.clone());
        let id = serde_json::to_string(&rel).unwrap_or_else(|_| "\"\"".into());
        let label = serde_json::to_string(&name).unwrap_or_else(|_| "\"\"".into());
        if !out.is_empty() {
            out.push(',');
        }
        if is_dir && depth > 1 {
            let mut children = String::new();
            fs_tree_walk(&path, root, depth - 1, &mut children);
            out.push_str(&format!(
                "{{\"id\":{},\"label\":{},\"children\":[{}],\"kind\":\"dir\",\"icon\":\"folder\",\"is_leaf\":false,\"badge\":\"\"}}",
                id, label, children
            ));
        } else if is_dir {
            out.push_str(&format!(
                "{{\"id\":{},\"label\":{},\"children\":[],\"kind\":\"dir\",\"icon\":\"folder\",\"is_leaf\":false,\"badge\":\"\"}}",
                id, label
            ));
        } else {
            out.push_str(&format!(
                "{{\"id\":{},\"label\":{},\"children\":[],\"kind\":\"file\",\"icon\":\"file-text\",\"is_leaf\":true,\"badge\":\"\"}}",
                id, label
            ));
        }
    }
}

fn copy_recursive(src: &str, dst: &str) -> Result<(), std::io::Error> {
    let src_path = std::path::Path::new(src);
    if src_path.is_dir() {
        let dst_path = std::path::Path::new(dst);
        std::fs::create_dir_all(dst_path)?;
        for entry in std::fs::read_dir(src_path)? {
            let entry = entry?;
            let src_child = entry.path();
            let dst_child = dst_path.join(entry.file_name());
            copy_recursive(&src_child.to_string_lossy(), &dst_child.to_string_lossy())?;
        }
    } else {
        std::fs::copy(src, dst)?;
    }
    Ok(())
}

// --- Hash extended (2814-2816) ---

// --- Random type (2870-2874) ---

// --- Fmt (2752) ---

// --- Cmp (2880) ---

// --- DateTime cmp (2794) ---

fn get_datetime_timestamp(vm: &AutoVM, handle: u64) -> i64 {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;
    if let Some(obj) = vm.get_heap_object(handle) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(dt) = rso.downcast_ref::<std::sync::Mutex<chrono::NaiveDateTime>>() {
                return dt.lock().unwrap().and_utc().timestamp();
            }
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Plan 626 T-04: fs.tree nested JSON builder ─────────────────────
    #[test]
    fn plan626_fs_tree_nested_json_schema_and_order() {
        let tmp = std::env::temp_dir().join(format!("auto_lang_fs_tree_{}", std::process::id()));
        let src = tmp.join("ws");
        let _ = std::fs::remove_dir_all(&src);
        std::fs::create_dir_all(src.join("b_dir")).unwrap();
        std::fs::create_dir_all(src.join("a_dir").join("nested")).unwrap();
        std::fs::write(src.join("z.txt"), "z").unwrap();
        std::fs::write(src.join("a_dir").join("f.at"), "fn main() {}").unwrap();
        std::fs::write(src.join("a_dir").join("nested").join("deep.md"), "# d").unwrap();
        std::fs::create_dir_all(src.join(".git")).unwrap();
        std::fs::write(src.join(".git").join("HEAD"), "ref").unwrap();
        std::fs::create_dir_all(src.join("target")).unwrap();

        let mut buf = String::new();
        fs_tree_walk(&src, &src, 3, &mut buf);
        let json = format!("[{}]", buf);
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");

        // Dirs first, then case-insensitive name order: a_dir, b_dir, z.txt.
        let items = parsed.as_array().expect("root array");
        assert_eq!(items.len(), 3, "dotfiles/build noise skipped: {json}");
        assert_eq!(items[0]["label"], "a_dir");
        assert_eq!(items[1]["label"], "b_dir");
        assert_eq!(items[2]["label"], "z.txt");
        // Node schema fields (TreeView direct-fit).
        assert_eq!(items[0]["kind"], "dir");
        assert_eq!(items[0]["icon"], "folder");
        assert_eq!(items[0]["is_leaf"], false);
        assert_eq!(items[2]["kind"], "file");
        assert_eq!(items[2]["icon"], "file-text");
        assert_eq!(items[2]["is_leaf"], true);
        // ids are '/'-separated paths relative to root.
        assert_eq!(items[0]["id"], "a_dir");
        assert_eq!(items[2]["id"], "z.txt");
        // Nested children carry relative ids too.
        let a_children = items[0]["children"].as_array().unwrap();
        assert_eq!(a_children.len(), 2, "f.at + nested: {json}");
        assert_eq!(a_children[0]["id"], "a_dir/nested");
        assert_eq!(a_children[1]["id"], "a_dir/f.at");
        let deep = a_children[0]["children"].as_array().unwrap();
        assert_eq!(deep[0]["id"], "a_dir/nested/deep.md");

        // max_depth caps recursion: depth 1 = root entries only.
        let mut shallow = String::new();
        fs_tree_walk(&src, &src, 1, &mut shallow);
        let parsed_shallow: serde_json::Value =
            serde_json::from_str(&format!("[{}]", shallow)).unwrap();
        let dir0 = &parsed_shallow.as_array().unwrap()[0];
        assert_eq!(dir0["label"], "a_dir");
        assert!(
            dir0["children"].as_array().unwrap().is_empty(),
            "depth-1 dirs stay collapsed"
        );
        let _ = std::fs::remove_dir_all(&src);
    }

    #[test]
    fn test_resolve_direct_lookup() {
        let mut ni = NativeInterface::new();
        ni.register_name("Result.map_err", 2070);
        assert_eq!(ni.resolve("Result.map_err"), Some(2070));
        assert_eq!(ni.resolve("nonexistent.method"), None);
    }

    #[test]
    fn test_resolve_canonical_normalization() {
        let mut ni = NativeInterface::new();
        ni.register_name("auto.list.push", 101);
        ni.register_name("auto.hashmap.insert", 120);
        ni.register_name("auto.list.join", 2080);

        assert_eq!(ni.resolve("List.push"), Some(101));
        assert_eq!(ni.resolve("List.join"), Some(2080));
        assert_eq!(ni.resolve("HashMap.insert"), Some(120));
        assert_eq!(ni.resolve("Map.insert"), Some(120));
    }

    #[test]
    fn test_resolve_no_canonical_for_qualified() {
        let mut ni = NativeInterface::new();
        ni.register_name("auto.list.push", 101);

        assert_eq!(ni.resolve("auto.list.push"), Some(101));
        assert_eq!(ni.resolve("rust.something"), None);
    }

    #[test]
    fn register_shim_routes_dynamic_band_ids_without_panicking() {
        let mut ni = NativeInterface::new();
        ni.register_shim_for_id(DYNAMIC_ID_START, |_, _| Ok(()));

        assert!(ni.get(DYNAMIC_ID_START).is_some());
        assert_eq!(ni.next_dynamic_id(), DYNAMIC_ID_START + 1);
    }

    // ── Plan 413: code editor payload natives (UI bridge) ──────────
    // PLAN-814 T-02 迁移上移：测试组消费上层 ui::code_editor 面，已迁
    // auto-lang src/tests/plan413_code_editor_natives.rs（原 SDK 死门
    // 勘误后上移，V-05）。

    // ── PLAN-656 review F-3: scroll controller natives 全管线复现 ──────
    // PLAN-814 T-02 迁移上移：本测试消费上层 UI 面（ui::scroll），已迁
    // auto-lang src/tests/plan656_scroll_controller_pipeline.rs（原 `ui`
    // feature 门在 SDK 无定义——死门、选择丢失，V-05 勘误）。

    // ── Plan 413 follow-up: console natives (in-app Console panel) ──────
    #[test]
    fn vm_console_natives_end_to_end() {
        crate::vm::ui_console::ui_console_clear();
        // Script-level native calls through the full Codegen → VM → shim
        // pipeline; plain `print()` must mirror into the ring buffer too.
        let (_result, out) = crate::run_with_capture(
            r#"
print("from-print")
let ok = console_log("from-console-log")
print(ok)
let panel = console_lines()
print(panel)
let n = 2
let m = "edit #" + n.str()
print(m)
"#,
        )
        .unwrap();
        // Newest-first: the console_log line precedes the mirrored print.
        let snapshot = crate::vm::ui_console::ui_console_lines(10);
        let log_pos = snapshot.find("from-console-log").expect("console_log line");
        let print_pos = snapshot.find("from-print").expect("mirrored print line");
        assert!(log_pos < print_pos, "newest first: {snapshot}");
        // console_lines() delivered the snapshot back into the script.
        assert!(out.contains("from-console-log"), "panel roundtrip: {out}");
        // int→str + concat composes with the natives (041 logs use it —
        // f-strings do NOT interpolate in VM expressions, verified above).
        assert!(out.contains("edit #2"), "concat: {out}");
        // clear() empties the ring buffer (other tests may print in parallel,
        // so assert on our markers rather than strict emptiness).
        crate::run_with_capture("let c = console_clear()").unwrap();
        let after = crate::vm::ui_console::ui_console_lines(50);
        assert!(!after.contains("from-console-log"), "cleared: {after}");
        assert!(!after.contains("from-print"), "cleared: {after}");
    }

    #[test]
    fn test_to_canonical() {
        assert_eq!(
            NativeInterface::to_canonical("List.push"),
            Some("auto.list.push".to_string())
        );
        assert_eq!(
            NativeInterface::to_canonical("HashMap.get"),
            Some("auto.hashmap.get".to_string())
        );
        assert_eq!(
            NativeInterface::to_canonical("Map.new"),
            Some("auto.hashmap.new".to_string())
        );
        assert_eq!(
            NativeInterface::to_canonical("Array.len"),
            Some("auto.list.len".to_string())
        );
        assert_eq!(NativeInterface::to_canonical("nopart"), None);
    }
}

#[cfg(test)]
mod plan738_contract_identity_tests {
    use super::NativeInterface;

    /// PLAN-738 §5.3：既有生产能力面的显式契约在 production() 工厂后可查
    /// （inventory 覆盖序不吞声明）；签名核对依赖这些契约在场。
    #[test]
    fn declared_contract_identities_resolve() {
        let mut interface = NativeInterface::production();
        // 复刻 engine::new 的 merge 路径，验证契约在 CFFI 合并后仍在位。
        let cffi = crate::vm::codegen::CFFI_GLOBAL.lock().unwrap();
        interface.merge(cffi.native_interface());
        let faces: &[(&str, &[&str], &str)] = &[
            ("auto.http.get", &["str"], "Response"),
            ("auto.http.get_stream", &["str"], "HTTPStream"),
            ("auto.http.request", &["str", "str"], "RequestBuilder"),
            (
                "auto.http.transfer_download",
                &["str", "str", "str"],
                "FileTransfer",
            ),
            (
                "auto.http.file_response",
                &["str", "str", "str"],
                "FileResponse",
            ),
            ("auto.http.upload_error", &["int", "str"], "UploadReceipt"),
            ("auto.json.is_valid", &["str"], "int"),
            ("auto.json.encode", &["T"], "str"),
            ("auto.json.decode", &["str"], "T"),
        ];
        for (name, parameters, returns) in faces {
            let id = crate::vm::native_registry::BIGVM_NATIVES
                .lock()
                .unwrap()
                .resolve_qualified(name)
                .unwrap_or_else(|| panic!("{name} unresolved"));
            let contract = interface
                .contract(id)
                .unwrap_or_else(|| panic!("{name} (#{id}) lacks contract"));
            assert_eq!(contract.parameters, *parameters, "{name}");
            assert_eq!(contract.returns, *returns, "{name}");
        }
        assert_eq!(
            interface
                .contract(
                    crate::vm::native_registry::BIGVM_NATIVES
                        .lock()
                        .unwrap()
                        .resolve_qualified("auto.json.encode")
                        .unwrap()
                )
                .unwrap()
                .generics,
            ["T"]
        );
    }
}

// ---- PLAN-843 结构拆分：native 实现按域分族（原单文件热点，纯搬移） ----
mod natives_chrono;
pub use natives_chrono::*;
mod natives_collections;
pub use natives_collections::*;
mod natives_formats;
pub use natives_formats::*;
mod natives_fs;
pub use natives_fs::*;
mod natives_host;
pub use natives_host::*;
mod natives_int;
pub use natives_int::*;
mod natives_io;
pub use natives_io::*;
mod natives_rand;
pub use natives_rand::*;
mod natives_strings;
pub use natives_strings::*;
mod natives_sys;
pub use natives_sys::*;
mod natives_test;
pub use natives_test::*;
mod natives_web;
pub use natives_web::*;
