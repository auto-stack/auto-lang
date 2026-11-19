//! PLAN-1106：AV 增量事件会话（V-02..V-05 的 av 侧固定用例）。
//!
//! oracle 位置（AC-06）：`SessionAudit` 在真实生产者/执行器调用边界计数
//! ——`handler_entries` 只在 `execute_program_entry_args` 真实派发成功后增长，
//! `entry_seqs` 取自执行器进程序号；读取（`read_states`）零派发、切换无
//! 派发。负控（V-06..08）在 UI 桥接线后沿同一观察面执行（T-06）。

use ace_av::component_event_sessions::{
    ComponentEventSessionProgram, ComponentFieldValue, EventOutcome,
};
use ace_core::source::age_h0::LegacyWidgetAlias;

/// M0 主例形 Counter（无参 int）。
const COUNTER_SRC: &str = "node Counter {\n    model {\n        count int = 0\n    }\n    on Inc {\n        count = count + 1\n    }\n    on Reset {\n        count = 0\n    }\n    view {\n        text \"x\" {}\n    }\n}\n";

/// int/bool/str/f32 四型字段 + int 载荷消息（V-03 支持域）。
const TYPED_SRC: &str = "node Panel {\n    msg Msg { Bump(int) }\n    model {\n        count int = 0\n        enabled bool = true\n        label str = \"init\"\n        ratio f32 = 0.5\n    }\n    on {\n        .Bump(by) -> {\n            .count = .count + by\n        }\n    }\n    on Off {\n        enabled = false\n    }\n    on SetRatio {\n        ratio = ratio + 0.25\n    }\n    view {\n        text \"x\" {}\n    }\n}\n";

/// 带 str 载荷的改名消息（borrowed 入口 + clone 局部）。
const PAYLOAD_SRC: &str = "node Editor {\n    msg Msg { Rename(str) }\n    model {\n        label str = \"init\"\n        hits int = 0\n    }\n    on {\n        .Rename(new_label) -> {\n            .label = new_label\n            .hits = .hits + 1\n        }\n    }\n    on Inc {\n        hits = hits + 1\n    }\n    view {\n        text \"x\" {}\n    }\n}\n";

/// 中途 trap 的 handler（除零）：业务零提交、旧态保留（V-04）。
const TRAP_SRC: &str = "node Trap {\n    model {\n        count int = 7\n        other int = 3\n    }\n    on Bad {\n        other = other + 1\n        count = count / (other - other)\n    }\n    on Good {\n        count = count + 1\n    }\n    view {\n        text \"x\" {}\n    }\n}\n";

fn compile(src: &str) -> ComponentEventSessionProgram {
    ComponentEventSessionProgram::compile(src, "ase_m0.counter.1106", LegacyWidgetAlias::Accept)
        .unwrap_or_else(|e| panic!("compile: {e}"))
}

fn states_of(s: &ace_av::component_event_sessions::ComponentEventSession) -> Vec<(String, String)> {
    s.read_states()
        .unwrap()
        .into_iter()
        .map(|(n, v)| (n, format!("{v:?}")))
        .collect()
}

fn field<'a>(
    s: &'a ace_av::component_event_sessions::ComponentEventSession,
    name: &str,
) -> ComponentFieldValue {
    s.read_state(name).unwrap().unwrap()
}

/// V-02：单事件增量——固定源一次编译，0/1/3/7/32 事件 handler 入口精确线性；
/// 值 3→0→1；读取/重复读取零新增入口；无按 N 展开的驱动源（compile_count 恒 1）。
#[test]
fn v02_incremental_entries_linear_and_reads_free() {
    for n in [0usize, 1, 3, 7, 32] {
        let prog = compile(COUNTER_SRC);
        assert_eq!(prog.compile_count, 1, "one compile for the fixed source");
        let mut s = prog.open_session(&[]).unwrap();
        for i in 0..n {
            s.dispatch("Inc", &[])
                .unwrap_or_else(|e| panic!("inc #{i}: {e}"));
        }
        assert_eq!(
            s.audit().handler_entries,
            n as u64,
            "exactly one handler entry per accepted event (n={n})"
        );
        let expect = if n == 0 { 0 } else { n as i32 };
        assert_eq!(field(&s, "count"), ComponentFieldValue::Int(expect));
        // 读取（单字段/整行/多次）零新增入口。
        let before = s.audit().handler_entries;
        for _ in 0..5 {
            let _ = s.read_states().unwrap();
            let _ = s.read_state("count").unwrap();
        }
        assert_eq!(
            s.audit().handler_entries,
            before,
            "reads are pure (zero dispatch)"
        );
    }
    // 3 → Reset → 0 → Inc → 1。
    let prog = compile(COUNTER_SRC);
    let mut s = prog.open_session(&[]).unwrap();
    for _ in 0..3 {
        s.dispatch("Inc", &[]).unwrap();
    }
    assert_eq!(field(&s, "count"), ComponentFieldValue::Int(3));
    s.dispatch("Reset", &[]).unwrap();
    assert_eq!(field(&s, "count"), ComponentFieldValue::Int(0));
    s.dispatch("Inc", &[]).unwrap();
    assert_eq!(field(&s, "count"), ComponentFieldValue::Int(1));
    assert_eq!(s.audit().handler_entries, 5);
    // 执行器序号与入口一一对应（真实派发证据，非 wrapper 自增）。
    assert_eq!(s.audit().entry_seqs.len(), 5 + 1 /*init*/);
}

