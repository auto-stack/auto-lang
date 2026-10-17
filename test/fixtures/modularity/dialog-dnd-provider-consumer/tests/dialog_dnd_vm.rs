//! PLAN-804 V-01..V-03：外部 dialog/dnd host provider 注入真 VM 断言
//!（无 UI feature 构建——本 manifest 即边界证明）。
//!
//! 覆盖（计划 §6 矩阵 + T-01 冻结矩阵）：
//! - V-01 真注入：fake provider 实值（互异 Unicode filter/default/JSON）
//!   经 VM 内建 2927/2928/2938 到达 stdout；参数 raw 精确到达（core 不
//!   预解析过滤/JSON）；provider 实现零 UI/OS 类型（独立 manifest 证明，
//!   闭包无 renderer/rfd/COM）；owned String 返回——池外缓冲销毁后 VM
//!   字符串仍有效。
//! - V-02 缺席/取消/形态：无 provider → dialog feature-disabled 形 Err
//!   文本逐字 / dnd false；两槽只装一个互不掩盖；取消=空串；坏/空 JSON
//!   原文透传（缺席臂恒 false）。
//! - V-03 装配/寿命：两 VM 不串线、构造期快照、实例覆盖、Clone/merge 保
//!   provider；no-ui 高层入口**不暗装**（跑过 run_with_capture 后进程
//!   默认槽仍 None）；注入不 bump binding_revision。
//!
//! 本 fixture 仅 no-ui 形态运行：启用 ui-dialog/native-dnd 会经兼容安装
//! 接真 OS 对话框/拖拽（模态/COM），不得作为测试环境。
//!
//! 进程默认槽是进程级全局——所有触槽测试经 `serialized()` 互斥串行并做
//! save/restore（F-785-R1-01 同款纪律）。

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

use auto_lang::vm::engine::AutoVM;
use auto_lang::vm::host_capabilities::{
    clear_process_default_dialog_host_provider, clear_process_default_dnd_host_provider,
    process_default_dialog_host_provider, process_default_dnd_host_provider,
    set_process_default_dialog_host_provider, set_process_default_dnd_host_provider,
    DialogHostProvider, DndHostProvider,
};
use auto_lang::vm::native::NativeInterface;
use auto_lang::vm::task::AutoTask;
use auto_lang::vm::virt_memory::VirtualFlash;

// ── 测试序列化（进程默认槽互斥） ─────────────────────────────────────────

fn serialized() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let m = LOCK.get_or_init(|| Mutex::new(()));
    m.lock().unwrap_or_else(|e| e.into_inner())
}

/// 双槽 save/restore RAII（测试结束还原进入前状态）。
struct DefaultGuard {
    dialog: Option<Arc<dyn DialogHostProvider>>,
    dnd: Option<Arc<dyn DndHostProvider>>,
}

impl DefaultGuard {
    fn new() -> Self {
        Self {
            dialog: process_default_dialog_host_provider(),
            dnd: process_default_dnd_host_provider(),
        }
    }
}

impl Drop for DefaultGuard {
    fn drop(&mut self) {
        match self.dialog.take() {
            Some(p) => set_process_default_dialog_host_provider(p),
            None => clear_process_default_dialog_host_provider(),
        }
        match self.dnd.take() {
            Some(p) => set_process_default_dnd_host_provider(p),
            None => clear_process_default_dnd_host_provider(),
        }
    }
}

// ── fake providers（零 UI/OS 类型；记录 raw 参数；返回可配 owned String）──

struct FakeDialog {
    open_ret: String,
    save_ret: String,
    received_open: Mutex<Vec<String>>,
    received_save: Mutex<Vec<String>>,
}

impl FakeDialog {
    fn new(tag: &'static str, open_ret: &str, save_ret: &str) -> Arc<Self> {
        Arc::new(Self {
            // 返回值经临时缓冲构造后销毁——owned String 无池外借用（V-01
            // 字符串寿命锚）。
            open_ret: format!("{}{}", tag, open_ret),
            save_ret: format!("{}{}", tag, save_ret),
            received_open: Mutex::new(Vec::new()),
            received_save: Mutex::new(Vec::new()),
        })
    }
}

