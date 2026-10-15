//! PLAN-779 V-01..V-03：外部 provider 注入真 VM 断言（无 UI feature 构建）。
//!
//! 覆盖（计划 §6 矩阵）：
//! - V-01 真注入：fake provider 实值经 VM 内建 9918/9919 到达 stdout
//!   （run_with_capture 全管线 + 低层双活实例）；A=(17,31)/B=(43,59) 不串线。
//! - V-02 表示：无 provider=0；大值 saturating 收窄 i32::MAX；int lane 出口。
//! - V-03 identity：canonical 名→钉扎 ID 解析、族间不撞；provider 注入不
//!   bump binding_revision；register/merge 重注册既有失效规则保持。
//!
//! 进程默认槽是进程级全局——所有测试经 `serialized()` 互斥串行并做
//! save/restore，避免并行线程互踩。

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock, RwLock};

use auto_lang::vm::engine::{AutoVM, VMError};
use auto_lang::vm::host_capabilities::{
    clear_process_default_frame_provider, process_default_frame_provider,
    set_process_default_frame_provider, FrameProvider,
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
struct DefaultGuard(Option<Arc<dyn FrameProvider>>);

impl DefaultGuard {
    fn new() -> Self {
        Self(process_default_frame_provider())
    }
}

impl Drop for DefaultGuard {
    fn drop(&mut self) {
        match self.0.take() {
            Some(p) => set_process_default_frame_provider(p),
            None => clear_process_default_frame_provider(),
        }
    }
}

// ── fake providers ──────────────────────────────────────────────────────

struct FakeFrame {
    begin: i64,
    present: i64,
}

impl FrameProvider for FakeFrame {
    fn begin_ms(&self) -> i64 {
        self.begin
    }
    fn present_ms(&self) -> i64 {
        self.present
    }
}

fn fake(begin: i64, present: i64) -> Arc<dyn FrameProvider> {
    Arc::new(FakeFrame { begin, present })
}

const FRAME_PROGRAM: &str = concat!("print(frame.begin_ms())\n", "print(frame.present_ms())\n",);

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

fn build_vm(compiled: &CompiledProgram) -> (AutoVM, Arc<RwLock<String>>) {
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

fn run_with(vm: &AutoVM, buf: &Arc<RwLock<String>>) -> String {
    let _task = vm.spawn_task(0, 16384);
    auto_lang::get_global_runtime().block_on(vm.run_task_loop());
    buf.read().unwrap().clone()
}

fn stdout_lines(s: &str) -> Vec<&str> {
    s.lines().collect()
}

// ── V-01 真注入 ─────────────────────────────────────────────────────────

/// V-01(a) 全管线：进程默认 A→stdout 17/31；换 B→43/59。真 VM stdout
/// 证明 external provider 被真实调用（非两函数默认 0 的伪解耦）。
#[test]
fn v01_process_default_reaches_real_vm_stdout() {
    let _g = serialized();
    let _guard = DefaultGuard::new();

    set_process_default_frame_provider(fake(17, 31));
    let (_, stdout_a) = auto_lang::run_with_capture(FRAME_PROGRAM).expect("run A");
    assert_eq!(
        stdout_lines(&stdout_a),
        vec!["17", "31"],
        "provider A 实值须到达 stdout（got {:?}）",
        stdout_a
    );

    set_process_default_frame_provider(fake(43, 59));
    let (_, stdout_b) = auto_lang::run_with_capture(FRAME_PROGRAM).expect("run B");
    assert_eq!(
        stdout_lines(&stdout_b),
        vec!["43", "59"],
        "provider B 实值须到达 stdout（got {:?}）",
        stdout_b
    );
}

/// V-01(b) 低层双活实例：两 VM 同时存活、各持不同 provider，读数互不
/// 串线；构造期快照语义——换进程默认不影响已建实例；实例级覆盖生效。
#[test]
fn v01_two_live_instances_do_not_cross_talk() {
    let _g = serialized();
    let _guard = DefaultGuard::new();

    let compiled = compile_program(FRAME_PROGRAM);

    // vm1 在默认 A 下构造（快照 A）……
    set_process_default_frame_provider(fake(17, 31));
    let (vm1, buf1) = build_vm(&compiled);
    // ……随后进程默认换 B：vm1 的快照不受影响，vm2 取 B。
    set_process_default_frame_provider(fake(43, 59));
    let (vm2, buf2) = build_vm(&compiled);

    assert!(vm1.frame_provider().is_some(), "vm1 应持有构造期快照");
    assert!(vm2.frame_provider().is_some());

    let out1 = run_with(&vm1, &buf1);
    let out2 = run_with(&vm2, &buf2);
    assert_eq!(stdout_lines(&out1), vec!["17", "31"], "vm1 保持快照 A");
    assert_eq!(stdout_lines(&out2), vec!["43", "59"], "vm2 取新默认 B");

    // 实例级覆盖：只影响被覆盖实例，不影响同时存活的 vm1。输出缓冲随
    // run 追加——重跑断言取尾部两行（本 run 的读数）。
    vm2.set_frame_provider(fake(7, 11));
    let out2b = run_with(&vm2, &buf2);
    let lines2b = stdout_lines(&out2b);
    assert_eq!(
        &lines2b[lines2b.len() - 2..],
        &["7", "11"],
        "实例级覆盖生效（got {:?}）",
        out2b
    );
    let out1_again = run_with(&vm1, &buf1);
    let lines1 = stdout_lines(&out1_again);
    assert_eq!(
        &lines1[lines1.len() - 2..],
        &["17", "31"],
        "vm2 覆盖不影响 vm1（不串线；got {:?}）",
        out1_again
    );
}

// ── V-02 表示/缺席 ──────────────────────────────────────────────────────

/// 无 provider（缺席）→ 未捕获语义 0（与 core 现状 no-ui/未捕获一致）。
#[test]
fn v02_absent_provider_yields_zero() {
    let _g = serialized();
    let _guard = DefaultGuard::new();

    clear_process_default_frame_provider();
    let compiled = compile_program(FRAME_PROGRAM);
    let (vm, buf) = build_vm(&compiled);
    assert!(vm.frame_provider().is_none(), "缺席=无 provider");
    let out = run_with(&vm, &buf);
    assert_eq!(stdout_lines(&out), vec!["0", "0"], "未捕获=0");
}

/// 大值 i32::MAX+1（及更宽 i64）→ saturating 收窄 i32::MAX，int lane
/// 单槽出口（十进制 i32 形，非 i64 双槽）。
#[test]
fn v02_large_values_clamp_to_i32_max() {
    let _g = serialized();
    let _guard = DefaultGuard::new();

    let compiled = compile_program(FRAME_PROGRAM);
    let (vm, buf) = build_vm(&compiled);
    vm.set_frame_provider(fake(i32::MAX as i64 + 1, i64::MAX / 2));
    let out = run_with(&vm, &buf);
    assert_eq!(
        stdout_lines(&out),
        vec!["2147483647", "2147483647"],
        "i32::MAX+1 与宽 i64 均 saturating 到 i32::MAX（int lane）"
    );
}

// ── V-03 identity/contract 规则 ─────────────────────────────────────────

/// canonical 名→ID 钉扎解析；族间不撞；provider 注入不 bump revision；
/// register/merge 重注册的既有失效规则保持。
#[test]
fn v03_identity_and_contract_rules_preserved() {
    use auto_lang::vm::native::{
        NATIVE_DIFF_DIRS, NATIVE_DIFF_FILES_WINDOW, NATIVE_FRAME_BEGIN_MS, NATIVE_FRAME_PRESENT_MS,
    };

    let _g = serialized();
    let _guard = DefaultGuard::new();

    // 名/ID 查找（production 接口，CALL_SPEC 同源表）。
    let ni = NativeInterface::production();
    assert_eq!(
        ni.resolve("auto.frame.begin_ms"),
        Some(NATIVE_FRAME_BEGIN_MS),
        "canonical begin_ms 解析到钉扎 ID（期望 9918）"
    );
    assert_eq!(
        ni.resolve("auto.frame.present_ms"),
        Some(NATIVE_FRAME_PRESENT_MS),
        "canonical present_ms 解析到钉扎 ID（期望 9919）"
    );
    assert_eq!(NATIVE_FRAME_BEGIN_MS, 9918);
    assert_eq!(NATIVE_FRAME_PRESENT_MS, 9919);
    // 族间不撞（邻段 9915-9921 实锚）。
    assert_ne!(NATIVE_FRAME_BEGIN_MS, NATIVE_DIFF_DIRS);
    assert_ne!(NATIVE_FRAME_PRESENT_MS, NATIVE_DIFF_FILES_WINDOW);

    // provider 注入不 bump binding_revision（绑定身份不变，proof 保持）。
    let rev0 = ni.binding_revision();
    ni.set_frame_provider(fake(1, 2));
    assert_eq!(
        ni.binding_revision(),
        rev0,
        "provider 注入不得撤销绑定身份/失效 proof"
    );

    // 既有规则保持：register_static 重注册 bump revision。
    let mut ni2 = NativeInterface::production();
    let rev2 = ni2.binding_revision();
    ni2.register_static(NATIVE_FRAME_BEGIN_MS, shim_override_seven);
    assert!(ni2.binding_revision() > rev2, "重注册须 bump revision");

    // 既有规则保持：merge 携带同 ID 覆盖亦 bump（proof 按 revision 失效）。
    let mut carrier = NativeInterface::new();
    carrier.register_static(NATIVE_FRAME_PRESENT_MS, shim_override_seven);
    let mut ni3 = NativeInterface::production();
    let rev_before = ni3.binding_revision();
    ni3.merge(&carrier);
    assert!(
        ni3.binding_revision() > rev_before,
        "merge 须 bump revision"
    );

    // 端到端：重注册覆盖派发——VM merge 覆盖 NI 后程序读覆盖值（7）。
    let compiled = compile_program(FRAME_PROGRAM);
    let (mut vm, buf) = build_vm(&compiled);
    vm.merge_native_interface(&carrier);
    let out = run_with(&vm, &buf);
    assert_eq!(
        stdout_lines(&out),
        vec!["0", "7"],
        "9919 被覆盖 shim 供值 7；9918 未覆盖仍缺席 0"
    );
}

/// 覆盖 shim：单槽 i32 推 7（与真 shim 出口 lane 同形）。
fn shim_override_seven(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    task.ram.push_i32(7);
    Ok(())
}
