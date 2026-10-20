//! # autodown_natives — `autodown_*` VM natives（plan 019 批次八，Phase 3）
//!
//! 让 `.at` handler 可编程操作 AutoDown 文档：parse / serialize / 取文本 /
//! 查块 / Op 级编辑（applyOp）/ 模板插入——全部由 autodown-core crate 支撑
//! （feature `autodown`；无 feature 时 natives 存在但返回构建错误，保持
//! catalog 表稳定）。
//!
//! 传输形态循 `auto.fs.read_dir` 先例：复杂结构以 **JSON 字符串** 过界，
//! `.at` 侧经 `json.*` natives 导航。文档 JSON 形状（`block_to_json`）：
//!
//! ```json
//! {
//!   "id": "block-0",
//!   "kind": "Heading",                       // BlockType 变体名
//!   "attrs": [["level", {"Int": 1}]],        // 顺序稳定的 [k, v] 对
//!   "inlines": [{"text": "t", "marks": ["Strong"], "attrs": [["href", {"Str": "..."}]]}],
//!   "children": [ ... ]
//! }
//! ```
//!
//! crate `Value` 的 JSON 形态：`null`（Null）/ `{"Str": s}` / `{"Int": i}` /
//! `{"Bool": b}` / `{"List": [...]}` / `{"Attrs": [[k, v]...]}`。

use crate::vm::engine::{AutoVM, VMError};
use crate::vm::task::AutoTask;

// ---------------------------------------------------------------- shims
// 栈序：参数自右向左弹出（与 shim_shell_exec_submit 同款约定）。

/// `autodown_parse(src, final) -> json`：markdown → 块树 JSON。
pub fn shim_autodown_parse(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let is_final = crate::vm::native::pop_arg_i32(task);
    let _stake_final = crate::vm::native::StakeGuard::new(vm, is_final as i64 as u64);
    let src = pop_string_arg(task, vm);
    let json = parse_to_json(&src, is_final != 0).map_err(VMError::RuntimeError)?;
    push_string(task, vm, &json)
}

/// `autodown_serialize(json, emit_ids) -> src`：块树 JSON → .ad 文本。
pub fn shim_autodown_serialize(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let emit_ids = crate::vm::native::pop_arg_i32(task);
    let _stake_emit = crate::vm::native::StakeGuard::new(vm, emit_ids as i64 as u64);
    let json = pop_string_arg(task, vm);
    let src = serialize_from_json(&json, emit_ids != 0).map_err(VMError::RuntimeError)?;
    push_string(task, vm, &src)
}

/// `autodown_text(json, block_id) -> str`：块内联文本（spansText）。
pub fn shim_autodown_text(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let block_id = pop_string_arg(task, vm);
    let json = pop_string_arg(task, vm);
    let text = text_from_json(&json, &block_id).map_err(VMError::RuntimeError)?;
    push_string(task, vm, &text)
}

/// `autodown_editor_text(key) -> str`：编辑壳当前全文（plan 019 批次九，
/// on_change 回环 payload 口；key 未注册 / 双 feature 缺失时返回空串）。
pub fn shim_autodown_editor_text(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let key = pop_string_arg(task, vm);
    // PLAN-814 R3-04：编辑壳全文经引擎端口（上层 code-editor+autodown 面；
    // 缺席=空串语义保持）。
    if autodown_engine().is_none() {
        crate::install_ui_host_capabilities();
    }
    let text = autodown_engine()
        .and_then(|engine| engine.editor_text(&key))
        .unwrap_or_default();
    push_string(task, vm, &text)
}

/// `autodown_find_block(json, block_id) -> bool`。
pub fn shim_autodown_find_block(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let block_id = pop_string_arg(task, vm);
    let json = pop_string_arg(task, vm);
    let found = find_in_json(&json, &block_id).map_err(VMError::RuntimeError)?;
    task.ram.push_nv(auto_val::encode_bool(found));
    Ok(())
}

/// `autodown_insert_text(json, block_id, offset, text) -> json`：
/// Op::InsertText 经 applyOp（树函数式更新，返回新树 JSON）。
pub fn shim_autodown_insert_text(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let text = pop_string_arg(task, vm);
    let offset = crate::vm::native::pop_arg_i32(task);
    let _stake_off = crate::vm::native::StakeGuard::new(vm, offset as i64 as u64);
    let block_id = pop_string_arg(task, vm);
    let json = pop_string_arg(task, vm);
    let out =
        insert_text_json(&json, &block_id, offset as i64, &text).map_err(VMError::RuntimeError)?;
    push_string(task, vm, &out)
}

