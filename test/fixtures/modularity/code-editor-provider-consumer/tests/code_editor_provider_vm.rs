//! PLAN-802 V-01/V-02/V-03：外部 code editor host provider 注入真 VM 断言。
//!
//! 覆盖（计划 §6 矩阵 + T-01 冻结矩阵）：
//! - V-01 真注入（no-ui 全管线）：fake provider 实值经 VM 内建 22 native
//!   到达 stdout（可区分 Unicode 键/文本/delta JSON/光标三元组/滚动偏移）；
//!   操作收精确参数（edit 区间、fold 1基→0基钳位、set_cursor 负参钳 0、
//!   scroll_to 双轴值）；provider 实现零 UI 类型（独立 manifest 证明）。
//! - V-02 缺席矩阵：21 项 raise 原 feature-off 文案逐字面值（含 cut/copy/
//!   paste 组合特征文案）；9913 scroll_to 唯一值缺席臂（弹参返 false 非
//!   raise）；ID 钉扎（22 canonical 名 + 9907 不占用）；Clone/merge 保
//!   provider；注入不 bump binding_revision；if_absent 不覆盖显式默认。
//! - V-03 delta/ownership：delta 每次 native 调用恰一次消费（FIFO 保序，
//!   空队列回水位哨兵——真编辑器形），text 不清队列；provider 替换后新
//!   VM 快照新值；VM 字符串池内容在 provider 替换后仍有效（shim 物化
//!   owned bytes）；重复调用清栈回基线。
//!
//! 进程默认槽是进程级全局——所有触槽测试经 `serialized()` 互斥串行并做
//! save/restore（F-785-R1-01 同款纪律）。本 crate 无 UI feature——不存在
//! 任何隐式装配路径，清槽即全局缺席（裸 `AutoVM::new` 即构造式缺席锚）。

use std::collections::VecDeque;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