/// V-03：A×3/B×7 同程序双会话隔离；四型字段/种子支持域；错型种子先验拒。
#[test]
fn v03_session_isolation_and_typed_support() {
    let prog = compile(COUNTER_SRC);
    let mut a = prog.open_session(&[]).unwrap();
    let mut b = prog.open_session(&[]).unwrap();
    for _ in 0..3 {
        a.dispatch("Inc", &[]).unwrap();
    }
    for _ in 0..7 {
        b.dispatch("Inc", &[]).unwrap();
    }
    assert_eq!(field(&a, "count"), ComponentFieldValue::Int(3));
    assert_eq!(field(&b, "count"), ComponentFieldValue::Int(7));
    assert_eq!(a.audit().handler_entries, 3);
    assert_eq!(b.audit().handler_entries, 7);
    assert_eq!(prog.link_count(), 2, "one link per session, zero on events");
    // 链接后 A/B 互不影响（B 再事件不动 A）。
    b.dispatch("Inc", &[]).unwrap();
    assert_eq!(field(&a, "count"), ComponentFieldValue::Int(3));

    // 四型字段 + 种子。
    let tprog = compile(TYPED_SRC);
    let mut t = tprog
        .open_session(&[
            ("count".to_string(), ComponentFieldValue::Int(10)),
            (
                "label".to_string(),
                ComponentFieldValue::Str("seeded".to_string()),
            ),
            ("ratio".to_string(), ComponentFieldValue::F32(1.5)),
        ])
        .unwrap();
    assert_eq!(field(&t, "count"), ComponentFieldValue::Int(10));
    assert_eq!(field(&t, "enabled"), ComponentFieldValue::Bool(true));
    assert_eq!(
        field(&t, "label"),
        ComponentFieldValue::Str("seeded".to_string())
    );
    assert_eq!(field(&t, "ratio"), ComponentFieldValue::F32(1.5));
    t.dispatch("Off", &[]).unwrap();
    t.dispatch("SetRatio", &[]).unwrap();
    t.dispatch("Bump", &[ComponentFieldValue::Int(5)]).unwrap();
    assert_eq!(field(&t, "enabled"), ComponentFieldValue::Bool(false));
    assert_eq!(field(&t, "ratio"), ComponentFieldValue::F32(1.75));
    assert_eq!(field(&t, "count"), ComponentFieldValue::Int(15));

    // 错型种子：效果前具名拒（零链接——会话不开）。
    let err = match tprog.open_session(&[("count".to_string(), ComponentFieldValue::Bool(true))]) {
        Ok(_) => panic!("type-mismatched seed must be refused"),
        Err(e) => e,
    };
    assert!(err.contains("component-initial-type"), "{err}");

    // 未知消息/错 arity/错载荷类型：具名拒 + 零入口增长。
    let before = t.audit().handler_entries;
    assert!(t
        .dispatch("Nope", &[])
        .unwrap_err()
        .contains("component-event"));
    assert!(t
        .dispatch("Bump", &[])
        .unwrap_err()
        .contains("component-event"));
    assert!(t
        .dispatch("Bump", &[ComponentFieldValue::Str("x".to_string())])
        .unwrap_err()
        .contains("type-mismatch"));
    assert_eq!(t.audit().handler_entries, before);
    assert_eq!(t.audit().rejected, 3);
}

/// V-03b：str 载荷（borrowed 入口 + clone 局部）真实绑定。
#[test]
fn v03b_str_payload_binding() {
    let prog = compile(PAYLOAD_SRC);
    let mut s = prog.open_session(&[]).unwrap();
    s.dispatch("Rename", &[ComponentFieldValue::Str("hello".to_string())])
        .unwrap();
    assert_eq!(
        field(&s, "label"),
        ComponentFieldValue::Str("hello".to_string())
    );
    assert_eq!(field(&s, "hits"), ComponentFieldValue::Int(1));
    // 第二次（不同 str）再绑定。
    s.dispatch("Rename", &[ComponentFieldValue::Str("world".to_string())])
        .unwrap();
    assert_eq!(
        field(&s, "label"),
        ComponentFieldValue::Str("world".to_string())
    );
    assert_eq!(field(&s, "hits"), ComponentFieldValue::Int(2));
    assert_eq!(s.audit().handler_entries, 2);
}