/// `autodown_insert_template(json, md, parent_id, index) -> json`：
/// 模板 markdown 解析为块序列，拼入 parent（空串 = 文档顶层）children[index]
/// （index 越界钳到末尾，负数 = 追加）。
pub fn shim_autodown_insert_template(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let index = crate::vm::native::pop_arg_i32(task);
    let _stake_idx = crate::vm::native::StakeGuard::new(vm, index as i64 as u64);
    let parent_id = pop_string_arg(task, vm);
    let md = pop_string_arg(task, vm);
    let json = pop_string_arg(task, vm);
    let out = insert_template_json(&json, &md, &parent_id, index as i64)
        .map_err(VMError::RuntimeError)?;
    push_string(task, vm, &out)
}

fn pop_string_arg(task: &mut AutoTask, vm: &AutoVM) -> String {
    let idx = task.ram.pop_str_idx() as u32;
    vm.get_string(idx)
        .map(|b| String::from_utf8_lossy(&b).to_string())
        .unwrap_or_default()
}

fn push_string(task: &mut AutoTask, vm: &AutoVM, s: &str) -> Result<(), VMError> {
    let idx = vm.add_string(s.as_bytes().to_vec());
    vm.rc_push_str_idx(task, idx as usize);
    Ok(())
}

// ------------------------------------------------- 实现（feature 门控）

// ------------------------------------------------- 实现端口（PLAN-814 R3-04）
// 真实实现（autodown-core 跨仓 crate）按 owner 留上层 auto-lang（其
// `autodown` feature 持依赖与装配）；SDK 经进程默认引擎端口供值——槽未
// 设时先触发 UI 宿主装配钩子（上层安装器注册真实引擎；纯 SDK 无钩子=
// 原构建错误语义保持，catalog 表恒稳定）。

/// autodown 文档引擎端口（对象安全；方法面 = 原实现 mod 的 fn 面）。
pub trait AutodownEngine: Send + Sync {
    fn parse_to_json(&self, src: &str, is_final: bool) -> Result<String, String>;
    fn serialize_from_json(&self, doc: &str, emit_ids: bool) -> Result<String, String>;
    fn text_from_json(&self, doc: &str, block_id: &str) -> Result<String, String>;
    fn find_in_json(&self, doc: &str, block_id: &str) -> Result<bool, String>;
    fn insert_text_json(
        &self,
        doc: &str,
        block_id: &str,
        offset: i64,
        text: &str,
    ) -> Result<String, String>;
    fn insert_template_json(
        &self,
        doc: &str,
        md: &str,
        parent_id: &str,
        index: i64,
    ) -> Result<String, String>;
    /// 编辑壳当前全文（key 未注册/引擎无编辑面 → None=空串语义保持）。
    fn editor_text(&self, key: &str) -> Option<String>;
}

static AUTODOWN_ENGINE: std::sync::RwLock<Option<std::sync::Arc<dyn AutodownEngine>>> =
    std::sync::RwLock::new(None);

/// 注册 autodown 引擎（显式注册权威——上层 autodown feature 装配点调用；
/// 纯 SDK 不注册，natives 保持构建错误回退）。
pub fn set_autodown_engine(engine: std::sync::Arc<dyn AutodownEngine>) {
    *AUTODOWN_ENGINE.write().unwrap() = Some(engine);
}

fn autodown_engine() -> Option<std::sync::Arc<dyn AutodownEngine>> {
    AUTODOWN_ENGINE.read().unwrap().clone()
}

// 桩/实现切换：引擎端口供值；槽未设先触发 UI 宿主装配钩子（上层真实
// 引擎经其 autodown feature 注册——直达 native 调用亦成立，R3-04 探针
// 形态），仍缺席则返回构建错误（natives 始终在表中可解析）。
macro_rules! dispatch {
    ($fname:ident($($arg:ident: $ty:ty),*) -> $retty:ty, $label:expr) => {
        pub fn $fname($($arg: $ty),*) -> Result<$retty, String> {
            if autodown_engine().is_none() {
                crate::install_ui_host_capabilities();
            }
            match autodown_engine() {
                Some(engine) => engine.$fname($($arg),*),
                None => Err($label.to_string()),
            }
        }
    };
}

dispatch!(parse_to_json(src: &str, is_final: bool) -> String,
    "built without the `autodown` feature: autodown_parse unavailable");
dispatch!(serialize_from_json(doc: &str, emit_ids: bool) -> String,
    "built without the `autodown` feature: autodown_serialize unavailable");
dispatch!(text_from_json(doc: &str, block_id: &str) -> String,
    "built without the `autodown` feature: autodown_text unavailable");
dispatch!(find_in_json(doc: &str, block_id: &str) -> bool,
    "built without the `autodown` feature: autodown_find_block unavailable");
dispatch!(insert_text_json(doc: &str, block_id: &str, offset: i64, text: &str) -> String,
    "built without the `autodown` feature: autodown_insert_text unavailable");
dispatch!(insert_template_json(doc: &str, md: &str, parent_id: &str, index: i64) -> String,
    "built without the `autodown` feature: autodown_insert_template unavailable");

// PLAN-814 R3-04：引擎实装往返测试随实现迁上层 auto-lang
// （ui/autodown_engine.rs——feature 门与依赖同 owner）。
