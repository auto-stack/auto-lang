//! PLAN-785 V-01..V-05：外部 clipboard text provider 注入真 VM 断言。
//!
//! 覆盖（计划 §6 矩阵 + T-01 反控输入 R-A..R-E）：
//! - V-01 真注入：fake provider 实值经 VM 内建 2925/2926 到达 stdout
//!   （run_with_capture 全管线 + 低层双活实例）；A/B 不串线；set 收到
//!   精确 UTF-8 参数（R-A）。
//! - V-02 ownership/UTF-8：临时 owned String、相邻连续读写后原文本仍
//!   可用（R-B）；空串/中文/emoji；None→空串（OS 不可用值语义）；显式
//!   失败 set→false 不计成功（R-C 部分）。
//! - V-03 缺席矩阵（R-C）：无 UI+无 provider→原 feature-disabled
//!   VMError（文本逐字断言）；低层裸 VM 无 provider 不产 stdout。
//! - V-04 优先矩阵（R-D；ui-host 形态）：显式默认不被兼容安装覆盖；
//!   空槽经真实宿主入口补缺（读侧恢复；断言成功而非内容——headless
//!   与桌面形态内容不定）；实例覆盖独立于进程默认。
//! - V-05 identity（R-E）：canonical 名→钉扎 ID、族间不撞、注入不 bump
//!   binding_revision、register/merge 既有失效规则、Clone/merge 不丢
//!   provider、覆盖 shim 端到端。
//!
//! 进程默认槽是进程级全局——所有测试经 `serialized()` 互斥串行并做
//! save/restore，避免并行线程互踩。
//!
//! 双构建门禁：除本 manifest 默认的 no-ui 形态外，同源测试须在
//! UI 启用形态复跑（`--features ui-host`——passthrough 开
//! auto-lang/ui + auto-lang/ui-clipboard；见 Cargo.toml 头注）。
//! 红线（计划 §5）：任何测试都不得经真实 arboard adapter 写用户剪贴板
//! ——显式 Fake 默认下兼容安装不覆盖（不会触 OS）；ui-host 形态的补缺
//! 断言只走读侧。

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