impl DialogHostProvider for FakeDialog {
    fn open(&self, filter: &str) -> String {
        self.received_open.lock().unwrap().push(filter.to_string());
        self.open_ret.clone()
    }
    fn save(&self, default_name: &str) -> String {
        self.received_save
            .lock()
            .unwrap()
            .push(default_name.to_string());
        self.save_ret.clone()
    }
}

struct FakeDnd {
    ret: bool,
    received: Mutex<Vec<String>>,
}

impl DndHostProvider for FakeDnd {
    fn start(&self, payload_json: String) -> bool {
        self.received.lock().unwrap().push(payload_json);
        self.ret
    }
}

fn fake_dnd(ret: bool) -> Arc<FakeDnd> {
    Arc::new(FakeDnd {
        ret,
        received: Mutex::new(Vec::new()),
    })
}

// ── 程序源 ──────────────────────────────────────────────────────────────

const DIALOG_PROGRAM: &str = concat!(
    "print(dialog_open(\" .at, .au 文件🎉 \"))\n",
    "print(dialog_save(\"默认名-乙🎉.md\"))\n",
);

const DND_PROGRAM: &str = concat!(
    "print(dnd_start(\"{\\\"text\\\":\\\"拖出-丙🎉\\\"}\"))\n",
    "print(dnd_start(\"not json\"))\n",
    "print(dnd_start(\"{}\"))\n",
);

// ── 低层编译链（公开 API：Parser→Codegen→Linker→VirtualFlash→AutoVM）──

struct CompiledProgram {
    code: Vec<u8>,
    exports: HashMap<String, u32>,
    object_keys: Vec<Vec<auto_val::ValueKey>>,
    object_types: Vec<Vec<auto_lang::vm::codegen::ObjectType>>,
    strings: Vec<Vec<u8>>,
}

fn compile_program(src: &str) -> CompiledProgram {
    use auto_lang::vm::codegen::Codegen;
    use auto_lang::vm::loader::Linker;
    use auto_lang::vm::opcode::OpCode;

    auto_lang::vm::native_registry::register_builtin_natives();
    let mut parser = auto_lang::parser::Parser::from(src);
    let ast = parser.parse().expect("parse");

    let mut codegen = Codegen::new();
    // 顶层语句包装（镜像 lib.rs 批处理路径：FN_PROLOG+RESERVE_STACK+HALT）。
    codegen.emit_op(OpCode::FN_PROLOG);
    codegen.emit_byte(0);
    codegen.emit_byte(16);
    codegen.emit_op(OpCode::RESERVE_STACK);
    codegen.emit_byte(16);
    for stmt in &ast.stmts {
        codegen.compile_stmt(stmt).expect("compile_stmt");
    }
    codegen.code.push(OpCode::HALT as u8);

    let strings = codegen.strings.clone();
    let object_keys = codegen.object_keys.clone();
    let object_types = codegen.object_types.clone();
    let module = codegen.finish("<main>".to_string());
    let mut linker = Linker::new();
    linker.add_entry_module(module);
    let (code, exports) = linker.link().expect("link");

    CompiledProgram {
        code,
        exports,
        object_keys,
        object_types,
        strings,
    }
}

fn build_vm(compiled: &CompiledProgram) -> (AutoVM, Arc<std::sync::RwLock<String>>) {
    let flash = VirtualFlash::from_vec_with_metadata(
        compiled.code.clone(),
        compiled.exports.clone(),
        compiled.object_keys.clone(),
        compiled.object_types.clone(),
    );
    let (mut vm, buf) = AutoVM::new_with_capture(flash, 8192);
    vm.load_strings(compiled.strings.clone());
    (vm, buf)
}

fn run_with(vm: &AutoVM, buf: &Arc<std::sync::RwLock<String>>) -> String {
    let _task = vm.spawn_task(0, 16384);
    auto_lang::get_global_runtime().block_on(vm.run_task_loop());
    buf.read().unwrap().clone()
}

