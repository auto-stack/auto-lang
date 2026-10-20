//! PLAN-804: OS 文件对话框宿主实现（rfd，同步 API）——`dialog_open`/
//! `dialog_save`（natives 2927/2928）的单一 OS 真源。
//!
//! 迁移自两处重复构建：`vm/native.rs` 原 shim 内联实现（parented——
//! Plan 428 债修的父窗锚定）与 `ui/dialog.rs` a2r host（unparented——
//! PLAN-681 §9 F1 缝）。公共算法（过滤解析、默认文件名、取消空串约定）
//! 只在本模块维护一份；两调用方仅选择不同父窗策略：
//! - VM 默认 adapter（[`ParentedOsDialogProvider`]）：Windows 下锚定主窗
//!   （[`dialog_parent`]，自 vm/native.rs 原样迁入）；
//! - a2r 高层门面（[`unparented_open`]/[`unparented_save`]，`ui::dialog`
//!   薄委托）：无父窗（旧 a2r 策略诚实保留——模态父窗对 a2r shell 是
//!   独立接线关注点，非语义分叉）。
//!
//! 门控：ui-dialog 单门控（rfd + raw-window-handle 依赖面），**独立于
//! ui renderer 门**——ui-dialog-only（无 ui）构建恢复迁移前"高层对话
//! 框路径"（经 lib.rs 统一安装入口装配兼容默认）。
//!
//! 两者返回 "" on cancel/unavailable（headless CI included）。

use auto_lang_core::vm::host_capabilities::DialogHostProvider;
use std::sync::Arc;

