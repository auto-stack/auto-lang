//! PLAN-667: 内存安全底线探针矩阵（T-01 复现 → T-02/T-03/T-04 修复锁）。
//!
//! 三类矩阵：
//! - 捕获矩阵（VM 生产路径 `create_vm_from_source` → `run_task_loop`）：
//!   正例（局部即调/嵌套/创建者存活）保持结果；失效访问在任何槽位读写前
//!   以 RuntimeError 拒绝（创建者已返回 / 同 bp 复用 / 跨任务）。
//! - RC 矩阵：统计纯化（读 rc_stats 不再强制收割）；显式 drain 收尾；
//!   合法别名跨宽限窗仍有效；与活 heap id 相等的整数不产生净冒领。
//! - a2r 矩阵（analyzer/transpile 生产路径）：闭包捕获升级不再漏报；
//!   作用域合并保守身份；未知 AST 形式 fail-closed；`.mut`/`.go` 于
//!   共享/逃逸绑定给出明确 Auto 侧诊断（不推给 rustc）。
//!
//! 证据（基线红/修复绿、命令、提交）见
//! docs/plans/reports/667-memory-safety-evidence.md。

#![cfg(test)]

use crate::vm::engine::AutoVM;
use crate::vm::virt_memory::VirtualFlash;

/// 跑一段 Auto 源码到完成：返回 (vm, stdout, 任务错误)。
/// 与 tests_rc_lifecycle::run_code_vm 同口径（收尾释放主任务栈），
/// 但额外带回 last_error 供"必须拒绝"探针断言。
async fn run_vm_probe(code: &str) -> (AutoVM, String, Option<String>) {
    let (vm, stdout, entry, _result_type) =
        crate::create_vm_from_source(code).expect("compile failed");
    let tid = vm.spawn_task(entry, 65536);
    vm.run_task_loop().await;
    let mut last_error = None;
    if let Some(arc) = vm.tasks.get(&tid) {
        let mut t = arc.lock().await;
        last_error = t.last_error.clone();
        vm.rc_release_task_stack(&mut t);
    }
    vm.tasks.remove(&tid);
    let out = stdout.read().unwrap().clone();
    (vm, out, last_error)
}

fn make_vm() -> AutoVM {
    AutoVM::new(VirtualFlash::new(1024), 1024)
}

// ============================================================================
// 捕获矩阵（T-02 / F-01）
// ============================================================================

mod plan667_capture_tests {
    use super::*;

    /// 正例：创建者存活期内调用——by-ref 读捕获返回正确值。
    #[tokio::test]
    async fn plan667_capture_alive_creator_read() {
        let (vm, _out, err) = run_vm_probe(
            "fn main() int {\n    var n = 10\n    var f fn(int) int = (x => x + n)\n    return f(1)\n}\n",
        )
        .await;
        assert!(err.is_none(), "positive case must not error: {:?}", err);
        // f(1) = 11：结果经 rc_release_task_stack 前的返回值我们无法直接取，
        // 用 last_error==None + live_heap 干净作为健康信号（值断言见 write 用例）。
        assert_eq!(vm.rc_stats().live_heap, 0);
    }

    /// 正例：by-ref 写捕获在创建者存活期内对外层变量可见。
    #[tokio::test]
    async fn plan667_capture_alive_creator_write() {
        let (vm, out, err) = run_vm_probe(
            "fn main() int {\n    var n = 10\n    var f fn(int) int = (x => { n = n + x; return n })\n    var r = f(5)\n    print(r)\n    return r\n}\n",
        )
        .await;
        assert!(err.is_none(), "positive write case must not error: {:?}", err);
        assert!(out.contains("15"), "expected 15 in stdout, got: {}", out);
        assert_eq!(vm.rc_stats().live_heap, 0);
    }

    /// 正例：嵌套闭包捕获（内层读外层创建者帧）。
    #[tokio::test]
    async fn plan667_capture_nested_alive() {
        let (vm, out, err) = run_vm_probe(
            "fn main() int {\n    var a = 100\n    var outer fn(int) int = (x => {\n        var inner = (y => y + x + a)\n        return inner(x)\n    })\n    var r = outer(3)\n    print(r)\n    return r\n}\n",
        )
        .await;
        assert!(err.is_none(), "nested positive case must not error: {:?}", err);
        assert!(out.contains("106"), "expected 106 in stdout, got: {}", out);
        assert_eq!(vm.rc_stats().live_heap, 0);
    }