fn stdout_lines(s: &str) -> Vec<&str> {
    s.lines().collect()
}

// ── V-01 真注入 ─────────────────────────────────────────────────────────

/// V-01(a) 全管线：进程默认 A → stdout 两行 A 值；换 B → B 值。真 VM
/// stdout 证明 external provider 被真实调用（非缺席臂伪供值）。
#[test]
fn v01_process_default_reaches_real_vm_stdout() {
    let _g = serialized();
    let _guard = DefaultGuard::new();

    let a = FakeDialog::new(
        "A:",
        "C:\\plan804\\打开-甲🎉.at",
        "C:\\plan804\\保存-甲🎉.md",
    );
    set_process_default_dialog_host_provider(a.clone());
    let (_, stdout_a) = auto_lang::run_with_capture(DIALOG_PROGRAM).expect("run A");
    assert_eq!(
        stdout_lines(&stdout_a),
        vec!["A:C:\\plan804\\打开-甲🎉.at", "A:C:\\plan804\\保存-甲🎉.md",],
        "provider A 实值须到达 stdout（got {:?}）",
        stdout_a
    );
    // raw 参数精确到达（含混合分隔/前后空白的原始串——core 不预解析）。
    assert_eq!(
        *a.received_open.lock().unwrap(),
        vec![" .at, .au 文件🎉 ".to_string()],
        "open 收到精确 raw filter"
    );
    assert_eq!(
        *a.received_save.lock().unwrap(),
        vec!["默认名-乙🎉.md".to_string()],
        "save 收到精确 raw default_name"
    );

    let b = FakeDialog::new("B:", "D:\\另处\\开启-β.at", "D:\\另处\\存为-β.txt");
    set_process_default_dialog_host_provider(b);
    let (_, stdout_b) = auto_lang::run_with_capture(DIALOG_PROGRAM).expect("run B");
    assert_eq!(
        stdout_lines(&stdout_b),
        vec!["B:D:\\另处\\开启-β.at", "B:D:\\另处\\存为-β.txt",],
        "换默认 B 后实值同链到达"
    );
}

/// V-01(b) dnd 全管线：进程默认回放 true → 三行（true/false=false 形由
/// fake 决定——本测回放恒定值证明链路）；owned JSON 三种形态（合法组合/
/// 坏 JSON/空对象）**原文透传**（core 不修复/不预解析）。
#[test]
fn v01_dnd_provider_json_passthrough() {
    let _g = serialized();
    let _guard = DefaultGuard::new();

    let dnd = fake_dnd(true);
    set_process_default_dnd_host_provider(dnd.clone());
    let (_, stdout) = auto_lang::run_with_capture(DND_PROGRAM).expect("run dnd");
    assert_eq!(
        stdout_lines(&stdout),
        vec!["true", "true", "true"],
        "回放 true 三连（got {:?}）",
        stdout
    );
    assert_eq!(
        *dnd.received.lock().unwrap(),
        vec![
            "{\"text\":\"拖出-丙🎉\"}".to_string(),
            "not json".to_string(),
            "{}".to_string(),
        ],
        "三形态 owned JSON 原文透传（坏/空 JSON 不在 core 侧变形）"
    );
}

