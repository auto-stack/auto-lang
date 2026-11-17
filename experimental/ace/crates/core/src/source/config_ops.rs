//! PLAN-982（B-CONFIG-I）：配置文档**源级原语目录**（`config.document/1`）。
//!
//! 设计（`docs/design/ace-config-document-evaluation.md` §3.2）：配置文档的值语义由
//! **受检原语**承载，表达式/`if`/`for`/模板仍由共享 ACE 指令执行；原语只做
//! "建节点/放值/追加/合并/取"这类**值和容器**操作，不解释 AST、不选业务分支。
//!
//! 本表是**单一真源**：源级 callee → 固定符号 → 精确参数 VT/结果 VT。运行时句柄
//! （节点/对象/列表/值）在源面一律是 i32；种类在运行期由原语核（kind 检查），
//! 不是靠静态类型假装。
//!
//! 覆盖范围（E3 当前切片）：根、节点（含 id 两形/位置与具名实参）、标量值
//! （i32/bool/str/char/f64）、节点值、对象与列表字面量的容器装载、追加、收尾。
//! 未覆盖（具名差额，后续增量）：`cfg.get` 读取、运行期 `for`（需要
//! `cfg.list.len/at` 计数原语）、合并语义（`cfg.merge`）。

use super::resolve::VT;

/// 文档求值能力键（进入 `requires`；运行时按 grant 核，不隐式授权）。
pub const CONFIG_CAPABILITY: &str = "config.document/1";

/// 一个源级配置原语。
#[derive(Clone, Copy, Debug)]
pub struct ConfigSourceOp {
    pub callee: &'static str,
    pub symbol: &'static str,
    pub params: &'static [VT],
    pub result: VT,
}

const P_EMPTY: &[VT] = &[];
const P_STR: &[VT] = &[VT::Str];
const P_H: &[VT] = &[VT::I32];
const P_H_STR: &[VT] = &[VT::I32, VT::Str];
const P_H_STR_I32: &[VT] = &[VT::I32, VT::Str, VT::I32];
const P_H_STR_BOOL: &[VT] = &[VT::I32, VT::Str, VT::Bool];
const P_H_STR_STR: &[VT] = &[VT::I32, VT::Str, VT::Str];
const P_H_STR_CHAR: &[VT] = &[VT::I32, VT::Str, VT::Char];
const P_H_STR_F64: &[VT] = &[VT::I32, VT::Str, VT::F64];
const P_H_I32: &[VT] = &[VT::I32, VT::I32];
const P_H_BOOL: &[VT] = &[VT::I32, VT::Bool];