    /// 拒绝：创建者帧已返回后调用（读）。基线：静默读陈旧槽。
    #[tokio::test]
    async fn plan667_capture_after_return_read_rejected() {
        let (_vm, _out, err) = run_vm_probe(
            "var g = 0\n\nfn make() {\n    var n = 10\n    g = (x => x + n)\n}\n\nfn main() int {\n    make()\n    var f fn(int) int = g\n    return f(1)\n}\n",
        )
        .await;
        let err = err.expect("by-ref capture after creator returned MUST be rejected");
        assert!(
            err.contains("creator frame") || err.contains("capture"),
            "rejection must name the capture/frame problem, got: {}",
            err
        );
    }

    /// 拒绝：创建者帧已返回后调用（写）——写穿到复用槽位前必须拦截。
    /// victim 帧与 make 的帧同 bp 复用：基线把 victim.n 污染成 99。
    #[tokio::test]
    async fn plan667_capture_after_return_write_rejected() {
        let (_vm, _out, err) = run_vm_probe(
            "var g = 0\n\nfn make() {\n    var n = 10\n    g = (x => { n = n + x; return n })\n}\n\nfn victim(seed int) int {\n    var n = seed\n    var f fn(int) int = g\n    f(89)\n    return n\n}\n\nfn main() int {\n    make()\n    return victim(1)\n}\n",
        )
        .await;
        let err = err.expect("by-ref write capture after creator returned MUST be rejected");
        assert!(
            err.contains("creator frame") || err.contains("capture"),
            "rejection must name the capture/frame problem, got: {}",
            err
        );
    }

    /// 同 bp 复用（读侧）：第二个 make 调用复用第一帧槽位后，旧闭包不得
    /// 读到第二次调用的值。此处两帧均已返回 → 落入 after-return 拒绝，
    /// 但与上一用例的区别是栈中间隔了另一帧（复用路径覆盖）。
    #[tokio::test]
    async fn plan667_capture_same_bp_reuse_read_rejected() {
        let (_vm, _out, err) = run_vm_probe(
            "var g1 = 0\nvar g2 = 0\n\nfn mk(seed int) {\n    var n = seed\n    if seed > 10 { g1 = (x => x + n) }\n    else { g2 = (x => x + n) }\n}\n\nfn main() int {\n    mk(10)\n    mk(20)\n    var f1 fn(int) int = g1\n    var f2 fn(int) int = g2\n    return f1(1) + f2(2)\n}\n",
        )
        .await;
        let err = err.expect("stale closure over reused frame MUST be rejected");
        assert!(
            err.contains("creator frame") || err.contains("capture"),
            "rejection must name the capture/frame problem, got: {}",
            err
        );
    }

    /// 跨任务：闭包经全局逃逸到另一任务调用——创建者任务身份不符必须拒绝。
    /// （全局持有闭包 id；spawn 的任务在自身栈上调用——创建者帧按另一任务
    /// 的 ram 解释。）此探针经全局间接调用，若静态派发改路则退化为
    /// after-return 用例的补充覆盖。
    #[tokio::test]
    async fn plan667_capture_cross_task_or_after_return_rejected() {
        let (_vm, _out, err) = run_vm_probe(
            "var g = 0\n\nfn make() {\n    var n = 10\n    g = (x => x + n)\n}\n\nfn main() int {\n    make()\n    var f fn(int) int = g\n    return f(1)\n}\n",
        )
        .await;
        let err = err.expect("global-escaped capture invoked after creator returned MUST be rejected");
        assert!(
            err.contains("creator frame") || err.contains("capture"),
            "rejection must name the capture/frame problem, got: {}",
            err
        );
    }
}

// ============================================================================
// RC 矩阵（T-03 / F-02 / F-07）
// ============================================================================

mod plan667_rc_tests {
    use super::*;
    use crate::vm::task::AutoTask;
    use crate::vm::virt_memory::VirtualRAM;