/// V-01(c) 低层双活实例：两 VM 同时存活、各持不同 provider，读数互不
/// 串线；构造期快照——换进程默认不影响已建实例；实例级覆盖生效。
#[test]
fn v01_two_live_instances_do_not_cross_talk() {
    let _g = serialized();
    let _guard = DefaultGuard::new();

    let compiled = compile_program(DIALOG_PROGRAM);

    set_process_default_dialog_host_provider(FakeDialog::new("A:", "p1-open.at", "p1-save.md"));
    let (vm1, buf1) = build_vm(&compiled);
    set_process_default_dialog_host_provider(FakeDialog::new("B:", "p2-open.at", "p2-save.md"));
    let (vm2, buf2) = build_vm(&compiled);

    assert!(vm1.dialog_host_provider().is_some(), "vm1 持构造期快照");
    assert!(vm2.dialog_host_provider().is_some());

    let out1 = run_with(&vm1, &buf1);
    let out2 = run_with(&vm2, &buf2);
    assert_eq!(
        stdout_lines(&out1),
        vec!["A:p1-open.at", "A:p1-save.md"],
        "vm1 保持快照 A"
    );
    assert_eq!(
        stdout_lines(&out2),
        vec!["B:p2-open.at", "B:p2-save.md"],
        "vm2 取新默认 B"
    );

    // 实例级覆盖：只影响被覆盖实例。
    vm2.set_dialog_host_provider(FakeDialog::new("C:", "ovr.at", "ovr.md"));
    let out2b = run_with(&vm2, &buf2);
    let lines2b = stdout_lines(&out2b);
    assert_eq!(
        &lines2b[lines2b.len() - 2..],
        &["C:ovr.at", "C:ovr.md"],
        "实例级覆盖生效"
    );
    let out1_again = run_with(&vm1, &buf1);
    let lines1 = stdout_lines(&out1_again);
    assert_eq!(
        &lines1[lines1.len() - 2..],
        &["A:p1-open.at", "A:p1-save.md"],
        "vm2 覆盖不影响 vm1（不串线）"
    );
}

// ── V-02 缺席/取消/槽独立 ───────────────────────────────────────────────

/// 无 provider → dialog feature-disabled 形 Err 文本逐字；同入口 dnd 侧
/// false（G5 值降级）。no-ui 高层入口**不暗装**（跑过后双槽仍 None）。
#[test]
fn v02_absent_contract_and_no_hidden_install() {
    let _g = serialized();
    let _guard = DefaultGuard::new();

    clear_process_default_dialog_host_provider();
    clear_process_default_dnd_host_provider();

    let err = auto_lang::run_with_capture(DIALOG_PROGRAM)
        .expect_err("no-ui 无 dialog provider 须 raise 原 feature-disabled 错误");
    assert!(
        err.to_string()
            .contains("dialog_open: the `ui-dialog` feature is disabled"),
        "open 缺席错误文本逐字保持（got {:?}）",
        err.to_string()
    );

    // dnd 缺席：值降级 false（非 raise）。
    let (_, stdout) = auto_lang::run_with_capture(DND_PROGRAM).expect("dnd 缺席=值降级");
    assert_eq!(
        stdout_lines(&stdout),
        vec!["false", "false", "false"],
        "缺席 dnd=false"
    );

    // 不暗装：高层入口跑过后双进程默认槽仍 None（no-ui 无安装路径——
    // "原始 AutoVM 仍按既有 provider 模式只快照"）。
    assert!(
        process_default_dialog_host_provider().is_none()
            && process_default_dnd_host_provider().is_none(),
        "no-ui 高层入口不得暗装 dialog/dnd 默认"
    );
}

/// 两槽只装一个：dialog 在位、dnd 缺席 → open 供值+dnd false；反向
/// dnd 在位、dialog 缺席 → dnd 供值+dialog Err（一槽在位不掩盖另一槽
/// 缺席）。
#[test]
fn v02_slots_are_independent() {
    let _g = serialized();
    let _guard = DefaultGuard::new();

    // 只装 dialog。
    clear_process_default_dnd_host_provider();
    set_process_default_dialog_host_provider(FakeDialog::new("X:", "only.at", "only.md"));
    let (_, stdout) = auto_lang::run_with_capture(DND_PROGRAM).expect("dnd 缺席=值降级");
    assert_eq!(
        stdout_lines(&stdout),
        vec!["false", "false", "false"],
        "dialog 在位不掩盖 dnd 缺席"
    );

    // 只装 dnd（清 dialog 槽后）。
    clear_process_default_dialog_host_provider();
    set_process_default_dnd_host_provider(fake_dnd(false));
    let err = auto_lang::run_with_capture(DIALOG_PROGRAM)
        .expect_err("仅 dnd 在位：dialog 仍缺席报错（槽独立）");
    assert!(
        err.to_string()
            .contains("dialog_open: the `ui-dialog` feature is disabled"),
        "槽独立：dialog 缺席错误不因 dnd 在位而改变"
    );
    let (_, stdout) = auto_lang::run_with_capture(DND_PROGRAM).expect("dnd 在位");
    assert_eq!(
        stdout_lines(&stdout),
        vec!["false", "false", "false"],
        "dnd 回放 false 到达"
    );
}

