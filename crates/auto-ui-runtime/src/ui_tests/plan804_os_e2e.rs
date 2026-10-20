//! PLAN-804 V-04/V-05：真 OS 实机证据（env 门控——不进日常档）。
//!
//! 门（缺 env 即 skip，日常 `cargo t` 零 OS 面零交互）：
//! - `AUTO_LANG_DIALOG_E2E=1`：真模态文件对话框（rfd/IFileDialog）实机
//!   往返——open 选真实 Unicode 临时文件、save 选目标并验证默认名种子、
//!   两者取消=空串；parented/unparented 父窗策略经 dialog owner 实锚；
//!   模态后 owner 窗可用性复原；VM 链（shim+provider）与高层入口
//!   （run_with_capture，ui-dialog-only 形态同跑）。
//! - `AUTO_LANG_DND_E2E=1`：真 DoDragDrop 会话（**非直调 COM 构造器**）——
//!   VM shim→真 provider→start_drag 在测试线程（=输入线程）内联阻塞，
//!   驱动线程 SendInput 合成按下-移动-释放，拖出 text/files/virtual_files
//!   至自建测试接收窗（RegisterDragDrop+自有消费探针），逐项断言接收
//!   内容/落地 bytes/完成效果通道/调用线程 ID/返回在会话结束之后；再跑
//!   ESC 取消会话证会话串行与拖旗标复原。
//!
//! ⚠ 实机注意：本档会短暂抢占输入（移动光标/弹真对话框/置顶接收窗）——
//! 只在授权的证据收集时段运行。对话框自动化用原始 user32 extern
//! （ui-dialog-only 构建无 windows crate 依赖也可编译）；拖拽档在
//! native-dnd feature 下用 windows crate COM。
//!
//! 运行（work 树实跑收据用，`--test-threads=1` 防多测抢输入）：
//! - dialog：`AUTO_LANG_DIALOG_E2E=1 cargo test -p auto-lang --lib plan804_os_e2e -- --test-threads=1`
//!   ui-dialog-only 形态：`AUTO_LANG_DIALOG_E2E=1 cargo test -p auto-lang --no-default-features --features ui-dialog --lib plan804_os_e2e -- --test-threads=1`
//! - dnd：`AUTO_LANG_DND_E2E=1 cargo test -p auto-lang --features native-dnd --lib plan804_os_e2e -- --test-threads=1`

#![cfg(windows)]

/// V-04：真模态文件对话框（cfg(ui-dialog)；windows 由外层 mod 门控）。
#[cfg(feature = "ui-dialog")]
mod dialog_e2e {
    use auto_lang_core::vm::host_capabilities::DialogHostProvider as _;
    use std::sync::mpsc;

    // ── 原始 user32/kernel32 extern（ui-dialog-only 构建无 windows crate
    //    依赖——与 host_adapters::dialog::dialog_parent 同款纪律）─────────
    #[repr(C)]
    struct WndClassW {
        style: u32,
        wndproc: Option<unsafe extern "system" fn(isize, u32, usize, isize) -> isize>,
        cls_extra: i32,
        wnd_extra: i32,
        inst: isize,
        icon: isize,
        cursor: isize,
        background: isize,
        menu_name: *const u16,
        class_name: *const u16,
    }

    #[repr(C)]
    struct Msg {
        hwnd: isize,
        message: u32,
        wparam: usize,
        lparam: isize,
        time: u32,
        pt_x: i32,
        pt_y: i32,
    }

    #[link(name = "user32")]
    extern "system" {
        fn RegisterClassW(wc: *const WndClassW) -> u16;
        fn CreateWindowExW(
            ex: u32,
            class: *const u16,
            name: *const u16,
            style: u32,
            x: i32,
            y: i32,
            w: i32,
            h: i32,
            parent: isize,
            menu: isize,
            inst: isize,
            param: *mut core::ffi::c_void,
        ) -> isize;
        fn DestroyWindow(hwnd: isize) -> i32;
        fn DefWindowProcW(hwnd: isize, msg: u32, wp: usize, lp: isize) -> isize;
        fn ShowWindow(hwnd: isize, cmd: i32) -> i32;
        fn IsWindowEnabled(hwnd: isize) -> i32;
        fn IsWindowVisible(hwnd: isize) -> i32;
        fn EnumWindows(cb: extern "system" fn(isize, isize) -> i32, lparam: isize) -> i32;
        fn GetWindowThreadProcessId(hwnd: isize, pid: *mut u32) -> u32;
        fn GetClassNameW(hwnd: isize, buf: *mut u16, max: i32) -> i32;
        fn SetWindowTextW(hwnd: isize, text: *const u16) -> i32;
        fn GetWindowTextW(hwnd: isize, buf: *mut u16, max: i32) -> i32;
        fn GetWindowTextLengthW(hwnd: isize) -> i32;
        fn PostMessageW(hwnd: isize, msg: u32, wp: usize, lp: isize) -> i32;
        fn GetWindow(hwnd: isize, cmd: u32) -> isize;
        fn EnumChildWindows(
            parent: isize,
            cb: extern "system" fn(isize, isize) -> i32,
            lparam: isize,
        ) -> i32;
        fn IsWindow(hwnd: isize) -> i32;
        fn PeekMessageW(msg: *mut Msg, hwnd: isize, min: u32, max: u32, remove: u32) -> i32;
        fn TranslateMessage(msg: *const Msg) -> i32;
        fn DispatchMessageW(msg: *const Msg) -> isize;
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn GetCurrentThreadId() -> u32;
    }

    unsafe extern "system" fn test_wndproc(hwnd: isize, msg: u32, wp: usize, lp: isize) -> isize {
        unsafe { DefWindowProcW(hwnd, msg, wp, lp) }
    }

    const WM_CLOSE: u32 = 0x0010;
    const BM_CLICK: u32 = 0x00F5;
    const GW_OWNER: u32 = 4;
    const SW_SHOWNORMAL: i32 = 1;
    const PM_REMOVE: u32 = 1;

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    fn class_of(hwnd: isize) -> String {
        let mut buf = [0u16; 64];
        let n = unsafe { GetClassNameW(hwnd, buf.as_mut_ptr(), buf.len() as i32) }.max(0) as usize;
        String::from_utf16_lossy(&buf[..n])
    }

    fn window_text(hwnd: isize) -> String {
        let len = unsafe { GetWindowTextLengthW(hwnd) }.max(0) as usize;
        let mut buf = vec![0u16; len + 1];
        let n = unsafe { GetWindowTextW(hwnd, buf.as_mut_ptr(), (len + 1) as i32) }.max(0) as usize;
        String::from_utf16_lossy(&buf[..n])
    }