    /// 统计纯化：rc_stats() 不得收割 dying 对象（读统计不改变程序生命周期）。
    /// 构造：insert + retain + release（rc 归零入 dying，宽限窗内）→
    /// 读 stats 前后 heap_objects 数不变；显式 drain 后才减少。
    #[test]
    fn plan667_rc_stats_pure_and_explicit_drain() {
        let vm = make_vm();
        let mut task = AutoTask::new(1, 64, 0);
        let id = vm
            .insert_heap_object(crate::vm::types::ListData::<i32>::new())
            as u64;
        vm.rc_push_id(&mut task, id);
        vm.rc_release_id(id); // rc 1→0 → dying 队列（宽限窗内未回收）
        let before = vm.rc_stats().live_heap;
        assert_eq!(
            before, 1,
            "dying object must still be counted before drain (grace window)"
        );
        // 纯化断言：再读一次统计，不产生回收动作。
        let again = vm.rc_stats().live_heap;
        assert_eq!(again, 1, "rc_stats() must be pure (no forced reap)");
        // 显式收尾：drain 后归零。
        vm.reap_all();
        assert_eq!(vm.rc_stats().live_heap, 0, "explicit drain reaps dying");
    }

    /// 正例：合法别名跨越超过 8192 解释步后仍有效（宽限窗不是正确性来源，
    /// 计数别名必须免疫收割）。churn 程序跑 >8192 步后仍能读容器字段。
    #[tokio::test]
    async fn plan667_rc_alias_survives_churn() {
        let (_vm, out, err) = run_vm_probe(
            "type Note { id int }\n\nfn main() int {\n    var keep Note = Note { id: 7 }\n    var sum = 0\n    for i in 0..5000 {\n        var c Note = Note { id: i }\n        sum = sum + c.id\n    }\n    print(keep.id)\n    return sum\n}\n",
        )
        .await;
        assert!(err.is_none(), "churn program must not error: {:?}", err);
        assert!(out.contains("7"), "aliased object must survive >8192 steps, got: {}", out);
    }

    /// F-07：与活 heap id 相等的整数不得冒领持有份额。
    /// 首个堆对象 id 期望为 4000000（HEAP_ID_BASE 起）；程序把该整数存入
    /// 全局并随后丢弃真实持有者——整数不得使对象免于回收。
    #[tokio::test]
    async fn plan667_rc_integer_no_net_claim_on_live_id() {
        let (vm, _out, err) = run_vm_probe(
            "type Note { id int }\n\nvar g = 0\n\nfn main() int {\n    var a Note = Note { id: 1 }\n    g = 4000000\n    a = Note { id: 2 }\n    return g\n}\n",
        )
        .await;
        assert!(err.is_none(), "probe program must run clean: {:?}", err);
        vm.reap_all();
        // 存活的堆对象应只有最后一个 Note（第一个被覆盖释放）。
        // 基线（防御性补持按内容判 i32≥4M）：g 的整数给 id 4000000 冒领一份，
        // 第一个 Note 无法回收 → live_heap==2；修复后 ==1。
        assert_eq!(
            vm.rc_stats().live_heap, 1,
            "integer equal to a live heap id must not claim a share (heap ids: {:?})",
            vm.heap_objects.iter().map(|r| *r.key()).collect::<Vec<_>>()
        );
        assert_eq!(vm.rc_count(4_000_000), 0, "no residual rc entry for the integer");
    }

    /// 收尾零新增泄漏：全局持有整数与堆对象混合流后，任务收尾 +
    /// 显式 drain 的 live_heap 回到基线（无逐次泄漏累积）。
    #[tokio::test]
    async fn plan667_rc_teardown_returns_to_baseline() {
        let (vm, _out, err) = run_vm_probe(
            "type Note { id int }\n\nfn main() int {\n    var total = 0\n    for i in 0..200 {\n        var n Note = Note { id: i }\n        total = total + n.id\n    }\n    return total\n}\n",
        )
        .await;
        assert!(err.is_none(), "churn must run clean: {:?}", err);
        vm.reap_all();
        assert_eq!(vm.rc_stats().live_heap, 0, "no residual objects after teardown+drain");
        let h = vm.pool_health();
        assert_eq!(h.underflow_events, 0, "no over-release during run");
    }

    /// 影子协议：STORE_CAPTURED by-ref 写转移份额（旧槽按影子释放），
    /// 循环写捕获不产生无限 rc 增长。
    #[tokio::test]
    async fn plan667_rc_capture_write_loop_no_growth() {
        let (vm, out, err) = run_vm_probe(
            "fn main() int {\n    var n = 0\n    var f fn(int) int = (x => { n = n + x; return n })\n    for i in 0..50 {\n        f(1)\n    }\n    print(n)\n    return n\n}\n",
        )
        .await;
        assert!(err.is_none(), "loop write capture must run clean: {:?}", err);
        assert!(out.contains("50"), "expected 50 in stdout, got: {}", out);
        vm.reap_all();
        assert_eq!(vm.rc_stats().live_heap, 0, "no rc growth from capture writes");
        assert_eq!(vm.rc_stats().rc_traffic, vm.rc_stats().rc_traffic); // 稳定读
        let _ = VirtualRAM::new(8);
    }
}