/// 取消 = 空串（值语义非 raise）：provider 回 "" → stdout 空行。
#[test]
fn v02_cancel_is_empty_string() {
    let _g = serialized();
    let _guard = DefaultGuard::new();

    set_process_default_dialog_host_provider(FakeDialog::new("", "", ""));
    let (_, stdout) = auto_lang::run_with_capture(DIALOG_PROGRAM).expect("取消=空串非错误");
    assert_eq!(stdout_lines(&stdout), vec!["", ""], "open/save 取消均空串");
}

// ── V-03 装配/身份/寿命 ─────────────────────────────────────────────────

/// NI clone/merge 保双槽 provider；注入不 bump binding_revision；ID/名
/// 钉扎（2927/2928/2938 解析不变）。
#[test]
fn v03_identity_and_merge_preserve_providers() {
    use auto_lang::vm::native::{NATIVE_DIALOG_OPEN, NATIVE_DIALOG_SAVE, NATIVE_DND_START};

    let _g = serialized();
    let _guard = DefaultGuard::new();

    // 名/ID 查找（production 接口，CALL_SPEC 同源表）。
    let ni = NativeInterface::production();
    assert_eq!(ni.resolve("auto.dialog.open"), Some(NATIVE_DIALOG_OPEN));
    assert_eq!(ni.resolve("auto.dialog.save"), Some(NATIVE_DIALOG_SAVE));
    assert_eq!(ni.resolve("auto.dnd.start"), Some(NATIVE_DND_START));
    assert_eq!(NATIVE_DIALOG_OPEN, 2927);
    assert_eq!(NATIVE_DIALOG_SAVE, 2928);
    assert_eq!(NATIVE_DND_START, 2938);

    // 注入不 bump binding_revision；Clone/merge 保 provider。
    let rev0 = ni.binding_revision();
    ni.set_dialog_host_provider(FakeDialog::new("I:", "i.at", "i.md"));
    ni.set_dnd_host_provider(fake_dnd(true));
    assert_eq!(
        ni.binding_revision(),
        rev0,
        "注入不得撤销绑定身份/失效 proof"
    );
    let cloned = ni.clone();
    assert!(
        cloned.dialog_host_provider().is_some(),
        "Clone 保 dialog 槽"
    );
    assert!(cloned.dnd_host_provider().is_some(), "Clone 保 dnd 槽");

    // VM 级 merge：目标实例两槽不丢。
    let compiled = compile_program(DIALOG_PROGRAM);
    let (mut vm, _buf) = build_vm(&compiled);
    vm.set_dialog_host_provider(FakeDialog::new("M:", "m.at", "m.md"));
    vm.set_dnd_host_provider(fake_dnd(true));
    let mut carrier = NativeInterface::new();
    carrier.register_static(NATIVE_DIALOG_OPEN, plan804_noop_shim);
    vm.merge_native_interface(&carrier);
    assert!(
        vm.dialog_host_provider().is_some(),
        "merge 后 dialog 槽保留"
    );
    assert!(vm.dnd_host_provider().is_some(), "merge 后 dnd 槽保留");
}

/// merge 载体用 noop shim（register_static 收 fn 指针）。
fn plan804_noop_shim(
    _task: &mut AutoTask,
    _vm: &AutoVM,
) -> Result<(), auto_lang::vm::engine::VMError> {
    Ok(())
}