    /// 时限内非阻塞泵（PeekMessage——GetMessage 会无限阻塞，破坏时限）。
    fn pump_for(d: std::time::Duration) {
        let deadline = std::time::Instant::now() + d;
        while std::time::Instant::now() < deadline {
            let mut msg = Msg {
                hwnd: 0,
                message: 0,
                wparam: 0,
                lparam: 0,
                time: 0,
                pt_x: 0,
                pt_y: 0,
            };
            unsafe {
                while PeekMessageW(&mut msg, 0, 0, 0, PM_REMOVE) != 0 {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    /// 测试用真实顶层可见窗（dialog_parent 的父窗候选）。
    struct OwnWindow {
        hwnd: isize,
    }

    impl OwnWindow {
        fn create(title: &str) -> Self {
            static N: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
            let n = N.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let class_name = wide(&format!("auto_lang_plan804_e2e_{n}"));
            let wc = WndClassW {
                style: 0,
                wndproc: Some(test_wndproc),
                cls_extra: 0,
                wnd_extra: 0,
                inst: 0,
                icon: 0,
                cursor: 0,
                background: 0,
                menu_name: std::ptr::null(),
                class_name: class_name.as_ptr(),
            };
            let atom = unsafe { RegisterClassW(&wc) };
            assert_ne!(atom, 0, "RegisterClassW failed");
            let title_w = wide(title);
            let hwnd = unsafe {
                CreateWindowExW(
                    0,
                    class_name.as_ptr(),
                    title_w.as_ptr(),
                    0x00CF0000, // WS_OVERLAPPEDWINDOW
                    80,
                    80,
                    480,
                    320,
                    0,
                    0,
                    0,
                    std::ptr::null_mut(),
                )
            };
            assert_ne!(hwnd, 0, "CreateWindowExW failed");
            unsafe { ShowWindow(hwnd, SW_SHOWNORMAL) };
            // 泵一小段让窗真实呈现（可见态=dialog_parent 候选条件）。
            pump_for(std::time::Duration::from_millis(150));
            Self { hwnd }
        }
    }

    impl Drop for OwnWindow {
        fn drop(&mut self) {
            unsafe { DestroyWindow(self.hwnd) };
        }
    }

    /// 进程级共享父窗（**专属长寿命线程**持有——libtest 每测一线程，
    /// 线程退出即销毁其窗口；dialog_parent 的 HWND OnceLock 是进程期
    /// 缓存，父窗必须全进程存活且句柄有效，否则后续 IFileDialog::Show
    /// 对死 owner 秒败（实测 5ms 返 None）。线程只建窗+永泵消息，测试
    /// 进程退出时一并回收）。
    fn shared_own_window() -> isize {
        static WND: std::sync::OnceLock<isize> = std::sync::OnceLock::new();
        *WND.get_or_init(|| {
            let (tx, rx) = mpsc::channel::<isize>();
            std::thread::Builder::new()
                .name("plan804-e2e-owner".into())
                .spawn(move || {
                    // 线程持有 OwnWindow 不 drop——线程永不退出即窗口全进程
                    // 存活（与 dialog_parent 的 HWND OnceLock 同寿命）。
                    let w = OwnWindow::create("plan804-e2e 共享父窗🎉");
                    let _ = tx.send(w.hwnd);
                    // 永泵（进程期）：owner 窗响应绘制/启用态流转；loop
                    // 不返回 = w 永不 drop = 窗口与 OnceLock 缓存同寿命。
                    loop {
                        pump_for(std::time::Duration::from_secs(1));
                    }
                })
                .expect("spawn owner thread");
            rx.recv_timeout(std::time::Duration::from_secs(5))
                .expect("owner window hwnd")
        })
    }

    thread_local! {
        static CHILD_HITS: std::cell::RefCell<Vec<isize>> = std::cell::RefCell::new(Vec::new());
    }

    extern "system" fn child_enum_cb(hwnd: isize, _l: isize) -> i32 {
        CHILD_HITS.with(|h| h.borrow_mut().push(hwnd));
        1
    }

    /// 全后代枚举找窗口（class 相等 + 可选文案包含）。
    fn find_descendant(root: isize, want_class: &str, want_text: Option<&str>) -> Option<isize> {
        CHILD_HITS.with(|h| h.borrow_mut().clear());
        unsafe { EnumChildWindows(root, child_enum_cb, 0) };
        CHILD_HITS.with(|h| {
            h.borrow().iter().copied().find(|&w| {
                if class_of(w) != want_class {
                    return false;
                }
                match want_text {
                    Some(t) => window_text(w).contains(t),
                    None => true,
                }
            })
        })
    }

    fn is_window_alive(hwnd: isize) -> bool {
        unsafe { IsWindow(hwnd) != 0 }
    }

    /// 驱动参数：定位本进程（tid==0 时任意线程）的可见 #32770 模态 →
    /// 记录编辑框初始文本 → 设目标路径 → WM_COMMAND(IDOK) 接受；
    /// accept=false 时 WM_CLOSE 取消。
    struct DriveSpec {
        caller_thread: u32,
        set_path: Option<String>,
        accept: bool,
        /// accept 按钮文案匹配（"打开"/"保存"——IFileDialog 真实按钮）。
        accept_label: &'static str,
        timeout: std::time::Duration,
    }

    thread_local! {
        static FOUND: std::cell::Cell<Option<isize>> = const { std::cell::Cell::new(None) };
        static WANT_TID: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
    }

    extern "system" fn enum_cb(hwnd: isize, _l: isize) -> i32 {
        let mut pid = 0u32;
        let tid = unsafe { GetWindowThreadProcessId(hwnd, &mut pid) };
        if pid != std::process::id() {
            return 1;
        }
        let want = WANT_TID.with(|t| t.get());
        if want != 0 && tid != want {
            return 1;
        }
        // 只认真模态本体：#32770 类 + 可见 + enabled（禁用的是 owner）。
        if class_of(hwnd) == "#32770"
            && unsafe { IsWindowVisible(hwnd) } != 0
            && unsafe { IsWindowEnabled(hwnd) } != 0
        {
            FOUND.with(|f| f.set(Some(hwnd)));
        }
        1
    }

    /// 等待并驱动模态（驱动线程体）；ack 回 (对话框 hwnd, 编辑框初始文本)。
    fn drive_dialog(spec: DriveSpec, ack: mpsc::Sender<(isize, String, isize)>) {
        let deadline = std::time::Instant::now() + spec.timeout;
        WANT_TID.with(|t| t.set(spec.caller_thread));

        let mut dlg = 0isize;
        while std::time::Instant::now() < deadline {
            FOUND.with(|f| f.set(None));
            unsafe { EnumWindows(enum_cb, 0) };
            if let Some(h) = FOUND.with(|f| f.get()) {
                dlg = h;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(40));
        }
        // owner 须在对话框存活期读（销毁后 GW_OWNER 恒 0）。
        let live_owner = unsafe { GetWindow(dlg, GW_OWNER) };
        if dlg == 0 {
            thread_local! {
                static ALL: std::cell::RefCell<Vec<String>> = std::cell::RefCell::new(Vec::new());
            }
            extern "system" fn dump_cb(hwnd: isize, _l: isize) -> i32 {
                let mut pid = 0u32;
                let _ = unsafe { GetWindowThreadProcessId(hwnd, &mut pid) };
                if pid == std::process::id() {
                    ALL.with(|a| {
                        a.borrow_mut().push(format!(
                            "{} vis={} en={}",
                            class_of(hwnd),
                            unsafe { IsWindowVisible(hwnd) },
                            unsafe { IsWindowEnabled(hwnd) }
                        ))
                    });
                }
                1
            }
            ALL.with(|a| a.borrow_mut().clear());
            unsafe { EnumWindows(dump_cb, 0) };
            eprintln!(
                "[plan804-diag] timeout: own-process windows: {:?}",
                ALL.with(|a| a.borrow().clone())
            );
            let _ = ack.send((0, String::new(), 0));
            return;
        }

        // 文件名编辑框：IFileDialog 的 Edit 在 ComboBoxEx32→ComboBox 深处
        //（非直接子窗），经 EnumChildWindows 全后代枚举取首个 Edit 类。
        let edit = find_descendant(dlg, "Edit", None);
        let initial = edit.map(window_text).unwrap_or_default();

        if spec.accept {
            if let (Some(edit), Some(p)) = (edit, &spec.set_path) {
                let w = wide(p);
                unsafe { SetWindowTextW(edit, w.as_ptr()) };
            }
            std::thread::sleep(std::time::Duration::from_millis(120));
            // 真实 accept 按钮（"打开(&O)"/"保存(&S)"）BM_CLICK——现代
            // IFileDialog 不响应 WM_COMMAND IDOK。
            let btn = find_descendant(dlg, "Button", Some(spec.accept_label));
            if let Some(btn) = btn {
                unsafe { PostMessageW(btn, BM_CLICK, 0, 0) };
            } else {
                // accept 按钮未找到：WM_CLOSE 收场（测试以断言失败暴露）。
                unsafe { PostMessageW(dlg, WM_CLOSE, 0, 0) };
            }
            // 兜底：8s 后对话框仍在（点击未生效）则 WM_CLOSE 收场，测试
            // 以清晰断言失败替代挂死（300s 停滞不再可能）。
            let deadline2 = std::time::Instant::now() + std::time::Duration::from_secs(8);
            while std::time::Instant::now() < deadline2 {
                if !is_window_alive(dlg) {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            if is_window_alive(dlg) {
                unsafe { PostMessageW(dlg, WM_CLOSE, 0, 0) };
            }
        } else {
            std::thread::sleep(std::time::Duration::from_millis(80));
            unsafe { PostMessageW(dlg, WM_CLOSE, 0, 0) };
        }
        let _ = ack.send((dlg, initial, live_owner));
    }

    fn env_gate() -> bool {
        if std::env::var("AUTO_LANG_DIALOG_E2E").map_or(false, |v| v == "1") {
            true
        } else {
            eprintln!("skip: AUTO_LANG_DIALOG_E2E != 1（真模态实机档不进日常门）");
            false
        }
    }

    fn unicode_temp_dir() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "plan804-对话框-{}-🎉",
            std::process::id() as u64 ^ fastrand::u64(..)
        ));
        std::fs::create_dir_all(&dir).expect("temp dir");
        dir
    }

    /// 同串比较（Windows 路径大小写不敏感；dialog 返回反斜杠形）。
    fn same_path(a: &str, b: &str) -> bool {
        let norm = |s: &str| s.replace('/', "\\").to_lowercase();
        norm(a) == norm(b)
    }

    /// V-04(a) provider 级真模态：open 选真实 Unicode 临时文件（parented
    /// 策略）；save 种子默认名+选目标；open 取消=空串；模态后 owner 窗
    /// 可用（Plan 428 债修语义经迁移保持）。
    #[test]
    fn v04_2_parented_open_save_cancel_roundtrip() {
        if !env_gate() {
            return;
        }
        let dir = unicode_temp_dir();
        let pick_me = dir.join("选中-文件🎉.at");
        std::fs::write(&pick_me, b"plan804 open e2e").expect("write pick file");
        let save_target = dir.join("保存-目标β.md");

        let provider = crate::host_adapters::dialog::ParentedOsDialogProvider;
        let own_hwnd = shared_own_window();

        // ── open：接受 ──
        let (tx, rx) = mpsc::channel();
        let spec = DriveSpec {
            caller_thread: unsafe { GetCurrentThreadId() },
            set_path: Some(pick_me.to_string_lossy().into_owned()),
            accept: true,
            accept_label: "打开",
            timeout: std::time::Duration::from_secs(10),
        };
        let driver = std::thread::spawn(move || drive_dialog(spec, tx));
        let got_open = provider.open(" .at, .au ;md");
        let (dlg, _initial, _) = rx.recv().expect("driver ack");
        driver.join().unwrap();
        assert!(dlg != 0, "未找到 #32770 模态（自动化失败）");
        assert!(
            same_path(&got_open, &pick_me.to_string_lossy()),
            "open 返回所选路径（got {got_open:?}）"
        );

        // ── save：默认名种子 + 接受 ──
        let (tx2, rx2) = mpsc::channel();
        let spec = DriveSpec {
            caller_thread: unsafe { GetCurrentThreadId() },
            set_path: Some(save_target.to_string_lossy().into_owned()),
            accept: true,
            accept_label: "保存",
            timeout: std::time::Duration::from_secs(10),
        };
        let driver = std::thread::spawn(move || drive_dialog(spec, tx2));
        let got_save = provider.save("默认名-种子🎉.txt");
        let (dlg2, initial_edit, _) = rx2.recv().expect("driver ack 2");
        driver.join().unwrap();
        assert!(dlg2 != 0, "save 模态未找到");
        assert!(
            initial_edit.contains("默认名-种子🎉"),
            "save 编辑框须预置默认文件名（got {initial_edit:?}）"
        );
        assert!(
            same_path(&got_save, &save_target.to_string_lossy()),
            "save 返回所选目标（got {got_save:?}）"
        );

        // ── open：取消 = 空串 ──
        let (tx3, rx3) = mpsc::channel();
        let spec = DriveSpec {
            caller_thread: unsafe { GetCurrentThreadId() },
            set_path: None,
            accept: false,
            accept_label: "打开",
            timeout: std::time::Duration::from_secs(10),
        };
        let driver = std::thread::spawn(move || drive_dialog(spec, tx3));
        let got_cancel = provider.open("");
        let (dlg3, _, _) = rx3.recv().expect("driver ack 3");
        driver.join().unwrap();
        assert!(dlg3 != 0, "cancel 模态未找到");
        assert_eq!(got_cancel, "", "取消 = 空串（值语义非错误）");

        // ── save：取消 = 空串（合同 V04"两者取消"——真实 save 模态取消，
        // 非 fake 返回）──
        let (tx4, rx4) = mpsc::channel();
        let spec = DriveSpec {
            caller_thread: unsafe { GetCurrentThreadId() },
            set_path: None,
            accept: false,
            accept_label: "保存",
            timeout: std::time::Duration::from_secs(10),
        };
        let driver = std::thread::spawn(move || drive_dialog(spec, tx4));
        let got_save_cancel = provider.save("取消场景默认名🎉.txt");
        let (dlg4, initial4, _) = rx4.recv().expect("driver ack 4");
        driver.join().unwrap();
        assert!(dlg4 != 0, "save cancel 模态未找到");
        assert!(
            initial4.contains("取消场景默认名🎉"),
            "save 取消腿同样验证默认名种子（got {initial4:?}）"
        );
        assert_eq!(got_save_cancel, "", "save 取消 = 空串（值语义非错误）");

        // 模态后父窗可用（Plan 428：owner 不被遗留禁用）。
        assert!(
            unsafe { IsWindowEnabled(own_hwnd) } != 0,
            "模态结束后 owner 窗须可用"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// V-04(b) VM 链 + parented 策略实锚：shim_dialog_open 经实例 provider
    /// 在真模态上选文件；对话框 GetWindow(GW_OWNER) == 我方顶层窗。
    #[test]
    fn v04_1_vm_shim_and_dialog_owner_parented() {
        if !env_gate() {
            return;
        }
        use auto_lang_core::vm::engine::AutoVM;
        use auto_lang_core::vm::native::shim_dialog_open;
        use auto_lang_core::vm::task::AutoTask;
        use auto_lang_core::vm::virt_memory::VirtualFlash;

        let dir = unicode_temp_dir();
        let pick = dir.join("vm-链选中🎉.at");
        std::fs::write(&pick, b"plan804 vm e2e").expect("write pick");

        let vm = AutoVM::new_without_host_snapshot(VirtualFlash::new(0), 256);
        vm.set_dialog_host_provider(std::sync::Arc::new(
            crate::host_adapters::dialog::ParentedOsDialogProvider,
        ));
        let own_hwnd = shared_own_window();

        let (tx, rx) = mpsc::channel();
        let spec = DriveSpec {
            caller_thread: unsafe { GetCurrentThreadId() },
            set_path: Some(pick.to_string_lossy().into_owned()),
            accept: true,
            accept_label: "打开",
            timeout: std::time::Duration::from_secs(10),
        };
        let driver = std::thread::spawn(move || drive_dialog(spec, tx));
        let mut task = AutoTask::new(0, 256, 0);
        let arg_idx = vm.add_string("at".as_bytes().to_vec());
        vm.rc_push_str_idx(&mut task, arg_idx);
        shim_dialog_open(&mut task, &vm).expect("VM 链真模态成功");
        let (dlg, _, live_owner) = rx.recv().expect("driver ack");
        driver.join().unwrap();
        assert!(dlg != 0, "模态未找到");
        assert_eq!(
            live_owner, own_hwnd,
            "parented 策略：对话框 owner == 我方顶层窗（dialog_parent 迁移后语义）"
        );
        let nv = task.ram.pop_nv();
        assert!(auto_val::is_string(nv), "String lane");
        let got = vm
            .get_string(auto_val::decode_string(nv))
            .expect("字符串池可读");
        assert!(
            same_path(&String::from_utf8_lossy(&got), &pick.to_string_lossy()),
            "VM 链返回所选路径"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// V-04(c) a2r 旧路径（unparented）：经保留的 `ui::dialog` 门面开真模态，
    /// **存活期** owner == 0（F804-R1-01：销毁后读 GW_OWNER 恒 0 无法区分
    /// 策略——owner 断言一律用 driver 存活期采集值；与 v04_1 的
    /// owner==我方窗 正控成对，错误策略会被双向拒绝）。
    #[test]
    fn v04_3_unparented_a2r_path_has_no_owner() {
        if !env_gate() {
            return;
        }
        let dir = unicode_temp_dir();
        let pick = dir.join("a2r-无父窗🎉.at");
        std::fs::write(&pick, b"plan804 a2r e2e").expect("write pick");

        let _own_hwnd = shared_own_window(); // 顶层窗在位仍须 owner==0（策略差）
        let (tx, rx) = mpsc::channel();
        let spec = DriveSpec {
            caller_thread: unsafe { GetCurrentThreadId() },
            set_path: Some(pick.to_string_lossy().into_owned()),
            accept: true,
            accept_label: "打开",
            timeout: std::time::Duration::from_secs(10),
        };
        let driver = std::thread::spawn(move || drive_dialog(spec, tx));
        let got = crate::ui::dialog::open(" .at ;md\tpng");
        let (dlg, _, live_owner) = rx.recv().expect("driver ack");
        driver.join().unwrap();
        assert!(dlg != 0, "模态未找到");
        assert_eq!(
            live_owner, 0,
            "unparented（a2r 旧策略）：存活期对话框不得有 owner"
        );
        assert!(
            same_path(&got, &pick.to_string_lossy()),
            "a2r 路径返回所选路径（got {got:?}）"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// V-04(d) 高层入口（run_with_capture）：真程序 print(dialog_open(..))
    /// ——ui-dialog-only 形态亦跑本测（见文件头门禁命令）。
    #[test]
    fn v04_4_high_level_entry_real_dialog() {
        if !env_gate() {
            return;
        }
        let _own_hwnd = shared_own_window();
        let dir = unicode_temp_dir();
        let pick = dir.join("高层-入口🎉.at");
        std::fs::write(&pick, b"plan804 high-level e2e").expect("write pick");

        let (tx, rx) = mpsc::channel();
        // run_with_capture 的 VM 跑在其内部线程——驱动按 pid 全扫
        // （caller_thread=0 不过滤线程）。
        let spec = DriveSpec {
            caller_thread: 0,
            set_path: Some(pick.to_string_lossy().into_owned()),
            accept: true,
            accept_label: "打开",
            timeout: std::time::Duration::from_secs(15),
        };
        let driver = std::thread::spawn(move || drive_dialog(spec, tx));
        let program = "let p = dialog_open(\" .at, 文件🎉 \")\nprint(p)\n";
        let (result, stdout) = auto_lang_core::run_with_capture(program).expect("高层入口成功");
        let _ = result;
        let (dlg, _, _) = rx.recv().expect("driver ack");
        driver.join().unwrap();
        assert!(dlg != 0, "模态未找到");
        let line = stdout.lines().next().unwrap_or("");
        assert!(
            same_path(line, &pick.to_string_lossy()),
            "高层入口 stdout 为所选路径（got {stdout:?}）"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}

/// V-05：真 DoDragDrop 会话（cfg(native-dnd + ui)；windows crate 可用）。
#[cfg(all(feature = "native-dnd", feature = "ui"))]
mod dnd_e2e {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Mutex;

    use windows::core::implement;
    use windows::Win32::Foundation::{POINT, POINTL};
    use windows::Win32::System::Com::{IDataObject, DVASPECT_CONTENT, FORMATETC, TYMED_HGLOBAL};
    use windows::Win32::System::DataExchange::RegisterClipboardFormatW;
    use windows::Win32::System::Memory::{GlobalLock, GlobalSize, GlobalUnlock};
    use windows::Win32::System::Ole::{
        IDropTarget, IDropTarget_Impl, OleInitialize, RegisterDragDrop, RevokeDragDrop, DROPEFFECT,
        DROPEFFECT_COPY,
    };
    use windows::Win32::System::SystemServices::MODIFIERKEYS_FLAGS;
    use windows::Win32::System::Threading::GetCurrentThreadId;
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE, KEYBDINPUT, KEYBD_EVENT_FLAGS,
        KEYEVENTF_KEYUP, MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP,
        MOUSEEVENTF_MOVE, MOUSEINPUT, MOUSE_EVENT_FLAGS, VIRTUAL_KEY, VK_ESCAPE,
    };
    use windows::Win32::UI::Shell::{CFSTR_FILECONTENTS, CFSTR_FILEDESCRIPTORW};
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DestroyWindow, GetCursorPos, GetSystemMetrics,
        GetWindowRect, PeekMessageW, RegisterClassW, SetCursorPos, SetWindowPos, ShowWindow,
        HWND_TOPMOST, PM_REMOVE, SM_CXSCREEN, SM_CXVIRTUALSCREEN, SM_CYSCREEN, SM_CYVIRTUALSCREEN,
        SWP_NOMOVE, SWP_NOSIZE, SW_SHOWNORMAL, WINDOW_STYLE, WNDCLASSW,
    };

    const CF_UNICODETEXT_U16: u16 = 13;
    const CF_HDROP_U16: u16 = 15;
    /// FILEDESCRIPTORW on x64：4+16+8+8+4+8+8+8+4+4=72 头 + cFileName[260]
    /// 双字 = 592；名偏移 = 592-520 = 72。
    const FD_SIZE: usize = 592;
    const FD_NAME_OFF: usize = 72;

    unsafe extern "system" fn sink_wndproc(
        hwnd: windows::Win32::Foundation::HWND,
        msg: u32,
        wp: windows::Win32::Foundation::WPARAM,
        lp: windows::Win32::Foundation::LPARAM,
    ) -> windows::Win32::Foundation::LRESULT {
        unsafe { DefWindowProcW(hwnd, msg, wp, lp) }
    }

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    /// 接收侧记录：Drop 按格式族抽取（text / CF_HDROP / 虚拟文件描述符+
    /// FileContents——Explorer 同款消费路径），记录回调线程与会话轨迹。
    #[derive(Default, Clone)]
    struct DropRecord {
        enter_thread: Option<u32>,
        drop_thread: Option<u32>,
        text: Option<String>,
        files: Vec<String>,
        vfile_names: Vec<String>,
        vfile_bytes: Vec<Vec<u8>>,
    }

    static RECORD: Mutex<Option<DropRecord>> = Mutex::new(None);
    static DROP_DONE: AtomicBool = AtomicBool::new(false);

    fn hglobal_bytes(hg: windows::Win32::Foundation::HGLOBAL) -> Option<Vec<u8>> {
        unsafe {
            let size = GlobalSize(hg);
            if size == 0 {
                return None;
            }
            let ptr = GlobalLock(hg);
            if ptr.is_null() {
                return None;
            }
            let bytes = std::slice::from_raw_parts(ptr as *const u8, size).to_vec();
            let _ = GlobalUnlock(hg);
            Some(bytes)
        }
    }

    fn formatetc(cf: u16, lindex: i32) -> FORMATETC {
        FORMATETC {
            cfFormat: cf,
            ptd: std::ptr::null_mut(),
            dwAspect: DVASPECT_CONTENT.0 as u32,
            lindex,
            tymed: TYMED_HGLOBAL.0 as u32,
        }
    }

    fn get_data_bytes(data: &IDataObject, cf: u16, lindex: i32) -> Option<Vec<u8>> {
        let mut medium = unsafe { data.GetData(&formatetc(cf, lindex)) }.ok()?;
        let bytes = hglobal_bytes(unsafe { medium.u.hGlobal });
        unsafe { windows::Win32::System::Ole::ReleaseStgMedium(&mut medium) };
        bytes
    }

    /// 注册格式 ID（与生产源同名注册——同进程同值）。
    fn file_descriptor_cf() -> u16 {
        static CF: std::sync::OnceLock<u32> = std::sync::OnceLock::new();
        *CF.get_or_init(|| unsafe { RegisterClipboardFormatW(CFSTR_FILEDESCRIPTORW) }) as u16
    }

    fn file_contents_cf() -> u16 {
        static CF: std::sync::OnceLock<u32> = std::sync::OnceLock::new();
        *CF.get_or_init(|| unsafe { RegisterClipboardFormatW(CFSTR_FILECONTENTS) }) as u16
    }

    /// CF_HDROP（DROPFILES）路径表解析（接收侧消费路径）。
    fn parse_hdrop(bytes: &[u8]) -> Vec<String> {
        if bytes.len() < 20 {
            return Vec::new();
        }
        let p_files = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
        let wide_flag = u32::from_le_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]) != 0;
        if !wide_flag || p_files < 20 || p_files >= bytes.len() {
            return Vec::new();
        }
        let units: Vec<u16> = bytes[p_files..]
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        let mut out = Vec::new();
        let mut cur = Vec::new();
        for u in units {
            if u == 0 {
                if cur.is_empty() {
                    break;
                }
                out.push(String::from_utf16_lossy(&cur));
                cur.clear();
            } else {
                cur.push(u);
            }
        }
        out
    }

    /// FILEGROUPDESCRIPTORW：cItems + N × FILEDESCRIPTORW（取 cFileName）。
    fn parse_descriptor_names(bytes: &[u8]) -> Vec<String> {
        if bytes.len() < 4 {
            return Vec::new();
        }
        let n = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
        let mut out = Vec::new();
        for i in 0..n {
            let base = 4 + i * FD_SIZE;
            if base + FD_SIZE > bytes.len() {
                break;
            }
            let units: Vec<u16> = bytes[base + FD_NAME_OFF..base + FD_SIZE]
                .chunks_exact(2)
                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                .take_while(|&u| u != 0)
                .collect();
            out.push(String::from_utf16_lossy(&units));
        }
        out
    }

    /// 接收窗探针目标（COPY 语义；Drop 抽取三格式族）。
    #[implement(IDropTarget)]
    struct SinkDropTarget;

    #[allow(non_snake_case)]
    impl IDropTarget_Impl for SinkDropTarget_Impl {
        fn DragEnter(
            &self,
            _data: Option<&IDataObject>,
            _keys: MODIFIERKEYS_FLAGS,
            _pt: &POINTL,
            effect: *mut DROPEFFECT,
        ) -> windows::core::Result<()> {
            RECORD
                .lock()
                .unwrap()
                .get_or_insert_with(DropRecord::default)
                .enter_thread = Some(unsafe { GetCurrentThreadId() });
            unsafe { *effect = DROPEFFECT_COPY };
            Ok(())
        }

        fn DragOver(
            &self,
            _keys: MODIFIERKEYS_FLAGS,
            _pt: &POINTL,
            effect: *mut DROPEFFECT,
        ) -> windows::core::Result<()> {
            unsafe { *effect = DROPEFFECT_COPY };
            Ok(())
        }

        fn DragLeave(&self) -> windows::core::Result<()> {
            Ok(())
        }

        fn Drop(
            &self,
            data: Option<&IDataObject>,
            _keys: MODIFIERKEYS_FLAGS,
            _pt: &POINTL,
            effect: *mut DROPEFFECT,
        ) -> windows::core::Result<()> {
            unsafe { *effect = DROPEFFECT_COPY };
            let Some(data) = data else {
                return Ok(());
            };
            let mut rec = RECORD.lock().unwrap().take().unwrap_or_default();
            rec.drop_thread = Some(unsafe { GetCurrentThreadId() });
            if let Some(b) = get_data_bytes(data, CF_UNICODETEXT_U16, -1) {
                let units: Vec<u16> = b
                    .chunks_exact(2)
                    .map(|c| u16::from_le_bytes([c[0], c[1]]))
                    .collect();
                let s = String::from_utf16_lossy(&units);
                rec.text = Some(s.trim_end_matches('\0').to_string());
            }
            if let Some(b) = get_data_bytes(data, CF_HDROP_U16, -1) {
                rec.files = parse_hdrop(&b);
            }
            if let Some(desc) = get_data_bytes(data, file_descriptor_cf(), -1) {
                rec.vfile_names = parse_descriptor_names(&desc);
                for i in 0..rec.vfile_names.len() {
                    if let Some(b) = get_data_bytes(data, file_contents_cf(), i as i32) {
                        rec.vfile_bytes.push(b);
                    }
                }
            }
            *RECORD.lock().unwrap() = Some(rec);
            DROP_DONE.store(true, Ordering::SeqCst);
            Ok(())
        }
    }

    fn send_mouse(flags: MOUSE_EVENT_FLAGS, dx: i32, dy: i32) {
        let input = INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx,
                    dy,
                    mouseData: 0,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        let _ = unsafe { SendInput(&[input], std::mem::size_of::<INPUT>() as i32) };
    }

    fn send_key(vk: VIRTUAL_KEY, up: bool) {
        let input = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: vk,
                    wScan: 0,
                    dwFlags: if up {
                        KEYEVENTF_KEYUP
                    } else {
                        KEYBD_EVENT_FLAGS::default()
                    },
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        unsafe { SendInput(&[input], std::mem::size_of::<INPUT>() as i32) };
    }

    /// 屏幕坐标 → 绝对归一化（MOUSEEVENTF_ABSOLUTE 域）。
    fn to_absolute(x: i32, y: i32) -> (i32, i32) {
        unsafe {
            let w = (GetSystemMetrics(SM_CXSCREEN) as i64).max(1);
            let h = (GetSystemMetrics(SM_CYSCREEN) as i64).max(1);
            (
                (x as i64 * 65535 / (w - 1).max(1)) as i32,
                (y as i64 * 65535 / (h - 1).max(1)) as i32,
            )
        }
    }

    /// 驱动线程：等待 → 移至 (x,y) → 按下 → 拖动微移 → 释放（或 ESC 取消）。
    /// ：本会话结束旗标（测试侧 shim 返回后置位）——看门狗
    /// 突发随它停止，不误杀同测试的后续会话。
    fn drive_drag(x: i32, y: i32, cancel: bool, session_done: std::sync::Arc<AtomicBool>) {
        std::thread::sleep(std::time::Duration::from_millis(600));
        let (ax, ay) = to_absolute(x, y);
        send_mouse(MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE, ax, ay);
        std::thread::sleep(std::time::Duration::from_millis(80));
        send_mouse(MOUSEEVENTF_LEFTDOWN, 0, 0);
        std::thread::sleep(std::time::Duration::from_millis(80));
        for d in [6, -6, 6] {
            let (bx, by) = to_absolute(x + d, y + (d / 2));
            send_mouse(MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE, bx, by);
            std::thread::sleep(std::time::Duration::from_millis(40));
        }
        if cancel {
            send_key(VK_ESCAPE, false);
            std::thread::sleep(std::time::Duration::from_millis(30));
            send_key(VK_ESCAPE, true);
        }
        send_mouse(MOUSEEVENTF_LEFTUP, 0, 0);
        // 看门狗突发（panic 中断主测时注入态不残留系统）：会话旗标置位
        // （正常收场）即停；至多 20s。
        for _ in 0..20 {
            std::thread::sleep(std::time::Duration::from_secs(1));
            if session_done.load(Ordering::SeqCst) {
                break;
            }
            send_key(VK_ESCAPE, false);
            send_key(VK_ESCAPE, true);
            send_mouse(MOUSEEVENTF_LEFTUP, 0, 0);
        }
    }

    struct SinkWindow {
        hwnd: windows::Win32::Foundation::HWND,
    }

    impl SinkWindow {
        fn create() -> Self {
            let class_name = wide("auto_lang_plan804_dnd_sink");
            let hmodule = unsafe {
                windows::Win32::System::LibraryLoader::GetModuleHandleW(None).expect("hmodule")
            };
            let hinst: windows::Win32::Foundation::HINSTANCE = hmodule.into();
            let wc = WNDCLASSW {
                lpfnWndProc: Some(sink_wndproc),
                hInstance: hinst,
                lpszClassName: windows::core::PCWSTR(class_name.as_ptr()),
                ..Default::default()
            };
            unsafe { RegisterClassW(&wc) };
            let title = wide("plan804 拖出接收窗🎉");
            // 全屏置顶接收窗：活桌面可能有其他置顶窗抢落点（实录：DragEnter
            // 命中而 Drop 被他窗吞）——全屏覆盖使任何释放位置都落本目标，
            // oracle 确定性不再依赖桌面瞬时 z 序（无框全屏，五会话约 5s）。
            let (sw, sh) = unsafe {
                (
                    GetSystemMetrics(SM_CXVIRTUALSCREEN).max(200),
                    GetSystemMetrics(SM_CYVIRTUALSCREEN).max(200),
                )
            };
            let hwnd = unsafe {
                CreateWindowExW(
                    Default::default(),
                    windows::core::PCWSTR(class_name.as_ptr()),
                    windows::core::PCWSTR(title.as_ptr()),
                    WINDOW_STYLE(0x80000000), // WS_POPUP（无框全屏）
                    0,
                    0,
                    sw,
                    sh,
                    None,
                    None,
                    Some(&hinst),
                    None,
                )
                .expect("CreateWindowExW")
            };
            unsafe {
                let _ = ShowWindow(hwnd, SW_SHOWNORMAL);
                // 证据窗置顶+前台（防桌面遮挡/焦点他窗吃掉合成输入）。
                let _ = SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
                let _ = windows::Win32::UI::WindowsAndMessaging::SetForegroundWindow(hwnd);
            }
            pump_for(std::time::Duration::from_millis(150));
            Self { hwnd }
        }

        fn center(&self) -> (i32, i32) {
            let mut r = windows::Win32::Foundation::RECT::default();
            unsafe { GetWindowRect(self.hwnd, &mut r).expect("GetWindowRect") };
            ((r.left + r.right) / 2, (r.top + r.bottom) / 2)
        }
    }

    impl Drop for SinkWindow {
        fn drop(&mut self) {
            unsafe {
                let _ = RevokeDragDrop(self.hwnd);
                let _ = DestroyWindow(self.hwnd);
            }
        }
    }

    fn pump_for(d: std::time::Duration) {
        let deadline = std::time::Instant::now() + d;
        while std::time::Instant::now() < deadline {
            let mut msg = windows::Win32::UI::WindowsAndMessaging::MSG::default();
            unsafe {
                while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                    let _ = windows::Win32::UI::WindowsAndMessaging::TranslateMessage(&msg);
                    windows::Win32::UI::WindowsAndMessaging::DispatchMessageW(&msg);
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    fn env_gate() -> bool {
        if std::env::var("AUTO_LANG_DND_E2E").map_or(false, |v| v == "1") {
            true
        } else {
            eprintln!("skip: AUTO_LANG_DND_E2E != 1（真拖出实机档不进日常门）");
            false
        }
    }

    /// V-05 主体：VM shim → 真 provider → 真 DoDragDrop（测试线程=输入
    /// 线程内联阻塞）；text/files/virtual_files 三载荷 + ESC 取消 + 复原。
    #[test]
    fn v05_real_dodragdrop_sessions_via_vm_shim() {
        if !env_gate() {
            return;
        }
        use auto_lang_core::vm::engine::AutoVM;
        use auto_lang_core::vm::native::shim_dnd_start;
        use auto_lang_core::vm::task::AutoTask;
        use auto_lang_core::vm::virt_memory::VirtualFlash;

        assert!(unsafe { OleInitialize(None) }.is_ok(), "OleInitialize");
        let caller_tid = unsafe { GetCurrentThreadId() };

        let dir = std::env::temp_dir().join(format!("plan804-dnd-{}-🎉", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let real_file = dir.join("现有-文件🎉.txt");
        std::fs::write(&real_file, b"plan804 real drag file").expect("write real file");

        let sink = SinkWindow::create();
        let target: IDropTarget = SinkDropTarget.into();
        unsafe { RegisterDragDrop(sink.hwnd, &target).expect("RegisterDragDrop") };
        let (cx, cy) = sink.center();

        let vm = AutoVM::new_without_host_snapshot(VirtualFlash::new(0), 512);
        vm.set_dnd_host_provider(std::sync::Arc::new(
            crate::ui::dnd_host_provider::UiDndHostProvider,
        ));

        // 输入清洁门：活桌面并发用户输入会 interleaved（实录：用户按住
        // 鼠标时 SEEN 提前置位、释放即提前终止会话）——启动前等待异步
        // 键态无按下（左/右键/ESC，至多 10s）。
        fn wait_clean_input() {
            use windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
            loop {
                let held = [0x01u16, 0x02, 0x1B]
                    .iter()
                    .any(|&vk| unsafe { GetAsyncKeyState(vk as i32) } as u16 & 0x8000 != 0);
                if !held || std::time::Instant::now() > deadline {
                    return;
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }

        // 输入复原守卫（F804-R2-02：曾出现测试成功退出后合成左键残留按下
        // ——独立反控复现）。Drop 在 panic/提前返回/正常路径统一保证：
        // 光标复原 + ESC/左右键配对释放（幂等），驱动线程由 run_session
        // 显式 join，正常断言一律在收尾完成之后。
        struct InputGuard {
            saved: POINT,
        }
        impl InputGuard {
            fn new() -> Self {
                let mut p = POINT::default();
                let _ = unsafe { GetCursorPos(&mut p) };
                Self { saved: p }
            }
        }
        impl Drop for InputGuard {
            fn drop(&mut self) {
                unsafe {
                    let _ = SetCursorPos(self.saved.x, self.saved.y);
                }
                send_key(VK_ESCAPE, true);
                send_mouse(MOUSEEVENTF_LEFTUP, 0, 0);
            }
        }

        // 单会话：清洁门 → 守卫 → spawn 驱动（保留 join 句柄）→ shim 内联
        // 阻塞至会话结束 → **join 驱动（收尾完成）** → 显式 drop 守卫 →
        // 返回 Bool 出口。此后的断言才允许运行。
        let run_session = |payload_json: &str, cancel: bool| -> bool {
            wait_clean_input();
            let guard = InputGuard::new();
            DROP_DONE.store(false, Ordering::SeqCst);
            *RECORD.lock().unwrap() = None;
            let done_flag = std::sync::Arc::new(AtomicBool::new(false));
            let flag_for_driver = done_flag.clone();
            let driver = std::thread::spawn(move || drive_drag(cx, cy, cancel, flag_for_driver));
            let mut task = AutoTask::new(0, 512, 0);
            let idx = vm.add_string(payload_json.as_bytes().to_vec());
            vm.rc_push_str_idx(&mut task, idx);
            shim_dnd_start(&mut task, &vm).expect("shim 成功");
            done_flag.store(true, Ordering::SeqCst);
            let ok = auto_val::decode_bool(task.ram.pop_nv());
            // 收尾完成（驱动 join——其尾段必发 LEFTUP；看门狗在旗标置位
            // 后 ≤1s 退出）才允许测试继续/退出；panic 时守卫 Drop 兜底。
            driver.join().expect("dnd driver join");
            drop(guard);
            ok
        };

        // ── 会话 1：text 载荷真落下 ──
        let text_json = "{\"text\":\"来自 plan804 真会话的拖出文本🎉\"}";
        assert!(run_session(text_json, false), "text 会话 true=执行过会话");
        assert!(
            DROP_DONE.load(Ordering::SeqCst),
            "返回时 Drop 已完成（返回在会话结束之后）"
        );
        let rec = RECORD.lock().unwrap().clone().expect("drop record");
        assert_eq!(
            rec.text.as_deref(),
            Some("来自 plan804 真会话的拖出文本🎉"),
            "接收内容一致"
        );
        assert_eq!(
            rec.drop_thread,
            Some(caller_tid),
            "Drop 回调线程 == 调用（输入）线程"
        );
        assert_eq!(rec.enter_thread, Some(caller_tid), "DragEnter 同线程");
        assert_eq!(
            crate::ui::native_dnd::win32::take_finished_effect().map(|e| e.as_str()),
            Some("copy"),
            "完成效果经事件通道归发起侧（copy）"
        );

        // ── 会话 2：现有文件（CF_HDROP）真落下 ──
        let file_json = format!(
            "{{\"files\":[\"{}\"]}}",
            real_file.to_string_lossy().replace('\\', "\\\\")
        );
        assert!(run_session(&file_json, false), "files 会话 true");
        let rec = RECORD.lock().unwrap().clone().expect("drop record 2");
        assert_eq!(rec.files.len(), 1, "CF_HDROP 一项");
        assert!(
            rec.files[0]
                .replace('/', "\\")
                .eq_ignore_ascii_case(&real_file.to_string_lossy().replace('/', "\\")),
            "接收路径一致（got {:?}）",
            rec.files
        );
        assert_eq!(
            crate::ui::native_dnd::win32::take_finished_effect().map(|e| e.as_str()),
            Some("copy")
        );

        // ── 会话 3：虚拟文件真落下（落地 bytes 一致）──
        let vf_json = concat!(
            "{\"virtual_files\":[",
            "{\"name\":\"虚拟-落地🎉.md\",\"content\":\"# plan804 虚拟文件内容\",\"mime\":\"text/markdown\"},",
            "{\"name\":\"bin-β.bin\",\"bytes_b64\":\"YmluYXJ5AA==\"}",
            "]}"
        );
        assert!(run_session(vf_json, false), "虚拟文件会话 true");
        let rec = RECORD.lock().unwrap().clone().expect("drop record 3");
        assert_eq!(
            rec.vfile_names,
            vec!["虚拟-落地🎉.md", "bin-β.bin"],
            "描述符名一致"
        );
        assert_eq!(
            rec.vfile_bytes,
            vec![
                "# plan804 虚拟文件内容".as_bytes().to_vec(),
                b"binary\0".to_vec()
            ],
            "FileContents 落地 bytes 一致（content 与 base64 双通道）"
        );
        assert_eq!(
            crate::ui::native_dnd::win32::take_finished_effect().map(|e| e.as_str()),
            Some("copy")
        );

        // ── 会话 4：ESC 取消——true（执行过会话）+ effect none + 无 Drop ──
        assert!(
            run_session(text_json, true),
            "取消会话仍 true=执行过会话（不得把 cancel 映射 false）"
        );
        assert!(!DROP_DONE.load(Ordering::SeqCst), "取消不产生 Drop");
        assert_eq!(
            crate::ui::native_dnd::win32::take_finished_effect().map(|e| e.as_str()),
            Some("none"),
            "取消会话完成效果 = none"
        );

        // ── 拖旗标/会话串行复原：取消后再来一次真落下仍成立 ──
        assert!(
            run_session(text_json, false),
            "取消后再次会话可用（DRAG_ACTIVE 复原）"
        );
        assert!(DROP_DONE.load(Ordering::SeqCst), "再次 Drop 完成");
        assert!(
            crate::ui::native_dnd::win32::drag_session_generation() >= 5,
            "代际随会话自增"
        );

        let _ = std::fs::remove_dir_all(&dir);
        drop(sink);
    }

    /// F804-R2-01（闭合 R1-F02 剩余不变量）：**真实宿主链**验收——全程仅
    /// 生产入口，零测试侧接线（不手工补锚定/不选交付目标/不注入事件）：
    /// `run_dynamic_desktop`（真 iced desktop 会话，单宿主窗+双虚拟窗）
    /// → 真实鼠标点击发起方按钮（SendInput）→ App handler 内 dnd_start
    /// （UI 线程内联模态 DoDragDrop）→ `session::dnd_finished_subscription`
    /// （生产 16ms 泵，tokio 侧不依赖 UI 线程）→ 生产 update 臂
    /// `DesktopEvent::DndFinished` → `dnd_finished_target`（生产锚定，由
    /// 生产 dispatch 环置位）→ `inject_native_event` → **发起方 App 的
    /// on_dnd_finished handler** → fs.write_text 落盘证据 + Process.exit(0)
    /// → run 返回。焦点他窗偏差场景（锚定优先于完成时焦点）由既有
    /// `dnd_finished_delivery_anchors_at_initiator`（renderer 单测，真函数
    /// +焦点在 B 对照）覆盖——两者互补构成 V05 完整 oracle：本测证明
    /// 真实链路端到端交付，该测证明路由偏差抗性。
    /// 输入纪律同 F804-R2-02：清洁门 + join 驱动 + panic 守卫。
    /// F804-R3-02：子进程所有权守卫——spawn 成功即归属，Drop（成功/断言
    /// 失败 panic/超时/任何 unwind 出口）终止并 wait，宿主子进程零遗留
    /// （复审两轮复现：启动超时后宿主存活）。
    struct ChildGuard(std::process::Child);
    impl ChildGuard {
        fn child(&mut self) -> &mut std::process::Child {
            &mut self.0
        }
    }
    impl Drop for ChildGuard {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }

    /// F804-R3-01：严格清洁门（真宿主档）——含本测注入的全部键
    /// （LBUTTON/RBUTTON/ESC/TAB/RETURN；复审实录：桌面预存 RETURN 按下
    /// 时不得注入键盘）。10s 内不清即 panic 快败，绝不外泄键入。
    fn wait_clean_input_strict() {
        use windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        loop {
            let held = [0x01u16, 0x02, 0x1B, 0x09, 0x0D]
                .iter()
                .any(|&vk| unsafe { GetAsyncKeyState(vk as i32) } as u16 & 0x8000 != 0);
            if !held {
                return;
            }
            if std::time::Instant::now() > deadline {
                panic!("输入未清洁（LBUTTON/RBUTTON/ESC/TAB/RETURN 有按下）——为避免键入外泄快败，清洁后重跑");
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    #[cfg(feature = "ui-iced")]
    #[test]
    fn v05_real_host_dnd_finished_delivery() {
        if !env_gate() {
            return;
        }
        use windows::Win32::Foundation::{HWND, RECT};
        use windows::Win32::UI::WindowsAndMessaging::GetWindowRect;

        // 真 iced 事件环必须驻留真 main 线程（winit 限制，实录：非主线程
        // panic "significant cross-platform compatibility hazard"）——真宿主
        // 以**子进程**运行（与生产 `auto run` 同形：独立进程真 main）。
        // bin = examples/plan804_real_host（仅显式构建，不进日常档）。
        // **测试内不再嵌套 cargo build**（实录：cargo test 持靶目录锁期间
        // 内层 build 死锁）——exe 由门禁命令先行构建：
        //   cargo build -p auto-lang --example plan804_real_host --features native-dnd
        let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repo root");
        let host_exe = repo_root
            .join("target/debug/examples/plan804_real_host.exe")
            .canonicalize()
            .unwrap_or_else(|_| {
                panic!(
                    "plan804_real_host.exe 未构建——先跑：cargo build -p auto-lang                      --example plan804_real_host --features native-dnd"
                )
            });

        let marker_dir = std::env::temp_dir().join(format!(
            "plan804-host-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.subsec_nanos())
                .unwrap_or(0)
        ));
        std::fs::create_dir_all(&marker_dir).expect("marker dir");
        let started = marker_dir.join("started.txt");
        let finish = marker_dir.join("finish.txt");
        let other_finish = marker_dir.join("other-finish.txt");

        // 启动真宿主子进程。
        let mut child_guard = ChildGuard(
            std::process::Command::new(&host_exe)
                .arg(&marker_dir)
                .spawn()
                .expect("spawn real host child"),
        );
        let child_pid = child_guard.child().id();

        // 等子进程宿主窗出现（按子进程 pid 找其顶层可见窗）。
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(25);
        let mut host = None;
        while host.is_none() {
            for h in top_windows_of(child_pid as u32) {
                let mut r = RECT::default();
                if unsafe { GetWindowRect(HWND(h as *mut _), &mut r) }.is_ok()
                    && r.right - r.left > 200
                    && r.bottom - r.top > 150
                {
                    host = Some((h, r));
                    break;
                }
            }
            if host.is_some() || std::time::Instant::now() > deadline {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(150));
        }
        let (_host_hwnd, r) =
            host.unwrap_or_else(|| panic!("真宿主窗 25s 未出现（child pid {child_pid}）"));

        // 输入纪律（F804-R2-02/R3-01）：严格清洁门（含 TAB/RETURN——本测
        // 注入键盘，预存按下时快败不注入）+ 守卫。
        wait_clean_input_strict();
        let guard = HostInputGuard::new();

        // 触发：纯鼠标（实录：desktop 模式 Tab/Enter 焦点在虚拟窗层，
        // 不达按钮 onclick；鼠标位置语义无焦点依赖、无键入外泄面）。
        // 发起方虚拟窗在 example 中为 component[1]（级联 +48=顶层；实录
        // component[0] 被 60% 尺寸后窗几乎全遮挡——按钮零命中），客户区
        // 实测约占宿主窗 x:20%..80%、y:37%..94%。目标点序**先取
        // (50%,57%)**——该窗客户区中心（按钮 w-full h-full 充满窗，任意
        // 客户点命中；中心点对 DPI 换算最鲁棒）。重试环容纳模态启动延迟：
        // 首次点击命中按钮即启动模态，后续 press/release 即真落下（宿主
        // 窗生产 DesktopDropTarget，效果 copy；目标未及注册则 none——均
        // 为真链路完成效果，由 finish marker 实录）。未中则外扩网格
        // （行 42%..86% 避顶部 chrome[实录：顶行误点关闭钮]与壳面板
        // [实录：固定 (30%,30%) 落 dashboard 而不达 App]）。零位移点按
        // 不拖窗。
        std::thread::sleep(std::time::Duration::from_millis(1500));
        let targets: [(u32, u32); 13] = [
            (3, 4), // 首选：发起方窗客户区中心
            (2, 3),
            (4, 3),
            (2, 5),
            (4, 5),
            (3, 3),
            (3, 5),
            (2, 4),
            (4, 4),
            (1, 3),
            (5, 3),
            (1, 6),
            (5, 6),
        ];
        let mut delivered = false;
        for (fx, fy) in targets {
            let cx = r.left + (r.right - r.left) * fx as i32 / 6;
            let cy = r.top + (r.bottom - r.top) * fy as i32 / 7;
            click_at(cx, cy);
            let poll_end = std::time::Instant::now() + std::time::Duration::from_millis(1200);
            while std::time::Instant::now() < poll_end {
                if finish.is_file() {
                    delivered = true;
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(150));
            }
            if delivered {
                break;
            }
        }
        assert!(
            delivered,
            "目标点序扫描未命中发起方按钮（finish marker 缺失）"
        );

        // 子进程退出（Process.exit(0)）。
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
        loop {
            match child_guard.child().try_wait().expect("try_wait child") {
                Some(status) => {
                    assert!(status.success(), "真宿主子进程须正常退出（got {status:?}）");
                    break;
                }
                None if std::time::Instant::now() > deadline => {
                    panic!("真宿主子进程 15s 未退出（守卫 Drop 将终止回收）");
                }
                None => std::thread::sleep(std::time::Duration::from_millis(100)),
            }
        }
        drop(guard);

        // 断言（全部在生产链产物上）：
        assert_eq!(
            std::fs::read_to_string(&started).unwrap_or_default(),
            "started=true",
            "发起方 handler 在真宿主内执行且 dnd_start 返 true（执行过会话）"
        );
        let finish_content = std::fs::read_to_string(&finish).unwrap_or_default();
        assert!(
            finish_content == "finish=copy" || finish_content == "finish=none",
            "完成事件经真订阅+生产 update 臂送达发起方（真落下效果 copy 或              目标未及注册 none；got {finish_content:?}）"
        );
        eprintln!("[plan804-host] finish marker = {finish_content}");
        assert!(
            !other_finish.is_file(),
            "干扰方 App 未收到完成事件（非广播，生产路由只交付一个目标）"
        );
        let _ = std::fs::remove_dir_all(&marker_dir);
    }

    /// 指定进程的顶层可见窗口集合（isize 句柄）。
    fn top_windows_of(pid: u32) -> Vec<isize> {
        use windows::Win32::Foundation::{HWND, LPARAM};
        use windows::Win32::UI::WindowsAndMessaging::{
            EnumWindows, GetWindowThreadProcessId, IsWindowVisible,
        };
        static HITS: std::sync::Mutex<Vec<isize>> = std::sync::Mutex::new(Vec::new());
        thread_local! {
            static PID: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
        }
        unsafe extern "system" fn cb(hwnd: HWND, _l: LPARAM) -> windows::Win32::Foundation::BOOL {
            let mut pid = 0u32;
            unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };
            if pid == PID.with(|p| p.get()) && unsafe { IsWindowVisible(hwnd) }.as_bool() {
                HITS.lock().unwrap().push(hwnd.0 as isize);
            }
            windows::Win32::Foundation::BOOL(1)
        }
        HITS.lock().unwrap().clear();
        PID.with(|c| c.set(pid));
        let _ = unsafe { EnumWindows(Some(cb), LPARAM(0)) };
        HITS.lock().unwrap().clone()
    }

    /// 真宿主档输入守卫（F804-R2-02 纪律：panic/早退路径配对释放+光标复原）。
    struct HostInputGuard {
        saved: POINT,
    }
    impl HostInputGuard {
        fn new() -> Self {
            let mut p = POINT::default();
            let _ = unsafe { GetCursorPos(&mut p) };
            Self { saved: p }
        }
    }
    impl Drop for HostInputGuard {
        fn drop(&mut self) {
            unsafe {
                let _ = SetCursorPos(self.saved.x, self.saved.y);
            }
            send_key(VK_ESCAPE, true);
            send_mouse(MOUSEEVENTF_LEFTUP, 0, 0);
        }
    }

    /// 屏幕坐标短点按（down→up 零位移）。
    fn click_at(x: i32, y: i32) {
        let w = unsafe { GetSystemMetrics(SM_CXSCREEN) as i64 }.max(1);
        let h = unsafe { GetSystemMetrics(SM_CYSCREEN) as i64 }.max(1);
        let ax = (x as i64 * 65535 / (w - 1).max(1)) as i32;
        let ay = (y as i64 * 65535 / (h - 1).max(1)) as i32;
        send_mouse(MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE, ax, ay);
        std::thread::sleep(std::time::Duration::from_millis(40));
        send_mouse(MOUSEEVENTF_LEFTDOWN, 0, 0);
        std::thread::sleep(std::time::Duration::from_millis(60));
        send_mouse(MOUSEEVENTF_LEFTUP, 0, 0);
    }
}