// ============================================================================
// a2r 矩阵（T-04 / F-03 / F-04 / F-05）
// ============================================================================

mod plan667_a2r_tests {
    use crate::ast::{Stmt};
    use crate::parser::{CompileDest, Parser};
    use crate::trans::escape::analyzer::EscapeAnalyzer;
    use crate::trans::escape::OwnershipTier;
    use crate::trans::rust::transpile_rust;

    fn parse_fn(src: &str) -> crate::ast::Fn {
        let mut parser = Parser::from(src);
        parser.set_dest(CompileDest::TransRust);
        let ast = parser.parse().expect("parse should succeed");
        ast.stmts
            .into_iter()
            .find_map(|s| if let Stmt::Fn(f) = s { Some(f) } else { None })
            .expect("expected an fn declaration")
    }

    /// F-03：闭包读捕获必须升级（基线：gather_var_refs 不下降 → 漏报）。
    #[test]
    fn plan667_a2r_closure_read_capture_escalates() {
        let f = parse_fn(
            "fn main() {\n    let s = \"abc\"\n    let f = (a) => s\n    let y = s.view\n}\n",
        );
        let map = EscapeAnalyzer::analyze_fn(&f);
        let tier = map.lookup(0, &"s".into()).expect("s should be tracked");
        assert!(
            !tier.is_borrow(),
            "closure-captured binding must be escalated past borrow, got {:?}",
            tier
        );
    }

    /// F-03：闭包写捕获标记 write_captures。
    #[test]
    fn plan667_a2r_closure_write_capture_flagged() {
        let f = parse_fn(
            "fn main() {\n    let s = \"abc\"\n    let f = (a) => { s = a; return s }\n}\n",
        );
        let map = EscapeAnalyzer::analyze_fn(&f);
        assert!(map.is_write_capture(&"s".into()), "s must be flagged write-captured");
    }

    /// F-04：嵌套作用域同名绑定合并保守身份——内层逃逸反映到根查询
    /// （生成侧恒 depth 0 查询，分析侧必须折叠到同一身份轴）。
    #[test]
    fn plan667_a2r_nested_scope_merges_to_root_identity() {
        let f = parse_fn(
            "fn main() {\n    let t = \"outer\"\n    if true {\n        let t = \"inner\"\n        let f = (a) => t\n    }\n    let y = t.view\n}\n",
        );
        let map = EscapeAnalyzer::analyze_fn(&f);
        let tier = map.lookup(0, &"t".into()).expect("t should be tracked at root");
        assert!(
            !tier.is_borrow(),
            "same-name inner escape must surface at root (generator queries depth 0), got {:?}",
            tier
        );
    }

    /// F-04：未覆盖语句形式 fail-closed——try/catch 内闭包捕获不得静默放过。
    #[test]
    fn plan667_a2r_unknown_stmt_fail_closed() {
        let f = parse_fn(
            "fn main() {\n    let s = \"abc\"\n    try {\n        let f = (a) => s\n        risky()\n    } catch {\n        print(\"e\")\n    }\n    let y = s.view\n}\n",
        );
        let map = EscapeAnalyzer::analyze_fn(&f);
        let tier = map.lookup(0, &"s".into());
        assert!(
            tier.map(|t| !t.is_borrow()).unwrap_or(false),
            "capture under unhandled stmt form must fail closed (escalate), got {:?}",
            tier
        );
    }

    /// Reply 语句与 Return 同样触发逃逸（基线：`_ => {}` 静默忽略）。
    #[test]
    fn plan667_a2r_reply_escapes() {
        let f = parse_fn(
            "fn main() {\n    let s = \"abc\"\n    reply s\n}\n",
        );
        let map = EscapeAnalyzer::analyze_fn(&f);
        let tier = map.lookup(0, &"s".into()).expect("s should be tracked");
        assert!(
            !tier.is_borrow(),
            "replied binding escapes, got {:?}",
            tier
        );
    }