use auto_lang::vm::engine::{AutoVM, VMError};
use auto_lang::vm::host_capabilities::{
    clear_process_default_clipboard_text_provider, process_default_clipboard_text_provider,
    set_process_default_clipboard_text_provider, ClipboardTextProvider,
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

/// 进程默认槽 save/restore RAII（测试结束还原进入前状态）。
struct DefaultGuard(Option<Arc<dyn ClipboardTextProvider>>);

impl DefaultGuard {
    fn new() -> Self {
        Self(process_default_clipboard_text_provider())
    }
}

impl Drop for DefaultGuard {
    fn drop(&mut self) {
        match self.0.take() {
            Some(p) => set_process_default_clipboard_text_provider(p),
            None => clear_process_default_clipboard_text_provider(),
        }
    }
}

// ── fake provider ───────────────────────────────────────────────────────

struct FakeClipboard {
    /// get 返回值；None = 宿主/OS 不可用（shim 落空串）。
    text_to_return: Mutex<Option<String>>,
    /// set_text 收到的参数记录（精确 UTF-8 断言用）。
    received: Mutex<Vec<String>>,
    /// set_text 返回值（false = 显式失败）。
    set_result: bool,
}

impl FakeClipboard {
    fn new(text: Option<&str>) -> Arc<Self> {
        Arc::new(Self {
            text_to_return: Mutex::new(text.map(|s| s.to_string())),
            received: Mutex::new(Vec::new()),
            set_result: true,
        })
    }

    fn failing() -> Arc<Self> {
        Arc::new(Self {
            text_to_return: Mutex::new(Some("never-read".to_string())),
            received: Mutex::new(Vec::new()),
            set_result: false,
        })
    }

    fn received_snapshot(&self) -> Vec<String> {
        self.received.lock().unwrap().clone()
    }
}

impl ClipboardTextProvider for FakeClipboard {
    fn text(&self) -> Option<String> {
        // 返回**新构造**的 owned String（R-B：provider 不返回借用缓冲，
        // 值生命周期与 provider 调用窗口解耦）。
        self.text_to_return.lock().unwrap().clone()
    }

    fn set_text(&self, text: &str) -> bool {
        self.received.lock().unwrap().push(text.to_string());
        self.set_result
    }
}

// ── 程序与低层编译链（公开 API：Parser→Codegen→Linker→VirtualFlash→AutoVM）──

const GET_PROGRAM: &str = "print(clipboard_text())\n";

const SET_PROGRAM: &str = concat!(
    "let ok = clipboard_set_text(\"hello-β🎉\")\n",
    "print(ok)\n",
);

/// 相邻连续读写 + 事后原文本复用（R-B 生命周期）。
const CYCLE_PROGRAM: &str = concat!(
    "let t = clipboard_text()\n",
    "print(t)\n",
    "let ok = clipboard_set_text(\"甲乙-1\")\n",
    "print(ok)\n",
    "print(clipboard_text())\n",
    "print(t)\n",
);

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
    let mut parser = auto_lang_core::parser::Parser::from(src);
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

// ── V-01 真注入（R-A） ──────────────────────────────────────────────────

/// V-01(a) 全管线：进程默认 A→stdout "alpha-α🎉"；换 B→"beta-β"。真 VM
/// stdout 证明 external provider 被真实调用（非两函数默认错误/空串的伪
/// 解耦）。set 侧：provider 收到精确 UTF-8 参数，VM 读回 true。
#[test]
fn v01_process_default_reaches_real_vm_stdout() {
    let _g = serialized();
    let _guard = DefaultGuard::new();

    let a = FakeClipboard::new(Some("alpha-α🎉"));
    set_process_default_clipboard_text_provider(a.clone());
    let (_, stdout_a) = auto_lang::run_with_capture(GET_PROGRAM).expect("run A");
    assert_eq!(
        stdout_lines(&stdout_a),
        vec!["alpha-α🎉"],
        "provider A 实值须到达 stdout（got {:?}）",
        stdout_a
    );
    assert!(a.received_snapshot().is_empty(), "get 侧不得触发 set 记录");

    let b = FakeClipboard::new(Some("beta-β"));
    set_process_default_clipboard_text_provider(b.clone());
    let (_, stdout_b) = auto_lang::run_with_capture(GET_PROGRAM).expect("run B");
    assert_eq!(stdout_lines(&stdout_b), vec!["beta-β"], "provider B 实值");

    // set 侧（进程默认 B 下）：精确 UTF-8 参数 + Bool true 出口。
    let (_, stdout_set) = auto_lang::run_with_capture(SET_PROGRAM).expect("run set");
    assert_eq!(
        stdout_lines(&stdout_set),
        vec!["true"],
        "set 成功=Bool true"
    );
    assert_eq!(
        b.received_snapshot(),
        vec!["hello-β🎉".to_string()],
        "set 收到精确 UTF-8 参数（got {:?}）",
        b.received_snapshot()
    );
}

/// V-01(b) 低层双活实例：两 VM 同时存活、各持不同 provider，读写互不
/// 串线；构造期快照——换进程默认不影响已建实例；实例级覆盖生效且
/// 不污染其他实例/进程默认。
#[test]
fn v01_two_live_instances_do_not_cross_talk() {
    let _g = serialized();
    let _guard = DefaultGuard::new();

    let compiled_get = compile_program(GET_PROGRAM);
    let compiled_set = compile_program(SET_PROGRAM);

    let a = FakeClipboard::new(Some("A-snapshot"));
    let b = FakeClipboard::new(Some("B-default"));
    let c = FakeClipboard::new(Some("C-override"));

    set_process_default_clipboard_text_provider(a.clone());
    let (vm1, buf1) = build_vm(&compiled_get);
    // set-VM 趁默认仍为 A 时构造（快照 A——断言 set 参数落到 A）。
    let (vm1s, buf1s) = build_vm(&compiled_set);
    set_process_default_clipboard_text_provider(b.clone());
    let (vm2, buf2) = build_vm(&compiled_get);

    assert!(vm1.clipboard_text_provider().is_some(), "vm1 持构造期快照");
    assert!(vm2.clipboard_text_provider().is_some());

    let out1 = run_with(&vm1, &buf1);
    let out2 = run_with(&vm2, &buf2);
    assert_eq!(stdout_lines(&out1), vec!["A-snapshot"], "vm1 保持快照 A");
    assert_eq!(stdout_lines(&out2), vec!["B-default"], "vm2 取新默认 B");

    // 实例级覆盖：只影响被覆盖实例。
    vm2.set_clipboard_text_provider(c.clone());
    let out2b = run_with(&vm2, &buf2);
    let lines2b = stdout_lines(&out2b);
    assert_eq!(
        lines2b[lines2b.len() - 1],
        "C-override",
        "实例级覆盖生效（got {:?}）",
        out2b
    );
    let out1_again = run_with(&vm1, &buf1);
    let lines1 = stdout_lines(&out1_again);
    assert_eq!(
        lines1[lines1.len() - 1],
        "A-snapshot",
        "vm2 覆盖不影响 vm1（got {:?}）",
        out1_again
    );
    // 实例覆盖不污染进程默认。
    let cur = process_default_clipboard_text_provider().expect("默认仍在");
    assert_eq!(
        ClipboardTextProvider::text(&*cur).as_deref(),
        Some("B-default"),
        "实例覆盖不得改写进程默认"
    );

    // set 侧不串线：vm1s（快照 A）收参数，B/C 收不到。
    let out1s = run_with(&vm1s, &buf1s);
    assert_eq!(stdout_lines(&out1s), vec!["true"]);
    assert_eq!(
        a.received_snapshot(),
        vec!["hello-β🎉".to_string()],
        "set 精确落到快照 provider A"
    );
    assert!(
        b.received_snapshot().is_empty() && c.received_snapshot().is_empty(),
        "其他实例/覆盖 provider 不得串线收参"
    );
}

// ── V-02 ownership / UTF-8 / 值语义（R-B + R-C 值臂） ───────────────────

/// 相邻连续读写（set→get→复用原变量 t）后原文本仍正确——provider 返回
/// 临时 owned String，VM 侧值生命周期与 provider 调用窗口解耦。
#[test]
fn v02_owned_string_survives_adjacent_cycles() {
    let _g = serialized();
    let _guard = DefaultGuard::new();

    let p = FakeClipboard::new(Some("中文🎉-keep"));
    set_process_default_clipboard_text_provider(p.clone());
    let (_, stdout) = auto_lang::run_with_capture(CYCLE_PROGRAM).expect("run cycles");
    assert_eq!(
        stdout_lines(&stdout),
        vec!["中文🎉-keep", "true", "中文🎉-keep", "中文🎉-keep"],
        "相邻读写后原文本 t 仍可用（got {:?}）",
        stdout
    );
    assert_eq!(
        p.received_snapshot(),
        vec!["甲乙-1".to_string()],
        "相邻 set 收到精确参数"
    );
}

/// 空串与 None（OS 不可用）同走 String lane 出口空串——值语义非 raise。
#[test]
fn v02_empty_and_unavailable_yield_empty_string() {
    let _g = serialized();
    let _guard = DefaultGuard::new();

    set_process_default_clipboard_text_provider(FakeClipboard::new(Some("")));
    let (_, stdout_empty) = auto_lang::run_with_capture(GET_PROGRAM).expect("run empty");
    assert_eq!(
        stdout_lines(&stdout_empty),
        vec![""],
        "空剪贴板=Some(\"\") → print 空行"
    );

    set_process_default_clipboard_text_provider(FakeClipboard::new(None));
    let (_, stdout_none) = auto_lang::run_with_capture(GET_PROGRAM).expect("run none");
    assert_eq!(
        stdout_lines(&stdout_none),
        vec![""],
        "OS 不可用=None → 空串（非错误）"
    );
}

/// 显式失败：set_text=false → Bool false，失败不计成功（R-C）。
#[test]
fn v02_explicit_set_failure_is_not_success() {
    let _g = serialized();
    let _guard = DefaultGuard::new();

    let f = FakeClipboard::failing();
    set_process_default_clipboard_text_provider(f.clone());
    let (_, stdout) = auto_lang::run_with_capture(SET_PROGRAM).expect("run failing set");
    assert_eq!(
        stdout_lines(&stdout),
        vec!["false"],
        "显式失败=Bool false（不得伪装成功；got {:?}）",
        stdout
    );
    assert_eq!(f.received_snapshot(), vec!["hello-β🎉".to_string()]);
}

// ── V-03 缺席矩阵（R-C） ────────────────────────────────────────────────

/// 无 UI+无 provider → **原 feature-disabled VMError**（文本逐字保持，
/// 函数名即 2925/2926 诊断定位面）。ui-host 形态下同入口经兼容安装恢复
/// OS 语义（成功、单行输出；内容 headless/桌面不定，不作内容断言，且
/// 只走读侧——红线：不写真实剪贴板）。
#[test]
fn v03_absent_capability_contract() {
    let _g = serialized();
    let _guard = DefaultGuard::new();

    clear_process_default_clipboard_text_provider();

    #[cfg(not(feature = "ui-host"))]
    {
        let err = auto_lang::run_with_capture(GET_PROGRAM)
            .expect_err("no-ui 无 provider 须 raise 原 feature-disabled 错误");
        let msg = err.to_string();
        assert!(
            msg.contains("clipboard_text: the `ui-clipboard` feature is disabled"),
            "错误文本逐字保持（got {:?}）",
            msg
        );

        let err_set = auto_lang::run_with_capture(SET_PROGRAM).expect_err("set 侧同形缺席错误");
        assert!(
            err_set
                .to_string()
                .contains("clipboard_set_text: the `ui-clipboard` feature is disabled"),
            "set 缺席错误文本保持（got {:?}）",
            err_set.to_string()
        );
    }

    #[cfg(feature = "ui-host")]
    {
        // 差分 oracle：缺席形（no-ui 臂）为 VMError；ui 构建经真实宿主
        // 入口兼容补缺后程序**成功**。内容/行数不定（真实剪贴板可为任意
        // 多行文本——2026-10-16 实测读得用户多行剪贴板内容），不作内容
        // 断言、不回显内容（隐私）；且只走读侧（红线：不写真实剪贴板）。
        let (_result, _stdout) = auto_lang::run_with_capture(GET_PROGRAM)
            .expect("ui 构建经真实宿主入口兼容补缺后须成功（读侧）");
    }
}

/// 低层裸 VM：无 provider → 槽位 None，程序在 native 调用处中止，
/// stdout 零输出（不产伪值）。
#[test]
fn v03_bare_vm_absent_yields_no_output() {
    let _g = serialized();
    let _guard = DefaultGuard::new();

    clear_process_default_clipboard_text_provider();
    let compiled = compile_program(GET_PROGRAM);
    let (vm, buf) = build_vm(&compiled);
    assert!(
        vm.clipboard_text_provider().is_none(),
        "缺席=无 provider（两构建同形——裸 VM 不经宿主入口）"
    );
    let out = run_with(&vm, &buf);
    assert!(
        stdout_lines(&out).iter().all(|l| l.is_empty()),
        "无 provider 不得产出剪贴板值（got {:?}）",
        out
    );
}

// ── V-04 优先矩阵（R-D；两构建同测，ui-host 形态含真实兼容安装竞争） ────

/// 显式默认经真实高层入口供值：no-ui 下 install 为 no-op；ui-host 下
/// 兼容安装 if-absent 不覆盖显式默认——两形态同一期望（779 V-01(a) 同
/// 构的双构建纪律）。
#[test]
fn v04_explicit_default_survives_entry_install() {
    let _g = serialized();
    let _guard = DefaultGuard::new();

    set_process_default_clipboard_text_provider(FakeClipboard::new(Some("explicit-wins")));
    let (_, stdout) = auto_lang::run_with_capture(GET_PROGRAM).expect("run");
    assert_eq!(
        stdout_lines(&stdout),
        vec!["explicit-wins"],
        "显式默认不被兼容安装覆盖（got {:?}）",
        stdout
    );
}

/// 实例覆盖独立于进程默认：默认 A 在槽、vm 实例覆盖 C——vm 读 C，进程
/// 默认保持 A；后建实例仍取 A。
#[test]
fn v04_instance_override_independent_of_default() {
    let _g = serialized();
    let _guard = DefaultGuard::new();

    let compiled = compile_program(GET_PROGRAM);
    set_process_default_clipboard_text_provider(FakeClipboard::new(Some("default-A")));
    let (vm, buf) = build_vm(&compiled);
    vm.set_clipboard_text_provider(FakeClipboard::new(Some("instance-C")));
    let out = run_with(&vm, &buf);
    assert_eq!(stdout_lines(&out), vec!["instance-C"]);

    let cur = process_default_clipboard_text_provider().unwrap();
    assert_eq!(
        ClipboardTextProvider::text(&*cur).as_deref(),
        Some("default-A"),
        "实例覆盖后进程默认不被污染"
    );

    let (vm2, buf2) = build_vm(&compiled);
    let out2 = run_with(&vm2, &buf2);
    assert_eq!(stdout_lines(&out2), vec!["default-A"], "新实例取默认快照");
}

// ── V-05 identity/contract 规则（R-E） ──────────────────────────────────

/// canonical 名→ID 钉扎解析；族间不撞；provider 注入不 bump revision；
/// register/merge 重注册既有失效规则保持；Clone/merge 路径不丢 provider；
/// 覆盖 shim 端到端派发。
#[test]
fn v05_identity_and_contract_rules_preserved() {
    use auto_lang::vm::native::{
        shim_clipboard_set_text, NATIVE_CLIPBOARD_FILES_GET, NATIVE_CLIPBOARD_SET_TEXT,
        NATIVE_CLIPBOARD_TEXT, NATIVE_DIALOG_OPEN,
    };

    let _g = serialized();
    let _guard = DefaultGuard::new();

    // 名/ID 查找（production 接口，CALL_SPEC 同源表）。
    let ni = NativeInterface::production();
    assert_eq!(
        ni.resolve("auto.clipboard.text"),
        Some(NATIVE_CLIPBOARD_TEXT),
        "canonical clipboard_text 解析到钉扎 ID"
    );
    assert_eq!(
        ni.resolve("auto.clipboard.set_text"),
        Some(NATIVE_CLIPBOARD_SET_TEXT),
        "canonical clipboard_set_text 解析到钉扎 ID"
    );
    assert_eq!(NATIVE_CLIPBOARD_TEXT, 2925);
    assert_eq!(NATIVE_CLIPBOARD_SET_TEXT, 2926);
    // 族间不撞（邻段实锚：dialog 2927 / files 2934）。
    assert_ne!(NATIVE_CLIPBOARD_TEXT, NATIVE_CLIPBOARD_SET_TEXT);
    assert_ne!(NATIVE_CLIPBOARD_TEXT, NATIVE_DIALOG_OPEN);
    assert_ne!(NATIVE_CLIPBOARD_SET_TEXT, NATIVE_CLIPBOARD_FILES_GET);

    // provider 注入不 bump binding_revision（绑定身份不变，proof 保持）。
    let rev0 = ni.binding_revision();
    ni.set_clipboard_text_provider(FakeClipboard::new(Some("x")));
    assert_eq!(
        ni.binding_revision(),
        rev0,
        "provider 注入不得撤销绑定身份/失效 proof"
    );

    // Clone 路径保留 provider（merge_native_interface 经 Clone+merge）。
    let clone = ni.clone();
    assert!(
        clone.clipboard_text_provider().is_some(),
        "NI Clone 不丢 clipboard provider"
    );

    // 既有规则保持：register_static 重注册 bump revision。
    let mut ni2 = NativeInterface::production();
    let rev2 = ni2.binding_revision();
    ni2.register_static(NATIVE_CLIPBOARD_TEXT, shim_override_text);
    assert!(ni2.binding_revision() > rev2, "重注册须 bump revision");

    // 既有规则保持：merge 携带同 ID 覆盖亦 bump（proof 按 revision 失效）。
    let mut carrier = NativeInterface::new();
    carrier.register_static(NATIVE_CLIPBOARD_SET_TEXT, shim_clipboard_set_text);
    let mut ni3 = NativeInterface::production();
    let rev_before = ni3.binding_revision();
    ni3.merge(&carrier);
    assert!(
        ni3.binding_revision() > rev_before,
        "merge 须 bump revision"
    );

    // 端到端：VM merge 覆盖 NI 后——provider 保留 + 覆盖 shim 供值。
    let compiled = compile_program(GET_PROGRAM);
    let (mut vm, buf) = build_vm(&compiled);
    vm.set_clipboard_text_provider(FakeClipboard::new(Some("real-provider")));
    let mut override_carrier = NativeInterface::new();
    override_carrier.register_static(NATIVE_CLIPBOARD_TEXT, shim_override_text);
    vm.merge_native_interface(&override_carrier);
    assert!(
        vm.clipboard_text_provider().is_some(),
        "merge 路径不丢 provider"
    );
    let out = run_with(&vm, &buf);
    assert_eq!(
        stdout_lines(&out),
        vec!["override-7"],
        "2925 被覆盖 shim 供值（真 shim 与覆盖 shim 同 String lane）"
    );
}

/// 覆盖 shim：String lane 出口（与真 shim 同形——add_string+rc_push）。
fn shim_override_text(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let idx = vm.add_string(b"override-7".to_vec());
    vm.rc_push_str_idx(task, idx as usize);
    Ok(())
}