/// 过滤解析（T-01 冻结语义，自 vm/native.rs 与 ui/dialog.rs 两处重复
/// 收敛为单一实现）：`,. ;`（逗号/分号/空格/tab）分割 → trim → 去前导
/// 点 → 弃空项。空表 = 不加 filter（不过滤）。
fn filter_exts(filter: &str) -> Vec<String> {
    filter
        .split([',', ';', ' ', '\t'])
        .map(|s| s.trim().trim_start_matches('.').to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Plan 428 debt fix: parent the blocking dialogs to the app's main
/// window. The dialogs previously had NO owner — they could open BEHIND
/// the app window ("frozen" UX) or, destroyed abnormally, leak the owner
/// disabled forever (428 acceptance finding, KNOWN-DEBT-AND-RISKS 428).
/// The HWND is discovered via Win32 (largest visible top-level window of
/// this process — no iced plumbing needed) and cached for the process
/// lifetime. Non-Windows targets keep the plain dialog.
///（PLAN-804 自 vm/native.rs 原样迁入——旧 HWND 进程期缓存策略不在本
/// 计划重写范围，见计划 §1 非目标。）
#[cfg(target_os = "windows")]
mod dialog_parent {
    use raw_window_handle::{
        DisplayHandle, HasDisplayHandle, HasWindowHandle, RawWindowHandle, Win32WindowHandle,
        WindowHandle,
    };
    use std::cell::Cell;
    use std::num::NonZeroIsize;
    use std::sync::OnceLock;

    #[repr(C)]
    struct Rect {
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    }

    #[link(name = "user32")]
    extern "system" {
        fn EnumWindows(cb: extern "system" fn(isize, isize) -> i32, lparam: isize) -> i32;
        fn IsWindowVisible(hwnd: isize) -> i32;
        fn GetWindowThreadProcessId(hwnd: isize, pid: *mut u32) -> u32;
        fn GetClassNameW(hwnd: isize, class: *mut u16, max: i32) -> i32;
        fn GetWindowRect(hwnd: isize, rect: *mut Rect) -> i32;
    }

    thread_local! {
        /// (hwnd, area) of the best main-window candidate so far.
        static BEST: Cell<(isize, i64)> = const { Cell::new((0, -1)) };
    }

    extern "system" fn enum_cb(hwnd: isize, _lparam: isize) -> i32 {
        // Skip invisible helpers (GPU/IME) and non-app windows.
        if unsafe { IsWindowVisible(hwnd) } == 0 {
            return 1;
        }
        let mut pid = 0u32;
        unsafe { GetWindowThreadProcessId(hwnd, &mut pid) };
        if pid != std::process::id() {
            return 1;
        }
        let mut buf = [0u16; 64];
        let n = unsafe { GetClassNameW(hwnd, buf.as_mut_ptr(), buf.len() as i32) }.max(0) as usize;
        let class = String::from_utf16_lossy(&buf[..n]);
        // #32770 = Win32 dialog (a dialog can't own our dialog); the
        // winit event target is a message-only-ish shim, not the window.
        if class == "#32770" || class == "Winit Thread Event Target" {
            return 1;
        }
        let mut r = Rect {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        if unsafe { GetWindowRect(hwnd, &mut r) } == 0 {
            return 1;
        }
        let area = (r.right - r.left).max(0) as i64 * (r.bottom - r.top).max(0) as i64;
        BEST.with(|b| {
            if area > b.get().1 {
                b.set((hwnd, area));
            }
        });
        1
    }

    fn main_hwnd() -> Option<NonZeroIsize> {
        static HWND: OnceLock<Option<NonZeroIsize>> = OnceLock::new();
        *HWND.get_or_init(|| {
            BEST.with(|b| b.set((0, -1)));
            unsafe { EnumWindows(enum_cb, 0) };
            let (hwnd, area) = BEST.with(|b| b.get());
            (area > 0).then(|| NonZeroIsize::new(hwnd)).flatten()
        })
    }

    /// rfd's `set_parent` bound: a Win32 HWND + the trivial Windows
    /// display handle.
    struct Owner(NonZeroIsize);

    impl HasWindowHandle for Owner {
        fn window_handle(&self) -> Result<WindowHandle<'_>, raw_window_handle::HandleError> {
            let raw = RawWindowHandle::Win32(Win32WindowHandle::new(self.0));
            // SAFETY: the HWND came from EnumWindows on a live window and
            // is cached for the process lifetime.
            Ok(unsafe { WindowHandle::borrow_raw(raw) })
        }
    }

    impl HasDisplayHandle for Owner {
        fn display_handle(&self) -> Result<DisplayHandle<'_>, raw_window_handle::HandleError> {
            Ok(DisplayHandle::windows())
        }
    }

    /// Attach `dialog` to the main window when one was found.
    pub fn attach(dialog: rfd::FileDialog) -> rfd::FileDialog {
        match main_hwnd() {
            Some(hwnd) => dialog.set_parent(&Owner(hwnd)),
            None => dialog,
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        /// The handle wrappers must satisfy rfd's `set_parent` bound and
        /// construct without error (the HWND value itself is opaque).
        ///（PLAN-804 自 vm/native.rs 原样迁入。）
        #[test]
        fn owner_satisfies_rfd_parent_bound() {
            let owner = Owner(NonZeroIsize::new(1).unwrap());
            assert!(owner.window_handle().is_ok());
            assert!(owner.display_handle().is_ok());
            // set_parent takes `&W: HasWindowHandle + HasDisplayHandle`.
            fn bound<W: HasWindowHandle + HasDisplayHandle>(_: &W) {}
            bound(&owner);
        }
    }
}

#[cfg(target_os = "windows")]
use dialog_parent::attach;
#[cfg(not(target_os = "windows"))]
fn attach(dialog: rfd::FileDialog) -> rfd::FileDialog {
    dialog
}

/// 单一 OS open 实现（策略仅父窗一项）：filter 解析 → 非空加 filter →
/// 按策略锚定父窗 → pick_file → `to_string_lossy`（取消/不可用 = ""）。
fn os_open(attach_parent: bool, filter: &str) -> String {
    let exts = filter_exts(filter);
    let mut dialog = rfd::FileDialog::new();
    if !exts.is_empty() {
        dialog = dialog.add_filter("Files", &exts);
    }
    if attach_parent {
        dialog = attach(dialog);
    }
    dialog
        .pick_file()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default()
}

/// 单一 OS save 实现：default_name 非空才 set_file_name → 按策略锚定
/// 父窗 → save_file → `to_string_lossy`（取消 = ""）。
fn os_save(attach_parent: bool, default_name: &str) -> String {
    let mut dialog = rfd::FileDialog::new();
    if !default_name.is_empty() {
        dialog = dialog.set_file_name(default_name);
    }
    if attach_parent {
        dialog = attach(dialog);
    }
    dialog
        .save_file()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default()
}

/// VM 默认 dialog host provider：**parented** 策略（迁移前 vm/native.rs
/// shim 内联实现的逐条语义——Windows 下锚定主窗，Plan 428 债修保持）。
pub struct ParentedOsDialogProvider;

impl DialogHostProvider for ParentedOsDialogProvider {
    fn open(&self, filter: &str) -> String {
        os_open(true, filter)
    }

    fn save(&self, default_name: &str) -> String {
        os_save(true, default_name)
    }
}

/// a2r 高层 open 门面（**unparented** 策略，旧 `ui::dialog::open` 语义——
/// PLAN-681 §9 F1 缝诚实保留）。`ui::dialog::open` 薄委托本函数。
pub fn unparented_open(filter: &str) -> String {
    os_open(false, filter)
}

/// a2r 高层 save 门面（**unparented** 策略，旧 `ui::dialog::save` 语义）。
pub fn unparented_save(default_name: &str) -> String {
    os_save(false, default_name)
}

/// 装配 UI 兼容默认 dialog host provider = parented OS adapter
///（**仅当进程默认槽为空时**安装——`set_..._if_absent` 同锁原子；优先
/// 关系与 779/785/790/794 逐条相同）：
/// 1. 调用方**显式**注册（`set_process_default_dialog_host_provider`）
///    恒胜——本兼容安装不覆盖；
/// 2. 空槽时补缺（ui-dialog 构建下内建宿主入口恢复迁移前对话框接线，
///    含 ui-dialog-only 无 ui 形态）；
/// 3. 实例级 `AutoVM::set_dialog_host_provider` 在 1/2 之上按 VM 独立
///    生效（构造期快照 + 实例覆盖，不受后续进程默认变化影响）。
pub fn install_process_default() {
    auto_lang_core::vm::host_capabilities::set_process_default_dialog_host_provider_if_absent(
        Arc::new(ParentedOsDialogProvider),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// F779-R2-01 优先关系镜像锚（779/785/790/794 同款纪律）：兼容安装
    /// 不覆盖显式默认；空槽才补缺；显式 set 恒可替换。Fake provider——
    /// 不弹真 OS 对话框（真模态证据 = V04 env 门控实机档）。F-785-R1-01：
    /// 本测试 clear/set/restore 写默认槽——须持 crate 级共享隔离锁
    /// （test_default_slot_lock），与缺席断言/桥装配测试互斥。
    #[test]
    fn compat_install_never_overrides_explicit_default() {
        struct Fake(&'static str);
        impl DialogHostProvider for Fake {
            fn open(&self, _filter: &str) -> String {
                format!("{}:open", self.0)
            }
            fn save(&self, _default_name: &str) -> String {
                format!("{}:save", self.0)
            }
        }

        let _slot = auto_lang_core::vm::host_capabilities::test_default_slot_lock();
        let saved = auto_lang_core::vm::host_capabilities::process_default_dialog_host_provider();

        // 空槽 → 补缺成功（仅入槽，不弹 OS 对话框——ParentedOsDialogProvider
        // 惰性：不调用即零 OS 面）。
        auto_lang_core::vm::host_capabilities::clear_process_default_dialog_host_provider();
        install_process_default();
        assert!(
            auto_lang_core::vm::host_capabilities::process_default_dialog_host_provider().is_some(),
            "空槽时兼容默认须补缺"
        );

        // 显式默认 → 兼容安装不覆盖（哨兵可区分）。
        auto_lang_core::vm::host_capabilities::set_process_default_dialog_host_provider(Arc::new(
            Fake("@plan804-explicit:哨兵-🎉"),
        ));
        install_process_default();
        let cur =
            auto_lang_core::vm::host_capabilities::process_default_dialog_host_provider().unwrap();
        assert_eq!(
            DialogHostProvider::open(&*cur, "f"),
            "@plan804-explicit:哨兵-🎉:open",
            "兼容安装不得覆盖显式默认"
        );

        // 还原。
        match saved {
            Some(p) => {
                auto_lang_core::vm::host_capabilities::set_process_default_dialog_host_provider(p)
            }
            None => {
                auto_lang_core::vm::host_capabilities::clear_process_default_dialog_host_provider()
            }
        }
    }

    /// 过滤解析单一实现锚（T-01 冻结语义）：分割符全集、去前导点、弃空、
    /// 混合分隔、全空输入 = 无 filter。
    #[test]
    fn filter_exts_frozen_semantics() {
        assert_eq!(filter_exts(".at, .au"), vec!["at", "au"]);
        assert_eq!(filter_exts("at;au\tmd  png"), vec!["at", "au", "md", "png"]);
        assert_eq!(filter_exts(" . . ;;; ,,"), Vec::<String>::new());
        assert_eq!(filter_exts(""), Vec::<String>::new());
        // 前导多点只剥一层点前缀的等效效果（trim_start_matches 全剥点）。
        assert_eq!(filter_exts("..hidden"), vec!["hidden"]);
        // Unicode 过滤词不受影响（非 ASCII 扩展名原样保留）。
        assert_eq!(filter_exts("文件型🎉"), vec!["文件型🎉"]);
    }
}