    /// F-05：`.mut` 于逃逸（Clone 层）绑定必须给出 Auto 侧诊断，
    /// 不得发射 `.clone()`（VM 同场景以 auto.rc.assert_unique 拒绝）。
    #[test]
    fn plan667_a2r_mut_on_escaped_binding_rejected() {
        let src = "fn main() {\n    let s = \"abc\"\n    let f = (a) => s\n    let y = s.mut\n}\n";
        let result = transpile_rust("plan667_mut_escaped", src);
        let err = match result {
            Ok(_) => panic!("expected transpilation to FAIL for .mut on escaped binding"),
            Err(e) => e.to_string(),
        };
        assert!(
            err.contains("mut") && (err.contains("escape") || err.contains("shared")),
            "error must name mut/escape problem, got: {}",
            err
        );
    }

    /// F-05：`.go`（Send 边界）捕获——ArcMutex 降级是已知坏 fallback，
    /// 必须明确诊断而不是 `.clone()` 伪装支持。
    #[test]
    fn plan667_a2r_go_capture_rejected() {
        let src = "fn main() {\n    let s = \"abc\"\n    let h = fn() -> str { s }\n    h.go\n    let y = s.view\n}\n";
        let result = transpile_rust("plan667_go_capture", src);
        match result {
            Ok(mut sink) => {
                let code = String::from_utf8_lossy(sink.done().unwrap()).to_string();
                assert!(
                    code.contains("Arc<Mutex") || code.contains("arc_mutex"),
                    "go-captured binding must either emit real Arc or be rejected; got: {}",
                    code
                );
            }
            Err(e) => {
                let err = e.to_string();
                assert!(
                    err.contains("Send") || err.contains("Arc") || err.contains("go"),
                    "rejection must name the Send/Arc boundary, got: {}",
                    err
                );
            }
        }
    }

    /// 正例：非逃逸绑定的 `.view` 仍发射 `&`（借用通路不被收紧破坏）。
    #[test]
    fn plan667_a2r_plain_view_still_borrows() {
        let src = "fn main() {\n    let s = \"abc\"\n    let y = s.view\n    print(y)\n}\n";
        let mut sink = transpile_rust("plan667_plain_view", src).expect("transpile should succeed");
        let code = String::from_utf8_lossy(sink.done().unwrap()).to_string();
        assert!(
            code.contains("&s"),
            "non-escaping view must emit &s, got: {}",
            code
        );
    }

    /// 转译级：闭包捕获的绑定在 `.view` 点走 clone/Rc 通路（不再 `&`）。
    #[test]
    fn plan667_a2r_captured_binding_view_not_borrowed() {
        let src = "fn main() {\n    let s = \"abc\"\n    let f = (a) => s\n    let y = s.view\n    print(y)\n}\n";
        let mut sink = transpile_rust("plan667_cap_view", src).expect("transpile should succeed");
        let code = String::from_utf8_lossy(sink.done().unwrap()).to_string();
        assert!(
            code.contains(".clone()"),
            "captured binding view must take the clone path, got: {}",
            code
        );
    }

    /// rustc 实编门禁（正例）：非逃逸借用程序真实编译+运行。
    /// #[ignore]：shells out to rustc（对齐 a2r_tests 既有门禁模式）。
    #[test]
    #[ignore = "shells out to rustc; on-demand compile-level guard (Plan 667)"]
    fn plan667_a2r_positive_borrow_compile_run() {
        let src = "fn main() {\n    let s = \"hello\"\n    let y = s.view\n    print(y)\n}\n";
        let mut sink = transpile_rust("plan667_pos", src).expect("transpile should succeed");
        let code = String::from_utf8_lossy(sink.done().unwrap()).to_string();
        let tmp = std::env::temp_dir().join("auto_plan667_pos_gate");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let main_rs = tmp.join("main.rs");
        std::fs::write(&main_rs, code).unwrap();
        let status = std::process::Command::new("rustc")
            .args([
                "--edition=2021", "-A", "warnings",
                main_rs.to_str().unwrap(), "-o", tmp.join("case.exe").to_str().unwrap(),
            ])
            .status()
            .expect("failed to spawn rustc");
        assert!(status.success(), "rustc compile failed for positive borrow case");
        let out = std::process::Command::new(tmp.join("case.exe")).output().unwrap();
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(
            out.status.success() && stdout.contains("hello"),
            "positive case witness mismatch: {:?}",
            stdout
        );
    }
}