// ── V-04 ui-dialog-only 高层入口实机档（cfg(ui-dialog-host) + env 门）────
//
// 运行（真模态，会短暂弹窗）：
//   AUTO_LANG_DIALOG_E2E=1 cargo test --manifest-path \
//     test/fixtures/modularity/dialog-dnd-provider-consumer/Cargo.toml \
//     --features ui-dialog-host v04_ui_dialog_only -- --test-threads=1
//
// 证明：ui-dialog 单开（无 ui renderer）下，run_with_capture 高层入口经
// 统一安装的兼容默认恢复真对话框接线（不弹显式 provider 也能工作——
// 迁移前"ui-dialog 单开走原高层 dialog 路径"语义保持）。
#[cfg(feature = "ui-dialog-host")]
mod ui_dialog_only_e2e {
    use std::sync::mpsc;

    #[link(name = "user32")]
    extern "system" {
        fn EnumWindows(cb: extern "system" fn(isize, isize) -> i32, lparam: isize) -> i32;
        fn EnumChildWindows(
            parent: isize,
            cb: extern "system" fn(isize, isize) -> i32,
            lparam: isize,
        ) -> i32;
        fn GetWindowThreadProcessId(hwnd: isize, pid: *mut u32) -> u32;
        fn GetClassNameW(hwnd: isize, buf: *mut u16, max: i32) -> i32;
        fn IsWindowVisible(hwnd: isize) -> i32;
        fn IsWindowEnabled(hwnd: isize) -> i32;
        fn IsWindow(hwnd: isize) -> i32;
        fn GetWindowTextW(hwnd: isize, buf: *mut u16, max: i32) -> i32;
        fn GetWindowTextLengthW(hwnd: isize) -> i32;
        fn SetWindowTextW(hwnd: isize, text: *const u16) -> i32;
        fn PostMessageW(hwnd: isize, msg: u32, wp: usize, lp: isize) -> i32;
    }

    const WM_CLOSE: u32 = 0x0010;
    const BM_CLICK: u32 = 0x00F5;

    fn class_of(hwnd: isize) -> String {
        let mut buf = [0u16; 64];
        let n = unsafe { GetClassNameW(hwnd, buf.as_mut_ptr(), 64) }.max(0) as usize;
        String::from_utf16_lossy(&buf[..n])
    }

    fn window_text(hwnd: isize) -> String {
        let len = unsafe { GetWindowTextLengthW(hwnd) }.max(0) as usize;
        let mut buf = vec![0u16; len + 1];
        let n = unsafe { GetWindowTextW(hwnd, buf.as_mut_ptr(), (len + 1) as i32) }.max(0) as usize;
        String::from_utf16_lossy(&buf[..n])
    }

    thread_local! {
        static FOUND: std::cell::Cell<Option<isize>> = const { std::cell::Cell::new(None) };
        static CHILD_HITS: std::cell::RefCell<Vec<isize>> = std::cell::RefCell::new(Vec::new());
    }