/// 配置原语目录。同一 callee 的**标量重载**按实参 VT 选择（节点/对象/列表
/// 句柄与 i32 值同型，故节点值走独立 callee，绝不靠猜）。
pub const CONFIG_SOURCE_OPS: &[ConfigSourceOp] = &[
    // ---- 容器/文档骨架 ----
    ConfigSourceOp {
        callee: "cfg_root",
        symbol: "auto.config.document.root.v1",
        params: P_EMPTY,
        result: VT::I32,
    },
    ConfigSourceOp {
        callee: "cfg_node",
        symbol: "auto.config.document.node.v1",
        params: P_STR,
        result: VT::I32,
    },
    ConfigSourceOp {
        callee: "cfg_obj_new",
        symbol: "auto.config.document.obj-new.v1",
        params: P_EMPTY,
        result: VT::I32,
    },
    ConfigSourceOp {
        callee: "cfg_list_new",
        symbol: "auto.config.document.list-new.v1",
        params: P_EMPTY,
        result: VT::I32,
    },
    // ---- 节点身份（两形都保原语义：裸名/字符串）----
    ConfigSourceOp {
        callee: "cfg_id_name",
        symbol: "auto.config.document.id.name.v1",
        params: P_H_STR,
        result: VT::I32,
    },
    ConfigSourceOp {
        callee: "cfg_id_text",
        symbol: "auto.config.document.id.text.v1",
        params: P_H_STR,
        result: VT::I32,
    },
    // ---- 位置实参（标量按 VT 重载）----
    ConfigSourceOp {
        callee: "cfg_arg",
        symbol: "auto.config.document.arg.i32.v1",
        params: P_H_I32,
        result: VT::I32,
    },
    ConfigSourceOp {
        callee: "cfg_arg",
        symbol: "auto.config.document.arg.bool.v1",
        params: P_H_BOOL,
        result: VT::I32,
    },
    ConfigSourceOp {
        callee: "cfg_arg",
        symbol: "auto.config.document.arg.str.v1",
        params: &[VT::I32, VT::Str],
        result: VT::I32,
    },
    ConfigSourceOp {
        callee: "cfg_arg",
        symbol: "auto.config.document.arg.char.v1",
        params: &[VT::I32, VT::Char],
        result: VT::I32,
    },
    ConfigSourceOp {
        callee: "cfg_arg",
        symbol: "auto.config.document.arg.f64.v1",
        params: &[VT::I32, VT::F64],
        result: VT::I32,
    },
    // ---- 具名实参（`dir(id: d)`）----
    ConfigSourceOp {
        callee: "cfg_arg_named",
        symbol: "auto.config.document.arg-named.i32.v1",
        params: P_H_STR_I32,
        result: VT::I32,
    },
    ConfigSourceOp {
        callee: "cfg_arg_named",
        symbol: "auto.config.document.arg-named.bool.v1",
        params: P_H_STR_BOOL,
        result: VT::I32,
    },
    ConfigSourceOp {
        callee: "cfg_arg_named",
        symbol: "auto.config.document.arg-named.str.v1",
        params: P_H_STR_STR,
        result: VT::I32,
    },
    ConfigSourceOp {
        callee: "cfg_arg_named",
        symbol: "auto.config.document.arg-named.char.v1",
        params: P_H_STR_CHAR,
        result: VT::I32,
    },
    ConfigSourceOp {
        callee: "cfg_arg_named",
        symbol: "auto.config.document.arg-named.f64.v1",
        params: P_H_STR_F64,
        result: VT::I32,
    },
    // ---- 属性写入（根/节点；标量按 VT 重载，节点值走独立 callee）----
    ConfigSourceOp {
        callee: "cfg_put",
        symbol: "auto.config.document.put.i32.v1",
        params: P_H_STR_I32,
        result: VT::I32,
    },
    ConfigSourceOp {
        callee: "cfg_put",
        symbol: "auto.config.document.put.bool.v1",
        params: P_H_STR_BOOL,
        result: VT::I32,
    },
    ConfigSourceOp {
        callee: "cfg_put",
        symbol: "auto.config.document.put.str.v1",
        params: P_H_STR_STR,
        result: VT::I32,
    },
    ConfigSourceOp {
        callee: "cfg_put",
        symbol: "auto.config.document.put.char.v1",
        params: P_H_STR_CHAR,
        result: VT::I32,
    },
    ConfigSourceOp {
        callee: "cfg_put",
        symbol: "auto.config.document.put.f64.v1",
        params: P_H_STR_F64,
        result: VT::I32,
    },
    ConfigSourceOp {
        callee: "cfg_put_node",
        symbol: "auto.config.document.put.node.v1",
        params: P_H_STR_I32,
        result: VT::I32,
    },
    // ---- 对象字面量装载 ----
    ConfigSourceOp {
        callee: "cfg_obj_put",
        symbol: "auto.config.document.obj-put.i32.v1",
        params: P_H_STR_I32,
        result: VT::I32,
    },
    ConfigSourceOp {
        callee: "cfg_obj_put",
        symbol: "auto.config.document.obj-put.bool.v1",
        params: P_H_STR_BOOL,
        result: VT::I32,
    },
    ConfigSourceOp {
        callee: "cfg_obj_put",
        symbol: "auto.config.document.obj-put.str.v1",
        params: P_H_STR_STR,
        result: VT::I32,
    },
    ConfigSourceOp {
        callee: "cfg_obj_put",
        symbol: "auto.config.document.obj-put.char.v1",
        params: P_H_STR_CHAR,
        result: VT::I32,
    },
    ConfigSourceOp {
        callee: "cfg_obj_put",
        symbol: "auto.config.document.obj-put.f64.v1",
        params: P_H_STR_F64,
        result: VT::I32,
    },
    ConfigSourceOp {
        callee: "cfg_obj_put_node",
        symbol: "auto.config.document.obj-put.node.v1",
        params: P_H_STR_I32,
        result: VT::I32,
    },
    // ---- 列表装载 ----
    ConfigSourceOp {
        callee: "cfg_list_push",
        symbol: "auto.config.document.list-push.i32.v1",
        params: P_H_I32,
        result: VT::I32,
    },
    ConfigSourceOp {
        callee: "cfg_list_push",
        symbol: "auto.config.document.list-push.bool.v1",
        params: P_H_BOOL,
        result: VT::I32,
    },
    ConfigSourceOp {
        callee: "cfg_list_push",
        symbol: "auto.config.document.list-push.str.v1",
        params: &[VT::I32, VT::Str],
        result: VT::I32,
    },
    ConfigSourceOp {
        callee: "cfg_list_push",
        symbol: "auto.config.document.list-push.char.v1",
        params: &[VT::I32, VT::Char],
        result: VT::I32,
    },
    ConfigSourceOp {
        callee: "cfg_list_push",
        symbol: "auto.config.document.list-push.f64.v1",
        params: &[VT::I32, VT::F64],
        result: VT::I32,
    },
    ConfigSourceOp {
        callee: "cfg_list_push_node",
        symbol: "auto.config.document.list-push.node.v1",
        params: P_H_I32,
        result: VT::I32,
    },
    // ---- 值引用（把容器句柄作为值塞进另一容器）----
    ConfigSourceOp {
        callee: "cfg_node_value",
        symbol: "auto.config.document.value.node.v1",
        params: P_H,
        result: VT::I32,
    },
    ConfigSourceOp {
        callee: "cfg_obj_value",
        symbol: "auto.config.document.value.obj.v1",
        params: P_H,
        result: VT::I32,
    },
    ConfigSourceOp {
        callee: "cfg_list_value",
        symbol: "auto.config.document.value.list.v1",
        params: P_H,
        result: VT::I32,
    },
    // ---- 子节点追加与收尾 ----
    ConfigSourceOp {
        callee: "cfg_append",
        symbol: "auto.config.document.append.v1",
        params: P_H_I32,
        result: VT::I32,
    },
    ConfigSourceOp {
        callee: "cfg_finish",
        symbol: "auto.config.document.finish.v1",
        params: P_EMPTY,
        result: VT::I32,
    },
    // ---- 容器值（节点/对象/列表句柄与 i32 同型：种类由源结构选 callee）----
    ConfigSourceOp {
        callee: "cfg_put_obj",
        symbol: "auto.config.document.put.obj.v1",
        params: P_H_STR_I32,
        result: VT::I32,
    },
    ConfigSourceOp {
        callee: "cfg_put_list",
        symbol: "auto.config.document.put.list.v1",
        params: P_H_STR_I32,
        result: VT::I32,
    },
    ConfigSourceOp {
        callee: "cfg_obj_put_obj",
        symbol: "auto.config.document.obj-put.obj.v1",
        params: P_H_STR_I32,
        result: VT::I32,
    },
    ConfigSourceOp {
        callee: "cfg_obj_put_list",
        symbol: "auto.config.document.obj-put.list.v1",
        params: P_H_STR_I32,
        result: VT::I32,
    },
    ConfigSourceOp {
        callee: "cfg_list_push_obj",
        symbol: "auto.config.document.list-push.obj.v1",
        params: P_H_I32,
        result: VT::I32,
    },
    ConfigSourceOp {
        callee: "cfg_list_push_list",
        symbol: "auto.config.document.list-push.list.v1",
        params: P_H_I32,
        result: VT::I32,
    },
    ConfigSourceOp {
        callee: "cfg_arg_node",
        symbol: "auto.config.document.arg.node.v1",
        params: P_H_I32,
        result: VT::I32,
    },
    ConfigSourceOp {
        callee: "cfg_arg_obj",
        symbol: "auto.config.document.arg.obj.v1",
        params: P_H_I32,
        result: VT::I32,
    },
    ConfigSourceOp {
        callee: "cfg_arg_list",
        symbol: "auto.config.document.arg.list.v1",
        params: P_H_I32,
        result: VT::I32,
    },
    ConfigSourceOp {
        callee: "cfg_arg_named_node",
        symbol: "auto.config.document.arg-named.node.v1",
        params: P_H_STR_I32,
        result: VT::I32,
    },
    ConfigSourceOp {
        callee: "cfg_arg_named_obj",
        symbol: "auto.config.document.arg-named.obj.v1",
        params: P_H_STR_I32,
        result: VT::I32,
    },
    ConfigSourceOp {
        callee: "cfg_arg_named_list",
        symbol: "auto.config.document.arg-named.list.v1",
        params: P_H_STR_I32,
        result: VT::I32,
    },
];