/// V-04：中途 trap 零提交（旧态保留）；序号重投不重复执行；两个合法同名
/// Inc 各执行一次；关闭后事件具名拒。
#[test]
fn v04_trap_rollback_seq_watermark_and_close() {
    let prog = compile(TRAP_SRC);
    let mut s = prog.open_session(&[]).unwrap();
    assert_eq!(field(&s, "count"), ComponentFieldValue::Int(7));
    // Bad 在 other+1 后除零 trap：整次失败、无半状态（other 回 3）。
    let err = s.dispatch("Bad", &[]).unwrap_err();
    assert!(err.contains("rolled back"), "{err}");
    assert_eq!(field(&s, "count"), ComponentFieldValue::Int(7));
    assert_eq!(
        field(&s, "other"),
        ComponentFieldValue::Int(3),
        "no half-state"
    );
    assert_eq!(s.audit().handler_entries, 0);
    // 失败后合法事件正常处理（不回放历史恢复）。
    s.dispatch("Good", &[]).unwrap();
    assert_eq!(field(&s, "count"), ComponentFieldValue::Int(8));

    // 序号水位：同 seq 重投不重复执行（返回 Duplicate + 上次状态）；
    // 新 seq 的同名合法事件各执行一次。
    let mut c = compile(COUNTER_SRC);
    let _ = &mut c;
    let prog2 = compile(COUNTER_SRC);
    let mut s2 = prog2.open_session(&[]).unwrap();
    s2.dispatch_seq(Some(1), "Inc", &[]).unwrap();
    s2.dispatch_seq(Some(2), "Inc", &[]).unwrap();
    assert_eq!(field(&s2, "count"), ComponentFieldValue::Int(2));
    // 重投 seq=1（已成功）：不重复执行。
    match s2.dispatch_seq(Some(1), "Inc", &[]).unwrap() {
        EventOutcome::Duplicate => {}
        other => panic!("expected Duplicate, got {other:?}"),
    }
    assert_eq!(field(&s2, "count"), ComponentFieldValue::Int(2));
    assert_eq!(s2.audit().handler_entries, 2);
    assert_eq!(s2.audit().duplicate_ignored, 1);
    // 同名合法新事件（新 seq）不去重。
    s2.dispatch_seq(Some(3), "Inc", &[]).unwrap();
    assert_eq!(field(&s2, "count"), ComponentFieldValue::Int(3));
    assert_eq!(s2.audit().handler_entries, 3);

    // 关闭：事件具名拒；关闭后读取具名拒。
    s2.close();
    assert!(s2.dispatch("Inc", &[]).unwrap_err().contains("closed"));
    assert!(s2.read_states().is_err());
    assert!(s2.audit().released);
}

/// V-05：关开 20 轮——每轮新代次、init 恰一次、旧 handler 不再进入；程序级
/// 链接计数线性、编译计数恒 1。
#[test]
fn v05_close_reopen_20_rounds_bounded() {
    let prog = compile(COUNTER_SRC);
    assert_eq!(prog.compile_count, 1);
    let mut generations = Vec::new();
    for round in 0..20 {
        let mut s = prog.open_session(&[]).unwrap();
        assert_eq!(s.audit().init_runs, 1, "init exactly once per generation");
        for _ in 0..(round % 3) {
            s.dispatch("Inc", &[]).unwrap();
        }
        let g = s.generation();
        s.close();
        assert!(s.audit().released);
        generations.push(g);
    }
    assert_eq!(prog.link_count(), 20, "one link per open, none on events");
    assert_eq!(prog.compile_count, 1, "no recompile across 20 generations");
    // 代次严格递增。
    let mut sorted = generations.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), 20, "each reopen is a new generation");
}

/// V-02 文本面：文本叶纯读求值（不执行业务 handler）；states-only 边界内
/// 的 f-string 源状态面仍可用（文本面具名不可用）。
#[test]
fn v02b_text_leaves_are_pure_reads() {
    // 静态文本叶（非 f-string）：文本可用，且 text_evals 不算业务入口。
    let prog = compile(COUNTER_SRC);
    let mut s = prog.open_session(&[]).unwrap();
    s.dispatch("Inc", &[]).unwrap();
    let before = s.audit().handler_entries;
    let texts = s.read_texts().unwrap();
    assert!(!texts.is_empty());
    assert_eq!(
        s.audit().handler_entries,
        before,
        "text evals are not business handlers"
    );
    assert!(s.audit().text_evals >= 1);

    // f-string 叶（states-only 边界）：状态面照常、文本面具名不可用。
    let fsrc = "node F {\n    model {\n        count int = 0\n    }\n    on Inc {\n        count = count + 1\n    }\n    view {\n        text `n ${count}` {}\n    }\n}\n";
    let fprog =
        ComponentEventSessionProgram::compile(fsrc, "ase_m0.f.1106", LegacyWidgetAlias::Accept)
            .unwrap();
    assert!(
        !fprog.texts_available,
        "f-string leaves stay on the states-only boundary"
    );
    assert!(fprog.texts_unavailable_reason.is_some());
    let mut fs = fprog.open_session(&[]).unwrap();
    fs.dispatch("Inc", &[]).unwrap();
    assert_eq!(field(&fs, "count"), ComponentFieldValue::Int(1));
    assert!(fs.read_texts().unwrap_err().contains("unavailable"));
    // 边界内的编译次数留痕（重试一次）。
    assert_eq!(fprog.compile_count, 2);
}