    extern "system" fn enum_cb(hwnd: isize, _l: isize) -> i32 {
        let mut pid = 0u32;
        let _ = unsafe { GetWindowThreadProcessId(hwnd, &mut pid) };
        if pid != std::process::id() {
            return 1;
        }
        if class_of(hwnd) == "#32770"
            && unsafe { IsWindowVisible(hwnd) } != 0
            && unsafe { IsWindowEnabled(hwnd) } != 0
        {
            FOUND.with(|f| f.set(Some(hwnd)));
        }
        1
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

    /// 驱动：找 #32770 → 深找文件名 Edit 设路径 → 真「打开」按钮 BM_CLICK
    /// （accept）或 WM_CLOSE 取消；8s 未关则 WM_CLOSE 兜底（防挂死）。
    fn drive(set_path: &str, accept: bool, ack: mpsc::Sender<(bool, String)>) {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
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
        if dlg == 0 {
            eprintln!("[plan804-uidiag] save-cancel driver: 15s 未找到 #32770");
            let _ = ack.send((false, String::new()));
            return;
        }
        let edit = find_descendant(dlg, "Edit", None);
        let initial = edit.map(window_text);
        if accept {
            if let Some(edit) = edit {
                let w: Vec<u16> = set_path.encode_utf16().chain(std::iter::once(0)).collect();
                unsafe { SetWindowTextW(edit, w.as_ptr()) };
            }
            std::thread::sleep(std::time::Duration::from_millis(120));
            let btn = find_descendant(dlg, "Button", Some("打开"));
            if let Some(btn) = btn {
                unsafe { PostMessageW(btn, BM_CLICK, 0, 0) };
            } else {
                unsafe { PostMessageW(dlg, WM_CLOSE, 0, 0) };
            }
        } else {
            std::thread::sleep(std::time::Duration::from_millis(120));
            unsafe { PostMessageW(dlg, WM_CLOSE, 0, 0) };
        }
        let deadline2 = std::time::Instant::now() + std::time::Duration::from_secs(8);
        while std::time::Instant::now() < deadline2 {
            if unsafe { IsWindow(dlg) } == 0 {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        if unsafe { IsWindow(dlg) } == 0 {
            let _ = ack.send((true, initial.unwrap_or_default()));
        } else {
            eprintln!("[plan804-uidiag] save-cancel driver: WM_CLOSE 后 8s 仍在");
            unsafe { PostMessageW(dlg, WM_CLOSE, 0, 0) };
            let _ = ack.send((false, initial.unwrap_or_default()));
        }
    }

    fn same_path(a: &str, b: &str) -> bool {
        a.replace('/', "\\").to_lowercase() == b.replace('/', "\\").to_lowercase()
    }

    #[test]
    fn v04_ui_dialog_only_high_level_entry_real_dialog() {
        if !std::env::var("AUTO_LANG_DIALOG_E2E").map_or(false, |v| v == "1") {
            eprintln!("skip: AUTO_LANG_DIALOG_E2E != 1（真模态实机档不进日常门）");
            return;
        }
        let dir = std::env::temp_dir().join(format!(
            "plan804-uidialog-{}-{}-🎉",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.subsec_nanos())
                .unwrap_or(0)
        ));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let pick = dir.join("ui-dialog-only-高层🎉.at");
        std::fs::write(&pick, b"plan804 ui-dialog-only e2e").expect("write pick");

        let (tx, rx) = mpsc::channel();
        let path_str = pick.to_string_lossy().into_owned();
        std::thread::spawn(move || drive(&path_str, true, tx));

        let program = "let p = dialog_open(\" .at, 文件🎉 \")\nprint(p)\n";
        let (result, stdout) =
            auto_lang::run_with_capture(program).expect("ui-dialog-only 高层入口成功");
        let _ = result;
        let (found, _) = rx.recv().expect("driver ack");
        assert!(found, "未找到/未驱动 #32770 模态");
        let line = stdout.lines().next().unwrap_or("");
        assert!(
            same_path(line, &pick.to_string_lossy()),
            "ui-dialog-only 高层入口 stdout 为所选路径（got {stdout:?}）"
        );
        // 兼容默认真实入槽（if-absent 补缺——非暗装 ui，仅 dialog 槽）。
        assert!(
            auto_lang::vm::host_capabilities::process_default_dialog_host_provider().is_some(),
            "高层入口经兼容安装补缺 dialog 默认槽"
        );

        // ── save 取消腿（合同 V04"两者取消"——真 save 模态 WM_CLOSE）──
        let (tx2, rx2) = mpsc::channel();
        std::thread::spawn(move || drive("", false, tx2));
        let program2 = "let p = dialog_save(\"ui-dialog-only-取消🎉\")
print(p)
";
        let (_r2, stdout2) = auto_lang::run_with_capture(program2).expect("save 高层入口成功");
        let (found2, seeded) = rx2.recv().expect("driver ack 2");
        assert!(found2, "save 取消模态未找到/未驱动");
        let line2 = stdout2.lines().next().unwrap_or("");
        assert_eq!(line2, "", "save 取消 = 空串（got {stdout2:?}）");
        // save 对话框编辑框须预置默认名（真种子，非 fake）。
        assert!(
            seeded.contains("ui-dialog-only-取消🎉"),
            "save 编辑框须预置默认文件名（got {seeded:?}）"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