/// 该拼写是否是配置原语 callee（无论重载是否匹配）。
pub fn is_callee(name: &str) -> bool {
    CONFIG_SOURCE_OPS.iter().any(|o| o.callee == name)
}

fn vt_admits(expected: VT, actual: VT) -> bool {
    match expected {
        // 文本参数接 `str`/`Str` 两种文本值（借/自有同一语义）。
        VT::Str => matches!(actual, VT::Str | VT::StrOwned),
        // `cfg_put`/`cfg_obj_put`/`cfg_list_push`/`cfg_arg` 的文本重载同理。
        other => expected_eq(other, actual),
    }
}

fn expected_eq(a: VT, b: VT) -> bool {
    a == b
}

/// 按 callee 名字 + 实参 VT 选重载（无匹配 = None，调用方须给具名错误）。
pub fn lookup_callee(name: &str, args: &[VT]) -> Option<&'static ConfigSourceOp> {
    CONFIG_SOURCE_OPS.iter().find(|o| {
        o.callee == name
            && o.params.len() == args.len()
            && o.params
                .iter()
                .zip(args.iter())
                .all(|(p, a)| vt_admits(*p, *a))
    })
}

/// 符号 → 原语（运行期/降级侧按符号查）。
pub fn by_symbol(sym: &str) -> Option<&'static ConfigSourceOp> {
    CONFIG_SOURCE_OPS.iter().find(|o| o.symbol == sym)
}

/// 符号的能力键（进入模块 `requires`；未知符号 = None，不静默给 TRACE）。
pub fn capability_of(sym: &str) -> Option<&'static str> {
    by_symbol(sym).map(|_| CONFIG_CAPABILITY)
}

/// 该拼写在此实参形状下是否**存在**重载（用于把"参数类型不匹配"和
/// "未知 callee"分开报）。
pub fn has_overload_for(name: &str) -> bool {
    is_callee(name)
}