use auto_lang::vm::engine::AutoVM;
use auto_lang::vm::host_capabilities::{
    clear_process_default_code_editor_host_provider, process_default_code_editor_host_provider,
    set_process_default_code_editor_host_provider,
    set_process_default_code_editor_host_provider_if_absent, CodeEditorCursorDto,
    CodeEditorHostProvider, CodeEditorScrollOffsetDto,
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
struct DefaultGuard(Option<Arc<dyn CodeEditorHostProvider>>);

impl DefaultGuard {
    fn new() -> Self {
        Self(process_default_code_editor_host_provider())
    }
}

impl Drop for DefaultGuard {
    fn drop(&mut self) {
        match self.0.take() {
            Some(p) => set_process_default_code_editor_host_provider(p),
            None => clear_process_default_code_editor_host_provider(),
        }
    }
}

// ── fake provider（可区分哨兵；零 UI 类型；全调用记录） ──────────────────

struct FakeEditor {
    /// 每次 provider 调用的方法名+参数摘要（精确断言用）。
    calls: Mutex<Vec<String>>,
    /// text() 返回值。
    text: String,
    /// delta FIFO 序列（依次弹出；耗尽后回水位哨兵——真编辑器"空队列返
    /// 当前水位空 deltas"形，None 仅表无此键编辑器）。
    delta_fifo: Mutex<VecDeque<String>>,
    /// delta() 调用计数（恰一次消费断言）。
    delta_calls: Mutex<usize>,
    /// cursor() 返回（None = 无此键）。
    cursor: Option<CodeEditorCursorDto>,
    /// load_file 命中表（key -> Some(bytes) / None）。
    load_map: std::collections::HashMap<String, Option<i64>>,
    /// save 命中表（key -> bool）。
    save_map: std::collections::HashMap<String, bool>,
    /// scroll_offset 返回值。
    scroll: CodeEditorScrollOffsetDto,
    /// scroll_to 记录 (key, x, y)。
    scroll_to_calls: Mutex<Vec<(String, f64, f64)>>,
    /// cut/copy/paste 回供（None = 宿主无剪贴板面）。
    clip_reply: Option<bool>,
}

impl FakeEditor {
    fn sentinel() -> Arc<Self> {
        let mut load_map = std::collections::HashMap::new();
        load_map.insert("键-装-🎉".to_string(), Some(123_456));
        load_map.insert("键-缺".to_string(), None);
        let mut save_map = std::collections::HashMap::new();
        save_map.insert("键-存".to_string(), true);
        save_map.insert("键-拒".to_string(), false);
        Arc::new(Self {
            calls: Mutex::new(Vec::new()),
            text: "文本-p802-🎉".to_string(),
            delta_fifo: Mutex::new(VecDeque::from(vec![
                r#"{"revision":7,"deltas":[]}"#.to_string(),
                r#"{"revision":8,"deltas":[{"start":1,"end":2,"replacement":"乙"}]}"#.to_string(),
            ])),
            delta_calls: Mutex::new(0),
            cursor: Some(CodeEditorCursorDto {
                line: 3,
                char_col: 11,
                selection_bytes: 29,
            }),
            load_map,
            save_map,
            scroll: CodeEditorScrollOffsetDto { x: 6.5, y: 13.75 },
            scroll_to_calls: Mutex::new(Vec::new()),
            clip_reply: Some(true),
        })
    }

    fn record(&self, entry: String) {
        self.calls.lock().unwrap().push(entry);
    }

    fn calls_snapshot(&self) -> Vec<String> {
        self.calls.lock().unwrap().clone()
    }

    fn delta_call_count(&self) -> usize {
        *self.delta_calls.lock().unwrap()
    }
}

impl CodeEditorHostProvider for FakeEditor {
    fn text(&self, key: &str) -> Option<String> {
        self.record(format!("text({key:?})"));
        Some(self.text.clone())
    }
    fn delta(&self, key: &str) -> Option<String> {
        self.record(format!("delta({key:?})"));
        *self.delta_calls.lock().unwrap() += 1;
        self.delta_fifo
            .lock()
            .unwrap()
            .pop_front()
            .or_else(|| Some(r#"{"revision":8,"deltas":[]}"#.to_string()))
    }
    fn edit(&self, key: &str, start: usize, end: usize, replacement: &str) -> bool {
        self.record(format!("edit({key:?},{start},{end},{replacement:?})"));
        true
    }
    fn cursor(&self, key: &str) -> Option<CodeEditorCursorDto> {
        self.record(format!("cursor({key:?})"));
        self.cursor
    }
    fn find(&self, key: &str) -> bool {
        self.record(format!("find({key:?})"));
        true
    }
    fn set_text(&self, key: &str, text: &str) -> bool {
        self.record(format!("set_text({key:?},{text:?})"));
        true
    }
    fn load_file(&self, key: &str, path: &str) -> Option<i64> {
        self.record(format!("load_file({key:?},{path:?})"));
        self.load_map.get(key).cloned().flatten()
    }
    fn save(&self, key: &str, path: &str) -> bool {
        self.record(format!("save({key:?},{path:?})"));
        self.save_map.get(key).copied().unwrap_or(false)
    }
    fn set_cursor(&self, key: &str, line: usize, char_col: usize) -> bool {
        self.record(format!("set_cursor({key:?},{line},{char_col})"));
        true
    }
    fn scroll_offset(&self, key: &str) -> CodeEditorScrollOffsetDto {
        self.record(format!("scroll_offset({key:?})"));
        self.scroll
    }
    fn scroll_to(&self, key: &str, x: f64, y: f64) -> bool {
        self.record(format!("scroll_to({key:?},{x},{y})"));
        self.scroll_to_calls
            .lock()
            .unwrap()
            .push((key.to_string(), x, y));
        true
    }
    fn fold_toggle(&self, key: &str, line_0: usize) -> bool {
        self.record(format!("fold_toggle({key:?},{line_0})"));
        true
    }
    fn fold_hidden_count(&self, key: &str) -> usize {
        self.record(format!("fold_hidden_count({key:?})"));
        42
    }
    fn undo(&self, key: &str) -> bool {
        self.record(format!("undo({key:?})"));
        true
    }
    fn redo(&self, key: &str) -> bool {
        self.record(format!("redo({key:?})"));
        false
    }
    fn select_all(&self, key: &str) -> bool {
        self.record(format!("select_all({key:?})"));
        true
    }
    fn cut(&self, key: &str) -> Option<bool> {
        self.record(format!("cut({key:?})"));
        self.clip_reply
    }
    fn copy(&self, key: &str) -> Option<bool> {
        self.record(format!("copy({key:?})"));
        self.clip_reply
    }
    fn paste(&self, key: &str) -> Option<bool> {
        self.record(format!("paste({key:?})"));
        self.clip_reply
    }
}

/// 非 sentinel 显式默认哨兵（if_absent 优先关系断言用）。
struct ExplicitOther;

impl CodeEditorHostProvider for ExplicitOther {
    fn text(&self, _k: &str) -> Option<String> {
        Some("@explicit".to_string())
    }
    fn delta(&self, _k: &str) -> Option<String> {
        None
    }
    fn edit(&self, _k: &str, _s: usize, _e: usize, _r: &str) -> bool {
        false
    }
    fn cursor(&self, _k: &str) -> Option<CodeEditorCursorDto> {
        None
    }
    fn find(&self, _k: &str) -> bool {
        false
    }
    fn set_text(&self, _k: &str, _t: &str) -> bool {
        false
    }
    fn load_file(&self, _k: &str, _p: &str) -> Option<i64> {
        None
    }
    fn save(&self, _k: &str, _p: &str) -> bool {
        false
    }
    fn set_cursor(&self, _k: &str, _l: usize, _c: usize) -> bool {
        false
    }
    fn scroll_offset(&self, _k: &str) -> CodeEditorScrollOffsetDto {
        CodeEditorScrollOffsetDto::default()
    }
    fn scroll_to(&self, _k: &str, _x: f64, _y: f64) -> bool {
        false
    }
    fn fold_toggle(&self, _k: &str, _l: usize) -> bool {
        false
    }
    fn fold_hidden_count(&self, _k: &str) -> usize {
        0
    }
    fn undo(&self, _k: &str) -> bool {
        false
    }
    fn redo(&self, _k: &str) -> bool {
        false
    }
    fn select_all(&self, _k: &str) -> bool {
        false
    }
    fn cut(&self, _k: &str) -> Option<bool> {
        None
    }
    fn copy(&self, _k: &str) -> Option<bool> {
        None
    }
    fn paste(&self, _k: &str) -> Option<bool> {
        None
    }
}

fn stdout_lines(s: &str) -> Vec<&str> {
    s.lines().collect()
}

// ── V-01 真注入（no-ui 全管线） ──────────────────────────────────────────

/// V-01(a) 全管线：进程默认 fake → 22 函数实值经真 VM stdout 到达 + 精确
/// 参数（负钳位/1基转换/双轴顺序/Unicode）。
#[test]
fn v01_process_default_reaches_real_vm_stdout() {
    let _g = serialized();
    let _guard = DefaultGuard::new();

    let fake = FakeEditor::sentinel();
    set_process_default_code_editor_host_provider(fake.clone());

    // 读族：text/cursor 三元组（互异值）/find/scroll 双读/delta FIFO。
    let prog = r#"
let t = code_editor_text("键-读-🎉")
print(t)
let l = code_editor_cursor_line("键-读-🎉")
let c = code_editor_cursor_col("键-读-🎉")
let s = code_editor_selection_len("键-读-🎉")
print(l)
print(c)
print(s)
let f = code_editor_find("键-读-🎉")
print(f)
let ox = code_editor_scroll_offset_x("键-读")
let oy = code_editor_scroll_offset_y("键-读")
print(ox)
print(oy)
let d1 = code_editor_delta("键-读-🎉")
print(d1)
let d2 = code_editor_delta("键-读-🎉")
print(d2)
let h = code_editor_fold_hidden_count("键-读")
print(h)
"#;
    let (_, out) = auto_lang::run_with_capture(prog).expect("run reads");
    assert_eq!(
        stdout_lines(&out),
        vec![
            "文本-p802-🎉",
            "3",
            "11",
            "29",
            "true",
            "6.5",
            "13.75",
            r#"{"revision":7,"deltas":[]}"#,
            r#"{"revision":8,"deltas":[{"start":1,"end":2,"replacement":"乙"}]}"#,
            "42",
        ],
        "读族实值到达 stdout（got {out:?}）"
    );

    // 写族（程序一）：set_text（Void 无 push——后续语句照常）/edit/
    // set_cursor 负钳/load_file 值语义（命中 bytes / 缺键 -1）/save 真假。
    // 程序拆两段：单作用域局部槽索引 ≥16 为预存 VM 4-bit 编码限制
    //（与 provider 迁移无关，KNOWN-DEBT 候选登记），每段 ≤13 局部。
    let prog = r#"
code_editor_set_text("键-写", "新文本-🎯")
print("after-set-text")
let e = code_editor_edit("键-写-🎉", 2, 7, "替换-🎯")
print(e)
let sc = code_editor_set_cursor("键-写", -3, 9)
print(sc)
let sc2 = code_editor_set_cursor("键-写", 4, -1)
print(sc2)
let n1 = code_editor_load_file("键-装-🎉", "C:/tmp/文 件.txt")
print(n1)
let n2 = code_editor_load_file("键-缺", "C:/nope")
print(n2)
let b1 = code_editor_save("键-存", "C:/out.txt")
let b2 = code_editor_save("键-拒", "C:/out.txt")
print(b1)
print(b2)
"#;
    let (_, out) = auto_lang::run_with_capture(prog).expect("run writes-1");
    assert_eq!(
        stdout_lines(&out),
        vec![
            "after-set-text",
            "true",   // edit
            "true",   // set_cursor(-3,9)
            "true",   // set_cursor(4,-1)
            "123456", // load 命中
            "-1",     // load 缺键 → 值非 raise
            "true",   // save 命中
            "false",  // save 拒
        ],
        "写族程序一实值到达 stdout（got {out:?}）"
    );

    // 写族（程序二）：scroll_to 双轴/fold 1基→0基/menu 族/cut-copy-paste
    // Some 值。
    let prog = r#"
let st = code_editor_scroll_to("键-写", 3.25, 9.5)
print(st)
let ft1 = code_editor_fold_toggle("键-写", 5)
let ft2 = code_editor_fold_toggle("键-写", 0)
let ft3 = code_editor_fold_toggle("键-写", -7)
print(ft1)
print(ft2)
print(ft3)
let u = code_editor_undo("键-写")
let r = code_editor_redo("键-写")
let sa = code_editor_select_all("键-写")
print(u)
print(r)
print(sa)
let cut = code_editor_cut("键-写")
let cp = code_editor_copy("键-写")
let pa = code_editor_paste("键-写")
print(cut)
print(cp)
print(pa)
"#;
    let (_, out) = auto_lang::run_with_capture(prog).expect("run writes-2");
    assert_eq!(
        stdout_lines(&out),
        vec![
            "true",  // scroll_to
            "true",  // fold(5)
            "true",  // fold(0)
            "true",  // fold(-7)
            "true",  // undo
            "false", // redo（fake 回供互异）
            "true",  // select_all
            "true",  // cut Some(true)
            "true",  // copy
            "true",  // paste
        ],
        "写族程序二实值到达 stdout（got {out:?}）"
    );

    // 精确参数断言（弹序/钳位/转换/Unicode 编码保持）。
    let calls = fake.calls_snapshot();
    assert!(
        calls.contains(&r#"edit("键-写-🎉",2,7,"替换-🎯")"#.to_string()),
        "edit 收到精确 (key,start,end,replacement)（got {calls:?}）"
    );
    assert!(
        calls.contains(&r#"set_text("键-写","新文本-🎯")"#.to_string()),
        "set_text 收到精确 (key,text)"
    );
    assert!(
        calls.contains(&"set_cursor(\"键-写\",0,9)".to_string()),
        "set_cursor 负 line 钳 0（got {calls:?}）"
    );
    assert!(
        calls.contains(&"set_cursor(\"键-写\",4,0)".to_string()),
        "set_cursor 负 col 钳 0（got {calls:?}）"
    );
    assert!(
        calls.contains(&"load_file(\"键-装-🎉\",\"C:/tmp/文 件.txt\")".to_string()),
        "load_file 收到精确 (key,path) Unicode（got {calls:?}）"
    );
    assert!(
        calls.contains(&"fold_toggle(\"键-写\",4)".to_string())
            && calls.contains(&"fold_toggle(\"键-写\",0)".to_string()),
        "fold 1基→0基（5→4；0/-7→max(1)-1=0）（got {calls:?}）"
    );
    assert_eq!(
        fake.scroll_to_calls.lock().unwrap().last().cloned(),
        Some(("键-写".to_string(), 3.25, 9.5)),
        "scroll_to 收到精确双轴值"
    );
    // cursor 三读各一次 provider 调用（与迁移前 code_editor_cursor 单调用
    // 同构——T-01 冻结）。
    assert_eq!(
        calls.iter().filter(|c| c.starts_with("cursor(")).count(),
        3,
        "cursor 三读各一次 provider 调用（got {calls:?}）"
    );
}

/// V-01(b) 低层双活实例：构造期快照 + 两 VM 不串线 + 实例覆盖独立
///（直调 shim，不经程序编译）。
#[test]
fn v01_two_live_instances_do_not_cross_talk() {
    use auto_lang::vm::native::shim_code_editor_undo;

    let _g = serialized();
    let _guard = DefaultGuard::new();

    let a = FakeEditor::sentinel();
    let b = FakeEditor::sentinel();
    let c = FakeEditor::sentinel();

    set_process_default_code_editor_host_provider(a.clone());
    let vm1 = AutoVM::new(VirtualFlash::new(0), 64); // 快照 A
    set_process_default_code_editor_host_provider(b.clone());
    let vm2 = AutoVM::new(VirtualFlash::new(0), 64); // 快照 B

    let mut task1 = AutoTask::new(0, 256, 0);
    let idx = vm1.add_string("@vm1-键".as_bytes().to_vec());
    vm1.rc_push_str_idx(&mut task1, idx);
    shim_code_editor_undo(&mut task1, &vm1).unwrap();
    assert!(auto_val::decode_bool(task1.ram.pop_nv()));
    assert_eq!(a.calls_snapshot(), vec![r#"undo("@vm1-键")"#.to_string()]);
    assert!(b.calls_snapshot().is_empty(), "vm1 不得串线到默认 B");

    // 实例覆盖 C：只影响 vm2。
    vm2.set_code_editor_provider(c.clone());
    let mut task2 = AutoTask::new(0, 256, 0);
    let idx = vm2.add_string("@vm2-键".as_bytes().to_vec());
    vm2.rc_push_str_idx(&mut task2, idx);
    shim_code_editor_undo(&mut task2, &vm2).unwrap();
    assert!(auto_val::decode_bool(task2.ram.pop_nv()));
    assert_eq!(c.calls_snapshot(), vec![r#"undo("@vm2-键")"#.to_string()]);
    assert!(
        a.calls_snapshot().len() == 1 && b.calls_snapshot().is_empty(),
        "vm2 覆盖不影响 vm1/默认"
    );
    // 进程默认不被实例覆盖改写。
    let cur = process_default_code_editor_host_provider().unwrap();
    CodeEditorHostProvider::undo(&*cur, "@default-键");
    assert_eq!(b.calls_snapshot().len(), 1, "进程默认仍=B");
}

// ── V-02 缺席矩阵（T-01 冻结缺席臂逐字面值） ────────────────────────────

/// 缺席=21 项 raise 原 feature-off 文案逐字；9913 scroll_to 弹参返 false
///（唯一值缺席臂）。清槽即全局缺席（no-ui 无隐式装配路径——裸 new 即
/// 构造式缺席锚）。
#[test]
fn v02_absent_capability_is_frozen_absence_arm() {
    let _g = serialized();
    let _guard = DefaultGuard::new();
    clear_process_default_code_editor_host_provider();

    // 21 项 raise：程序轨 Err 文本逐字（错误是唯一可观察面——栈随 unwind
    // 弃置）。
    let raise_cases: &[(&str, &str)] = &[
        (
            r#"let x = code_editor_text("k")"#,
            "code_editor_text: the `code-editor` feature is disabled",
        ),
        (
            r#"let x = code_editor_cursor_line("k")"#,
            "code_editor_cursor_line: the `code-editor` feature is disabled",
        ),
        (
            r#"let x = code_editor_cursor_col("k")"#,
            "code_editor_cursor_col: the `code-editor` feature is disabled",
        ),
        (
            r#"let x = code_editor_selection_len("k")"#,
            "code_editor_selection_len: the `code-editor` feature is disabled",
        ),
        (
            r#"let x = code_editor_find("k")"#,
            "code_editor_find: the `code-editor` feature is disabled",
        ),
        (
            r#"code_editor_set_text("k", "t")"#,
            "code_editor_set_text: the `code-editor` feature is disabled",
        ),
        (
            r#"let x = code_editor_undo("k")"#,
            "code_editor_undo: the `code-editor` feature is disabled",
        ),
        (
            r#"let x = code_editor_redo("k")"#,
            "code_editor_redo: the `code-editor` feature is disabled",
        ),
        (
            r#"let x = code_editor_select_all("k")"#,
            "code_editor_select_all: the `code-editor` feature is disabled",
        ),
        (
            r#"let x = code_editor_cut("k")"#,
            "code_editor_cut: the `code-editor`+`ui-clipboard` features are required",
        ),
        (
            r#"let x = code_editor_copy("k")"#,
            "code_editor_copy: the `code-editor`+`ui-clipboard` features are required",
        ),
        (
            r#"let x = code_editor_paste("k")"#,
            "code_editor_paste: the `code-editor`+`ui-clipboard` features are required",
        ),
        (
            r#"let x = code_editor_fold_toggle("k", 1)"#,
            "code_editor_fold_toggle: the `code-editor` feature is disabled",
        ),
        (
            r#"let x = code_editor_fold_hidden_count("k")"#,
            "code_editor_fold_hidden_count: the `code-editor` feature is disabled",
        ),
        (
            r#"let x = code_editor_delta("k")"#,
            "code_editor_delta: the `code-editor` feature is disabled",
        ),
        (
            r#"let x = code_editor_edit("k", 0, 1, "a")"#,
            "code_editor_edit: the `code-editor` feature is disabled",
        ),
        (
            r#"let x = code_editor_load_file("k", "p")"#,
            "code_editor_load_file: the `code-editor` feature is disabled",
        ),
        (
            r#"let x = code_editor_save("k", "p")"#,
            "code_editor_save: the `code-editor` feature is disabled",
        ),
        (
            r#"let x = code_editor_set_cursor("k", 0, 0)"#,
            "code_editor_set_cursor: the `code-editor` feature is disabled",
        ),
        (
            r#"let x = code_editor_scroll_offset_x("k")"#,
            "code_editor_scroll_offset_x: the `code-editor` feature is disabled",
        ),
        (
            r#"let x = code_editor_scroll_offset_y("k")"#,
            "code_editor_scroll_offset_y: the `code-editor` feature is disabled",
        ),
    ];
    for (prog, expected) in raise_cases {
        let err = auto_lang::run(prog)
            .err()
            .unwrap_or_else(|| panic!("缺席须 raise：{prog}"));
        let text = format!("{err:?}");
        assert!(
            text.contains(expected),
            "缺席错误文本逐字保持（want …{expected:?}，got {text:?}）"
        );
    }

    // 9913：唯一值缺席臂——弹 3 参、push false、程序正常完成。
    let (_, out) = auto_lang::run_with_capture(
        "let b = code_editor_scroll_to(\"k\", 1.5, 2.5)\nprint(b)\nprint(\"done\")\n",
    )
    .expect("scroll_to 缺席为值臂非 raise");
    assert_eq!(
        stdout_lines(&out),
        vec!["false", "done"],
        "缺席 scroll_to=false 且程序继续（got {out:?}）"
    );

    // 裸 VM 构造式缺席锚（no-ui 进程无装配路径——new 即无快照源）。
    let vm = AutoVM::new(VirtualFlash::new(0), 64);
    assert!(
        vm.code_editor_provider().is_none(),
        "no-ui 裸 VM 无 provider"
    );
}

/// ID 钉扎（22 canonical 名双射 + 9907 不占用）+ Clone/merge 保 provider +
/// 注入不 bump binding_revision + if_absent 不覆盖显式默认。
#[test]
fn v02_identity_clone_merge_rules_preserved() {
    let ni = NativeInterface::production();
    let pinned: &[(&str, u16)] = &[
        ("auto.code_editor.text", 2910),
        ("auto.code_editor.cursor_line", 2911),
        ("auto.code_editor.cursor_col", 2912),
        ("auto.code_editor.selection_len", 2913),
        ("auto.code_editor.find", 2914),
        ("auto.code_editor.set_text", 2915),
        ("auto.code_editor.undo", 2919),
        ("auto.code_editor.redo", 2920),
        ("auto.code_editor.select_all", 2921),
        ("auto.code_editor.cut", 2922),
        ("auto.code_editor.copy", 2923),
        ("auto.code_editor.paste", 2924),
        ("auto.code_editor.fold_toggle", 2932),
        ("auto.code_editor.fold_hidden_count", 2933),
        ("auto.code_editor.delta", 2939),
        ("auto.code_editor.edit", 9906),
        ("auto.code_editor.load_file", 9908),
        ("auto.code_editor.save", 9909),
        ("auto.code_editor.set_cursor", 9910),
        ("auto.code_editor.scroll_offset_x", 9911),
        ("auto.code_editor.scroll_offset_y", 9912),
        ("auto.code_editor.scroll_to", 9913),
    ];
    for (name, id) in pinned {
        assert_eq!(ni.resolve(name), Some(*id), "ID 钉扎：{name}");
    }
    // 9907 归 Env.track（P673-D2），不得被本族占用。
    assert_ne!(ni.resolve("auto.code_editor.edit"), Some(9907));

    let rev0 = ni.binding_revision();
    ni.set_code_editor_provider(FakeEditor::sentinel());
    assert_eq!(
        ni.binding_revision(),
        rev0,
        "provider 注入不 bump binding_revision"
    );
    let clone = ni.clone();
    assert!(
        clone.code_editor_provider().is_some(),
        "NI Clone 不丢 provider"
    );

    let mut vm = AutoVM::new(VirtualFlash::new(0), 64);
    vm.set_code_editor_provider(FakeEditor::sentinel());
    let mut carrier = NativeInterface::new();
    carrier.register_dynamic(|_t: &mut AutoTask, _v: &AutoVM| Ok(()));
    vm.merge_native_interface(&carrier);
    assert!(
        vm.code_editor_provider().is_some(),
        "VM merge 路径不丢 provider"
    );
}

/// 兼容安装优先关系（无 UI 构建下直接验 host_capabilities 槽语义）：显式
/// set 恒胜 if_absent；空槽才补缺。
#[test]
fn v02_if_absent_respects_explicit_default() {
    let _g = serialized();
    let _guard = DefaultGuard::new();

    clear_process_default_code_editor_host_provider();
    let a = FakeEditor::sentinel();
    // 空槽 → 补缺。
    set_process_default_code_editor_host_provider_if_absent(a.clone());
    assert_eq!(
        process_default_code_editor_host_provider().map(|p| CodeEditorHostProvider::text(&*p, "k")),
        Some(Some("文本-p802-🎉".to_string())),
        "空槽补缺成功"
    );
    // 显式默认 → if_absent 不覆盖（哨兵可区分）。
    set_process_default_code_editor_host_provider(Arc::new(ExplicitOther));
    set_process_default_code_editor_host_provider_if_absent(FakeEditor::sentinel());
    assert_eq!(
        process_default_code_editor_host_provider().map(|p| CodeEditorHostProvider::text(&*p, "k")),
        Some(Some("@explicit".to_string())),
        "兼容安装不得覆盖显式默认"
    );
}

// ── V-03 delta/ownership 纪律 ────────────────────────────────────────────

/// delta 每次 native 调用恰一次消费（FIFO 保序，空队列回水位哨兵）；
/// text 不清队列（调用次数与序不受穿插影响）。
#[test]
fn v03_delta_exactly_once_fifo_and_text_no_drain() {
    let _g = serialized();
    let _guard = DefaultGuard::new();

    let fake = FakeEditor::sentinel();
    set_process_default_code_editor_host_provider(fake.clone());

    let prog = r#"
let t1 = code_editor_text("键")
let d1 = code_editor_delta("键")
let t2 = code_editor_text("键")
let d2 = code_editor_delta("键")
let d3 = code_editor_delta("键")
print(t1)
print(d1)
print(d2)
print(d3)
"#;
    let (_, out) = auto_lang::run_with_capture(prog).expect("run fifo");
    assert_eq!(
        stdout_lines(&out),
        vec![
            "文本-p802-🎉",
            r#"{"revision":7,"deltas":[]}"#,
            r#"{"revision":8,"deltas":[{"start":1,"end":2,"replacement":"乙"}]}"#,
            r#"{"revision":8,"deltas":[]}"#,
        ],
        "FIFO 保序 + 空队列水位哨兵（got {out:?}）"
    );
    // 恰一次：3 次 native delta = 3 次 provider delta；2 次 text 独立计数。
    assert_eq!(
        fake.delta_call_count(),
        3,
        "delta 每次 native 调用恰一次消费"
    );
    assert_eq!(
        fake.calls_snapshot()
            .iter()
            .filter(|c| c.starts_with("text("))
            .count(),
        2,
        "text 调用次数不受 delta 穿插影响"
    );
}

/// ownership：VM 字符串池内容在 provider 替换/释放后仍有效（shim 物化
/// owned bytes——`add_string(text.into_bytes())` 拷贝语义）；provider 替换
/// 后新 VM 快照新值；重复直调清栈回基线。
#[test]
fn v03_vm_string_outlives_provider_replacement() {
    use auto_lang::vm::native::shim_code_editor_text;

    let _g = serialized();
    let _guard = DefaultGuard::new();

    let fake = FakeEditor::sentinel();
    set_process_default_code_editor_host_provider(fake.clone());
    let vm = AutoVM::new(VirtualFlash::new(0), 256);

    let mut task = AutoTask::new(0, 512, 0);
    let sp0 = task.ram.sp;
    let idx_key = vm.add_string("键".as_bytes().to_vec());
    vm.rc_push_str_idx(&mut task, idx_key);
    shim_code_editor_text(&mut task, &vm).unwrap();
    // 取得 VM 字符串索引并弹出（模拟消费）。
    let nv = task.ram.pop_nv();
    assert!(auto_val::is_string(nv), "text 走 String lane");
    let str_idx = auto_val::decode_string(nv);

    // 替换实例 provider（原 Arc 仅剩本测试局部引用——释放）。
    drop(fake);
    vm.set_code_editor_provider(FakeEditor::sentinel());
    // 池内字符串仍有效且字节不变（owned bytes 物化，无 provider 借用）。
    let bytes = vm.get_string(str_idx).expect("池内字符串仍存在");
    assert_eq!(
        String::from_utf8_lossy(&bytes),
        "文本-p802-🎉",
        "VM 字符串在 provider 替换后逐字节有效"
    );

    // 重复调用清栈纪律：32 轮 push/pop 配平。
    for _ in 0..32 {
        let idx_key = vm.add_string("键".as_bytes().to_vec());
        vm.rc_push_str_idx(&mut task, idx_key);
        shim_code_editor_text(&mut task, &vm).unwrap();
        let _ = task.ram.pop_nv();
    }
    assert_eq!(task.ram.sp, sp0, "重复调用后任务栈回基线");

    // 进程默认替换 → 新 VM 快照新值（旧 VM 不受影响）。
    set_process_default_code_editor_host_provider(Arc::new(ExplicitOther));
    let vm2 = AutoVM::new(VirtualFlash::new(0), 64);
    let mut task2 = AutoTask::new(0, 256, 0);
    let idx_key = vm2.add_string("键".as_bytes().to_vec());
    vm2.rc_push_str_idx(&mut task2, idx_key);
    shim_code_editor_text(&mut task2, &vm2).unwrap();
    let nv = task2.ram.pop_nv();
    let str_idx = auto_val::decode_string(nv);
    let bytes = vm2.get_string(str_idx).expect("新 VM 字符串");
    assert_eq!(
        String::from_utf8_lossy(&bytes),
        "@explicit",
        "新 VM 快照替换后的默认"
    );
    // 旧 vm（实例覆盖同 fake 哨兵族）仍读原 sentinel。
    let idx_key = vm.add_string("键".as_bytes().to_vec());
    vm.rc_push_str_idx(&mut task, idx_key);
    shim_code_editor_text(&mut task, &vm).unwrap();
    let nv = task.ram.pop_nv();
    let str_idx = auto_val::decode_string(nv);
    let bytes = vm.get_string(str_idx).expect("旧 VM 字符串");
    assert_eq!(
        String::from_utf8_lossy(&bytes),
        "文本-p802-🎉",
        "旧 VM 保持构造期/实例 provider"
    );
}
