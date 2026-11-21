//! 严格投影：auto-source CST → 现有 `ast::Program`（PLAN-927）。
//!
//! - 结构驱动：从 CST 节点/字段/叶读取并构造 AST——不是第二份 token 流
//!   parser；字面量解码与 i32 域检查在投影层（语法身份承载于 CST/grammar）。
//! - 能力/形状拒绝按旧 parser 的逐构造顺序产生诊断；全部收集后取**首个**
//!   （span 起点序 = 旧首错事实）。
//! - 单一显式帧机：块/语句/表达式/实参/构造字段全迭代——深链（嵌套组/
//!   链式调用/深块/长一元链）不依赖线程栈；访问预算受控（资源态，不是
//!   语法 unsupported）。帧协议：需要 N 个子值的帧先压"汇总帧"，再逆序
//!   压子任务——子值按文档序留在 Val 栈上，汇总帧弹出组装。

use std::collections::{BTreeMap, BTreeSet};

use super::kinds::SourceKinds;
use super::lex;
use crate::atom_text::{Diagnostic, Span, Stage, SurfaceFact};
use crate::source::age_package::widget_lists::WidgetListProjection;
use crate::source::ast::*;
use crate::source::SourceProfile;
// PLAN-980（B-DIALECT-I 完整批）：组件面 widget 成员槽位的结构类型。
use crate::source::age_component::{
    ComponentAssign, ComponentDesignEntry, ComponentEntryValue, ComponentField, ComponentHandler, ComponentStmt,
    ComponentClassPart, ComponentMemberRow, ComponentMsgVariant, ComponentViewElement, ComponentViewEntry,
    ComponentViewItem,
};
use ace_syntax_runtime::cst::{CstTree, FLAG_TRIVIA};
use ace_syntax_runtime::program::{GrammarProgram, KIND_CALL, KIND_POSTFIX};
use ace_syntax_runtime::snapshot::Snapshot;

const I32_MIN_MAG: u128 = 2147483648;
const I32_MAX: u128 = 2147483647;
/// 投影工作预算（访问帧数；超出 = 资源态受控拒绝）。
const PROJECTION_NODE_BUDGET: usize = 1 << 22;

#[derive(Clone, Copy, Debug, Default)]
pub struct SourcePragmas {
    pub script: Option<Span>,
    pub rust: Option<Span>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceMode {
    Normal,
    Script,
}

#[derive(Clone, Debug, Default)]
pub struct SourceMetadata {
    pub pragmas: SourcePragmas,
    pub annotations: Vec<SourceAnnotation>,
}

#[derive(Clone, Debug)]
pub struct SourceAnnotation {
    pub declaration: String,
    pub owner_span: Span,
    pub span: Span,
    pub av_visible: bool,
    pub kind: SourceAnnotationKind,
}

#[derive(Clone, Debug)]
pub enum SourceAnnotationKind {
    Api {
        method: String,
        path: String,
    },
    Export {
        abi: String,
    },
    Rust {
        name: String,
        arguments: Option<Span>,
    },
}

impl SourcePragmas {
    pub fn mode(self, path: &str) -> SourceMode {
        if self.rust.is_some() {
            SourceMode::Normal
        } else if self.script.is_some()
            || std::path::Path::new(path)
                .extension()
                .and_then(std::ffi::OsStr::to_str)
                .is_some_and(|ext| ext.eq_ignore_ascii_case("as"))
        {
            SourceMode::Script
        } else {
            SourceMode::Normal
        }
    }

    fn insert(&mut self, name: &str, span: Span) -> Result<(), Diagnostic> {
        let slot = match name {
            "script" => &mut self.script,
            "rust" => &mut self.rust,
            _ => unreachable!("source mode is a schema contextual name"),
        };
        if slot.is_some() {
            return Err(Diagnostic::new(
                Stage::Bind,
                "source.attribute-duplicate",
                format!("duplicate file pragma `{name}`"),
                span,
            ));
        }
        *slot = Some(span);
        Ok(())
    }

    pub(super) fn merge(&mut self, next: Self) -> Result<(), Diagnostic> {
        if let Some(span) = next.script {
            self.insert("script", span)?;
        }
        if let Some(span) = next.rust {
            self.insert("rust", span)?;
        }
        Ok(())
    }
}

pub(crate) fn project_program(
    p: &GrammarProgram,
    k: &SourceKinds,
    tree: &CstTree,
    snap: &Snapshot,
    profile: SourceProfile,
) -> Result<Program, Vec<Diagnostic>> {
    project_program_with_pragmas(p, k, tree, snap, profile).map(|(program, _)| program)
}

pub(crate) fn project_program_with_pragmas(
    p: &GrammarProgram,
    k: &SourceKinds,
    tree: &CstTree,
    snap: &Snapshot,
    profile: SourceProfile,
) -> Result<(Program, SourceMetadata), Vec<Diagnostic>> {
    project_program_with_caps(
        p,
        k,
        tree,
        snap,
        profile,
        false,
        profile.is_auto_language(),
        false,
        false,
    )
}

/// PLAN-963（V-G06）：包模块的投影入口——记录模板**推迟**到扁平程序（全部模块
/// 拼接、符号已按包/模块 mangle）后统单态化。模块内就地单态化只能看到本模块
/// 模板，跨模块使用点会失败，且各模块各自合成会破坏类型身份（同一
/// `(声明模块, 模板, 实参)` 必须只有一个类型符号）。
pub(crate) fn project_program_deferring_templates(
    p: &GrammarProgram,
    k: &SourceKinds,
    tree: &CstTree,
    snap: &Snapshot,
    profile: SourceProfile,
    age_face: bool,
) -> Result<(Program, SourceMetadata), Vec<Diagnostic>> {
    project_program_with_caps(
        p, k, tree, snap, profile, false, profile.is_auto_language(), true, age_face,
    )
}

/// PLAN-980 P3：把**单个**表达式 CST 节点投影为 AST 表达式（style 值等由包前门
/// 独立取值的场景用；与整程序投影同一 Engine/同一语法，不建第二条表达式投影）。
pub(crate) fn project_expr_node(
    p: &GrammarProgram,
    k: &SourceKinds,
    tree: &CstTree,
    snap: &Snapshot,
    profile: SourceProfile,
    node: u32,
) -> Expr {
    let mut eng = Engine::new(p, k, tree, snap, profile, false, false, true);
    eng.project_expr_sync(node)
}

/// PLAN-935/963（T-02）：能力感知投影。类型位裸 ident 只在拼写命中**当前
/// 声明**的类型参数表时投影为 `TypeRef::TyVar`（instances 面的 fn 模板与
/// PLAN-963 的泛型记录成员同一规则）；fn-tail 的 `<T>` 投影为
/// `FnItem::type_params`，`type Name<T>` 投影为 `TypeItem::type_params`。
/// plain 面与旧路径逐字节同形。
pub(crate) fn project_program_with_caps(
    p: &GrammarProgram,
    k: &SourceKinds,
    tree: &CstTree,
    snap: &Snapshot,
    profile: SourceProfile,
    ct_face: bool,
    instances_face: bool,
    defer_templates: bool,
    age_face: bool,
) -> Result<(Program, SourceMetadata), Vec<Diagnostic>> {
    // 词法层先行（旧 tokenize 全文件先行的首事实序）。
    // PLAN-936（T-04）：ct 面上字符串 token 仅作为 meta 字段选择子进入词法事实（值位置由
    // resolve 拒绝）。
    if let Some(d) = lex::first_lex_fact_ct(tree, snap, k, profile, ct_face) {
        return Err(vec![d]);
    }
    let mut eng = Engine::new(p, k, tree, snap, profile, ct_face, instances_face, age_face);
    let tree_has_errors = tree.has_error_nodes();
    // PLAN-960（T-02）：const/var 模块项 kind（仅 packages-modules 裁剪存在；
    // 其余面无此二规则 = 保持原分派行为）。
    let const_item_kind = p
        .rules
        .iter()
        .position(|r| r.name == "const-item")
        .map(|i| ace_syntax_runtime::program::RULE_KIND_BASE + i as u32);
    // PLAN-981（IF-981-ORIGINAL）：模块级 `let` 资源绑定（不可变，同 const）。
    let age_let_item_kind = p
        .rules
        .iter()
        .position(|r| r.name == "age-let-item")
        .map(|i| ace_syntax_runtime::program::RULE_KIND_BASE + i as u32);
    let auto_var_item_kind = p
        .rules
        .iter()
        .position(|r| r.name == "auto-var-item")
        .map(|i| ace_syntax_runtime::program::RULE_KIND_BASE + i as u32);
    let var_item_kind = p
        .rules
        .iter()
        .position(|r| r.name == "var-item")
        .map(|i| ace_syntax_runtime::program::RULE_KIND_BASE + i as u32);
    // PLAN-967（F0/T-10）：`use.rs`/`use.c` 项 kind（仅 text-record 裁剪存在）。
    let use_rs_item_kind = p
        .rules
        .iter()
        .position(|r| r.name == "use-rs-item")
        .map(|i| ace_syntax_runtime::program::RULE_KIND_BASE + i as u32);
    // PLAN-1095: the H0 node declaration kind (ordinary Auto face).
    let age_node_item_kind = p
        .rules
        .iter()
        .position(|r| r.name == "age-node-item")
        .map(|i| ace_syntax_runtime::program::RULE_KIND_BASE + i as u32);
    let use_c_item_kind = p
        .rules
        .iter()
        .position(|r| r.name == "use-c-item")
        .map(|i| ace_syntax_runtime::program::RULE_KIND_BASE + i as u32);
    // PLAN-1026：`use.py <module.path>: <members>` 项 kind（aggregates 裁剪）。
    let use_py_item_kind = p
        .rules
        .iter()
        .position(|r| r.name == "use-py-item")
        .map(|i| ace_syntax_runtime::program::RULE_KIND_BASE + i as u32);
    let use_py_mod_tail_kind = p
        .rules
        .iter()
        .position(|r| r.name == "use-py-mod-tail")
        .map(|i| ace_syntax_runtime::program::RULE_KIND_BASE + i as u32);
    let use_py_member_tail_kind = p
        .rules
        .iter()
        .position(|r| r.name == "use-py-member-tail")
        .map(|i| ace_syntax_runtime::program::RULE_KIND_BASE + i as u32);
    let mut items = Vec::new();
    // PLAN-963（T-03）：`spec` 声明在投影期擦除——收集到投影侧表，遵从检查
    // 与默认方法注入在语法期单态化之后统一完成（spec 不进入 AST/downstream，
    // 也不存在供后端猜测的解释路径）。
    let mut specs: Vec<SpecDecl> = Vec::new();
    // PLAN-963（T-03）：`type Name = T` 别名同样在投影期消化——先收集，再在
    // 单态化之前把每个类型位置的别名引用展开成目标（含泛型实参位）。
    let mut aliases: Vec<TypeAliasDecl> = Vec::new();
    // PLAN-963（T-03）：`ext Type { .. }` 扩展块同属投影期消化（合入类型的
    // methods，先于 `as Spec` 遵从检查）。
    let mut exts: Vec<ExtDecl> = Vec::new();
    let mut saw_script = false;
    let mut pragmas = SourcePragmas::default();
    let file_mode_kind = eng.rule_kind_of("file-mode-item");
    // F-941-12（r9）：前一个非产物顶层 item 是否含错误子树（恢复产物 script
    // 区判定用）。
    let mut last_item_errored = false;
    for d in &tree.decls {
        // decl 单元 = `item` 分派节点；真实声明是其首个非 trivia 孩子。
        let node = {
            let dn = d.node;
            if tree.kind(dn) == k.item {
                tree.children(dn)
                    .into_iter()
                    .find(|&c| tree.node(c).flags & FLAG_TRIVIA == 0)
                    .unwrap_or(dn)
            } else {
                dn
            }
        };
        if Some(tree.kind(node)) == file_mode_kind {
            if !tree_has_errors || !subtree_has_errors(tree, node) {
                let mode = eng.cx.field_child(node, "mode").expect("file mode");
                let name = eng.cx.spelling(mode);
                let span = eng.cx.span_of(mode);
                if let Err(diagnostic) = pragmas.insert(&name, span) {
                    eng.cx.reject(span, || diagnostic);
                }
            }
            continue;
        }
        let attributed = attributed_owner(p, tree, node).map(|owner| (node, owner));
        let node = attributed.map_or(node, |(_, owner)| owner);
        // PLAN-950: public visibility is retained in ModuleSyntax, while
        // the existing function projector consumes the nested fn item.
        // PLAN-960（T-02）：public-type/public-const 前缀同样在分派前剥壳。
        let node = if p
            .rules
            .iter()
            .position(|r| r.name == "public-fn")
            .is_some_and(|i| {
                tree.kind(node) == ace_syntax_runtime::program::RULE_KIND_BASE + i as u32
            }) {
            p.field_id("function")
                .and_then(|f| tree.field_child(node, f))
                .unwrap_or(node)
        } else if p
            .rules
            .iter()
            .position(|r| r.name == "public-item")
            .is_some_and(|i| {
                tree.kind(node) == ace_syntax_runtime::program::RULE_KIND_BASE + i as u32
            })
        {
            // PLAN-960（T-02）：public-body 分派节点的实体孩子（fn/type/const/use）。
            p.field_id("body")
                .and_then(|f| tree.field_child(node, f))
                .and_then(|b| {
                    tree.children(b)
                        .into_iter()
                        .find(|&c| tree.node(c).flags & FLAG_TRIVIA == 0)
                })
                .unwrap_or(node)
        } else {
            node
        };
        let kind = tree.kind(node);
        // 错误子树跳过乐观投影（结构层映射负责其诊断；干净子树照常投影
        // 以保真能力拒绝的位置序）。PLAN-928（T-04）：脚本区首 token 的
        // 位置序拒绝先于跳过（R04 类：恢复区内部能力拒绝不得抢位）。
        if kind == k.script_item.unwrap_or(u32::MAX) {
            let mut n = node;
            loop {
                if eng.cx.tree.node(n).child_count == 0 {
                    break;
                }
                match eng
                    .cx
                    .children(n)
                    .into_iter()
                    .find(|&c| eng.cx.tree.node(c).flags & FLAG_TRIVIA == 0)
                {
                    Some(c) => n = c,
                    None => break,
                }
            }
            let head = eng.cx.spelling(n);
            if matches!(head.as_str(), "type" | "enum" | "static") {
                let span = eng.cx.span_of(n);
                let h = head.clone();
                eng.cx.reject(span, move || {
                    Diagnostic::new(
                        Stage::Text,
                        "source.syntax",
                        format!("expected statement, found `{h}`"),
                        span,
                    )
                });
            }
        }
        let node_errored =
            tree_has_errors && subtree_has_errors(tree, attributed.map_or(node, |(wrapper, _)| wrapper));
        // F-941-12（PLAN-941 r9）：恢复产物 script 区——错误树中紧跟含错误
        // item 的顶层 script 是引擎在最后一个完整块 `}` 处提前闭合外层 body、
        // 把尾部语句重释为顶层的产物（旧 parser 线性解析不产生该区域）；其
        // script 形能力拒绝（entry-top-return/entry-order）会按更早位置抢在
        // 结构层首诊断之前。跳过其投影与 saw_script（产物 run 内粘滞），首
        // 诊断交结构层期望游走；上方头部 type/enum/static 拒绝（R04 位置序）
        // 与干净树的真 script 不受影响。
        if tree_has_errors && kind == k.script_item.unwrap_or(u32::MAX) && last_item_errored {
            continue; // 产物 run 延续：不更新 last_item_errored
        }
        if node_errored {
            last_item_errored = true;
            continue;
        }
        last_item_errored = false;
        let av_visible =
            attributed.is_none_or(|(wrapper, _)| attribute_targets_av(p, tree, snap, wrapper));
        let (attribute_params, test, single) = attributed
            .map_or_else(Default::default, |(wrapper, _)| {
                eng.project_decl_attributes(wrapper, node, av_visible)
            });
        if !av_visible {
            continue;
        }
        if kind == k.fn_item
            || kind == k.type_item.unwrap_or(u32::MAX)
            || kind == k.enum_item.unwrap_or(u32::MAX)
            // PLAN-962（T-08）：tag/union 声明同走声明项门。
            || kind == k.tag_item.unwrap_or(u32::MAX)
            || kind == k.union_item.unwrap_or(u32::MAX)
            || Some(kind) == const_item_kind
            || Some(kind) == var_item_kind
            || Some(kind) == auto_var_item_kind
            // PLAN-981（IF-981-ORIGINAL）：模块级 `let` 也是声明位项。
            || Some(kind) == age_let_item_kind
            || Some(kind) == k.spec_item
            // PLAN-963（T-03）：`ext` 块也是声明项（须在 script 前）。
            || Some(kind) == k.ext_item
            // PLAN-967（F0/T-10）：导入前门也是声明区项（须在 script 前）。
            || Some(kind) == use_rs_item_kind
            || Some(kind) == use_c_item_kind
            // PLAN-1026：py 导入前门也是声明区项（须在 script 前）。
            || Some(kind) == use_py_item_kind
            // PLAN-968: `task` declarations are declaration-position items.
            || Some(kind) == k.task_item
            // PLAN-1095: H0 node declarations are declaration-position items
            // on the ordinary Auto face.
            || Some(kind) == age_node_item_kind
        {
            if saw_script {
                let span = first_leaf_span(&eng.cx, node);
                eng.cx.reject(span, move || {
                    Diagnostic::new(
                        Stage::Text,
                        "source.entry-order",
                        "declarations must precede the script statement sequence; a script runs to end of file",
                        span,
                    )
                });
                continue;
            }
            if kind == k.fn_item {
                let mut function = eng.run_fn_item_with_params(
                    node,
                    None,
                    Receiver::Free,
                    attribute_params,
                );
                function.test_attribute = test;
                if let Some(span) = test {
                    if matches!(function.ret.kind, TypeRef::Inferred) {
                        function.ret.kind = TypeRef::Builtin(TypeWord::Void);
                    }
                    if function.name.name == "main"
                        || !function.params.is_empty()
                        || !function.type_params.is_empty()
                        || !matches!(
                            function.ret.kind,
                            TypeRef::Builtin(TypeWord::Void | TypeWord::Int | TypeWord::I32)
                        )
                    {
                        eng.cx.reject(span, || {
                            Diagnostic::new(
                                Stage::Bind,
                                "source.attribute-test-signature",
                                "`test` requires a zero-parameter non-generic function other than main returning unit or int",
                                span,
                            )
                        });
                    }
                }
                items.push(Item::Fn(function));
            } else if kind == k.enum_item.unwrap_or(u32::MAX)
                || kind == k.tag_item.unwrap_or(u32::MAX)
                || kind == k.union_item.unwrap_or(u32::MAX)
            {
                // PLAN-962（T-08）：tag = enum 别名；union = tagged 安全联合
                // ——同构投影到 Enum AST（union 的读语义由变体 guard 承载：
                // 非活动臂 is-分支不匹配）。
                items.push(Item::Enum(eng.run_enum_item(node)));
            } else if Some(kind) == age_let_item_kind {
                // PLAN-981（IF-981-ORIGINAL）：模块级 `let` 无注解，类型由**初值**
                // 推断。本切片只覆盖作者实际写法：初值是 AGE 资源服务调用，其返回
                // 类型由声明表给出（`material2d.color -> Material2D` 等）。其它初值
                // 形状具名拒绝——不假装有通用推断器，也不猜成 i32。
                let span = first_leaf_span(&eng.cx, node);
                let value_n = match eng.cx.field_child(node, "value") {
                    Some(v) => v,
                    None => continue,
                };
                let inferred = age_service_result_port(&eng.cx, value_n);
                match inferred {
                    Some(port) => {
                        // 与 `const` 同一 Item 形状，注解取推断出的端口类型。
                        let name_leaf = eng.cx.field_child(node, "name").expect("let name");
                        let name = ident_of(&mut eng.cx, name_leaf);
                        let ann = TypeAnn {
                            kind: TypeRef::Record(Ident {
                                name: port.to_string(),
                                span: eng.cx.span_of(value_n),
                            }),
                            span: eng.cx.span_of(value_n),
                        };
                        let value = eng.project_expr_sync(value_n);
                        items.push(Item::Const(crate::source::ast::ConstItem {
                            name,
                            ty: ann,
                            value,
                            span: eng.cx.span_of(node),
                        }));
                        continue;
                    }
                    None => {
                        eng.cx.reject(span, move || {
                            Diagnostic::new(
                                Stage::Text,
                                "source.module-let-inference",
                                "a module-level `let` is typed from its initializer; this slice \
                                 infers only AGE resource-service calls (e.g. \
                                 `material2d.color(...)`). Write `const <name>: <type> = <expr>` \
                                 for other initializers",
                                span,
                            )
                        });
                        continue;
                    }
                }
            } else if Some(kind) == const_item_kind
                || Some(kind) == var_item_kind
                || Some(kind) == auto_var_item_kind
            {
                items.push(eng.run_global_item(node, Some(kind) != const_item_kind));
            } else if Some(kind) == use_rs_item_kind || Some(kind) == use_c_item_kind {
                // PLAN-967（F0）：`use.rs <provider>` —— lang 叶必须恰为
                // `rs`；任何其它拼写是命名拒绝，不是静默的模块 use。
                let lang_leaf = eng.cx.field_child(node, "lang").expect("use.rs lang leaf");
                let lang = eng.cx.spelling(lang_leaf);
                let kw_leaf = eng.cx.field_child(node, "kw").expect("use.rs kw leaf");
                if lang != "rs" && lang != "c" {
                    let span = eng.cx.span_of(lang_leaf);
                    eng.cx.reject(span, move || {
                        Diagnostic::new(
                            Stage::Text,
                            "source.use-rs.syntax",
                            format!(
                                "`use.{lang}` is not a supported import; the imports are `use.rs`/`use.c`"
                            ),
                            span,
                        )
                    });
                    continue;
                }
                let provider_leaf = eng
                    .cx
                    .field_child(node, "provider")
                    .expect("use.rs provider leaf");
                let kw_span = eng.cx.span_of(kw_leaf);
                // PLAN-1093: `use.c <string.h>` — the C header spelling
                // projects the header name (ident spellings keep the
                // normal name checks).
                let provider = {
                    let text = eng.cx.spelling(provider_leaf);
                    if text.starts_with('<') {
                        Ident {
                            name: text
                                .trim_start_matches('<')
                                .trim_end_matches('>')
                                .trim()
                                .to_string(),
                            span: eng.cx.span_of(provider_leaf),
                        }
                    } else {
                        ident_of(&mut eng.cx, provider_leaf)
                    }
                };
                let span = Span {
                    start: kw_span.start,
                    end: provider.span.end,
                };
                items.push(Item::UseRs(UseRsItem {
                    lang,
                    provider,
                    span,
                }));
            } else if Some(kind) == use_py_item_kind {
                // PLAN-1026：`use.py <module.path>: <members>` —— lang 叶
                // 必须恰为 `py`；点路径段与成员清单从 many 尾子树收集。
                let lang_leaf = eng.cx.field_child(node, "lang").expect("use.py lang leaf");
                let lang = eng.cx.spelling(lang_leaf);
                let kw_leaf = eng.cx.field_child(node, "kw").expect("use.py kw leaf");
                if lang != "py" {
                    let span = eng.cx.span_of(lang_leaf);
                    eng.cx.reject(span, move || {
                        Diagnostic::new(
                            Stage::Text,
                            "source.use-py.syntax",
                            format!(
                                "`use.{lang}` is not the Python import; the spelling is `use.py <module>: <members>`"
                            ),
                            span,
                        )
                    });
                    continue;
                }
                let head_leaf = eng
                    .cx
                    .field_child(node, "mod-head")
                    .expect("use.py module head");
                let mut module = vec![ident_of(&mut eng.cx, head_leaf)];
                // PLAN-1094: members may spell Auto keywords (Python builtins
                // like `int`/`float`/`bool`) — they are bridge strings, not
                // Auto declarations, so the spelling is read WITHOUT the
                // regular name check.
                let member_leaf_of = |cx: &mut Cx, node: u32| -> u32 {
                    let name = cx
                        .field_child(node, "name")
                        .or_else(|| cx.field_child(node, "member-head"))
                        .or_else(|| cx.field_child(node, "member"))
                        .expect("use.py member name leaf");
                    name
                };
                let member_ident_of = |cx: &Cx, leaf: u32| Ident {
                    name: cx.spelling(leaf),
                    span: cx.span_of(leaf),
                };
                let member_head_leaf = eng
                    .cx
                    .field_child(node, "member-head")
                    .expect("use.py member head");
                let head_leaf_n = member_leaf_of(&mut eng.cx, member_head_leaf);
                let mut members = vec![member_ident_of(&eng.cx, head_leaf_n)];
                let mut last_end = eng.cx.span_of(member_head_leaf).end;
                for c in eng.cx.children(node) {
                    let ck = eng.cx.tree.kind(c);
                    if Some(ck) == use_py_mod_tail_kind {
                        let seg = eng.cx.field_child(c, "seg").expect("use.py module segment");
                        module.push(ident_of(&mut eng.cx, seg));
                    } else if Some(ck) == use_py_member_tail_kind {
                        let m = eng.cx.field_child(c, "member").expect("use.py member");
                        let leaf_n = member_leaf_of(&mut eng.cx, m);
                        let id = member_ident_of(&eng.cx, leaf_n);
                        last_end = id.span.end;
                        members.push(id);
                    }
                }
                let kw_span = eng.cx.span_of(kw_leaf);
                let span = Span {
                    start: kw_span.start,
                    end: last_end,
                };
                items.push(Item::UsePy(UsePyItem {
                    module,
                    members,
                    span,
                }));
            } else if Some(kind) == k.spec_item {
                // PLAN-963（T-03）：spec 声明——只进投影侧表，不产出 Item。
                specs.push(eng.run_spec_item(node));
            } else if Some(kind) == k.ext_item {
                // PLAN-963（T-03）：`ext Type { .. }`——投影侧表，随后合入该
                // 类型的 methods（必须在遵从检查之前：ext 可满足 spec 方法）。
                exts.push(eng.run_ext_item(node));
            } else if eng.is_type_alias(node) {
                // PLAN-963（T-03）：类型别名——只进投影侧表，不产出 Item。
                aliases.push(eng.run_type_alias_item(node));
            } else if Some(kind) == k.task_item {
                // PLAN-968: task actor declaration.
                let mut task = eng.run_task_item(node);
                task.single = single;
                items.push(Item::Task(task));
            } else if p
                .rules
                .iter()
                .position(|r| r.name == "age-store-item")
                .map(|i| ace_syntax_runtime::program::RULE_KIND_BASE + i as u32)
                .is_some_and(|k2| kind == k2)
            {
                // PLAN-105 T-02（U03 真语料 013/016）：store 声明结构已入语法
                // （S1 首刀）；语义（共享实例/.store 绑定/store 方法）未接线——
                // **具名拒**，绝不静默降级成同名记录类型。
                let span = eng.cx.span_of(node);
                eng.cx.reject(span, || {
                    crate::atom_text::Diagnostic::new(
                        crate::atom_text::Stage::Text,
                        "source.age.component-unsupported",
                        "store declarations are parsed; store semantics are not wired yet (S1)"
                            .to_string(),
                        span,
                    )
                });
                continue;
            } else if p
                .rules
                .iter()
                .position(|r| {
                    r.name == "age-node-item"
                        || r.name == "age-node-c-item"
                        || r.name == "age-pub-node-item"
                        // PLAN-980 P3/PLAN-1072 刀A：style 声明由领域前门单独投影
                        // （包面=str const/fn 糖；组件面=惰性视图数据表）。
                        || r.name == "style-item"
                })
                .map(|i| ace_syntax_runtime::program::RULE_KIND_BASE + i as u32)
                .is_some_and(|k2| kind == k2)
            {
                // PLAN-1095: on the ordinary Auto face the H0 node
                // declaration is a real AST item (CPU-only — the view text
                // facts come from the 979 producer on the same source).
                let node_kind = p
                    .rules
                    .iter()
                    .position(|r| r.name == "age-node-item")
                    .map(|i| ace_syntax_runtime::program::RULE_KIND_BASE + i as u32);
                if profile.is_auto_language() && node_kind == Some(kind) {
                    items.push(Item::Node(eng.run_ast_node_item(node)));
                    continue;
                }
                // PLAN-979/981：AGE node 声明由**领域前门**单独投影（候选 + 脱糖），
                // 不是普通声明项。这里必须显式跳过——否则下面的 `Item::Type`
                // 兜底会把一个 node 静默变成同名记录类型（本批实测过的静默丢弃）。
                continue;
            } else {
                items.push(Item::Type(eng.run_type_item_with_params(node, attribute_params)));
            }
        } else if kind == k.script_item.unwrap_or(u32::MAX) {
            saw_script = true;
            items.push(Item::Fn(eng.run_script_item(node)));
        }
    }
    if tree_has_errors {
        // 结构层首错映射（与干净子树的能力拒绝按位置竞争，最早者胜）。
        if let Some(d) = structural_first_diag(&eng.cx, tree) {
            eng.cx.diags.push((d.span.start as u64, d));
        }
        if let Some(first) = eng.cx.first_diag() {
            return Err(vec![first]);
        }
        let end = snap.len() as usize;
        return Err(vec![Diagnostic::new(
            Stage::Text,
            "source.syntax",
            "expected statement or `}`, found end of file",
            Span { start: end, end },
        )]);
    }
    if let Some(first) = eng.cx.first_diag() {
        return Err(vec![first]);
    }
    // PLAN-963（T-03）：类型别名展开（先于泛型记录单态化：别名目标里的
    // `Point<int>` 在展开后落在使用位，再由单态化遍换名）。
    for d in expand_type_aliases(&mut items, &aliases, profile.is_auto_language()) {
        eng.cx.diags.push((d.span.start as u64, d));
    }
    if let Some(first) = eng.cx.first_diag() {
        return Err(vec![first]);
    }
    // PLAN-963（T-03）：`ext Type { .. }` 的方法合入已有类型（先于单态化：
    // 扩展方法随模板/具体记录一起参与下游；也先于遵从检查：ext 可满足 spec 方法）。
    for d in merge_ext_blocks(&mut items, &exts) {
        eng.cx.diags.push((d.span.start as u64, d));
    }
    if let Some(first) = eng.cx.first_diag() {
        return Err(vec![first]);
    }
    // PLAN-963（T-02）：用户泛型记录类型的语法期单态化（每个使用位都显式
    // 携带类型实参，无需语义推理）。下游 resolve/HIR/布局/四端只见具体
    // 记录类型；模板本身从程序中移除。包模块面推迟到扁平程序后统一做
    // （V-G06：跨模块使用点共享实例身份）。
    if !defer_templates {
        for d in expand_user_generic_types(&mut items, &specs, profile.is_auto_language()) {
            eng.cx.diags.push((d.span.start as u64, d));
        }
    }
    // PLAN-963（T-03）：`type X as S` 遵从检查 + spec 默认方法注入。放在单态化
    // 之后：合成实例同样按具体方法签名核验，注入的默认体随实例一起进入 resolve。
    for d in apply_spec_conformance(&mut items, &specs, profile.is_auto_language()) {
        eng.cx.diags.push((d.span.start as u64, d));
    }
    if profile.is_auto_language() && !defer_templates && eng.cx.first_diag().is_none() {
        for d in materialize_spec_values(&mut items, &specs) {
            eng.cx.diags.push((d.span.start as u64, d));
        }
    }
    // PLAN-963（T-03）：spec 默认体注入的类型注解未经过首遍展开（spec 声明不在
    // `items` 里），这里再展开一遍——只影响刚注入的方法，普通类型第二遍为空转。
    for d in expand_type_aliases(&mut items, &aliases, profile.is_auto_language()) {
        eng.cx.diags.push((d.span.start as u64, d));
    }
    if let Some(first) = eng.cx.first_diag() {
        return Err(vec![first]);
    }
    if items.is_empty() {
        // 空程序区分（旧合同）：无任何非 trivia token → "expected `fn`, found end
        // of file"（span=EOF）；只有分隔符 token → Ok(empty)。
        let leaves = lex::collect_leaves(tree);
        if leaves.is_empty() {
            let end = snap.len() as usize;
            return Err(vec![Diagnostic::new(
                Stage::Text,
                "source.syntax",
                "expected `fn`, found end of file",
                Span { start: end, end },
            )]);
        }
    }
    Ok((
        Program { items },
        SourceMetadata {
            pragmas,
            annotations: eng.cx.annotations,
        },
    ))
}

pub(crate) fn attributed_owner(p: &GrammarProgram, tree: &CstTree, node: u32) -> Option<u32> {
    let rule = tree
        .kind(node)
        .checked_sub(ace_syntax_runtime::program::RULE_KIND_BASE)?;
    if !matches!(
        p.rules.get(rule as usize)?.name.as_str(),
        "attributed-item" | "attributed-method"
    ) {
        return None;
    }
    let owner = tree.field_child(node, p.field_id("owner")?)?;
    tree.children(owner)
        .into_iter()
        .find(|&child| tree.node(child).flags & FLAG_TRIVIA == 0)
}

fn attribute_entries(
    program: &GrammarProgram,
    tree: &CstTree,
    wrapper: u32,
) -> Vec<u32> {
    let kind = |name: &str| {
        program
            .rules
            .iter()
            .position(|rule| rule.name == name)
            .map(|index| ace_syntax_runtime::program::RULE_KIND_BASE + index as u32)
    };
    let attribute_kind = kind("declaration-attribute");
    let tail_kind = kind("attribute-entry-tail");
    let entry_kind = kind("attribute-entry");
    let mut entries = Vec::new();
    for attribute in tree.children(wrapper) {
        if Some(tree.kind(attribute)) != attribute_kind {
            continue;
        }
        for child in tree.children(attribute) {
            if Some(tree.kind(child)) == entry_kind {
                entries.push(child);
            } else if Some(tree.kind(child)) == tail_kind {
                entries.extend(
                    tree.children(child)
                        .into_iter()
                        .filter(|&entry| Some(tree.kind(entry)) == entry_kind),
                );
            }
        }
    }
    entries
}

pub(crate) fn attribute_targets_av(
    p: &GrammarProgram,
    tree: &CstTree,
    snap: &Snapshot,
    wrapper: u32,
) -> bool {
    let Some(name_field) = p.field_id("name") else { return true };
    let mut native_only = false;
    for entry in attribute_entries(p, tree, wrapper) {
        let Some(name) = tree.field_child(entry, name_field) else { continue };
        let (start, end) = tree.abs_range(name);
        match snap.read_str(start, end).as_deref() {
            Ok("vm") => return true,
            Ok("c" | "rs") => native_only = true,
            _ => {}
        }
    }
    !native_only
}

/// 子树是否含 error/missing 节点（迭代下降）。
fn subtree_has_errors(tree: &CstTree, root: u32) -> bool {
    use ace_syntax_runtime::cst::{FLAG_ERROR, FLAG_MISSING};
    let mut stack = vec![root];
    while let Some(n) = stack.pop() {
        let nd = tree.node(n);
        if nd.flags & (FLAG_ERROR | FLAG_MISSING) != 0 {
            return true;
        }
        for c in tree.children(n) {
            stack.push(c);
        }
    }
    false
}

/// 顶层不被支持的纯 ident 单词（旧 TOPLEVEL_UNSUPPORTED_WORDS 的 ident 部分）。
const TOPLEVEL_IDENT_WORDS: &[&str] = &[
    "struct", "trait", "impl", "pub", "checked", "unsafe", "macro", "import", "package",
];

/// 结构层首错映射：文档序最早的 error/missing 位置 → 旧诊断（首切上下文
/// 分类：item 位 / stmt 位 / expr 位；profile 特定消息由能力事实驱动）。
/// 旧 expect_ident 的 host-console 保留名拒绝（print/eprint/say）。
fn console_name_check(cx: &mut Cx, name: &Ident) {
    if cx.profile == SourceProfile::HostConsole
        && (crate::source::console::is_reserved_name(&name.name)
            || crate::source::platform_io::is_reserved_name(&name.name))
    {
        let span = name.span;
        cx.reject(span, move || {
            Diagnostic::new(
                Stage::Text,
                "source.console-reserved",
                "console names cannot be bound in the host-console profile",
                span,
            )
        });
    }
}

fn structural_first_diag(cx: &Cx, tree: &CstTree) -> Option<Diagnostic> {
    use ace_syntax_runtime::cst::{FLAG_ERROR, FLAG_MISSING};
    // 候选：带内容（len>0）的 ERROR 节点优先（恢复路径在区域起点插入的
    // 零宽 MISSING 标记不携带位置事实，仅作无 ERROR 候选时的后备）。
    let mut best_err: Option<u32> = None;
    let mut best_missing: Option<u32> = None;
    let mut stack = vec![tree.root];
    while let Some(n) = stack.pop() {
        let nd = tree.node(n);
        let flags = nd.flags & (FLAG_ERROR | FLAG_MISSING);
        if flags != 0 {
            let (s, e) = tree.abs_range(n);
            if flags & FLAG_ERROR != 0 && e > s {
                let better = match best_err {
                    None => true,
                    Some(b) => (s, e) < tree.abs_range(b),
                };
                if better {
                    best_err = Some(n);
                }
            } else {
                let better = match best_missing {
                    None => true,
                    Some(b) => (s, e) < tree.abs_range(b),
                };
                if better {
                    best_missing = Some(n);
                }
            }
            continue;
        }
        for c in tree.children(n) {
            stack.push(c);
        }
    }
    let node = best_err.or(best_missing)?;
    // 期望序列游走（首切）：对 item/stmt 位的错误区域按旧 parser 的期望
    // 序列走叶，报告**分歧点**（旧 parser 在第一个不满足期望的 token 报错）。
    if let Some(d) = walk_expectations(cx, tree, node) {
        return Some(d);
    }
    // 消息定位用区域**首个叶**（旧 parser 在 token 上报错）；missing 节点
    // （零宽）用自身位置。
    let leaf = first_meaningful_leaf(cx, tree, node);
    let span = match leaf {
        Some(l) => cx.span_of(l),
        None => {
            let (s, e) = tree.abs_range(node);
            Span {
                start: s as usize,
                end: e as usize,
            }
        }
    };
    let mut path: Vec<u32> = Vec::new();
    if !path_to(tree, tree.root, node, &mut path) {
        return Some(Diagnostic::new(
            Stage::Text,
            "source.syntax",
            "source contains a syntax error",
            span,
        ));
    }
    let leaf = first_meaningful_leaf(cx, tree, node);
    let text = leaf.map(|l| cx.spelling(l)).unwrap_or_default();
    let kind = leaf.map(|l| tree.kind(l));
    let k = cx.k;
    let is_kw = kind.is_some_and(|g| {
        [
            k.tok_let,
            k.tok_var,
            k.tok_if,
            k.tok_else,
            k.tok_while,
            k.tok_loop,
            k.tok_break,
            k.tok_continue,
            k.tok_return,
            k.tok_true,
            k.tok_false,
            k.tok_for,
            k.tok_int_kw,
            k.tok_i32,
            k.tok_bool,
            k.tok_type_kw,
            k.tok_static,
            k.tok_mut,
            k.tok_enum_kw,
            k.tok_is,
            k.tok_in,
        ]
        .contains(&g)
    });
    let word_msg = |w: &str| {
        Diagnostic::new(
            Stage::Text,
            "source.unsupported",
            format!("`{}` is not part of this source profile", w),
            span,
        )
    };
    let at_root = (path.iter().any(|&g| g == cx.k.item)
        && !path
            .iter()
            .any(|&g| g == cx.k.script_item.unwrap_or(u32::MAX)))
        || path.len() <= 1; // 根直挂尾随 ERROR 叶（路径仅含 program kind）
    let mut in_stmt = path
        .iter()
        .any(|&g| g == cx.k.stmt || g == cx.k.block || g == cx.k.script_item.unwrap_or(u32::MAX));
    let in_expr = path.iter().any(|&g| {
        cx.k.expression_kind(g) == cx.k.expr || cx.k.expression_kind(g) == cx.k.unary_expr
    });
    if cx.profile.is_entry_unit() && !in_expr {
        // entry 面：items() 把一切非 `fn` 顶层 token 路由进脚本语句序列
        // （旧 script_item → parse_stmt_head）——顶层错误即语句位。
        in_stmt = true;
    }
    if let Some(g) = kind {
        if g == cx.k.tok_while && cx.profile.surface_is_scalar_or_wider() && in_stmt {
            return Some(Diagnostic::new(
                Stage::Text,
                "source.unsupported",
                "`while` is not part of this source profile; conditional loops use `for <condition> { ... }`",
                span,
            ));
        }
        if g == cx.k.tok_for && !cx.profile.surface_is_scalar_or_wider() && in_stmt {
            return Some(word_msg("for"));
        }
        if g == cx.k.tok_static && at_root {
            return Some(Diagnostic::new(
                Stage::Text,
                "source.unsupported",
                "`static` is only legal before `fn` inside a `type` block of the record profile",
                span,
            ));
        }
    }
    if kind == Some(cx.k.tok_minus) && in_expr && !cx.profile.surface_is_scalar_or_wider() {
        let end = leaf_end(cx, tree, node).max(span.end);
        return Some(Diagnostic::new(
            Stage::Text,
            "source.unsupported",
            "unary minus is only supported directly before an integer literal in this source profile",
            Span { start: span.start, end },
        ));
    }
    // 关键字：item 位 = "`X` is not part..."；stmt/script 位 = "expected
    // statement, found X"（旧 parse_stmt_head 只有 Reserved 走 unsupported）。
    if is_kw || text == "fn" {
        if in_stmt {
            let found = describe_leaf_text(cx, tree, leaf);
            return Some(Diagnostic::new(
                Stage::Text,
                "source.syntax",
                format!("expected statement, found {}", found),
                span,
            ));
        }
        if !in_expr {
            return Some(word_msg(&text));
        }
    }
    if kind == Some(cx.k.tok_r_move) || kind == Some(cx.k.tok_r_to) {
        if !in_expr {
            return Some(word_msg(&text));
        }
        return Some(Diagnostic::new(
            Stage::Text,
            "source.unsupported",
            format!("keyword `{}` cannot be used as a name", text),
            span,
        ));
    }
    // 其余保留 token（use/enum/view/...）：item 位 = "`X` is not part..."
    //（旧 items() 对 Reserved 的拒绝面）；stmt/expr 位由上方 keyword 分支
    // 与表达式游走按各自消息处理。
    if kind.is_some_and(|g| cx.k.reserved_toks.contains(&g)) && !in_stmt && !in_expr {
        return Some(word_msg(&text));
    }
    if let Some(g) = kind {
        let describe = if g == cx.k.tok_affix_num && !cx.profile.is_numeric() {
            // PLAN-961: the numeric profile's typed literals are decoded
            // in the expression walk — never a raw rejection there.
            Some("suffixed/radix/float numeric literal")
        } else if g == cx.k.tok_str_lit {
            Some("string literal")
        } else if g == cx.k.tok_char_lit {
            Some("char literal")
        } else if g == cx.k.tok_block_open {
            Some("block comment")
        } else {
            None
        };
        if let Some(desc) = describe {
            let quoted = if g == cx.k.tok_affix_num || g == cx.k.tok_block_open {
                format!("{} is not part of this source profile", desc)
            } else {
                format!("{} `{}` is not part of this source profile", desc, text)
            };
            return Some(Diagnostic::new(
                Stage::Text,
                "source.unsupported",
                quoted,
                span,
            ));
        }
    }
    if kind == Some(cx.k.tok_ident) && at_root {
        if TOPLEVEL_IDENT_WORDS.contains(&text.as_str()) {
            return Some(word_msg(&text));
        }
        return Some(Diagnostic::new(
            Stage::Text,
            "source.syntax",
            format!("expected `fn`, found `{}`", text),
            span,
        ));
    }
    let found = describe_leaf_text(cx, tree, leaf);
    if in_stmt || in_expr {
        return Some(Diagnostic::new(
            Stage::Text,
            "source.syntax",
            format!("expected statement, found {}", found),
            span,
        ));
    }
    Some(Diagnostic::new(
        Stage::Text,
        "source.syntax",
        format!("expected `fn`, found {}", found),
        span,
    ))
}

/// 错误区域之后首个 token 的终点（旧 `span.join(end)` 形态的近似）。
fn leaf_end(_cx: &Cx, tree: &CstTree, node: u32) -> usize {
    let leaves = lex::collect_leaves(tree);
    let (s, _) = tree.abs_range(node);
    let mut best: Option<usize> = None;
    for l in &leaves {
        if l.start >= s + 1 {
            best = Some(l.end as usize);
            break;
        }
    }
    best.unwrap_or(s as usize)
}

// ---------------------------------------------------------------- 期望序列游走
//
// 错误区域的叶序列按旧 parser 的期望顺序游走：第一个不满足期望的叶 =
// 旧首错位置。这是**诊断映射**（CST 叶事实 → 旧消息），不是第二份语法
// 实现——接受/拒绝决定仍由规则程序与投影能力层做。

/// 错误区域内的非 trivia 叶（文档序，含 error 叶）。
fn region_leaves(cx: &Cx, tree: &CstTree, node: u32) -> Vec<(u32, u32, String)> {
    let mut out = Vec::new();
    let mut stack = vec![node];
    // 先收集再排序（迭代下降）。
    let mut all: Vec<u32> = Vec::new();
    while let Some(n) = stack.pop() {
        let nd = tree.node(n);
        if nd.flags & FLAG_TRIVIA != 0 {
            continue;
        }
        if nd.child_count == 0 {
            // 恢复标记（零宽 MISSING 叶）不携带源码字节——跳过。
            let (s, e) = tree.abs_range(n);
            if e > s {
                all.push(n);
            }
            continue;
        }
        let kids = tree.children(n);
        for c in kids.into_iter().rev() {
            stack.push(c);
        }
    }
    all.sort_by_key(|&n| tree.abs_range(n).0);
    for n in all {
        out.push((n, tree.kind(n), cx.spelling(n)));
    }
    out
}

/// 全树非 trivia 叶（根的孩子经树级 root_edges 目录起种）。
fn tree_leaves(cx: &Cx, tree: &CstTree) -> Vec<(u32, u32, String)> {
    let mut out = Vec::new();
    let mut stack = tree.children(tree.root);
    let mut all: Vec<u32> = Vec::new();
    while let Some(n) = stack.pop() {
        let nd = tree.node(n);
        if nd.flags & FLAG_TRIVIA != 0 {
            continue;
        }
        if nd.child_count == 0 {
            let (s, e) = tree.abs_range(n);
            if e > s {
                all.push(n);
            }
            continue;
        }
        let kids = tree.children(n);
        for c in kids.into_iter().rev() {
            stack.push(c);
        }
    }
    all.sort_by_key(|&n| tree.abs_range(n).0);
    for n in all {
        out.push((n, tree.kind(n), cx.spelling(n)));
    }
    out
}

fn walk_expectations(cx: &Cx, tree: &CstTree, node: u32) -> Option<Diagnostic> {
    let region = region_leaves(cx, tree, node);
    if region.is_empty() {
        return None;
    }
    // 旧 parser 无"区域"概念——线性走到文件尾。恢复截断（sync 停在 `}`）
    // 会把区域外的 token 丢出叶列，导致多级嵌套块在区域边界误判 EOF。
    // 因此游走叶列 = 从区域首叶起的全树后缀（文档序），跨区域边界继续。
    // PLAN-928（T-04）：区域含于 fn 声明内时自**该声明头**重根（旧解析
    // 从声明头线性走——签名/前体能力拒绝先于体内错误按位置胜出）。
    let mut first_start = tree.abs_range(region[0].0).0;
    {
        // path_to 收集的是 kind——这里需要**节点**路径找最近 fn-item 祖先。
        let mut node_path: Vec<u32> = Vec::new();
        if node_path_to(tree, tree.root, node, &mut node_path) {
            let mut anc: Option<u32> = None;
            for &a in node_path.iter().rev() {
                if tree.kind(a) == cx.k.fn_item {
                    anc = Some(a);
                    break;
                }
            }
            if let Some(a) = anc {
                let ar = tree.abs_range(a);
                if let Some(&(l, _, _)) = tree_leaves(cx, tree).iter().find(|&&(l, _, _)| {
                    let r = tree.abs_range(l);
                    r.0 >= ar.0 && r.1 <= ar.1
                }) {
                    first_start = first_start.min(tree.abs_range(l).0);
                }
            }
        }
    }
    let mut leaves = tree_leaves(cx, tree);
    assert!(
        !leaves.is_empty(),
        "tree leaves empty but region had leaves"
    );
    leaves.retain(|(n, _, _)| tree.abs_range(*n).0 >= first_start);
    // 区域截断（恢复停在 sync）时，其后的首 token 供 fn-tail 期望游走使用
    //（旧 parser 在区域后第一个 token 上报"expected return type"等）。
    let region_end = tree.abs_range(node).1;
    let beyond: Option<(u32, u64, u64, String)> = lex::collect_leaves(tree)
        .into_iter()
        .filter(|l| {
            l.start >= region_end && !cx.spelling_by_range(l.start, l.end).trim().is_empty()
        })
        .min_by_key(|l| l.start)
        .map(|l| (l.kind, l.start, l.end, cx.spelling_by_range(l.start, l.end)));
    let first = &leaves[0];
    // item 形态：`fn ...` / `type ...`。
    if first.2 == "fn" {
        return walk_fn_item(
            cx,
            &leaves,
            beyond.as_ref().map(|b| {
                (
                    b.0,
                    b.3.as_str(),
                    Span {
                        start: b.1 as usize,
                        end: b.2 as usize,
                    },
                )
            }),
        );
    }
    if first.2 == "type" && cx.profile.is_record() && !cx.profile.is_closures() {
        return walk_type_item(cx, &leaves);
    }
    // stmt 形态（块内错误区域）：完整语句期望游走（与旧 parse_stmt_head
    // 同序——return/if/for/赋值/表达式语句各自镜像）。仅语句位区域——
    // item 位（块/脚本外）由叶层分类器按旧 items() 消息处理。
    let mut path_kinds: Vec<u32> = Vec::new();
    let _ = path_to(tree, tree.root, node, &mut path_kinds);
    let in_stmt_ctx = path_kinds
        .iter()
        .any(|&g| g == cx.k.stmt || g == cx.k.block || g == cx.k.script_item.unwrap_or(u32::MAX))
        || cx.profile.is_entry_unit();
    if in_stmt_ctx && leaves.len() >= 1 {
        let mut w = Walker::new(cx, &leaves);
        if let Some(d) = w.walk_stmt_head(false) {
            return Some(d);
        }
    }
    None
}

fn leaf_span(cx: &Cx, l: u32) -> Span {
    cx.span_of(l)
}

/// PLAN-928：叶级 fn 型跳跃（closures 面 ret/ann 位；`fn ( 词, ... ) [->] 词|嵌套fn型`）。
/// 入参 `i` 指向 `fn` 词；返回越过整个 fn 型后的下标。
fn skip_fn_type_leaves(leaves: &[(u32, u32, String)], mut i: usize) -> usize {
    i += 1; // `fn`
    if i < leaves.len() && leaves[i].2 == "(" {
        i += 1;
        let mut depth = 1usize;
        while i < leaves.len() && depth > 0 {
            if leaves[i].2 == "(" {
                depth += 1;
            } else if leaves[i].2 == ")" {
                depth -= 1;
            }
            i += 1;
        }
    }
    if i < leaves.len() && leaves[i].2 == "->" {
        i += 1;
    }
    if i < leaves.len() && leaves[i].2 == "fn" {
        return skip_fn_type_leaves(leaves, i);
    }
    i + 1
}

/// 旧 fn_item_tail 期望序列（fn 关键字已匹配）。`beyond` = 区域后首 token
/// (kind, spelling)——恢复截断在 `)` 时，结果位错误在该 token 上。
fn walk_fn_item(
    cx: &Cx,
    leaves: &[(u32, u32, String)],
    beyond: Option<(u32, &str, Span)>,
) -> Option<Diagnostic> {
    walk_fn_item_ex(cx, leaves, beyond).1
}

/// 同 [`walk_fn_item`]，但成功走完时返回结束游标（`}` 后一位；相对
/// `leaves` 下标）——type-item 成员序列游走续位用。
fn walk_fn_item_ex(
    cx: &Cx,
    leaves: &[(u32, u32, String)],
    beyond: Option<(u32, &str, Span)>,
) -> (usize, Option<Diagnostic>) {
    let k = cx.k;
    let mut i = 1usize; // 0 = `fn`
    let at = |idx: usize| -> (Span, u32, &str) {
        let (n, g, s) = &leaves[idx.min(leaves.len() - 1)];
        (leaf_span(cx, *n), *g, s.as_str())
    };
    // 1) 函数名。
    if i >= leaves.len() {
        return (i, Some(eof_expected(cx, "function name")));
    }
    {
        if cx.profile.is_aggregates()
            && i < leaves.len()
            && matches!(leaves[i].2.as_str(), "?" | "!" | "~")
        {
            i += 1;
            if i >= leaves.len() {
                return (i, Some(eof_expected(cx, "return type")));
            }
        }
        let (sp, g, text) = at(i);
        if g != k.tok_ident {
            if g == k.tok_affix_num
                || g == k.tok_str_lit
                || g == k.tok_char_lit
                || g == k.tok_block_open
            {
                return (
                    i,
                    Some(Diagnostic::new(
                        Stage::Text,
                        "source.unsupported",
                        format!(
                            "{} cannot be used as a name in this source profile",
                            raw_describe(g, text, &k)
                        ),
                        sp,
                    )),
                );
            }
            if g == k.tok_r_move || g == k.tok_r_to || is_kw_leaf(g, &k) || text == "fn" {
                return (
                    i,
                    Some(Diagnostic::new(
                        Stage::Text,
                        "source.unsupported",
                        format!("keyword `{}` cannot be used as a name", text),
                        sp,
                    )),
                );
            }
            return (
                i,
                Some(Diagnostic::new(
                    Stage::Text,
                    "source.syntax",
                    format!("expected function name, found {}", describe_g(g, text, &k)),
                    sp,
                )),
            );
        }
        i += 1;
    }
    // 2) `(`。
    if i >= leaves.len() {
        return (i, Some(eof_expected(cx, "`(`")));
    }
    {
        if cx.profile.is_aggregates()
            && i < leaves.len()
            && matches!(leaves[i].2.as_str(), "?" | "!" | "~")
        {
            i += 1;
            if i >= leaves.len() {
                return (i, Some(eof_expected(cx, "return type")));
            }
        }
        let (sp, g, text) = at(i);
        if text != "(" {
            return (
                i,
                Some(Diagnostic::new(
                    Stage::Text,
                    "source.syntax",
                    format!("expected `(`, found {}", describe_g(g, text, &k)),
                    sp,
                )),
            );
        }
        i += 1;
    }
    // 3) 参数序列。
    if i < leaves.len() && leaves[i].2 != ")" {
        loop {
            // mode 词（可选）。
            if i < leaves.len() {
                let (_, g, text) = at(i);
                let is_move = g == k.tok_r_move;
                let is_mut = text == "mut";
                // PLAN-928：closures 面的 move 参数（`fn make(move s Str)`）。
                let admits_move = cx.profile.is_text() || cx.profile.is_closures();
                let admits_mut = cx.profile.is_text() || cx.profile.is_record();
                if (is_move && admits_move) || (is_mut && admits_mut) {
                    i += 1;
                }
            }
            // 参数名。
            if i >= leaves.len() {
                return (i, Some(eof_expected(cx, "parameter name")));
            }
            {
                let (sp, g, text) = at(i);
                if g != k.tok_ident {
                    if g == k.tok_r_move || g == k.tok_r_to || is_kw_leaf(g, &k) || text == "fn" {
                        return (
                            i,
                            Some(Diagnostic::new(
                                Stage::Text,
                                "source.unsupported",
                                format!("keyword `{}` cannot be used as a name", text),
                                sp,
                            )),
                        );
                    }
                    if g == k.tok_affix_num
                        || g == k.tok_str_lit
                        || g == k.tok_char_lit
                        || g == k.tok_block_open
                    {
                        return (
                            i,
                            Some(Diagnostic::new(
                                Stage::Text,
                                "source.unsupported",
                                format!(
                                    "{} cannot be used as a name in this source profile",
                                    raw_describe(g, text, &k)
                                ),
                                sp,
                            )),
                        );
                    }
                    return (
                        i,
                        Some(Diagnostic::new(
                            Stage::Text,
                            "source.syntax",
                            format!("expected parameter name, found {}", describe_g(g, text, &k)),
                            sp,
                        )),
                    );
                }
                i += 1;
            }
            // PLAN-928（T-03）：collections/collections 参数注解的可选糖 `?T`。
            if cx.profile.is_collections() {
                if i < leaves.len() && leaves[i].2 == "?" {
                    i += 1;
                }
            }
            // 参数类型词。
            if let Some(d) = walk_type_word(
                cx,
                leaves,
                &mut i,
                "parameter type annotation (`int`/`i32`/`bool`) is required",
            ) {
                return (i, Some(d));
            }
            // PLAN-928（T-03）：collections/closures 参数注解的参数化类型
            // `List<T>` / `Result<T, E>`（平衡 `<...>` 跳跃；`>` 后无逗号
            // 嵌套——type-arg 内嵌套泛型由同一深度计数覆盖）。
            if cx.profile.is_collections() && i < leaves.len() && leaves[i].2 == "<" {
                let mut depth = 0usize;
                while i < leaves.len() {
                    if leaves[i].2 == "<" {
                        depth += 1;
                    } else if leaves[i].2 == ">" {
                        depth -= 1;
                        if depth == 0 {
                            i += 1;
                            break;
                        }
                    }
                    i += 1;
                }
            }
            // `,` 或 `)`。
            if i >= leaves.len() {
                return (i, Some(eof_expected(cx, "`,` or `)`")));
            }
            let (sp, g, text) = at(i);
            if text == "," {
                i += 1;
                if i < leaves.len() && leaves[i].2 == ")" {
                    break;
                }
                continue;
            }
            if text == ")" {
                break;
            }
            return (
                i,
                Some(Diagnostic::new(
                    Stage::Text,
                    "source.syntax",
                    format!("expected `,` or `)`, found {}", describe_g(g, text, &k)),
                    sp,
                )),
            );
        }
    }
    // 4) `)`。
    if i >= leaves.len() {
        return (i, Some(eof_expected(cx, "`)`")));
    }
    {
        let (sp, _g, text) = at(i);
        if text != ")" {
            return (
                i,
                Some(Diagnostic::new(
                    Stage::Text,
                    "source.syntax",
                    format!("expected `)`, found {}", text),
                    sp,
                )),
            );
        }
        i += 1;
    }
    let name_span = leaf_span(cx, leaves[1].0);
    let rparen_span = leaf_span(cx, leaves[i - 1].0);
    // 5) 结果类型。
    if i >= leaves.len() {
        let Some((bg, btext, bspan)) = beyond else {
            return (i, Some(eof_expected(cx, "return type")));
        };
        let sp = bspan;
        let builtin = btext == "int" || btext == "i32" || btext == "bool";
        let text_word =
            bg == cx.k.tok_ident && cx.profile.is_text() && matches!(btext, "str" | "Str" | "char");
        // PLAN-928（T-04）：collections 词域——`Str` 收（char 仅 closures），
        // 小写 `str`/`char` 具名拒绝（镜像旧 collections_type）。
        let coll_word = bg == cx.k.tok_ident
            && cx.profile.is_collections()
            && (btext == "Str" || (btext == "char" && cx.profile.is_closures()));
        let coll_reject = bg == cx.k.tok_ident
            && cx.profile.is_collections()
            && (btext == "str" || (btext == "char" && !cx.profile.is_closures()));
        let entry_void = bg == cx.k.tok_ident && cx.profile.is_entry_unit() && btext == "void";
        let record_name = bg == cx.k.tok_ident && cx.profile.is_record();
        // collections 词域拒绝先于名义接受门（str/char 命名拒绝不得被
        // record_name 的 ident 接受吞掉）。
        if coll_reject {
            return (
                i,
                Some(Diagnostic::new(
                    Stage::Text,
                    "source.unsupported",
                    format!("type `{}` is not part of this source profile", btext),
                    sp,
                )),
            );
        }
        if !builtin && !text_word && !entry_void && !record_name && !coll_word {
            if btext == "{" {
                return (i, Some(Diagnostic::new(
                    Stage::Text,
                    "source.unsupported",
                    "function return type annotation (`int`/`i32`/`bool` or a record type) is required",
                    name_span.join(rparen_span),
                )));
            }
            if bg == cx.k.tok_ident || bg == cx.k.tok_r_move || bg == cx.k.tok_r_to {
                return (
                    i,
                    Some(Diagnostic::new(
                        Stage::Text,
                        "source.unsupported",
                        format!("type `{}` is not part of this source profile", btext),
                        sp,
                    )),
                );
            }
            return (
                i,
                Some(Diagnostic::new(
                    Stage::Text,
                    "source.syntax",
                    format!("expected return type, found `{}`", btext),
                    sp,
                )),
            );
        }
        return (i, Some(eof_expected(cx, "return type")));
    }
    {
        if cx.profile.is_aggregates()
            && i < leaves.len()
            && matches!(leaves[i].2.as_str(), "?" | "!" | "~")
        {
            i += 1;
            if i >= leaves.len() {
                return (i, Some(eof_expected(cx, "return type")));
            }
        }
        let (sp, g, text) = at(i);
        // PLAN-928：closures 面 ret 位 fn 型（`fn(T0, ...) [->] R`，可嵌套）。
        if cx.profile.is_closures() && text == "fn" {
            i = skip_fn_type_leaves(leaves, i);
        } else {
            // 旧 fn_item_tail 结果位分类：int/i32/bool 全域；text 面 ident 仅
            // str/Str/char；entry 面 ident 仅 void；record 面任意 ident（名义）。
            let builtin = text == "int" || text == "i32" || text == "bool";
            let text_word =
                g == k.tok_ident && cx.profile.is_text() && matches!(text, "str" | "Str" | "char");
            // PLAN-928（T-04）：collections 词域（同上）。
            let coll_word = g == k.tok_ident
                && cx.profile.is_collections()
                && (text == "Str" || (text == "char" && cx.profile.is_closures()));
            let coll_reject = g == k.tok_ident
                && cx.profile.is_collections()
                && (text == "str" || (text == "char" && !cx.profile.is_closures()));
            let entry_void = g == k.tok_ident && cx.profile.is_entry_unit() && text == "void";
            let record_name = g == k.tok_ident && cx.profile.is_record();
            if coll_reject {
                return (
                    i,
                    Some(Diagnostic::new(
                        Stage::Text,
                        "source.unsupported",
                        format!("type `{}` is not part of this source profile", text),
                        sp,
                    )),
                );
            }
            if !builtin && !text_word && !entry_void && !record_name && !coll_word {
                if text == "{" {
                    return (i, Some(Diagnostic::new(
                    Stage::Text,
                    "source.unsupported",
                    "function return type annotation (`int`/`i32`/`bool` or a record type) is required",
                    name_span.join(rparen_span),
                )));
                }
                if g == k.tok_ident || g == k.tok_r_move || g == k.tok_r_to {
                    return (
                        i,
                        Some(Diagnostic::new(
                            Stage::Text,
                            "source.unsupported",
                            format!("type `{}` is not part of this source profile", text),
                            sp,
                        )),
                    );
                }
                return (
                    i,
                    Some(Diagnostic::new(
                        Stage::Text,
                        "source.syntax",
                        format!("expected return type, found {}", describe_g(g, text, &k)),
                        sp,
                    )),
                );
            }
            i += 1;
            if cx.profile.is_aggregates() && i < leaves.len() && leaves[i].2 == "<" {
                let mut depth = 0usize;
                while i < leaves.len() {
                    if leaves[i].2 == "<" {
                        depth += 1;
                    } else if leaves[i].2 == ">" {
                        depth = depth.saturating_sub(1);
                        if depth == 0 {
                            i += 1;
                            break;
                        }
                    }
                    i += 1;
                }
            }
        }
    }
    // 6) `{`（体）→ 语句序列游走。
    if i >= leaves.len() {
        return (i, Some(eof_expected(cx, "`{`")));
    }
    {
        if cx.profile.is_aggregates()
            && i < leaves.len()
            && matches!(leaves[i].2.as_str(), "?" | "!" | "~")
        {
            i += 1;
            if i >= leaves.len() {
                return (i, Some(eof_expected(cx, "return type")));
            }
        }
        let (sp, g, text) = at(i);
        if text != "{" {
            return (
                i,
                Some(Diagnostic::new(
                    Stage::Text,
                    "source.syntax",
                    format!("expected `{{`, found {}", describe_g(g, text, &k)),
                    sp,
                )),
            );
        }
        i += 1;
    }
    let mut w = Walker::new(cx, leaves);
    w.i = i;
    match w.walk_block_body(false) {
        Some(d) => (w.i, Some(d)),
        None => (w.i, None),
    }
}

/// 旧 EOF 期望：`expected X, found end of file` @ snap.len()（旧 eof_span）。
fn eof_expected(cx: &Cx, what: &str) -> Diagnostic {
    let e = cx.snap.len() as usize;
    Diagnostic::new(
        Stage::Text,
        "source.syntax",
        format!("expected {what}, found end of file"),
        Span { start: e, end: e },
    )
}

/// 旧 type_item 期望序列（type 关键字已匹配；record 族）——镜像旧
/// type_item() 成员循环：分隔纪律、field（self/可选 `:`/member_type
/// 分类/duplicate）、static fn / fn 方法（fn_item_tail 镜像）、`mut fn`
/// 与其他关键字拒绝、空 record 拒绝。
fn walk_type_item(cx: &Cx, leaves: &[(u32, u32, String)]) -> Option<Diagnostic> {
    let k = cx.k;
    let w = Walker::new(cx, leaves);
    let at = |idx: usize| -> (Span, u32, &str) {
        let (n, g, s) = &leaves[idx.min(leaves.len() - 1)];
        (leaf_span(cx, *n), *g, s.as_str())
    };
    if leaves.len() < 2 {
        return None;
    }
    {
        let (sp, g, text) = at(1);
        if g != k.tok_ident {
            return Some(Diagnostic::new(
                Stage::Text,
                "source.syntax",
                format!("expected type name, found {}", describe_g(g, text, &k)),
                sp,
            ));
        }
    }
    if leaves.len() <= 2 {
        return Some(Diagnostic::new(
            Stage::Text,
            "source.syntax",
            "expected `{`, found end of file".to_string(),
            at(1).0,
        ));
    }
    {
        let (sp, _g, text) = at(2);
        if text != "{" {
            return Some(Diagnostic::new(
                Stage::Text,
                "source.syntax",
                format!("expected `{{`, found {}", text),
                sp,
            ));
        }
    }
    let type_kw_span = at(0).0;
    let mut i = 3usize;
    let mut seen_fields: Vec<String> = Vec::new();
    let mut any_member = false;
    let mut any_field = false;
    let mut close_span = None;
    while i < leaves.len() {
        let (_, _g, text) = at(i);
        if text == ";" {
            i += 1;
            continue;
        }
        if text == "}" {
            close_span = Some(at(i).0);
            break;
        }
        // 分隔纪律（旧 733-764）：首成员可同行；此后须 `;` 后或换行后。
        if any_member {
            let prev_semi = at(i - 1).2 == ";";
            let cur_line = w.at(i).map(|c| c.3).unwrap_or(0);
            let prev_line = w.at(i - 1).map(|c| c.3).unwrap_or(0);
            if !prev_semi && cur_line <= prev_line {
                let (sp, gg, tt) = at(i);
                let mut diag = Diagnostic::new(
                    Stage::Text,
                    "source.syntax",
                    format!(
                        "expected `;`, newline or `}}`, found {}",
                        describe_g(gg, tt, &k)
                    ),
                    sp,
                );
                if let Some(fact) = surface_shape_of_token(tt) {
                    diag = diag.tagged(fact);
                }
                return Some(diag);
            }
        }
        any_member = true;
        match text {
            "static" => {
                if i + 1 >= leaves.len() || at(i + 1).2 != "fn" {
                    let sp = at((i + 1).min(leaves.len() - 1)).0;
                    return Some(Diagnostic::new(
                        Stage::Text,
                        "source.syntax",
                        "expected `fn` after `static`".to_string(),
                        sp,
                    ));
                }
                let (end, d) = walk_fn_item_ex(cx, &leaves[i + 1..], None);
                if let Some(d) = d {
                    return Some(d);
                }
                i += 1 + end;
            }
            "fn" => {
                let (end, d) = walk_fn_item_ex(cx, &leaves[i..], None);
                if let Some(d) = d {
                    return Some(d);
                }
                i += end;
            }
            "mut" => {
                return Some(Diagnostic::new(
                    Stage::Text,
                    "source.unsupported",
                    "`mut fn` (mutable instance methods) is not part of this profile; methods are readonly views".to_string(),
                    at(i).0,
                ));
            }
            _ => {
                let (sp, gg, tt) = at(i);
                if gg == k.tok_ident {
                    if tt == "self" {
                        return Some(Diagnostic::new(
                            Stage::Text,
                            "source.unsupported",
                            "`self` cannot be a field name".to_string(),
                            sp,
                        ));
                    }
                    let mut j = i + 1;
                    if j < leaves.len() && at(j).2 == ":" {
                        j += 1;
                    }
                    if j >= leaves.len() {
                        return None;
                    }
                    let (tsp, tg, ttext) = at(j);
                    let is_text = cx.profile.is_text();
                    if tg == k.tok_ident && !is_text && matches!(ttext, "str" | "Str" | "char") {
                        return Some(Diagnostic::new(
                            Stage::Text,
                            "source.unsupported",
                            format!("type `{}` is not part of this source profile", ttext),
                            tsp,
                        ));
                    }
                    let builtin = tg == k.tok_int_kw || tg == k.tok_i32 || tg == k.tok_bool;
                    if !(builtin || (tg == k.tok_ident && cx.profile.is_record())) {
                        return Some(Diagnostic::new(
                            Stage::Text,
                            "source.unsupported",
                            "field type annotation (`int`/`i32`/`bool` or a record type) is required".to_string(),
                            sp,
                        ));
                    }
                    if seen_fields.contains(&tt.to_string()) {
                        return Some(Diagnostic::new(
                            Stage::Text,
                            "source.duplicate",
                            format!("field `{}` is declared more than once", tt),
                            sp,
                        ));
                    }
                    seen_fields.push(tt.to_string());
                    any_field = true;
                    i = j + 1;
                } else if is_kw_leaf(gg, &k) {
                    return Some(Diagnostic::new(
                        Stage::Text,
                        "source.unsupported",
                        format!("`{}` is not a valid type member here", tt),
                        sp,
                    ));
                } else {
                    return Some(Diagnostic::new(
                        Stage::Text,
                        "source.syntax",
                        format!("expected field or method, found {}", describe_g(gg, tt, &k)),
                        sp,
                    ));
                }
            }
        }
    }
    if !any_field {
        // 空 record：kw..`}`（区域截断时以区域末叶代 `}`）。
        let end = close_span
            .or_else(|| leaves.last().map(|(n, _, _)| leaf_span(cx, *n)))
            .unwrap_or(type_kw_span);
        let span = type_kw_span.join(end);
        return Some(Diagnostic::new(
            Stage::Text,
            "source.unsupported",
            "empty records are not part of this profile; a record must declare at least one field"
                .to_string(),
            span,
        ));
    }
    None
}

// ---------------------------------------------------------------- 叶序列游走器
//
// 镜像旧 parser 的语句/表达式失败点（parse_stmt_head / expr 的帧机）：
// 只用于定位**首个**违反期望的叶并给出旧消息；不构造 AST。

struct Walker<'c, 'a> {
    cx: &'c Cx<'a>,
    leaves: &'c [(u32, u32, String)],
    i: usize,
    lines: Vec<u32>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Br {
    Paren,
    Call,
    Ctor,
}

impl<'c, 'a> Walker<'c, 'a> {
    fn new(cx: &'c Cx<'a>, leaves: &'c [(u32, u32, String)]) -> Self {
        // 行号：换行位置一次扫描 + 二分（后缀叶列可达全树——逐叶前缀计数
        // 是 O(n²)，567 语料差分会到分钟级）。
        let total = cx.snap.len();
        let text = cx.snap.read_str(0, total).unwrap_or_default();
        let bytes = text.as_bytes();
        let nl: Vec<u64> = bytes
            .iter()
            .enumerate()
            .filter(|(_, &b)| b == b'\n')
            .map(|(i, _)| i as u64)
            .collect();
        let mut lines = Vec::with_capacity(leaves.len());
        for (n, _, _) in leaves {
            let (s, e) = cx.tree.abs_range(*n);
            // 旧 lexer：多行 token 记结束行。
            let _ = s;
            lines.push(1 + nl.partition_point(|&x| x < e.min(total)) as u32);
        }
        Walker {
            cx,
            leaves,
            i: 0,
            lines,
        }
    }

    fn k(&self) -> &'a super::kinds::SourceKinds {
        self.cx.k
    }
    fn at(&self, idx: usize) -> Option<(Span, u32, &str, u32)> {
        let (n, g, s) = self.leaves.get(idx)?;
        Some((self.cx.span_of(*n), *g, s.as_str(), self.lines[idx]))
    }
    fn cur(&self) -> Option<(Span, u32, &str, u32)> {
        self.at(self.i)
    }
    fn eof_span(&self) -> Span {
        let end = self.cx.snap.len() as usize;
        Span { start: end, end }
    }
    fn syntax(&self, msg: String) -> Diagnostic {
        let span = self.cur().map(|c| c.0).unwrap_or_else(|| self.eof_span());
        Diagnostic::new(Stage::Text, "source.syntax", msg, span)
    }
    fn unsupported(&self, msg: String, span: Span) -> Diagnostic {
        // PLAN-969 (AUDIT-03): every projector "not part of this source
        // profile" refusal carries the structured surface fact.
        Diagnostic::new(Stage::Text, "source.unsupported", msg, span)
            .tagged(SurfaceFact::ProfileSurface)
    }
    fn describe_cur(&self) -> String {
        match self.cur() {
            Some((_, g, text, _)) => describe_g(g, text, self.k()),
            None => "end of file".to_string(),
        }
    }
    fn prev_line(&self) -> u32 {
        if self.i == 0 {
            1
        } else {
            self.lines[self.i - 1]
        }
    }
    fn is_kw(&self, g: u32, text: &str) -> bool {
        is_kw_leaf(g, self.k()) || text == "fn"
    }
    fn is_reserved(&self, g: u32) -> bool {
        g == self.k().tok_r_move || g == self.k().tok_r_to
    }
    fn is_raw(&self, g: u32) -> bool {
        let k = self.k();
        // PLAN-981：AGE 作者面合法使用浮点字面量，本面不把 affix-num 当裸 token。
        let affix = g == k.tok_affix_num && !self.cx.age_face;
        affix || g == k.tok_str_lit || g == k.tok_char_lit || g == k.tok_block_open
    }

    /// 块体：语句序列到 `}`（或 EOF 错误）。
    fn walk_block_body(&mut self, script_top: bool) -> Option<Diagnostic> {
        loop {
            // skip semis
            while matches!(self.cur(), Some((_, _, ";", _))) {
                self.i += 1;
            }
            match self.cur() {
                None => {
                    if script_top {
                        return None;
                    }
                    // 区域截断（恢复 sync 停在 `}`）：区域后首 token 代 cur——
                    // 旧语义是分隔纪律/块闭合检查，不是文件 EOF。
                    let region_end = self
                        .leaves
                        .last()
                        .map(|(n, _, _)| self.cx.tree.abs_range(*n).1)
                        .unwrap_or(0);
                    if let Some((g, text, span)) = self.cx.leaf_after(region_end) {
                        let k = self.k();
                        if text == "}" {
                            return None;
                        }
                        let cur_line = self.cx.line_of_pos(span.start as u64);
                        let prev_line = self
                            .leaves
                            .last()
                            .map(|(n, _, _)| self.cx.line_of_pos(self.cx.tree.abs_range(*n).0))
                            .unwrap_or(0);
                        if text != ";" && cur_line <= prev_line {
                            let mut diag = Diagnostic::new(
                                Stage::Text,
                                "source.syntax",
                                format!(
                                    "expected `;`, newline or `}}`, found {}",
                                    describe_g(g, &text, k)
                                ),
                                span,
                            );
                            if let Some(fact) = surface_shape_of_token(&text) {
                                diag = diag.tagged(fact);
                            }
                            return Some(diag);
                        }
                        return Some(Diagnostic::new(
                            Stage::Text,
                            "source.syntax",
                            format!(
                                "expected statement or `}}`, found {}",
                                describe_g(g, &text, k)
                            ),
                            span,
                        ));
                    }
                    return Some(
                        self.syntax("expected statement or `}`, found end of file".into()),
                    );
                }
                Some((_, _, "}", _)) => {
                    self.i += 1;
                    return None;
                }
                Some(_) => {
                    if let Some(d) = self.walk_stmt_head(script_top) {
                        return Some(d);
                    }
                }
            }
        }
    }

    /// 语句分隔检查（`;`/`}`/换行）。
    fn sep_check(&self) -> Option<Diagnostic> {
        match self.cur() {
            None => None,
            Some((_, _, ";", _)) | Some((_, _, "}", _)) => None,
            Some((_, _, _, line)) if line > self.prev_line() => None,
            Some((_, _, text, _)) => {
                let mut diag = self.syntax(format!(
                    "expected `;`, newline or `}}`, found {}",
                    self.describe_cur()
                ));
                if let Some(fact) = surface_shape_of_token(text) {
                    diag = diag.tagged(fact);
                }
                Some(diag)
            }
        }
    }

    fn walk_stmt_head(&mut self, script_top: bool) -> Option<Diagnostic> {
        let k = self.k();
        let (sp, g, text, _) = self.cur()?;
        let wide = self.cx.profile.surface_is_scalar_or_wider();
        match text {
            "let" | "var" => {
                let kw = sp;
                self.i += 1;
                // name
                let (nsp, ng, ntext, _) = match self.cur() {
                    Some(c) => c,
                    None => {
                        return Some(self.syntax("expected local name, found end of file".into()))
                    }
                };
                if ng != k.tok_ident {
                    return Some(self.name_error(ng, ntext, nsp, "local name"));
                }
                self.i += 1;
                // 注解
                if let Some((asp, ag, atext, _)) = self.cur() {
                    let is_type = atext == "int" || atext == "i32" || atext == "bool";
                    // PLAN-928（T-04）：collections 词域——Str/char(closures) 收，
                    // 小写 str/char 拒；text 面三词全收。
                    let coll_ok = self.cx.profile.is_collections()
                        && (atext == "Str" || (atext == "char" && self.cx.profile.is_closures()));
                    let admits_text = self.cx.profile.is_text();
                    if ag == k.tok_ident
                        && !admits_text
                        && !coll_ok
                        && matches!(atext, "str" | "Str" | "char")
                    {
                        return Some(self.unsupported(
                            format!("type `{}` is not part of this source profile", atext),
                            asp,
                        ));
                    }
                    let ident_type = ag == k.tok_ident
                        && ((admits_text && matches!(atext, "str" | "Str" | "char"))
                            || coll_ok
                            || self.cx.profile.is_entry_unit() && atext == "void"
                            || self.cx.profile.is_record());
                    if is_type || ident_type {
                        self.i += 1;
                    } else if ag == k.tok_ident || self.is_reserved(ag) {
                        return Some(self.unsupported(
                            format!("type `{}` is not part of this source profile", atext),
                            asp,
                        ));
                    }
                }
                // `=`
                match self.cur() {
                    Some((_, _, "=", _)) => {
                        self.i += 1;
                    }
                    _ => {
                        let last_end = self.at(self.i - 1).map(|c| c.0.end).unwrap_or(kw.end);
                        return Some(
                            self.unsupported(
                                "locals must be initialized at declaration in this source profile"
                                    .into(),
                                Span {
                                    start: kw.start,
                                    end: last_end,
                                },
                            ),
                        );
                    }
                }
                if let Some(d) = self.walk_expr(false) {
                    return Some(d);
                }
                self.sep_check()
            }
            "return" => {
                self.i += 1;
                if script_top {
                    return Some(Diagnostic::new(
                        Stage::Text,
                        "source.entry-top-return",
                        "a script's result is its trailing expression; `return` is not part of the script form (use a declared `fn main` for an explicit exit)",
                        sp,
                    ));
                }
                if let Some(d) = self.walk_expr(false) {
                    return Some(d);
                }
                self.sep_check()
            }
            "if" => {
                self.i += 1;
                if let Some(d) = self.walk_condition() {
                    return Some(d);
                }
                if let Some(d) = self.expect_text("{") {
                    return Some(d);
                }
                if let Some(d) = self.walk_block_body(false) {
                    return Some(d);
                }
                if matches!(self.cur(), Some((_, _, "else", _))) {
                    self.i += 1;
                    if let Some(d) = self.expect_text("{") {
                        return Some(d);
                    }
                    if let Some(d) = self.walk_block_body(false) {
                        return Some(d);
                    }
                }
                self.sep_check()
            }
            "while" => {
                if wide {
                    return Some(self.unsupported(
                        "`while` is not part of this source profile; conditional loops use `for <condition> { ... }`".into(),
                        sp,
                    ));
                }
                self.i += 1;
                if let Some(d) = self.walk_condition() {
                    return Some(d);
                }
                if let Some(d) = self.expect_text("{") {
                    return Some(d);
                }
                if let Some(d) = self.walk_block_body(false) {
                    return Some(d);
                }
                self.sep_check()
            }
            "for" => {
                // PLAN-928（T-03）：collections/closures 面 `for x in <list>`。
                if self.cx.profile.is_collections() {
                    self.i += 1;
                    match self.cur() {
                        Some((_, ng, _, _)) if ng == k.tok_ident => {
                            self.i += 1;
                        }
                        _ => {
                            let (spn, gn, tn, _) = match self.cur() {
                                Some(c) => c,
                                None => {
                                    return Some(self.syntax(
                                        "expected iteration binding, found end of file".into(),
                                    ))
                                }
                            };
                            return Some(self.name_error(gn, tn, spn, "iteration binding"));
                        }
                    }
                    match self.cur() {
                        Some((_, _, "in", _)) => {
                            self.i += 1;
                        }
                        _ => {
                            return Some(self.syntax(format!(
                                "expected `in` after the `for` binding, found {}",
                                self.describe_cur()
                            )));
                        }
                    }
                    // 合同 C7：迭代器是裸 List 局部名（非表达式）。
                    match self.cur() {
                        Some((_, ig, _, _)) if ig == k.tok_ident => {
                            self.i += 1;
                        }
                        _ => {
                            let (spn, gn, tn, _) = match self.cur() {
                                Some(c) => c,
                                None => return Some(self.syntax("expected for-in iterator (a List local), found end of file".into())),
                            };
                            return Some(self.name_error(
                                gn,
                                tn,
                                spn,
                                "for-in iterator (a List local)",
                            ));
                        }
                    }
                    if let Some(d) = self.expect_text("{") {
                        return Some(d);
                    }
                    if let Some(d) = self.walk_block_body(false) {
                        return Some(d);
                    }
                    return self.sep_check();
                }
                if !wide {
                    return Some(
                        self.unsupported("`for` is not part of this source profile".into(), sp),
                    );
                }
                self.i += 1;
                if let Some(d) = self.walk_condition() {
                    return Some(d);
                }
                if let Some(d) = self.expect_text("{") {
                    return Some(d);
                }
                if let Some(d) = self.walk_block_body(false) {
                    return Some(d);
                }
                self.sep_check()
            }
            "is" if self.cx.profile.is_collections() => {
                // PLAN-928（T-03）：`is <local> { Arm [(v)]? -> { .. } }`。
                self.i += 1;
                match self.cur() {
                    Some((_, sg, _, _)) if sg == k.tok_ident => {
                        self.i += 1;
                    }
                    _ => {
                        let (spn, gn, tn, _) = match self.cur() {
                            Some(c) => c,
                            None => {
                                return Some(
                                    self.syntax("expected scrutinee, found end of file".into()),
                                )
                            }
                        };
                        return Some(self.name_error(gn, tn, spn, "scrutinee"));
                    }
                }
                if let Some(d) = self.expect_text("{") {
                    return Some(d);
                }
                loop {
                    while matches!(self.cur(), Some((_, _, ";", _))) {
                        self.i += 1;
                    }
                    match self.cur() {
                        None => return Some(self.syntax("expected `}` before end of file".into())),
                        Some((_, _, "}", _)) => {
                            self.i += 1;
                            break;
                        }
                        Some((_, ag, _, _)) if ag == k.tok_ident => {
                            // 限定形式 `Enum.Name`
                            self.i += 1;
                            if let Some((_, _, ".", _)) = self.cur() {
                                self.i += 1;
                                match self.cur() {
                                    Some((_, pg, _, _)) if pg == k.tok_ident => {
                                        self.i += 1;
                                    }
                                    _ => {
                                        let (spn, gn, tn, _) =
                                            match self.cur() {
                                                Some(c) => c,
                                                None => return Some(
                                                    self.syntax(
                                                        "expected arm pattern, found end of file"
                                                            .into(),
                                                    ),
                                                ),
                                            };
                                        return Some(self.name_error(gn, tn, spn, "arm pattern"));
                                    }
                                }
                            }
                            // 载荷绑定 `(v)`
                            if let Some((_, _, "(", _)) = self.cur() {
                                self.i += 1;
                                match self.cur() {
                                    Some((_, bg, _, _)) if bg == k.tok_ident => {
                                        self.i += 1;
                                    }
                                    _ => {
                                        let (spn, gn, tn, _) = match self.cur() {
                                            Some(c) => c,
                                            None => return Some(self.syntax("expected arm payload binding, found end of file".into())),
                                        };
                                        return Some(self.name_error(
                                            gn,
                                            tn,
                                            spn,
                                            "arm payload binding",
                                        ));
                                    }
                                }
                                match self.cur() {
                                    Some((_, _, ")", _)) => {
                                        self.i += 1;
                                    }
                                    _ => {
                                        return Some(self.syntax(
                                            "expected `)` after the arm payload binding".into(),
                                        ))
                                    }
                                }
                            }
                            match self.cur() {
                                Some((_, _, "->", _)) => {
                                    self.i += 1;
                                }
                                _ => {
                                    return Some(
                                        self.syntax("expected `->` after the arm pattern".into()),
                                    )
                                }
                            }
                            if let Some(d) = self.expect_text("{") {
                                return Some(d);
                            }
                            if let Some(d) = self.walk_block_body(false) {
                                return Some(d);
                            }
                        }
                        Some((_, t2, _, _)) => {
                            return Some(
                                self.syntax(format!("expected an arm pattern, found `{}`", t2)),
                            );
                        }
                    }
                }
                self.sep_check()
            }
            "loop" => {
                self.i += 1;
                if let Some(d) = self.expect_text("{") {
                    return Some(d);
                }
                if let Some(d) = self.walk_block_body(false) {
                    return Some(d);
                }
                self.sep_check()
            }
            "break" | "continue" => {
                self.i += 1;
                self.sep_check()
            }
            _ => {
                if self.is_reserved(g) {
                    return Some(self.unsupported(
                        format!("`{}` is not part of this source profile", text),
                        sp,
                    ));
                }
                if g == k.tok_ident {
                    // 赋值/路径/索引赋值/表达式语句（旧投机臂）。
                    if let Some((_, _, t2, _)) = self.at(self.i + 1) {
                        if t2 == "[" {
                            // PLAN-964（L-16/L-20/L-48）：索引赋值
                            // `base[key] = value`——对象动态键写与 List
                            // 整数索引写共用（resolve 按 base 类型分派）。
                            self.i += 2; // ident + `[`
                            if let Some(d) = self.walk_expr(true) {
                                return Some(d);
                            }
                            if let Some(d) = self.expect_text("]") {
                                return Some(d);
                            }
                            if let Some(d) = self.expect_text("=") {
                                return Some(d);
                            }
                            if let Some(d) = self.walk_expr(false) {
                                return Some(d);
                            }
                            return self.sep_check();
                        }
                        if t2 == "=" || t2 == "." {
                            // 路径：ident (. ident)* 然后 `=`
                            let mark = self.i;
                            self.i += 1;
                            let mut is_assign = true;
                            while matches!(self.cur(), Some((_, _, ".", _))) {
                                if !self.cx.profile.is_record() {
                                    let dsp = self.cur().unwrap().0;
                                    return Some(self.unsupported(
                                        "field access is not part of this source profile".into(),
                                        dsp,
                                    ));
                                }
                                self.i += 1;
                                match self.cur() {
                                    Some((_, fg, ftext, _)) if fg == k.tok_ident => {
                                        let _ = ftext;
                                        self.i += 1;
                                    }
                                    Some((fsp, fg, ftext, _)) => {
                                        return Some(self.name_error(
                                            fg,
                                            ftext,
                                            fsp,
                                            "field name after `.`",
                                        ));
                                    }
                                    None => {
                                        return Some(
                                            self.syntax(
                                                "expected field name after `.`, found end of file"
                                                    .into(),
                                            ),
                                        )
                                    }
                                }
                            }
                            if !matches!(self.cur(), Some((_, _, "=", _))) {
                                is_assign = false;
                            }
                            if !is_assign {
                                self.i = mark;
                                let start = self.cur().map(|c| c.0).unwrap_or(sp);
                                if let Some(d) = self.walk_expr(false) {
                                    return Some(d);
                                }
                                let end = self.at(self.i - 1).map(|c| c.0.end).unwrap_or(start.end);
                                if !self.cx.allows_expr_stmt() {
                                    return Some(self.unsupported(
                                        "expression statements are not part of this source profile; calls are only allowed in initializers, assignments, returns and conditions".into(),
                                        Span { start: start.start, end },
                                    ));
                                }
                                return self.sep_check();
                            }
                            self.i += 1; // `=`
                            if let Some(d) = self.walk_expr(false) {
                                return Some(d);
                            }
                            return self.sep_check();
                        }
                    }
                }
                if g == k.tok_ident || text == "true" || text == "false" || g == k.tok_int_lit {
                    let start = sp;
                    if let Some(d) = self.walk_expr(false) {
                        return Some(d);
                    }
                    let end = self.at(self.i - 1).map(|c| c.0.end).unwrap_or(start.end);
                    if !self.cx.allows_expr_stmt() {
                        return Some(self.unsupported(
                            "expression statements are not part of this source profile; calls are only allowed in initializers, assignments, returns and conditions".into(),
                            Span { start: start.start, end },
                        ));
                    }
                    return self.sep_check();
                }
                Some(self.syntax(format!("expected statement, found {}", self.describe_cur())))
            }
        }
    }

    fn name_error(&self, g: u32, text: &str, sp: Span, what: &str) -> Diagnostic {
        if self.is_raw(g) {
            return self.unsupported(
                format!(
                    "{} cannot be used as a name in this source profile",
                    raw_describe(g, text, self.k())
                ),
                sp,
            );
        }
        if self.is_kw(g, text) || self.is_reserved(g) {
            return self.unsupported(format!("keyword `{}` cannot be used as a name", text), sp);
        }
        self.syntax(format!(
            "expected {}, found {}",
            what,
            describe_g(g, text, self.k())
        ))
    }

    fn expect_text(&mut self, t: &str) -> Option<Diagnostic> {
        match self.cur() {
            Some((_, _, x, _)) if x == t => {
                self.i += 1;
                None
            }
            _ => Some(self.syntax(format!("expected `{}`, found {}", t, self.describe_cur()))),
        }
    }

    /// 二元算子表（按 profile）：Some(prec) = 本面算子；None = 非算子。
    fn prec_of(&self, text: &str) -> Option<u8> {
        if self.cx.profile.surface_is_scalar_or_wider() {
            match text {
                "||" => Some(1),
                "&&" => Some(2),
                "==" | "!=" => Some(3),
                "<" | ">" | "<=" | ">=" => Some(4),
                "+" | "-" => Some(5),
                "*" | "/" | "%" => Some(6),
                _ => None,
            }
        } else {
            match text {
                "<" => Some(1),
                "+" => Some(2),
                "*" => Some(3),
                _ => None,
            }
        }
    }
    fn op_of_other_surface(&self, text: &str) -> bool {
        if self.cx.profile.surface_is_scalar_or_wider() {
            return false;
        }
        matches!(
            text,
            "||" | "&&" | "==" | "!=" | ">" | "<=" | ">=" | "-" | "/" | "%"
        )
    }

    /// PLAN-928（T-07）：`( 参数? ) =>` 闭包头前瞻（token 级；镜像旧
    /// closure_header_ahead 的匹配 `)` 后随 `=>` 判定）。
    fn closure_header_ahead_tokens(&self, open: usize) -> bool {
        let mut depth = 0usize;
        let mut j = open;
        while let Some((_, _, t, _)) = self.at(j) {
            if t == "(" {
                depth += 1;
            } else if t == ")" {
                depth -= 1;
                if depth == 0 {
                    return matches!(self.at(j + 1), Some((_, _, "=>", _)));
                }
            }
            j += 1;
        }
        false
    }

    /// F-941-11（PLAN-941 r8）：lambda 头/参数/body 期望游走——镜像旧 parser
    /// `closure_expr`：`(`、参数名（旧 expect_ident("lambda parameter")）+
    /// 注解（旧 param_type→closures_type）序列、`,` 或 `)`、`=>`、`{` 块体
    /// （旧 body_machine）或表达式体（旧 expr）。入参 `i` 在 `(`（可选的
    /// `move` 前缀由调用方消费）；返回 Some = 分歧点首诊断。
    fn walk_closure_expr(&mut self) -> Option<Diagnostic> {
        let k = self.k();
        self.i += 1; // `(`
        if !matches!(self.cur(), Some((_, _, ")", _))) {
            loop {
                // 参数名。
                match self.cur() {
                    Some((_, g, _, _)) if g == k.tok_ident => self.i += 1,
                    Some((sp, g, text, _)) if self.is_kw(g, text) || self.is_reserved(g) => {
                        return Some(self.unsupported(
                            format!("keyword `{}` cannot be used as a name", text),
                            sp,
                        ));
                    }
                    Some((sp, g, text, _)) if self.is_raw(g) => {
                        return Some(self.unsupported(
                            format!(
                                "{} cannot be used as a name in this source profile",
                                raw_describe(g, text, k)
                            ),
                            sp,
                        ));
                    }
                    _ => {
                        return Some(self.syntax(format!(
                            "expected lambda parameter, found {}",
                            self.describe_cur()
                        )));
                    }
                }
                // 参数注解。
                if let Some(d) = self.walk_closure_param_type() {
                    return Some(d);
                }
                // `,` 或 `)`（旧 parser 不收尾逗号：`,` 后重入参数名期望）。
                match self.cur() {
                    Some((_, _, ",", _)) => self.i += 1,
                    Some((_, _, ")", _)) => break,
                    _ => {
                        return Some(self.syntax("expected `,` or `)` in lambda parameters".into()))
                    }
                }
            }
        }
        // `)` 然后 `=>`。
        self.i += 1;
        match self.cur() {
            Some((_, _, "=>", _)) => self.i += 1,
            _ => return Some(self.syntax(format!("expected `=>`, found {}", self.describe_cur()))),
        }
        // body：`{` 块体或表达式体。
        if matches!(self.cur(), Some((_, _, "{", _))) {
            self.i += 1;
            self.walk_block_body(false)
        } else {
            self.walk_expr(false)
        }
    }

    /// F-941-11（PLAN-941 r8）：lambda 参数注解期望游走——镜像旧 parser
    /// `closures_type`：`?T` 糖、`fn(...) [->] R` 函数型（复用
    /// skip_fn_type_leaves）、int/i32/bool、`void`/`Str`/`char`(closures)/
    /// 泛型 `Id<...>`/record 名义 ident（小写 `str` 与非 closures `char`
    /// 具名拒绝）；其余 → "expected a type"。
    fn walk_closure_param_type(&mut self) -> Option<Diagnostic> {
        let k = self.k();
        if matches!(self.cur(), Some((_, _, "?", _))) {
            self.i += 1;
            return self.walk_closure_param_type();
        }
        let Some((sp, g, text, _)) = self.cur() else {
            return Some(self.syntax("expected a type, found end of file".into()));
        };
        if text == "fn" {
            self.i = skip_fn_type_leaves(self.leaves, self.i);
            return None;
        }
        if text == "int" || text == "i32" || text == "bool" {
            self.i += 1;
            return None;
        }
        if g == k.tok_ident {
            if text == "str" || (text == "char" && !self.cx.profile.is_closures()) {
                return Some(self.unsupported(
                    format!("type `{}` is not part of this source profile", text),
                    sp,
                ));
            }
            self.i += 1;
            if matches!(self.cur(), Some((_, _, "<", _))) {
                let mut depth = 0usize;
                while let Some((_, _, t, _)) = self.cur() {
                    if t == "<" {
                        depth += 1;
                    } else if t == ">" {
                        depth -= 1;
                        if depth == 0 {
                            self.i += 1;
                            break;
                        }
                    }
                    self.i += 1;
                }
            }
            return None;
        }
        Some(self.syntax(format!("expected a type, found {}", self.describe_cur())))
    }

    /// PLAN-928（T-03）：`ident <` 位是否为泛型接收——平衡 `<...>` 后随
    /// `.` 才提交（镜像旧 parser 的 generic_scan_commits）。
    fn generic_scan_commits(&self, open_lt: usize) -> bool {
        let k = self.k();
        let mut depth = 0usize;
        let mut j = open_lt;
        while let Some((_, g, text, _)) = self.at(j) {
            if text == "<" {
                depth += 1;
            } else if text == ">" {
                depth -= 1;
                if depth == 0 {
                    return matches!(self.at(j + 1), Some((_, _, ".", _)));
                }
            } else if !(g == k.tok_ident
                || g == k.tok_int_kw
                || g == k.tok_i32
                || g == k.tok_bool
                || text == ","
                || text == "?")
            {
                return false;
            }
            j += 1;
        }
        false
    }

    /// 表达式游走：primary + 算子/关闭循环（括号帧跟踪，行纪律）。
    fn walk_expr(&mut self, _bracketed: bool) -> Option<Diagnostic> {
        self.walk_expr_context(false)
    }

    fn walk_condition(&mut self) -> Option<Diagnostic> {
        self.walk_expr_context(true)
    }

    fn walk_expr_context(&mut self, condition: bool) -> Option<Diagnostic> {
        let k = self.k();
        let mut frames: Vec<Br> = Vec::new();
        let mut anchored = false; // 算子后需同行 primary
        let wide = self.cx.profile.surface_is_scalar_or_wider();
        loop {
            // ---- primary ----
            let in_br = !frames.is_empty();
            if anchored && !in_br {
                if let Some((psp, _, _, line)) = self.cur() {
                    if line != self.prev_line() {
                        return Some(Diagnostic::new(
                            Stage::Text,
                            "source.syntax",
                            "unbracketed expression cannot continue across a newline",
                            psp,
                        ));
                    }
                }
            }
            let Some((sp, g, text, line)) = self.cur() else {
                return match frames.last() {
                    Some(Br::Call) => Some(self.syntax("expected argument expression".into())),
                    Some(Br::Paren) => Some(self.syntax("expected `)`".into())),
                    Some(Br::Ctor) => Some(self.syntax("unterminated expression".into())),
                    None => Some(self.syntax(format!(
                        "expected expression, found {}",
                        self.describe_cur()
                    ))),
                };
            };
            if g == k.tok_int_lit {
                self.i += 1;
            } else if text == "-" {
                match self.at(self.i + 1) {
                    Some((dsp, dg, _, dline)) if dg == k.tok_int_lit => {
                        if !in_br && dline != line {
                            return Some(Diagnostic::new(
                                Stage::Text,
                                "source.syntax",
                                "unbracketed expression cannot continue across a newline",
                                dsp,
                            ));
                        }
                        self.i += 2;
                    }
                    _ if wide => {
                        self.i += 1;
                        anchored = true;
                        continue;
                    }
                    Some((esp, eg, etext, _))
                        if eg == k.tok_ident
                            || etext == "true"
                            || etext == "false"
                            || etext == "(" =>
                    {
                        return Some(self.unsupported(
                            "unary minus is only supported directly before an integer literal in this source profile".into(),
                            sp.join(esp),
                        ));
                    }
                    _ => return Some(self.syntax("expected expression after `-`".into())),
                }
            } else if g == k.tok_str_lit || g == k.tok_char_lit {
                if self.cx.profile.admits_text_literals() {
                    self.i += 1;
                } else {
                    return Some(self.unsupported(
                        format!(
                            "{} `{}` is not part of this source profile",
                            raw_describe(g, text, k),
                            text
                        ),
                        sp,
                    ));
                }
            } else if text == "true" || text == "false" {
                self.i += 1;
            } else if text == "!" {
                if wide {
                    self.i += 1;
                    anchored = true;
                    continue;
                }
                return Some(
                    self.unsupported("unary `!` is not part of this source profile".into(), sp),
                );
            } else if text == "+" && wide {
                return Some(
                    self.unsupported("unary `+` is not part of this source profile".into(), sp),
                );
            } else if text == "type" || text == "static" || text == "mut" {
                return Some(self.unsupported(
                    format!("`{}` is not a value in this source profile", text),
                    sp,
                ));
            } else if g == k.tok_r_move
                && self.cx.profile.is_closures()
                && self.closure_header_ahead_tokens(self.i + 1)
            {
                // F-941-11（PLAN-941 r8）：`move ( 参数 ) =>`——镜像旧 parser 的
                // move 闭包分支（closure_expr(true)）：消费 `move` 后走 lambda
                // 头期望游走，不落入 reserved 关键字拒绝。
                self.i += 1;
                if let Some(d) = self.walk_closure_expr() {
                    return Some(d);
                }
            } else if (g == k.tok_r_await || g == k.tok_r_go)
                && (self.cx.profile.is_scheduler() || self.cx.profile.is_auto_language())
            {
                // PLAN-968: `await`/`go` prefix operators — consumed, the
                // operand walk continues (mirrors the oracle's prefix parse).
                self.i += 1;
                anchored = true;
                continue;
            } else if self.is_reserved(g) {
                return Some(self.unsupported(
                    format!("keyword `{}` is not part of this source profile", text),
                    sp,
                ));
            } else if self.is_raw(g) {
                let msg = if g == k.tok_affix_num || g == k.tok_block_open {
                    format!(
                        "{} `{}` is not part of this source profile",
                        raw_describe(g, text, k),
                        text
                    )
                } else {
                    format!(
                        "{} `{}` is not part of this source profile",
                        raw_describe(g, text, k),
                        text
                    )
                };
                return Some(self.unsupported(msg, sp));
            } else if text == "(" {
                if self.cx.profile.is_closures() && self.closure_header_ahead_tokens(self.i) {
                    // F-941-11（PLAN-941 r8）：`( 参数 ) =>` lambda 头——镜像旧
                    // parser 的 `(` 闭包优先分支（closure_header_ahead →
                    // closure_expr）：按参数/注解/`=>`/body 期望序列游走，不落入
                    // Br::Paren 把参数注解当括号表达式（整源会把后续错误误报到
                    // 前一个合法 lambda 的注解上）。lambda 完成后按值进入算子循环。
                    if let Some(d) = self.walk_closure_expr() {
                        return Some(d);
                    }
                } else {
                    self.i += 1;
                    frames.push(Br::Paren);
                    anchored = false;
                    continue;
                }
            } else if text == "." && self.cx.profile.is_record() {
                // leading-dot self
                self.i += 1;
                match self.cur() {
                    Some((_, mg, _, _)) if mg == k.tok_ident => {
                        self.i += 1;
                        if matches!(self.cur(), Some((_, _, "(", l2)) if in_br || l2 == self.prev_line())
                        {
                            self.i += 1;
                            if matches!(self.cur(), Some((_, _, ")", _))) {
                                self.i += 1;
                                // 旧怪癖保真：空实参 leading-dot 调用后旧 parser
                                // continue-'outer 重入 primary（不落回算子循环）——
                                // 下一 token 须能起始 primary；anchored 保持，未
                                // 括号待归约时锚定调用行（L04/L05/FIN864B 差分）。
                                continue;
                            } else {
                                frames.push(Br::Call);
                                anchored = false;
                                continue;
                            }
                        }
                    }
                    _ => {
                        return Some(self.syntax(format!(
                            "expected field or method name after `.`, found {}",
                            self.describe_cur()
                        )))
                    }
                }
            } else if g == k.tok_ident
                && self.cx.profile.is_closures()
                && text == "async"
                && self.closure_header_ahead_tokens(self.i + 1)
            {
                // PLAN-928（T-07）：`async (参数) =>` —— 闭包面前缀保留名
                // （镜像旧 parser 的 async 闭包拒绝；专用码）。
                return Some(Diagnostic::new(
                    crate::atom_text::Stage::Text,
                    "source.closure-unsupported",
                    "async closures are not supported by the closures source profile".to_string(),
                    sp,
                ));
            } else if g == k.tok_ident {
                // PLAN-928（T-03）：`List<T>` 接收位——平衡 `<...>` 后随 `.`
                // 才提交（镜像旧 generic_scan_commits；普通 `a < b` 比较不受
                // 影响——扫描失败即落回普通 ident + 二元 `<`）。
                if self.cx.profile.is_collections() {
                    if let Some((_, _, "<", _)) = self.at(self.i + 1) {
                        if self.generic_scan_commits(self.i + 1) {
                            self.i += 1;
                            let mut depth = 0usize;
                            loop {
                                match self.cur() {
                                    Some((_, _, "<", _)) => depth += 1,
                                    Some((_, _, ">", _)) => depth -= 1,
                                    Some(_) => {}
                                    None => break,
                                }
                                self.i += 1;
                                if depth == 0 {
                                    break;
                                }
                            }
                            anchored = false;
                            if let Some(d) = self.after_value(&mut frames, &mut anchored) {
                                return d;
                            }
                            continue;
                        }
                    }
                }
                // ctor（record 面，`{` 同行或括号内）
                if self.cx.profile.is_record()
                    && (!condition || in_br || matches!(self.at(self.i + 3), Some((_, _, ":", _))))
                {
                    if let Some((_, _, "{", l2)) = self.at(self.i + 1) {
                        if in_br || l2 == line {
                            self.i += 2;
                            if matches!(self.cur(), Some((_, _, "}", _))) {
                                self.i += 1;
                                // 旧怪癖保真：空 ctor `Ty { }` 同样走 continue-
                                // 'outer 重入 primary（edge#47 差分）。
                                continue;
                            } else {
                                frames.push(Br::Ctor);
                                // label?
                                self.take_label();
                                anchored = false;
                                continue;
                            }
                        }
                    }
                }
                // call（`(` 同行或括号内）
                if let Some((_, _, "(", l2)) = self.at(self.i + 1) {
                    if in_br || l2 == line {
                        self.i += 2;
                        if matches!(self.cur(), Some((_, _, ")", _))) {
                            self.i += 1;
                        } else {
                            frames.push(Br::Call);
                            self.take_label();
                            anchored = false;
                            continue;
                        }
                    } else {
                        self.i += 1;
                    }
                } else {
                    self.i += 1;
                }
            } else if text == "{"
                && (self.cx.profile.is_aggregates() || self.cx.profile.is_closures())
            {
                // PLAN-964（L-16/L-20）：裸对象字面量 `{ k: v, ... }`——
                // 键为 ident/字符串字面量；值为表达式；`,` 分隔；`}` 收尾。
                self.i += 1;
                loop {
                    match self.cur() {
                        None => return Some(self.syntax("unterminated object literal".into())),
                        Some((_, _, "}", _)) => {
                            self.i += 1;
                            break;
                        }
                        Some((ksp, kg, ktxt, _)) if kg == k.tok_ident || kg == k.tok_str_lit => {
                            let _ = ksp;
                            let _ = ktxt;
                            self.i += 1;
                            if let Some(d) = self.expect_text(":") {
                                return Some(d);
                            }
                            if let Some(d) = self.walk_expr(true) {
                                return Some(d);
                            }
                            match self.cur() {
                                Some((_, _, ",", _)) => {
                                    self.i += 1;
                                }
                                Some((_, _, "}", _)) => {
                                    self.i += 1;
                                    break;
                                }
                                _ => {
                                    return Some(
                                        self.syntax(
                                            "expected `,` or `}` in an object literal".into(),
                                        ),
                                    )
                                }
                            }
                        }
                        Some((_, _, other, _)) => {
                            return Some(
                                self.syntax(format!("expected object key, found `{other}`")),
                            )
                        }
                    }
                }
                // 对象字面量是完整值：落入算子/收尾循环（不得 continue 到
                // primary——下一个语句关键字会被误报"expected expression"）。
                anchored = false;
                if let Some(d) = self.after_value(&mut frames, &mut anchored) {
                    return d;
                }
                if frames.is_empty() && !anchored {
                    return None;
                }
                if anchored {
                    continue;
                }
            } else if self.is_kw(g, text)
                || text == ")"
                || text == "}"
                || text == ","
                || text == ";"
                || text == "{"
                || text == "="
                || text == ":"
                || text == "]"
                || text == "["
            {
                return Some(self.syntax(format!(
                    "expected expression, found {}",
                    if g == k.tok_ident {
                        "token".to_string()
                    } else {
                        format!("`{}`", text)
                    }
                )));
            } else {
                return Some(self.syntax(format!("expected expression, found `{}`", text)));
            }
            // ---- operator / close loop ----
            anchored = false;
            if let Some(d) = self.after_value(&mut frames, &mut anchored) {
                return d;
            }
            if frames.is_empty() && !anchored {
                return None;
            }
            if anchored {
                continue;
            }
            // 括号内等待下一 primary（刚 push 了帧）
        }
    }

    fn take_label(&mut self) {
        if let (Some((_, g, _, _)), Some((_, _, ":", _))) = (self.cur(), self.at(self.i + 1)) {
            if g == self.k().tok_ident {
                self.i += 2;
            }
        }
    }

    /// 值完成后：算子/点后缀/关闭括号/逗号。返回 Some(Some(diag)) = 错误；
    /// Some(None) = 表达式结束；None = 继续（已消费算子，anchored 置位）。
    fn after_value(
        &mut self,
        frames: &mut Vec<Br>,
        anchored: &mut bool,
    ) -> Option<Option<Diagnostic>> {
        let k = self.k();
        loop {
            let in_br = !frames.is_empty();
            // PLAN-964（L-16/L-20/L-48）：索引后缀 `base[key]`——对象动态键
            // 读与 List 整数索引共用此步（resolve 按 base 类型分派）。
            if let Some((dsp, _, "[", dline)) = self.cur() {
                if !in_br && dline != self.prev_line() {
                    return Some(Some(Diagnostic::new(
                        Stage::Text,
                        "source.syntax",
                        "unbracketed expression cannot continue across a newline",
                        dsp,
                    )));
                }
                self.i += 1;
                if let Some(d) = self.walk_expr(true) {
                    return Some(Some(d));
                }
                if let Some(d) = self.expect_text("]") {
                    return Some(Some(d));
                }
                continue;
            }
            // 点后缀
            if let Some((dsp, _, ".", dline)) = self.cur() {
                let is_record = self.cx.profile.is_record();
                let is_text = self.cx.profile.is_text();
                let admits_text = is_text || self.cx.profile.admits_text_literals();
                let next = self.at(self.i + 1);
                let next_is_to = matches!(next, Some((_, ng, _, _)) if ng == k.tok_r_to);
                if is_record && !(next_is_to && admits_text) {
                    if !in_br && dline != self.prev_line() {
                        return Some(Some(Diagnostic::new(
                            Stage::Text,
                            "source.syntax",
                            "unbracketed expression cannot continue across a newline",
                            dsp,
                        )));
                    }
                    self.i += 1;
                    match self.cur() {
                        Some((_, mg, _, _))
                            if mg == k.tok_r_move
                                && (is_text || self.cx.profile.is_collections()) =>
                        {
                            self.i += 1;
                            continue;
                        }
                        Some((_, _, "mut", _)) => {
                            self.i += 1;
                            continue;
                        }
                        Some((_, mg, _, mline)) if mg == k.tok_ident => {
                            self.i += 1;
                            if matches!(self.cur(), Some((_, _, "(", l2)) if in_br || l2 == mline) {
                                self.i += 1;
                                if matches!(self.cur(), Some((_, _, ")", _))) {
                                    self.i += 1;
                                    continue;
                                }
                                frames.push(Br::Call);
                                self.take_label();
                                *anchored = false;
                                return None;
                            }
                            continue;
                        }
                        _ => {
                            return Some(Some(self.syntax(format!(
                                "expected field or method name after `.`, found {}",
                                self.describe_cur()
                            ))))
                        }
                    }
                }
                if admits_text {
                    match next {
                        Some((msp, mg, _, mline)) if mg == k.tok_r_move => {
                            if !in_br && mline != self.prev_line() {
                                return Some(Some(Diagnostic::new(
                                    Stage::Text,
                                    "source.syntax",
                                    "unbracketed expression cannot continue across a newline",
                                    msp,
                                )));
                            }
                            self.i += 2;
                            continue;
                        }
                        Some((_, mg, _, _)) if mg == k.tok_r_to => {
                            self.i += 2;
                            if let Some(d) = self.expect_text("(") {
                                return Some(Some(d));
                            }
                            match self.cur() {
                                Some((_, wg, w, _))
                                    if wg == k.tok_ident && (w == "str" || w == "Str") =>
                                {
                                    self.i += 1;
                                }
                                _ => {
                                    return Some(Some(
                                        self.syntax("`.to` expects `str` or `Str`".into()),
                                    ))
                                }
                            }
                            if let Some(d) = self.expect_text(")") {
                                return Some(Some(d));
                            }
                            continue;
                        }
                        Some((_, mg, mname, mline)) if mg == k.tok_ident => {
                            let call = matches!(self.at(self.i + 2), Some((_, _, "(", l3)) if in_br || l3 == mline);
                            if !call {
                                return Some(Some(
                                    self.syntax(format!(
                                        "method `.{}` requires parentheses",
                                        mname
                                    )),
                                ));
                            }
                            self.i += 3;
                            if matches!(self.cur(), Some((_, _, ")", _))) {
                                self.i += 1;
                                continue;
                            }
                            frames.push(Br::Call);
                            self.take_label();
                            *anchored = false;
                            return None;
                        }
                        _ => {
                            return Some(Some(
                                self.syntax("expected a method name after `.`".into()),
                            ))
                        }
                    }
                }
                // 非 record/text：`.` 不是算子 → unsupported_op
                return Some(Some(self.unsupported(
                    "operator `.` is not part of this source profile".into(),
                    dsp,
                )));
            }
            let Some((sp, _g, text, line)) = self.cur() else {
                return Some(match frames.last() {
                    Some(Br::Call) => Some(self.syntax("expected `,` or `)`".into())),
                    Some(Br::Paren) => Some(self.syntax("expected `)`".into())),
                    Some(Br::Ctor) => Some(self.syntax("unterminated expression".into())),
                    None => None,
                });
            };
            if let Some(_) = self.prec_of(text) {
                if !in_br && line != self.prev_line() {
                    // 算子换行：表达式结束
                    return Some(match frames.last() {
                        Some(Br::Call) => Some(self.syntax("expected `,` or `)`".into())),
                        Some(Br::Paren) => Some(self.syntax("expected `)`".into())),
                        _ => None,
                    });
                }
                self.i += 1;
                *anchored = true;
                return None;
            }
            if self.op_of_other_surface(text) {
                return Some(Some(self.unsupported(
                    format!("operator `{}` is not part of this source profile", text),
                    sp,
                )));
            }
            match text {
                "(" => {
                    let mut diag =
                        self.syntax(format!("expected operator, found {}", self.describe_cur()));
                    if let Some((_, _, text, _)) = self.cur() {
                        if let Some(fact) = surface_shape_of_token(text) {
                            diag = diag.tagged(fact);
                        }
                    }
                    return Some(Some(diag));
                }
                ")" => match frames.last() {
                    Some(Br::Paren) | Some(Br::Call) => {
                        frames.pop();
                        self.i += 1;
                        continue;
                    }
                    _ => return Some(None),
                },
                "," => match frames.last() {
                    Some(Br::Call) => {
                        self.i += 1;
                        self.take_label();
                        *anchored = false;
                        return None;
                    }
                    Some(Br::Ctor) => {
                        self.i += 1;
                        self.take_label();
                        *anchored = false;
                        return None;
                    }
                    // PLAN-969 (AUDIT-03): the tuple-literal comma shape is
                    // aggregates-owned — structured, not message-routed.
                    _ => {
                        return Some(Some(
                            self.syntax("expected operator, found `,`".into())
                                .tagged(SurfaceFact::OwnedShape),
                        ))
                    }
                },
                "}" => match frames.last() {
                    Some(Br::Ctor) => {
                        frames.pop();
                        self.i += 1;
                        continue;
                    }
                    Some(Br::Call) => return Some(Some(self.syntax("expected `,` or `)`".into()))),
                    Some(Br::Paren) => return Some(Some(self.syntax("expected `)`".into()))),
                    None => return Some(None),
                },
                _ => {
                    if matches!(frames.last(), Some(Br::Ctor)) && line > self.prev_line() {
                        self.take_label();
                        *anchored = false;
                        return None;
                    }
                    return Some(match frames.last() {
                        Some(Br::Call) => Some(self.syntax("expected `,` or `)`".into())),
                        Some(Br::Paren) => Some(self.syntax("expected `)`".into())),
                        Some(Br::Ctor) => Some(self.syntax("expected `,` or `)`".into())),
                        None => None,
                    });
                }
            }
        }
    }
}

/// 类型词游走（int/i32/bool/门控 ident；失败给 required 消息或类型拒绝）。
fn walk_type_word(
    cx: &Cx,
    leaves: &[(u32, u32, String)],
    i: &mut usize,
    required_msg: &str,
) -> Option<Diagnostic> {
    let k = cx.k;
    if *i >= leaves.len() {
        return None;
    }
    let (sp, g, text) = (
        leaf_span(cx, leaves[*i].0),
        leaves[*i].1,
        leaves[*i].2.as_str(),
    );
    let is_builtin = text == "int" || text == "i32" || text == "bool";
    let ident_ok = g == k.tok_ident
        && (cx.profile.is_text() || cx.profile.is_record() || cx.profile.is_entry_unit());
    if is_builtin || ident_ok {
        // text 族拼写分类 + 非 text 的 str/Str/char 拒绝由投影/叶层处理；
        // 这里只推进。
        *i += 1;
        return None;
    }
    // PLAN-982（aggregates 文本入口裁定，2026-11-19）：aggregates 面**函数
    // 参数位**的借用 `str` 放行推进（本函数唯一调用点＝参数位）；拼写分类
    // 与入口专属纪律由投影层/车道门持有。
    if g == k.tok_ident && cx.profile.is_aggregates() && text == "str" {
        *i += 1;
        return None;
    }
    if g == k.tok_ident && !cx.profile.is_text() && matches!(text, "str" | "Str" | "char") {
        return Some(Diagnostic::new(
            Stage::Text,
            "source.unsupported",
            format!("type `{}` is not part of this source profile", text),
            sp,
        ));
    }
    if g == k.tok_ident || g == k.tok_r_move || g == k.tok_r_to {
        return Some(Diagnostic::new(
            Stage::Text,
            "source.unsupported",
            format!("type `{}` is not part of this source profile", text),
            sp,
        ));
    }
    let _ = required_msg;
    None
}

fn is_kw_leaf(g: u32, k: &super::kinds::SourceKinds) -> bool {
    [
        k.tok_let,
        k.tok_var,
        k.tok_if,
        k.tok_else,
        k.tok_while,
        k.tok_loop,
        k.tok_break,
        k.tok_continue,
        k.tok_return,
        k.tok_true,
        k.tok_false,
        k.tok_for,
        k.tok_int_kw,
        k.tok_i32,
        k.tok_bool,
        k.tok_type_kw,
        k.tok_static,
        k.tok_mut,
        k.tok_enum_kw,
        k.tok_is,
        k.tok_in,
    ]
    .contains(&g)
}

/// PLAN-961: extract a numeric `.as(T)`/`.to(T)` step's type word —
/// the num-ty-word child's keyword leaf mapped to its TypeWord.
#[allow(clippy::type_complexity)]
fn numeric_step_word(cx: &Cx, step: u32) -> Option<(Span, TypeWord)> {
    let k = &cx.k;
    // The `.as(T)`/`.to(T)` step carries the num-ty-word NODE (field
    // "word"); the keyword LEAF is its direct child.
    let word = match cx.field_child(step, "word") {
        Some(nty) => cx.children(nty).into_iter().next()?,
        None => {
            // Fallback: a direct keyword leaf (shape tolerance).
            cx.children(step).into_iter().find(|&c| {
                let g = cx.tree.kind(c);
                k.tok_i8_kw == Some(g)
                    || k.tok_u8_kw == Some(g)
                    || k.tok_byte_kw == Some(g)
                    || k.tok_uint_kw == Some(g)
                    || k.tok_u32_kw == Some(g)
                    || k.tok_i64_kw == Some(g)
                    || k.tok_u64_kw == Some(g)
                    || k.tok_float_kw == Some(g)
                    || k.tok_f32_kw == Some(g)
                    || k.tok_double_kw == Some(g)
                    || k.tok_f64_kw == Some(g)
                    || g == k.tok_int_kw
                    || g == k.tok_i32
                    || g == k.tok_bool
            })?
        }
    };
    let span = cx.span_of(word);
    let g = cx.tree.kind(word);
    let tw = if k.tok_i8_kw == Some(g) {
        TypeWord::I8
    } else if k.tok_u8_kw == Some(g) {
        TypeWord::U8
    } else if k.tok_byte_kw == Some(g) {
        TypeWord::Byte
    } else if k.tok_uint_kw == Some(g) {
        TypeWord::Uint
    } else if k.tok_u32_kw == Some(g) {
        TypeWord::U32
    } else if k.tok_i64_kw == Some(g) {
        TypeWord::I64
    } else if k.tok_u64_kw == Some(g) {
        TypeWord::U64
    } else if k.tok_float_kw == Some(g) || k.tok_f32_kw == Some(g) {
        TypeWord::Float
    } else if k.tok_double_kw == Some(g) {
        TypeWord::Double
    } else if k.tok_f64_kw == Some(g) {
        TypeWord::F64
    } else if g == k.tok_int_kw {
        TypeWord::Int
    } else if g == k.tok_i32 {
        TypeWord::I32
    } else if g == k.tok_bool {
        TypeWord::Bool
    } else {
        return None;
    };
    Some((span, tw))
}

/// PLAN-961: decode a typed numeric literal word per the frozen T-01 §1
/// table — prefixes `0x`/`0b`, `_` separators, integer suffixes
/// (none/`i`/`i32`→i32, `i8`, `u`/`u32`, `u8`, `u64`, `i64`, `b`→u8) and
/// float text (`d.d` with `f`/`f32`/`d`/`f64`, dot-without-suffix→f32),
/// parsed with the TARGET width's correctly-rounded std parser; a
/// magnitude that overflows the target is a `source.literal-range`
/// rejection (never an inf literal).
/// PLAN-961: minus-aware typed-literal decode (the int-literal node's
/// minus arm): a negative literal folds into the signed payload (an
/// in-range magnitude never overflows on fold; unsigned negatives are
/// range rejections; i64 accepts exactly -2^63).
fn decode_typed_literal_neg(raw: &str, neg: bool) -> Result<ExprKind, String> {
    if !neg {
        return decode_typed_literal(raw);
    }
    // The i64 MIN magnitude (2^63) is representable ONLY negated — accept
    // it here before the positive-range table rejects it.
    let body = raw
        .strip_prefix("0x")
        .map(|b| (16u32, b))
        .or_else(|| raw.strip_prefix("0b").map(|b| (2u32, b)))
        .unwrap_or((10u32, raw));
    let digits_only: String = body
        .1
        .chars()
        .take_while(|c| {
            c.is_ascii_digit()
                || (body.0 == 16 && c.to_ascii_lowercase() >= 'a' && c.to_ascii_lowercase() <= 'f')
                || *c == '_'
        })
        .filter(|c| *c != '_')
        .collect();
    let mag: u128 = u128::from_str_radix(&digits_only, body.0).unwrap_or(u128::MAX);
    // The MIN magnitude (2^(bits-1)) is representable ONLY negated — for
    // every signed width (i8/i32/i64; the fold then yields exactly MIN).
    for (suffix, word, bits) in [
        ("i8", crate::source::ast::TypeWord::I8, 8u32),
        ("i32", crate::source::ast::TypeWord::Int, 32),
        ("i64", crate::source::ast::TypeWord::I64, 64),
    ] {
        if body.1.ends_with(suffix) && mag == 1u128 << (bits - 1) {
            return Ok(ExprKind::TypedLit {
                ty: word,
                bits: 1u64 << (bits - 1),
            });
        }
    }
    // Decode the positive magnitude with the frozen table, then fold.
    let kind = decode_typed_literal(raw)?;
    let ExprKind::TypedLit { ty, bits } = kind else {
        return Ok(kind);
    };
    use crate::source::ast::TypeWord as TW;
    let folded = match ty {
        TW::I8 => (bits as u8).wrapping_neg() as u64,
        TW::Int | TW::I32 => (bits as u32).wrapping_neg() as u64,
        TW::I64 => (bits as u64).wrapping_neg(),
        TW::Float => ((bits as u32) ^ 0x8000_0000) as u64,
        TW::Double | TW::F64 => bits ^ 0x8000_0000_0000_0000,
        TW::U8 | TW::Byte | TW::Uint | TW::U32 | TW::U64 => {
            return Err(format!(
                "negative literal `-{raw}` is not representable in an unsigned type"
            ))
        }
        TW::Bool | TW::Str | TW::StrOwned | TW::Char | TW::Void => return Ok(kind),
    };
    Ok(ExprKind::TypedLit { ty, bits: folded })
}

fn decode_typed_literal(raw: &str) -> Result<ExprKind, String> {
    use crate::source::ast::TypeWord as TW;
    let (radix, rest) = if let Some(b) = raw.strip_prefix("0x") {
        (16u32, b)
    } else if let Some(b) = raw.strip_prefix("0b") {
        (2u32, b)
    } else {
        (10u32, raw)
    };
    // Split the alphabetic suffix from the numeric body (the token shape
    // guarantees digits lead and the suffix is contiguous letters).
    // 没有字母后缀（`0.9`/`12`/`1_000`）是**合法**的无后缀字面量：body 即整个
    // 数字体、suffix 为空。只有连一个数字都没有时才报错。
    //
    // PLAN-1079: the body is the run of *radix* digits plus `_` separators
    // (and one fraction dot); the suffix starts at the first character
    // outside that set. Hex digits `a`..`f` are body characters, so
    // `0xFFu8` splits as body `FF` + suffix `u8`; a plain alphabetic split
    // would take `F` as the suffix and reject the literal as "carries no
    // digits" (the pre-1079 behaviour for every radix spelling).
    //
    // The dot stays a BODY character in every radix (review F-02): a radix
    // literal with a fraction must reach the explicit "radix literal cannot
    // carry a fraction" rejection below, not be reported as an unknown
    // suffix `.5`. Keeping the dot in the body is what makes that branch
    // reachable.
    let digit = |ch: char| match radix {
        16 => ch.is_ascii_hexdigit(),
        2 => ch == '0' || ch == '1',
        _ => ch.is_ascii_digit(),
    };
    let split = rest.find(|ch: char| !(digit(ch) || ch == '_' || ch == '.'));
    let (body, suffix) = match split {
        Some(i) => (&rest[..i], &rest[i..]),
        None => (rest, ""),
    };
    if body.is_empty() || !body.chars().any(digit) {
        return Err(format!("numeric literal `{raw}` carries no digits"));
    }
    let has_dot = body.contains('.');
    if has_dot && radix != 10 {
        return Err(format!("radix literal `{raw}` cannot carry a fraction"));
    }
    let clean: String = body.chars().filter(|c| *c != '_').collect();
    if has_dot {
        // Float text: parse with the target width's own std parser
        // (correctly rounded); overflow-to-inf is a range rejection.
        let word = match suffix {
            "f" | "f32" => TW::Float,
            "d" | "f64" => TW::Double,
            "" => TW::Float,
            other => return Err(format!("suffix `{other}` is not a float literal suffix")),
        };
        let bits = if matches!(word, TW::Double) {
            let v: f64 = clean
                .parse()
                .map_err(|_| format!("float literal `{raw}` is not a number"))?;
            if v.is_infinite() {
                return Err(format!("float literal `{raw}` overflows the f64 range"));
            }
            v.to_bits()
        } else {
            let v: f32 = clean
                .parse()
                .map_err(|_| format!("float literal `{raw}` is not a number"))?;
            if v.is_infinite() {
                return Err(format!("float literal `{raw}` overflows the f32 range"));
            }
            v.to_bits() as u64
        };
        return Ok(ExprKind::TypedLit { ty: word, bits });
    }
    // Integer text: magnitude in the radix, then per-type range checks.
    let mag = u128::from_str_radix(&clean, radix)
        .map_err(|_| format!("integer literal `{raw}` is out of range"))?;
    let (word, bits) = match suffix {
        "" | "i" | "i32" => {
            if mag > i32::MAX as u128 {
                return Err(format!(
                    "integer literal `{raw}` is not representable as int (i32)"
                ));
            }
            (TW::Int, mag as u64)
        }
        "i8" => {
            if mag > i8::MAX as u128 {
                return Err(format!(
                    "integer literal `{raw}` is not representable as i8"
                ));
            }
            (TW::I8, mag as u8 as u64)
        }
        "u8" | "b" => {
            if mag > u8::MAX as u128 {
                return Err(format!(
                    "integer literal `{raw}` is not representable as u8"
                ));
            }
            (TW::U8, mag as u8 as u64)
        }
        "u" | "u32" => {
            if mag > u32::MAX as u128 {
                return Err(format!(
                    "integer literal `{raw}` is not representable as u32"
                ));
            }
            (TW::U32, mag as u32 as u64)
        }
        "i64" => {
            if mag > i64::MAX as u128 {
                return Err(format!(
                    "integer literal `{raw}` is not representable as i64"
                ));
            }
            (TW::I64, mag as u64)
        }
        "u64" => {
            if mag > u64::MAX as u128 {
                return Err(format!(
                    "integer literal `{raw}` is not representable as u64"
                ));
            }
            (TW::U64, mag as u64)
        }
        other => return Err(format!("suffix `{other}` is not an integer literal suffix")),
    };
    Ok(ExprKind::TypedLit { ty: word, bits })
}

fn raw_describe(g: u32, _text: &str, k: &super::kinds::SourceKinds) -> &'static str {
    if g == k.tok_affix_num {
        "suffixed/radix/float numeric literal"
    } else if g == k.tok_str_lit {
        "string literal"
    } else if g == k.tok_char_lit {
        "char literal"
    } else {
        "block comment"
    }
}

fn describe_g(g: u32, text: &str, k: &super::kinds::SourceKinds) -> String {
    if g == k.tok_int_lit {
        "integer literal".to_string()
    } else if g == k.tok_str_lit {
        "string literal".to_string()
    } else if g == k.tok_char_lit {
        "char literal".to_string()
    } else if g == k.tok_affix_num {
        "suffixed/radix/float numeric literal".to_string()
    } else if g == k.tok_block_open {
        "block comment".to_string()
    } else {
        format!("`{}`", text)
    }
}

fn describe_leaf_text(cx: &Cx, tree: &CstTree, leaf: Option<u32>) -> String {
    let Some(l) = leaf else {
        return "end of file".to_string();
    };
    let g = tree.kind(l);
    let text = cx.spelling(l);
    if g == cx.k.tok_ident {
        format!("`{}`", text)
    } else if g == cx.k.tok_int_lit {
        "integer literal".to_string()
    } else if g == cx.k.tok_str_lit {
        "string literal".to_string()
    } else if g == cx.k.tok_char_lit {
        "char literal".to_string()
    } else if g == cx.k.tok_affix_num {
        "suffixed/radix/float numeric literal".to_string()
    } else if g == cx.k.tok_block_open {
        "block comment".to_string()
    } else {
        format!("`{}`", text)
    }
}

fn first_meaningful_leaf(_cx: &Cx, tree: &CstTree, node: u32) -> Option<u32> {
    let mut stack = vec![node];
    while let Some(n) = stack.pop() {
        let nd = tree.node(n);
        if nd.flags & FLAG_TRIVIA != 0 {
            continue;
        }
        if nd.child_count == 0 {
            let (s, e) = tree.abs_range(n);
            if e > s {
                return Some(n);
            }
            continue;
        }
        let kids = tree.children(n);
        for c in kids.into_iter().rev() {
            stack.push(c);
        }
    }
    None
}

/// 节点 id 路径（path_to 收集 kind；结构层重根需要真实祖先节点）。
fn node_path_to(tree: &CstTree, from: u32, target: u32, path: &mut Vec<u32>) -> bool {
    if from == target {
        return true;
    }
    path.push(from);
    for c in tree.children(from) {
        if node_path_to(tree, c, target, path) {
            return true;
        }
    }
    path.pop();
    false
}

fn path_to(tree: &CstTree, from: u32, target: u32, path: &mut Vec<u32>) -> bool {
    if from == target {
        return true;
    }
    path.push(tree.kind(from));
    for c in tree.children(from) {
        if path_to(tree, c, target, path) {
            return true;
        }
    }
    path.pop();
    false
}

// ---------------------------------------------------------------- engine

enum Val {
    E(Expr),
    A(Arg),
    CF(NamedCtorField),
    S(Stmt),
    B(Block),
    F(FnItem),
}

enum Frame {
    // -- 块/语句 --
    Block(BlockFrame),
    /// PLAN-936（T-04）：`#{ expr }` 原子——inner 表达式入场后包裹。
    HashExpr {
        span: Span,
    },
    /// PLAN-966（T-02）：语句级 `#if`——then 块收尾（有 else 时续营 else）。
    CtIfThen {
        cond: CtExpr,
        els_n: Option<u32>,
        span: Span,
    },
    CtIfElse {
        cond: CtExpr,
        then: Block,
        span: Span,
    },
    /// PLAN-966（T-02）：语句级 `#for`——body 块收尾。
    CtForBody {
        var: Ident,
        start: CtExpr,
        end: CtExpr,
        span: Span,
    },
    /// PLAN-966（T-02）：语句级 `#is`——逐臂块 + 汇总。
    CtIsArmBlock {
        ctx: Box<CtIsCtx>,
        pattern: Option<CtExpr>,
    },
    /// PLAN-966（T-02）：宏调用 `#name(args)`——实参收尾后成调用语句。
    MacroCallFinish {
        name: Ident,
        argc: usize,
        span: Span,
    },
    LetFinish {
        node: u32,
        kw: Span,
        name: Ident,
        ann: Option<TypeAnn>,
        mutable: bool,
    },
    /// PLAN-1094：`with <expr> [as <name>] { body }` —— 管理器语句。
    /// 表达式与 body 各自入场，finish 时组装 Stmt::With。
    WithFinish {
        kw_span: Span,
        bind: Option<Ident>,
    },
    /// PLAN-960（T-02）：module-level `const`/`var` 项——初始化表达式入场
    /// 后包装为 Item::Const/Store。
    GlobalItemFinish {
        node: u32,
        name: Ident,
        ann: TypeAnn,
        mutable: bool,
    },
    AssignFinish {
        name: Ident,
        path: Vec<Ident>,
    },
    ReturnFinish {
        kw: Span,
    },
    IfCond {
        then_n: u32,
        els_blk: Option<u32>,
        node: u32,
    },
    IfThen {
        cond: Expr,
        els_blk: Option<u32>,
        node: u32,
    },
    ElseIfBlock {
        span: Span,
    },
    IfElse {
        cond: Expr,
        then: Block,
        node: u32,
    },
    ScopeFinish,
    IfValueCond {
        span: Span,
        then_node: u32,
        else_node: u32,
    },
    IfValueThen {
        span: Span,
        cond: Expr,
        else_node: u32,
    },
    IfValueElse {
        span: Span,
        cond: Expr,
        then_block: Block,
    },
    WhileCond {
        body_n: u32,
        node: u32,
    },
    WhileBody {
        cond: Expr,
        node: u32,
    },
    LoopBody {
        node: u32,
    },
    ExprStmtFinish,
    FnBodyFinish {
        node: u32,
        name: Ident,
        type_params: Vec<TypeParamDecl>,
        params: Vec<Param>,
        ret: TypeAnn,
        receiver: Receiver,
        owner: Option<Ident>,
    },
    /// PLAN-1081 (T-03)：语句位嵌套 fn 项收尾——弹 Val::F 包装为
    /// Stmt::NestedFn，并恢复外层类型变量域。
    NestedFnFinish {
        saved_ty_params: Vec<String>,
    },
    ScriptFinish {
        start_span: Span,
    },
    /// PLAN-928（T-03）：`for x in <List 局部> { .. }` 体收尾帧。
    ForInBody {
        var: Ident,
        iter: Expr,
        node: u32,
    },
    /// PLAN-965（M2-A）：`for x in <expr> { .. }` 体收尾帧（vals 栈：body、iter）。
    ForInExprBody {
        var: Ident,
        node: u32,
    },
    /// PLAN-962（L-19）：`for i in a..b { }` 收尾帧（vals 栈自底向上：
    /// body、start、end）。
    ForRangeBody {
        var: Ident,
        inclusive: bool,
        node: u32,
    },
    /// PLAN-962（L-48）：`xs[i] = v` 收尾帧（vals 栈自底向上：index、value）。
    IndexAssignFinish {
        name: Ident,
        span: Span,
    },
    /// PLAN-1082：place 链 `base (.f | [e])* (op)= v` 收尾帧（vals 栈自底
    /// 向上：idx1..idxn（源序）、value）。步骤里的占位 Expr 按弹出序回填。
    PlaceAssignFinish {
        name: Ident,
        steps: Vec<PlaceStep>,
        op: PlaceOp,
        span: Span,
    },
    /// PLAN-962（L-15）：`let (a, b, ...) = e` 收尾帧。
    LetDestructFinish {
        names: Vec<Ident>,
        span: Span,
    },
    /// PLAN-962（T-08，L-07）：hold 收尾帧。
    HoldFinish {
        name: Ident,
        bind: Ident,
        span: Span,
    },
    /// PLAN-928（T-03）：`is <局部> { 臂 }` 逐臂帧 + 汇总帧。
    IsArms(IsMatchCtx),
    IsArmBlock {
        ctx: Box<IsMatchCtx>,
        qual: Option<Ident>,
        pattern: Ident,
        binding: Option<Ident>,
        kw: Span,
    },
    /// PLAN-965 (M2-A): aggregates `is` (stmt & expr), `try/catch/finally`, `yield`.
    AggIsArms(AggIsMatchCtx),
    AggIsArmRhs {
        ctx: Box<AggIsMatchCtx>,
        /// PLAN-1083: one entry per or-arm pattern (`A | B -> body` shares
        /// the rhs); the single-arm case is a one-element vec.
        pats: Vec<MatchPattern>,
        arm_start: Span,
        is_block: bool,
        /// PLAN-1083: `pat if cond -> body` — the guard expression (pops
        /// before the rhs value when present).
        has_guard: bool,
        statement_span: Option<Span>,
    },
    TryFinish {
        has_catch: bool,
        catch_bind: Option<Ident>,
        catch_span: Span,
        has_finally: bool,
        kw_span: Span,
    },
    YieldFinish {
        kw_span: Span,
    },
    // -- 表达式 --
    ExprEnter(u32),
    ArrayLiteralFinish {
        span: Span,
        count: usize,
    },
    ArrayConditionalElemFinish { span: Span },
    ObjectLiteralFinish {
        span: Span,
        fields: Vec<(crate::hir::ObjectKey, Span)>,
    },
    /// PLAN-928：lambda 体收尾（块体/表达式体两形；参数与 move 词已冻结）。
    LambdaBlockBody {
        move_capture: bool,
        params: Vec<Param>,
        result: Option<TypeAnn>,
        start: Span,
    },
    LambdaExprBody {
        move_capture: bool,
        params: Vec<Param>,
        start: Span,
    },
    OpPost {
        op: BinOp,
    },
    UnaryPost {
        op: UnaryOp,
    },
    /// PLAN-968: `await`/`go` prefix wrap-up (scheduler face).
    AsyncBlockFinish {
        span: Span,
    },
    SchedPrefixFinish {
        is_await: bool,
        kw_span: Span,
    },
    /// PLAN-968: `reply(v)` statement wrap-up (unit call for effect).
    ReplyStmtFinish {
        kw: Span,
    },
    /// PLAN-968: `go <call>` statement wrap-up.
    GoStmtFinish {
        kw: Span,
    },
    PostfixFoldAll {
        steps: Vec<u32>,
    },
    /// 调用收尾：callee 语义（纯名字 / 方法链尾步）+ 一元链 + 实参数 +
    /// 调用节点终点。接收者值（方法路径）在实参之下。
    CallAssemble {
        callee: CalleePlan,
        argc: usize,
        close_end: usize,
    },
    ArgBuild {
        named: Option<Ident>,
    },
    CtorFieldBuild {
        label: Ident,
    },
    CtorFinish {
        node: u32,
        ty: Ident,
        /// PLAN-963（T-02）：显式类型实参（空 = 旧具名构造写法）。
        args: Vec<TypeAnn>,
        count: usize,
    },
}

/// 调用 callee 的静态分析结果（enter_call 时求出；组装时消费）。
enum CalleePlan {
    /// 纯名字调用（ident 叶）+ 外→内一元链。
    Plain {
        ident: Ident,
        unary_ops: Vec<UnaryOp>,
    },
    /// PLAN-928（T-07）：表达式被调（括号组/λ 等值位 callee）→ 间接调用
    /// `CallValue`（镜像旧 CallFrame callee_expr；callee 名 "<closure>"）。
    Value {
        callee_n: u32,
        unary_ops: Vec<UnaryOp>,
    },
    /// 方法链：base（接收者求值任务节点）+ 中间步 + 链尾成员叶。
    Method {
        base: u32,
        member: u32,
        mids: Vec<u32>,
        unary_ops: Vec<UnaryOp>,
    },
}

/// PLAN-928（T-03）：is 分派的逐臂装配上下文。
struct IsMatchCtx {
    scrutinee: Ident,
    arms: Vec<MatchArm>,
    remaining: Vec<u32>,
    node: u32,
}

struct AggIsMatchCtx {
    is_expr: bool,
    scrutinee: Option<Expr>,
    arms: Vec<AggMatchArm>,
    remaining: Vec<u32>,
    node: u32,
}

/// 块帧：语句节点序列 + 组装 span。
struct BlockFrame {
    nodes: Vec<u32>,
    cursor: usize,
    stmts: Vec<Stmt>,
    script_top: bool,
    returns_tail: bool,
    span: Span,
}

/// PLAN-966（T-02）：`#is` 语句的臂收集上下文（源序逐臂投影；`else` 臂的
/// pattern 为 None）。
struct CtIsCtx {
    scrutinee: CtExpr,
    pending: Vec<(Option<CtExpr>, u32)>,
    arms: Vec<HashIsArm>,
    span: Span,
}

pub(crate) struct Cx<'a> {
    p: &'a GrammarProgram,
    k: &'a SourceKinds,
    tree: &'a CstTree,
    snap: &'a Snapshot,
    profile: SourceProfile,
    /// PLAN-936（T-04）：ct 能力面激活（scalar-ct 裁剪）——字符串字面量仅作为
    /// meta 字段选择子进入 AST；值位置由 resolve 受控拒。
    ct_face: bool,
    /// PLAN-963（T-04）：实例能力面激活——`<const N>` 值参数与
    /// `generic-call`（`rep<2>(21)`）只在这一面成立；plain 面沿用 935 的
    /// 具名拒（`source.instances-unsupported`）。
    instances_face: bool,
    /// PLAN-981（IF-981-ORIGINAL）：AGE 作者面（`age-package` 裁剪）激活——
    /// 浮点字面量等作者构造合法。由包前门显式传入（不按 profile 猜：包门在面
    /// 之上用 TextRecord 投影）。
    age_face: bool,
    /// PLAN-935：当前投影中函数声明的类型参数名（fn 投影期间有效——
    /// 引擎栈在 run_fn_item 内驱完，无跨函数交错）。
    ty_params: Vec<String>,
    visited: usize,
    diags: Vec<(u64, Diagnostic)>,
    annotations: Vec<SourceAnnotation>,
    over_budget: bool,
}

/// PLAN-968 helper: the closing `)` leaf span of one fn-tail node (the
/// missing-ret N42 diagnostic spans `name .. )`, matching the oracle).
fn fn_tail_rparen_span(cx: &Cx, tail: u32) -> Span {
    let mut best: Option<Span> = None;
    for c in cx.children(tail) {
        if cx.tree.kind(c) == cx.k.tok_rparen {
            best = Some(cx.span_of(c));
        }
    }
    best.unwrap_or(Span { start: 0, end: 0 })
}

impl<'a> Cx<'a> {
    fn allows_expr_stmt(&self) -> bool {
        self.age_face || self.profile.is_entry_unit() || self.profile.is_collections()
    }

    fn reject(&mut self, span: Span, make: impl FnOnce() -> Diagnostic) {
        let d = make();
        self.diags.push((span.start as u64, d));
    }
    fn first_diag(&self) -> Option<Diagnostic> {
        self.diags
            .iter()
            .min_by_key(|(pos, _)| *pos)
            .map(|(_, d)| d.clone())
    }
    fn span_of(&self, n: u32) -> Span {
        let (s, e) = self.tree.abs_range(n);
        Span {
            start: s as usize,
            end: e as usize,
        }
    }
    fn spelling(&self, n: u32) -> String {
        let (s, e) = self.tree.abs_range(n);
        self.snap.read_str(s, e).unwrap_or_default()
    }
    fn spelling_by_range(&self, s: u64, e: u64) -> String {
        self.snap.read_str(s, e).unwrap_or_default()
    }
    /// `pos` 之后的首个非 trivia 叶（错误叶也算——保真旧 describe_cur）。
    fn leaf_after(&self, pos: u64) -> Option<(u32, String, Span)> {
        lex::collect_leaves(&self.tree)
            .into_iter()
            .filter(|l| l.start >= pos)
            .min_by_key(|l| l.start)
            .map(|l| {
                (
                    l.kind,
                    self.spelling_by_range(l.start, l.end),
                    Span {
                        start: l.start as usize,
                        end: l.end as usize,
                    },
                )
            })
    }
    /// 叶起始行（快照换行计数）。
    fn line_of_pos(&self, pos: u64) -> u32 {
        let text = self
            .snap
            .read_str(0, pos.min(self.snap.len() as u64))
            .unwrap_or_default();
        1 + text.bytes().filter(|&b| b == b'\n').count() as u32
    }
    fn children(&self, n: u32) -> Vec<u32> {
        self.tree.children(n)
    }
    fn field_child(&self, n: u32, field: &str) -> Option<u32> {
        let f = self.p.field_id(field)?;
        self.tree.field_child(n, f)
    }
    fn expression_kind(&self, node: u32) -> u32 {
        self.k.expression_kind(self.tree.kind(node))
    }

    fn child_of_kind(&self, n: u32, kind: u32) -> Option<u32> {
        self.children(n)
            .into_iter()
            .find(|&c| self.expression_kind(c) == self.k.expression_kind(kind))
    }
    fn check_name(&mut self, id: &Ident) {
        if self.profile == SourceProfile::HostConsole
            && (crate::source::console::is_reserved_name(&id.name)
                || crate::source::platform_io::is_reserved_name(&id.name))
        {
            let span = id.span;
            self.reject(span, || {
                Diagnostic::new(
                    Stage::Text,
                    "source.console-reserved",
                    "console names cannot be bound in the host-console profile",
                    span,
                )
            });
        }
    }
    fn bump(&mut self) -> bool {
        self.visited += 1;
        if self.visited > PROJECTION_NODE_BUDGET {
            if !self.over_budget {
                self.over_budget = true;
                let z = Span { start: 0, end: 0 };
                self.reject(z, || {
                    Diagnostic::new(
                        Stage::Text,
                        "source.resource",
                        "projection work budget exceeded",
                        z,
                    )
                });
            }
            return false;
        }
        true
    }
}

struct Engine<'a> {
    cx: Cx<'a>,
    stack: Vec<Frame>,
    vals: Vec<Val>,
    /// PLAN-960（T-02）：`run_global_item` 的完成槽（const/var 项一次一 个，
    /// 调用方立即取走；不与 vals 的 Stmt/Fn 流混用）。
    global_item: Option<Item>,
}

fn first_leaf_span(cx: &Cx, node: u32) -> Span {
    let mut n = node;
    loop {
        let nd = cx.tree.node(n);
        if nd.flags & FLAG_TRIVIA != 0 || nd.child_count == 0 {
            break;
        }
        let kids = cx.tree.children(n);
        match kids
            .iter()
            .find(|&&c| cx.tree.node(c).flags & FLAG_TRIVIA == 0)
        {
            Some(&c) => n = c,
            None => break,
        }
    }
    cx.span_of(n)
}

/// PLAN-962 (L-19): the (start, end, inclusive) parts of a range-iter
/// rule node (`expr, dots token, expr`).
fn range_parts(cx: &Cx, node: u32) -> (u32, u32, bool) {
    let mut inclusive = false;
    // the dots live under the range-dots rule node (a field child)
    if let Some(dots) = cx.field_child(node, "dots") {
        for c in cx.children(dots) {
            if let Some(dde) = cx.k.tok_dotdoteq {
                if cx.tree.kind(c) == dde {
                    inclusive = true;
                }
            }
        }
    }
    let start_n = cx.field_child(node, "start").expect("range start");
    let end_n = cx.field_child(node, "end").expect("range end");
    (start_n, end_n, inclusive)
}

/// PLAN-981（IF-981-ORIGINAL）：初值是 AGE 资源服务调用时，其返回的端口类型名。
///
/// 与 resolver 的服务分类**同一判据**：`MethodCall{recv: Var(ns), name}` 且
/// `ns.op` 命中声明表 → 该 op 的 `result_port`。其它形状返回 None（调用方具名
/// 拒绝，不猜类型）。
fn age_service_result_port(cx: &Cx, expr_n: u32) -> Option<&'static str> {
    // 表达式节点 → 找到其中的 `MethodCall-like` 形状。CST 里点分调用是
    // `expr` 下的 receiver `ident` + `.` + `ident` + 实参块；这里用词法子叶
    // 序列判定，避免依赖投影后的 AST。
    let mut leaves: Vec<(u64, String)> = Vec::new();
    let mut stack = vec![expr_n];
    while let Some(n) = stack.pop() {
        let kids = cx.children(n);
        if kids.is_empty() {
            if cx.tree.node(n).flags & ace_syntax_runtime::cst::FLAG_TRIVIA == 0 {
                let (a, _) = cx.tree.abs_range(n);
                leaves.push((a, cx.spelling(n)));
            }
        }
        for c in kids.into_iter().rev() {
            stack.push(c);
        }
    }
    leaves.sort_by_key(|(a, _)| *a);
    if leaves.len() < 3 {
        return None;
    }
    let dotted = format!("{}.{}", leaves[0].1, leaves[2].1);
    if let Some(op) = crate::source::age_resources::lookup(&dotted) {
        return Some(op.result_port);
    }
    // 精灵叶属性面（差额表 #4b）：gfx 服务里**静态资源车道**（ValueCtor 纯值
    // 构造 / HostIdentity 宿主注入身份）的端口结果也可声明为模块资源——
    // `image2d.png` → Image2D、`frame2d.at` → Frame2D。逐 Tick 动态
    // （HostAlgorithm：query2d/input/tick/row/render2d）不是静态资源，不进
    // 声明面——保持具名拒。
    if let Some(gop) = crate::source::age_resources::gfx_lookup(&dotted) {
        if matches!(
            gop.lane,
            crate::source::age_resources::AgeGfxLane::ValueCtor
                | crate::source::age_resources::AgeGfxLane::HostIdentity
        ) {
            if let Some(port) = gop.result_port_name() {
                return Some(port);
            }
        }
    }
    None
}

fn ident_of(cx: &mut Cx, leaf: u32) -> Ident {
    let id = Ident {
        name: cx.spelling(leaf),
        span: cx.span_of(leaf),
    };
    cx.check_name(&id);
    id
}

/// PLAN-979（T-H0-I）：AGE 领域候选投影（未受信结构 + 原 span + 表达式）。
///
/// 只在 age-h0 裁剪面（`age-node-item` 规则存在）时有内容；结构名按
/// `contextual` 表声明的 **role** 归属（换拼写不改角色）。
pub(crate) fn project_age_nodes(
    p: &GrammarProgram,
    k: &SourceKinds,
    tree: &CstTree,
    snap: &Snapshot,
    profile: SourceProfile,
) -> Result<Vec<crate::source::age_h0::AgeNodeItem>, Vec<Diagnostic>> {
    let age_kind = match p
        .rules
        .iter()
        .position(|r| r.name == "age-node-item")
        .map(|i| ace_syntax_runtime::program::RULE_KIND_BASE + i as u32)
    {
        Some(kind) => kind,
        None => return Ok(Vec::new()),
    };
    let mut eng = Engine::new(p, k, tree, snap, profile, false, false, false);
    let mut out = Vec::new();
    for d in &tree.decls {
        let node = if tree.kind(d.node) == k.item {
            tree.children(d.node)
                .into_iter()
                .find(|&c| tree.node(c).flags & FLAG_TRIVIA == 0)
                .unwrap_or(d.node)
        } else {
            d.node
        };
        if tree.kind(node) != age_kind {
            continue;
        }
        out.push(eng.run_age_node_item(node));
    }
    if let Some(first) = eng.cx.first_diag() {
        return Err(vec![first]);
    }
    Ok(out)
}

/// PLAN-979（T-03）：AGE **组件面**候选投影（model/on/view 成员 → typed 候选）。
///
/// 只在 `age-component` 裁剪面有内容；结构名按 `contextual` 的 role 归属。
pub(crate) fn project_component_nodes(
    p: &GrammarProgram,
    k: &SourceKinds,
    tree: &CstTree,
    snap: &Snapshot,
    profile: SourceProfile,
    widget_lists: Option<&WidgetListProjection>,
) -> Result<Vec<crate::source::age_component::ComponentNode>, Vec<Diagnostic>> {
    let item_kind = match p
        .rules
        .iter()
        .position(|r| r.name == "age-node-c-item")
        .map(|i| ace_syntax_runtime::program::RULE_KIND_BASE + i as u32)
    {
        Some(kind) => kind,
        None => return Ok(Vec::new()),
    };
    let mut eng = Engine::new(p, k, tree, snap, profile, false, false, true);
    let mut out = Vec::new();
    // PLAN-105 T-02（U03 真语料 013/016）：模块级 `store` 声明的结构候选
    // （字段/消息/处理器）——统一收集后附到各候选节点。
    let store_kind = p
        .rules
        .iter()
        .position(|r| r.name == "age-store-item")
        .map(|i| ace_syntax_runtime::program::RULE_KIND_BASE + i as u32);
    let mut stores = Vec::new();
    for d in &tree.decls {
        let node = if tree.kind(d.node) == k.item {
            tree.children(d.node)
                .into_iter()
                .find(|&c| tree.node(c).flags & FLAG_TRIVIA == 0)
                .unwrap_or(d.node)
        } else {
            d.node
        };
        if Some(tree.kind(node)) == store_kind {
            stores.push(eng.run_component_store_item(node));
            continue;
        }
        // PLAN-981（IF-981-REF）：`pub node Foo { .. }` 壳——可见性位进候选，
        // 实体是它包裹的 node 声明。
        let public = p
            .rules
            .iter()
            .position(|r| r.name == "age-pub-node-item")
            .map(|i| ace_syntax_runtime::program::RULE_KIND_BASE + i as u32)
            .is_some_and(|pk| tree.kind(node) == pk);
        // `pub node` 与本面 node 声明**同形**（head/name/body 字段一致），只是
        // 多一个 `pub` 前缀，故直接投影该壳节点本身（不再找嵌套子节点）。
        if tree.kind(node) != item_kind && !public {
            continue;
        }
        out.push(eng.run_component_node_item(node, public, widget_lists));
    }
    if let Some(first) = eng.cx.first_diag() {
        return Err(vec![first]);
    }
    for n in &mut out {
        n.stores = stores.clone();
    }
    Ok(out)
}

/// PLAN-105 T-02（U03 真语料 013/016）：模块级 `store` 声明的结构候选投影
/// （独立于 node 候选：store 单元可以没有 node）。
pub(crate) fn project_component_stores(
    p: &GrammarProgram,
    k: &SourceKinds,
    tree: &CstTree,
    snap: &Snapshot,
    profile: SourceProfile,
) -> Result<Vec<crate::source::age_component::ComponentStore>, Vec<Diagnostic>> {
    let Some(store_kind) = p
        .rules
        .iter()
        .position(|r| r.name == "age-store-item")
        .map(|i| ace_syntax_runtime::program::RULE_KIND_BASE + i as u32)
    else {
        return Ok(Vec::new());
    };
    let mut eng = Engine::new(p, k, tree, snap, profile, false, false, true);
    let mut out = Vec::new();
    for d in &tree.decls {
        let node = if tree.kind(d.node) == k.item {
            tree.children(d.node)
                .into_iter()
                .find(|&c| tree.node(c).flags & FLAG_TRIVIA == 0)
                .unwrap_or(d.node)
        } else {
            d.node
        };
        if tree.kind(node) == store_kind {
            out.push(eng.run_component_store_item(node));
        }
    }
    if let Some(first) = eng.cx.first_diag() {
        return Err(vec![first]);
    }
    Ok(out)
}

/// PLAN-982（B-CONFIG-I E2）：配置文档候选投影（文档行 + 原 span + 共享表达式）。
///
/// 只在 config-document 裁剪面（`cfg-document-item` 规则存在）时有内容；结构与
/// span 全部来自 CST（`cfg-*` 规则），不做字符串扫描；行的值表达式仍是共享
/// 表达式投影（`project_expr_sync`），不新建表达式求值器。
pub(crate) fn project_config_document(
    p: &GrammarProgram,
    k: &SourceKinds,
    tree: &CstTree,
    snap: &Snapshot,
    profile: SourceProfile,
    grammar_id: String,
    input_hash: String,
) -> Result<crate::source::config_document::ConfigDocumentSyntax, Vec<Diagnostic>> {
    use crate::source::config_document::ConfigDocumentSyntax;
    let item_kind = match config_rule_kind(p, "cfg-document-item") {
        Some(kind) => kind,
        None => {
            return Ok(ConfigDocumentSyntax {
                lines: Vec::new(),
                grammar_id,
                input_hash,
            })
        }
    };
    let mut eng = Engine::new(p, k, tree, snap, profile, false, false, false);
    let mut lines = Vec::new();
    for d in &tree.decls {
        let node = if tree.kind(d.node) == k.item {
            tree.children(d.node)
                .into_iter()
                .find(|&c| tree.node(c).flags & FLAG_TRIVIA == 0)
                .unwrap_or(d.node)
        } else {
            d.node
        };
        if tree.kind(node) != item_kind {
            continue;
        }
        lines.extend(eng.run_cfg_document_item(node));
    }
    // 文档行是严格面：任何 CST 错误都具名拒绝（保原 span），空结果更不能
    // 当成"空文档成功"——否则坏邻例会被静默吞成零行。
    // 具名拒绝要带**有用的位置**：通用"trailing unparsed tokens"（0..0）只是
    // 恢复兜底，若有带真实 span 的槽位/构造级诊断则优先用它（坏邻例仍保原源）。
    let surfaced = tree
        .diags
        .iter()
        .find(|d| d.end > d.start)
        .or_else(|| tree.diags.first());
    if let Some(d) = surfaced {
        return Err(vec![Diagnostic::new(
            Stage::Text,
            "source.syntax",
            d.message.clone(),
            Span {
                start: d.start as usize,
                end: d.end as usize,
            },
        )]);
    }
    if let Some(first) = eng.cx.first_diag() {
        return Err(vec![first]);
    }
    if lines.is_empty() {
        return Err(vec![Diagnostic::new(
            Stage::Text,
            "source.config-document/structure",
            "the document produced no document line (expected `key: value`, a node, or a var/if/for line)",
            Span {
                start: 0,
                end: snap.len() as usize,
            },
        )]);
    }
    Ok(ConfigDocumentSyntax {
        lines,
        grammar_id,
        input_hash,
    })
}

/// 前序遍历收集指定 rule kind 的节点（配置文档 JSON 形条目在尾链里嵌套）。
fn collect_kind(eng: &mut Engine<'_>, node: u32, want: u32, out: &mut Vec<u32>) {
    for c in eng.cx.children(node) {
        if eng.cx.tree.kind(c) == want {
            out.push(c);
        } else {
            collect_kind(eng, c, want, out);
        }
    }
}
/// 规则名 → CST 规则 kind（配置文档规则没有 `SourceKinds` 具名字段）。
fn config_rule_kind(p: &GrammarProgram, name: &str) -> Option<u32> {
    p.rules
        .iter()
        .position(|r| r.name == name)
        .map(|i| ace_syntax_runtime::program::RULE_KIND_BASE + i as u32)
}

/// PLAN-982（B-CONFIG-I E2）：配置文档行投影（挂在共享 `Engine` 上——表达式
/// 仍走共享 `project_expr_sync`）。
impl<'a> Engine<'a> {
    fn cfg_kind(&self, name: &str) -> Option<u32> {
        config_rule_kind(self.cx.p, name)
    }

    /// 顶层文档项：`cfg-document-item` 的子行按源顺序。
    fn run_cfg_document_item(
        &mut self,
        node: u32,
    ) -> Vec<crate::source::config_document::ConfigLine> {
        let line_kind = match self.cfg_kind("cfg-line") {
            Some(k) => k,
            None => return Vec::new(),
        };
        let mut out = Vec::new();
        for c in self.cx.children(node) {
            if self.cx.tree.kind(c) == line_kind {
                out.extend(self.run_cfg_line(c));
            }
        }
        out
    }

    /// 一行文档：`cfg-line` 的派发子节点决定形态。
    fn run_cfg_line(&mut self, node: u32) -> Vec<crate::source::config_document::ConfigLine> {
        use crate::source::config_document::ConfigLine;
        let pair_k = self.cfg_kind("cfg-pair");
        let node_k = self.cfg_kind("cfg-node");
        let var_k = self.cfg_kind("cfg-var");
        let if_k = self.cfg_kind("cfg-if");
        let for_k = self.cfg_kind("cfg-for");
        let json_k = self.cfg_kind("cfg-json-document");
        for c in self.cx.children(node) {
            let kind = self.cx.tree.kind(c);
            if Some(kind) == pair_k {
                return vec![ConfigLine::Pair(self.run_cfg_pair(c))];
            }
            if Some(kind) == node_k {
                return vec![ConfigLine::Node(self.run_cfg_node(c))];
            }
            if Some(kind) == var_k {
                return vec![ConfigLine::Var(self.run_cfg_var(c))];
            }
            if Some(kind) == if_k {
                return vec![ConfigLine::If(self.run_cfg_if(c))];
            }
            if Some(kind) == for_k {
                return vec![ConfigLine::For(self.run_cfg_for(c))];
            }
            if Some(kind) == json_k {
                return self.run_cfg_json_document(c);
            }
        }
        Vec::new()
    }

    /// JSON 形文档：每个条目 = 一个顶层 pair（键可为标识符或字符串字面量）。
    fn run_cfg_json_document(
        &mut self,
        node: u32,
    ) -> Vec<crate::source::config_document::ConfigLine> {
        use crate::source::config_document::{ConfigLine, ConfigPair, ConfigPairValue};
        let entry_k = match self.cfg_kind("cfg-json-entry") {
            Some(k) => k,
            None => return Vec::new(),
        };
        // `cfg-json-fields` 里是「首条目 + comma 尾链」，条目可能嵌在
        // `cfg-json-entry-tail` 内；按**前序遍历**收集，顺序即源顺序。
        let mut entries: Vec<u32> = Vec::new();
        collect_kind(self, node, entry_k, &mut entries);
        let mut out = Vec::new();
        for c in entries {
            let Some(key_rule) = self.cx.field_child(c, "key") else {
                continue;
            };
            let key = self.cfg_key_ident(key_rule);
            let Some(value_node) = self.cx.field_child(c, "value") else {
                continue;
            };
            let value = self.run_cfg_pair_value(value_node);
            out.push(ConfigLine::Pair(ConfigPair {
                key,
                value,
                span: self.cx.span_of(c),
            }));
        }
        // 条目值若都是表达式（不是带体节点），pair 语义已足够；这里保持
        // `ConfigPair` 不动，避免第二套行模型。
        let _ = std::marker::PhantomData::<ConfigPairValue>;
        out
    }

    /// 文档体（节点/if/for 的 `{ … }`）：子行按源顺序。
    fn run_cfg_body(&mut self, node: u32) -> crate::source::config_document::ConfigBody {
        use crate::source::config_document::{ConfigBody, ConfigLine};
        let line_kind = self.cfg_kind("cfg-line");
        let mut lines: Vec<ConfigLine> = Vec::new();
        for c in self.cx.children(node) {
            if Some(self.cx.tree.kind(c)) == line_kind {
                lines.extend(self.run_cfg_line(c));
            }
        }
        ConfigBody {
            lines,
            span: self.cx.span_of(node),
        }
    }

    /// 文档键的 Ident：`cfg-doc-key` 是派发规则，先下潜到**第一个叶子**，
    /// 再按叶子种类解码（字符串键去引号并解码转义；其余按拼写）。
    fn cfg_key_ident(&mut self, field: u32) -> Ident {
        let mut node = field;
        loop {
            if self.cx.tree.node(node).flags & FLAG_TRIVIA != 0 {
                let raw = self.cx.spelling(node);
                return Ident {
                    name: raw,
                    span: self.cx.span_of(node),
                };
            }
            let kids: Vec<u32> = self
                .cx
                .children(node)
                .into_iter()
                .filter(|&n| self.cx.tree.node(n).flags & FLAG_TRIVIA == 0)
                .collect();
            if kids.len() != 1 {
                break;
            }
            node = kids[0];
        }
        if self.cx.tree.kind(node) == self.cx.k.tok_str_lit {
            let raw = self.cx.spelling(node);
            let inner = raw
                .strip_prefix('"')
                .and_then(|s| s.strip_suffix('"'))
                .unwrap_or(raw.as_str());
            Ident {
                name: lex::decode_str_value(inner.as_bytes()),
                span: self.cx.span_of(node),
            }
        } else {
            ident_of(&mut self.cx, node)
        }
    }
    /// `key: value`（值可为表达式或带体节点值）。
    fn run_cfg_pair(&mut self, node: u32) -> crate::source::config_document::ConfigPair {
        use crate::source::config_document::{ConfigPair, ConfigPairValue};
        let key_field = self.cx.field_child(node, "key").expect("cfg pair key leaf");
        let key = self.cfg_key_ident(key_field);
        let value_node = self
            .cx
            .field_child(node, "value")
            .expect("cfg pair value node");
        let value = self.run_cfg_pair_value(value_node);
        ConfigPair {
            key,
            value,
            span: self.cx.span_of(node),
        }
    }

    /// `cfg-pair-value` 派发节点 → 文档值（带体节点或表达式）。
    ///
    /// `cfg-pair-value` 是派发规则：其子节点是 `cfg-value-node` 或表达式派发
    /// 节点；JSON 形文档条目共用同一投影（同一语义，不是第二套行模型）。
    fn run_cfg_pair_value(
        &mut self,
        value_node: u32,
    ) -> crate::source::config_document::ConfigPairValue {
        use crate::source::config_document::ConfigPairValue;
        let inner = self
            .cx
            .children(value_node)
            .into_iter()
            .find(|&c| self.cx.tree.child_count(c) > 0)
            .unwrap_or(value_node);
        let node_kind = self.cfg_kind("cfg-node");
        let value_node_kind = self.cfg_kind("cfg-value-node");
        let block_kind = self.cfg_kind("cfg-object-block");
        let kind = self.cx.tree.kind(inner);
        if Some(kind) == node_kind || Some(kind) == value_node_kind {
            ConfigPairValue::Node(self.run_cfg_node(inner))
        } else if Some(kind) == block_kind {
            // 块形对象值 → 普通对象字面量表达式（物化侧已有对象值路径）。
            ConfigPairValue::Expr(self.run_cfg_object_block(inner))
        } else if crate::source::config_document::wide_scalar_surface_is_open() {
            // PLAN-982（④ S2-D）：门控打开时，**恰好一个 affix-num 叶**的值升成宽
            // 标量（`ConfigPairValue::Scalar`），由 `cfg_put_f64_text`/`u64_text` 写进
            // 文档；默认（门控关）仍走下方通用表达式投影 ⇒ `source.unsupported`。
            let (start, end) = self.cx.tree.abs_range(inner);
            let leaf = self.cx.leaf_after(start);
            match leaf {
                Some((_g, text, span))
                    if span.start == start as usize
                        && span.end == end as usize
                        && crate::source::config_document::parse_wide_scalar(&text).is_some() =>
                {
                    ConfigPairValue::Scalar(
                        crate::source::config_document::parse_wide_scalar(&text)
                            .expect("checked above"),
                    )
                }
                _ => ConfigPairValue::Expr(self.project_expr_sync(inner)),
            }
        } else {
            ConfigPairValue::Expr(self.project_expr_sync(inner))
        }
    }

    /// 块形对象值：`{ k: v … }` → `ObjectLit`（嵌套块形对象递归成嵌套字段）。
    ///
    /// 只接受**表达式**值：块里出现带体节点值（`k: file(…) {…}`）在此位置没有
    /// 对象字段语义，具名拒绝而不是静默丢字段。
    fn run_cfg_object_block(&mut self, node: u32) -> crate::source::ast::Expr {
        use crate::source::ast::{Expr, ExprKind, ObjectField};
        let pair_k = self.cfg_kind("cfg-pair");
        let mut fields: Vec<ObjectField> = Vec::new();
        let mut pairs: Vec<u32> = Vec::new();
        collect_kind(self, node, pair_k.unwrap_or(u32::MAX), &mut pairs);
        for p in pairs {
            let key_leaf = self.cx.field_child(p, "key");
            let value_node = self.cx.field_child(p, "value");
            let (Some(key_leaf), Some(value_node)) = (key_leaf, value_node) else {
                continue;
            };
            // 键可以是 ident 或字符串字面量：字符串键要**解码**（去引号/转义），
            // 否则对象字段名会带引号（真实 pac.at 的 `"@scope/name": …`）。
            let key_text = self.cx.spelling(key_leaf);
            let quoted =
                key_text.len() >= 2 && key_text.starts_with('"') && key_text.ends_with('"');
            let key = if quoted {
                Ident {
                    name: lex::decode_str_value(&key_text.as_bytes()[1..key_text.len() - 1]),
                    span: self.cx.span_of(key_leaf),
                }
            } else {
                ident_of(&mut self.cx, key_leaf)
            };
            let span = self.cx.span_of(p);
            match self.run_cfg_pair_value(value_node) {
                crate::source::config_document::ConfigPairValue::Expr(value) => {
                    fields.push(ObjectField {
                        key: crate::hir::ObjectKey::Text(key.name),
                        value,
                        span,
                    });
                }
                crate::source::config_document::ConfigPairValue::Scalar(_) => {
                    // PLAN-982（④）：宽标量还没有对象字段的承载面（S2-C 之后再谈）。
                    self.cx.reject(span, || {
                        crate::atom_text::Diagnostic::new(
                            crate::atom_text::Stage::Text,
                            "source.config-document/pending",
                            "f64/u64 scalar object fields need the `cfg-put.f64`/`cfg-put.u64` \
                             primitives (PLAN-982 ④ S2-C)",
                            span,
                        )
                    });
                }
                crate::source::config_document::ConfigPairValue::Node(_) => {
                    self.cx.reject(span, || {
                        crate::atom_text::Diagnostic::new(
                            crate::atom_text::Stage::Text,
                            "source.config-document/object-node-value",
                            "a node value with a body cannot be an object field; \
                             use a document line instead",
                            span,
                        )
                    });
                }
            }
        }
        Expr {
            kind: ExprKind::ObjectLit { fields },
            span: self.cx.span_of(node),
        }
    }

    /// 节点语句：头拼写 + 可选 id/参数 + 可选体。
    fn run_cfg_node(&mut self, node: u32) -> crate::source::config_document::ConfigNode {
        use crate::source::config_document::{ConfigNode, ConfigNodeId};
        let head = self.cx.field_child(node, "head").expect("cfg node head");
        let word = self
            .cx
            .field_child(head, "word")
            .expect("cfg node head spelling");
        let name = self.cx.spelling(word);
        let name_span = self.cx.span_of(word);
        let mut id = None;
        let mut args = Vec::new();
        if let Some(tail) = self.cx.field_child(node, "tail") {
            let id_kind = self.cfg_kind("cfg-node-id");
            let args_kind = self.cfg_kind("cfg-node-args");
            let call_kind = self.cfg_kind("cfg-node-call");
            for c in self.cx.children(tail) {
                let kind = self.cx.tree.kind(c);
                if Some(kind) == call_kind {
                    // 节点头模板实参：`name __fstr("", expr)` → 普通位置实参。
                    if let Some(expr) = self.run_cfg_head_call(c) {
                        args.push(crate::source::config_document::ConfigArg::Pos(expr));
                    }
                    continue;
                }
                if Some(kind) == id_kind {
                    if let Some(leaf) = self.cx.field_child(c, "id") {
                        let span = self.cx.span_of(leaf);
                        if self.cx.tree.kind(leaf) == self.cx.k.tok_str_lit {
                            let raw = self.cx.spelling(leaf);
                            let inner = raw
                                .strip_prefix('"')
                                .and_then(|s| s.strip_suffix('"'))
                                .unwrap_or(raw.as_str());
                            let text = lex::decode_str_value(inner.as_bytes());
                            // legacy 口径：节点头里的 `${…}` 是**字面**占位（不插值），
                            // 且同时落成身份与一个位置实参（`_arg0`）。
                            if text.starts_with("${") && text.ends_with('}') {
                                args.push(crate::source::config_document::ConfigArg::Pos(Expr {
                                    span,
                                    kind: super::super::ast::ExprKind::StrLit(text.clone()),
                                }));
                            }
                            id = Some(ConfigNodeId::Text { text, span });
                        } else {
                            id = Some(ConfigNodeId::Bare(Ident {
                                name: self.cx.spelling(leaf),
                                span,
                            }));
                        }
                    }
                } else if Some(kind) == args_kind {
                    args = self.run_cfg_args(c);
                }
            }
        }
        let body = self
            .cx
            .field_child(node, "body")
            .map(|b| self.run_cfg_body(b));
        ConfigNode {
            name,
            name_span,
            id,
            args,
            body,
            span: self.cx.span_of(node),
        }
    }

    /// 节点头模板实参：`__fstr("", expr)` 形调用 → 位置实参表达式。
    fn run_cfg_head_call(&mut self, node: u32) -> Option<crate::source::ast::Expr> {
        use crate::source::ast::{Arg, Expr, ExprKind};
        let word = self.cx.field_child(node, "word")?;
        let name = self.cx.spelling(word);
        let mut call_args: Vec<Arg> = Vec::new();
        for arg in self.run_cfg_args(node) {
            match arg {
                crate::source::config_document::ConfigArg::Pos(e) => call_args.push(Arg::Pos(e)),
                crate::source::config_document::ConfigArg::Named { name, value } => {
                    call_args.push(Arg::Named {
                        span: name.span.join(value.span),
                        name,
                        value,
                    })
                }
            }
        }
        Some(Expr {
            span: self.cx.span_of(node),
            kind: ExprKind::Call {
                callee: Ident {
                    name,
                    span: self.cx.span_of(word),
                },
                args: call_args,
            },
        })
    }
    /// 节点参数表：位置/具名实参按源顺序（`args` 规则 = 首 `arg` + `arg-tail` 链）。
    fn run_cfg_args(&mut self, node: u32) -> Vec<crate::source::config_document::ConfigArg> {
        use crate::source::config_document::ConfigArg;
        let mut out = Vec::new();
        let args_n = match self.cx.field_child(node, "args") {
            Some(a) => a,
            None => return out,
        };
        let arg_kind = match self.cfg_kind("arg") {
            Some(k) => k,
            None => return out,
        };
        let arg_nodes = if self.cx.children(args_n).is_empty() {
            Vec::new()
        } else {
            collect_seq(&self.cx, args_n, arg_kind, "arg-tail")
        };
        let named_kind = self.cfg_kind("named-arg");
        for arg in arg_nodes {
            let inner = self.cx.children(arg).into_iter().find(|&c| {
                self.cx.tree.child_count(c) > 0 || Some(self.cx.tree.kind(c)) == named_kind
            });
            let Some(inner) = inner else { continue };
            if Some(self.cx.tree.kind(inner)) == named_kind {
                let label = self.cx.field_child(inner, "label");
                let value = self.cx.field_child(inner, "value");
                if let (Some(label), Some(value)) = (label, value) {
                    let name = ident_of(&mut self.cx, label);
                    out.push(ConfigArg::Named {
                        name,
                        value: self.project_expr_sync(value),
                    });
                }
            } else {
                out.push(ConfigArg::Pos(self.project_expr_sync(inner)));
            }
        }
        out
    }

    /// `var <name> = <expr>`（类型注解属 E3 值面，当前具名拒绝）。
    fn run_cfg_var(&mut self, node: u32) -> crate::source::config_document::ConfigVar {
        use crate::source::config_document::ConfigVar;
        let kw = self.cx.field_child(node, "kw").expect("cfg var kw");
        let name_leaf = self.cx.field_child(node, "name").expect("cfg var name");
        let name = ident_of(&mut self.cx, name_leaf);
        if let Some(ann) = self.cx.field_child(node, "ann") {
            let span = self.cx.span_of(ann);
            self.cx.reject(span, || {
                Diagnostic::new(
                    Stage::Text,
                    "source.config-document/annotation",
                    "typed document locals (`var x: T = …`) are not part of the current \
                     document slice; drop the annotation",
                    span,
                )
            });
        }
        let value_n = self.cx.field_child(node, "value").expect("cfg var value");
        ConfigVar {
            kw_span: self.cx.span_of(kw),
            name,
            value: self.project_expr_sync(value_n),
            span: self.cx.span_of(node),
        }
    }

    /// `if <cond> { … } [else { … }]`。
    fn run_cfg_if(&mut self, node: u32) -> crate::source::config_document::ConfigIf {
        use crate::source::config_document::ConfigIf;
        let cond_n = self.cx.field_child(node, "cond").expect("cfg if cond");
        let cond = self.project_expr_sync(cond_n);
        let then_n = self.cx.field_child(node, "then").expect("cfg if then");
        let then = self.run_cfg_body(then_n);
        let else_kind = self.cfg_kind("cfg-else");
        let body_kind = self.cfg_kind("cfg-body");
        let mut els = None;
        for c in self.cx.children(node) {
            if Some(self.cx.tree.kind(c)) != else_kind {
                continue;
            }
            if let Some(b) = self
                .cx
                .children(c)
                .into_iter()
                .find(|&g| Some(self.cx.tree.kind(g)) == body_kind)
            {
                els = Some(self.run_cfg_body(b));
            }
        }
        ConfigIf {
            cond,
            then,
            els,
            span: self.cx.span_of(node),
        }
    }

    /// `for <name> in <expr> { … }`。
    fn run_cfg_for(&mut self, node: u32) -> crate::source::config_document::ConfigFor {
        use crate::source::config_document::ConfigFor;
        let var_leaf = self.cx.field_child(node, "var").expect("cfg for var");
        let var = ident_of(&mut self.cx, var_leaf);
        let iter_n = self.cx.field_child(node, "iter").expect("cfg for iter");
        // `cfg-iter` 两形：裸名（文档最常见的列表局部）或括号表达式。
        let iter = {
            let inner = self.cx.children(iter_n).into_iter().find(|&c| {
                self.cx.tree.child_count(c) > 0 || self.cx.tree.kind(c) == self.cx.k.tok_ident
            });
            match inner {
                Some(c) if self.cx.tree.kind(c) == self.cx.k.tok_ident => Expr {
                    span: self.cx.span_of(c),
                    kind: ExprKind::Var(Ident {
                        name: self.cx.spelling(c),
                        span: self.cx.span_of(c),
                    }),
                },
                Some(c) => {
                    if let Some(expr_n) = self.cx.field_child(c, "inner") {
                        self.project_expr_sync(expr_n)
                    } else {
                        self.project_expr_sync(c)
                    }
                }
                None => {
                    let span = self.cx.span_of(iter_n);
                    self.cx.reject(span, || {
                        Diagnostic::new(
                            Stage::Text,
                            "source.config-document/for-iter",
                            "expected the iterated value after `in`",
                            span,
                        )
                    });
                    Expr {
                        span,
                        kind: ExprKind::IntLit(0),
                    }
                }
            }
        };
        let body_n = self.cx.field_child(node, "body").expect("cfg for body");
        let body = self.run_cfg_body(body_n);
        ConfigFor {
            var,
            iter,
            body,
            span: self.cx.span_of(node),
        }
    }
}

impl<'a> Engine<'a> {
    fn new(
        p: &'a GrammarProgram,
        k: &'a SourceKinds,
        tree: &'a CstTree,
        snap: &'a Snapshot,
        profile: SourceProfile,
        ct_face: bool,
        instances_face: bool,
        age_face: bool,
    ) -> Self {
        Engine {
            cx: Cx {
                p,
                k,
                tree,
                snap,
                profile,
                ct_face,
                instances_face,
                age_face,
                ty_params: Vec::new(),
                visited: 0,
                diags: Vec::new(),
                annotations: Vec::new(),
                over_budget: false,
            },
            stack: Vec::new(),
            vals: Vec::new(),
            global_item: None,
        }
    }

    fn pop_e(&mut self) -> Expr {
        match self.vals.pop() {
            Some(Val::E(e)) => e,
            _ => Expr {
                kind: ExprKind::IntLit(0),
                span: Span { start: 0, end: 0 },
            },
        }
    }
    fn pop_b(&mut self) -> Block {
        match self.vals.pop() {
            Some(Val::B(b)) => b,
            _ => Block {
                stmts: Vec::new(),
                span: Span { start: 0, end: 0 },
            },
        }
    }
    fn pop_s(&mut self) -> Stmt {
        match self.vals.pop() {
            Some(Val::S(s)) => s,
            _ => Stmt::Break {
                span: Span { start: 0, end: 0 },
            },
        }
    }

    fn drive(&mut self) {
        while let Some(frame) = self.stack.pop() {
            if !self.cx.bump() {
                // 预算耗尽：保留诊断，清栈提前结束（值由调用方兜底）。
                self.stack.clear();
                return;
            }
            self.step(frame);
        }
    }

    // ------------------------------------------------------------ items

    /// PLAN-979：某规则第一个上下文名称步声明的 (role, spelling)。
    /// 投影按 role 归属领域语义——换 Schema 拼写不改角色。
    fn ctx_role_of_rule(&self, rule: &str) -> Option<(String, String)> {
        let r = self.cx.p.rules.iter().find(|r| r.name == rule)?;
        for alt in &r.alts {
            for s in &alt.steps {
                if let ace_syntax_runtime::program::Atom::CtxName(c) = s.atom {
                    if let Some(spec) = self.cx.p.ctx_name(c) {
                        return Some((spec.role.clone(), spec.spelling.clone()));
                    }
                }
            }
        }
        None
    }

    /// PLAN-979（T-H0-I）：一个 AGE node 声明的未受信候选投影。
    /// PLAN-1095: the ordinary Auto face projects an H0 node declaration as
    /// an opaque AST item (name + span; the body is never re-projected).
    fn run_ast_node_item(&mut self, node: u32) -> crate::source::ast::NodeItem {
        use crate::source::ast::{Ident, NodeItem};
        let name_leaf = self
            .cx
            .field_child(node, "name")
            .expect("age node name leaf");
        let name = ident_of(&mut self.cx, name_leaf);
        let span = self.cx.span_of(node);
        NodeItem { name, span }
    }

    fn run_age_node_item(&mut self, node: u32) -> crate::source::age_h0::AgeNodeItem {
        use crate::source::age_h0::{AgeNodeItem, AgeTextLeaf, AgeViewNode};
        let head_node = self
            .cx
            .field_child(node, "head")
            .expect("age node head rule");
        let head_leaf = self
            .cx
            .field_child(head_node, "kw")
            .expect("age node head spelling leaf");
        let head_spelling = self.cx.spelling(head_leaf);
        let head_span = self.cx.span_of(head_leaf);
        let head_role = self
            .ctx_role_of_rule("age-node-head")
            .map(|(role, _)| role)
            .unwrap_or_else(|| "node".to_string());
        let name_leaf = self
            .cx
            .field_child(node, "name")
            .expect("age node name leaf");
        let name = ident_of(&mut self.cx, name_leaf);
        let body = self.cx.field_child(node, "body").expect("age node body");
        let view_leaf = self.cx.field_child(body, "view").expect("age view head");
        let view_spelling = self.cx.spelling(view_leaf);
        let view_span = self.cx.span_of(view_leaf);
        let view_role = self
            .ctx_role_of_rule("age-node-body")
            .map(|(role, _)| role)
            .unwrap_or_else(|| "view".to_string());
        let text_role = self
            .ctx_role_of_rule("age-text-item")
            .map(|(role, _)| role)
            .unwrap_or_else(|| "text".to_string());
        let text_kind = self
            .cx
            .p
            .rules
            .iter()
            .position(|r| r.name == "age-text-item")
            .map(|i| ace_syntax_runtime::program::RULE_KIND_BASE + i as u32);
        let mut texts = Vec::new();
        for c in self.cx.children(body) {
            if Some(self.cx.tree.kind(c)) != text_kind {
                continue;
            }
            let comp = self
                .cx
                .field_child(c, "component")
                .expect("age text component leaf");
            let expr_node = self.cx.field_child(c, "text").expect("age text expression");
            let expr = self.project_expr_sync(expr_node);
            texts.push(AgeTextLeaf {
                role: text_role.clone(),
                spelling: self.cx.spelling(comp),
                component_span: self.cx.span_of(comp),
                expr,
                span: self.cx.span_of(c),
            });
        }
        if texts.len() != 1 {
            // H0 冻结形状：一个 view 恰一个 text 根（0/多 都具名拒，不静默取首个）。
            let span = self.cx.span_of(body);
            let found = texts.len();
            self.cx.reject(span, move || {
                Diagnostic::new(
                    Stage::Text,
                    "source.age.text-count",
                    format!(
                        "the H0 AGE slice holds exactly one `text` root per view; found {found}"
                    ),
                    span,
                )
            });
        }
        // 兼容别名判定按**Schema 声明**：同一 role 的首个声明是规范拼写，
        // 其余同 role 声明是别名（不在代码里写死 `widget` 字面量）。
        let canonical_head = self
            .cx
            .p
            .ctx_names
            .iter()
            .find(|n| n.role == head_role)
            .map(|n| n.spelling.clone())
            .unwrap_or_else(|| head_spelling.clone());
        AgeNodeItem {
            head_role,
            head_spelling: head_spelling.clone(),
            head_span,
            legacy_widget: head_spelling != canonical_head,
            name,
            view: AgeViewNode {
                role: view_role,
                spelling: view_spelling,
                span: view_span,
                texts,
            },
            span: self.cx.span_of(node),
        }
    }

    /// PLAN-105 T-02（U03 真语料 013/016）：`store <Name> { … }` → `ComponentStore`
    /// 结构候选（字段/消息/处理器与 span）。字段投影沿用 model 字段规则（与
    /// 组件 node 同一条拼写/初值/可变位判定；此处内联复制以免动摇既有大 match）。
    fn run_component_store_item(
        &mut self,
        node: u32,
    ) -> crate::source::age_component::ComponentStore {
        use crate::source::age_component::ComponentStore;
        let name_leaf = self
            .cx
            .field_child(node, "name")
            .expect("store declaration name");
        let name = ident_of(&mut self.cx, name_leaf);
        let mut fields: Vec<crate::source::age_component::ComponentField> = Vec::new();
        let mut messages = Vec::new();
        let mut handlers = Vec::new();
        let mut members: Vec<u32> = Vec::new();
        for child in self.cx.children(node) {
            if self.rule_name_of(child) == "age-store-member" {
                // `many { age-store-member }` 的包装层——真实成员是它的首个
                // 非 trivia 孩子。
                if let Some(inner) = self
                    .cx
                    .children(child)
                    .into_iter()
                    .find(|&c| self.cx.tree.node(c).flags & FLAG_TRIVIA == 0)
                {
                    members.push(inner);
                }
            } else {
                members.push(child);
            }
        }
        for member in members {
            match self.rule_name_of(member).as_str() {
                "age-model-member" => {
                    for field in self.cx.children(member) {
                        let field_kind = self.cx.tree.kind(field);
                        let is_field = self.cx.p.rules.iter().enumerate().any(|(i, r)| {
                            r.name == "age-model-field"
                                && ace_syntax_runtime::program::RULE_KIND_BASE + i as u32
                                    == field_kind
                        });
                        if !is_field {
                            continue;
                        }
                        let Some(ty_node) = self.cx.field_child(field, "ty") else {
                            // 视图数据字段（无类型列表字面量）——store 场合同样只读。
                            continue;
                        };
                        let ty_leaf = self
                            .cx
                            .children(ty_node)
                            .into_iter()
                            .find(|&c| self.cx.tree.node(c).flags & FLAG_TRIVIA == 0)
                            .unwrap_or(ty_node);
                        let init_node = self
                            .cx
                            .field_child(field, "init")
                            .expect("store model field init");
                        let init = self.project_expr_sync(init_node);
                        let mutable = self
                            .cx
                            .field_child(field, "kw")
                            .map(|kw| self.cx.spelling(kw) == "var")
                            .unwrap_or(false);
                        let name_leaf = self
                            .cx
                            .field_child(field, "field")
                            .expect("store model field name");
                        fields.push(crate::source::age_component::ComponentField {
                            name: self.cx.spelling(name_leaf),
                            ty: self.cx.spelling(ty_leaf),
                            ty_span: self.cx.span_of(ty_leaf),
                            collection_type: self.cx.field_child(ty_node, "generic").map(|generic| self.project_generic_ty(generic)),
                            init,
                            mutable,
                            span: self.cx.span_of(field),
                        });
                    }
                }
                "age-msg-member" => self.collect_msg_variants(member, &mut messages),
                "age-on-widget-member" => {
                    self.collect_widget_handlers(member, &mut handlers, &[]);
                }
                _ => {}
            }
        }
        ComponentStore {
            name,
            fields,
            messages,
            handlers,
            span: self.cx.span_of(node),
        }
    }

    /// PLAN-979（T-03）：一个组件 node 声明的未受信候选（model 字段 / on 处理器 / view）。
    fn run_component_node_item(
        &mut self,
        node: u32,
        public: bool,
        widget_lists: Option<&WidgetListProjection>,
    ) -> crate::source::age_component::ComponentNode {
        use crate::source::age_component::{
            ComponentHandler, ComponentMemberRow, ComponentNode, ComponentView,
        };
        let head_node = self
            .cx
            .field_child(node, "head")
            .expect("component node head rule");
        let head_leaf = self
            .cx
            .field_child(head_node, "kw")
            .expect("component node head spelling leaf");
        let head_spelling = self.cx.spelling(head_leaf);
        let head_span = self.cx.span_of(head_leaf);
        let head_role = self
            .ctx_role_of_rule("age-node-head")
            .map(|(role, _)| role)
            .unwrap_or_else(|| "node".to_string());
        let canonical_head = self
            .cx
            .p
            .ctx_names
            .iter()
            .find(|n| n.role == head_role)
            .map(|n| n.spelling.clone())
            .unwrap_or_else(|| head_spelling.clone());
        let name_leaf = self
            .cx
            .field_child(node, "name")
            .expect("component node name leaf");
        let mut name = ident_of(&mut self.cx, name_leaf);
        let authored_name = name.name.clone();
        if let Some(specialized) = widget_lists.and_then(|projection| projection.specialized_name.as_ref()) {
            name.name.clone_from(specialized);
        }
        let body = self
            .cx
            .field_child(node, "body")
            .expect("component node body");
        // PLAN-981（IF-981-ORIGINAL）：作者 typed 参数（`(x: float, …)`）。
        // 语法面已产出；此处投影，避免合法参数被静默丢弃。
        let mut params = Vec::new();
        if let Some(pn) = self.cx.field_child(node, "params") {
            // 逐个 `age-node-param`：第一个是 `first`，后续嵌在 `age-node-param-tail`
            // 里面（逗号 + param），故按子树收集而不是只看直接孩子。
            let want = |slf: &Self, name: &str, n: u32| -> bool {
                slf.cx.p.rules.iter().enumerate().any(|(i, r)| {
                    r.name == name
                        && ace_syntax_runtime::program::RULE_KIND_BASE + i as u32
                            == slf.cx.tree.kind(n)
                })
            };
            let mut collect = |slf: &mut Self, n: u32| {
                let name_leaf = slf.cx.field_child(n, "name").expect("param name");
                let ty_node = slf.cx.field_child(n, "ty").expect("param type");
                let ty_leaf = slf
                    .cx
                    .children(ty_node)
                    .into_iter()
                    .find(|&x| {
                        slf.cx.tree.node(x).flags & ace_syntax_runtime::cst::FLAG_TRIVIA == 0
                    })
                    .unwrap_or(ty_node);
                let name = slf.cx.spelling(name_leaf);
                let collection_type = slf.cx.field_child(ty_node, "generic")
                    .map(|generic| slf.project_generic_ty(generic));
                let ty = match &collection_type {
                    Some(TypeAnn { kind: TypeRef::Generic { name, .. }, .. }) => name.name.clone(),
                    _ => slf.cx.spelling(ty_leaf),
                };
                let ty_span = slf.cx.span_of(ty_leaf);
                let span = slf.cx.span_of(n);
                let default = slf
                    .cx
                    .field_child(n, "default")
                    .and_then(|clause| slf.cx.field_child(clause, "value"))
                    .map(|value| slf.project_expr_sync(value));
                params.push(crate::source::age_component::ComponentParam {
                    name,
                    ty,
                    ty_span,
                    collection_type,
                    default,
                    span,
                });
            };
            for c in self.cx.children(pn) {
                if want(self, "age-node-param", c) {
                    collect(self, c);
                } else if want(self, "age-node-param-tail", c) {
                    for inner in self.cx.children(c) {
                        if want(self, "age-node-param", inner) {
                            collect(self, inner);
                        }
                    }
                }
            }
        }
        let text_role = self
            .ctx_role_of_rule("age-text-item")
            .map(|(role, _)| role)
            .unwrap_or_else(|| "text".to_string());
        let mut fields = Vec::new();
        let mut package_imports = Vec::new();
        // PLAN-1072 刀B：仅视图数据字段（无类型列表字面量——1071 边界）。
        let mut view_data = Vec::new();
        let mut view_data_nodes = Vec::new();
        let mut handlers = Vec::new();
        let mut texts = Vec::new();
        let mut node_refs = Vec::new();
        // PLAN-981（IF-981-ORIGINAL）：场景出口里的组件实例行。
        let mut scene_components = Vec::new();
        let mut view_role = "view".to_string();
        // PLAN-981（轮 51）：view 出口（`view` / `view-2d` / `view-3d`）是作者输入，
        // 必须按**后缀 token 的字节**记录——grammar 的 2d/3d 成员 alt 共用同一
        // `name(age-view-head)` 与同形后缀规则，命中哪个 alt 不能决定出口。
        let mut view_exit = "view".to_string();
        let mut view_span = Span { start: 0, end: 0 };
        // PLAN-981（B-LANGUAGE-I）：**多出口**——不同键并存、同键重复拒绝。此前 `view_role`
        // 只是被重复赋值，于是两个 `view-2d`（或两个 `view`、或 2D+3D 并存）
        // 会静默合并——后者还把两个出口的行融成一条扁平列表，丢失归属。
        // 现在是 AGE-L03 的多出口规则（同键重复仍是具名错误）。
        let mut exits: Vec<crate::source::age_component::ComponentExit> = Vec::new();
        let mut text_exits: Vec<Option<String>> = Vec::new();
        let mut named_trees: Vec<(String, Vec<ComponentViewItem>)> = Vec::new();
        // PLAN-980（B-DIALECT-I 完整批）：widget 成员槽位投影状态。
        let mut messages = Vec::new();
        let mut message_set_name: Option<String> = None;
        let mut design = Vec::new();
        let mut member_rows = Vec::new();
        let mut tree = Vec::new();
        for raw in self.cx.children(body) {
            // 成员列表的每项先由 `age-node-member` 的 alt 包一层；取其实质成员。
            let member = self
                .cx
                .children(raw)
                .into_iter()
                .find(|&c| self.cx.tree.node(c).flags & FLAG_TRIVIA == 0)
                .unwrap_or(raw);
            let member_kind = self.cx.tree.kind(member);
            let member_rule = self
                .cx
                .p
                .rules
                .iter()
                .enumerate()
                .find(|(i, _)| {
                    ace_syntax_runtime::program::RULE_KIND_BASE + *i as u32 == member_kind
                })
                .map(|(_, r)| r.name.clone())
                .unwrap_or_default();
            match member_rule.as_str() {
                "age-package-use-member" => {
                    let entry_kind = self
                        .rule_kind_of("age-package-use-entry")
                        .expect("component package entry rule");
                    for entry in self.cx.children(member) {
                        if self.cx.tree.kind(entry) != entry_kind {
                            continue;
                        }
                        let package = self
                            .cx
                            .field_child(entry, "package")
                            .expect("package name");
                        let source = self
                            .cx
                            .field_child(entry, "source")
                            .expect("package directory");
                        let quoted = self.cx.spelling(source);
                        package_imports.push(crate::source::age_component::ComponentPackageImport {
                            package: Ident {
                                name: self.cx.spelling(package),
                                span: self.cx.span_of(package),
                            },
                            directory: lex::decode_str_value(&quoted.as_bytes()[1..quoted.len() - 1]),
                            span: self.cx.span_of(entry),
                        });
                    }
                }
                "age-model-member" => {
                    for field in self.cx.children(member) {
                        let field_kind = self.cx.tree.kind(field);
                        let is_field = self.cx.p.rules.iter().enumerate().any(|(i, r)| {
                            r.name == "age-model-field"
                                && ace_syntax_runtime::program::RULE_KIND_BASE + i as u32
                                    == field_kind
                        });
                        if !is_field {
                            continue;
                        }
                        let name_leaf = self
                            .cx
                            .field_child(field, "field")
                            .expect("model field name");
                        let name_spelled = self.cx.spelling(name_leaf);
                        let init_node = self
                            .cx
                            .field_child(field, "init")
                            .expect("model field init");
                        // PLAN-1072 刀B：无类型形（`var faqs = [ … ]`）——仅
                        // 视图数据字段（1071 边界）。init=age-view-data-lit
                        // 节点的**原始 span 文本**（惰性数据——不经表达式投
                        // 影/verify；文法已限定列表字面量形）。
                        let Some(ty_node) = self.cx.field_child(field, "ty") else {
                            view_data_nodes.push(init_node);
                            view_data.push(crate::source::age_component::ComponentViewDataField {
                                name: name_spelled,
                                literal: self.cx.spelling(init_node),
                                span: self.cx.span_of(field),
                            });
                            continue;
                        };
                        let ty_leaf = self
                            .cx
                            .children(ty_node)
                            .into_iter()
                            .find(|&c| self.cx.tree.node(c).flags & FLAG_TRIVIA == 0)
                            .unwrap_or(ty_node);
                        let init = self.project_expr_sync(init_node);
                        let collection_type = self.cx.field_child(ty_node, "generic")
                            .map(|generic| self.project_generic_ty(generic));
                        let mutable = self
                            .cx
                            .field_child(field, "kw")
                            .map(|kw| self.cx.spelling(kw) == "var")
                            .unwrap_or(false);
                        fields.push(ComponentField {
                            name: self.cx.spelling(name_leaf),
                            ty: self.cx.spelling(ty_leaf),
                            ty_span: self.cx.span_of(ty_leaf),
                            collection_type,
                            init,
                            mutable,
                            span: self.cx.span_of(field),
                        });
                    }
                }
                "age-on-member" => {
                    let msg_leaf = self.cx.field_child(member, "msg").expect("on message name");
                    // PLAN-981：限定消息名（`Life.Tick`）——尾巴只在 age-package
                    // 裁剪存在；拼成与单段名同形的点分串交领域层，领域层按
                    // Schema 语义解释（不在此处硬编码任何消息名）。
                    let mut msg = self.cx.spelling(msg_leaf);
                    if let Some(tail) = self.cx.field_child(member, "msg_tail") {
                        if let Some(seg) = self.cx.field_child(tail, "seg") {
                            msg.push('.');
                            msg.push_str(&self.cx.spelling(seg));
                        }
                    }
                    // PLAN-981（G2）：`on` 体按**源序**收完整语句面
                    // （assign / hook-local let / if-else；后两者只在 age-package
                    // 裁剪存在）。赋值子集同步填 `assigns`（旧消费者兼容）。
                    let body_children: Vec<u32> = self
                        .cx
                        .children(member)
                        .into_iter()
                        .filter(|&c| self.rule_name_of(c) == "age-on-stmt")
                        .collect();
                    let mut stmts = self.project_on_stmts(&body_children);
                    Self::resolve_handler_scope(&mut stmts, &mut BTreeSet::new(), false);
                    let body_foreign = Self::stmts_have_foreign_call(&stmts);
                    // 兼容面：赋值子集=**全深度**收平（含 if/else 分支里的赋值，
                    // 源序）——"写了哪些字段"不因藏在分支里而消失；**执行语义**
                    // （条件/顺序）以 `stmts` 为准，assign-only 消费者见 Let/If
                    // 必须具名拒（见 ComponentHandler.stmts 契约）。
                    let mut assigns = Vec::new();
                    Self::flat_handler_assigns(&stmts, &mut assigns);
                    handlers.push(ComponentHandler {
                        msg,
                        // 979 形态 `on <Msg> { … }` 没有参数段（参数属于 #6i widget 形）。
                        params: Vec::new(),
                        param_annotations: Vec::new(),
                        from_widget: false,
                        body_foreign,
                        // 979 形态无 widget 头参数——无通道面。
                        channel_calls: Vec::new(),
                        stmts,
                        assigns,
                        span: self.cx.span_of(member),
                    });
                }
                // PLAN-981（IF-981-ORIGINAL）：2D/3D 场景出口。成员是组件实例
                // （`Disc red_disc { x: -20.0 … }`），不是文本叶/节点引用——必须
                // 单独处理，否则实例块会被当成普通成员解析（实测把 `Material2D`
                // 变成了一个不存在的类型注解）。
                "age-view2d-member" | "age-view3d-member" => {
                    let view_leaf = self
                        .cx
                        .field_child(member, "view")
                        .expect("scene view head spelling leaf");
                    view_role = self
                        .ctx_role_of_rule("age-view-head")
                        .map(|(role, _)| role)
                        .unwrap_or_else(|| "view".to_string());
                    view_span = self.cx.span_of(view_leaf);
                    // 出口由**后缀 token 的字节**决定（`2d` / `3d`）；未知后缀具名拒绝，
                    // 不让 `view-2x` 之类的拼写静默落成普通 `view`。
                    let suffix_leaf = self
                        .cx
                        .field_child(member, "suffix")
                        .and_then(|s| self.cx.field_child(s, "word"));
                    let suffix = suffix_leaf.map(|w| self.cx.spelling(w)).unwrap_or_default();
                    let kind = match suffix.as_str() {
                        "2d" => "view-2d".to_string(),
                        "3d" => "view-3d".to_string(),
                        "portrait" => "view-portrait".to_string(),
                        "widget" => "view-widget".to_string(),
                        _ => {
                            let span = suffix_leaf.map(|w| self.cx.span_of(w)).unwrap_or(view_span);
                            let msg = format!(
                                "a view exit must be `view-2d`, `view-3d`, `view-portrait` or `view-widget`; \
                                 `view-{suffix}` is not an AGE export"
                            );
                            self.cx.reject(span, move || {
                                Diagnostic::new(
                                    Stage::Text,
                                    "source.age.view-suffix",
                                    msg.clone(),
                                    span,
                                )
                            });
                            "view".to_string()
                        }
                    };
                    // B-LANGUAGE-I（AGE-L03）：不同键的直属出口可以并存；**同一键**
                    // 写两次仍拒绝（"重复直属选择键拒绝"）。出口值由后缀字节落定。
                    if exits.iter().any(|e| e.kind == kind) {
                        let msg = format!(
                            "the `{}` exit is declared twice; distinct exits (view / view-2d / \
                             view-3d / view-portrait / view-widget) may coexist, but the same \
                             exit key may appear only once",
                            kind
                        );
                        self.cx.reject(view_span, move || {
                            Diagnostic::new(
                                Stage::Text,
                                "source.age.view-exit",
                                msg.clone(),
                                view_span,
                            )
                        });
                    } else {
                        if exits.is_empty() {
                            view_exit = kind.clone();
                        }
                        exits.push(crate::source::age_component::ComponentExit {
                            kind: kind.clone(),
                            span: view_span,
                        });
                    }
                    while text_exits.len() < texts.len() {
                        text_exits.push(Some(kind.clone()));
                    }
                    for raw in self.cx.children(member) {
                        let c = self
                            .cx
                            .children(raw)
                            .into_iter()
                            .find(|&x| {
                                self.cx.tree.node(x).flags & ace_syntax_runtime::cst::FLAG_TRIVIA
                                    == 0
                            })
                            .unwrap_or(raw);
                        let is_comp = self.cx.p.rules.iter().enumerate().any(|(i, r)| {
                            r.name == "age-comp-item"
                                && ace_syntax_runtime::program::RULE_KIND_BASE + i as u32
                                    == self.cx.tree.kind(c)
                        });
                        if !is_comp {
                            continue;
                        }
                        // 组件实例 → 一条场景行：kind + name + 属性值表达式。
                        let kind_leaf = self.cx.field_child(c, "kind").expect("component kind");
                        let name_leaf = self.cx.field_child(c, "name").expect("component name");
                        let kind = self.cx.spelling(kind_leaf);
                        let cname = self.cx.spelling(name_leaf);
                        let mut attrs = Vec::new();
                        for a in self.cx.children(c) {
                            let is_attr = self.cx.p.rules.iter().enumerate().any(|(i, r)| {
                                r.name == "age-attr"
                                    && ace_syntax_runtime::program::RULE_KIND_BASE + i as u32
                                        == self.cx.tree.kind(a)
                            });
                            if !is_attr {
                                continue;
                            }
                            // PLAN-981（G1）：`age-attr` 现在是双臂包装——内层
                            // 是 `age-layer-attr`（冻结 `layer: render-only` 行，
                            // 值是单 token）或 `age-attr-generic`（普通值）。
                            let inner = self
                                .cx
                                .children(a)
                                .into_iter()
                                .find(|&x| {
                                    self.cx.tree.node(x).flags
                                        & ace_syntax_runtime::cst::FLAG_TRIVIA
                                        == 0
                                })
                                .unwrap_or(a);
                            let inner_rule = self.rule_name_of(inner);
                            let key_leaf = self.cx.field_child(inner, "key").expect("attr key");
                            let key = self.cx.spelling(key_leaf);
                            let value_n = self.cx.field_child(inner, "value").expect("attr value");
                            let value = if inner_rule == "age-layer-attr" {
                                // 冻结符号：值 token 的字节序已由文法钉死；
                                // 投影成 Var("render-only")（原 span），前门按
                                // AttrValueKind::Layer 校验并落 typed 行事实。
                                let tok_span = self.cx.span_of(value_n);
                                crate::source::ast::Expr {
                                    span: tok_span,
                                    kind: crate::source::ast::ExprKind::Var(
                                        crate::source::ast::Ident {
                                            name: "render-only".to_string(),
                                            span: tok_span,
                                        },
                                    ),
                                }
                            } else {
                                self.project_expr_sync(value_n)
                            };
                            attrs.push(crate::source::age_component::ComponentSceneAttr {
                                key,
                                value,
                                span: self.cx.span_of(a),
                            });
                        }
                        scene_components.push(
                            crate::source::age_component::ComponentSceneInstance {
                                kind,
                                name: cname,
                                attrs,
                                span: self.cx.span_of(c),
                            },
                        );
                    }
                }
                "age-view-member" | "age-view-portrait-member" | "age-view-widget-member" => {
                    let view_leaf = self
                        .cx
                        .field_child(member, "view")
                        .expect("view head spelling leaf");
                    view_role = self
                        .ctx_role_of_rule("age-view-member")
                        .map(|(role, _)| role)
                        .unwrap_or_else(|| "view".to_string());
                    view_span = self.cx.span_of(view_leaf);
                    let mut kind = "view".to_string();
                    // B-LANGUAGE-I（AGE-L03）：portrait/widget 是直属呈现出口。两条
                    // 成员规则的形状相同（`view - <suffix>`），所以出口必须按**后缀
                    // 字节**落定，不能按命中的规则名（否则 `view-widget` 会先命中
                    // portrait alt 而记成 portrait）。
                    if matches!(
                        member_rule.as_str(),
                        "age-view-portrait-member" | "age-view-widget-member"
                    ) {
                        let suffix_leaf = self
                            .cx
                            .field_child(member, "suffix")
                            .and_then(|s| self.cx.field_child(s, "word"));
                        let suffix = suffix_leaf.map(|w| self.cx.spelling(w)).unwrap_or_default();
                        kind = match suffix.as_str() {
                            "portrait" => "view-portrait".to_string(),
                            "widget" => "view-widget".to_string(),
                            _ => {
                                let span =
                                    suffix_leaf.map(|w| self.cx.span_of(w)).unwrap_or(view_span);
                                let msg = format!(
                                    "a view exit must be `view-2d`, `view-3d`, \
                                     `view-portrait` or `view-widget`; `view-{suffix}` \
                                     is not an AGE export"
                                );
                                self.cx.reject(span, move || {
                                    Diagnostic::new(
                                        Stage::Text,
                                        "source.age.view-suffix",
                                        msg.clone(),
                                        span,
                                    )
                                });
                                "view".to_string()
                            }
                        };
                    }
                    // B-LANGUAGE-I（AGE-L03）：不同键的直属出口可以并存；**同一键**
                    // 写两次仍拒绝（"重复直属选择键拒绝"）。出口值由后缀字节落定。
                    if exits.iter().any(|e| e.kind == kind) {
                        let msg = format!(
                            "the `{}` exit is declared twice; distinct exits (view / view-2d / \
                             view-3d / view-portrait / view-widget) may coexist, but the same \
                             exit key may appear only once",
                            kind
                        );
                        self.cx.reject(view_span, move || {
                            Diagnostic::new(
                                Stage::Text,
                                "source.age.view-exit",
                                msg.clone(),
                                view_span,
                            )
                        });
                    } else {
                        if exits.is_empty() {
                            view_exit = kind.clone();
                        }
                        exits.push(crate::source::age_component::ComponentExit {
                            kind: kind.clone(),
                            span: view_span,
                        });
                    }
                    while text_exits.len() < texts.len() {
                        text_exits.push(Some(kind.clone()));
                    }
                    for raw in self.cx.children(member) {
                        // `age-view-item` 是分派 wrapper：实质成员是它的首个非 trivia 孩子。
                        let c = self
                            .cx
                            .children(raw)
                            .into_iter()
                            .find(|&x| {
                                self.cx.tree.node(x).flags & ace_syntax_runtime::cst::FLAG_TRIVIA
                                    == 0
                            })
                            .unwrap_or(raw);
                        // PLAN-981（IF-981-REF）：`Meter m1 {}` = 节点实例引用。
                        if let Some(ref_kind) = self
                            .cx
                            .p
                            .rules
                            .iter()
                            .position(|r| r.name == "age-node-ref")
                            .map(|i| ace_syntax_runtime::program::RULE_KIND_BASE + i as u32)
                        {
                            if self.cx.tree.kind(c) == ref_kind {
                                let ty_leaf = self.cx.field_child(c, "ty").expect("node ref type");
                                let name_leaf =
                                    self.cx.field_child(c, "name").expect("node ref name");
                                let ty = ident_of(&mut self.cx, ty_leaf);
                                let n = ident_of(&mut self.cx, name_leaf);
                                node_refs.push(crate::source::age_component::ComponentNodeRef {
                                    ty,
                                    name: n,
                                    span: self.cx.span_of(c),
                                });
                                continue;
                            }
                        }
                    }
                    tree = self.collect_view_items(member, &mut texts, &text_role);
                    // R60-8：文本在收集之后才有叶，归属按本出口补齐（含 `view` 本身）。
                    while text_exits.len() < texts.len() {
                        text_exits.push(Some(kind.clone()));
                    }
                }
                "age-msg-member" => {
                    self.collect_msg_variants(member, &mut messages);
                    // PLAN-980 WEB #6i：可选的具名段（`msg Msg { … }`）。
                    if let Some(name) = self.cx.field_child(member, "msg_name") {
                        let spelling = self.cx.spelling(name);
                        let span = self.cx.span_of(name);
                        match message_set_name.clone() {
                            Some(prev) => {
                                // R65-4：第二个具名集合不能静默覆盖第一个。
                                let msg = format!(
                                    "a second named message set `{spelling}` would hide the \
                                     earlier set `{prev}`; declare one named set per node"
                                );
                                self.cx.reject(span, move || {
                                    Diagnostic::new(
                                        Stage::Text,
                                        "source.age.msg-set-duplicate",
                                        msg.clone(),
                                        span,
                                    )
                                });
                            }
                            None => message_set_name = Some(spelling),
                        }
                    }
                }
                "age-design-member" => {
                    self.collect_design_entries(member, &mut design);
                }
                "age-on-widget-member" => {
                    // PLAN-980 WEB #6p：通道判定需要头参数名（`on_select` ∈
                    // `widget Counter(on_select: msg)` 头表）——头参数在成员
                    // 循环前已投影，按名传入。
                    let channel_names: Vec<String> =
                        params.iter().map(|p| p.name.clone()).collect();
                    self.collect_widget_handlers(member, &mut handlers, &channel_names);
                }
                "age-named-view-member" => {
                    let face = self
                        .cx
                        .field_child(member, "face")
                        .expect("named view face name");
                    let face_name = self.cx.spelling(face);
                    // R60-2：`view <name> { … }` 的**体**此前被整个丢掉（不投影、
                    // 不进 verify）。这里按 `view` 同一机制收集文本叶，role 带上
                    // 命名面（`named-view:<name>`），由前门生成隐藏函数交给唯一
                    // verify；结构树本切片暂不投影（具名边界见 §6.7）。
                    let named_role = format!("named-view:{face_name}");
                    let mut named_texts = Vec::new();
                    let named_tree = self.collect_view_items(member, &mut named_texts, &named_role);
                    texts.extend(named_texts);
                    named_trees.push((face_name.clone(), named_tree));
                    member_rows.push(ComponentMemberRow {
                        role: "named-view",
                        name: face_name,
                        value: None,
                        text: None,
                        num: None,
                        stmts: Vec::new(),
                        sources: Vec::new(),
                        source_spans: Vec::new(),
                        span: self.cx.span_of(member),
                    });
                }
                "age-bind-member" => {
                    self.collect_member_rows(member, "age-bind-key", "bind", &mut member_rows);
                }
                "age-routes-member" => {
                    self.collect_member_rows(member, "age-route-def", "route", &mut member_rows);
                }
                "age-expose-member" => {
                    self.collect_expose_names(member, &mut member_rows);
                }
                "age-lifecycle-member" => {
                    self.collect_member_rows(
                        member,
                        "age-lifecycle-method",
                        "lifecycle",
                        &mut member_rows,
                    );
                }
                "age-computed-member" => {
                    self.collect_member_rows(
                        member,
                        "age-computed-property",
                        "computed",
                        &mut member_rows,
                    );
                }
                "age-watch-member" => {
                    self.collect_member_rows(member, "age-watch-decl", "watch", &mut member_rows);
                }
                "age-timer-member" => {
                    self.collect_member_rows(member, "age-timer-entry", "timer", &mut member_rows);
                }
                "age-scoped-style-member" => {
                    let css = self.cx.field_child(member, "css").expect("style block");
                    let open = self.cx.field_child(css, "open").expect("style opening brace");
                    let close = self.cx.field_child(css, "close").expect("style closing brace");
                    let start = self.cx.span_of(open).end as u64;
                    let end = self.cx.span_of(close).start as u64;
                    member_rows.push(ComponentMemberRow {
                        role: "scoped-style",
                        name: String::new(),
                        value: None,
                        text: Some(self.cx.spelling_by_range(start, end)),
                        num: None,
                        stmts: Vec::new(),
                        sources: Vec::new(),
                        source_spans: Vec::new(),
                        span: self.cx.span_of(member),
                    });
                }
                _ => {}
            }
        }
        let inferred_records = if self.cx.profile == SourceProfile::AgePackage {
            if head_spelling != canonical_head {
                Self::admit_legacy_widget_handlers(&fields, &handlers, &mut messages);
                Self::admit_legacy_interval_tick(&fields, &handlers, &mut messages, &mut member_rows);
            }
            self.admit_typed_model_data(
                &name, &handlers, &mut view_data, view_data_nodes, &mut fields,
                widget_lists.map(|projection| &projection.model_fields),
            )
        } else {
            Vec::new()
        };
        self.reject_duplicate_variants(&messages);
        let mut component = ComponentNode {
            head_role,
            head_spelling: head_spelling.clone(),
            head_span,
            legacy_widget: head_spelling != canonical_head,
            public,
            package_imports,
            params,
            name,
            authored_name,
            fields,
            handlers,
            view: ComponentView {
                role: view_role,
                exit: view_exit,
                exits,
                text_exits,
                named_trees,
                span: view_span,
                texts,
                node_refs,
                scene_components,
                tree,
            },
            messages,
            message_set_name,
            design,
            members: member_rows,
            view_data,
            inferred_records,
            stores: Vec::new(),
            span: self.cx.span_of(node),
        };
        if self.cx.profile == SourceProfile::AgePackage {
            if let Some(projection) = widget_lists {
                for param in &mut component.params {
                    if let Some(ty) = projection.inputs.get(&param.ty_span.start) {
                        param.collection_type = Some(ty.clone());
                    }
                }
            }
            if let Some(projection) = widget_lists.filter(|projection| !projection.selectors.is_empty()) {
                super::widget_selectors::specialize(&mut component, &projection.selectors);
                for handler in &mut component.handlers {
                    handler.assigns.clear();
                    Self::flat_handler_assigns(&handler.stmts, &mut handler.assigns);
                }
            }
            if component.legacy_widget {
                super::legacy_record_lists::infer_built_lists(
                    &component.name, &mut component.fields, &component.params,
                    &mut component.handlers, &mut component.inferred_records,
                    widget_lists.map(|projection| &projection.record_fields),
                );
            }
        }
        component
    }

    /// Legacy handlers can introduce unit messages or payloads pinned by
    /// direct model assignments. Explicit signatures and timed Tick take precedence.
    fn admit_legacy_widget_handlers(
        fields: &[ComponentField],
        handlers: &[ComponentHandler],
        messages: &mut Vec<ComponentMsgVariant>,
    ) {
        for handler in handlers {
            if !handler.from_widget || handler.msg == "Tick"
                || messages.iter().any(|message| message.name == handler.msg)
            {
                continue;
            }
            let payload = if handler.params.is_empty() {
                Some(Vec::new())
            } else {
                Self::legacy_assignment_payload(fields, handler)
            };
            if let Some(payload) = payload {
                messages.push(ComponentMsgVariant {
                    name: handler.msg.clone(),
                    outbound: false,
                    payload,
                    span: handler.span,
                });
            }
        }
    }

    fn legacy_assignment_payload(fields: &[ComponentField], handler: &ComponentHandler) -> Option<Vec<String>> {
        use crate::source::age_component::{payload_type, ComponentStmt};
        if handler.body_foreign || !handler.channel_calls.is_empty()
            || !handler.stmts.iter().all(|statement| matches!(statement, ComponentStmt::Assign(_)))
        {
            return None;
        }
        handler.params.iter().map(|(parameter, _)| {
            let mut targets = handler.assigns.iter().filter_map(|assign| {
                if matches!(&assign.rhs.kind, ExprKind::Var(name) if name.name == *parameter) {
                    Some(fields.iter().find(|field| field.name == assign.field)?)
                } else {
                    None
                }
            });
            let first = targets.next()?;
            let expected = payload_type(&first.ty)?;
            targets.all(|field| payload_type(&field.ty) == Some(expected)).then(|| first.ty.clone())
        }).collect()
    }

    /// Legacy widgets use a positive `interval` model value to schedule a
    /// unit `.Tick` handler. Keep the resulting message and timer in the
    /// same description as explicitly authored declarations.
    fn admit_legacy_interval_tick(
        fields: &[ComponentField],
        handlers: &[ComponentHandler],
        messages: &mut Vec<ComponentMsgVariant>,
        members: &mut Vec<ComponentMemberRow>,
    ) {
        let Some(handler) = handlers.iter().find(|handler| {
            handler.from_widget && handler.msg == "Tick" && handler.params.is_empty()
        }) else { return };
        let Some(interval) = fields.iter().find(|field| {
            field.name == "interval" && matches!(field.ty.as_str(), "int" | "i32")
        }) else { return };
        let ExprKind::IntLit(ms) = interval.init.kind else { return };
        if ms <= 0 {
            return;
        }
        match messages.iter().find(|message| message.name == "Tick") {
            Some(message) if !message.payload.is_empty() => return,
            Some(_) => {}
            None => messages.push(ComponentMsgVariant {
                name: "Tick".into(), outbound: false, payload: Vec::new(), span: handler.span,
            }),
        }
        if !members.iter().any(|row| row.role == "timer" && row.name == "Tick") {
            let running = fields.iter().find_map(|field| {
                if field.name != "running" {
                    return None;
                }
                let span = field.span;
                let value = Expr {
                    kind: ExprKind::Field {
                        base: Box::new(Expr {
                            kind: ExprKind::Var(Ident { name: "self".into(), span }),
                            span,
                        }),
                        name: Ident { name: field.name.clone(), span },
                    },
                    span,
                };
                match field.ty.as_str() {
                    "str" | "Str" => Some(Expr {
                        kind: ExprKind::Binary {
                            op: BinOp::Eq,
                            lhs: Box::new(value),
                            rhs: Box::new(Expr { kind: ExprKind::StrLit("true".into()), span }),
                        },
                        span,
                    }),
                    "bool" => Some(value),
                    _ => None,
                }
            });
            members.push(ComponentMemberRow {
                role: "timer", name: "Tick".into(), num: Some(i64::from(ms)), value: running,
                text: None, stmts: Vec::new(), sources: Vec::new(), source_spans: Vec::new(),
                span: interval.span,
            });
        }
    }

    fn admit_typed_model_data(
        &mut self,
        node: &Ident,
        handlers: &[ComponentHandler],
        view_data: &mut Vec<crate::source::age_component::ComponentViewDataField>,
        initializers: Vec<u32>,
        fields: &mut Vec<ComponentField>,
        promoted_model_data: Option<&BTreeSet<usize>>,
    ) -> Vec<TypeItem> {
        use crate::source::age_component::ComponentStmt;
        let mut names = BTreeSet::new();
        let mut pending: Vec<_> = handlers.iter().map(|handler| handler.stmts.as_slice()).collect();
        while let Some(block) = pending.pop() {
            for stmt in block {
                match stmt {
                    ComponentStmt::For { iter, body, .. } => {
                        if let ExprKind::Field { base, name } = &iter.kind {
                            if matches!(&base.kind, ExprKind::Var(id) if id.name == "self") {
                                names.insert(name.name.as_str());
                            }
                        }
                        pending.push(body);
                    }
                    ComponentStmt::While { body, .. } => pending.push(body),
                    ComponentStmt::If { then, els, .. } => {
                        pending.push(then);
                        pending.extend(els.as_deref());
                    }
                    _ => {}
                }
            }
        }

        let mut records: Vec<TypeItem> = Vec::new();
        for (data, initial) in std::mem::take(view_data).into_iter().zip(initializers) {
            let widget_input = promoted_model_data.is_some_and(|spans| spans.contains(&data.span.start));
            let typed = names.contains(data.name.as_str()) || widget_input;
            let inferred = typed.then(|| self.project_iterated_record_list(initial, widget_input)).flatten();
            let Some((init, schema, sparse)) = inferred else {
                view_data.push(data);
                continue;
            };
            let span = data.span;
            let record = super::legacy_record_lists::intern_record(node, span, schema, &mut records, sparse);
            fields.push(ComponentField {
                name: data.name,
                ty: format!("List<{}>", record.name),
                ty_span: span,
                collection_type: Some(super::legacy_record_lists::list_of(record, span)),
                init,
                mutable: true,
                span,
            });
        }
        fields.sort_by_key(|field| field.span.start);
        records
    }

    /// Infer scalar fields from authored records. Bare widget inputs can also
    /// carry sparse text fields when the remaining record shape is known.
    fn project_iterated_record_list(&mut self, initial: u32, sparse_text: bool) -> Option<(Expr, Vec<FieldDecl>, bool)> {
        let entry_kind = self.rule_kind_of("age-view-data-entry")?;
        let inline_kind = self.rule_kind_of("age-view-data-lit-inline")?;
        let tail_kind = self.rule_kind_of("age-view-data-inline-tail")?;
        let record_kind = self.rule_kind_of("age-view-data-record")?;
        let field_kind = self.rule_kind_of("age-view-data-field")?;
        let list = self.unwrap_child(initial);
        let mut entries = Vec::new();
        if self.cx.tree.kind(list) == inline_kind {
            entries.extend(self.cx.field_child(list, "first"));
            entries.extend(self.cx.children(list).into_iter()
                .filter(|&node| self.cx.tree.kind(node) == tail_kind)
                .filter_map(|tail| self.cx.field_child(tail, "item")));
        } else {
            entries.extend(self.cx.children(list).into_iter()
                .filter(|&node| self.cx.tree.kind(node) == entry_kind));
        }
        let mut schema = BTreeMap::<String, FieldDecl>::new();
        let mut items = Vec::new();
        for entry in entries {
            let row = self.unwrap_child(entry);
            if self.cx.tree.kind(row) != record_kind {
                return None;
            }
            let mut columns = BTreeMap::new();
            let mut values = Vec::new();
            for field in self.cx.children(row) {
                if self.cx.tree.kind(field) != field_kind {
                    continue;
                }
                let name_node = self.cx.field_child(field, "key")?;
                let name = ident_of(&mut self.cx, name_node);
                let span = self.cx.span_of(field);
                let expr = self.unwrap_child(self.cx.field_child(field, "value")?);
                let value = self.project_expr_sync(expr);
                let word = match &value.kind {
                    ExprKind::IntLit(_) => TypeWord::Int,
                    ExprKind::BoolLit(_) => TypeWord::Bool,
                    ExprKind::StrLit(_) => TypeWord::Str,
                    ExprKind::TypedLit { ty, .. } => *ty,
                    ExprKind::Unary { op: UnaryOp::Neg, operand }
                        if matches!(operand.kind, ExprKind::IntLit(_)) => TypeWord::Int,
                    _ => return None,
                };
                let column = FieldDecl { name: name.clone(), ty: TypeAnn {
                    kind: TypeRef::Builtin(word), span: value.span,
                }, span };
                if columns.insert(name.name.clone(), column).is_some() {
                    return None;
                }
                values.push(ObjectField { key: crate::hir::ObjectKey::Text(name.name), value, span });
            }
            if columns.is_empty() {
                return None;
            }
            for (name, column) in columns {
                match schema.entry(name) {
                    std::collections::btree_map::Entry::Vacant(entry) => {
                        entry.insert(column);
                    }
                    std::collections::btree_map::Entry::Occupied(entry) => {
                        if entry.get().ty.kind != column.ty.kind {
                            return None;
                        }
                    }
                }
            }
            items.push(Expr { kind: ExprKind::ObjectLit { fields: values }, span: self.cx.span_of(row) });
        }
        if items.is_empty() {
            return None;
        }
        let mut sparse = false;
        for row in &mut items {
            let ExprKind::ObjectLit { fields } = &mut row.kind else { unreachable!() };
            if fields.len() == schema.len() {
                continue;
            }
            if !sparse_text {
                return None;
            }
            sparse = true;
            let present: BTreeSet<_> = fields.iter().filter_map(|field| match &field.key {
                crate::hir::ObjectKey::Text(key) => Some(key.clone()),
                _ => None,
            }).collect();
            for (key, column) in &schema {
                if present.contains(key) {
                    continue;
                }
                if column.ty.kind != TypeRef::Builtin(TypeWord::Str) {
                    return None;
                }
                fields.push(ObjectField {
                    key: crate::hir::ObjectKey::Text(key.clone()),
                    value: Expr { kind: ExprKind::StrLit(String::new()), span: row.span },
                    span: row.span,
                });
            }
        }
        Some((Expr { kind: ExprKind::ArrayLit(items), span: self.cx.span_of(initial) }, schema.into_values().collect(), sparse))
    }

    /// PLAN-980：规则名 → kind（本裁剪面内）。
    fn rule_kind_of(&self, name: &str) -> Option<u32> {
        self.cx
            .p
            .rules
            .iter()
            .position(|r| r.name == name)
            .map(|i| ace_syntax_runtime::program::RULE_KIND_BASE + i as u32)
    }

    /// 子节点里按规则名找第一个非 trivia 实质子节点（alt/包装解包）。
    fn unwrap_child(&self, node: u32) -> u32 {
        self.cx
            .children(node)
            .into_iter()
            .find(|&c| self.cx.tree.node(c).flags & FLAG_TRIVIA == 0)
            .unwrap_or(node)
    }

    fn project_route_link(
        &mut self,
        node: u32,
        texts: &mut Vec<crate::source::age_h0::AgeTextLeaf>,
        text_role: &str,
    ) -> ComponentViewElement {
        let tag = self.cx.field_child(node, "tag").expect("route link tag");
        let destination = self.cx.field_child(node, "to").expect("route link destination");
        let span = self.cx.span_of(destination);
        let mut items = vec![ComponentViewItem::Entry(ComponentViewEntry {
            key: "to".into(),
            is_event: false,
            value: ComponentEntryValue::Expr(self.project_expr_sync(destination)),
            span,
        })];
        let extra_kind = self.rule_kind_of("age-route-link-extra");
        for child in self.cx.children(node) {
            if Some(self.cx.tree.kind(child)) == extra_kind {
                items.extend(self.collect_view_items(child, texts, text_role));
            }
        }
        items.extend(self.collect_view_items(node, texts, text_role));
        ComponentViewElement {
            tag: self.cx.spelling(tag),
            label: None,
            items,
            span: self.cx.span_of(node),
        }
    }

    /// Project the component view tree and collect its text leaves in tree order.
    fn project_class_list(&mut self, list: u32) -> Vec<ComponentClassPart> {
        let prefix = self.rule_kind_of("age-class-list-prefix");
        let tail = self.rule_kind_of("age-class-list-tail");
        let conditional = self.rule_kind_of("age-class-if");
        let mut parts = Vec::new();
        for child in self.cx.children(list) {
            let kind = Some(self.cx.tree.kind(child));
            let item = if kind == prefix {
                self.cx.field_child(child, "value").expect("class value")
            } else if kind == tail {
                self.unwrap_child(self.cx.field_child(child, "item").expect("class item"))
            } else if kind == conditional {
                child
            } else {
                continue;
            };
            if Some(self.cx.tree.kind(item)) == conditional {
                let cond = self.cx.field_child(item, "cond").expect("class condition");
                let then = self.cx.field_child(item, "then").expect("true class branch");
                let otherwise = self.cx.field_child(item, "else").expect("false class branch");
                let then_value = self.cx.field_child(then, "value").expect("true class value");
                let else_value = self.cx.field_child(otherwise, "value").expect("false class value");
                parts.push(ComponentClassPart::If {
                    cond: self.project_expr_sync(self.unwrap_child(cond)),
                    then_value: self.project_expr_sync(then_value),
                    else_value: self.project_expr_sync(else_value),
                });
            } else {
                parts.push(ComponentClassPart::Value(self.project_expr_sync(item)));
            }
        }
        parts
    }

    fn collect_view_items(
        &mut self,
        list_node: u32,
        texts: &mut Vec<crate::source::age_h0::AgeTextLeaf>,
        text_role: &str,
    ) -> Vec<ComponentViewItem> {
        let text_item = self.rule_kind_of("age-text-item");
        let text_bare = self.rule_kind_of("age-text-bare");
        let text_props = self.rule_kind_of("age-text-props");
        let text_literal_props = self.rule_kind_of("age-text-literal-props");
        let entry_kind = self.rule_kind_of("age-view-entry");
        let element_kind = self.rule_kind_of("age-view-element");
        let cond_kind = self.rule_kind_of("age-view-if");
        let cond_block_kind = self.rule_kind_of("age-view-if-then");
        let cond_else_kind = self.rule_kind_of("age-view-else");
        // PLAN-1072 刀B：view 级循环项 kind。
        let for_kind = self.rule_kind_of("age-view-for");
        let _ = (cond_block_kind, cond_else_kind);
        let mut out = Vec::new();
        for raw in self.cx.children(list_node) {
            let raw_kind = self.cx.tree.kind(raw);
            let raw_is_item = Some(raw_kind) == text_item
                || Some(raw_kind) == text_bare
                || Some(raw_kind) == text_props
                || Some(raw_kind) == text_literal_props
                || Some(raw_kind) == entry_kind
                || Some(raw_kind) == element_kind
                || Some(raw_kind) == cond_kind
                || Some(raw_kind) == for_kind;
            // PLAN-980 B-AV U01（#6m 投影缺口根因）：先按自身 kind 匹配
            // ——props 段条目（age-view-entry）是具体规则、不是 age-view-item
            // 包装，unwrap 会错误下钻到其首个孩子（age-event-key）后整条
            // 丢失；包装（age-view-item）才需要解一层。
            let item = if raw_is_item {
                raw
            } else {
                self.unwrap_child(raw)
            };
            let kind = self.cx.tree.kind(item);
            if Some(kind) == text_item || Some(kind) == text_bare {
                let comp = self
                    .cx
                    .field_child(item, "component")
                    .expect("text head leaf");
                let expr_node = self.cx.field_child(item, "text").expect("text expression");
                let expr = self.project_expr_sync(expr_node);
                let leaf = crate::source::age_h0::AgeTextLeaf {
                    role: text_role.to_string(),
                    spelling: self.cx.spelling(comp),
                    component_span: self.cx.span_of(comp),
                    expr,
                    span: self.cx.span_of(item),
                };
                texts.push(leaf.clone());
                out.push(ComponentViewItem::Text(leaf));
            } else if Some(kind) == text_props || Some(kind) == text_literal_props {
                // 带属性体的文本（001 形）：tag=text 的元素 + label 表达式同时
                // 入 texts（绑定验证通道与其它文本叶一致）。
                let comp = self
                    .cx
                    .field_child(item, "component")
                    .expect("text head leaf");
                let expr_node = self.cx.field_child(item, "text").expect("text expression");
                let label = if Some(kind) == text_literal_props {
                    let raw = self.cx.spelling(expr_node);
                    Expr {
                        kind: ExprKind::StrLit(lex::decode_str_value(&raw.as_bytes()[1..raw.len() - 1])),
                        span: self.cx.span_of(expr_node),
                    }
                } else {
                    self.project_expr_sync(expr_node)
                };
                texts.push(crate::source::age_h0::AgeTextLeaf {
                    role: text_role.to_string(),
                    spelling: self.cx.spelling(comp),
                    component_span: self.cx.span_of(comp),
                    expr: label.clone(),
                    span: self.cx.span_of(item),
                });
                let items = self.collect_view_items(item, texts, text_role);
                out.push(ComponentViewItem::Element(ComponentViewElement {
                    tag: "text".to_string(),
                    label: Some(label),
                    items,
                    span: self.cx.span_of(item),
                }));
            } else if Some(kind) == entry_kind {
                // PLAN-980 WEB #6m：事件键=age-event-key（ident 基名+点修饰
                // 链，`onclick.self`）——拼写=基名+`.`+修饰符（is_event 按
                // 前缀判不受影响）。
                let key_node = self.cx.field_child(item, "key").expect("view entry key");
                let key_name = if let Some(quoted) = self.cx.field_child(key_node, "quoted") {
                    let spelling = self.cx.spelling(quoted);
                    let name = &spelling[1..spelling.len() - 1];
                    let valid = name.split('-').all(|segment| {
                        let mut bytes = segment.bytes();
                        bytes
                            .next()
                            .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
                            && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
                    });
                    let event = name.get(..2).is_some_and(|head| head.eq_ignore_ascii_case("on"));
                    if !valid || event {
                        let span = self.cx.span_of(quoted);
                        self.cx.reject(span, || {
                            Diagnostic::new(
                                Stage::Text,
                                "source.age.view-key",
                                "expected a quoted non-event property name made of hyphen-separated identifiers",
                                span,
                            )
                        });
                        continue;
                    }
                    name.to_owned()
                } else {
                    self.cx.spelling(key_node)
                };
                let value_node = self
                    .cx
                    .field_child(item, "value")
                    .expect("view entry value");
                let inner = self.unwrap_child(value_node);
                let inner_kind = self.cx.tree.kind(inner);
                let lambda_kind = self.rule_kind_of("age-inline-lambda");
                let event_binding_kind = self.rule_kind_of("age-event-binding");
                let child_event_kind = self.rule_kind_of("age-child-event-binding");
                let if_expr_kind = self.rule_kind_of("age-if-expr");
                let if_else_kind = self.rule_kind_of("age-if-else");
                let class_list_kind = self.rule_kind_of("age-conditional-class-list");
                let is_event = key_name.starts_with("on");
                let value = if Some(inner_kind) == class_list_kind {
                    ComponentEntryValue::ClassList(self.project_class_list(inner))
                } else if Some(inner_kind) == if_expr_kind {
                    // PLAN-1072 刀B：条件样式值（design 三段同形结构承载）。
                    let cond = self
                        .cx
                        .field_child(inner, "cond")
                        .map(|w| self.project_expr_sync(self.unwrap_child(w)))
                        .expect("if-expr cond");
                    let then_value = self
                        .cx
                        .field_child(inner, "then")
                        .and_then(|w| self.cx.field_child(w, "value"))
                        .map(|v| self.project_expr_sync(v))
                        .expect("if-expr then");
                    let else_value = self
                        .cx
                        .children(inner)
                        .into_iter()
                        .find(|&w| Some(self.cx.tree.kind(w)) == if_else_kind)
                        .and_then(|w| self.cx.field_child(w, "value"))
                        .map(|v| self.project_expr_sync(v))
                        .expect("if-expr else");
                    ComponentEntryValue::If {
                        cond,
                        then_value,
                        else_value,
                    }
                } else if Some(inner_kind) == lambda_kind {
                    // PLAN-980 lambda-if 刀：体投影接共享结构化收集
                    //（project_view_stmt_children——if 形进
                    // ComponentStmt::If 序保持；调用形置 foreign fail-closed
                    // 不铸，#6p 合同保持：lambda 面无通道参数，通道调用承载
                    // =后续切片）；assigns 深平铺兼容面（既有消费者默认路径
                    // 不变）。
                    let mut foreign = false;
                    let stmts = self.project_view_stmt_children(inner, &[], &mut foreign);
                    // 增量同步适配（F-R21-3 后续）：lambda-if 落地时调用的
                    // 单句版 flatten_stmt_assigns 已在 981 并集删除（语义被
                    // 切片版包含）——此处等价改道 flat_handler_assigns
                    //（逐句循环 == 整切片收平，源序/深度不变）。
                    let mut assigns = Vec::new();
                    Self::flat_handler_assigns(&stmts, &mut assigns);
                    ComponentEntryValue::Lambda {
                        assigns,
                        stmts,
                        foreign,
                    }
                } else if Some(inner_kind) == child_event_kind {
                    let msg = self.cx.field_child(inner, "msg").expect("child event target");
                    ComponentEntryValue::ChildEvent(Ident {
                        name: self.cx.spelling(msg),
                        span: self.cx.span_of(msg),
                    })
                } else if Some(inner_kind) == event_binding_kind {
                    // PLAN-980 WEB #6m：事件绑定调用形 `.Msg($event, …)`——
                    // 投影为**裸 `.Msg` 字段引用**（既有 event_binding_msg 识
                    // 别/跳过通道）；实参是宿主（DOM 事件）通道，本面不验证
                    // 不求值（模板发射由方言侧方言 AST 承担，此投影无消费
                    // 方——丢弃无损失，具名边界）。
                    let msg = self
                        .cx
                        .field_child(inner, "msg")
                        .expect("event binding msg ident");
                    let msg_span = self.cx.span_of(msg);
                    ComponentEntryValue::Expr(Expr {
                        span: msg_span,
                        kind: ExprKind::Field {
                            base: Box::new(Expr {
                                span: msg_span,
                                kind: ExprKind::Var(Ident {
                                    name: "self".to_string(),
                                    span: msg_span,
                                }),
                            }),
                            name: Ident {
                                name: self.cx.spelling(msg),
                                span: msg_span,
                            },
                        },
                    })
                } else {
                    let mut expr = self.project_expr_sync(inner);
                    // The ordinary value projector spells `.Msg(...)` as a
                    // free call. Keep the receiver in an event entry so it
                    // cannot be confused with `fn Msg(...)`.
                    let full_span = self.cx.span_of(inner);
                    if is_event && self.cx.age_face && self.cx.spelling(inner).starts_with('.') {
                        if let ExprKind::Call { callee, args } = &expr.kind {
                            let dot = Span { start: full_span.start, end: full_span.start + 1 };
                            expr = Expr {
                                span: full_span,
                                kind: ExprKind::MethodCall {
                                    recv: Box::new(Expr {
                                        span: dot,
                                        kind: ExprKind::Var(Ident { name: "self".into(), span: dot }),
                                    }),
                                    name: callee.clone(),
                                    args: args.clone(),
                                    instance: None,
                                },
                            };
                        }
                    }
                    ComponentEntryValue::Expr(expr)
                };
                out.push(ComponentViewItem::Entry(ComponentViewEntry {
                    is_event,
                    key: key_name,
                    value,
                    span: self.cx.span_of(item),
                }));
            } else if Some(kind) == element_kind {
                // 元素具体形（labeled-block/block/labeled）在子节点上——解包后取字段。
                let variant = self.unwrap_child(item);
                if Some(self.cx.tree.kind(variant)) == self.rule_kind_of("age-route-link-block") {
                    let link = self.project_route_link(variant, texts, text_role);
                    out.push(ComponentViewItem::Element(link));
                    continue;
                }
                let tag = self
                    .cx
                    .field_child(variant, "tag")
                    .expect("element tag ident");
                let mut label = self
                    .cx
                    .field_child(variant, "label")
                    .map(|l| self.project_expr_sync(l));
                let mut items = self.collect_view_items(variant, texts, text_role);
                let tag_name = self.cx.spelling(tag);
                if self.cx.profile == SourceProfile::AgePackage && tag_name == "text" && label.is_none() {
                    let text_entry = items.iter().position(|item| matches!(item,
                        ComponentViewItem::Entry(entry)
                            if entry.key == "text" && matches!(entry.value, ComponentEntryValue::Expr(_))));
                    if let Some(index) = text_entry {
                        let ComponentViewItem::Entry(ComponentViewEntry {
                            value: ComponentEntryValue::Expr(expr), ..
                        }) = items.remove(index) else {
                            unreachable!("selected a text expression")
                        };
                        texts.push(crate::source::age_h0::AgeTextLeaf {
                            role: text_role.to_string(),
                            spelling: tag_name.clone(),
                            component_span: self.cx.span_of(tag),
                            expr: expr.clone(),
                            span: self.cx.span_of(item),
                        });
                        label = Some(expr);
                    }
                }
                out.push(ComponentViewItem::Element(ComponentViewElement {
                    tag: tag_name,
                    label,
                    items,
                    span: self.cx.span_of(item),
                }));
            } else if Some(kind) == for_kind {
                // PLAN-1072 刀B：view 级循环项（结构承载惰性语义）。体文本叶
                // **不入 texts**——循环变量不在受检单位绑定域（入 hidden fn
                // 即 verify 红）；丢弃 sink 只取结构。
                let index = self
                    .cx
                    .field_child(item, "index")
                    .and_then(|binding| self.cx.field_child(binding, "name"))
                    .map(|name| self.cx.spelling(name));
                let value_leaf = self.cx.field_child(item, "value").expect("for value");
                let iter_node = self.cx.field_child(item, "iter").expect("for iter");
                let iter = self.project_expr_sync(iter_node);
                let mut sink = Vec::new();
                let items = self.collect_view_items(item, &mut sink, text_role);
                out.push(ComponentViewItem::For(
                    crate::source::age_component::ComponentViewFor {
                        index,
                        value: self.cx.spelling(value_leaf),
                        iter,
                        items,
                        span: self.cx.span_of(item),
                    },
                ));
            } else if Some(kind) == cond_kind {
                // PLAN-980 view-if 刀：view 级条件项（结构承载；文本叶随
                // collect 平铺；else-if 链=else_items 内单一嵌套 Cond）。
                out.push(self.project_view_if_cond(item, texts, text_role));
            } else if !raw_is_item
                && self
                    .cx
                    .children(raw)
                    .iter()
                    .any(|&c| self.cx.tree.node(c).flags & FLAG_TRIVIA == 0)
            {
                // PLAN-980 B-AV U01（#6m 投影缺口闭合）：非 item 形的结构包装
                // 子节点（`age-element-props-block` 的 list-nl 括号段/
                // `age-element-props-body` 体块——unwrapper 落到 lbrace 叶）
                // 此前被静默跳过——括号属性（value/oninput）与体条目全部丢
                // 失。此处对包装 raw 递归下钻恢复（本函数只在 view 体/元素
                // 体语境调用，包装层不含其它语义）。
                let inner = self.collect_view_items(raw, texts, text_role);
                out.extend(inner);
            }
        }
        out
    }

    fn project_age_format_string(&self, node: u32) -> Result<Expr, Diagnostic> {
        use crate::source::fstr::{split_fstring, Part};

        let span = self.cx.span_of(node);
        let prefix = self.cx.field_child(node, "prefix").expect("format prefix");
        let quoted = self.cx.field_child(node, "quoted").expect("format quoted text");
        let prefix_span = self.cx.span_of(prefix);
        if prefix_span.end != self.cx.span_of(quoted).start {
            return Err(format_diag("expected a quote immediately after `f`", span));
        }
        let raw = self.cx.spelling(node);
        let mut args = Vec::new();
        for part in split_fstring(&raw).map_err(|message| format_diag(message, span))? {
            let value = match part {
                Part::Text(text, range) => Expr {
                    kind: ExprKind::StrLit(lex::decode_str_value(text.as_bytes())),
                    span: Span { start: span.start + range.start, end: span.start + range.end },
                },
                Part::Expr(range) => {
                    let offset = span.start + range.start;
                    self.project_age_interpolation(&raw[range], offset)?
                }
            };
            args.push(Arg::Pos(value));
        }
        Ok(Expr {
            kind: ExprKind::Call {
                callee: Ident { name: "__fstr".into(), span: prefix_span },
                args,
            },
            span,
        })
    }

    fn project_age_interpolation(&self, src: &str, offset: usize) -> Result<Expr, Diagnostic> {
        use ace_syntax_runtime::parse::parse_snapshot_with_ctx;
        use ace_syntax_runtime::program::{Budget, RULE_KIND_BASE};
        use ace_syntax_runtime::snapshot::PageStore;

        // Interpolation uses the same AGE rules and projector as authored
        // expressions. A different start symbol confines parsing to one value.
        static EXPRESSION_PROGRAM: std::sync::OnceLock<GrammarProgram> = std::sync::OnceLock::new();
        let program = EXPRESSION_PROGRAM.get_or_init(|| {
            let mut program = self.cx.p.clone();
            program.start = self.cx.k.expr - RULE_KIND_BASE;
            program.outline = None;
            program
        });
        let span = Span {
            start: offset,
            end: offset + src.len(),
        };
        let grammar = super::frontdoor::profile_grammar(SourceProfile::AgePackage)
            .map_err(|message| format_diag(message, span))?;
        let pages = std::rc::Rc::new(PageStore::new());
        let snap = Snapshot::from_text(&pages, src);
        let ctx_text = super::frontdoor::SnapshotTokenText { snap: &snap };
        let parsed = parse_snapshot_with_ctx(
            program,
            grammar.scanner(),
            &snap,
            &Budget::default(),
            Some(&ctx_text),
        )
        .map_err(|error| format_diag(error.to_string(), span))?;
        if parsed.tree.has_error_nodes() {
            let message = parsed
                .tree
                .diags
                .first()
                .map(|diag| diag.message.as_str())
                .unwrap_or("expected one interpolation expression");
            return Err(format_diag(message, span));
        }
        let mut eng = Engine::new(
            program,
            self.cx.k,
            &parsed.tree,
            &snap,
            self.cx.profile,
            false,
            false,
            true,
        );
        let mut value = eng.project_expr_sync(parsed.tree.root);
        if let Some(mut diagnostic) = eng.cx.first_diag() {
            super::shift::shift_diag(&mut diagnostic, offset as i64);
            return Err(diagnostic);
        }
        super::shift::shift_expr(&mut value, offset as i64);
        Ok(value)
    }

    /// PLAN-980 view-if 刀：`age-view-if` 节点 → `ComponentViewItem::Cond`
    /// （结构承载；else-if 链展开为 else_items 内的单一嵌套 Cond）。
    fn project_view_if_cond(
        &mut self,
        item: u32,
        texts: &mut Vec<crate::source::age_h0::AgeTextLeaf>,
        text_role: &str,
    ) -> ComponentViewItem {
        let cond_node = self.cx.field_child(item, "cond").expect("view-if cond");
        let cond = self.project_expr_sync(cond_node);
        let then_node = self
            .cx
            .field_child(item, "then")
            .expect("view-if then body");
        let then_items = self.collect_view_items(then_node, texts, text_role);
        let mut else_items = Vec::new();
        if let Some(else_node) = self.cx.field_child(item, "else") {
            // age-view-else 的 alt 臂孩子=else-kw + 具体规则（kw 先于规则）——
            // 按 kind 定位具体臂（unwrap 会落在 kw token 上）。
            let if_kind = self.rule_kind_of("age-view-if");
            let block_kind = self.rule_kind_of("age-view-if-then");
            let mut arm = None;
            for ch in self.cx.children(else_node) {
                let k = self.cx.tree.kind(ch);
                if Some(k) == if_kind || Some(k) == block_kind {
                    arm = Some(ch);
                    break;
                }
            }
            match arm {
                Some(ch) if Some(self.cx.tree.kind(ch)) == if_kind => {
                    // else-if 链：单一嵌套条件项（递归）。
                    else_items.push(self.project_view_if_cond(ch, texts, text_role));
                }
                Some(ch) => {
                    // else 块（age-view-if-then 形）。
                    else_items = self.collect_view_items(ch, texts, text_role);
                }
                None => {}
            }
        }
        ComponentViewItem::Cond(crate::source::age_component::ComponentViewCond {
            cond,
            then_items,
            else_items,
            span: self.cx.span_of(item),
        })
    }

    /// PLAN-981（G2 全深度收平；on-if 组件面刀提为两面共用）：语句面 →
    /// 赋值兼容面（含 if/else 分支里的赋值，源序）。执行语义以 `stmts` 为
    /// 准；assign-only 消费者见 Let/If 必须具名拒（stmts 契约）。
    /// 并集注记（F-R21-3）：980 on-if 刀的单句版 `flatten_stmt_assigns`
    /// 语义被本切片版包含（其唯一调用点=widget 循环，并集后改走本
    /// helper），按并集政策删除、全仓保单一份。
    fn flat_handler_assigns(
        stmts: &[crate::source::age_component::ComponentStmt],
        out: &mut Vec<crate::source::age_component::ComponentAssign>,
    ) {
        for st in stmts {
            match st {
                crate::source::age_component::ComponentStmt::Assign(a) => out.push(a.clone()),
                crate::source::age_component::ComponentStmt::If { then, els, .. } => {
                    Self::flat_handler_assigns(then, out);
                    if let Some(e) = els {
                        Self::flat_handler_assigns(e, out);
                    }
                }
                crate::source::age_component::ComponentStmt::For { body, .. }
                | crate::source::age_component::ComponentStmt::While { body, .. } => {
                    Self::flat_handler_assigns(body, out);
                }
                crate::source::age_component::ComponentStmt::Let { .. }
                | crate::source::age_component::ComponentStmt::AssignLocal { .. }
                | crate::source::age_component::ComponentStmt::Call { .. }
                | crate::source::age_component::ComponentStmt::Effect { .. }
                | crate::source::age_component::ComponentStmt::Send { .. } => {}
            }
        }
    }

    /// Resolve bare targets in lexical order, including widget host commands
    /// whose receiver is outside the authored handler's bindings.
    fn resolve_handler_scope(
        stmts: &mut Vec<crate::source::age_component::ComponentStmt>,
        locals: &mut BTreeSet<String>,
        widget: bool,
    ) -> bool {
        use crate::source::age_component::ComponentStmt;
        let mut host_command = false;
        stmts.retain_mut(|stmt| {
            match stmt {
                ComponentStmt::Let { name, .. } => { locals.insert(name.name.clone()); }
                ComponentStmt::AssignLocal { name, value, span } if !locals.contains(&name.name) => {
                    *stmt = ComponentStmt::Assign(ComponentAssign {
                        field: name.name.clone(),
                        bare: true,
                        rhs: value.clone(),
                        span: *span,
                    });
                }
                ComponentStmt::If { then, els, .. } => {
                    host_command |= Self::resolve_handler_scope(then, &mut locals.clone(), widget);
                    if let Some(els) = els {
                        host_command |= Self::resolve_handler_scope(els, &mut locals.clone(), widget);
                    }
                }
                ComponentStmt::For { name, body, .. } => {
                    let mut scoped = locals.clone();
                    scoped.insert(name.name.clone());
                    host_command |= Self::resolve_handler_scope(body, &mut scoped, widget);
                }
                ComponentStmt::While { body, .. } => {
                    host_command |= Self::resolve_handler_scope(body, &mut locals.clone(), widget);
                }
                ComponentStmt::Effect { value, .. } if widget => {
                    if let ExprKind::Method { recv, name, .. } = &value.kind {
                        if name.name == "push" && matches!(&recv.kind,
                            ExprKind::Var(id) if id.name == "router" && !locals.contains(&id.name))
                        {
                            host_command = true;
                            return false;
                        }
                    }
                }
                _ => {}
            }
            true
        });
        host_command
    }

    /// `.field (=|+=|-=) expr` 语句 → 赋值行（复合赋值脱糖为二元）。
    fn collect_view_stmt(&mut self, stmt: u32, assigns: &mut Vec<ComponentAssign>) {
        let field = self
            .cx
            .field_child(stmt, "field")
            .expect("stmt field ident");
        let rhs_node = self.cx.field_child(stmt, "rhs").expect("stmt rhs");
        let mut rhs = self.project_expr_sync(rhs_node);
        let field_name = self.cx.spelling(field);
        let op_node = self.cx.field_child(stmt, "op").expect("stmt assign op");
        let op_inner = self.unwrap_child(op_node);
        let op_rule = self.rule_name_of_kind(self.cx.tree.kind(op_inner));
        let span = self.cx.span_of(stmt);
        let has_dot = self.cx.field_child(stmt, "dot").is_some();
        if op_rule == "age-op-plus-assign" || op_rule == "age-op-minus-assign" {
            let op = if op_rule == "age-op-plus-assign" {
                crate::source::ast::BinOp::Add
            } else {
                crate::source::ast::BinOp::Sub
            };
            // 裸名复合赋值（无点）：读侧是裸名 `Var`（执行期按绑定表解析为
            // 局部量或字段）；点前缀形读侧是 `self.<field>`。
            let lhs = if has_dot {
                Expr {
                    span,
                    kind: ExprKind::Field {
                        base: Box::new(Expr {
                            span,
                            kind: ExprKind::Var(Ident {
                                name: "self".to_string(),
                                span,
                            }),
                        }),
                        name: Ident {
                            name: field_name.clone(),
                            span,
                        },
                    },
                }
            } else {
                Expr {
                    span,
                    kind: ExprKind::Var(Ident {
                        name: field_name.clone(),
                        span,
                    }),
                }
            };
            rhs = Expr {
                span,
                kind: ExprKind::Binary {
                    op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                },
            };
        }
        // PLAN-105 T-02：点号缺省即裸名赋值（局部遮蔽候选，语义同
        // `age-on-assign`）；显式 `.field` 恒指实例字段。
        let bare = self.cx.field_child(stmt, "dot").is_none();
        assigns.push(ComponentAssign {
            field: field_name,
            bare,
            rhs,
            span,
        });
    }

    fn rule_name_of_kind(&self, kind: u32) -> String {
        self.cx
            .p
            .rules
            .iter()
            .enumerate()
            .find(|(i, _)| ace_syntax_runtime::program::RULE_KIND_BASE + *i as u32 == kind)
            .map(|(_, r)| r.name.clone())
            .unwrap_or_default()
    }

    /// msg 变体（含逗号 tail 包装内的递归变体）。
    fn collect_msg_variants(&mut self, member: u32, messages: &mut Vec<ComponentMsgVariant>) {
        self.collect_msg_variants_inner(member, messages);
    }

    /// R65-1：重复变体必须**跨 `msg` 块**也拒——块内去重会漏掉 `msg { Inc }` +
    /// `msg { Inc }`，并让 payload 的匹配命中错误变体。
    fn reject_duplicate_variants(&mut self, messages: &[ComponentMsgVariant]) {
        let mut seen: Vec<(String, Span)> = Vec::new();
        for v in messages {
            if let Some((name, first)) = seen.iter().find(|(n, _)| *n == v.name) {
                let msg = format!(
                    "duplicate message variant `{}` (first declared at byte {}); a duplicate \
                     would silently merge two handlers",
                    name, first.start
                );
                self.cx.reject(v.span, move || {
                    Diagnostic::new(Stage::Text, "source.age.msg-duplicate", msg.clone(), v.span)
                });
            } else {
                seen.push((v.name.clone(), v.span));
            }
        }
    }

    fn collect_msg_variants_inner(&mut self, member: u32, messages: &mut Vec<ComponentMsgVariant>) {
        let variant_kind = self.rule_kind_of("age-msg-variant").expect("variant rule");
        fn walk(
            eng: &mut Engine,
            node: u32,
            variant_kind: u32,
            out: &mut Vec<ComponentMsgVariant>,
        ) {
            for c in eng.cx.children(node) {
                if eng.cx.tree.kind(c) == variant_kind {
                    let name = eng.cx.field_child(c, "name").expect("variant name");
                    // B-LANGUAGE-I：payload 类型在 `age-msg-payload` 包装里（`age-msg-ty`），
                    // 旧写法（`age-model-ty` 直接挂在 variant 下）仍兼容。
                    let mut payload = Vec::new();
                    let wanted: Vec<u32> = ["age-msg-ty", "age-model-ty"]
                        .iter()
                        .filter_map(|n| eng.rule_kind_of(n))
                        .collect();
                    fn collect(eng: &mut Engine, node: u32, wanted: &[u32], out: &mut Vec<String>) {
                        for p in eng.cx.children(node) {
                            if wanted.contains(&eng.cx.tree.kind(p)) {
                                out.push(eng.cx.spelling(p));
                            } else {
                                collect(eng, p, wanted, out);
                            }
                        }
                    }
                    collect(eng, c, &wanted, &mut payload);
                    let spelling = eng.cx.spelling(name);
                    let quoted = spelling.strip_prefix('"').and_then(|s| s.strip_suffix('"'));
                    out.push(ComponentMsgVariant {
                        name: quoted
                            .map_or(spelling.clone(), |s| super::lex::decode_str_value(s.as_bytes())),
                        outbound: quoted.is_some(),
                        payload,
                        span: eng.cx.span_of(c),
                    });
                } else {
                    walk(eng, c, variant_kind, out);
                }
            }
        }
        walk(self, member, variant_kind, messages);
    }

    /// design 条目（value=表达式；if/else 形结构性保留 value=None）。
    fn collect_design_entries(&mut self, member: u32, design: &mut Vec<ComponentDesignEntry>) {
        let entry_kind = self
            .rule_kind_of("age-design-entry")
            .expect("design entry rule");
        let if_kind = self.rule_kind_of("age-if-expr");
        let else_kind = self.rule_kind_of("age-if-else");
        for c in self.cx.children(member) {
            if self.cx.tree.kind(c) != entry_kind {
                continue;
            }
            // R60-4：`key: if <cond> { <then> } else { <else> }`——三段都按表达式
            // 投影（字段是包装规则时先解包到 pratt 表达式）；普通条目三项为空。
            let mut cond = None;
            let mut then_value = None;
            let mut else_value = None;
            if let Some(if_node) = self.cx.field_child(c, "value") {
                let inner = self.unwrap_child(if_node);
                if Some(self.cx.tree.kind(inner)) == if_kind {
                    cond = self.cx.field_child(inner, "cond").map(|w| {
                        let e = self.unwrap_child(w);
                        self.project_expr_sync(e)
                    });
                    then_value = self
                        .cx
                        .field_child(inner, "then")
                        .and_then(|w| self.cx.field_child(w, "value"))
                        .map(|v| self.project_expr_sync(v));
                    // else 分支不是 inner 的字段，而是 `age-if-else` 子规则。
                    else_value = self
                        .cx
                        .children(inner)
                        .into_iter()
                        .find(|&w| Some(self.cx.tree.kind(w)) == else_kind)
                        .and_then(|w| self.cx.field_child(w, "value"))
                        .map(|v| self.project_expr_sync(v));
                }
            }
            let key = self.cx.field_child(c, "key").expect("design key");
            let value = match self.cx.field_child(c, "value") {
                Some(v) => {
                    let inner = self.unwrap_child(v);
                    if Some(self.cx.tree.kind(inner)) == if_kind {
                        None
                    } else {
                        Some(self.project_expr_sync(inner))
                    }
                }
                None => None,
            };
            design.push(ComponentDesignEntry {
                key: self.cx.spelling(key),
                value,
                cond,
                then_value,
                else_value,
                span: self.cx.span_of(c),
            });
        }
    }

    /// `on { .Msg -> { stmts } }` widget 形 → 与 979 `on <Msg> { assigns }` 同构。
    /// PLAN-981（IF-981-EVENT-A）：handler 参数名与 span（`.Add(t, u)`），按出现序。
    fn handler_params(&mut self, handler: u32) -> Vec<((String, Span), Option<(String, Span)>)> {
        let mut out = Vec::new();
        let Some(params) = self.cx.field_child(handler, "params") else {
            return out;
        };
        let mut stack = vec![params];
        while let Some(n) = stack.pop() {
            let annotation_node = self.cx.field_child(n, "param_ann");
            if let Some(p) = self.cx.field_child(n, "param") {
                let annotation = annotation_node
                    .map(|node| (self.cx.spelling(node), self.cx.span_of(node)));
                out.push(((self.cx.spelling(p), self.cx.span_of(p)), annotation));
            }
            stack.extend(self.cx.children(n).into_iter().filter(|&child| Some(child) != annotation_node));
        }
        out.sort_by_key(|((_, span), _)| span.start);
        out
    }

    /// PLAN-105 T-02：语句面里是否有**非列表方法**的调用语句（→ 域外体）。
    fn stmts_have_foreign_call(
        stmts: &[crate::source::age_component::ComponentStmt],
    ) -> bool {
        fn list_method(call: &crate::source::ast::Expr) -> bool {
            match &call.kind {
                crate::source::ast::ExprKind::MethodCall { name, .. } => matches!(
                    name.name.as_str(),
                    "push" | "get" | "len" | "pop" | "clear" | "set"
                ),
                _ => false,
            }
        }
        fn walk(stmts: &[crate::source::age_component::ComponentStmt]) -> bool {
            stmts.iter().any(|st| match st {
                crate::source::age_component::ComponentStmt::Call { call, .. } => {
                    !list_method(call)
                }
                crate::source::age_component::ComponentStmt::If { then, els, .. } => {
                    walk(then) || els.as_ref().is_some_and(|e| walk(e))
                }
                crate::source::age_component::ComponentStmt::For { body, .. }
                | crate::source::age_component::ComponentStmt::While { body, .. } => walk(body),
                _ => false,
            })
        }
        walk(stmts)
    }

    fn collect_widget_handlers(
        &mut self,
        member: u32,
        handlers: &mut Vec<ComponentHandler>,
        channel_names: &[String],
    ) {
        let handler_kind = self
            .rule_kind_of("age-on-widget-handler")
            .expect("handler rule");
        for c in self.cx.children(member) {
            if self.cx.tree.kind(c) != handler_kind {
                continue;
            }
            let msg = self.cx.field_child(c, "msg").expect("widget handler msg");
            let mut body_foreign = false;
            let mut channel_calls = Vec::new();
            // PLAN-981（on-if 组件面刀）：handler 体语句 = age-on-widget-stmt
            // （赋值/调用 + hook-local let/if——006 真语料形）。并集注记
            // （F-R21-3）：age-view-if-stmt 为**兜底臂**（980 view 方言——
            // 分支内调用形 foreign/链式作者面），PEG 序后置于 age-on-if，
            // 仅 age-on-if 整体失败时命中，走 980 的 project_view_if
            // （channel/foreign 语义原样）。
            let stmt_kind = self.rule_kind_of("age-on-widget-stmt");
            let view_if_kind = self.rule_kind_of("age-view-if-stmt");
            // 语句面按**源序**收集（G2 直接形同合同）：赋值双进平铺 assigns
            // （兼容面，消费方见 Let/If 必须具名拒——窗口声明在计划收据）
            // 与 stmts；let/if 只进 stmts。
            let mut stmts = Vec::new();
            for stmt in self.cx.children(c) {
                if Some(self.cx.tree.kind(stmt)) == stmt_kind {
                    // PLAN-980 WEB #6o/#6p + 981 on-if 并集：语句=alt——解包
                    // 后判形：age-view-if-stmt（兜底 view 方言 if）走 980 结构
                    // 化分支；age-view-assign-stmt（field/rhs/op 字段）走赋值
                    // 模型；调用形：callee ∈ widget 头参数=通道调用行（宿主
                    // 通道 prop，原源 span；实参仅位置形）；let/if 走 on 体
                    // 共享投影；否则 body_foreign（④ 外部名面族，
                    // fail-closed——976 后继目标域决策）。
                    let inner = self.unwrap_child(stmt);
                    if Some(self.cx.tree.kind(inner)) == view_if_kind {
                        stmts.push(self.project_view_if(inner, channel_names, &mut body_foreign));
                    } else if self.rule_name_of(inner) == "age-view-assign-stmt" {
                        let mut one = Vec::new();
                        self.collect_view_stmt(inner, &mut one);
                        if let Some(a) = one.pop() {
                            stmts.push(self.project_handler_assignment(a));
                        }
                    } else if let Some(call_node) = self.cx.field_child(inner, "call") {
                        match self.project_channel_call(call_node, channel_names) {
                            Some(cc) => channel_calls.push(cc),
                            None => {
                                // PLAN-105 T-02：非通道调用保留为 Call 语句（store
                                // 方法在 store 内联时展开；列表方法在 collections
                                // 域按表达式语句降级）；其余仍是域外体，派发具名拒。
                                let call = self.project_expr_sync(call_node);
                                let is_list_method = match &call.kind {
                                    crate::source::ast::ExprKind::MethodCall { name, .. } => {
                                        matches!(
                                            name.name.as_str(),
                                            "push" | "get" | "len" | "pop" | "clear" | "set"
                                        )
                                    }
                                    _ => false,
                                };
                                stmts.push(crate::source::age_component::ComponentStmt::Call {
                                    call,
                                    span: self.cx.span_of(inner),
                                });
                                if !is_list_method {
                                    body_foreign = true;
                                }
                            }
                        }
                    } else if let Some(st) = self.project_on_stmt_inner(inner) {
                        stmts.push(st);
                    } else {
                        body_foreign = true;
                    }
                }
            }
            // 兼容面：赋值子集 = stmts **全深度**收平（与直接形同一条
            // flat_handler_assigns——分支里的赋值不消失；执行语义以 stmts
            // 为准；980 view-if 兜底臂的 If 同经此收平）。
            let (params, param_annotations): (Vec<_>, Vec<_>) =
                self.handler_params(c).into_iter().unzip();
            let mut locals = params.iter().map(|(name, _)| name.clone()).collect();
            body_foreign |= Self::resolve_handler_scope(&mut stmts, &mut locals, true);
            let mut assigns = Vec::new();
            Self::flat_handler_assigns(&stmts, &mut assigns);
            // PLAN-105 T-02：分支/循环体里出现的**非列表方法**调用仍然是域外
            // 体（980 on-if 合同的负控：`side_effect()` 让 handler 落 foreign）。
            let body_foreign = body_foreign || Self::stmts_have_foreign_call(&stmts);
            handlers.push(ComponentHandler {
                msg: self.cx.spelling(msg),
                params,
                param_annotations,
                from_widget: true,
                body_foreign,
                channel_calls,
                stmts,
                assigns,
                span: self.cx.span_of(c),
            });
        }
    }

    /// PLAN-980 WEB #6p：调用语句 → 通道调用行。callee 是 widget 头参数名
    /// 且实参全为位置形时投影为行（args 原样、原源 span）；任何其它形状
    /// （非头参数 callee/具名实参/非调用表达式）返回 None（调用方落
    /// body_foreign——fail-closed，不静默降级）。
    fn project_channel_call(
        &mut self,
        call_node: u32,
        channel_names: &[String],
    ) -> Option<crate::source::age_component::ComponentChannelCall> {
        let expr = self.project_expr_sync(call_node);
        let span = self.cx.span_of(call_node);
        let crate::source::ast::ExprKind::Call { callee, args } = &expr.kind else {
            return None;
        };
        if !channel_names.iter().any(|n| *n == callee.name) {
            return None;
        }
        let mut values = Vec::new();
        for a in args {
            match a {
                crate::source::ast::Arg::Pos(v) | crate::source::ast::Arg::MovePos(v) => {
                    values.push(v.clone())
                }
                // 具名实参=通道面未承载形（fail-closed，不猜位置）。
                crate::source::ast::Arg::Named { .. } => return None,
            }
        }
        Some(crate::source::age_component::ComponentChannelCall {
            channel: callee.name.clone(),
            args: values,
            span,
        })
    }

    /// R61-2/R62-2：`watch { .a, .b -> … }` 的全部源（head + 每个 tail），
    /// 返回（拼写, 源 token span）；修饰形 `.a.b` 保留完整拼写。
    fn member_sources(&mut self, item: u32) -> Vec<(String, Span)> {
        let source_kind = self.rule_kind_of("age-watch-source");
        let mut out = Vec::new();
        fn walk(
            eng: &mut Engine,
            node: u32,
            source_kind: Option<u32>,
            out: &mut Vec<(String, Span)>,
        ) {
            for c in eng.cx.children(node) {
                if Some(eng.cx.tree.kind(c)) == source_kind {
                    if let Some(s) = eng.cx.field_child(c, "source") {
                        let mut text = eng.cx.spelling(s);
                        // R62-7：修饰在独立的 `age-watch-modifier` 子规则里。
                        let mut modifier = eng.cx.field_child(c, "modifier");
                        if modifier.is_none() {
                            for sub in eng.cx.children(c) {
                                if let Some(m) = eng.cx.field_child(sub, "modifier") {
                                    modifier = Some(m);
                                    break;
                                }
                            }
                        }
                        if let Some(m) = modifier {
                            text.push('.');
                            text.push_str(&eng.cx.spelling(m));
                        }
                        out.push((text, eng.cx.span_of(s)));
                    }
                } else {
                    walk(eng, c, source_kind, out);
                }
            }
        }
        walk(self, item, source_kind, &mut out);
        out
    }
    /// B-LANGUAGE-I（R59-3）：lifecycle/watch 体内语句（`.field = <rhs>`）按声明序。
    fn member_stmts(&mut self, item: u32) -> Vec<(String, crate::source::ast::Expr)> {
        let stmt_kind = self.rule_kind_of("age-view-stmt");
        let mut out = Vec::new();
        fn walk(
            eng: &mut Engine,
            node: u32,
            stmt_kind: Option<u32>,
            out: &mut Vec<(String, crate::source::ast::Expr)>,
        ) {
            for c in eng.cx.children(node) {
                if Some(eng.cx.tree.kind(c)) == stmt_kind {
                    // #6o：语句=alt——解包赋值形后取字段（调用形无 field，
                    // lifecycle/watch 体按方言合同不含调用——跳过）。
                    let inner = eng.unwrap_child(c);
                    let field = eng
                        .cx
                        .field_child(inner, "field")
                        .map(|f| eng.cx.spelling(f))
                        .unwrap_or_default();
                    if let Some(rhs) = eng.cx.field_child(inner, "rhs") {
                        // R60-1：同 `value`——`rhs` 是完整表达式，不再 unwrap 成首原子。
                        let expr = eng.project_expr_sync(rhs);
                        out.push((field, expr));
                    }
                } else {
                    walk(eng, c, stmt_kind, out);
                }
            }
        }
        walk(self, item, stmt_kind, &mut out);
        out
    }
    /// 通用成员槽位行（bind/route/lifecycle/computed/watch/timer）。
    fn collect_member_rows(
        &mut self,
        member: u32,
        item_rule: &str,
        role: &'static str,
        rows: &mut Vec<ComponentMemberRow>,
    ) {
        let item_kind = self
            .rule_kind_of(item_rule)
            .unwrap_or_else(|| panic!("{item_rule}"));
        for c in self.cx.children(member) {
            if self.cx.tree.kind(c) != item_kind {
                continue;
            }
            // 直接字段优先；`watch { .src -> … }` 的源在嵌套 `age-watch-source`
            // 里（字段 `source`），所以要能往下找一层（R59-4：此前 watch 行名字为空）。
            let name = self
                .cx
                .field_child(c, "key")
                .or_else(|| self.cx.field_child(c, "path"))
                .or_else(|| self.cx.field_child(c, "method"))
                .or_else(|| self.cx.field_child(c, "name"))
                .or_else(|| self.cx.field_child(c, "event"))
                .or_else(|| self.cx.field_child(c, "source"))
                .or_else(|| {
                    let mut found = None;
                    for sub in self.cx.children(c) {
                        if let Some(v) = self.cx.field_child(sub, "source") {
                            found = Some(v);
                            break;
                        }
                    }
                    found
                });
            let name = match name {
                Some(n) => self.cx.spelling(n),
                None => String::new(),
            };
            // R60-1：`pratt` 字段记录的是完整表达式树（二元链在这里），**不能**先
            // `unwrap_child` 取第一个原子——那会把 `count + 1` 截成 `count`。
            let value = self
                .cx
                .field_child(c, "value")
                .or_else(|| self.cx.field_child(c, "guard")
                    .and_then(|guard| self.cx.field_child(guard, "value")))
                .map(|v| self.project_expr_sync(v));
            let text = self
                .cx
                .field_child(c, "handler")
                .or_else(|| self.cx.field_child(c, "module"))
                .or_else(|| {
                    // watch 的"声明文本"就是它的源（`.count` / `.a.b`）。
                    let mut found = None;
                    for sub in self.cx.children(c) {
                        if let Some(v) = self.cx.field_child(sub, "source") {
                            found = Some(v);
                            break;
                        }
                    }
                    found
                })
                .map(|t| {
                    let mut spelling = self.cx.spelling(t);
                    if role == "route" {
                        spelling.retain(|c| !c.is_ascii_whitespace());
                    }
                    spelling
                });
            let num = self
                .cx
                .field_child(c, "v1")
                .map(|v| self.cx.spelling(v).parse::<i64>().unwrap_or(0));
            rows.push(ComponentMemberRow {
                role,
                name,
                value,
                text,
                num,
                stmts: self.member_stmts(c),
                sources: self
                    .member_sources(c)
                    .iter()
                    .map(|(s, _)| s.clone())
                    .collect(),
                source_spans: self.member_sources(c).iter().map(|(_, sp)| *sp).collect(),
                span: self.cx.span_of(c),
            });
        }
    }

    /// expose 名单（含逗号 tail）。
    fn collect_expose_names(&mut self, member: u32, rows: &mut Vec<ComponentMemberRow>) {
        let expose_span = self.cx.span_of(member);
        let mut names: Vec<String> = Vec::new();
        fn walk(eng: &mut Engine, node: u32, out: &mut Vec<String>) {
            // `expose { a, b }`：名字是 member 的**直接** ident 子叶，尾随名字在
            // `age-expose-name-tail` 的 `name` 字段里。必须收集**所有** name 字段
            // 再递归（R59-1：此前的 early-return 只留下第一个名字）。
            if let Some(n) = eng.cx.field_child(node, "name") {
                out.push(eng.cx.spelling(n));
            }
            for c in eng.cx.children(node) {
                walk(eng, c, out);
            }
        }
        walk(self, member, &mut names);
        for n in names {
            rows.push(ComponentMemberRow {
                role: "expose",
                name: n,
                value: None,
                text: None,
                num: None,
                stmts: Vec::new(),
                sources: Vec::new(),
                source_spans: Vec::new(),
                span: expose_span,
            });
        }
    }

    /// PLAN-968: `task <Name> { members }` -> `ast::TaskItem`. Members walk in
    /// source order; the projection enforces hook names (start/stop only),
    /// duplicate hooks and a single non-empty `on` block with the same named
    /// diagnostics as the handwritten oracle (`source.task-*`).
    fn run_task_item(&mut self, node: u32) -> TaskItem {
        let kw_leaf = self.cx.field_child(node, "kw").expect("task kw");
        let name_leaf = self.cx.field_child(node, "name").expect("task name");
        let name = ident_of(&mut self.cx, name_leaf);
        let member_kind = self.cx.k.task_member;
        let mut fields: Vec<TaskFieldDecl> = Vec::new();
        let mut seen_fields: Vec<String> = Vec::new();
        let mut start: Option<FnItem> = None;
        let mut stop: Option<FnItem> = None;
        let mut arms: Vec<OnArm> = Vec::new();
        let mut on_seen = false;
        for member in self.cx.children(node) {
            if Some(self.cx.tree.kind(member)) != member_kind {
                continue;
            }
            if let Some(hook_n) = self.cx.field_child(member, "hook") {
                let f = self.run_fn_item(hook_n, None, Receiver::Free);
                let hook = f.name.name.clone();
                let hook_span = f.name.span;
                if hook != "start" && hook != "stop" {
                    self.cx.reject(hook_span, || {
                        Diagnostic::new(
                            Stage::Text,
                            "source.task-lifecycle",
                            "only the `start` and `stop` lifecycle hooks may be declared inside a `task`",
                            hook_span,
                        )
                    });
                    continue;
                }
                let slot = if hook == "start" {
                    &mut start
                } else {
                    &mut stop
                };
                if slot.is_some() {
                    self.cx.reject(hook_span, || {
                        Diagnostic::new(
                            Stage::Text,
                            "source.task-lifecycle",
                            format!("duplicate `{hook}` lifecycle hook in the same task"),
                            hook_span,
                        )
                    });
                    continue;
                }
                *slot = Some(f);
            } else if let Some(on_n) = self.cx.field_child(member, "handler") {
                if on_seen {
                    let head = self.cx.field_child(on_n, "on").expect("on head");
                    let span = self.cx.span_of(head);
                    self.cx.reject(span, || {
                        Diagnostic::new(
                            Stage::Text,
                            "source.task-handler",
                            "duplicate `on` message-handler block in the same task",
                            span,
                        )
                    });
                    continue;
                }
                on_seen = true;
                arms = self.run_task_on(on_n);
            } else if let Some(field_n) = self.cx.field_child(member, "field") {
                let fname_leaf = self.cx.field_child(field_n, "name").expect("field name");
                let fname = ident_of(&mut self.cx, fname_leaf);
                if fname.name == "on" {
                    let span = fname.span;
                    self.cx.reject(span, || {
                        Diagnostic::new(
                            Stage::Text,
                            "source.task-handler",
                            "`on` introduces the message-handler block and cannot be a state field",
                            span,
                        )
                    });
                    continue;
                }
                if seen_fields.contains(&fname.name) {
                    let span = fname.span;
                    self.cx.reject(span, || {
                        Diagnostic::new(
                            Stage::Text,
                            "source.task-field",
                            format!("duplicate task state field `{}`", fname.name),
                            span,
                        )
                    });
                    continue;
                }
                seen_fields.push(fname.name.clone());
                let value_n = self
                    .cx
                    .field_child(field_n, "value")
                    .expect("field default");
                self.stack.push(Frame::ExprEnter(value_n));
                self.drive();
                let default = self.pop_e();
                let span = fname.span.join(default.span);
                fields.push(TaskFieldDecl {
                    name: fname,
                    default,
                    span,
                });
            }
        }
        if !on_seen {
            let span = self.cx.span_of(kw_leaf);
            self.cx.reject(span, || {
                Diagnostic::new(
                    Stage::Text,
                    "source.task-handler",
                    "a `task` declaration requires one `on { ... }` message-handler block",
                    span,
                )
            });
        }
        let span = self.cx.span_of(node);
        TaskItem {
            name,
            single: false,
            fields,
            start,
            stop,
            arms,
            span,
        }
    }

    /// PLAN-968: the `on { arms }` block — arms in source order.
    fn run_task_on(&mut self, node: u32) -> Vec<OnArm> {
        let arm_kind = self.cx.k.task_arm;
        let mut arms = Vec::new();
        for arm in self.cx.children(node) {
            if Some(self.cx.tree.kind(arm)) != arm_kind {
                continue;
            }
            let pat_n = self.cx.field_child(arm, "pattern").expect("arm pattern");
            let pattern = self.run_task_pattern(pat_n);
            let body_n = self.cx.field_child(arm, "body").expect("arm body");
            self.enter_block(body_n);
            self.drive();
            let body = self.pop_b();
            let span = self.cx.span_of(arm);
            let guard = self
                .cx
                .field_child(arm, "guard")
                .and_then(|guard| self.cx.field_child(guard, "cond"))
                .map(|cond| self.project_expr_sync(cond));
            arms.push(OnArm {
                pattern,
                guard,
                body,
                span,
            });
        }
        arms
    }

    /// PLAN-968: one arm pattern — literal / typed binding / wildcard, with
    /// the same named rejections as the oracle (`source.task-pattern`,
    /// `source.literal-range`).
    fn run_task_pattern(&mut self, node: u32) -> TaskPattern {
        if let Some(fallback) = self.cx.field_child(node, "fallback") {
            return TaskPattern::Wildcard(self.cx.span_of(fallback));
        }
        if let Some(lit) = self.cx.child_of_kind(node, self.cx.k.tok_int_lit) {
            let span = self.cx.span_of(lit);
            let raw = self.cx.spelling(lit);
            let mag: u128 = raw.parse().unwrap_or(u128::MAX);
            if mag > 2147483647 {
                self.cx.reject(span, || {
                    Diagnostic::new(
                        Stage::Text,
                        "source.literal-range",
                        format!(
                            "integer literal `{raw}` is outside the signed 32-bit range of this source profile"
                        ),
                        span,
                    )
                });
                return TaskPattern::Lit(Expr {
                    span,
                    kind: ExprKind::IntLit(0),
                });
            }
            return TaskPattern::Lit(Expr {
                span,
                kind: ExprKind::IntLit(mag as i32),
            });
        }
        if let Some(lit) = self.cx.child_of_kind(node, self.cx.k.tok_str_lit) {
            let span = self.cx.span_of(lit);
            let raw = self.cx.spelling(lit);
            let inner = &raw.as_bytes()[1..raw.len() - 1];
            let value = lex::decode_str_value(inner);
            return TaskPattern::Lit(Expr {
                span,
                kind: ExprKind::StrLit(value),
            });
        }
        let bind_leaf = self
            .cx
            .child_of_kind(node, self.cx.k.tok_ident)
            .expect("pattern ident");
        let bind = ident_of(&mut self.cx, bind_leaf);
        let ty = self.cx.field_child(node, "ty");
        if self.cx.profile.is_auto_language() && bind.name != "_" && ty.is_none() {
            let mut bindings = Vec::new();
            if let Some(payload) = self.cx.field_child(node, "payload") {
                let mut nodes = vec![payload];
                while let Some(child) = nodes.pop() {
                    if self.cx.tree.kind(child) == self.cx.k.tok_ident {
                        bindings.push(ident_of(&mut self.cx, child));
                    } else {
                        nodes.extend(self.cx.children(child).into_iter().rev());
                    }
                }
            }
            return TaskPattern::Named {
                name: bind,
                bindings,
            };
        }
        if bind.name == "_" {
            if let Some(ty_n) = ty {
                let span = self.cx.span_of(ty_n);
                self.cx.reject(span, || {
                    Diagnostic::new(
                        Stage::Text,
                        "source.task-pattern",
                        "the `_` wildcard pattern cannot carry a type annotation",
                        span,
                    )
                });
            }
            return TaskPattern::Wildcard(bind.span);
        }
        match ty {
            Some(ty_n) => {
                let ann = self.classify_word(ty_n, false, false);
                TaskPattern::Bind {
                    name: bind,
                    ty: ann,
                }
            }
            None => {
                let span = bind.span;
                self.cx.reject(span, || {
                    Diagnostic::new(
                        Stage::Text,
                        "source.task-pattern",
                        "a message binding pattern requires a type annotation (`n int ->`)",
                        span,
                    )
                });
                TaskPattern::Wildcard(span)
            }
        }
    }

    /// PLAN-1080（D-05）：`fn name(params) ret;` 原型臂投影——签名先行、无
    /// 体。`proto=true` + 空体 FnItem；补全遍负责与同签名实体合一并移除。
    fn run_fn_proto(&mut self, node: u32, owner: Option<Ident>, receiver: Receiver) -> FnItem {
        let name_leaf = self.cx.field_child(node, "name").expect("proto fn name");
        let name = ident_of(&mut self.cx, name_leaf);
        let params = self.project_params(self.cx.field_child(node, "params"));
        let ret = match self.cx.field_child(node, "ty") {
            Some(ty_n) => self.classify_word(ty_n, true, false),
            None => {
                let span = name.span;
                self.cx.reject(span, || {
                    Diagnostic::new(
                        Stage::Text,
                        "source.unsupported",
                        "a prototype declaration requires a return type annotation (`fn test() int;`)",
                        span,
                    )
                });
                TypeAnn {
                    kind: TypeRef::Inferred,
                    span,
                }
            }
        };
        let span = self.cx.span_of(node);
        FnItem {
            name,
            type_params: Vec::new(),
            params,
            ret,
            body: crate::source::ast::Block {
                stmts: Vec::new(),
                span,
            },
            span,
            receiver,
            owner,
            script: false,
            proto: true,
            test_attribute: None,
        }
    }

    fn run_fn_item(&mut self, node: u32, owner: Option<Ident>, receiver: Receiver) -> FnItem {
        self.run_fn_item_with_params(node, owner, receiver, Vec::new())
    }

    fn run_fn_item_with_params(
        &mut self,
        node: u32,
        owner: Option<Ident>,
        receiver: Receiver,
        attribute_params: Vec<TypeParamDecl>,
    ) -> FnItem {
        // PLAN-1080（D-05）：签名先行原型臂（`fn test() int;`——auto 面文法
        // 专属，无 fn-tail 包装）。投影为 proto=true 的空体 FnItem，补全遍
        // 与同签名实体合一；无实体 = 具名 bind 错。
        if self.cx.field_child(node, "proto").is_some() {
            let proto = self.run_fn_proto(node, owner, receiver);
            if let Some(param) = attribute_params.first() {
                let span = param.name.span;
                self.cx.reject(span, || {
                    Diagnostic::new(
                        Stage::Bind,
                        "source.attribute-position",
                        "`with` requires a function body",
                        span,
                    )
                });
            }
            return proto;
        }
        let tail = self
            .cx
            .children(node)
            .into_iter()
            .find(|&c| self.cx.tree.kind(c) == self.cx.k.fn_tail)
            .expect("fn-item carries fn-tail");
        let name_leaf = self.cx.field_child(tail, "name").expect("fn name");
        let name = ident_of(&mut self.cx, name_leaf);
        if receiver.is_instance() && name.name == "self" {
            let span = name.span;
            self.cx.reject(span, || self_method_name(span));
        }
        // PLAN-935（T-02）：`<T>` 类型参数投影（scalar-instances 面才有
        // 该字段；plain 面无此步 = 无字段）。PLAN-963：多类型参数
        // `<T, U>` 为通用实例面的一部分（声明序保住）；约束/const 形状仍
        // 具名拒（T-01 冻结的独立切片），模板方法与模板入口仍拒。
        let declared = self.project_type_params(self.cx.field_child(tail, "type_params"));
        let type_params = self.merge_attribute_params(declared, attribute_params);
        if !type_params.is_empty() {
            let tp_span = type_params[0].name.span;
            if !self.cx.profile.is_auto_language()
                && (receiver != Receiver::Free || owner.is_some())
            {
                self.cx.reject(tp_span, || {
                    Diagnostic::new(
                        Stage::Text,
                        "source.instances-unsupported",
                        "template methods are not part of the finite instances surface",
                        tp_span,
                    )
                });
            } else if receiver == Receiver::Free && name.name == "main" {
                self.cx.reject(tp_span, || {
                    Diagnostic::new(
                        Stage::Text,
                        "source.instances-unsupported",
                        "the entry function cannot be a template",
                        tp_span,
                    )
                });
            }
        }
        // fn 投影期间类型变量可见（参数/返回/体内注解位）。
        // fn 投影期间类型变量可见（参数/返回/体内注解位）。PLAN-963：泛型
        // 记录的成员方法继承外层声明的类型参数（`Point<T>` 的方法里 `T` 仍
        // 是类型变量），故与当前作用域取并集而非覆盖。
        let saved_ty_params = std::mem::take(&mut self.cx.ty_params);
        self.cx.ty_params = saved_ty_params
            .iter()
            .cloned()
            .chain(type_params.iter().map(|t| t.name.name.clone()))
            .collect();
        let params = self.project_params(self.cx.field_child(tail, "params"));
        if receiver.is_instance() {
            for prm in &params {
                if prm.name.name == "self" {
                    let span = prm.name.span;
                    self.cx.reject(span, || self_param(span));
                }
            }
        }
        // PLAN-968: bare unit-result fns (`fn main() { ... }`) — the ret-ty
        // step is optional in the grammar; the scheduler face projects a
        // synthesized Void result, every other face keeps the historical
        // required-ret contract (the N42 `source.unsupported` rejection,
        // byte-identical with the pre-968 parse-failure mapping).
        let ret = match self.cx.field_child(tail, "ty") {
            Some(ty_n) => self.classify_word(ty_n, true, false),
            None => {
                let span = name.span.join(fn_tail_rparen_span(&self.cx, tail));
                if self.cx.profile.is_scheduler() || self.cx.field_child(tail, "effect").is_some() {
                    TypeAnn {
                        kind: TypeRef::Builtin(TypeWord::Void),
                        span,
                    }
                } else if self.cx.profile.is_auto_language() {
                    // PLAN-1080（D-04）：auto 面无返回注解 = 推导标记（补全遍
                    // 从体推导；`fn main()` 无尾值时落 Unit）。
                    TypeAnn {
                        kind: TypeRef::Inferred,
                        span,
                    }
                } else {
                    self.cx.reject(span, || {
                        Diagnostic::new(
                            Stage::Text,
                            "source.unsupported",
                            "function return type annotation (`int`/`i32`/`bool` or a record type) is required",
                            span,
                        )
                    });
                    TypeAnn {
                        kind: TypeRef::Builtin(TypeWord::Void),
                        span,
                    }
                }
            }
        };
        let body_n = self.cx.field_child(tail, "body").expect("fn body");
        let returns_tail = self.cx.age_face && !matches!(ret.kind, TypeRef::Builtin(TypeWord::Void));
        let owner_for_frame = owner.clone();
        self.stack.push(Frame::FnBodyFinish {
            node,
            name,
            type_params,
            params,
            ret,
            receiver,
            owner: owner_for_frame,
        });
        self.enter_block_with_tail(body_n, returns_tail);
        self.drive();
        self.cx.ty_params = saved_ty_params;
        match self.vals.pop() {
            Some(Val::F(f)) => f,
            _ => FnItem {
                name: Ident {
                    name: String::new(),
                    span: Span { start: 0, end: 0 },
                },
                type_params: Vec::new(),
                params: Vec::new(),
                ret: TypeAnn {
                    kind: TypeRef::Builtin(TypeWord::Int),
                    span: Span { start: 0, end: 0 },
                },
                body: Block {
                    stmts: Vec::new(),
                    span: Span { start: 0, end: 0 },
                },
                span: Span { start: 0, end: 0 },
                receiver,
                owner: owner.clone(),
                script: false,
                proto: false,
                test_attribute: None,
            },
        }
    }

    /// PLAN-935（T-02）：fn-tail `<T>` → `Vec<Ident>`（声明序；重名具名拒）。
    /// PLAN-935（T-02）/PLAN-963（T-02）：fn 模板的类型参数声明。
    /// `scalar-instances` 面只投影 `<T>`；`record-instances` 面另有
    /// `<T has S>`（约束随声明携带）。`const N` 值参数仍具名拒（独立切片）。
    fn project_type_params(&mut self, node: Option<u32>) -> Vec<TypeParamDecl> {
        self.project_type_param_decls(node)
    }

    fn project_decl_attributes(
        &mut self,
        wrapper: u32,
        owner: u32,
        av_visible: bool,
    ) -> (Vec<TypeParamDecl>, Option<Span>, bool) {
        let kind = self.cx.tree.kind(owner);
        let function = kind == self.cx.k.fn_item;
        let method = kind == self.cx.k.static_method || kind == self.cx.k.view_method;
        let record = Some(kind) == self.cx.k.type_item;
        let body = function && self.cx.field_child(owner, "proto").is_none();
        let mut params = Vec::new();
        let mut test = None;
        let mut single = false;
        let mut seen = BTreeSet::new();
        for entry in attribute_entries(self.cx.p, self.cx.tree, wrapper) {
            let name_node = self.cx.field_child(entry, "name").expect("attribute name");
            let name = self.cx.spelling(name_node);
            let span = self.cx.span_of(name_node);
            let args = self.cx.field_child(entry, "args");
            let error = if !seen.insert(name.clone()) {
                Some((
                    "source.attribute-duplicate",
                    format!("duplicate attribute `{name}`"),
                ))
            } else if name == "test" {
                if !body || args.is_some() {
                    Some((
                        "source.attribute-position",
                        "`test` marks a function body and takes no arguments".into(),
                    ))
                } else {
                    test = Some(span);
                    continue;
                }
            } else if matches!(name.as_str(), "c" | "vm" | "rs") {
                if (function || method || record) && args.is_none() {
                    continue;
                }
                Some((
                    "source.attribute-position",
                    format!("`{name}` selects a function or type target and takes no arguments"),
                ))
            } else if name == "single" {
                if Some(kind) == self.cx.k.task_item && args.is_none() {
                    single = true;
                    continue;
                }
                Some((
                    "source.attribute-position",
                    "`single` marks a task and takes no arguments".into(),
                ))
            } else if name == "async" {
                if (body || method) && args.is_none() {
                    // The function's existing ~T/effect surface owns execution.
                    continue;
                }
                Some((
                    "source.attribute-position",
                    "`async` marks a function and takes no arguments".into(),
                ))
            } else if matches!(
                name.as_str(),
                "api" | "export" | "derive" | "serde" | "tokio" | "allow" | "cfg"
            ) {
                if let Err(diagnostic) =
                    self.project_host_attribute(owner, &name, span, args, av_visible)
                {
                    self.cx.reject(diagnostic.span, || diagnostic);
                }
                continue;
            } else if matches!(name.as_str(), "script" | "rust") {
                Some((
                    "source.attribute-position",
                    "file mode pragmas apply to the source file".into(),
                ))
            } else if name != "with" {
                Some((
                    "source.attribute-unknown",
                    format!("unknown attribute `{name}`"),
                ))
            } else if !function && !method && !(record && !self.is_type_alias(owner)) {
                Some((
                    "source.attribute-position",
                    "`with` requires a function or record declaration".into(),
                ))
            } else {
                None
            };
            if let Some((code, message)) = error {
                self.cx.reject(span, || Diagnostic::new(Stage::Bind, code, message, span));
                continue;
            }
            let args = args.expect("with arguments");
            let parameter_kind = self
                .rule_kind_of("with-attribute-parameter")
                .expect("with attribute parameter");
            let mut names = BTreeSet::new();
            for parameter in collect_seq(&self.cx, args, parameter_kind, "with-attribute-tail") {
                let leaf = self.cx.field_child(parameter, "name").expect("type parameter name");
                let name = ident_of(&mut self.cx, leaf);
                if !names.insert(name.name.clone()) {
                    self.cx.reject(name.span, || {
                        Diagnostic::new(
                            Stage::Bind,
                            "source.attribute-duplicate",
                            format!("duplicate type parameter `{}`", name.name),
                            name.span,
                        )
                    });
                    continue;
                }
                let bound = self
                    .cx
                    .field_child(parameter, "constraint")
                    .and_then(|constraint| self.cx.field_child(constraint, "bound"))
                    .map(|bound| ident_of(&mut self.cx, bound));
                params.push(TypeParamDecl {
                    name,
                    bound,
                    kind: TypeParamKind::Type,
                });
            }
        }
        (params, test, single)
    }

    fn merge_attribute_params(
        &mut self,
        mut declared: Vec<TypeParamDecl>,
        attribute_params: Vec<TypeParamDecl>,
    ) -> Vec<TypeParamDecl> {
        for param in attribute_params {
            let existing = declared.iter_mut().find(|ty| ty.name.name == param.name.name);
            match existing {
                None => declared.push(param),
                Some(existing) if existing.kind == TypeParamKind::Type && existing.bound.is_none() => {
                    existing.bound = param.bound;
                }
                Some(_) => {
                    let span = param.name.span;
                    self.cx.reject(span, || {
                        Diagnostic::new(
                            Stage::Bind,
                            "source.attribute-duplicate",
                            format!(
                                "duplicate constraint or conflicting declaration for `{}`",
                                param.name.name
                            ),
                            span,
                        )
                    });
                }
            }
        }
        declared
    }

    fn project_host_attribute(
        &mut self,
        owner: u32,
        name: &str,
        span: Span,
        args: Option<u32>,
        av_visible: bool,
    ) -> Result<(), Diagnostic> {
        let owner_kind = self.cx.tree.kind(owner);
        let function = owner_kind == self.cx.k.fn_item;
        let method = owner_kind == self.cx.k.static_method || owner_kind == self.cx.k.view_method;
        let error = |code, message: &str| Diagnostic::new(Stage::Bind, code, message, span);
        if matches!(name, "api" | "export") && !function {
            return Err(error(
                "source.attribute-position",
                "this attribute requires a function",
            ));
        }
        let declaration = if function || method {
            let node = self
                .cx
                .child_of_kind(owner, self.cx.k.fn_tail)
                .or_else(|| self.cx.field_child(owner, "proto"))
                .expect("function declaration header");
            self.cx.field_child(node, "name")
        } else {
            self.cx.field_child(owner, "name")
        };
        let Some(declaration) = declaration else {
            return Err(error(
                "source.attribute-position",
                "this attribute requires a named declaration",
            ));
        };
        let kind = match name {
            "api" => {
                let mut method = "GET".to_string();
                let mut path = "/".to_string();
                let mut keys = BTreeSet::new();
                for arg in self.annotation_values(args) {
                    let key = self
                        .cx
                        .field_child(arg, "key")
                        .map(|key| self.cx.spelling(key));
                    let Some(key) = key else {
                        return Err(error(
                            "source.attribute-argument",
                            "`api` expects named arguments",
                        ));
                    };
                    if !keys.insert(key.clone()) {
                        return Err(error(
                            "source.attribute-duplicate",
                            "duplicate `api` argument",
                        ));
                    }
                    let node = self.cx.field_child(arg, "value").expect("attribute value");
                    let value = self.project_expr_sync(node);
                    let ExprKind::StrLit(value) = &value.kind else {
                        return Err(error(
                            "source.attribute-argument",
                            "`api` expects string literals",
                        ));
                    };
                    match key.as_str() {
                        "method" => method.clone_from(value),
                        "path" => path.clone_from(value),
                        _ => {
                            return Err(error(
                                "source.attribute-argument",
                                "unknown `api` argument",
                            ))
                        }
                    }
                }
                SourceAnnotationKind::Api { method, path }
            }
            "export" => {
                let values = self.annotation_values(args);
                let abi = match values.as_slice() {
                    [] => String::new(),
                    [arg] if self.cx.field_child(*arg, "key").is_none() => {
                        let node = self.cx.field_child(*arg, "value").expect("attribute value");
                        match &self.project_expr_sync(node).kind {
                            ExprKind::StrLit(abi) => abi.clone(),
                            ExprKind::Var(abi) => abi.name.clone(),
                            _ => {
                                return Err(error(
                                    "source.attribute-argument",
                                    "`export` expects one ABI name",
                                ))
                            }
                        }
                    }
                    _ => {
                        return Err(error(
                            "source.attribute-argument",
                            "`export` expects one ABI name",
                        ))
                    }
                };
                let abi = match abi.as_str() {
                    "" | "c" | "C" => "C".into(),
                    "stdcall" => "system".into(),
                    _ => abi,
                };
                SourceAnnotationKind::Export { abi }
            }
            _ => SourceAnnotationKind::Rust {
                name: name.into(),
                arguments: args.map(|node| self.cx.span_of(node)),
            },
        };
        self.cx.annotations.push(SourceAnnotation {
            declaration: self.cx.spelling(declaration),
            owner_span: self.cx.span_of(owner),
            span,
            av_visible,
            kind,
        });
        Ok(())
    }

    fn annotation_values(&self, args: Option<u32>) -> Vec<u32> {
        let Some(args) = args else { return Vec::new() };
        let values_kind = self
            .rule_kind_of("attribute-values")
            .expect("attribute values");
        let arg_kind = self
            .rule_kind_of("attribute-value")
            .expect("attribute value");
        self.cx
            .child_of_kind(args, values_kind)
            .map_or_else(Vec::new, |values| {
                collect_seq(&self.cx, values, arg_kind, "attribute-value-tail")
            })
    }

    fn project_record_method(&mut self, wrapper: u32, owner: &Ident) -> Option<FnItem> {
        let attributed = attributed_owner(self.cx.p, self.cx.tree, wrapper);
        let node = attributed.unwrap_or(wrapper);
        let kind = self.cx.tree.kind(node);
        let receiver = if kind == self.cx.k.static_method {
            Receiver::Static
        } else if kind == self.cx.k.view_method {
            if self.cx.field_child(node, "mut_receiver").is_some() {
                Receiver::Mut
            } else {
                Receiver::View
            }
        } else {
            return None;
        };
        let mut params = Vec::new();
        if attributed.is_some() {
            let visible = attribute_targets_av(self.cx.p, self.cx.tree, self.cx.snap, wrapper);
            (params, _, _) = self.project_decl_attributes(wrapper, node, visible);
            if !visible {
                return None;
            }
        }
        Some(self.run_fn_item_with_params(node, Some(owner.clone()), receiver, params))
    }

    /// PLAN-1043（T-02）：`pub spec fn name(params) Ret`（fn-require-p 节点，
    /// packages-ct 面）→ 需求签名（bodyless；params/ret 复用 fn-tail 同一
    /// 投影机器）。无类型参数（语法面即无）。
    fn run_require_item(&mut self, node: u32) -> (Ident, Vec<Param>, TypeAnn) {
        let name_leaf = self.cx.field_child(node, "name").expect("require name");
        let name = ident_of(&mut self.cx, name_leaf);
        let params = self.project_params(self.cx.field_child(node, "params"));
        let ret = self.classify_word(
            self.cx.field_child(node, "ty").expect("require ret"),
            true,
            false,
        );
        self.drive();
        (name, params, ret)
    }

    /// PLAN-963（T-03）：用户泛型类型的参数声明（`<T>` / `<T has S>`）。
    /// 只有 record/aggregates 面允许 `has`；`const N` 仍具名拒（值参数未在
    /// 本分片实现）。
    fn project_type_param_decls(&mut self, node: Option<u32>) -> Vec<TypeParamDecl> {
        let mut out = Vec::new();
        let Some(n) = node else { return out };
        let Some(tp_kind) = self.cx.k.type_param else {
            return out;
        };
        for tp in collect_seq(&self.cx, n, tp_kind, "type-param-tail") {
            let leaf = self.cx.field_child(tp, "name").expect("type param name");
            let name = ident_of(&mut self.cx, leaf);
            // PLAN-963（T-04）：值参数。唯一接受形 `<const N>` / `<const N u32>`
            // （声明类型缺省 u32）；其他声明类型与非实例面具名拒。
            let mut kind = TypeParamKind::Type;
            if let Some(cst_n) = self.cx.field_child(tp, "const_kw") {
                let span = self.cx.span_of(cst_n);
                if !self.cx.instances_face {
                    self.cx.reject(span, || {
                        Diagnostic::new(
                            Stage::Text,
                            "source.instances-unsupported",
                            "const (value) type parameters require the instances capability",
                            span,
                        )
                    });
                } else if let Some(ty_node) = self.cx.field_child(tp, "ty") {
                    let leaf = self.cx.field_child(ty_node, "word").unwrap_or(ty_node);
                    let spelled = self.cx.spelling(leaf);
                    if spelled != "u32" {
                        let tsp = self.cx.span_of(leaf);
                        self.cx.reject(tsp, || {
                            Diagnostic::new(
                                Stage::Text,
                                "source.instances-unsupported",
                                format!(
                                    "const parameter `{}` declares type `{}`; only `u32` is supported",
                                    name.name, spelled
                                ),
                                tsp,
                            )
                        });
                    }
                }
                kind = TypeParamKind::Const;
            }
            let bound = self
                .cx
                .field_child(tp, "constraint")
                .and_then(|c| self.cx.field_child(c, "bound"))
                .map(|b| ident_of(&mut self.cx, b));
            if out.iter().any(|p: &TypeParamDecl| p.name.name == name.name) {
                let span = name.span;
                self.cx.reject(span, || {
                    Diagnostic::new(
                        Stage::Text,
                        "source.duplicate",
                        format!("duplicate type parameter `{}`", name.name),
                        span,
                    )
                });
            }
            out.push(TypeParamDecl { name, bound, kind });
        }
        out
    }

    fn project_params(&mut self, params_node: Option<u32>) -> Vec<Param> {
        let mut out = Vec::new();
        let Some(pn) = params_node else { return out };
        let direct = collect_seq(&self.cx, pn, self.cx.k.param, "param-tail");
        for param_n in direct {
            let name_leaf = self.cx.field_child(param_n, "name").expect("param name");
            let name = ident_of(&mut self.cx, name_leaf);
            let mode_leaf = self.cx.field_child(param_n, "mode");
            let mode = match mode_leaf.map(|m| self.cx.tree.kind(m)) {
                Some(mk) if mk == self.cx.k.tok_r_move => {
                    // PLAN-928：closures 面的 move 参数（`fn make(move s Str)`）
                    // 与 text 面同为合法 mode 词。
                    if !(self.cx.profile.is_text() || self.cx.profile.is_closures()) {
                        let mspan = self.cx.span_of(mode_leaf.unwrap());
                        self.cx.reject(mspan, || {
                            Diagnostic::new(
                                Stage::Text,
                                "source.unsupported",
                                "keyword `move` cannot be used as a name",
                                mspan,
                            )
                        });
                    }
                    ParamMode::Move
                }
                Some(mk) if mk == self.cx.k.tok_mut => {
                    if !self.cx.profile.is_text() && !self.cx.profile.is_record() {
                        let mspan = self.cx.span_of(mode_leaf.unwrap());
                        self.cx.reject(mspan, || {
                            Diagnostic::new(
                                Stage::Text,
                                "source.unsupported",
                                "keyword `mut` cannot be used as a name",
                                mspan,
                            )
                        });
                    }
                    ParamMode::Mut
                }
                _ => ParamMode::Default,
            };
            // PLAN-1080（D-04）：auto 面无类型形参（`fn add(a, b)`）——注解
            // 步在文法层可选；缺位投影为 Inferred 标记，由补全遍定 concrete
            // 类型。冻结窄面文法必选注解，此分支不可达。
            let ty = match self.cx.field_child(param_n, "ty") {
                Some(ty_n) => self.classify_word(ty_n, false, true),
                None => {
                    let span = name.span;
                    TypeAnn {
                        kind: TypeRef::Inferred,
                        span,
                    }
                }
            };
            // PLAN-963（T-08，L-24）：`n int = 42` 的默认值。默认值在**调用位**
            // 按调用者作用域求值，缺席实参时每个调用只求值一次。
            let default = self
                .cx
                .field_child(param_n, "default")
                .and_then(|d| self.cx.field_child(d, "value"))
                .map(|v| Box::new(self.project_expr_sync(v)));
            let span = match mode_leaf {
                Some(m) => self.cx.span_of(m).join(ty.span),
                None => name.span.join(ty.span),
            };
            out.push(Param {
                span,
                name,
                ty,
                mode,
                default,
            });
        }
        out
    }

    /// ret-ty/ann-ty → TypeAnn（词法分类 + profile 拒绝；非递归）。
    /// `pos`：Ret 位在 record 面收**任意** ident（含 str/Str/char——旧
    /// fn_item_tail 的名义分支）；param/field/let 注解位先做文本词拒绝。
    fn classify_word(&mut self, node: u32, ret_pos: bool, param_pos: bool) -> TypeAnn {
        self.classify_word_at(node, ret_pos, param_pos, false)
    }

    fn classify_storage_word(&mut self, node: u32) -> TypeAnn {
        self.classify_word_at(node, false, false, true)
    }

    fn classify_word_at(
        &mut self,
        node: u32,
        ret_pos: bool,
        param_pos: bool,
        stored: bool,
    ) -> TypeAnn {
        // PLAN-962（L-15）：tuple 类型（ann 位的 tuple-ty 子节点承载）。
        if let Some(tt) = self.cx.field_child(node, "tuple") {
            let span = self.cx.span_of(tt);
            let mut args: Vec<TypeAnn> = Vec::new();
            if let Some(head) = self.cx.field_child(tt, "head") {
                let hw = self
                    .cx
                    .child_of_kind(head, self.cx.k.ann_ty)
                    .unwrap_or(head);
                args.push(self.classify_word(hw, false, false));
            }
            // tail ann-ty children of tuple-ty-tail nodes, absolute order
            let tail_kind = self.cx.k.tuple_ty_tail.unwrap_or(u32::MAX);
            let mut tails: Vec<u32> = self
                .cx
                .children(tt)
                .into_iter()
                .filter(|&c| self.cx.tree.kind(c) == tail_kind)
                .collect();
            tails.sort_by_key(|&n| self.cx.tree.abs_range(n).0);
            for t in tails {
                if let Some(a) = self
                    .cx
                    .children(t)
                    .into_iter()
                    .find(|&c| self.cx.tree.kind(c) == self.cx.k.ann_ty)
                {
                    args.push(self.classify_word(a, false, false));
                }
            }
            let _ = ret_pos;
            return TypeAnn {
                kind: TypeRef::Tuple(args),
                span,
            };
        }
        // PLAN-928：closures 面函数类型（ret/ann 位的 fn-type 子节点承载）。
        if let Some(ft) = self.cx.field_child(node, "fn") {
            return self.project_fn_type(ft);
        }
        // PLAN-928（T-03）：collections/closures 类型位参数化/可选糖。
        if let Some(t) = self.cx.field_child(node, "tilde") {
            return self.project_tilde_ty(t);
        }
        if let Some(e) = self.cx.field_child(node, "err") {
            return self.project_err_ty(e);
        }
        if let Some(g) = self.cx.field_child(node, "generic") {
            return self.project_generic_ty(g);
        }
        // PLAN-105 T-02（U03 真语料 012）：`[]T` / `[N]T` 列表注解（age-list-ty）
        // ——按 `List<T>` 承载；元素注解递归同一入口。此前落到「type word leaf」
        // 的 `.expect`（真语料 012 直接 panic），现在按结构投影。
        if let Some(l) = self.cx.field_child(node, "list") {
            let span = self.cx.span_of(l);
            let elem_node = self.cx.field_child(l, "elem").unwrap_or(l);
            let mut elem = self.classify_word(elem_node, false, false);
            // 元素位用 owned 文本（`[]str` → `List<Str>`；822 元素域不收借用 str）。
            if matches!(elem.kind, TypeRef::Builtin(crate::source::ast::TypeWord::Str)) {
                elem.kind = TypeRef::Builtin(crate::source::ast::TypeWord::StrOwned);
            }
            return TypeAnn {
                kind: TypeRef::Generic {
                    name: Ident {
                        name: "List".to_string(),
                        span,
                    },
                    args: vec![elem],
                },
                span,
            };
        }
        if let Some(o) = self.cx.field_child(node, "optional") {
            return self.project_optional_ty(o);
        }
        let word_leaf = self
            .cx
            .children(node)
            .into_iter()
            .find(|&c| {
                let k = self.cx.tree.kind(c);
                k == self.cx.k.tok_int_kw
                    || k == self.cx.k.tok_i32
                    || k == self.cx.k.tok_bool
                    || k == self.cx.k.tok_ident
                    // PLAN-961: the numeric type-word tokens (numeric
                    // trim only; Option kinds elsewhere).
                    || self.cx.k.tok_i8_kw == Some(k)
                    || self.cx.k.tok_u8_kw == Some(k)
                    || self.cx.k.tok_byte_kw == Some(k)
                    || self.cx.k.tok_uint_kw == Some(k)
                    || self.cx.k.tok_u32_kw == Some(k)
                    || self.cx.k.tok_i64_kw == Some(k)
                    || self.cx.k.tok_u64_kw == Some(k)
                    || self.cx.k.tok_float_kw == Some(k)
                    || self.cx.k.tok_f32_kw == Some(k)
                    || self.cx.k.tok_double_kw == Some(k)
                    || self.cx.k.tok_f64_kw == Some(k)
            })
            .expect("type word leaf");
        let span = self.cx.span_of(word_leaf);
        let int = |tw: TypeWord| TypeAnn {
            kind: TypeRef::Builtin(tw),
            span,
        };
        match self.cx.tree.kind(word_leaf) {
            x if x == self.cx.k.tok_int_kw => return int(TypeWord::Int),
            x if x == self.cx.k.tok_i32 => return int(TypeWord::I32),
            x if x == self.cx.k.tok_bool => return int(TypeWord::Bool),
            // PLAN-961: numeric type words (numeric trim only — gated
            // tokens cannot reach a type position in any other profile).
            x if self.cx.k.tok_i8_kw == Some(x) => return int(TypeWord::I8),
            x if self.cx.k.tok_u8_kw == Some(x) => return int(TypeWord::U8),
            x if self.cx.k.tok_byte_kw == Some(x) => return int(TypeWord::Byte),
            x if self.cx.k.tok_uint_kw == Some(x) => return int(TypeWord::Uint),
            x if self.cx.k.tok_u32_kw == Some(x) => return int(TypeWord::U32),
            x if self.cx.k.tok_i64_kw == Some(x) => return int(TypeWord::I64),
            x if self.cx.k.tok_u64_kw == Some(x) => return int(TypeWord::U64),
            x if self.cx.k.tok_float_kw == Some(x) || self.cx.k.tok_f32_kw == Some(x) => {
                return int(TypeWord::Float)
            }
            x if self.cx.k.tok_double_kw == Some(x) => return int(TypeWord::Double),
            x if self.cx.k.tok_f64_kw == Some(x) => return int(TypeWord::F64),
            _ => {}
        }
        let word = self.cx.spelling(word_leaf);
        // PLAN-935（T-02）：instances 面类型位裸 ident = 声明类型变量；PLAN-963
        // 把同一规则扩到「当前声明带类型参数」的任意面（generic record 的字段/
        // 方法注解）。仍只在拼写匹配当前声明参数表时成立——同名 record/局部不
        // 误绑，未匹配的 ident 走下方具名拒绝，不猜默认类型）。
        if self.cx.ty_params.iter().any(|t| t == &word) {
            return TypeAnn {
                kind: TypeRef::TyVar(Ident { name: word, span }),
                span,
            };
        }
        let is_text = self.cx.profile.is_text();
        // PLAN-928（T-04，镜像旧 collections_type）：collections 面 `Str`
        // → StrOwned；`char` 仅 closures（909 共享 cell 值域）→ Char；小写
        // `str`/`char` 具名拒绝。text 面维持 str/Str/char 三词。
        if self.cx.profile.is_collections() {
            match word.as_str() {
                // PLAN-982（aggregates 文本入口裁定，2026-11-19）：aggregates
                // 面**函数参数位**的借用 `str`（受检文本入口专属面——
                // `VT::Str` ⇒ MIR `StrView`）；字段/返回位与其余 collections
                // 面维持具名拒绝，入口专属纪律由车道门把守。
                "str" if ret_pos && self.cx.profile.is_auto_language() => {
                    return int(TypeWord::Str)
                }
                "str" if param_pos && self.cx.profile.is_aggregates() => return int(TypeWord::Str),
                // The ordinary face accepts the legacy spelling at owned
                // storage positions. Binding retains the spelling to identify
                // legacy record layouts before normalizing the field type.
                "str" if stored && self.cx.profile.is_auto_language() => return int(TypeWord::Str),
                "Str" => return int(TypeWord::StrOwned),
                "char" if self.cx.profile.is_closures() => return int(TypeWord::Char),
                "str" | "char" => {
                    let w = word.clone();
                    self.cx.reject(span, move || unsupported_ty(&w, span));
                    return int(TypeWord::Int);
                }
                _ => {}
            }
        }
        let admits_str = is_text;
        if admits_str && matches!(word.as_str(), "str" | "Str" | "char") {
            let tw = match word.as_str() {
                "str" => TypeWord::Str,
                "Str" => TypeWord::StrOwned,
                _ => TypeWord::Char,
            };
            return int(tw);
        }
        if !ret_pos
            && !admits_str
            && !self.cx.profile.is_collections()
            && matches!(word.as_str(), "str" | "Str" | "char")
        {
            self.cx.reject(span, move || unsupported_ty(&word, span));
            return int(TypeWord::Int);
        }
        if (self.cx.profile.is_entry_unit() || self.cx.profile.is_auto_language()) && word == "void"
        {
            return int(TypeWord::Void);
        }
        if self.cx.profile.is_record() {
            return TypeAnn {
                kind: TypeRef::Record(Ident { name: word, span }),
                span,
            };
        }
        self.cx.reject(span, move || unsupported_ty(&word, span));
        int(TypeWord::Int)
    }

    /// 节点命中的规则名（无命中给空串——调用方按名分派）。
    fn rule_name_of(&self, node: u32) -> String {
        let kind = self.cx.tree.kind(node);
        self.cx
            .p
            .rules
            .iter()
            .enumerate()
            .find(|(i, _)| ace_syntax_runtime::program::RULE_KIND_BASE + *i as u32 == kind)
            .map(|(_, r)| r.name.clone())
            .unwrap_or_default()
    }

    /// PLAN-981（G2，2026-11-18 裁定）：`age-on-stmt` 节点序列 → **源序**的
    /// `ComponentStmt` 面（赋值 / hook-local `let` / `if-else`——后两者只在
    /// age-package 裁剪存在；其它裁剪里这些节点不出现）。`if` 的 then/else
    /// 体递归走同一函数；类型注解按 `ann-ty` 词法分类（裸 ident → 名义
    /// record，如 `Sweep2D`；无注解交共享 resolver 推断）。
    /// PLAN-980 on-if 刀：`age-view-if-stmt` 节点 → `ComponentStmt::If`
    /// （view-stmt 方言体——带点赋值/嵌套 if/调用形；调用形置 foreign——
    /// fail-closed 不做部分体）。assigns 深平铺由调用方经
    /// `flatten_stmt_assigns` 完成。
    fn project_view_if(
        &mut self,
        inner: u32,
        channel_names: &[String],
        foreign: &mut bool,
    ) -> crate::source::age_component::ComponentStmt {
        let cond_node = self.cx.field_child(inner, "cond").expect("view-if cond");
        let cond = self.project_expr_sync(cond_node);
        let then = self.project_view_stmt_children(inner, channel_names, foreign);
        let els = self.cx.field_child(inner, "els").map(|e| {
            if let Some(chain) = self.cx.field_child(e, "chain") {
                // else-if 链：单一嵌套条件（递归）。
                vec![self.project_view_if(chain, channel_names, foreign)]
            } else {
                self.project_view_stmt_children(e, channel_names, foreign)
            }
        });
        crate::source::age_component::ComponentStmt::If {
            cond,
            then,
            els,
            span: self.cx.span_of(inner),
        }
    }

    /// view-if 体（age-view-stmt 子节点，声明序）→ 结构化语句面（赋值 +
    /// 嵌套 if；调用形置 foreign）。
    fn project_view_stmt_children(
        &mut self,
        node: u32,
        channel_names: &[String],
        foreign: &mut bool,
    ) -> Vec<crate::source::age_component::ComponentStmt> {
        use crate::source::age_component::ComponentStmt;
        let stmt_kind = self.rule_kind_of("age-view-stmt");
        let if_kind = self.rule_kind_of("age-view-if-stmt");
        let mut out: Vec<ComponentStmt> = Vec::new();
        let mut sink: Vec<ComponentAssign> = Vec::new();
        for ch in self.cx.children(node) {
            if Some(self.cx.tree.kind(ch)) != stmt_kind {
                continue;
            }
            let inner = self.unwrap_child(ch);
            if Some(self.cx.tree.kind(inner)) == if_kind {
                out.push(self.project_view_if(inner, channel_names, foreign));
            } else if self.cx.field_child(inner, "field").is_some() {
                sink.clear();
                self.collect_view_stmt(inner, &mut sink);
                if let Some(a) = sink.last() {
                    out.push(ComponentStmt::Assign(a.clone()));
                }
            } else if self.cx.field_child(inner, "call").is_some() {
                *foreign = true;
            } else {
                *foreign = true;
            }
        }
        out
    }

    fn project_on_stmts(
        &mut self,
        nodes: &[u32],
    ) -> Vec<crate::source::age_component::ComponentStmt> {
        // 并集顺手闭（F-R21-3）：981 提取 helper 时遗留的孤儿 import
        //（增量编译不重发警告，复审时漏网——本提交触该文件一并清）。
        let mut out = Vec::new();
        for &n in nodes {
            // 每个 `age-on-stmt` 恰有一个内层语句节点（assign/let/if）。
            let inner = self
                .cx
                .children(n)
                .into_iter()
                .find(|&c| self.cx.tree.node(c).flags & ace_syntax_runtime::cst::FLAG_TRIVIA == 0)
                .unwrap_or(n);
            if let Some(st) = self.project_on_stmt_inner(inner) {
                out.push(st);
            }
        }
        out
    }

    fn project_handler_loop_body(&mut self, node: u32) -> Vec<crate::source::age_component::ComponentStmt> {
        let body: Vec<_> = self.cx.children(node).into_iter()
            .filter(|&c| self.rule_name_of(c) == "age-on-branch-stmt")
            .collect();
        self.project_on_stmts(&body)
    }

    fn project_handler_assignment(&self, assign: ComponentAssign) -> ComponentStmt {
        if self.cx.profile == SourceProfile::AgePackage && assign.bare {
            ComponentStmt::AssignLocal {
                name: Ident { name: assign.field, span: assign.span },
                value: assign.rhs,
                span: assign.span,
            }
        } else {
            ComponentStmt::Assign(assign)
        }
    }

    /// Project one inner handler statement. Widget host calls are collected
    /// separately by the caller.
    fn project_on_stmt_inner(
        &mut self,
        inner: u32,
    ) -> Option<crate::source::age_component::ComponentStmt> {
        use crate::source::age_component::{ComponentAssign, ComponentStmt};
        match self.rule_name_of(inner).as_str() {
            // PLAN-981（on-if 组件面刀）：widget 形点前缀赋值（分支体两拼写
            // 之一）——复用 view 语句赋值模型（复合赋值脱糖同一条）。
            "age-view-assign-stmt" => {
                let mut tmp = Vec::new();
                self.collect_view_stmt(inner, &mut tmp);
                tmp.pop().map(|assign| self.project_handler_assignment(assign))
            }
            // PLAN-105 T-02（U03 真语料 013）：C 风格条件循环 `for <cond> { … }`。
            "age-on-for-cond" => {
                let cond_node = self.cx.field_child(inner, "cond")?;
                let cond = self.project_expr_sync(cond_node);
                let body_children: Vec<u32> = self
                    .cx
                    .children(inner)
                    .into_iter()
                    .filter(|&c| self.rule_name_of(c) == "age-on-branch-stmt")
                    .collect();
                let body = self.project_on_stmts(&body_children);
                Some(ComponentStmt::While {
                    cond,
                    body,
                    span: self.cx.span_of(inner),
                })
            }
            // PLAN-105 T-02：调用语句（`store.M(args)` 等）结构入候选；
            // store 方法在 store 内联时展开，其余调用维持 body_foreign 合同。
            "age-view-call-stmt" => {
                let call_node = self.cx.field_child(inner, "call")?;
                let call = self.project_expr_sync(call_node);
                Some(ComponentStmt::Call {
                    call,
                    span: self.cx.span_of(inner),
                })
            }
            "age-on-assign" => {
                let field_leaf = self
                    .cx
                    .field_child(inner, "field")
                    .expect("on assign field name");
                let rhs_node = self.cx.field_child(inner, "rhs").expect("on assign rhs");
                let field_name = self.cx.spelling(field_leaf);
                let span = self.cx.span_of(inner);
                let mut rhs = self.project_expr_sync(rhs_node);
                // PLAN-105 T-02：复合赋值脱糖为二元（裸名目标=Var，执行期按
                // 绑定表解析为局部量或字段——与 981 遮蔽语义同源）。
                if let Some(op_node) = self.cx.field_child(inner, "op") {
                    let op_rule = self.rule_name_of_kind(self.cx.tree.kind(self.unwrap_child(op_node)));
                    if op_rule == "age-op-plus-assign" || op_rule == "age-op-minus-assign" {
                        let op = if op_rule == "age-op-plus-assign" {
                            crate::source::ast::BinOp::Add
                        } else {
                            crate::source::ast::BinOp::Sub
                        };
                        rhs = Expr {
                            span,
                            kind: ExprKind::Binary {
                                op,
                                lhs: Box::new(Expr {
                                    span,
                                    kind: ExprKind::Var(Ident {
                                        name: field_name.clone(),
                                        span,
                                    }),
                                }),
                                rhs: Box::new(rhs),
                            },
                        };
                    }
                }
                if self.cx.profile == SourceProfile::AgePackage {
                    Some(ComponentStmt::AssignLocal {
                        name: Ident { name: field_name, span: self.cx.span_of(field_leaf) },
                        value: rhs,
                        span,
                    })
                } else {
                    Some(ComponentStmt::Assign(ComponentAssign {
                        field: field_name,
                        bare: true,
                        rhs,
                        span,
                    }))
                }
            }
            "age-on-let" | "age-on-var" => {
                let (let_node, mutable) = if self.rule_name_of(inner) == "age-on-var" {
                    (inner, true)
                } else {
                    let variant = self.unwrap_child(inner);
                    let tail = self.cx.children(variant).into_iter()
                        .find(|&child| self.rule_name_of(child) == "age-on-let-tail")
                        .expect("on let binding tail");
                    (tail, self.rule_name_of(variant) == "age-on-let-var")
                };
                let name_leaf = self.cx.field_child(let_node, "name").expect("on let name");
                let ann = self
                    .cx
                    .field_child(let_node, "ann")
                    // 增量同步适配（982 aggregates 文本入口刀）：classify_word
                    // 增第三参 param_pos——hook-local let 注解位非入口参数位，
                    // 恒 false（982 主线同点位同值）。
                    .map(|a| self.classify_word(a, false, false));
                let value_node = self
                    .cx
                    .field_child(let_node, "value")
                    .expect("on let value");
                Some(ComponentStmt::Let {
                    name: ident_of(&mut self.cx, name_leaf),
                    mutable,
                    ann,
                    value: self.project_expr_sync(value_node),
                    span: self.cx.span_of(inner),
                })
            }
            "age-on-while" => {
                let cond = self.cx.field_child(inner, "cond").expect("on while condition");
                let body = self.project_handler_loop_body(inner);
                Some(ComponentStmt::While {
                    cond: self.project_expr_sync(cond), body, span: self.cx.span_of(inner),
                })
            }
            "age-on-index-set" => {
                let span = self.cx.span_of(inner);
                let name = self.cx.field_child(inner, "receiver").expect("indexed local");
                let name = ident_of(&mut self.cx, name);
                let index = self.cx.field_child(inner, "index").expect("index");
                let value = self.cx.field_child(inner, "value").expect("index assignment value");
                let recv = Expr { span: name.span, kind: ExprKind::Var(name) };
                Some(ComponentStmt::Effect {
                    value: Expr {
                        kind: ExprKind::Method {
                            recv: Box::new(recv), name: Ident { name: "set".into(), span },
                            args: vec![Arg::Pos(self.project_expr_sync(index)), Arg::Pos(self.project_expr_sync(value))],
                        },
                        span,
                    },
                    span,
                })
            }
            "age-on-for" | "age-component-on-for" => {
                let name = self.cx.field_child(inner, "name").expect("on for binding");
                let iter = self.cx.field_child(inner, "iter").expect("on for iterable");
                let body = self.project_handler_loop_body(inner);
                Some(ComponentStmt::For {
                    name: ident_of(&mut self.cx, name),
                    iter: self.project_expr_sync(iter),
                    body,
                    span: self.cx.span_of(inner),
                })
            }
            "age-on-push" | "age-on-pop" => {
                let span = self.cx.span_of(inner);
                let name = self.cx.field_child(inner, "receiver").expect("list effect receiver");
                let recv = ident_of(&mut self.cx, name);
                let receiver_span = recv.span;
                let kind = if let Some(dot) = self.cx.field_child(inner, "model") {
                    let dot_span = self.cx.span_of(dot);
                    ExprKind::Field {
                        base: Box::new(Expr {
                            kind: ExprKind::Var(Ident { name: "self".into(), span: dot_span }),
                            span: dot_span,
                        }),
                        name: recv,
                    }
                } else {
                    ExprKind::Var(recv)
                };
                let receiver = Expr { kind, span: receiver_span };
                let method_node = self.cx.field_child(inner, "method").expect("list effect method");
                let method = ident_of(&mut self.cx, method_node);
                let call = |name: Ident, args| Expr {
                    kind: ExprKind::Method { recv: Box::new(receiver.clone()), name, args },
                    span,
                };
                if let Some(arg) = self.cx.field_child(inner, "arg") {
                    return Some(ComponentStmt::Effect {
                        value: call(method, vec![Arg::Pos(self.project_expr_sync(arg))]),
                        span,
                    });
                }
                // Discarding a pop on an empty stack leaves it empty. Lower
                // to the same checked list operations used by normal reads
                // and removals. The receiver is a named local or model field.
                let ident = |name: &str| Ident { name: name.into(), span: method.span };
                let len = call(ident("len"), Vec::new());
                let number = |n| Expr { kind: ExprKind::IntLit(n), span: method.span };
                let binary = |op, lhs, rhs| Expr {
                    kind: ExprKind::Binary { op, lhs: Box::new(lhs), rhs: Box::new(rhs) },
                    span,
                };
                Some(ComponentStmt::If {
                    cond: binary(BinOp::Gt, len.clone(), number(0)),
                    then: vec![ComponentStmt::Let {
                        name: ident("__age_p_hidden_popped"),
                        mutable: false,
                        ann: None,
                        value: call(ident("remove"), vec![Arg::Pos(binary(BinOp::Sub, len, number(1)))]),
                        span,
                    }],
                    els: None,
                    span,
                })
            }
            "age-on-message" => {
                let message = self.cx.field_child(inner, "message").expect("handler message");
                let mut values = Vec::new();
                if let Some(list) = self.cx.field_child(inner, "args") {
                    values.push(self.cx.field_child(list, "value").expect("first message argument"));
                    let tail = self.rule_kind_of("age-on-message-arg-tail");
                    values.extend(
                        self.cx.children(list).into_iter()
                            .filter(|&item| Some(self.cx.tree.kind(item)) == tail)
                            .map(|item| self.cx.field_child(item, "value").expect("message argument")),
                    );
                }
                Some(ComponentStmt::Send {
                    message: self.cx.spelling(message),
                    args: values.into_iter().map(|value| self.project_expr_sync(value)).collect(),
                    span: self.cx.span_of(inner),
                })
            }
            "age-on-if" => {
                let cond_node = self.cx.field_child(inner, "cond").expect("on if cond");
                let cond = self.project_expr_sync(cond_node);
                // PLAN-981（on-if 组件面刀）：分支体语句包装两面拼写
                //（age-on-stmt 裸名 / age-on-branch-stmt 并集——点前缀
                // 赋值经 helper 的 age-view-assign-stmt 臂）。
                let branch_kind = self.rule_kind_of("age-on-branch-stmt");
                let stmt_wrapper_kind = self.rule_kind_of("age-on-stmt");
                let is_branch = |k: u32| Some(k) == branch_kind || Some(k) == stmt_wrapper_kind;
                let then_nodes: Vec<u32> = self
                    .cx
                    .children(inner)
                    .into_iter()
                    .filter(|&c| is_branch(self.cx.tree.kind(c)))
                    .collect();
                let then = self.project_on_stmts(&then_nodes);
                let els = self.cx.field_child(inner, "els").map(|e| {
                    // F-R21-3 并集：else-if 链臂（镜像 980 的 age-view-if-else
                    // chain——本刀前 age-on-else 无链形，980 已交付的链式作者
                    // 面不能因换 union 规则回归）= 单一嵌套条件（递归）。
                    if let Some(chain) = self.cx.field_child(e, "chain") {
                        match self.project_on_stmt_inner(chain) {
                            Some(st) => vec![st],
                            None => Vec::new(),
                        }
                    } else {
                        let else_nodes: Vec<u32> = self
                            .cx
                            .children(e)
                            .into_iter()
                            .filter(|&c| is_branch(self.cx.tree.kind(c)))
                            .collect();
                        self.project_on_stmts(&else_nodes)
                    }
                });
                Some(ComponentStmt::If {
                    cond,
                    then,
                    els,
                    span: self.cx.span_of(inner),
                })
            }
            _ => None,
        }
    }

    /// PLAN-928（T-03）：PLAN-864 有限 enum 投影（臂声明序 = 判别序）。
    fn run_enum_item(&mut self, node: u32) -> EnumItem {
        let name_leaf = self.cx.field_child(node, "name").expect("enum name");
        let name = ident_of(&mut self.cx, name_leaf);
        let type_params = self.project_type_params(self.cx.field_child(node, "type_params"));
        let saved = std::mem::take(&mut self.cx.ty_params);
        self.cx.ty_params = type_params.iter().map(|p| p.name.name.clone()).collect();
        for param in &type_params {
            if param.kind == TypeParamKind::Const {
                self.cx.reject(param.name.span, || {
                    Diagnostic::new(
                        Stage::Text,
                        "source.instances-unsupported",
                        "enum parameters are types",
                        param.name.span,
                    )
                });
            }
        }
        // PLAN-1083: arms ride the enum-member wrapper (transparent for the
        // frozen profiles — projection only looks at arm/method children);
        // auto-face enum bodies may carry view-method members.
        let member_kind = self.cx.k.enum_member.unwrap_or(u32::MAX);
        let arm_kind = self.cx.k.enum_arm.unwrap_or(u32::MAX);
        let mut member_nodes: Vec<u32> = self
            .cx
            .children(node)
            .into_iter()
            .filter(|&c| self.cx.tree.kind(c) == member_kind || self.cx.tree.kind(c) == arm_kind)
            .collect();
        member_nodes.sort_by_key(|&n| self.cx.tree.abs_range(n).0);
        let mut arms = Vec::new();
        let mut methods = Vec::new();
        for m in member_nodes {
            if self.cx.tree.kind(m) == arm_kind {
                arms.push(self.run_enum_arm(m));
                continue;
            }
            if let Some(a) = self.cx.field_child(m, "arm") {
                arms.push(self.run_enum_arm(a));
            } else if let Some(f) = self.cx.field_child(m, "method") {
                methods.push(self.run_fn_item(f, Some(name.clone()), Receiver::View));
            }
        }
        self.cx.ty_params = saved;
        if !type_params.is_empty() {
            if let Some(arm) = arms.iter().find(|arm| arm.disc.is_some()) {
                self.cx.reject(arm.span, || {
                    Diagnostic::new(
                        Stage::Text,
                        "source.instances-unsupported",
                        "a generic enum cannot declare integer discriminants",
                        arm.span,
                    )
                });
            }
        }
        EnumItem {
            name,
            type_params,
            instance: None,
            spec_witnesses: false,
            arms,
            methods,
            span: self.cx.span_of(node),
        }
    }

    /// One enum arm (PLAN-1083: tuple-tail payloads fold into
    /// `TypeRef::Tuple`; named-field payloads carry `fields`; `= N` carries
    /// `disc`).
    fn run_enum_arm(&mut self, a: u32) -> EnumArm {
        let aname_leaf = self.cx.field_child(a, "name").expect("arm name");
        let aname = ident_of(&mut self.cx, aname_leaf);
        // Named-field payload `Arm { f int, ... }` — the field members in
        // declaration order (resolve interns the synthesized record).
        let field_kind = self.cx.k.field_member;
        let mut fields: Vec<(Ident, TypeAnn)> = Vec::new();
        let mut fnodes: Vec<u32> = self
            .cx
            .children(a)
            .into_iter()
            .filter(|&c| self.cx.tree.kind(c) == field_kind)
            .collect();
        fnodes.sort_by_key(|&n| self.cx.tree.abs_range(n).0);
        for f in fnodes {
            let fname_leaf = self.cx.field_child(f, "name").expect("field name");
            let fname = ident_of(&mut self.cx, fname_leaf);
            let fty_leaf = self.cx.field_child(f, "ty").expect("field type");
            let fty = self.classify_word(fty_leaf, false, false);
            fields.push((fname, fty));
        }
        // Explicit scalar discriminant `Arm = N`.
        let disc = self
            .cx
            .field_child(a, "disc")
            .map(|l| self.cx.spelling(l).parse::<i32>().unwrap_or(0));
        // Paren payload `Arm(T)` — plus auto-face tuple tails folding into
        // a multi-element `TypeRef::Tuple` payload.
        let mut payload = self
            .cx
            .field_child(a, "payload")
            .map(|t| self.classify_storage_word(t));
        let tail_kind = self.cx.k.enum_payload_tail.unwrap_or(u32::MAX);
        let mut tails: Vec<u32> = self
            .cx
            .children(a)
            .into_iter()
            .filter(|&c| self.cx.tree.kind(c) == tail_kind)
            .collect();
        if !tails.is_empty() {
            tails.sort_by_key(|&n| self.cx.tree.abs_range(n).0);
            let mut elems = Vec::new();
            if let Some(head) = payload.take() {
                elems.push(head);
            }
            let ann_kind = self.cx.k.ann_ty;
            for t in tails {
                if let Some(ty) = self
                    .cx
                    .children(t)
                    .into_iter()
                    .find(|&c| self.cx.tree.kind(c) == ann_kind)
                {
                    elems.push(self.classify_word(ty, false, false));
                }
            }
            let span = self.cx.span_of(a);
            payload = Some(TypeAnn {
                kind: TypeRef::Tuple(elems),
                span,
            });
        }
        let span = match (&payload, disc, fields.first()) {
            (Some(t), _, _) => aname.span.join(t.span),
            (_, Some(_), _) => aname.span,
            (_, _, Some((_, ty))) => aname.span.join(ty.span),
            _ => aname.span,
        };
        EnumArm {
            name: aname,
            payload,
            fields,
            disc,
            span,
        }
    }

    fn project_instance_arguments(&mut self, group: u32) -> InstanceArguments {
        let mut types = Vec::new();
        let mut consts = Vec::new();
        let kind = self
            .rule_kind_of("instance-arg")
            .expect("instance argument kind");
        for item in collect_seq(&self.cx, group, kind, "instance-arg-tail") {
            if let Some(leaf) = self.cx.field_child(item, "lit") {
                consts.push(self.project_const_argument(leaf));
            } else if let Some(ty) = self.cx.field_child(item, "ty") {
                types.push(self.classify_storage_word(ty));
            }
        }
        InstanceArguments { types, consts }
    }

    fn project_call_arguments(&mut self, c: u32) -> Vec<Arg> {
        let mut args: Vec<Arg> = Vec::new();
        if let Some(an) = self.cx.field_child(c, "args") {
            for a in collect_seq(&self.cx, an, self.cx.k.arg, "arg-tail") {
                if let Some(named) = self.cx.child_of_kind(a, self.cx.k.named_arg) {
                    let label_leaf = self.cx.field_child(named, "label").expect("arg label");
                    let label = ident_of(&mut self.cx, label_leaf);
                    let value_n = self.cx.field_child(named, "value").expect("arg value");
                    let vspan = self.cx.span_of(value_n);
                    let value = self.project_expr_sync(value_n);
                    args.push(Arg::Named {
                        name: label,
                        value,
                        span: vspan,
                    });
                } else if let Some(value_n) = self.cx.child_of_kind(a, self.cx.k.expr) {
                    args.push(Arg::Pos(self.project_expr_sync(value_n)));
                }
            }
        }
        args
    }

    // Value arguments have u32 identity but materialize into an i32 body.
    fn project_const_argument(&mut self, leaf: u32) -> ConstArgUse {
        let span = self.cx.span_of(leaf);
        let text = self.cx.spelling(leaf);
        let value = match text.parse::<u64>() {
            Ok(value) if value <= i32::MAX as u64 => value,
            value => {
                let domain = if value.is_ok_and(|n| n <= u32::MAX as u64) {
                    "exceeds the i32 body domain"
                } else {
                    "is outside the u32 domain"
                };
                self.cx.reject(span, move || {
                    Diagnostic::new(
                        Stage::Text,
                        "source.literal-range",
                        format!("value argument `{text}` {domain}"),
                        span,
                    )
                });
                0
            }
        };
        ConstArgUse { value, span }
    }

    /// PLAN-928（T-03）：类型位的参数化名义 `List<T>` / `Result<T, E>`。
    fn project_generic_ty(&mut self, node: u32) -> TypeAnn {
        let name_leaf = self
            .cx
            .field_child(node, "name")
            .expect("generic type name");
        let name = ident_of(&mut self.cx, name_leaf);
        let mut args = Vec::new();
        if let Some(an) = self.cx.field_child(node, "args") {
            for a in collect_seq(
                &self.cx,
                an,
                self.cx.k.type_arg.unwrap_or(u32::MAX),
                "type-arg-tail",
            ) {
                let arg = if matches!(name.name.as_str(), "List" | "Iter")
                    && self.cx.profile.is_auto_language()
                {
                    // Collection and stream payloads are owned. Keep the
                    // legacy spelling for storage normalization in binding.
                    self.classify_word(a, false, true)
                } else {
                    self.classify_ty_arg(a)
                };
                args.push(arg);
            }
        }
        TypeAnn {
            span: self.cx.span_of(node),
            kind: TypeRef::Generic { name, args },
        }
    }

    /// PLAN-928（T-03）：`?T` 可选糖 → `TypeRef::Optional`。
    fn project_optional_ty(&mut self, node: u32) -> TypeAnn {
        let inner = match self.cx.field_child(node, "inner") {
            Some(i) => self.classify_storage_word(i),
            None => self.classify_word(node, false, false),
        };
        TypeAnn {
            span: self.cx.span_of(node),
            kind: TypeRef::Optional(Box::new(inner)),
        }
    }

    /// PLAN-965 (M2-A): `~Iter<T>` generator stream type -> `TypeRef::Stream`.
    fn project_tilde_ty(&mut self, node: u32) -> TypeAnn {
        if let Some(inner) = self.cx.field_child(node, "inner") {
            let span = self.cx.span_of(node);
            return TypeAnn {
                kind: TypeRef::Generic {
                    name: Ident {
                        name: "Future".into(),
                        span,
                    },
                    args: vec![self.classify_storage_word(inner)],
                },
                span,
            };
        }
        let g = self.cx.field_child(node, "generic").expect("tilde generic");
        let inner = match self.project_generic_ty(g) {
            TypeAnn {
                kind: TypeRef::Generic { name, mut args },
                ..
            } if name.name == "Iter" && args.len() == 1 => args.remove(0),
            other if self.cx.profile.is_auto_language() => {
                let span = self.cx.span_of(node);
                return TypeAnn {
                    kind: TypeRef::Generic {
                        name: Ident {
                            name: "Future".into(),
                            span,
                        },
                        args: vec![other],
                    },
                    span,
                };
            }
            other => other,
        };
        TypeAnn {
            span: self.cx.span_of(node),
            kind: TypeRef::Stream(Box::new(inner)),
        }
    }

    /// PLAN-965 (M2-A): `!T` error type sugar -> `TypeRef::ResultBang`.
    fn project_err_ty(&mut self, node: u32) -> TypeAnn {
        let inner = match self.cx.field_child(node, "inner") {
            Some(i) => self.classify_word(i, false, false),
            None => self.classify_word(node, false, false),
        };
        TypeAnn {
            span: self.cx.span_of(node),
            kind: TypeRef::ResultBang(Box::new(inner)),
        }
    }

    /// PLAN-928（T-03）：类型实参（词/嵌套泛型/可选糖）。
    fn classify_ty_arg(&mut self, n: u32) -> TypeAnn {
        if let Some(g) = self.cx.field_child(n, "generic") {
            return self.project_generic_ty(g);
        }
        if let Some(o) = self.cx.field_child(n, "optional") {
            return self.project_optional_ty(o);
        }
        self.classify_storage_word(n)
    }

    /// PLAN-928：函数类型 `fn(T0, ...) [->] R`（嵌套允许；`->` 缺席 = K 语料
    /// 相邻结果拼写兼容形）。参数/结果经 classify_word（其 fn 分支承载嵌套）。
    fn project_fn_type(&mut self, node: u32) -> TypeAnn {
        let span = self.cx.span_of(node);
        let mut params = Vec::new();
        if let Some(pn) = self.cx.field_child(node, "params") {
            let plist: Vec<u32> = collect_seq(
                &self.cx,
                pn,
                self.cx.k.fn_type_param.unwrap_or(u32::MAX),
                "fn-type-param-tail",
            );
            for p in plist {
                params.push(self.classify_word(p, false, false));
            }
        }
        // 结果：fn-type-result 节点（词或嵌套 fn 型——classify_word 的 fn
        // 分支处理嵌套；词分支按拼写+profile 分类）。
        let result = match self.cx.field_child(node, "result") {
            Some(res) => self.classify_word(res, true, false),
            None => self.classify_word(node, true, false),
        };
        TypeAnn {
            span,
            kind: TypeRef::Function {
                params,
                result: Box::new(result),
            },
        }
    }

    /// PLAN-960（T-02）：module-level `const NAME: T = expr` / `var` 项投影
    /// （packages-modules 面；名字/注解/初始化表达式同 let 语句机器）。
    fn run_global_item(&mut self, node: u32, mutable: bool) -> Item {
        let name_leaf = self.cx.field_child(node, "name").expect("global name");
        let name = ident_of(&mut self.cx, name_leaf);
        let ann = self
            .cx
            .field_child(node, "ty")
            .map(|n| self.classify_word(n, false, false))
            .unwrap_or(TypeAnn {
                kind: TypeRef::Inferred,
                span: name.span,
            });
        let value_n = self.cx.field_child(node, "value").expect("global value");
        self.stack.push(Frame::GlobalItemFinish {
            node,
            name,
            ann,
            mutable,
        });
        self.stack.push(Frame::ExprEnter(value_n));
        self.drive();
        self.global_item.take().expect("global item finished")
    }

    fn run_type_item(&mut self, node: u32) -> TypeItem {
        self.run_type_item_with_params(node, Vec::new())
    }

    fn run_type_item_with_params(
        &mut self,
        node: u32,
        attribute_params: Vec<TypeParamDecl>,
    ) -> TypeItem {
        let name_leaf = self.cx.field_child(node, "name").expect("type name");
        let name = ident_of(&mut self.cx, name_leaf);
        // PLAN-963（T-02）：声明序类型参数；成员注解投影期间这些名字按类型
        // 变量（`TypeRef::TyVar`）投影，随后由语法期单态化替换为具体类型。
        // PLAN-963（T-03）：参数可带 `has S` 约束——约束在单态化时按实参的
        // 遵从列表核验（不在 resolve 里再造一套 bound 求解）。
        // PLAN-963（T-03）：`type` 的尾部（类型参数/遵从列表/记录体）由
        // `type-tail` 子规则承载（与 `fn-item`+`fn-tail` 同形，别名形是其
        // 另一 alt）；投影统一从 tail 节点读。
        let tail = self.cx.field_child(node, "tail").unwrap_or(node);
        let declared = self.project_type_param_decls(self.cx.field_child(node, "type_params"));
        let type_params = self.merge_attribute_params(declared, attribute_params);
        // PLAN-963（T-03）：`as Spec` 遵从列表（投影期消化；记录本身保持普通
        // TypeItem，下游只见方法）。
        let conforms = self.project_specs(self.cx.field_child(tail, "specs"));
        let saved_ty_params = std::mem::take(&mut self.cx.ty_params);
        self.cx.ty_params = type_params.iter().map(|t| t.name.name.clone()).collect();
        let mut fields: Vec<FieldDecl> = Vec::new();
        let mut methods: Vec<FnItem> = Vec::new();
        let mut seen: Vec<String> = Vec::new();
        for c_raw in self.cx.children(tail) {
            // member 是分派节点：取首个非 trivia 孩子（field/static/view）。
            let c = self
                .cx
                .children(c_raw)
                .into_iter()
                .find(|&x| self.cx.tree.node(x).flags & FLAG_TRIVIA == 0)
                .unwrap_or(c_raw);
            let kind = self.cx.tree.kind(c);
            if kind == self.cx.k.field_member {
                if self.cx.field_child(c, "attribute").is_some() {
                    self.project_decl_attributes(c, c, true);
                }
                let fname_leaf = self.cx.field_child(c, "name").expect("field name");
                let fname = ident_of(&mut self.cx, fname_leaf);
                if fname.name == "self" {
                    let span = fname.span;
                    self.cx.reject(span, || {
                        Diagnostic::new(
                            Stage::Text,
                            "source.unsupported",
                            "`self` cannot be a field name",
                            span,
                        )
                    });
                }
                let ty =
                    self.classify_storage_word(self.cx.field_child(c, "ty").expect("field ty"));
                let span = fname.span.join(ty.span);
                if seen.contains(&fname.name) {
                    let (f, t) = (fname.name.clone(), name.name.clone());
                    let sp = fname.span;
                    self.cx.reject(sp, move || {
                        Diagnostic::new(
                            Stage::Text,
                            "source.duplicate",
                            format!("field `{}` is declared more than once in type `{}`", f, t),
                            sp,
                        )
                    });
                }
                seen.push(fname.name.clone());
                fields.push(FieldDecl {
                    span,
                    name: fname,
                    ty,
                });
            } else if let Some(method) = self.project_record_method(c, &name) {
                methods.push(method);
            }
        }
        if fields.is_empty() && !self.cx.profile.is_auto_language() {
            let span = self.cx.span_of(node);
            self.cx.reject(span, || {
                Diagnostic::new(
                    Stage::Text,
                    "source.unsupported",
                    "empty records are not part of this profile; a record must declare at least one field",
                    span,
                )
            });
        }
        self.cx.ty_params = saved_ty_params;
        TypeItem {
            span: self.cx.span_of(node),
            name,
            type_params,
            conforms,
            instance: None,
            spec_memberships: Vec::new(),
            fields,
            methods,
        }
    }

    /// PLAN-963（T-03）：`type Name = T` → 投影侧别名声明（不进 AST）。
    /// 目标按普通类型位置投影（`ann-ty`），展开由 `expand_type_aliases` 做。
    fn is_type_alias(&mut self, node: u32) -> bool {
        self.cx
            .field_child(node, "tail")
            .and_then(|t| self.cx.field_child(t, "alias"))
            .is_some()
    }

    fn run_type_alias_item(&mut self, node: u32) -> TypeAliasDecl {
        let name_leaf = self.cx.field_child(node, "name").expect("alias name");
        let name = ident_of(&mut self.cx, name_leaf);
        let target_node = self
            .cx
            .field_child(node, "tail")
            .and_then(|t| self.cx.field_child(t, "alias"))
            .expect("alias target");
        let target = self.classify_word(target_node, false, false);
        TypeAliasDecl { name, target }
    }

    /// PLAN-963（T-03）：`ext Type { members }` → 投影侧扩展块。只收方法
    /// （记录布局只能在声明处固定；字段出现在 ext 里具名拒）。
    fn run_ext_item(&mut self, node: u32) -> ExtDecl {
        let ty_leaf = self.cx.field_child(node, "ty").expect("ext type");
        let ty = ident_of(&mut self.cx, ty_leaf);
        let mut methods: Vec<FnItem> = Vec::new();
        for c_raw in self.cx.children(node) {
            let c = self
                .cx
                .children(c_raw)
                .into_iter()
                .find(|&x| self.cx.tree.node(x).flags & FLAG_TRIVIA == 0)
                .unwrap_or(c_raw);
            let kind = self.cx.tree.kind(c);
            if kind == self.cx.k.field_member {
                let span = self.cx.span_of(c);
                self.cx.reject(span, || {
                    Diagnostic::new(
                        Stage::Text,
                        "source.unsupported",
                        "`ext` blocks declare methods only; a record's layout is fixed at its declaration",
                        span,
                    )
                });
            } else if let Some(method) = self.project_record_method(c, &ty) {
                methods.push(method);
            }
        }
        ExtDecl { ty, methods }
    }

    fn run_spec_item(&mut self, node: u32) -> SpecDecl {
        let name_leaf = self.cx.field_child(node, "name").expect("spec name");
        let name = ident_of(&mut self.cx, name_leaf);
        // PLAN-963（T-03）：通用 spec `spec Storage<T>` 的类型参数。只在
        // `as Storage<int>` 的实例化中有意义；值参数在 spec 里没有可实例化
        // 语义，具名拒（不静默忽略）。
        let decls = self.project_type_param_decls(self.cx.field_child(node, "type_params"));
        let mut type_params: Vec<Ident> = Vec::new();
        for p in decls {
            match p.kind {
                TypeParamKind::Type => type_params.push(p.name),
                TypeParamKind::Const => {
                    let span = p.name.span;
                    self.cx.reject(span, || {
                        Diagnostic::new(
                            Stage::Text,
                            "source.unsupported",
                            "`spec` declarations accept type parameters only",
                            span,
                        )
                    });
                }
            }
        }
        // PLAN-963（T-05）：super-spec `spec Tool: Send + Sync`（书写序）。
        let mut supers: Vec<Ident> = Vec::new();
        if let Some(sn) = self.cx.field_child(node, "supers") {
            if let Some(head) = self.cx.field_child(sn, "head") {
                supers.push(ident_of(&mut self.cx, head));
            }
            let tail_kind = self.cx.k.super_spec_tail.unwrap_or(u32::MAX);
            for c in self.cx.children(sn) {
                if self.cx.tree.kind(c) != tail_kind {
                    continue;
                }
                if let Some(leaf) = self.cx.field_child(c, "head") {
                    supers.push(ident_of(&mut self.cx, leaf));
                }
            }
        }
        let member_kind = self.cx.k.spec_member.unwrap_or(u32::MAX);
        let mut methods: Vec<SpecMethodDecl> = Vec::new();
        let mut assoc: Vec<Ident> = Vec::new();
        for c in self.cx.children(node) {
            if self.cx.tree.kind(c) != member_kind {
                continue;
            }
            // PLAN-963（T-05）：关联类型声明 `type Item`（无方法形状）。
            if let Some(leaf) = self.cx.field_child(c, "assoc") {
                let aname = ident_of(&mut self.cx, leaf);
                if assoc.iter().any(|a| a.name == aname.name) {
                    let sp = aname.span;
                    let n = aname.name.clone();
                    self.cx.reject(sp, move || {
                        Diagnostic::new(
                            Stage::Text,
                            "source.duplicate",
                            format!("associated type `{n}` is declared more than once"),
                            sp,
                        )
                    });
                    continue;
                }
                assoc.push(aname);
                continue;
            }
            let mname_leaf = self.cx.field_child(c, "name").expect("spec method name");
            let mname = ident_of(&mut self.cx, mname_leaf);
            let params = self.project_params(self.cx.field_child(c, "params"));
            for prm in &params {
                if prm.name.name == "self" {
                    let span = prm.name.span;
                    self.cx.reject(span, || self_param(span));
                }
            }
            let ret =
                self.classify_word(self.cx.field_child(c, "ty").expect("spec ret"), true, false);
            // 默认体（`fn m(..) R { .. }`）按普通块投影；签名（无体）= 抽象方法。
            let body = match self.cx.field_child(c, "body") {
                Some(b) => {
                    self.enter_block(b);
                    self.drive();
                    Some(self.pop_b())
                }
                None => None,
            };
            let span = match &body {
                Some(b) => mname.span.join(b.span),
                None => mname.span.join(ret.span),
            };
            if methods
                .iter()
                .any(|m: &SpecMethodDecl| m.name.name == mname.name)
            {
                let sp = mname.span;
                let n = mname.name.clone();
                self.cx.reject(sp, move || {
                    Diagnostic::new(
                        Stage::Text,
                        "source.duplicate",
                        format!("spec method `{n}` is declared more than once"),
                        sp,
                    )
                });
            }
            methods.push(SpecMethodDecl {
                name: mname,
                params,
                ret,
                body,
                span,
            });
        }
        SpecDecl {
            name,
            span: self.cx.span_of(node),
            type_params,
            supers,
            assoc,
            methods,
        }
    }

    /// PLAN-963（T-03）：`as A, B` 遵从列表（书写序；空=无遵从声明）。
    fn project_specs(&mut self, node: Option<u32>) -> Vec<ConformRef> {
        let mut out = Vec::new();
        let Some(n) = node else { return out };
        if let Some(head) = self.cx.field_child(n, "head") {
            out.push(self.project_conform_ref(head));
        }
        let tail_kind = self.cx.k.as_spec_tail.unwrap_or(u32::MAX);
        for c in self.cx.children(n) {
            if self.cx.tree.kind(c) == tail_kind {
                if let Some(leaf) = self.cx.field_child(c, "head") {
                    out.push(self.project_conform_ref(leaf));
                }
            }
        }
        out
    }

    /// PLAN-963（T-03/T-05）：一条 `as S` / `as S<int>` / `as S<int, Item=str>`。
    fn project_conform_ref(&mut self, node: u32) -> ConformRef {
        let spec_node = self.cx.field_child(node, "spec").expect("spec name");
        let name = ident_of(&mut self.cx, spec_node);
        let mut args: Vec<TypeAnn> = Vec::new();
        let mut assoc: Vec<(Ident, TypeAnn)> = Vec::new();
        let spec_args_kind = self.cx.k.spec_args.unwrap_or(u32::MAX);
        let args_node = self
            .cx
            .child_of_kind(node, spec_args_kind)
            .and_then(|sa| self.cx.field_child(sa, "args"));
        if let Some(an) = args_node {
            for a in collect_seq(
                &self.cx,
                an,
                self.cx.k.spec_arg.unwrap_or(u32::MAX),
                "spec-arg-tail",
            ) {
                let ty_node = self.cx.field_child(a, "ty").expect("spec argument");
                let ty = self.classify_ty_arg(ty_node);
                let Some(bind) = self.cx.field_child(a, "assoc") else {
                    args.push(ty);
                    continue;
                };
                // PLAN-963（T-05）：命名绑定 `Item=str`——`=` 左侧必须是裸名，
                // 右侧是绑定类型；其它左值（`int=x` 等）具名拒，不静默当位置实参。
                let TypeRef::Record(key) = ty.kind else {
                    let sp = ty.span;
                    self.cx.reject(sp, move || {
                        Diagnostic::new(
                            Stage::Text,
                            "source.unsupported",
                            "an associated-type binding requires a bare name on the left of `=`",
                            sp,
                        )
                    });
                    continue;
                };
                let bound_node = self.cx.field_child(bind, "ty").expect("assoc binding type");
                let bound = self.classify_ty_arg(bound_node);
                assoc.push((key, bound));
            }
        }
        ConformRef { name, args, assoc }
    }

    fn run_script_item(&mut self, node: u32) -> FnItem {
        let start_span = first_leaf_span(&self.cx, node);
        self.stack.push(Frame::ScriptFinish { start_span });
        let stmts: Vec<u32> = self
            .cx
            .children(node)
            .into_iter()
            .filter(|&c| self.cx.tree.kind(c) == self.cx.k.stmt)
            .collect();
        let end = self.cx.span_of(node).end;
        self.stack.push(Frame::Block(BlockFrame {
            nodes: stmts,
            cursor: 0,
            stmts: Vec::new(),
            script_top: true,
            returns_tail: false,
            span: start_span.join(Span { start: end, end }),
        }));
        self.drive();
        match self.vals.pop() {
            Some(Val::F(f)) => f,
            _ => FnItem {
                name: Ident {
                    name: "main".to_string(),
                    span: start_span,
                },
                type_params: Vec::new(),
                params: Vec::new(),
                ret: TypeAnn {
                    kind: TypeRef::Builtin(TypeWord::Void),
                    span: start_span,
                },
                body: Block {
                    stmts: Vec::new(),
                    span: start_span,
                },
                span: start_span,
                receiver: Receiver::Free,
                owner: None,
                script: true,
                proto: false,
                test_attribute: None,
            },
        }
    }

    // ------------------------------------------------------------ blocks

    fn enter_block(&mut self, node: u32) {
        self.enter_block_with_tail(node, false);
    }

    fn enter_block_with_tail(&mut self, node: u32, returns_tail: bool) {
        let stmts: Vec<u32> = self
            .cx
            .children(node)
            .into_iter()
            .filter(|&c| self.cx.tree.kind(c) == self.cx.k.stmt)
            .collect();
        let span = self.cx.span_of(node);
        self.stack.push(Frame::Block(BlockFrame {
            nodes: stmts,
            cursor: 0,
            stmts: Vec::new(),
            script_top: false,
            returns_tail,
            span,
        }));
    }

    fn step_block(&mut self, mut frame: BlockFrame) {
        if frame.cursor > 0 {
            let s = self.pop_s();
            frame.stmts.push(s);
        }
        if frame.cursor >= frame.nodes.len() {
            let span = frame.span;
            self.vals.push(Val::B(Block {
                stmts: frame.stmts,
                span,
            }));
            return;
        }
        let stmt_n = frame.nodes[frame.cursor];
        let script_top = frame.script_top;
        frame.cursor += 1;
        let returns_tail = frame.returns_tail && frame.cursor == frame.nodes.len();
        self.stack.push(Frame::Block(frame));
        // Only a declared value function's root tail becomes a return.
        if returns_tail {
            if let Some(stmt) = self.cx.child_of_kind(stmt_n, self.cx.k.expr_stmt) {
                let value = self.cx.field_child(stmt, "value").expect("tail expression");
                let kw = self.cx.span_of(value);
                self.stack.push(Frame::ReturnFinish { kw });
                self.stack.push(Frame::ExprEnter(value));
                return;
            }
        }
        self.enter_stmt(stmt_n, script_top);
    }

    // ---------------------------------------------------- 语句级编译期（966）

    /// 下一臂入栈并投影其块（源序；`else` 臂 pattern=None 同样入队）。
    fn start_ct_arm(&mut self, mut ctx: Box<CtIsCtx>) {
        let (pattern, body_n) = ctx.pending.remove(0);
        self.stack.push(Frame::CtIsArmBlock { ctx, pattern });
        self.enter_block(body_n);
    }

    /// in-ct-expr 节点 → 共享 CT 标量表达式（与项位 #in 同一谓词域/同一映射；
    /// 越域按 Text 阶段具名拒，不静默成运行期表达式）。
    fn ct_expr(&mut self, node: u32) -> CtExpr {
        if Some(self.cx.tree.kind(node)) == self.cx.k.in_ct_expr {
            let inner = self
                .cx
                .children(node)
                .into_iter()
                .find(|&c| self.cx.tree.child_count(c) > 0)
                .unwrap_or(node);
            return self.ct_expr_node(inner);
        }
        self.ct_expr_node(node)
    }

    fn ct_expr_node(&mut self, node: u32) -> CtExpr {
        let kind = self.cx.tree.kind(node);
        let k = self.cx.k;
        let op = if kind == k.op_eq {
            Some(CtBinOp::Eq)
        } else if kind == k.op_ne {
            Some(CtBinOp::Ne)
        } else if kind == k.op_lt {
            Some(CtBinOp::Lt)
        } else if kind == k.op_le {
            Some(CtBinOp::Le)
        } else if kind == k.op_gt {
            Some(CtBinOp::Gt)
        } else if kind == k.op_ge {
            Some(CtBinOp::Ge)
        } else if kind == k.op_andand {
            Some(CtBinOp::And)
        } else if kind == k.op_oror {
            Some(CtBinOp::Or)
        } else {
            None
        };
        if let Some(op) = op {
            let kids: Vec<u32> = self
                .cx
                .children(node)
                .into_iter()
                .filter(|&c| self.cx.tree.node(c).flags & FLAG_TRIVIA == 0)
                .collect();
            if kids.len() < 2 {
                let span = self.cx.span_of(node);
                self.cx.reject(span, || {
                    Diagnostic::new(
                        Stage::Text,
                        "source.cte.in-expr",
                        "a compile-time comparison needs exactly two operands",
                        span,
                    )
                });
                return CtExpr::Int(0);
            }
            let lhs = self.ct_expr_node(kids[0]);
            let rhs = self.ct_expr_node(kids[kids.len() - 1]);
            let span = lhs.span().join(rhs.span());
            return CtExpr::Bin {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
                span,
            };
        }
        // primary：in-ct-unary 包 int-literal/true/false/ns-ref。取首个非
        // trivia 孩子（`true`/`false` 是 token 叶，无孩子——只按"有孩子"过滤
        // 会把 bool 字面量退回 unary 节点而误拒）。
        let inner = if Some(kind) == k.in_ct_unary {
            self.cx
                .children(node)
                .into_iter()
                .find(|&c| self.cx.tree.node(c).flags & FLAG_TRIVIA == 0)
                .unwrap_or(node)
        } else {
            node
        };
        let ik = self.cx.tree.kind(inner);
        if ik == k.int_literal {
            let mut value = String::new();
            let mut stack = vec![inner];
            while let Some(c) = stack.pop() {
                if self.cx.tree.kind(c) == k.tok_minus {
                    value.push('-');
                }
                if self.cx.tree.kind(c) == k.tok_int_lit {
                    value.push_str(&self.cx.spelling(c));
                }
                for g in self.cx.children(c).into_iter().rev() {
                    stack.push(g);
                }
            }
            return match value.parse::<i32>() {
                Ok(v) => CtExpr::Int(v),
                Err(_) => {
                    let span = self.cx.span_of(inner);
                    self.cx.reject(span, || {
                        Diagnostic::new(
                            Stage::Text,
                            "source.cte.in-domain",
                            format!("`{value}` is outside the i32 compile-time domain"),
                            span,
                        )
                    });
                    CtExpr::Int(0)
                }
            };
        }
        if ik == k.tok_true {
            return CtExpr::Bool(true);
        }
        if ik == k.tok_false {
            return CtExpr::Bool(false);
        }
        if Some(ik) == k.ns_ref {
            let head = self
                .cx
                .field_child(inner, "head")
                .map(|h| self.cx.spelling(h))
                .unwrap_or_default();
            let member = self
                .cx
                .field_child(inner, "member")
                .map(|m| self.cx.spelling(m));
            return CtExpr::Ref {
                head,
                member,
                span: self.cx.span_of(inner),
            };
        }
        let span = self.cx.span_of(inner);
        self.cx.reject(span, || {
            Diagnostic::new(
                Stage::Text,
                "source.cte.in-expr",
                "compile-time expressions accept int/bool literals and namespace names only",
                span,
            )
        });
        CtExpr::Int(0)
    }

    /// `#for` 的迭代域：分解形（start/end 字段）或融合单 token（`0..3` 被
    /// affix-num 连续字符集吞成单 token——按 `..` 拆分），与项位 in-range 同。
    fn ct_range(&mut self, iter: u32) -> (CtExpr, CtExpr) {
        if let Some(both) = self.cx.field_child(iter, "both") {
            if let Some(lit) = self.cx.field_child(both, "lit") {
                let text = self.cx.spelling(lit);
                let span = self.cx.span_of(lit);
                if let Some((a, b)) = text.split_once("..") {
                    if !a.is_empty() && !b.is_empty() && !b.contains("..") {
                        let atom = |t: &str, off: usize| -> CtExpr {
                            match t.parse::<i32>() {
                                Ok(v) => CtExpr::Int(v),
                                Err(_) => CtExpr::Ref {
                                    head: t.to_string(),
                                    member: None,
                                    span: Span {
                                        start: span.start + off,
                                        end: span.start + off + t.len(),
                                    },
                                },
                            }
                        };
                        return (atom(a, 0), atom(b, a.len() + 2));
                    }
                }
                self.cx.reject(span, || {
                    Diagnostic::new(
                        Stage::Text,
                        "source.cte.in-expr",
                        format!("`{text}` is not a compile-time range"),
                        span,
                    )
                });
                return (CtExpr::Int(0), CtExpr::Int(0));
            }
        }
        let start = self
            .cx
            .field_child(iter, "start")
            .map(|n| self.ct_expr(n))
            .unwrap_or(CtExpr::Int(0));
        let end = self
            .cx
            .field_child(iter, "end")
            .map(|n| self.ct_expr(n))
            .unwrap_or(CtExpr::Int(0));
        (start, end)
    }

    fn enter_if(&mut self, node: u32) {
        let cond_n = self.cx.field_child(node, "cond").expect("if cond");
        let then_n = self.cx.field_child(node, "then").expect("if then");
        let els_blk = self
            .cx
            .child_of_kind(node, self.cx.k.else_block)
            .and_then(|branch| {
                self.cx
                    .child_of_kind(branch, self.cx.k.block)
                    .or_else(|| self.cx.child_of_kind(branch, self.cx.k.if_stmt))
            });
        self.stack.push(Frame::IfCond {
            then_n,
            els_blk,
            node,
        });
        self.stack.push(Frame::ExprEnter(cond_n));
    }

    fn enter_stmt(&mut self, stmt_dispatch: u32, script_top: bool) {
        let inner = self
            .cx
            .children(stmt_dispatch)
            .into_iter()
            .find(|&c| {
                self.cx.tree.node(c).flags & FLAG_TRIVIA == 0
                    && self.cx.tree.node(c).flags & ace_syntax_runtime::cst::FLAG_ERROR == 0
            })
            .unwrap_or(stmt_dispatch);
        let kind = self.cx.tree.kind(inner);
        if kind == self.cx.k.let_stmt || kind == self.cx.k.var_stmt {
            let kw_kind = if kind == self.cx.k.let_stmt {
                self.cx.k.tok_let
            } else {
                self.cx.k.tok_var
            };
            let kw = self
                .cx
                .child_of_kind(inner, kw_kind)
                .map(|l| self.cx.span_of(l))
                .unwrap_or(Span { start: 0, end: 0 });
            let name_leaf = self.cx.field_child(inner, "name").expect("let name");
            let name = ident_of(&mut self.cx, name_leaf);
            console_name_check(&mut self.cx, &name);
            let ann = self
                .cx
                .field_child(inner, "ann")
                .map(|n| self.classify_word(n, false, false));
            let mutable = kind == self.cx.k.var_stmt;
            let value_n = self.cx.field_child(inner, "value").expect("let value");
            self.stack.push(Frame::LetFinish {
                node: inner,
                kw,
                name,
                ann,
                mutable,
            });
            self.stack.push(Frame::ExprEnter(value_n));
        } else if kind == self.cx.k.assign_stmt
            || kind == self.cx.k.path_assign_stmt.unwrap_or(u32::MAX)
        {
            let name_leaf = self.cx.field_child(inner, "name").expect("assign name");
            let name = ident_of(&mut self.cx, name_leaf);
            let mut path = Vec::new();
            if kind == self.cx.k.path_assign_stmt.unwrap_or(u32::MAX) {
                // 路径段标识符住在 `path-seg` 子节点内（`many { path-seg }`
                // 直接平铺），不在语句节点的直接 ident 孩子里。
                let seg_kind = self.cx.k.path_seg.unwrap_or(u32::MAX);
                let mut seg_nodes: Vec<u32> = self
                    .cx
                    .children(inner)
                    .into_iter()
                    .filter(|&c| self.cx.tree.kind(c) == seg_kind)
                    .collect();
                seg_nodes.sort_by_key(|&n| self.cx.tree.abs_range(n).0);
                for seg in seg_nodes {
                    if let Some(pl) = self.cx.field_child(seg, "path") {
                        let kind = self.cx.tree.kind(pl);
                        if kind == self.cx.k.tok_int_lit {
                            let span = self.cx.span_of(pl);
                            let name = self.cx.spelling(pl);
                            if name.parse::<i32>().is_err() {
                                self.cx.reject(span, || {
                                    Diagnostic::new(
                                        Stage::Text,
                                        "source.literal-range",
                                        "object integer key exceeds i32 range",
                                        span,
                                    )
                                });
                            }
                            path.push(Ident { name, span });
                        } else if kind == self.cx.k.tok_true || kind == self.cx.k.tok_false {
                            path.push(Ident {
                                name: self.cx.spelling(pl),
                                span: self.cx.span_of(pl),
                            });
                        } else {
                            path.push(ident_of(&mut self.cx, pl));
                        }
                    }
                }
            }
            let value_n = self.cx.field_child(inner, "value").expect("assign value");
            self.stack.push(Frame::AssignFinish { name, path });
            self.stack.push(Frame::ExprEnter(value_n));
        } else if kind == self.cx.k.let_hold_stmt.unwrap_or(u32::MAX) {
            // PLAN-962（T-08，L-07）：`let hold x as v { }`——整局部 hold。
            let name_leaf = self.cx.field_child(inner, "name").expect("hold name");
            let name = ident_of(&mut self.cx, name_leaf);
            let bind_leaf = self.cx.field_child(inner, "bind").expect("hold bind");
            let bind = ident_of(&mut self.cx, bind_leaf);
            let body_n = self.cx.field_child(inner, "body").expect("hold body");
            let span = self.cx.span_of(inner);
            self.stack.push(Frame::HoldFinish { name, bind, span });
            self.enter_block(body_n);
        } else if kind == self.cx.k.let_destr_stmt.unwrap_or(u32::MAX) {
            // PLAN-962（L-15）：`let (a, b, ...) = expr`——绑定名列表 + 值。
            let names_n = self.cx.field_child(inner, "names").expect("destr names");
            let mut names: Vec<Ident> = Vec::new();
            if let Some(h) = self.cx.field_child(names_n, "head") {
                names.push(ident_of(&mut self.cx, h));
            }
            for t in self.cx.children(names_n) {
                if let Some(id_leaf) = self.cx.field_child(t, "") {
                    let _ = id_leaf;
                }
                // tail idents are field-less children of destr-name-tail
                for c2 in self.cx.children(t) {
                    if self.cx.tree.kind(c2) == self.cx.k.tok_ident {
                        names.push(ident_of(&mut self.cx, c2));
                    }
                }
            }
            let value_n = self.cx.field_child(inner, "value").expect("destr value");
            let span = self.cx.span_of(inner);
            self.stack.push(Frame::LetDestructFinish { names, span });
            self.stack.push(Frame::ExprEnter(value_n));
        } else if kind == self.cx.k.index_assign_stmt.unwrap_or(u32::MAX) {
            // PLAN-962 (L-48)：`xs[i] = v`——base 名、index/value 两表达式。
            let name_leaf = self
                .cx
                .field_child(inner, "name")
                .expect("index assign name");
            let name = ident_of(&mut self.cx, name_leaf);
            let index_n = self
                .cx
                .field_child(inner, "index")
                .expect("index assign index");
            let value_n = self
                .cx
                .field_child(inner, "value")
                .expect("index assign value");
            let span = self.cx.span_of(inner);
            self.stack.push(Frame::IndexAssignFinish { name, span });
            self.stack.push(Frame::ExprEnter(value_n));
            self.stack.push(Frame::ExprEnter(index_n));
        } else if kind == self.cx.k.self_place_assign_stmt.unwrap_or(u32::MAX) {
            // PLAN-1083（auto 面）：隐式 self 字段写 `.f .. (op)= v`——与
            // place 链同构，base = 合成 `self` 局部，首字段是第一步。
            let dot_span = self
                .cx
                .children(inner)
                .into_iter()
                .find(|&l| self.cx.tree.kind(l) == self.cx.k.tok_dot)
                .map(|l| self.cx.span_of(l))
                .unwrap_or_else(|| self.cx.span_of(inner));
            let name = Ident {
                name: "self".to_string(),
                span: dot_span,
            };
            let field_leaf = self
                .cx
                .field_child(inner, "field")
                .expect("self place first field");
            let first = PlaceStep::Field(ident_of(&mut self.cx, field_leaf));
            self.enter_place_assign_like(inner, name, Some(first));
            return;
        } else if kind == self.cx.k.place_assign_stmt.unwrap_or(u32::MAX) {
            // PLAN-1082（T-02）：place 链 `base (.f | [e])* (op)= v`——步骤
            // 节点按绝对区间排序；field 步直接取 ident，index 步的子表达式
            // 压栈（vals 栈自底向上：idx1..idxn、value，源序求值）。
            let name_leaf = self
                .cx
                .field_child(inner, "name")
                .expect("place assign name");
            let name = ident_of(&mut self.cx, name_leaf);
            self.enter_place_assign_like(inner, name, None);
            return;
        } else if kind == self.cx.k.return_stmt {
            let kw = self
                .cx
                .child_of_kind(inner, self.cx.k.tok_return)
                .map(|l| self.cx.span_of(l))
                .unwrap_or(Span { start: 0, end: 0 });
            if script_top {
                self.cx.reject(kw, || {
                    Diagnostic::new(
                        Stage::Text,
                        "source.entry-top-return",
                        "a script's result is its trailing expression; `return` is not part of the script form (use a declared `fn main` for an explicit exit)",
                        kw,
                    )
                });
            }
            let value_n = self.cx.field_child(inner, "value").expect("return value");
            self.stack.push(Frame::ReturnFinish { kw });
            self.stack.push(Frame::ExprEnter(value_n));
        } else if self.cx.profile.is_auto_language() && kind == self.cx.k.block {
            self.stack.push(Frame::ScopeFinish);
            self.enter_block(inner);
        } else if kind == self.cx.k.if_stmt {
            self.enter_if(inner);
        } else if Some(kind) == self.cx.k.hash_if_stmt {
            // PLAN-966（T-02）：`#if <ct> { } [else { }]`——谓词域是共享 CT
            // 标量表达式（sys 事实/字面量），两臂都投影进 AST，绑定期折叠。
            let cond = match self.cx.field_child(inner, "cond") {
                Some(n) => self.ct_expr(n),
                None => {
                    let span = self.cx.span_of(inner);
                    self.cx.reject(span, || {
                        Diagnostic::new(
                            Stage::Text,
                            "source.cte.in-expr",
                            "expected a compile-time condition after `#if`",
                            span,
                        )
                    });
                    CtExpr::Int(0)
                }
            };
            let then_n = self.cx.field_child(inner, "then").expect("ct if then");
            let els_n = self
                .cx
                .child_of_kind(inner, self.cx.k.else_block)
                .and_then(|eb| self.cx.child_of_kind(eb, self.cx.k.block));
            let span = self.cx.span_of(inner);
            self.stack.push(Frame::CtIfThen { cond, els_n, span });
            self.enter_block(then_n);
        } else if Some(kind) == self.cx.k.hash_for_stmt {
            // PLAN-966（T-02）：`#for i in a..b { }`——计数域与项位同（i32）。
            let var_leaf = self.cx.field_child(inner, "var").expect("ct for var");
            let var = ident_of(&mut self.cx, var_leaf);
            let iter = self.cx.field_child(inner, "iter").expect("ct for iter");
            let (start, end) = self.ct_range(iter);
            let body_n = self.cx.field_child(inner, "body").expect("ct for body");
            let span = self.cx.span_of(inner);
            self.stack.push(Frame::CtForBody {
                var,
                start,
                end,
                span,
            });
            self.enter_block(body_n);
        } else if Some(kind) == self.cx.k.hash_is_stmt {
            // PLAN-966（T-02）：`#is <ct> { pat -> { } ... }`。
            let scrutinee = match self.cx.field_child(inner, "scrutinee") {
                Some(n) => self.ct_expr(n),
                None => CtExpr::Int(0),
            };
            let span = self.cx.span_of(inner);
            let arm_kind = self.cx.k.hash_is_arm.unwrap_or(u32::MAX);
            let mut pending: Vec<(Option<CtExpr>, u32)> = Vec::new();
            for arm in self
                .cx
                .children(inner)
                .into_iter()
                .filter(|&c| self.cx.tree.kind(c) == arm_kind)
            {
                let pattern = match self.cx.field_child(arm, "pattern") {
                    Some(p) => {
                        if self.cx.field_child(p, "else").is_some() {
                            None
                        } else {
                            self.cx.field_child(p, "value").map(|v| self.ct_expr(v))
                        }
                    }
                    None => None,
                };
                let body_n = self.cx.field_child(arm, "body").expect("ct is arm body");
                pending.push((pattern, body_n));
            }
            if pending.is_empty() {
                self.cx.reject(span, || {
                    Diagnostic::new(
                        Stage::Text,
                        "source.cte.in-stmt",
                        "a compile-time `#is` needs at least one arm",
                        span,
                    )
                });
                self.vals.push(Val::S(Stmt::HashIs {
                    scrutinee,
                    arms: Vec::new(),
                    span,
                }));
            } else {
                let ctx = Box::new(CtIsCtx {
                    scrutinee,
                    pending,
                    arms: Vec::new(),
                    span,
                });
                self.start_ct_arm(ctx);
            }
        } else if Some(kind) == self.cx.k.hash_macro_stmt {
            // PLAN-966（T-02）：`#name(args...)`——语句位调用糖：实参按普通
            // 表达式投影，名字按调用在绑定期解析（无独立 Log 运行时；
            // 目标专有宏发射归 967/目标 owner）。命名实参不在本面。
            let name_leaf = self.cx.field_child(inner, "name").expect("macro name");
            let name = ident_of(&mut self.cx, name_leaf);
            let args_n = self.cx.field_child(inner, "args");
            let arg_nodes: Vec<u32> = args_n
                .map(|an| collect_seq(&self.cx, an, self.cx.k.arg, "arg-tail"))
                .unwrap_or_default();
            let span = self.cx.span_of(inner);
            let mut exprs: Vec<u32> = Vec::with_capacity(arg_nodes.len());
            for a in &arg_nodes {
                let leaf = self
                    .cx
                    .children(*a)
                    .into_iter()
                    .find(|&c| self.cx.tree.node(c).flags & FLAG_TRIVIA == 0)
                    .unwrap_or(*a);
                if self.cx.tree.kind(leaf) == self.cx.k.named_arg {
                    let aspan = self.cx.span_of(leaf);
                    self.cx.reject(aspan, || {
                        Diagnostic::new(
                            Stage::Text,
                            "source.cte.macro-args",
                            "macro calls take positional arguments only",
                            aspan,
                        )
                    });
                    continue;
                }
                exprs.push(leaf);
            }
            let argc = exprs.len();
            self.stack.push(Frame::MacroCallFinish { name, argc, span });
            for e in exprs.into_iter().rev() {
                self.stack.push(Frame::ExprEnter(e));
            }
        } else if kind == self.cx.k.while_stmt.unwrap_or(u32::MAX)
            || kind == self.cx.k.for_cond_stmt.unwrap_or(u32::MAX)
        {
            let kw_kind = if kind == self.cx.k.while_stmt.unwrap_or(u32::MAX) {
                self.cx.k.tok_while
            } else {
                self.cx.k.tok_for
            };
            let _kw_leaf = self.cx.child_of_kind(inner, kw_kind);
            let cond_n = self.cx.field_child(inner, "cond").expect("loop cond");
            let body_n = self.cx.field_child(inner, "body").expect("loop body");
            self.stack.push(Frame::WhileCond {
                body_n,
                node: inner,
            });
            self.stack.push(Frame::ExprEnter(cond_n));
        } else if kind == self.cx.k.for_in_stmt.unwrap_or(u32::MAX) {
            let var_leaf = self.cx.field_child(inner, "var").expect("for-in var");
            let var = ident_of(&mut self.cx, var_leaf);
            let iter_n = self.cx.field_child(inner, "iter").expect("for-in iter");
            let body_n = self.cx.field_child(inner, "body").expect("for-in body");
            if self.cx.tree.kind(iter_n) == self.cx.k.tok_ident {
                // 合同 C7：迭代器是裸 List 局部名（ident，非表达式——体块 `{`
                // 不得被误读为具名 ctor）。
                let iter_ident = ident_of(&mut self.cx, iter_n);
                let iter_span = iter_ident.span;
                let iter = Expr {
                    span: iter_span,
                    kind: ExprKind::Var(iter_ident),
                };
                self.stack.push(Frame::ForInBody {
                    var,
                    iter,
                    node: inner,
                });
                self.enter_block(body_n);
            } else {
                self.stack.push(Frame::ForInExprBody { var, node: inner });
                self.stack.push(Frame::ExprEnter(iter_n));
                self.enter_block(body_n);
            }
        } else if kind == self.cx.k.for_range_stmt.unwrap_or(u32::MAX) {
            // PLAN-962 (L-19): `for i in a..b/a..=b { }` — projects to a
            // ForIn whose iter is a Range expr; the resolver desugars.
            let var_leaf = self.cx.field_child(inner, "var").expect("for-range var");
            let var = ident_of(&mut self.cx, var_leaf);
            let iter_n = self.cx.field_child(inner, "iter").expect("range iter");
            let (start_n, end_n, inclusive) = range_parts(&self.cx, iter_n);
            let body_n = self.cx.field_child(inner, "body").expect("for-range body");
            self.stack.push(Frame::ForRangeBody {
                var,
                inclusive,
                node: inner,
            });
            self.stack.push(Frame::ExprEnter(end_n));
            self.stack.push(Frame::ExprEnter(start_n));
            self.enter_block(body_n);
        } else if kind == self.cx.k.is_stmt.unwrap_or(u32::MAX) {
            let scrutinee_leaf = self
                .cx
                .field_child(inner, "scrutinee")
                .expect("is scrutinee");
            let scrutinee = ident_of(&mut self.cx, scrutinee_leaf);
            let arm_kind = self.cx.k.match_arm.unwrap_or(u32::MAX);
            let mut arm_nodes: Vec<u32> = self
                .cx
                .children(inner)
                .into_iter()
                .filter(|&c| self.cx.tree.kind(c) == arm_kind)
                .collect();
            arm_nodes.sort_by_key(|&n| self.cx.tree.abs_range(n).0);
            self.stack.push(Frame::IsArms(IsMatchCtx {
                scrutinee,
                arms: Vec::new(),
                remaining: arm_nodes,
                node: inner,
            }));
        } else if kind == self.cx.k.agg_is_stmt.unwrap_or(u32::MAX) {
            self.enter_agg_is(inner, false);
        } else if kind == self.cx.k.try_stmt.unwrap_or(u32::MAX) {
            let kw = self
                .cx
                .field_child(inner, "kw")
                .map(|k| self.cx.span_of(k))
                .unwrap_or_else(|| self.cx.span_of(inner));
            let body_n = self.cx.field_child(inner, "body").expect("try body");
            let catch_n = self.cx.field_child(inner, "catch");
            let finally_n = self.cx.field_child(inner, "finally");
            if catch_n.is_none() && finally_n.is_none() {
                let sp = self.cx.span_of(inner);
                self.cx.reject(sp, || {
                    syntax_diag("`catch` or `finally` clause after `try`", sp)
                });
            }
            let (has_catch, catch_bind, catch_span, catch_body_n) = match catch_n {
                Some(cn) => {
                    let bind = self
                        .cx
                        .field_child(cn, "bind")
                        .and_then(|b| self.cx.field_child(b, "name"))
                        .map(|n| ident_of(&mut self.cx, n));
                    let cb = self.cx.field_child(cn, "body").expect("catch body");
                    (true, bind, self.cx.span_of(cn), Some(cb))
                }
                None => (false, None, kw, None),
            };
            let (has_finally, finally_body_n) = match finally_n {
                Some(fn_) => (
                    true,
                    Some(self.cx.field_child(fn_, "body").expect("finally body")),
                ),
                None => (false, None),
            };
            self.stack.push(Frame::TryFinish {
                has_catch,
                catch_bind,
                catch_span,
                has_finally,
                kw_span: kw,
            });
            if let Some(fb) = finally_body_n {
                self.enter_block(fb);
            }
            if let Some(cb) = catch_body_n {
                self.enter_block(cb);
            }
            self.enter_block(body_n);
        } else if kind == self.cx.k.with_stmt.unwrap_or(u32::MAX) {
            // PLAN-1094: `with <expr> [as <name>] { body }`.
            let kw_span = self
                .cx
                .field_child(inner, "kw")
                .map(|k| self.cx.span_of(k))
                .unwrap_or_else(|| self.cx.span_of(inner));
            let bind = self
                .cx
                .field_child(inner, "bind")
                .and_then(|b| self.cx.field_child(b, "name"))
                .map(|n| ident_of(&mut self.cx, n));
            let expr_n = self.cx.field_child(inner, "expr").expect("with expr");
            let body_n = self.cx.field_child(inner, "body").expect("with body");
            self.stack.push(Frame::WithFinish { kw_span, bind });
            self.stack.push(Frame::ExprEnter(expr_n));
            self.enter_block(body_n);
        } else if kind == self.cx.k.yield_stmt.unwrap_or(u32::MAX) {
            let kw_span = self
                .cx
                .field_child(inner, "kw")
                .map(|k| self.cx.span_of(k))
                .unwrap_or_else(|| self.cx.span_of(inner));
            let value_n = self.cx.field_child(inner, "value").expect("yield value");
            self.stack.push(Frame::YieldFinish { kw_span });
            self.stack.push(Frame::ExprEnter(value_n));
        } else if kind == self.cx.k.loop_stmt {
            let body_n = self.cx.field_child(inner, "body").expect("loop body");
            self.stack.push(Frame::LoopBody { node: inner });
            self.enter_block(body_n);
        } else if Some(kind) == self.cx.k.go_stmt {
            // PLAN-968: `go <call>` — a dedicated statement form carrying
            // the Go expression node (the desugar rewrites it).
            let value_n = self.cx.field_child(inner, "value").expect("go value");
            let kw = self
                .cx
                .field_child(inner, "kw")
                .map(|l| self.cx.span_of(l))
                .unwrap_or(Span { start: 0, end: 0 });
            self.stack.push(Frame::GoStmtFinish { kw });
            self.stack.push(Frame::ExprEnter(value_n));
        } else if Some(kind) == self.cx.k.reply_stmt {
            // PLAN-968: `reply(v)` — a dedicated statement form; projects
            // as the __sched_reply intrinsic call for the desugar/resolve
            // contract (statement position only).
            let kw = self
                .cx
                .field_child(inner, "kw")
                .map(|l| self.cx.span_of(l))
                .unwrap_or(Span { start: 0, end: 0 });
            let value_n = self.cx.field_child(inner, "value").expect("reply value");
            self.stack.push(Frame::ReplyStmtFinish { kw });
            self.stack.push(Frame::ExprEnter(value_n));
        } else if Some(kind) == self.cx.k.nested_fn_stmt {
            // PLAN-1081 (T-03): statement-position `fn` item (auto face).
            // Mirrors run_fn_item's pre-body half (no prototype arm here);
            // the body rides the block machine and NestedFnFinish wraps
            // the finished item. Bare ret = the D-04 inference marker.
            let tail = self
                .cx
                .children(inner)
                .into_iter()
                .find(|&c| self.cx.tree.kind(c) == self.cx.k.fn_tail)
                .expect("nested-fn-stmt carries fn-tail");
            let name_leaf = self.cx.field_child(tail, "name").expect("fn name");
            let name = ident_of(&mut self.cx, name_leaf);
            let type_params = self.project_type_params(self.cx.field_child(tail, "type_params"));
            if !type_params.is_empty() {
                let tp_span = type_params[0].name.span;
                self.cx.reject(tp_span, || {
                    Diagnostic::new(
                        Stage::Text,
                        "source.instances-unsupported",
                        "a nested fn item cannot be a template",
                        tp_span,
                    )
                });
            }
            let saved_ty_params = std::mem::take(&mut self.cx.ty_params);
            let params = self.project_params(self.cx.field_child(tail, "params"));
            let ret = match self.cx.field_child(tail, "ty") {
                Some(ty_n) => self.classify_word(ty_n, true, false),
                None => {
                    let span = name.span;
                    TypeAnn {
                        kind: TypeRef::Inferred,
                        span,
                    }
                }
            };
            let body_n = self.cx.field_child(tail, "body").expect("fn body");
            self.stack.push(Frame::NestedFnFinish { saved_ty_params });
            self.stack.push(Frame::FnBodyFinish {
                node: inner,
                name,
                type_params,
                params,
                ret,
                receiver: Receiver::Free,
                owner: None,
            });
            self.enter_block(body_n);
        } else if kind == self.cx.k.expr_stmt {
            let value_n = self
                .cx
                .field_child(inner, "value")
                .expect("expr stmt value");
            // 旧门 = surface.is_entry_unit()（含 HostConsole——控制台脚本面）。
            // 旧 stmt head 的表达式候选只从 ident/true/false/int 起始——其余
            // 起始（如 `(`）是"expected statement, found X"（c03 差分）。
            if !self.cx.profile.is_entry_unit() && self.cx.profile != SourceProfile::Auto {
                let first_leaf = lex::collect_leaves(&self.cx.tree)
                    .into_iter()
                    .filter(|l| l.start >= self.cx.tree.abs_range(value_n).0)
                    .min_by_key(|l| l.start);
                if let Some(l) = first_leaf {
                    let k = &self.cx.k;
                    let cand = l.kind == k.tok_ident
                        || l.kind == k.tok_true
                        || l.kind == k.tok_false
                        || l.kind == k.tok_int_lit;
                    if !cand {
                        let span = Span {
                            start: l.start as usize,
                            end: l.end as usize,
                        };
                        let text = self.cx.spelling_by_range(l.start, l.end);
                        let found = describe_g(l.kind, &text, k);
                        self.cx.reject(span, move || {
                            Diagnostic::new(
                                Stage::Text,
                                "source.syntax",
                                format!("expected statement, found {found}"),
                                span,
                            )
                        });
                    }
                }
            }
            if !self.cx.allows_expr_stmt() {
                let span = self.cx.span_of(value_n);
                self.cx.reject(span, || {
                    Diagnostic::new(
                        Stage::Text,
                        "source.unsupported",
                        "expression statements are not part of this source profile; calls are only allowed in initializers, assignments, returns and conditions",
                        span,
                    )
                });
            }
            self.stack.push(Frame::ExprStmtFinish);
            self.stack.push(Frame::ExprEnter(value_n));
        } else {
            // break/continue 叶。
            let text = self.cx.spelling(inner);
            let span = self.cx.span_of(inner);
            let stmt = if text == "break" {
                Stmt::Break { span }
            } else if text == "continue" {
                Stmt::Continue { span }
            } else {
                let found = self.describe_node(inner);
                self.cx.reject(span, move || {
                    Diagnostic::new(
                        Stage::Text,
                        "source.syntax",
                        format!("expected statement, found {}", found),
                        span,
                    )
                });
                Stmt::Break { span }
            };
            self.vals.push(Val::S(stmt));
        }
    }

    // ------------------------------------------------------------ exprs

    /// PLAN-962: project ONE expression subtree synchronously (used by
    /// the array-literal atom and the index postfix step, whose inner
    /// expressions ride under rule nodes the frame walk reaches later).
    /// Runs a bounded sub-walk of the same stepper; the sub-projection
    /// pushes exactly one value.
    fn project_expr_sync(&mut self, node: u32) -> Expr {
        let depth_mark = self.stack.len();
        let vals_mark = self.vals.len();
        self.stack.push(Frame::ExprEnter(node));
        while self.stack.len() > depth_mark {
            let f = self.stack.pop().expect("sub-projection frame");
            self.step(f);
        }
        match self.vals.pop() {
            Some(Val::E(e)) => e,
            _ => {
                self.vals.truncate(vals_mark);
                let span = self.cx.span_of(node);
                self.cx.reject(span, || syntax_diag("expression", span));
                Expr {
                    span,
                    kind: ExprKind::IntLit(0),
                }
            }
        }
    }

    fn enter_expr(&mut self, node: u32) {
        let kind = self.cx.expression_kind(node);
        if Some(kind) == self.cx.k.async_block {
            let span = self.cx.span_of(node);
            self.stack.push(Frame::AsyncBlockFinish { span });
            self.enter_block(self.cx.field_child(node, "body").expect("async body"));
            return;
        }
        if kind == self.cx.k.expr {
            // 分派节点 → 内层产物。
            let inner = self
                .cx
                .children(node)
                .into_iter()
                .find(|&c| self.cx.tree.child_count(c) > 0);
            match inner {
                Some(c) => self.stack.push(Frame::ExprEnter(c)),
                None => {
                    let span = first_leaf_span(&self.cx, node);
                    let found = self.describe_node(node);
                    self.cx.reject(span, move || {
                        Diagnostic::new(
                            Stage::Text,
                            "source.syntax",
                            format!("expected expression, found {}", found),
                            span,
                        )
                    });
                    self.vals.push(Val::E(Expr {
                        kind: ExprKind::IntLit(0),
                        span,
                    }));
                }
            }
            return;
        }
        if kind == self.cx.k.unary_expr {
            if let Some(lit) = self.cx.child_of_kind(node, self.cx.k.int_literal) {
                self.stack.push(Frame::ExprEnter(lit));
                return;
            }
            // PLAN-968: `await <expr>` / `go <expr>` (scheduler face; the
            // grammar only admits these tokens here on the scheduler trim).
            if let Some(kw) = self
                .cx
                .child_of_kind(node, self.cx.k.tok_r_await)
                .or_else(|| self.cx.child_of_kind(node, self.cx.k.tok_r_go))
            {
                let is_await = self.cx.tree.kind(kw) == self.cx.k.tok_r_await;
                let kw_span = self.cx.span_of(kw);
                let operand = self
                    .cx
                    .child_of_kind(node, self.cx.k.expr)
                    .or_else(|| self.cx.child_of_kind(node, self.cx.k.unary_expr))
                    .or_else(|| self.cx.child_of_kind(node, self.cx.k.postfix_expr));
                match operand {
                    Some(o) => {
                        self.stack
                            .push(Frame::SchedPrefixFinish { is_await, kw_span });
                        self.stack.push(Frame::ExprEnter(o));
                    }
                    None => {
                        self.cx
                            .reject(kw_span, || syntax_diag("expected expression", kw_span));
                        self.vals.push(Val::E(Expr {
                            kind: ExprKind::IntLit(0),
                            span: kw_span,
                        }));
                    }
                }
                return;
            }
            let has_minus = self.cx.child_of_kind(node, self.cx.k.tok_minus).is_some();
            let has_bang = self.cx.child_of_kind(node, self.cx.k.tok_bang).is_some();
            if has_minus || has_bang {
                let op = if has_bang { UnaryOp::Not } else { UnaryOp::Neg };
                let operand = self
                    .cx
                    .child_of_kind(node, self.cx.k.unary_expr)
                    .or_else(|| self.cx.child_of_kind(node, self.cx.k.postfix_expr));
                match operand {
                    Some(o) => {
                        // 内→外包裹：UnaryPost 先压（后弹）。
                        self.stack.push(Frame::UnaryPost { op });
                        self.stack.push(Frame::ExprEnter(o));
                    }
                    None => {
                        let span = first_leaf_span(&self.cx, node);
                        self.cx
                            .reject(span, || syntax_diag("expected expression", span));
                        self.vals.push(Val::E(Expr {
                            kind: ExprKind::IntLit(0),
                            span,
                        }));
                    }
                }
                return;
            }
            if let Some(post) = self.cx.child_of_kind(node, self.cx.k.postfix_expr) {
                self.stack.push(Frame::ExprEnter(post));
                return;
            }
            let span = first_leaf_span(&self.cx, node);
            self.cx
                .reject(span, || syntax_diag("expected expression", span));
            self.vals.push(Val::E(Expr {
                kind: ExprKind::IntLit(0),
                span,
            }));
            return;
        }
        if kind == self.cx.k.postfix_expr || kind == KIND_POSTFIX {
            let (base, steps) = self.split_postfix(node);
            if steps.is_empty() {
                self.stack.push(Frame::ExprEnter(base));
            } else {
                self.stack.push(Frame::PostfixFoldAll { steps });
                self.stack.push(Frame::ExprEnter(base));
            }
            return;
        }
        if kind == self.cx.k.int_literal {
            let e = self.project_int_literal(node);
            self.vals.push(Val::E(e));
            return;
        }
        if kind == KIND_CALL {
            self.enter_call(node);
            return;
        }
        if self.is_op_token(kind) {
            let kids = self.cx.children(node);
            if kids.len() >= 2 {
                let op = self.op_of_token(kind);
                self.stack.push(Frame::OpPost { op });
                self.stack.push(Frame::ExprEnter(kids[kids.len() - 1]));
                self.stack.push(Frame::ExprEnter(kids[0]));
            } else {
                let span = first_leaf_span(&self.cx, node);
                self.cx
                    .reject(span, || syntax_diag("binary operands", span));
                self.vals.push(Val::E(Expr {
                    kind: ExprKind::IntLit(0),
                    span,
                }));
            }
            return;
        }
        self.enter_atom(node);
    }

    fn enter_atom(&mut self, node: u32) {
        for c in self.cx.children(node) {
            let ck = self.cx.expression_kind(c);
            if Some(ck) == self.cx.k.async_block {
                self.stack.push(Frame::ExprEnter(c));
                return;
            }
            // PLAN-950: preserve the qualified source identity for module binding.
            if self
                .cx
                .p
                .rules
                .iter()
                .position(|r| r.name == "qualified-name")
                .is_some_and(|r| ck == ace_syntax_runtime::program::RULE_KIND_BASE + r as u32)
            {
                if let Some(id) = self.first_ident_opt(c) {
                    self.vals.push(Val::E(Expr {
                        span: id.span,
                        kind: ExprKind::Var(id),
                    }));
                    return;
                }
            }
            // PLAN-962（T-08，L-09）：`take(x)` 原子（弃用 Move 别名；
            // 投影为 take 调用形，由 resolve 拦截为 Move）。
            if ck == self.cx.k.take_atom.unwrap_or(u32::MAX) {
                let v_n = self.cx.field_child(c, "value").expect("take operand");
                let v = self.project_expr_sync(v_n);
                let kw = self.cx.span_of(c);
                let span = kw.join(v.span);
                self.vals.push(Val::E(Expr {
                    span,
                    kind: ExprKind::Call {
                        callee: Ident {
                            name: "take".to_string(),
                            span: kw,
                        },
                        args: vec![Arg::Pos(v)],
                    },
                }));
                return;
            }
            // PLAN-968: `reply(v)` — the reserved reply call atom on the
            // scheduler face; projects as the __sched_reply intrinsic call.
            if ck == self.cx.k.reply_atom.unwrap_or(u32::MAX) {
                let v_n = self.cx.field_child(c, "value").expect("reply operand");
                let v = self.project_expr_sync(v_n);
                let kw = self.cx.span_of(c);
                let span = kw.join(v.span);
                self.vals.push(Val::E(Expr {
                    span,
                    kind: ExprKind::Call {
                        callee: Ident {
                            name: "__sched_reply".to_string(),
                            span: kw,
                        },
                        args: vec![Arg::Pos(v)],
                    },
                }));
                return;
            }
            // PLAN-962（T-08，L-14）：grid 原子 `grid (cols) { [行]; ... }`。
            if ck == self.cx.k.grid_atom.unwrap_or(u32::MAX) {
                // 列名：args 子节点的 ident 叶（按位序）。
                let mut cols: Vec<Ident> = Vec::new();
                if let Some(cols_n) = self.cx.field_child(c, "cols") {
                    // args→arg→ident 嵌套：子树内全部 ident 叶（位序）。
                    let mut stack = vec![cols_n];
                    let mut id_leaves: Vec<u32> = Vec::new();
                    while let Some(n) = stack.pop() {
                        for ch in self.cx.children(n) {
                            if self.cx.tree.kind(ch) == self.cx.k.tok_ident {
                                id_leaves.push(ch);
                            } else {
                                stack.push(ch);
                            }
                        }
                    }
                    id_leaves.sort_by_key(|&x| self.cx.tree.abs_range(x).0);
                    for l in id_leaves {
                        cols.push(ident_of(&mut self.cx, l));
                    }
                }
                // 行：grid-row 子节点，每行 array-elems 的表达式孩子。
                let row_kind = self.cx.k.grid_row.unwrap_or(u32::MAX);
                let mut row_nodes: Vec<u32> = self
                    .cx
                    .children(c)
                    .into_iter()
                    .filter(|&x| self.cx.tree.kind(x) == row_kind)
                    .collect();
                row_nodes.sort_by_key(|&x| self.cx.tree.abs_range(x).0);
                let mut rows: Vec<Vec<Expr>> = Vec::new();
                let mut span = self.cx.span_of(c);
                for rn in row_nodes {
                    let mut cells: Vec<Expr> = Vec::new();
                    if let Some(cells_n) = self.cx.field_child(rn, "cells") {
                        let mut expr_nodes: Vec<u32> = Vec::new();
                        let mut head_id = None;
                        if let Some(head) = self.cx.field_child(cells_n, "head") {
                            expr_nodes.push(head);
                            head_id = Some(head);
                        }
                        for t in self.cx.children(cells_n) {
                            if Some(t) == head_id {
                                continue;
                            }
                            let tk = self.cx.tree.kind(t);
                            if tk == self.cx.k.tok_comma
                                || tk == self.cx.k.tok_lbracket
                                || tk == self.cx.k.tok_rbracket
                            {
                                continue;
                            }
                            // tail expr children (field-less)
                            for ec in self.cx.children(t) {
                                let ek = self.cx.tree.kind(ec);
                                if ek != self.cx.k.tok_comma
                                    && ek != self.cx.k.tok_lbracket
                                    && ek != self.cx.k.tok_rbracket
                                {
                                    expr_nodes.push(ec);
                                }
                            }
                        }
                        expr_nodes.sort_by_key(|&x| self.cx.tree.abs_range(x).0);
                        for en in expr_nodes {
                            let e = self.project_expr_sync(en);
                            span = span.join(e.span);
                            cells.push(e);
                        }
                    }
                    rows.push(cells);
                }
                self.vals.push(Val::E(Expr {
                    span,
                    kind: ExprKind::GridLit { cols, rows },
                }));
                return;
            }
            // PLAN-962（L-15）：tuple 字面量原子 `(e1, e2, ...)`（元素
            // 同步投影；≥2 元由语法保证）。
            if ck == self.cx.k.tuple_lit_rule.unwrap_or(u32::MAX) {
                let mut elems: Vec<Expr> = Vec::new();
                let mut span = self.cx.span_of(c);
                if let Some(head_n) = self.cx.field_child(c, "head") {
                    let he = self.project_expr_sync(head_n);
                    span = span.join(he.span);
                    elems.push(he);
                }
                let tail_kind = self.cx.k.tuple_lit_tail.unwrap_or(u32::MAX);
                let mut tails: Vec<u32> = self
                    .cx
                    .children(c)
                    .into_iter()
                    .filter(|&x| self.cx.tree.kind(x) == tail_kind)
                    .collect();
                tails.sort_by_key(|&n| self.cx.tree.abs_range(n).0);
                for t in tails {
                    if let Some(en) = self
                        .cx
                        .children(t)
                        .into_iter()
                        .find(|&x| self.cx.tree.kind(x) != self.cx.k.tok_comma)
                    {
                        let e = self.project_expr_sync(en);
                        span = span.join(e.span);
                        elems.push(e);
                    }
                }
                self.vals.push(Val::E(Expr {
                    span,
                    kind: ExprKind::TupleLit(elems),
                }));
                return;
            }
            // PLAN-962（L-16）：数组/List 字面量原子 `[e, ...]`。
            // PLAN-982（CONFIG）：配置文档的空列表 `[]` ⇒ 空 `ArrayLit`。独立规则
            // 让共享的 `array-lit`（aggregates/scheduler）保持"至少一个元素"。
            if ck == self.cx.k.cfg_empty_array_lit.unwrap_or(u32::MAX) {
                self.vals.push(Val::E(Expr {
                    span: self.cx.span_of(c),
                    kind: ExprKind::ArrayLit(Vec::new()),
                }));
                return;
            }
            // PLAN-1082（D-1082-04）：aggregates/auto/scheduler 的空 `[]`
            // 同样投影为空 ArrayLit——resolve 以 `source.array-lit-empty`
            // 具名拒（不引入空表元素类型推导）。
            if ck == self.cx.k.empty_array_lit.unwrap_or(u32::MAX) {
                self.vals.push(Val::E(Expr {
                    span: self.cx.span_of(c),
                    kind: ExprKind::ArrayLit(Vec::new()),
                }));
                return;
            }
            if Some(ck) == self.cx.k.age_format_string {
                let value = match self.project_age_format_string(c) {
                    Ok(value) => value,
                    Err(diagnostic) => {
                        self.cx.reject(diagnostic.span, || diagnostic);
                        Expr { kind: ExprKind::StrLit(String::new()), span: self.cx.span_of(c) }
                    }
                };
                self.vals.push(Val::E(value));
                return;
            }
            let ordinary_array = Some(ck) == self.rule_kind_of("auto-array-lit");
            if ordinary_array || Some(ck) == self.cx.k.array_lit {
                let mut expr_nodes = Vec::new();
                if let Some(elems_n) = self.cx.field_child(c, "elems") {
                    // head 是 expr 孩子；尾部规则孩子（array-elem-tail）
                    // 各携带一个无字段 expr 孩子——按绝对序收集。
                    let head = self
                        .cx
                        .field_child(elems_n, "head")
                        .expect("array head element");
                    expr_nodes.push(head);
                    let mut previous_end = self.cx.span_of(head).end;
                    for t in self.cx.children(elems_n) {
                        if t == head {
                            continue;
                        }
                        let mut comma = false;
                        for child in self.cx.children(t) {
                            if self.cx.tree.kind(child) == self.cx.k.tok_comma {
                                comma = true;
                                continue;
                            }
                            let element = self.cx.span_of(child);
                            if !comma
                                && self.cx.profile == SourceProfile::Auto
                                && self.cx.line_of_pos(previous_end.saturating_sub(1) as u64)
                                    == self.cx.line_of_pos(element.start as u64)
                            {
                                self.cx.reject(element, || {
                                    Diagnostic::new(
                                        Stage::Text,
                                        "source.syntax",
                                        "array elements require a comma or a newline",
                                        element,
                                    )
                                });
                            }
                            previous_end = element.end;
                            expr_nodes.push(child);
                        }
                    }
                    expr_nodes.sort_by_key(|&n| self.cx.tree.abs_range(n).0);
                }
                // Child expressions stay in the projection work stack even
                // when arrays and objects alternate many times.
                self.stack.push(Frame::ArrayLiteralFinish {
                    span: self.cx.span_of(c),
                    count: expr_nodes.len(),
                });
                let conditional = self.rule_kind_of("array-conditional-elem");
                for element in expr_nodes.into_iter().rev() {
                    let node = if ordinary_array { self.unwrap_child(element) } else { element };
                    if Some(self.cx.tree.kind(node)) == conditional {
                        let cond = self.cx.field_child(node, "cond").expect("element condition");
                        let value = self.cx.field_child(node, "value").expect("element value");
                        self.stack.push(Frame::ArrayConditionalElemFinish { span: self.cx.span_of(node) });
                        self.stack.push(Frame::ExprEnter(value));
                        self.stack.push(Frame::ExprEnter(cond));
                    } else {
                        self.stack.push(Frame::ExprEnter(node));
                    }
                }
                return;
            }
            // PLAN-964（L-16/L-20）：裸对象字面量原子 `{ k: v, ... }`——
            // 键为 ident/字符串叶子，值表达式按源序投影。
            if ck == self.cx.k.object_lit.unwrap_or(u32::MAX) {
                let mut fields = Vec::new();
                let mut values = Vec::new();
                if let Some(fields_n) = self.cx.field_child(c, "fields") {
                    let fk = self.cx.k.object_field.unwrap_or(u32::MAX);
                    let mut nodes: Vec<u32> =
                        collect_seq(&self.cx, fields_n, fk, "object-field-tail");
                    nodes.sort_by_key(|&n| self.cx.tree.abs_range(n).0);
                    for f in nodes {
                        use crate::hir::ObjectKey;
                        let key = match self.cx.field_child(f, "key") {
                            Some(key_node) => {
                                let k = self.cx.field_child(key_node, "key").unwrap_or(key_node);
                                let magnitude = self.cx.field_child(k, "magnitude");
                                let text = match magnitude {
                                    Some(leaf) => format!("-{}", self.cx.spelling(leaf)),
                                    None => self.cx.spelling(k),
                                };
                                match self.cx.tree.kind(k) {
                                    tk if tk == self.cx.k.tok_int_lit || magnitude.is_some() => {
                                        let key_span = self.cx.span_of(k);
                                        match text.parse() {
                                            Ok(key) => ObjectKey::Int(key),
                                            Err(_) => {
                                                self.cx.reject(key_span, || {
                                                    Diagnostic::new(
                                                        Stage::Text,
                                                        "source.literal-range",
                                                        "object integer key exceeds i32 range",
                                                        key_span,
                                                    )
                                                });
                                                continue;
                                            }
                                        }
                                    }
                                    tk if tk == self.cx.k.tok_true => ObjectKey::Bool(true),
                                    tk if tk == self.cx.k.tok_false => ObjectKey::Bool(false),
                                    _ if text.starts_with('"') && text.ends_with('"') => {
                                        ObjectKey::Text(lex::decode_str_value(
                                            &text.as_bytes()[1..text.len() - 1],
                                        ))
                                    }
                                    _ => ObjectKey::Text(text),
                                }
                            }
                            None => {
                                let span = self.cx.span_of(f);
                                self.cx
                                    .reject(span, || syntax_diag("object field key", span));
                                continue;
                            }
                        };
                        fields.push((key, self.cx.span_of(f)));
                        values.push(self.cx.field_child(f, "value").expect("object field value"));
                    }
                }
                self.stack.push(Frame::ObjectLiteralFinish { span: self.cx.span_of(c), fields });
                self.stack.extend(values.into_iter().rev().map(Frame::ExprEnter));
                return;
            }
            // PLAN-964（L-21）：nil 字面量原子（nil/null 统一）。
            if ck == self.cx.k.nil_lit.unwrap_or(u32::MAX) {
                let span = self.cx.span_of(c);
                self.vals.push(Val::E(Expr {
                    span,
                    kind: ExprKind::NilLit,
                }));
                return;
            }
            // PLAN-963（T-04）：`rep<2>(21)` 值实参调用原子（实例面）。
            if ck == self.cx.k.generic_call.unwrap_or(u32::MAX) {
                let name_leaf = self.cx.field_child(c, "name").expect("generic call name");
                let name = ident_of(&mut self.cx, name_leaf);
                let mut consts = Vec::new();
                let mut types = Vec::new();
                if let Some(group) = self.cx.field_child(c, "cargs") {
                    let kind = self.cx.k.const_arg.unwrap_or(u32::MAX);
                    for item in collect_seq(&self.cx, group, kind, "const-arg-tail") {
                        let leaf = self.cx.field_child(item, "lit").expect("constant argument");
                        consts.push(self.project_const_argument(leaf));
                    }
                }
                if let Some(group) = self.cx.field_child(c, "iargs") {
                    let explicit = self.project_instance_arguments(group);
                    types = explicit.types;
                    consts = explicit.consts;
                }
                let args = self.project_call_arguments(c);
                let span = self.cx.span_of(c);
                self.vals.push(Val::E(Expr {
                    span,
                    kind: ExprKind::ConstCall {
                        callee: name,
                        types,
                        consts,
                        args,
                    },
                }));
                return;
            }
            // PLAN-928（T-03）：`List<T>.member` 接收原子。
            if ck == self.cx.k.generic_atom.unwrap_or(u32::MAX) {
                let name_leaf = self.cx.field_child(c, "name").expect("generic atom name");
                let name = ident_of(&mut self.cx, name_leaf);
                let mut args = Vec::new();
                if let Some(an) = self.cx.field_child(c, "args") {
                    for a in collect_seq(
                        &self.cx,
                        an,
                        self.cx.k.type_arg.unwrap_or(u32::MAX),
                        "type-arg-tail",
                    ) {
                        args.push(self.classify_ty_arg(a));
                    }
                }
                let gt_leaf = self
                    .cx
                    .children(c)
                    .into_iter()
                    .find(|&l| self.cx.expression_kind(l) == self.cx.k.op_gt)
                    .map(|l| self.cx.span_of(l))
                    .unwrap_or_else(|| self.cx.span_of(c));
                let g_span = name.span.join(gt_leaf);
                let g = Expr {
                    span: g_span,
                    kind: ExprKind::GenericType { name, args },
                };
                let member_leaf = self
                    .cx
                    .field_child(c, "member")
                    .expect("generic atom member");
                let mspan = self.cx.span_of(member_leaf);
                let mname = self.cx.spelling(member_leaf);
                self.vals.push(Val::E(Expr {
                    span: g_span.join(mspan),
                    kind: ExprKind::Field {
                        base: Box::new(g),
                        name: Ident {
                            name: mname,
                            span: mspan,
                        },
                    },
                }));
                return;
            }
            // PLAN-936（T-04）：`#{ expr }` CT 需求原子（scalar-ct 面）。
            if ck == self.cx.k.hash_expr.unwrap_or(u32::MAX) {
                let span = self.cx.span_of(c);
                match self.cx.field_child(c, "expr") {
                    Some(inner) => {
                        self.stack.push(Frame::HashExpr { span });
                        self.stack.push(Frame::ExprEnter(inner));
                    }
                    None => {
                        self.cx.reject(span, || {
                            syntax_diag("expected an expression inside `#{ ... }`", span)
                        });
                        self.vals.push(Val::E(Expr {
                            kind: ExprKind::IntLit(0),
                            span,
                        }));
                    }
                }
                return;
            }
            if Some(ck) == self.cx.k.if_value {
                let span = self.cx.span_of(c);
                let cond_node = self.cx.field_child(c, "cond").expect("if value condition");
                let then_node = self.cx.field_child(c, "then").expect("if value then");
                let else_node = self.cx.field_child(c, "else").expect("if value else");
                self.stack.push(Frame::IfValueCond {
                    span,
                    then_node,
                    else_node,
                });
                self.stack.push(Frame::ExprEnter(cond_node));
                return;
            }
            // PLAN-965 (M2-A): `is` expression atom (aggregates only).
            if ck == self.cx.k.is_expr.unwrap_or(u32::MAX) {
                self.enter_agg_is(c, true);
                return;
            }
            // PLAN-928：lambda 原子（closures 面）——先于其余原子形状。
            if ck == self.cx.k.lambda_expr.unwrap_or(u32::MAX) {
                self.enter_lambda(c);
                return;
            }
            if Some(ck) == self.cx.k.leading_dot || Some(ck) == self.rule_kind_of("age-leading-type") {
                let dot_leaf = self
                    .cx
                    .children(c)
                    .into_iter()
                    .find(|&l| self.cx.expression_kind(l) == self.cx.k.tok_dot);
                let dspan = dot_leaf
                    .map(|l| self.cx.span_of(l))
                    .unwrap_or_else(|| self.cx.span_of(c));
                let member = self
                    .cx
                    .field_child(c, "member")
                    .expect("leading-dot member");
                let mspan = self.cx.span_of(member);
                let mname = self.cx.spelling(member);
                let recv = Expr {
                    span: dspan,
                    kind: ExprKind::Var(Ident {
                        name: "self".to_string(),
                        span: dspan,
                    }),
                };
                let span = dspan.join(mspan);
                // `.name(args)`：args 由 KIND_CALL 收尾（callee 含此后缀链）。
                self.vals.push(Val::E(Expr {
                    span,
                    kind: ExprKind::Field {
                        base: Box::new(recv),
                        name: Ident {
                            name: mname,
                            span: mspan,
                        },
                    },
                }));
                return;
            }
            if ck == self.cx.k.ctor_expr.unwrap_or(u32::MAX)
                || ck == self.cx.k.typed_ctor_expr.unwrap_or(u32::MAX)
            {
                let ty_leaf = self.cx.field_child(c, "ty").expect("ctor type");
                let ty = ident_of(&mut self.cx, ty_leaf);
                // PLAN-963（T-02）：`Point<int> { .. }` 的显式类型实参。
                let mut args = Vec::new();
                if let Some(an) = self.cx.field_child(c, "args") {
                    for a in collect_seq(
                        &self.cx,
                        an,
                        self.cx.k.type_arg.unwrap_or(u32::MAX),
                        "type-arg-tail",
                    ) {
                        args.push(self.classify_ty_arg(a));
                    }
                }
                let field_nodes: Vec<u32> = self
                    .cx
                    .child_of_kind(c, self.cx.k.ctor_fields.unwrap_or(u32::MAX))
                    .map(|fl| {
                        collect_seq(
                            &self.cx,
                            fl,
                            self.cx.k.ctor_field.unwrap_or(u32::MAX),
                            "ctor-tail",
                        )
                    })
                    .unwrap_or_default();
                let count = field_nodes.len();
                if count == 0 {
                    // 旧怪癖：空 ctor `Ty { }` 重入 primary（同 leading-dot 空调用）。
                    self.empty_value_quirk(self.cx.span_of(c).end);
                }
                self.stack.push(Frame::CtorFinish {
                    node: c,
                    ty,
                    args,
                    count,
                });
                for f in field_nodes.into_iter().rev() {
                    let label_leaf = self.cx.field_child(f, "label").expect("ctor label");
                    let label = ident_of(&mut self.cx, label_leaf);
                    let value_n = self.cx.field_child(f, "value").expect("ctor value");
                    self.stack.push(Frame::CtorFieldBuild { label });
                    self.stack.push(Frame::ExprEnter(value_n));
                }
                return;
            }
            if ck == self.cx.k.tok_str_lit {
                let span = self.cx.span_of(c);
                let raw = self.cx.spelling(c);
                // PLAN-928：collections/closures 面拥有 Str 字面量（元素域）。
                // PLAN-936（T-04）：ct 面上字符串字面量进入 AST（唯一合法位置是
                // ct_has_field 的字段选择子，由 resolve 受控验证）。
                if !self.cx.profile.admits_text_literals() && !self.cx.ct_face {
                    // 旧合同：非 text 面 lexer 产 Raw(StrLit)、primary 拒绝。
                    self.cx.reject(span, move || {
                        Diagnostic::new(
                            Stage::Text,
                            "source.unsupported",
                            format!("string literal `{raw}` is not part of this source profile"),
                            span,
                        )
                    });
                    self.vals.push(Val::E(Expr {
                        kind: ExprKind::StrLit(String::new()),
                        span,
                    }));
                    return;
                }
                let inner = &raw.as_bytes()[1..raw.len() - 1];
                let value = lex::decode_str_value(inner);
                self.vals.push(Val::E(Expr {
                    kind: ExprKind::StrLit(value),
                    span,
                }));
                return;
            }
            if ck == self.cx.k.tok_char_lit {
                let span = self.cx.span_of(c);
                let raw = self.cx.spelling(c);
                // PLAN-928：collections/closures 面拥有 char 字面量。
                if !self.cx.profile.admits_text_literals() {
                    self.cx.reject(span, move || {
                        Diagnostic::new(
                            Stage::Text,
                            "source.unsupported",
                            format!("char literal `{raw}` is not part of this source profile"),
                            span,
                        )
                    });
                    self.vals.push(Val::E(Expr {
                        kind: ExprKind::CharLit(0),
                        span,
                    }));
                    return;
                }
                let inner = &raw.as_bytes()[1..raw.len() - 1];
                let scalar = lex::decode_char_value(inner);
                self.vals.push(Val::E(Expr {
                    kind: ExprKind::CharLit(scalar),
                    span,
                }));
                return;
            }
            if ck == self.cx.k.tok_affix_num {
                let span = self.cx.span_of(c);
                let raw = self.cx.spelling(c);
                // PLAN-961: the numeric profile decodes the typed literal
                // (prefix/suffix/fraction per the frozen T-01 §1 table);
                // every other profile keeps the frozen raw rejection.
                if self.cx.profile.is_numeric() || self.cx.age_face {
                    match decode_typed_literal(&raw) {
                        Ok(kind) => {
                            self.vals.push(Val::E(Expr { kind, span }));
                            return;
                        }
                        Err(msg) => {
                            let m = msg;
                            self.cx.reject(span, move || {
                                Diagnostic::new(
                                    Stage::Text,
                                    "source.literal-range",
                                    m.clone(),
                                    span,
                                )
                            });
                            self.vals.push(Val::E(Expr {
                                kind: ExprKind::IntLit(0),
                                span,
                            }));
                            return;
                        }
                    }
                }
                self.cx.reject(span, move || {
                    Diagnostic::new(
                        Stage::Text,
                        "source.unsupported",
                        format!(
                            "suffixed/radix/float numeric literal `{raw}` is not part of this source profile"
                        ),
                        span,
                    )
                });
                self.vals.push(Val::E(Expr {
                    kind: ExprKind::IntLit(0),
                    span,
                }));
                return;
            }
            if ck == self.cx.k.tok_true || ck == self.cx.k.tok_false {
                let span = self.cx.span_of(c);
                let v = ck == self.cx.k.tok_true;
                self.vals.push(Val::E(Expr {
                    kind: ExprKind::BoolLit(v),
                    span,
                }));
                return;
            }
            if ck == self.cx.k.tok_ident {
                let span = self.cx.span_of(c);
                let name = self.cx.spelling(c);
                self.vals.push(Val::E(Expr {
                    kind: ExprKind::Var(Ident { name, span }),
                    span,
                }));
                return;
            }
            // PLAN-1094: a keyword-spelled name atom (`int(...)` etc. on the
            // ordinary face) — the num-ty-word rule node carries the token
            // leaf under `word` (py-member-name's `name` is kept for the
            // use.py member list); spelling is used (no name check).
            if self
                .cx
                .k
                .name_word
                .is_some_and(|kind| kind == ck)
                || self.cx.k.num_ty_word.is_some_and(|kind| kind == ck)
            {
                let leaf = self
                    .cx
                    .field_child(c, "name")
                    .or_else(|| self.cx.field_child(c, "word"))
                    .expect("name-word leaf");
                let span = self.cx.span_of(leaf);
                let name = self.cx.spelling(leaf);
                self.vals.push(Val::E(Expr {
                    kind: ExprKind::Var(Ident { name, span }),
                    span,
                }));
                return;
            }
            if ck == self.cx.k.expr {
                // 括号组：内层表达式（span 保持内层自身——旧合同）。
                let inner = self
                    .cx
                    .children(c)
                    .into_iter()
                    .find(|&g| self.cx.tree.child_count(g) > 0);
                match inner {
                    Some(g) => self.stack.push(Frame::ExprEnter(g)),
                    None => {
                        let span = first_leaf_span(&self.cx, node);
                        let found = self.describe_node(node);
                        self.cx.reject(span, move || {
                            Diagnostic::new(
                                Stage::Text,
                                "source.syntax",
                                format!("expected expression, found {}", found),
                                span,
                            )
                        });
                        self.vals.push(Val::E(Expr {
                            kind: ExprKind::IntLit(0),
                            span,
                        }));
                    }
                }
                return;
            }
        }
        let span = first_leaf_span(&self.cx, node);
        let found = self.describe_node(node);
        self.cx.reject(span, move || {
            Diagnostic::new(
                Stage::Text,
                "source.syntax",
                format!("expected expression, found {}", found),
                span,
            )
        });
        self.vals.push(Val::E(Expr {
            kind: ExprKind::IntLit(0),
            span,
        }));
    }

    /// PLAN-928 T-02：lambda 投影（closures 面）。`move? ( 参数? ) => 块|表达式`
    /// ——参数 = ident + 类型（无 mode 词，旧 closure_expr 合同）；块体走既有
    /// 语句帧机；span = move/`(` 起点join体终点。
    fn enter_lambda(&mut self, node: u32) {
        let mode_leaf = self.cx.child_of_kind(node, self.cx.k.tok_r_move);
        let move_capture = mode_leaf.is_some();
        let start = match mode_leaf {
            Some(m) => self.cx.span_of(m),
            None => self
                .cx
                .child_of_kind(node, self.cx.k.tok_lparen)
                .map(|l| self.cx.span_of(l))
                .unwrap_or_else(|| self.cx.span_of(node)),
        };
        let typed_block = self.cx.field_child(node, "typed-body");
        let mut params = Vec::new();
        if let Some(pn) = self.cx.field_child(node, "params") {
            let plist: Vec<u32> = collect_seq(
                &self.cx,
                pn,
                self.cx.k.lambda_param.unwrap_or(u32::MAX),
                "lambda-param-tail",
            );
            for p in plist {
                let name_leaf = self.cx.field_child(p, "name").expect("lambda param name");
                let name = ident_of(&mut self.cx, name_leaf);
                let ty = self
                    .cx
                    .field_child(p, "ty")
                    .map(|t| self.classify_word(t, false, typed_block.is_some()))
                    .unwrap_or(TypeAnn {
                        kind: TypeRef::Builtin(TypeWord::Int),
                        span: name.span,
                    });
                let span = name.span.join(ty.span);
                params.push(Param {
                    span,
                    name,
                    ty,
                    mode: ParamMode::Default,
                    default: None,
                });
            }
        }
        if let Some(block) = typed_block {
            let result = self.cx.field_child(node, "ret")
                .map(|ret| self.classify_word(ret, true, false));
            let start = self.cx.field_child(node, "fn").map_or(start, |kw| self.cx.span_of(kw));
            self.stack.push(Frame::LambdaBlockBody { move_capture, params, result, start });
            self.enter_block(block);
            return;
        }
        if let Some(body_n) = self.cx.field_child(node, "body") {
            if let Some(bb) = self.cx.field_child(body_n, "block-body") {
                self.stack.push(Frame::LambdaBlockBody {
                    move_capture,
                    params,
                    result: None,
                    start,
                });
                self.enter_block(bb);
            } else if let Some(eb) = self.cx.field_child(body_n, "expr-body") {
                self.stack.push(Frame::LambdaExprBody {
                    move_capture,
                    params,
                    start,
                });
                self.stack.push(Frame::ExprEnter(eb));
            } else {
                let span = self.cx.span_of(node);
                self.cx.reject(span, || {
                    syntax_diag("expected lambda body after `=>`", span)
                });
                self.vals.push(Val::E(Expr {
                    kind: ExprKind::IntLit(0),
                    span,
                }));
            }
        } else {
            let span = self.cx.span_of(node);
            self.cx.reject(span, || {
                syntax_diag("expected lambda body after `=>`", span)
            });
            self.vals.push(Val::E(Expr {
                kind: ExprKind::IntLit(0),
                span,
            }));
        }
    }

    fn project_int_literal(&mut self, node: u32) -> Expr {
        // PLAN-961: the numeric profile's int-literal node may carry an
        // affix-num leaf (typed word, optionally under the minus arm —
        // `-9223372036854775808i64` folds HERE where the minus is visible).
        if self.cx.profile.is_numeric() {
            if let Some(affix) = self.cx.child_of_kind(node, self.cx.k.tok_affix_num) {
                let raw = self.cx.spelling(affix);
                let neg = self.cx.child_of_kind(node, self.cx.k.tok_minus).is_some();
                let span = self.cx.span_of(node);
                return match decode_typed_literal_neg(&raw, neg) {
                    Ok(kind) => Expr { kind, span },
                    Err(msg) => {
                        self.cx.reject(span, move || {
                            Diagnostic::new(Stage::Text, "source.literal-range", msg.clone(), span)
                        });
                        Expr {
                            kind: ExprKind::IntLit(0),
                            span,
                        }
                    }
                };
            }
        }
        let int_leaf = self
            .cx
            .child_of_kind(node, self.cx.k.tok_int_lit)
            .expect("int digits");
        let neg = self.cx.child_of_kind(node, self.cx.k.tok_minus).is_some();
        let digits = self.cx.spelling(int_leaf);
        let mag: u128 = digits.parse().unwrap_or(u128::MAX);
        let span = self.cx.span_of(node);
        if neg {
            if mag > I32_MIN_MAG {
                let digits2 = digits.clone();
                self.cx.reject(span, move || {
                    Diagnostic::new(
                        Stage::Text,
                        "source.literal-range",
                        format!(
                            "integer literal `-{}` is outside the signed 32-bit range of this source profile",
                            digits2
                        ),
                        span,
                    )
                });
            }
            Expr {
                kind: ExprKind::IntLit((mag as i128 * -1) as i32),
                span,
            }
        } else {
            if mag > I32_MAX {
                let digits2 = digits.clone();
                self.cx.reject(span, move || {
                    Diagnostic::new(
                        Stage::Text,
                        "source.literal-range",
                        format!(
                            "integer literal `{}` is outside the signed 32-bit range of this source profile",
                            digits2
                        ),
                        span,
                    )
                });
            }
            Expr {
                kind: ExprKind::IntLit(mag as i32),
                span,
            }
        }
    }

    fn split_postfix(&self, node: u32) -> (u32, Vec<u32>) {
        // KIND_POSTFIX（调用收尾续读）：孩子 = [操作数, postfix-tail 规则节点]。
        if self.cx.expression_kind(node) == KIND_POSTFIX {
            let kids = self.cx.children(node);
            let base = *kids.first().expect("postfix wrap operand");
            let mut steps: Vec<u32> = Vec::new();
            if let Some(&tail) = kids.get(1) {
                let step_kind = self.cx.k.postfix_step.unwrap_or(u32::MAX);
                for c in self.cx.children(tail) {
                    if self.cx.expression_kind(c) == step_kind {
                        steps.push(c);
                    }
                }
            }
            return (base, steps);
        }
        let mut base = None;
        let mut steps: Vec<u32> = Vec::new();
        for c in self.cx.children(node) {
            let k = self.cx.expression_kind(c);
            if k == self.cx.k.atom_expr && base.is_none() {
                base = Some(c);
            } else if k == self.cx.k.postfix_step.unwrap_or(u32::MAX) {
                steps.push(c);
            }
        }
        (base.expect("postfix base"), steps)
    }

    /// 折叠全部后缀步（迭代；不递归）。PLAN-1080（T-03）：管道步与点后缀
    /// 同构脱糖（紧结合——管道作用于紧邻后缀链，二元算子在其后作用于整
    /// 个管道结果；legacy 低结合形需显式括号，见 §6.5 D-03 边界记录）。
    fn fold_postfix(&mut self, recv: Expr, steps: Vec<u32>) -> Expr {
        let mut value = recv;
        for step in steps {
            let member = self.step_member(step);
            value = self.fold_one_postfix(value, step, member);
        }
        value
    }

    fn step_member(&self, step: u32) -> Option<u32> {
        self.cx.children(step).into_iter().find(|&m| {
            let mk = self.cx.tree.kind(m);
            mk == self.cx.k.tok_ident
                || mk == self.cx.k.tok_mut
                || mk == self.cx.k.tok_r_move
                // PLAN-962 (T-08): `.view` member leaf.
                || mk == self.cx.k.tok_r_view
                // PLAN-965 (M2-A): `May.nil` member leaf.
                || self.cx.k.tok_r_nil.is_some_and(|n| mk == n)
                // PLAN-968: `Task.spawn` member leaf (`spawn` is reserved).
                || mk == self.cx.k.tok_r_spawn
                || (self.cx.profile.is_auto_language()
                    && (mk == self.cx.k.tok_r_await || mk == self.cx.k.tok_r_go))
                || (self.cx.profile.is_auto_language()
                    && ([self.cx.k.tok_int_lit, self.cx.k.tok_true, self.cx.k.tok_false]
                        .contains(&mk)
                        || self.cx.k.tok_double_kw == Some(mk)))
        })
    }

    /// PLAN-964: whether a postfix step is a bracket index step (`[expr]`,
    /// no dot member) rather than a member/call step.
    fn step_is_index(&self, step: u32) -> bool {
        self.cx
            .children(step)
            .into_iter()
            .any(|c| self.cx.tree.kind(c) == self.cx.k.tok_lbracket)
    }

    /// PLAN-965: whether a postfix step is `.?` or `.?(fallback)`.
    fn step_is_dotquestion(&self, step: u32) -> bool {
        self.cx.k.tok_dotquestion.is_some_and(|dq| {
            self.cx
                .children(step)
                .into_iter()
                .any(|c| self.cx.tree.kind(c) == dq)
        })
    }

    fn fold_one_postfix(&mut self, recv: Expr, step: u32, member: Option<u32>) -> Expr {
        if let Some(op) = member {
            let token = self.cx.tree.kind(op);
            if token == self.cx.k.tok_r_await || token == self.cx.k.tok_r_go {
                return Expr {
                    span: recv.span.join(self.cx.span_of(op)),
                    kind: if token == self.cx.k.tok_r_await {
                        ExprKind::Await(Box::new(recv))
                    } else {
                        ExprKind::Go(Box::new(recv))
                    },
                };
            }
        }
        // PLAN-965: `.?` and `.?(fallback)` steps (aggregates only).
        if self.step_is_dotquestion(step) {
            let span = recv.span.join(self.cx.span_of(step));
            if let Some(fallback_n) = self.cx.field_child(step, "fallback") {
                let fallback = self.project_expr_sync(fallback_n);
                return Expr {
                    span,
                    kind: ExprKind::UnwrapOr {
                        inner: Box::new(recv),
                        fallback: Box::new(fallback),
                    },
                };
            }
            return Expr {
                span,
                kind: ExprKind::Propagate {
                    inner: Box::new(recv),
                },
            };
        }
        if let Some(range) = self.cx.field_child(step, "slice_range") {
            let range_span = self.cx.span_of(range);
            let start = self
                .cx
                .field_child(step, "slice_start")
                .map(|node| self.project_expr_sync(node))
                .unwrap_or(Expr {
                    span: range_span,
                    kind: ExprKind::IntLit(0),
                });
            let mut args = vec![Arg::Pos(start)];
            if let Some(end) = self.cx.field_child(step, "slice_end") {
                args.push(Arg::Pos(self.project_expr_sync(end)));
            }
            return Expr {
                span: recv.span.join(self.cx.span_of(step)),
                kind: ExprKind::MethodCall {
                    instance: None,
                    recv: Box::new(recv),
                    name: Ident {
                        name: "slice".into(),
                        span: range_span,
                    },
                    args,
                },
            };
        }
        // PLAN-962 (L-48)：索引步 `[expr]`（member 为 None、步带 lbracket
        // 孩子；内层 index 同步投影）。
        let has_lbracket = self
            .cx
            .children(step)
            .into_iter()
            .any(|c| self.cx.tree.kind(c) == self.cx.k.tok_lbracket);
        if has_lbracket {
            // 内层索引表达式是步里唯一带孩子的规则节点（lbracket/rbracket
            // 是 token 叶子；token 的 expression_kind 非 MAX，不能作选择键）。
            let idx_node = self
                .cx
                .children(step)
                .into_iter()
                .find(|&c| self.cx.tree.child_count(c) > 0)
                .or_else(|| {
                    self.cx.children(step).into_iter().find(|&c| {
                        self.cx.tree.kind(c) >= ace_syntax_runtime::program::RULE_KIND_BASE
                    })
                });
            let Some(idx_node) = idx_node else {
                let span = recv.span;
                self.cx
                    .reject(span, || syntax_diag("index expression", span));
                return recv;
            };
            let index = self.project_expr_sync(idx_node);
            let end = if self.cx.profile.is_auto_language() {
                self.cx.span_of(step)
            } else {
                index.span
            };
            let span = recv.span.join(end);
            return Expr {
                span,
                kind: ExprKind::Index {
                    base: Box::new(recv),
                    index: Box::new(index),
                },
            };
        }
        // `.to(...)`：步自带括号与词。
        let has_to = self
            .cx
            .children(step)
            .into_iter()
            .any(|c| self.cx.tree.kind(c) == self.cx.k.tok_r_to);
        if has_to {
            // PLAN-961: the numeric `.to(T)` step carries a num-ty-word
            // child whose word leaf is a numeric KEYWORD token.
            let numeric_word = numeric_step_word(&self.cx, step);
            if numeric_word.is_some() {
                let (word_span, tw) = numeric_word.expect("checked");
                let span = recv.span.join(word_span);
                return Expr {
                    span,
                    kind: ExprKind::Method {
                        recv: Box::new(recv),
                        name: Ident {
                            name: "to".to_string(),
                            span: word_span,
                        },
                        args: vec![Arg::Pos(Expr {
                            span: word_span,
                            kind: ExprKind::TypeWordArg(tw),
                        })],
                    },
                };
            }
            let word_leaf = self
                .cx
                .children(step)
                .into_iter()
                .find(|&c| self.cx.tree.kind(c) == self.cx.k.tok_ident)
                .expect(".to word");
            let word_span = self.cx.span_of(word_leaf);
            let word = self.cx.spelling(word_leaf);
            let tw = match word.as_str() {
                "str" => Some(TypeWord::Str),
                "Str" => Some(TypeWord::StrOwned),
                _ => None,
            };
            return match tw {
                Some(tw) => {
                    let span = recv.span.join(word_span);
                    Expr {
                        span,
                        kind: ExprKind::Method {
                            recv: Box::new(recv),
                            name: Ident {
                                name: "to".to_string(),
                                span: word_span,
                            },
                            args: vec![Arg::Pos(Expr {
                                span: word_span,
                                kind: ExprKind::TypeWordArg(tw),
                            })],
                        },
                    }
                }
                None => {
                    self.cx.reject(word_span, || {
                        Diagnostic::new(
                            Stage::Text,
                            "source.syntax",
                            "`.to` expects `str` or `Str`",
                            word_span,
                        )
                    });
                    let span = recv.span.join(word_span);
                    Expr {
                        span,
                        kind: ExprKind::Method {
                            recv: Box::new(recv),
                            name: Ident {
                                name: "to".to_string(),
                                span: word_span,
                            },
                            args: Vec::new(),
                        },
                    }
                }
            };
        }
        // PLAN-961: the numeric `.as(T)` step (r-as member + num-ty-word).
        let has_as = self
            .cx
            .children(step)
            .into_iter()
            .any(|c| self.cx.tree.kind(c) == self.cx.k.tok_r_as);
        if has_as {
            match numeric_step_word(&self.cx, step) {
                Some((word_span, tw)) => {
                    let span = recv.span.join(word_span);
                    return Expr {
                        span,
                        kind: ExprKind::Method {
                            recv: Box::new(recv),
                            name: Ident {
                                name: "as".to_string(),
                                span: word_span,
                            },
                            args: vec![Arg::Pos(Expr {
                                span: word_span,
                                kind: ExprKind::TypeWordArg(tw),
                            })],
                        },
                    };
                }
                None => {
                    let span = recv.span;
                    self.cx.reject(span, || {
                        Diagnostic::new(
                            Stage::Text,
                            "source.syntax",
                            "`.as` expects a numeric type word".to_string(),
                            span,
                        )
                    });
                    return recv;
                }
            }
        }
        let Some(member) = member else {
            return recv;
        };
        if let Some(group) = self.cx.field_child(step, "iargs") {
            let name = ident_of(&mut self.cx, member);
            let instance = Box::new(self.project_instance_arguments(group));
            let args = self.project_call_arguments(step);
            return Expr {
                span: recv.span.join(self.cx.span_of(step)),
                kind: ExprKind::MethodCall {
                    recv: Box::new(recv),
                    name,
                    args,
                    instance: Some(instance),
                },
            };
        }
        let mk = self.cx.tree.kind(member);
        // PLAN-962 (T-08, L-06): `.view` — readonly borrow argument.
        if mk == self.cx.k.tok_r_view {
            let mspan = self.cx.span_of(member);
            let name = match &recv.kind {
                ExprKind::Var(id) => id.clone(),
                _ => {
                    let span = recv.span.join(mspan);
                    self.cx.reject(span, || {
                        Diagnostic::new(
                            Stage::Text,
                            "source.unsupported",
                            "`.view` applies to a local variable".to_string(),
                            span,
                        )
                    });
                    Ident {
                        name: String::new(),
                        span: mspan,
                    }
                }
            };
            let span = name.span.join(mspan);
            return Expr {
                kind: ExprKind::ViewArg { name },
                span,
            };
        }
        if mk == self.cx.k.tok_mut {
            let mspan = self.cx.span_of(member);
            let name = match &recv.kind {
                ExprKind::Var(id) => id.clone(),
                _ => {
                    let span = recv.span.join(mspan);
                    self.cx.reject(span, || {
                        Diagnostic::new(
                            Stage::Text,
                            "source.unsupported",
                            "`.mut` applies to a local variable",
                            span,
                        )
                    });
                    Ident {
                        name: String::new(),
                        span: mspan,
                    }
                }
            };
            let span = name.span.join(mspan);
            return Expr {
                kind: ExprKind::MutArg { name },
                span,
            };
        }
        if mk == self.cx.k.tok_r_move {
            let mspan = self.cx.span_of(member);
            let span = recv.span.join(mspan);
            return Expr {
                span,
                kind: ExprKind::Method {
                    recv: Box::new(recv),
                    name: Ident {
                        name: "move".to_string(),
                        span: mspan,
                    },
                    args: Vec::new(),
                },
            };
        }
        // `.name`：record 族 = 字段读；text-only 面 = 无括号文本方法（旧拒绝）。
        let mspan = self.cx.span_of(member);
        let mname = self.cx.spelling(member);
        if mk == self.cx.k.tok_int_lit && mname.parse::<i32>().is_err() {
            self.cx.reject(mspan, || {
                Diagnostic::new(
                    Stage::Text,
                    "source.literal-range",
                    "object integer key exceeds i32 range",
                    mspan,
                )
            });
        }
        if self.cx.profile.is_record() {
            let span = recv.span.join(mspan);
            Expr {
                span,
                kind: ExprKind::Field {
                    base: Box::new(recv),
                    name: Ident {
                        name: mname,
                        span: mspan,
                    },
                },
            }
        } else {
            let dspan = self
                .cx
                .children(step)
                .into_iter()
                .find(|&l| self.cx.tree.kind(l) == self.cx.k.tok_dot)
                .map(|l| self.cx.span_of(l))
                .unwrap_or(mspan);
            let mname2 = mname.clone();
            self.cx.reject(dspan, move || {
                Diagnostic::new(
                    Stage::Text,
                    "source.syntax",
                    format!("method `.{}` requires parentheses", mname2),
                    dspan,
                )
            });
            let span = recv.span.join(mspan);
            Expr {
                span,
                kind: ExprKind::Field {
                    base: Box::new(recv),
                    name: Ident {
                        name: mname,
                        span: mspan,
                    },
                },
            }
        }
    }

    // ------------------------------------------------------------ calls

    fn enter_call(&mut self, node: u32) {
        let kids = self.cx.children(node);
        let callee_n = kids
            .iter()
            .copied()
            .find(|&c| self.cx.expression_kind(c) != self.cx.k.args)
            .expect("call callee");
        let args_n = kids
            .iter()
            .copied()
            .find(|&c| self.cx.expression_kind(c) == self.cx.k.args);
        let close_end = self.cx.span_of(node).end;
        let arg_nodes: Vec<u32> = args_n
            .map(|an| collect_seq(&self.cx, an, self.cx.k.arg, "arg-tail"))
            .unwrap_or_default();
        let argc = arg_nodes.len();
        // 旧 parser 怪癖保真（L04/L05/LSTR864/FIN864B 差分）：leading-dot
        // 空实参调用 `.m()` 走 continue-'outer 重入 primary——下一 token
        // 必须能起始 primary；且存在未括号待归约运算时锚定调用行（跨行
        // → unbracketed-newline；同行非 primary → expected expression）。
        if argc == 0 && self.callee_base_leading_dot(callee_n) {
            self.empty_value_quirk(close_end);
        }
        // callee 静态分析：expr 分派 → 一元链 → postfix/atom。callee 可能自身
        // 就是 unary_expr（pratt 把 primary 的 unary 节点作 callee——`!f(x)`
        // 的 Not/Neg 在此先收，is-digit 差分）；随后与原路径一致地下钻。
        let plan = {
            let mut unary_pre: Vec<UnaryOp> = Vec::new();
            let mut wrap = callee_n;
            while self.cx.expression_kind(wrap) == self.cx.k.unary_expr {
                let hm = self.cx.child_of_kind(wrap, self.cx.k.tok_minus).is_some();
                let hb = self.cx.child_of_kind(wrap, self.cx.k.tok_bang).is_some();
                if !hm && !hb {
                    break;
                }
                unary_pre.push(if hb { UnaryOp::Not } else { UnaryOp::Neg });
                match self
                    .cx
                    .child_of_kind(wrap, self.cx.k.unary_expr)
                    .or_else(|| self.cx.child_of_kind(wrap, self.cx.k.postfix_expr))
                {
                    Some(o) => wrap = o,
                    None => break,
                }
            }
            // PLAN-928（T-07）：括号组被调（paren-group callee）恒为间接调用
            // CallValue（镜像旧 CallFrame callee_expr——含 (f)(3) 形）。
            let mut paren_callee: Option<u32> = None;
            if self.cx.expression_kind(wrap) == self.cx.k.expr {
                paren_callee = Some(wrap);
            }
            if paren_callee.is_none()
                && self.cx.expression_kind(wrap) != KIND_POSTFIX
                && self.cx.expression_kind(wrap) != self.cx.k.postfix_expr
            {
                wrap = self
                    .cx
                    .children(wrap)
                    .into_iter()
                    .find(|&c| self.cx.tree.child_count(c) > 0)
                    .unwrap_or(wrap);
            }
            // 括号组 callee 在 atom 层显式探测（沿结构链降到 atom-expr，
            // 其 k.expr 孩子即括号组值位）。
            // PLAN-928（T-10 修正）：括号组 callee 在 atom 层显式探测（沿
            // 结构链降到 atom-expr，其 k.expr 孩子即括号组值位）。下降途中
            // 若穿过 postfix-step（如 `(a + "z").len()` 的 `.len`），括号组
            // 是**接收者基座**而非被调——不得强制 CallValue；仅当括号组是
            // 整个 callee（`(f)(3)` 形，无点步）才是间接调用。T-07 首版漏
            // 此区分，Text profile 的 `(组).方法()` 被误投 CallValue
            // （a2c_rt ledger 面实测暴露）。
            {
                let mut a = wrap;
                let mut through_postfix_step = false;
                loop {
                    let ka = self.cx.expression_kind(a);
                    if ka == self.cx.k.atom_expr {
                        break;
                    }
                    if ka == self.cx.k.unary_expr
                        || ka == self.cx.k.postfix_expr
                        || ka == KIND_POSTFIX
                    {
                        if let Some(step_kind) = self.cx.k.postfix_step {
                            if self
                                .cx
                                .children(a)
                                .into_iter()
                                .any(|c| self.cx.expression_kind(c) == step_kind)
                            {
                                through_postfix_step = true;
                            }
                        }
                        match self
                            .cx
                            .children(a)
                            .into_iter()
                            .find(|&c| self.cx.tree.child_count(c) > 0)
                        {
                            Some(c) => a = c,
                            None => break,
                        }
                    } else {
                        break;
                    }
                }
                if !through_postfix_step && self.cx.expression_kind(a) == self.cx.k.atom_expr {
                    if let Some(e) = self
                        .cx
                        .children(a)
                        .into_iter()
                        .find(|&c| self.cx.expression_kind(c) == self.cx.k.expr)
                    {
                        paren_callee = Some(e);
                    }
                }
            }
            let mut unary_ops: Vec<UnaryOp> = unary_pre;
            loop {
                if self.cx.expression_kind(wrap) == self.cx.k.unary_expr {
                    let has_minus = self.cx.child_of_kind(wrap, self.cx.k.tok_minus).is_some();
                    let has_bang = self.cx.child_of_kind(wrap, self.cx.k.tok_bang).is_some();
                    if !has_minus && !has_bang {
                        break;
                    }
                    unary_ops.push(if has_bang { UnaryOp::Not } else { UnaryOp::Neg });
                    let inner2 = self
                        .cx
                        .child_of_kind(wrap, self.cx.k.unary_expr)
                        .or_else(|| self.cx.child_of_kind(wrap, self.cx.k.postfix_expr));
                    match inner2 {
                        Some(i) => wrap = i,
                        None => break,
                    }
                } else {
                    break;
                }
            }
            if self.cx.expression_kind(wrap) == self.cx.k.postfix_expr
                || self.cx.expression_kind(wrap) == KIND_POSTFIX
            {
                let (base, steps) = self.split_postfix(wrap);
                // PLAN-928（T-07）：括号组 callee 优先于名字/方法分析。
                if let Some(e) = paren_callee {
                    CalleePlan::Value {
                        callee_n: e,
                        unary_ops,
                    }
                } else {
                    match steps.split_last() {
                        Some((last, mids)) => {
                            if self.step_is_index(*last) || self.step_is_dotquestion(*last) {
                                // PLAN-964: the call applies to the folded
                                // Index node (`fns["twice"](3)`) — the index
                                // step is NOT a member name; project the whole
                                // chain and dispatch as a value call.
                                CalleePlan::Value {
                                    callee_n: wrap,
                                    unary_ops,
                                }
                            } else {
                                CalleePlan::Method {
                                    base,
                                    member: self.step_member(*last).unwrap_or(*last),
                                    mids: mids.to_vec(),
                                    unary_ops,
                                }
                            }
                        }
                        None => {
                            // PLAN-928（T-03）：`List<T>.m(...)`——泛型接收原子自带
                            // 尾随 dot 成员（generic-atom）。
                            let ga = self.cx.children(base).into_iter().find(|&c| {
                                self.cx.expression_kind(c)
                                    == self.cx.k.generic_atom.unwrap_or(u32::MAX)
                            });
                            match ga.and_then(|g| self.cx.field_child(g, "member")) {
                                Some(member) => CalleePlan::Method {
                                    base,
                                    member,
                                    mids: Vec::new(),
                                    unary_ops,
                                },
                                None => match self.first_ident_opt(base) {
                                    Some(ident) => CalleePlan::Plain { ident, unary_ops },
                                    None => CalleePlan::Value {
                                        callee_n: base,
                                        unary_ops,
                                    },
                                },
                            }
                        }
                    }
                }
            } else if let Some(e) = paren_callee {
                CalleePlan::Value {
                    callee_n: e,
                    unary_ops,
                }
            } else {
                match self.first_ident_opt(wrap) {
                    Some(ident) => CalleePlan::Plain { ident, unary_ops },
                    None => CalleePlan::Value {
                        callee_n: wrap,
                        unary_ops,
                    },
                }
            }
        };
        // 压帧序（后压先弹）：接收者 base … 接收者折叠 … 实参任务 … 汇总帧。
        // 值栈布局：接收者值在实参值**之下**（assemble_call 先弹实参再弹接收者）。
        let push_args = |this: &mut Self, arg_nodes: Vec<u32>| {
            for a in arg_nodes.into_iter().rev() {
                if let Some(named) = this.cx.child_of_kind(a, this.cx.k.named_arg) {
                    let label_leaf = this.cx.field_child(named, "label").expect("arg label");
                    let label = ident_of(&mut this.cx, label_leaf);
                    let value_n = this.cx.field_child(named, "value").expect("arg value");
                    this.stack.push(Frame::ArgBuild { named: Some(label) });
                    this.stack.push(Frame::ExprEnter(value_n));
                } else if let Some(value_n) = this.cx.child_of_kind(a, this.cx.k.expr) {
                    this.stack.push(Frame::ArgBuild { named: None });
                    this.stack.push(Frame::ExprEnter(value_n));
                }
            }
        };
        match plan {
            CalleePlan::Value {
                callee_n,
                unary_ops,
            } => {
                self.stack.push(Frame::CallAssemble {
                    callee: CalleePlan::Value {
                        callee_n,
                        unary_ops,
                    },
                    argc,
                    close_end,
                });
                push_args(self, arg_nodes);
                self.stack.push(Frame::ExprEnter(callee_n));
            }
            CalleePlan::Method {
                base,
                member,
                mids,
                unary_ops,
            } => {
                let fold_mids = mids.clone();
                self.stack.push(Frame::CallAssemble {
                    callee: CalleePlan::Method {
                        base,
                        member,
                        mids,
                        unary_ops,
                    },
                    argc,
                    close_end,
                });
                push_args(self, arg_nodes);
                if !fold_mids.is_empty() {
                    self.stack.push(Frame::PostfixFoldAll { steps: fold_mids });
                }
                self.stack.push(Frame::ExprEnter(base));
            }
            plan @ CalleePlan::Plain { .. } => {
                self.stack.push(Frame::CallAssemble {
                    callee: plan,
                    argc,
                    close_end,
                });
                push_args(self, arg_nodes);
            }
        }
    }

    /// 旧 parser continue-'outer 怪癖（空实参 leading-dot 调用 / 空 ctor）：
    /// 值完成后重入 primary——下一 token 须能起始 primary；存在未括号待归约
    /// 运算时锚定值行（跨行 → unbracketed-newline；非 primary → expected
    /// expression）。L04/L05/LSTR864/FIN864B/edge47 差分。
    fn empty_value_quirk(&mut self, close_end: usize) {
        let close_end_u64 = close_end as u64;
        if let Some((g, text, span)) = self.cx.leaf_after(close_end_u64) {
            let pending_unbracketed = self.pending_unbracketed_op();
            let call_line = self.cx.line_of_pos(close_end_u64.saturating_sub(1));
            let next_line = self.cx.line_of_pos(span.start as u64);
            if pending_unbracketed && next_line > call_line {
                self.cx.reject(span, || {
                    Diagnostic::new(
                        Stage::Text,
                        "source.syntax",
                        "unbracketed expression cannot continue across a newline",
                        span,
                    )
                });
            } else {
                let k = &self.cx.k;
                let primary_start = matches!(
                    g,
                    x if x == k.tok_ident
                        || x == k.tok_int_lit
                        || x == k.tok_str_lit
                        || x == k.tok_char_lit
                        || x == k.tok_true
                        || x == k.tok_false
                        || x == k.tok_lparen
                        || x == k.tok_minus
                        || x == k.tok_bang
                        || (x == k.tok_dot && self.cx.profile.is_record())
                );
                if !primary_start {
                    let found = describe_g(g, &text, k);
                    self.cx.reject(span, move || {
                        Diagnostic::new(
                            Stage::Text,
                            "source.syntax",
                            format!("expected expression, found {found}"),
                            span,
                        )
                    });
                }
            }
        }
    }

    /// callee 链的 base atom 是否**裸 leading-dot**（`.m()` 的调用本体）。
    /// 有后缀步（`.text.len()`——调用挂在最后一步成员上）= false：
    /// 旧怪癖只覆盖 leading-dot 空 call 本体（p01 差分）。
    fn enter_place_assign_like(&mut self, inner: u32, name: Ident, first: Option<PlaceStep>) {
            let step_kind = self.cx.k.place_step.unwrap_or(u32::MAX);
            let mut step_nodes: Vec<u32> = self
                .cx
                .children(inner)
                .into_iter()
                .filter(|&c| self.cx.tree.kind(c) == step_kind)
                .collect();
            step_nodes.sort_by_key(|&n| self.cx.tree.abs_range(n).0);
            let k = &self.cx.k;
            // the compound operators live under the `place-assign-op`
            // rule child (plain `=` renders as the bare assign token).
            let op_node = self
                .cx
                .child_of_kind(inner, k.place_assign_op.unwrap_or(u32::MAX));
            let op = op_node
                .and_then(|on| {
                    if self
                        .cx
                        .child_of_kind(on, k.plus_assign.unwrap_or(u32::MAX))
                        .is_some()
                    {
                        Some(PlaceOp::Add)
                    } else if self
                        .cx
                        .child_of_kind(on, k.minus_assign.unwrap_or(u32::MAX))
                        .is_some()
                    {
                        Some(PlaceOp::Sub)
                    } else if self
                        .cx
                        .child_of_kind(on, k.star_assign.unwrap_or(u32::MAX))
                        .is_some()
                    {
                        Some(PlaceOp::Mul)
                    } else if self
                        .cx
                        .child_of_kind(on, k.slash_assign.unwrap_or(u32::MAX))
                        .is_some()
                    {
                        Some(PlaceOp::Div)
                    } else {
                        None
                    }
                })
                .unwrap_or(PlaceOp::Set);
            let mut steps: Vec<PlaceStep> = first.into_iter().collect();
            let mut index_nodes: Vec<u32> = Vec::new();
            for &sn in &step_nodes {
                if let Some(f) = self.cx.field_child(sn, "field") {
                    steps.push(PlaceStep::Field(ident_of(&mut self.cx, f)));
                } else if let Some(ix) = self.cx.field_child(sn, "index") {
                    index_nodes.push(ix);
                    steps.push(PlaceStep::Index(Expr {
                        span: self.cx.span_of(ix),
                        kind: ExprKind::IntLit(0),
                    }));
                }
            }
            let value_n = self
                .cx
                .field_child(inner, "value")
                .expect("place assign value");
            let span = self.cx.span_of(inner);
            self.stack.push(Frame::PlaceAssignFinish {
                name,
                steps,
                op,
                span,
            });
            self.stack.push(Frame::ExprEnter(value_n));
            for &ix in index_nodes.iter().rev() {
                self.stack.push(Frame::ExprEnter(ix));
            }
        }

    fn callee_base_leading_dot(&self, callee_n: u32) -> bool {
        let mut n = callee_n;
        for _ in 0..10 {
            let kind = self.cx.expression_kind(n);
            if kind == self.cx.k.postfix_expr || kind == KIND_POSTFIX {
                let (base, steps) = self.split_postfix(n);
                if !steps.is_empty() {
                    return false;
                }
                n = base;
                continue;
            }
            if kind == KIND_CALL {
                match self
                    .cx
                    .children(n)
                    .into_iter()
                    .find(|&c| self.cx.expression_kind(c) != self.cx.k.args)
                {
                    Some(c) => {
                        n = c;
                        continue;
                    }
                    None => return false,
                }
            }
            if kind == self.cx.k.atom_expr {
                return self
                    .cx
                    .child_of_kind(n, self.cx.k.leading_dot.unwrap_or(u32::MAX))
                    .is_some();
            }
            match self
                .cx
                .children(n)
                .into_iter()
                .find(|&c| self.cx.tree.child_count(c) > 0)
            {
                Some(c) => n = c,
                None => return false,
            }
        }
        false
    }

    /// 栈顶向下首个标记帧：OpPost/UnaryPost = 未括号待归约；ArgBuild/
    /// CtorFieldBuild = 括号内（实参/构造字段——旷新行为空白）。
    fn pending_unbracketed_op(&self) -> bool {
        for f in self.stack.iter().rev() {
            match f {
                Frame::OpPost { .. } | Frame::UnaryPost { .. } => return true,
                Frame::ArgBuild { .. } | Frame::CtorFieldBuild { .. } => return false,
                _ => {}
            }
        }
        false
    }

    /// PLAN-928（T-07）：callee 无 ident 叶（括号组/λ 值位被调）→ None。
    fn first_ident_opt(&self, node: u32) -> Option<Ident> {
        // 复用 first_ident 的下降逻辑但返回 Option。
        let mut stack = vec![node];
        while let Some(n) = stack.pop() {
            if self
                .cx
                .p
                .rules
                .iter()
                .position(|r| r.name == "qualified-name")
                .is_some_and(|r| {
                    self.cx.tree.kind(n) == ace_syntax_runtime::program::RULE_KIND_BASE + r as u32
                })
            {
                let a = self.cx.field_child(n, "module")?;
                let b = self.cx.field_child(n, "member")?;
                return Some(Ident {
                    name: format!("{}.{}", self.cx.spelling(a), self.cx.spelling(b)),
                    span: self.cx.span_of(a).join(self.cx.span_of(b)),
                });
            }
            if self.cx.tree.kind(n) == self.cx.k.tok_ident {
                let span = self.cx.span_of(n);
                let name = self.cx.spelling(n);
                return Some(Ident { name, span });
            }
            let kids = self.cx.children(n);
            for c in kids.into_iter().rev() {
                stack.push(c);
            }
        }
        None
    }

    fn describe_node(&self, node: u32) -> String {
        let mut n = node;
        loop {
            if self.cx.tree.node(n).child_count == 0 {
                break;
            }
            let kids = self.cx.children(n);
            match kids
                .iter()
                .find(|&&c| self.cx.tree.node(c).flags & FLAG_TRIVIA == 0)
            {
                Some(&c) => n = c,
                None => break,
            }
        }
        let kind = self.cx.tree.kind(n);
        let text = self.cx.spelling(n);
        if kind == self.cx.k.tok_ident {
            format!("`{}`", text)
        } else if kind == self.cx.k.tok_int_lit {
            "integer literal".to_string()
        } else if kind == self.cx.k.tok_str_lit {
            "string literal".to_string()
        } else if kind == self.cx.k.tok_char_lit {
            "char literal".to_string()
        } else if kind == self.cx.k.tok_affix_num {
            "suffixed/radix/float numeric literal".to_string()
        } else if kind == self.cx.k.tok_block_open {
            "block comment".to_string()
        } else {
            format!("`{}`", text)
        }
    }

    fn is_op_token(&self, kind: u32) -> bool {
        [
            self.cx.k.op_plus,
            self.cx.k.op_minus,
            self.cx.k.op_star,
            self.cx.k.op_slash,
            self.cx.k.op_percent,
            self.cx.k.op_lt,
            self.cx.k.op_gt,
            self.cx.k.op_le,
            self.cx.k.op_ge,
            self.cx.k.op_eq,
            self.cx.k.op_ne,
            self.cx.k.op_andand,
            self.cx.k.op_oror,
        ]
        .contains(&kind)
            // PLAN-961: the numeric bit-op tokens (numeric trim only).
            || self.cx.k.op_amp == Some(kind)
            || self.cx.k.op_vbar == Some(kind)
            || self.cx.k.op_caret == Some(kind)
            // PLAN-965: null-coalescing `??` (aggregates only).
            || self.cx.k.op_questionquestion == Some(kind)
    }

    fn op_of_token(&self, kind: u32) -> BinOp {
        if kind == self.cx.k.op_plus {
            BinOp::Add
        } else if kind == self.cx.k.op_minus {
            BinOp::Sub
        } else if kind == self.cx.k.op_star {
            BinOp::Mul
        } else if kind == self.cx.k.op_slash {
            BinOp::Div
        } else if kind == self.cx.k.op_percent {
            BinOp::Rem
        } else if kind == self.cx.k.op_lt {
            BinOp::Lt
        } else if kind == self.cx.k.op_gt {
            BinOp::Gt
        } else if kind == self.cx.k.op_le {
            BinOp::Le
        } else if kind == self.cx.k.op_ge {
            BinOp::Ge
        } else if kind == self.cx.k.op_eq {
            BinOp::Eq
        } else if kind == self.cx.k.op_ne {
            BinOp::Ne
        } else if kind == self.cx.k.op_andand {
            BinOp::And
        } else if kind == self.cx.k.op_oror {
            BinOp::Or
        } else if self.cx.k.op_amp == Some(kind) {
            BinOp::BitAnd
        } else if self.cx.k.op_vbar == Some(kind) {
            BinOp::BitOr
        } else if self.cx.k.op_caret == Some(kind) {
            BinOp::BitXor
        } else if self.cx.k.op_questionquestion == Some(kind) {
            BinOp::NullCoalesce
        } else {
            BinOp::Or
        }
    }

    // ------------------------------------------------------------ step

    fn enter_agg_is(&mut self, node: u32, is_expr: bool) {
        let scrutinee_n = self
            .cx
            .field_child(node, "scrutinee")
            .expect("agg is scrutinee");
        let arm_kind = self.cx.k.agg_match_arm.unwrap_or(u32::MAX);
        let mut arm_nodes: Vec<u32> = self
            .cx
            .children(node)
            .into_iter()
            .filter(|&c| self.cx.tree.kind(c) == arm_kind)
            .collect();
        arm_nodes.sort_by_key(|&n| self.cx.tree.abs_range(n).0);
        self.stack.push(Frame::AggIsArms(AggIsMatchCtx {
            is_expr,
            scrutinee: None,
            arms: Vec::new(),
            remaining: arm_nodes,
            node,
        }));
        self.stack.push(Frame::ExprEnter(scrutinee_n));
    }

    fn project_agg_pat(&mut self, pat_n: u32) -> MatchPattern {
        let k = &self.cx.k;
        for c in self.cx.children(pat_n) {
            let ck = self.cx.tree.kind(c);
            if ck == k.tok_else {
                return MatchPattern::Else;
            }
            if ck == k.tok_true {
                return MatchPattern::BoolLit(true);
            }
            if ck == k.tok_false {
                return MatchPattern::BoolLit(false);
            }
        }
        if let Some(lit_leaf) = self.cx.field_child(pat_n, "lit") {
            let neg = self.cx.child_of_kind(pat_n, k.tok_minus).is_some();
            let digits = self.cx.spelling(lit_leaf);
            let mag: u128 = digits.parse().unwrap_or(u128::MAX);
            let span = self.cx.span_of(pat_n);
            if neg {
                if mag > I32_MIN_MAG {
                    let d2 = digits.clone();
                    self.cx.reject(span, move || {
                        Diagnostic::new(
                            Stage::Text,
                            "source.literal-range",
                            format!("integer pattern `-{}` exceeds signed 32-bit range", d2),
                            span,
                        )
                    });
                }
                return MatchPattern::IntLit((mag as i128 * -1) as i32);
            }
            if mag > I32_MAX {
                let d2 = digits.clone();
                self.cx.reject(span, move || {
                    Diagnostic::new(
                        Stage::Text,
                        "source.literal-range",
                        format!("integer pattern `{}` exceeds signed 32-bit range", d2),
                        span,
                    )
                });
            }
            return MatchPattern::IntLit(mag as i32);
        }
        let qual = self
            .cx
            .field_child(pat_n, "qual")
            .and_then(|q| self.cx.field_child(q, "qual"))
            .map(|q| ident_of(&mut self.cx, q));
        let pat_leaf = self
            .cx
            .field_child(pat_n, "pattern")
            .expect("agg arm pattern");
        let name = ident_of(&mut self.cx, pat_leaf);
        // PLAN-1083: the `_` wildcard projects to the Else pattern (an
        // unconditional catch-all arm; must be last, like `else`).
        if name.name == "_" && qual.is_none() {
            return MatchPattern::Else;
        }
        // PLAN-1083: multi-binding destructuring `Arm(a, b, ..)` — the head
        // binding plus every comma tail ident, in source order.
        let mut bindings: Vec<Ident> = self
            .cx
            .field_child(pat_n, "payload")
            .and_then(|b| self.cx.field_child(b, "binding"))
            .map(|b| ident_of(&mut self.cx, b))
            .into_iter()
            .collect();
        if let Some(payload_n) = self.cx.field_child(pat_n, "payload") {
            let tail_kind = self.cx.k.arm_binding_tail.unwrap_or(u32::MAX);
            let mut tails: Vec<u32> = self
                .cx
                .children(payload_n)
                .into_iter()
                .filter(|&c| self.cx.tree.kind(c) == tail_kind)
                .collect();
            tails.sort_by_key(|&n| self.cx.tree.abs_range(n).0);
            let ident_kind = self.cx.k.tok_ident;
            for t in tails {
                if let Some(leaf) = self
                    .cx
                    .children(t)
                    .into_iter()
                    .find(|&c| self.cx.tree.kind(c) == ident_kind)
                {
                    bindings.push(ident_of(&mut self.cx, leaf));
                }
            }
        }
        MatchPattern::Variant {
            qual,
            name,
            bindings,
        }
    }

    fn step(&mut self, frame: Frame) {
        match frame {
            Frame::Block(bf) => self.step_block(bf),
            Frame::ForInBody { var, iter, node } => {
                let body = self.pop_b();
                let kw = self
                    .cx
                    .child_of_kind(node, self.cx.k.tok_for)
                    .map(|l| self.cx.span_of(l))
                    .unwrap_or(Span { start: 0, end: 0 });
                let span = kw.join(body.span);
                self.vals.push(Val::S(Stmt::ForIn {
                    var,
                    iter,
                    body,
                    span,
                }));
            }
            Frame::ForInExprBody { var, node } => {
                let iter = self.pop_e();
                let body = self.pop_b();
                let kw = self
                    .cx
                    .child_of_kind(node, self.cx.k.tok_for)
                    .map(|l| self.cx.span_of(l))
                    .unwrap_or(Span { start: 0, end: 0 });
                let span = kw.join(body.span);
                self.vals.push(Val::S(Stmt::ForIn {
                    var,
                    iter,
                    body,
                    span,
                }));
            }
            Frame::ForRangeBody {
                var,
                inclusive,
                node,
            } => {
                let end = self.pop_e();
                let start = self.pop_e();
                let body = self.pop_b();
                let kw = self
                    .cx
                    .child_of_kind(node, self.cx.k.tok_for)
                    .map(|l| self.cx.span_of(l))
                    .unwrap_or(Span { start: 0, end: 0 });
                let span = kw.join(body.span);
                self.vals.push(Val::S(Stmt::ForIn {
                    var,
                    iter: Expr {
                        span: start.span.join(end.span),
                        kind: ExprKind::Range {
                            start: Box::new(start),
                            end: Box::new(end),
                            inclusive,
                        },
                    },
                    body,
                    span,
                }));
            }
            Frame::LetDestructFinish { names, span } => {
                let value = self.pop_e();
                self.vals
                    .push(Val::S(Stmt::LetDestruct { names, value, span }));
            }
            Frame::HoldFinish { name, bind, span } => {
                let body = self.pop_b();
                self.vals.push(Val::S(Stmt::Hold {
                    name,
                    bind,
                    body,
                    span,
                }));
            }
            Frame::IndexAssignFinish { name, span } => {
                let value = self.pop_e();
                let index = self.pop_e();
                self.vals.push(Val::S(Stmt::IndexAssign(IndexAssign {
                    base: name,
                    index,
                    value,
                    span,
                })));
            }
            Frame::PlaceAssignFinish {
                name,
                mut steps,
                op,
                span,
            } => {
                // vals 栈自底向上：idx1..idxn（源序）、value —— 弹出 value 后
                // 逐个弹出 index，按声明序回填进步骤占位。
                let value = self.pop_e();
                let n_idx = steps
                    .iter()
                    .filter(|st| matches!(st, PlaceStep::Index(_)))
                    .count();
                let mut pending = Vec::new();
                for _ in 0..n_idx {
                    pending.push(self.pop_e());
                }
                for st in steps.iter_mut() {
                    if let PlaceStep::Index(slot) = st {
                        *slot = pending.pop().expect("place index expr");
                    }
                }
                self.vals.push(Val::S(Stmt::PlaceAssign(PlaceAssign {
                    base: name,
                    steps,
                    op,
                    value,
                    span,
                })));
            }
            Frame::IsArms(ctx) => {
                if let Some(&arm_n) = ctx.remaining.first() {
                    let mut rest = ctx.remaining.clone();
                    rest.remove(0);
                    let qual = self
                        .cx
                        .field_child(arm_n, "qual")
                        .and_then(|q| self.cx.field_child(q, "qual"))
                        .map(|q| ident_of(&mut self.cx, q));
                    let pattern_leaf = self.cx.field_child(arm_n, "pattern").expect("arm pattern");
                    let pattern = ident_of(&mut self.cx, pattern_leaf);
                    let binding = self
                        .cx
                        .field_child(arm_n, "payload")
                        .and_then(|b| self.cx.field_child(b, "binding"))
                        .map(|b| ident_of(&mut self.cx, b));
                    let body_n = self.cx.field_child(arm_n, "body").expect("arm body");
                    // 旧合同：臂 span 恒从模式词起（限定 `Enum.Name` 亦然）。
                    let start = pattern.span;
                    self.stack.push(Frame::IsArmBlock {
                        ctx: Box::new(IsMatchCtx {
                            remaining: rest,
                            ..ctx
                        }),
                        qual,
                        pattern,
                        binding,
                        kw: start,
                    });
                    self.enter_block(body_n);
                } else {
                    let span = self.cx.span_of(ctx.node);
                    self.vals.push(Val::S(Stmt::IsMatch {
                        scrutinee: ctx.scrutinee,
                        arms: ctx.arms,
                        span,
                    }));
                }
            }
            Frame::IsArmBlock {
                ctx,
                qual,
                pattern,
                binding,
                kw,
            } => {
                let body = self.pop_b();
                let span = kw.join(body.span);
                let mut ctx = *ctx;
                ctx.arms.push(MatchArm {
                    qual,
                    pattern,
                    binding,
                    body,
                    span,
                });
                self.stack.push(Frame::IsArms(ctx));
            }
            Frame::AggIsArms(mut ctx) => {
                if ctx.scrutinee.is_none() {
                    ctx.scrutinee = Some(self.pop_e());
                }
                if let Some(&arm_n) = ctx.remaining.first() {
                    let mut rest = ctx.remaining.clone();
                    rest.remove(0);
                    let pat_n = self.cx.field_child(arm_n, "pat").expect("agg arm pat");
                    let pat = self.project_agg_pat(pat_n);
                    // PLAN-1083: or-arm tails `A | B -> body` — every extra
                    // pattern shares this rhs; the arity check is bind-time
                    // per expanded arm (projection keeps the source order).
                    let or_tail_kind = self.cx.k.agg_or_arm_tail.unwrap_or(u32::MAX);
                    let mut or_tails: Vec<u32> = self
                        .cx
                        .children(arm_n)
                        .into_iter()
                        .filter(|&c| self.cx.tree.kind(c) == or_tail_kind)
                        .collect();
                    or_tails.sort_by_key(|&n| self.cx.tree.abs_range(n).0);
                    let mut pats = vec![pat];
                    for t in or_tails {
                        if let Some(p) = self
                            .cx
                            .children(t)
                            .into_iter()
                            .find(|&c| self.cx.tree.kind(c) == self.cx.k.agg_arm_pat.unwrap_or(u32::MAX))
                        {
                            pats.push(self.project_agg_pat(p));
                        }
                    }
                    let guard_expr_n = self
                        .cx
                        .field_child(arm_n, "guard")
                        .and_then(|g| self.cx.field_child(g, "cond"));
                    let rhs_n = self.cx.field_child(arm_n, "rhs").expect("agg arm rhs");
                    let arm_start = self.cx.span_of(pat_n);
                    let statement_n = self.cx.field_child(rhs_n, "stmt");
                    let body_n = self.cx.field_child(rhs_n, "body");
                    self.stack.push(Frame::AggIsArmRhs {
                        ctx: Box::new(AggIsMatchCtx {
                            remaining: rest,
                            ..ctx
                        }),
                        pats,
                        arm_start,
                        is_block: body_n.is_some(),
                        has_guard: guard_expr_n.is_some(),
                        statement_span: statement_n.map(|stmt| self.cx.span_of(stmt)),
                    });
                    if let Some(g) = guard_expr_n {
                        self.stack.push(Frame::ExprEnter(g));
                    }
                    if let Some(body_n) = body_n {
                        self.enter_block(body_n);
                    } else if statement_n.is_some() {
                        self.enter_stmt(rhs_n, false);
                    } else {
                        let expr_n = self.cx.field_child(rhs_n, "expr").expect("agg arm expr");
                        self.stack.push(Frame::ExprEnter(expr_n));
                    }
                } else {
                    let scrutinee = ctx.scrutinee.expect("agg is scrutinee");
                    let span = self.cx.span_of(ctx.node);
                    if ctx.is_expr {
                        self.vals.push(Val::E(Expr {
                            span,
                            kind: ExprKind::IsExpr {
                                scrutinee: Box::new(scrutinee),
                                arms: ctx.arms,
                            },
                        }));
                    } else {
                        self.vals.push(Val::S(Stmt::AggIsMatch {
                            scrutinee,
                            arms: ctx.arms,
                            span,
                        }));
                    }
                }
            }
            Frame::AggIsArmRhs {
                ctx,
                pats,
                arm_start,
                is_block,
                has_guard,
                statement_span,
            } => {
                // PLAN-1083: the guard value (pushed above the rhs frames)
                // pops first; it rides every expanded or-arm copy.
                let guard = if has_guard { Some(self.pop_e()) } else { None };
                let (body, end_span) = if is_block {
                    let b = self.pop_b();
                    let s = b.span;
                    (AggArmBody::Block(b), s)
                } else if let Some(span) = statement_span {
                    let statement = self.pop_s();
                    (
                        AggArmBody::Block(Block {
                            stmts: vec![statement],
                            span,
                        }),
                        span,
                    )
                } else {
                    let e = self.pop_e();
                    let span = e.span;
                    (AggArmBody::Expr(e), span)
                };
                let mut ctx = *ctx;
                let span = arm_start.join(end_span);
                for pat in pats {
                    ctx.arms.push(AggMatchArm {
                        pat,
                        body: body.clone(),
                        guard: guard.clone(),
                        span,
                    });
                }
                self.stack.push(Frame::AggIsArms(ctx));
            }
            Frame::TryFinish {
                has_catch,
                catch_bind,
                catch_span,
                has_finally,
                kw_span,
            } => {
                let finally = if has_finally {
                    Some(self.pop_b())
                } else {
                    None
                };
                let catch = if has_catch {
                    let body = self.pop_b();
                    Some(CatchClause {
                        span: catch_span.join(body.span),
                        bind: catch_bind,
                        body,
                    })
                } else {
                    None
                };
                let body = self.pop_b();
                let mut end_span = body.span;
                if let Some(c) = &catch {
                    end_span = c.span;
                }
                if let Some(f) = &finally {
                    end_span = f.span;
                }
                self.vals.push(Val::S(Stmt::Try {
                    body,
                    catch,
                    finally,
                    span: kw_span.join(end_span),
                }));
            }
            Frame::WithFinish { kw_span, bind } => {
                let expr = self.pop_e();
                let body = self.pop_b();
                let body_span = body.span;
                self.vals.push(Val::S(Stmt::With {
                    expr,
                    bind,
                    body,
                    span: kw_span.join(body_span),
                }));
            }
            Frame::YieldFinish { kw_span } => {
                let value = self.pop_e();
                let span = kw_span.join(value.span);
                self.vals.push(Val::S(Stmt::Yield { value, span }));
            }
            Frame::LambdaBlockBody {
                move_capture,
                params,
                result,
                start,
            } => {
                let body = self.pop_b();
                let span = start.join(body.span);
                self.vals.push(Val::E(Expr {
                    span,
                    kind: ExprKind::Closure {
                        move_capture,
                        params,
                        result,
                        body: Box::new(ClosureBody::Block(body)),
                    },
                }));
            }
            Frame::LambdaExprBody {
                move_capture,
                params,
                start,
            } => {
                let value = self.pop_e();
                let span = start.join(value.span);
                self.vals.push(Val::E(Expr {
                    span,
                    kind: ExprKind::Closure {
                        move_capture,
                        params,
                        result: None,
                        body: Box::new(ClosureBody::Expr(value)),
                    },
                }));
            }
            Frame::LetFinish {
                node,
                kw,
                name,
                ann,
                mutable,
            } => {
                let value = self.pop_e();
                let span = Span {
                    start: self.cx.span_of(node).start,
                    end: value.span.end,
                };
                self.vals.push(Val::S(Stmt::Let {
                    kw,
                    name,
                    ann,
                    mutable,
                    value,
                    span,
                }));
            }
            Frame::GlobalItemFinish {
                node,
                name,
                ann,
                mutable,
            } => {
                let value = self.pop_e();
                let span = Span {
                    start: self.cx.span_of(node).start,
                    end: value.span.end,
                };
                let item = if mutable {
                    Item::Store(StoreItem {
                        name,
                        ty: ann,
                        value,
                        span,
                    })
                } else {
                    Item::Const(ConstItem {
                        name,
                        ty: ann,
                        value,
                        span,
                    })
                };
                self.global_item = Some(item);
            }
            Frame::AssignFinish { name, path } => {
                let value = self.pop_e();
                let span = Span {
                    start: name.span.start,
                    end: value.span.end,
                };
                self.vals.push(Val::S(Stmt::Assign {
                    target: AssignTarget { base: name, path },
                    value,
                    span,
                }));
            }
            Frame::ReturnFinish { kw } => {
                let value = self.pop_e();
                let span = kw.join(value.span);
                self.vals.push(Val::S(Stmt::Return { value, span }));
            }
            Frame::IfCond {
                then_n,
                els_blk,
                node,
            } => {
                let cond = self.pop_e();
                self.stack.push(Frame::IfThen {
                    cond,
                    els_blk,
                    node,
                });
                self.enter_block(then_n);
            }
            Frame::IfThen {
                cond,
                els_blk,
                node,
            } => {
                let then = self.pop_b();
                match els_blk {
                    Some(eb) => {
                        self.stack.push(Frame::IfElse { cond, then, node });
                        if self.cx.tree.kind(eb) == self.cx.k.if_stmt {
                            self.stack.push(Frame::ElseIfBlock {
                                span: self.cx.span_of(eb),
                            });
                            self.enter_if(eb);
                        } else {
                            self.enter_block(eb);
                        }
                    }
                    None => {
                        let kw = self
                            .cx
                            .child_of_kind(node, self.cx.k.tok_if)
                            .map(|l| self.cx.span_of(l))
                            .unwrap_or(Span { start: 0, end: 0 });
                        let span = kw.join(then.span);
                        self.vals.push(Val::S(Stmt::If {
                            cond,
                            then,
                            els: None,
                            span,
                        }));
                    }
                }
            }
            Frame::ElseIfBlock { span } => {
                let branch = self.pop_s();
                self.vals.push(Val::B(Block {
                    stmts: vec![branch],
                    span,
                }));
            }
            Frame::IfElse { cond, then, node } => {
                let els = self.pop_b();
                let kw = self
                    .cx
                    .child_of_kind(node, self.cx.k.tok_if)
                    .map(|l| self.cx.span_of(l))
                    .unwrap_or(Span { start: 0, end: 0 });
                let span = kw.join(then.span).join(els.span);
                self.vals.push(Val::S(Stmt::If {
                    cond,
                    then,
                    els: Some(els),
                    span,
                }));
            }
            Frame::ScopeFinish => {
                let then = self.pop_b();
                let span = then.span;
                // A bare statement block uses the existing lexical branch scope.
                let cond = Expr {
                    kind: ExprKind::BoolLit(true),
                    span,
                };
                self.vals.push(Val::S(Stmt::If {
                    cond,
                    then,
                    els: None,
                    span,
                }));
            }
            Frame::IfValueCond {
                span,
                then_node,
                else_node,
            } => {
                let cond = self.pop_e();
                self.stack.push(Frame::IfValueThen {
                    span,
                    cond,
                    else_node,
                });
                self.enter_block(then_node);
            }
            Frame::IfValueThen {
                span,
                cond,
                else_node,
            } => {
                let then_block = self.pop_b();
                self.stack.push(Frame::IfValueElse {
                    span,
                    cond,
                    then_block,
                });
                self.enter_block(else_node);
            }
            Frame::IfValueElse {
                span,
                cond,
                then_block,
            } => {
                let else_block = self.pop_b();
                let arms = [(true, then_block), (false, else_block)]
                    .into_iter()
                    .map(|(value, block)| AggMatchArm {
                        pat: MatchPattern::BoolLit(value),
                        span: block.span,
                        body: AggArmBody::Block(block),
                        guard: None,
                    })
                    .collect();
                self.vals.push(Val::E(Expr {
                    kind: ExprKind::IsExpr {
                        scrutinee: Box::new(cond),
                        arms,
                    },
                    span,
                }));
            }
            Frame::CtIfThen { cond, els_n, span } => {
                let then = self.pop_b();
                match els_n {
                    Some(eb) => {
                        self.stack.push(Frame::CtIfElse { cond, then, span });
                        self.enter_block(eb);
                    }
                    None => self.vals.push(Val::S(Stmt::HashIf {
                        cond,
                        then,
                        els: None,
                        span,
                    })),
                }
            }
            Frame::CtIfElse { cond, then, span } => {
                let els = self.pop_b();
                self.vals.push(Val::S(Stmt::HashIf {
                    cond,
                    then,
                    els: Some(els),
                    span,
                }));
            }
            Frame::CtForBody {
                var,
                start,
                end,
                span,
            } => {
                let body = self.pop_b();
                self.vals.push(Val::S(Stmt::HashFor {
                    var,
                    start,
                    end,
                    body,
                    span,
                }));
            }
            Frame::CtIsArmBlock { mut ctx, pattern } => {
                let body = self.pop_b();
                let arm_span = body.span;
                ctx.arms.push(HashIsArm {
                    pattern,
                    body,
                    span: arm_span,
                });
                if ctx.pending.is_empty() {
                    let CtIsCtx {
                        scrutinee,
                        arms,
                        span,
                        ..
                    } = *ctx;
                    self.vals.push(Val::S(Stmt::HashIs {
                        scrutinee,
                        arms,
                        span,
                    }));
                } else {
                    self.start_ct_arm(ctx);
                }
            }
            Frame::MacroCallFinish { name, argc, span } => {
                let mut args: Vec<Arg> = Vec::with_capacity(argc);
                for _ in 0..argc {
                    args.push(Arg::Pos(self.pop_e()));
                }
                args.reverse();
                if self.cx.profile.is_auto_language() {
                    let log = match name.name.as_str() {
                        "debug" => Some(("print", "[DEBUG] ")),
                        "info" => Some(("print", "[INFO] ")),
                        "warn" => Some(("print", "[WARN] ")),
                        "error" => Some(("eprint", "[ERROR] ")),
                        _ => None,
                    };
                    if let Some((callee, prefix)) = log {
                        if argc != 1 {
                            self.cx.reject(span, || {
                                Diagnostic::new(
                                    Stage::Text,
                                    "source.cte.log-arity",
                                    "a log macro takes one text argument",
                                    span,
                                )
                            });
                            self.vals.push(Val::S(Stmt::Break { span }));
                            return;
                        }
                        let Arg::Pos(message) = args.pop().expect("one log argument") else {
                            unreachable!("macro arguments are positional");
                        };
                        let value = Expr {
                            span,
                            kind: ExprKind::Call {
                                callee: Ident {
                                    name: callee.into(),
                                    span: name.span,
                                },
                                args: vec![Arg::Pos(Expr {
                                    span: message.span,
                                    kind: ExprKind::Binary {
                                        op: BinOp::Add,
                                        lhs: Box::new(Expr {
                                            span: name.span,
                                            kind: ExprKind::StrLit(prefix.into()),
                                        }),
                                        rhs: Box::new(message),
                                    },
                                })],
                            },
                        };
                        self.vals.push(Val::S(Stmt::ExprStmt { value, span }));
                        return;
                    }
                }
                let value = Expr {
                    span,
                    kind: ExprKind::Call { callee: name, args },
                };
                // PLAN-966（T-02）：丢弃值语义在本面不合法（PLAN-862 只允许
                // 脚本尾表达式），宏调用因此落成卫生局部绑定：名字取保留前缀
                // `__ace_macro_<源位置>`，用户自定义同名会撞绑定错误而不是被
                // 静默捕获（V-01）。
                let local = Ident {
                    name: format!("__ace_macro_{}", span.start),
                    span,
                };
                self.vals.push(Val::S(Stmt::Let {
                    kw: span,
                    name: local,
                    ann: None,
                    mutable: false,
                    value,
                    span,
                }));
            }
            Frame::WhileCond { body_n, node } => {
                let cond = self.pop_e();
                self.stack.push(Frame::WhileBody { cond, node });
                self.enter_block(body_n);
            }
            Frame::WhileBody { cond, node } => {
                let body = self.pop_b();
                let kw_kind = if self.cx.tree.kind(node) == self.cx.k.while_stmt.unwrap_or(u32::MAX)
                {
                    self.cx.k.tok_while
                } else {
                    self.cx.k.tok_for
                };
                let kw = self
                    .cx
                    .child_of_kind(node, kw_kind)
                    .map(|l| self.cx.span_of(l))
                    .unwrap_or(Span { start: 0, end: 0 });
                let span = kw.join(body.span);
                // 条件 for 复用 While AST 形状（旧合同）。
                self.vals.push(Val::S(Stmt::While { cond, body, span }));
            }
            Frame::LoopBody { node } => {
                let body = self.pop_b();
                let kw = self
                    .cx
                    .child_of_kind(node, self.cx.k.tok_loop)
                    .map(|l| self.cx.span_of(l))
                    .unwrap_or(Span { start: 0, end: 0 });
                let span = kw.join(body.span);
                self.vals.push(Val::S(Stmt::Loop { body, span }));
            }
            Frame::ExprStmtFinish => {
                let value = self.pop_e();
                let span = value.span;
                self.vals.push(Val::S(Stmt::ExprStmt { value, span }));
            }
            Frame::FnBodyFinish {
                node,
                name,
                type_params,
                params,
                mut ret,
                receiver,
                owner,
            } => {
                let mut body = self.pop_b();
                let span = self.cx.span_of(node);
                if self.cx.profile.is_auto_language()
                    && self
                        .cx
                        .spelling_by_range(ret.span.start as u64, ret.span.end as u64)
                        .starts_with('~')
                {
                    if let TypeRef::Generic { name: future, args } = &ret.kind {
                        if future.name == "Future" {
                            let result = args.first().cloned().expect("async result type");
                            if name.name == "main" && receiver == Receiver::Free {
                                // The AV entry itself runs on a scheduler chain.
                                ret = result;
                            } else {
                                let value = async_future(body, Some(result), span);
                                body = Block {
                                    stmts: vec![Stmt::Return { value, span }],
                                    span,
                                };
                            }
                        }
                    }
                }
                self.vals.push(Val::F(FnItem {
                    span,
                    name,
                    type_params,
                    params,
                    ret,
                    body,
                    receiver,
                    owner,
                    script: false,
                    proto: false,
                    test_attribute: None,
                }));
            }
            Frame::NestedFnFinish { saved_ty_params } => {
                // PLAN-1081 (T-03)：弹 Val::F 包装为 NestedFn，恢复类型变量域。
                let item = match self.vals.pop() {
                    Some(Val::F(f)) => f,
                    _ => FnItem {
                        name: Ident {
                            name: String::new(),
                            span: Span { start: 0, end: 0 },
                        },
                        type_params: Vec::new(),
                        params: Vec::new(),
                        ret: TypeAnn {
                            kind: TypeRef::Builtin(TypeWord::Int),
                            span: Span { start: 0, end: 0 },
                        },
                        body: Block {
                            stmts: Vec::new(),
                            span: Span { start: 0, end: 0 },
                        },
                        span: Span { start: 0, end: 0 },
                        receiver: Receiver::Free,
                        owner: None,
                        script: false,
                        proto: false,
                        test_attribute: None,
                    },
                };
                self.cx.ty_params = saved_ty_params;
                self.vals.push(Val::S(Stmt::NestedFn { item }));
            }
            Frame::ScriptFinish { start_span } => {
                let mut body = self.pop_b();
                let span = start_span.join(body.span);
                body.span = span;
                self.vals.push(Val::F(FnItem {
                    name: Ident {
                        name: "main".to_string(),
                        span: start_span,
                    },
                    type_params: Vec::new(),
                    params: Vec::new(),
                    ret: TypeAnn {
                        kind: TypeRef::Builtin(TypeWord::Void),
                        span: start_span,
                    },
                    body,
                    span,
                    receiver: Receiver::Free,
                    owner: None,
                    script: true,
                    proto: false,
                    test_attribute: None,
                }));
            }
            Frame::ExprEnter(n) => self.enter_expr(n),
            Frame::ArrayLiteralFinish { mut span, count } => {
                let mut elements = Vec::with_capacity(count);
                for _ in 0..count {
                    let element = self.pop_e();
                    span = span.join(element.span);
                    elements.push(element);
                }
                elements.reverse();
                self.vals.push(Val::E(Expr { span, kind: ExprKind::ArrayLit(elements) }));
            }
            Frame::ArrayConditionalElemFinish { span } => {
                let value = Box::new(self.pop_e());
                let cond = Box::new(self.pop_e());
                self.vals.push(Val::E(Expr { span, kind: ExprKind::ArrayConditionalElem { cond, value } }));
            }
            Frame::ObjectLiteralFinish { mut span, fields } => {
                let mut entries = Vec::with_capacity(fields.len());
                for (key, field_span) in fields.into_iter().rev() {
                    let value = self.pop_e();
                    span = span.join(value.span);
                    entries.push(ObjectField { key, value, span: field_span });
                }
                entries.reverse();
                self.vals.push(Val::E(Expr { span, kind: ExprKind::ObjectLit { fields: entries } }));
            }
            Frame::HashExpr { span } => {
                let inner = self.pop_e();
                self.vals.push(Val::E(Expr {
                    span,
                    kind: ExprKind::HashExpr {
                        expr: Box::new(inner),
                    },
                }));
            }
            Frame::OpPost { op } => {
                let rhs = self.pop_e();
                let lhs = self.pop_e();
                let span = lhs.span.join(rhs.span);
                self.vals.push(Val::E(Expr {
                    span,
                    kind: ExprKind::Binary {
                        op,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    },
                }));
            }
            Frame::UnaryPost { op } => {
                let operand = self.pop_e();
                let span = operand.span;
                self.vals.push(Val::E(Expr {
                    span,
                    kind: ExprKind::Unary {
                        op,
                        operand: Box::new(operand),
                    },
                }));
            }
            Frame::GoStmtFinish { kw } => {
                let operand = self.pop_e();
                let span = kw.join(operand.span);
                self.vals.push(Val::S(Stmt::ExprStmt {
                    value: Expr {
                        span,
                        kind: ExprKind::Go(Box::new(operand)),
                    },
                    span,
                }));
            }
            Frame::ReplyStmtFinish { kw } => {
                let operand = self.pop_e();
                let span = kw.join(operand.span);
                let e = Expr {
                    span,
                    kind: ExprKind::Call {
                        callee: Ident {
                            name: "__sched_reply".to_string(),
                            span: kw,
                        },
                        args: vec![Arg::Pos(operand)],
                    },
                };
                self.vals.push(Val::S(Stmt::ExprStmt { value: e, span }));
            }
            Frame::AsyncBlockFinish { span } => {
                let body = self.pop_b();
                self.vals.push(Val::E(async_future(body, None, span)));
            }
            Frame::SchedPrefixFinish { is_await, kw_span } => {
                let operand = self.pop_e();
                let span = kw_span.join(operand.span);
                self.vals.push(Val::E(Expr {
                    span,
                    kind: if is_await {
                        ExprKind::Await(Box::new(operand))
                    } else {
                        ExprKind::Go(Box::new(operand))
                    },
                }));
            }
            Frame::PostfixFoldAll { steps } => {
                let recv = self.pop_e();
                let e = self.fold_postfix(recv, steps);
                self.vals.push(Val::E(e));
            }
            Frame::CallAssemble {
                callee,
                argc,
                close_end,
            } => {
                let e = self.assemble_call(callee, argc, close_end);
                self.vals.push(Val::E(e));
            }
            Frame::ArgBuild { named } => {
                let value = self.pop_e();
                let arg = match named {
                    Some(name) => Arg::Named {
                        span: name.span.join(value.span),
                        name,
                        value,
                    },
                    None => Arg::Pos(value),
                };
                self.vals.push(Val::A(arg));
            }
            Frame::CtorFieldBuild { label } => {
                let value = self.pop_e();
                let span = label.span.join(value.span);
                self.vals.push(Val::CF(NamedCtorField {
                    name: label,
                    value,
                    span,
                }));
            }
            Frame::CtorFinish {
                node,
                ty,
                args,
                count,
            } => {
                let mut fields = Vec::with_capacity(count);
                for _ in 0..count {
                    match self.vals.pop() {
                        Some(Val::CF(f)) => fields.push(f),
                        _ => {}
                    }
                }
                fields.reverse();
                let span = self.cx.span_of(node);
                self.vals.push(Val::E(Expr {
                    span,
                    kind: ExprKind::CtorNamed { ty, args, fields },
                }));
            }
        }
    }

    /// 调用组装（CalleePlan 已含静态分析；接收者值在实参之下）。
    fn assemble_call(&mut self, callee: CalleePlan, argc: usize, close_end: usize) -> Expr {
        let mut args = Vec::with_capacity(argc);
        for _ in 0..argc {
            match self.vals.pop() {
                Some(Val::A(a)) => args.push(a),
                _ => {}
            }
        }
        args.reverse();
        let is_record = self.cx.profile.is_record();
        let is_text = self.cx.profile.is_text();
        let mut e = match callee {
            CalleePlan::Plain { ident, unary_ops } => {
                let mut e = Expr {
                    span: Span {
                        start: ident.span.start,
                        end: close_end,
                    },
                    kind: ExprKind::Call {
                        callee: ident,
                        args,
                    },
                };
                for op in unary_ops.iter().rev() {
                    let sp = e.span;
                    e = Expr {
                        span: sp,
                        kind: ExprKind::Unary {
                            op: *op,
                            operand: Box::new(e),
                        },
                    };
                }
                e
            }
            CalleePlan::Value { unary_ops, .. } => {
                let callee = self.pop_e();
                let mut e = Expr {
                    span: Span {
                        start: callee.span.start,
                        end: close_end,
                    },
                    kind: ExprKind::CallValue {
                        callee: Box::new(callee),
                        args,
                    },
                };
                for op in unary_ops.iter().rev() {
                    let sp = e.span;
                    e = Expr {
                        span: sp,
                        kind: ExprKind::Unary {
                            op: *op,
                            operand: Box::new(e),
                        },
                    };
                }
                e
            }
            CalleePlan::Method {
                member,
                mids: _,
                unary_ops,
                ..
            } => {
                let mut recv = self.pop_e();
                // PLAN-928（T-03）：`List<T>.m(...)`——泛型接收原子投影为
                // Field{GenericType}，此处解包为直接 GenericType 接收者
                // （镜像旧 postfix 折叠的 MethodCall 形状）。
                {
                    let want = self.cx.spelling(member).to_string();
                    let unwrapped = match &recv.kind {
                        ExprKind::Field { base, name }
                            if matches!(base.kind, ExprKind::GenericType { .. })
                                && name.name == want =>
                        {
                            Some(*base.clone())
                        }
                        _ => None,
                    };
                    if let Some(r) = unwrapped {
                        recv = r;
                    }
                }
                let mk = self.cx.tree.kind(member);
                let mspan = self.cx.span_of(member);
                let mname = self.cx.spelling(member);
                // type-level 接收者：char.from_codepoint（text 面与 closures/aggregates 含 char 词域）。
                let recv =
                    if (is_text || self.cx.profile.is_closures()) && mname == "from_codepoint" {
                        let mut r = recv;
                        if let ExprKind::Var(id) = &r.kind {
                            if id.name == "char" {
                                r = Expr {
                                    span: id.span,
                                    kind: ExprKind::TypeWordArg(TypeWord::Char),
                                };
                            }
                        }
                        r
                    } else {
                        recv
                    };
                let full = Span {
                    start: recv.span.start,
                    end: close_end,
                };
                let mut e = if mk == self.cx.k.tok_mut {
                    self.cx.reject(mspan, || {
                        Diagnostic::new(
                            Stage::Text,
                            "source.unsupported",
                            "`.mut` applies to a local variable",
                            mspan,
                        )
                    });
                    Expr {
                        span: full,
                        kind: ExprKind::MutArg {
                            name: Ident {
                                name: mname,
                                span: mspan,
                            },
                        },
                    }
                } else if is_record {
                    Expr {
                        span: full,
                        kind: ExprKind::MethodCall {
                            instance: None,
                            recv: Box::new(recv),
                            name: Ident {
                                name: mname,
                                span: mspan,
                            },
                            args,
                        },
                    }
                } else {
                    Expr {
                        span: full,
                        kind: ExprKind::Method {
                            recv: Box::new(recv),
                            name: Ident {
                                name: mname,
                                span: mspan,
                            },
                            args,
                        },
                    }
                };
                for op in unary_ops.iter().rev() {
                    let sp = e.span;
                    e = Expr {
                        span: sp,
                        kind: ExprKind::Unary {
                            op: *op,
                            operand: Box::new(e),
                        },
                    };
                }
                e
            }
        };
        let _ = &mut e;
        e
    }
}

/// 序列收集：列表节点直接孩子 + tail 包裹（`X many { X-tail { , X } }`）内
/// 的项目节点，按文档序（参数/实参/构造字段共用）。
fn collect_seq(cx: &Cx, list_node: u32, item_kind: u32, tail_name: &str) -> Vec<u32> {
    let tail_kind =
        cx.p.rules
            .iter()
            .position(|r| r.name == tail_name)
            .map(|i| ace_syntax_runtime::program::RULE_KIND_BASE + i as u32);
    let mut out: Vec<u32> = Vec::new();
    for c in cx.children(list_node) {
        let ck = cx.tree.kind(c);
        if ck == item_kind {
            out.push(c);
        } else if Some(ck) == tail_kind {
            for g in cx.children(c) {
                if cx.tree.kind(g) == item_kind {
                    out.push(g);
                }
            }
        }
    }
    out.sort_by_key(|&n| cx.tree.abs_range(n).0);
    out
}

/// PLAN-969 (AUDIT-03): the structured surface fact for an unexpected
/// token at a statement/expression boundary — the wider-profile-owned
/// token spellings (`[`, `..`, `..=`) and the named-ctor `{` shape. This
/// replaces message-substring routing: the escalation ladders read THIS.
fn surface_shape_of_token(text: &str) -> Option<SurfaceFact> {
    match text {
        "{" => Some(SurfaceFact::OwnedShape),
        "[" | ".." | "..=" => Some(SurfaceFact::OwnedToken),
        _ => None,
    }
}

fn format_diag(message: impl Into<String>, span: Span) -> Diagnostic {
    Diagnostic::new(Stage::Text, "source.fstr-syntax", message, span)
}

fn syntax_diag(msg: &str, span: Span) -> Diagnostic {
    Diagnostic::new(Stage::Text, "source.syntax", msg.to_string(), span)
}

fn unsupported_ty(word: &str, span: Span) -> Diagnostic {
    Diagnostic::new(
        Stage::Text,
        "source.unsupported",
        format!("type `{}` is not part of this source profile", word),
        span,
    )
    .tagged(SurfaceFact::ProfileSurface)
}

fn self_method_name(span: Span) -> Diagnostic {
    Diagnostic::new(
        Stage::Text,
        "source.unsupported",
        "`self` is the implicit receiver of an instance method and cannot be a method name",
        span,
    )
}

fn self_param(span: Span) -> Diagnostic {
    Diagnostic::new(
        Stage::Text,
        "source.unsupported",
        "`self` is the implicit receiver of an instance method and cannot be declared as a parameter",
        span,
    )
}

// ---------------------------------------------------------------------------
// PLAN-963（T-03）：spec 声明与 `as` 遵从（投影期擦除）
//
// spec 不是运行期对象：投影期把 `spec` 声明收进侧表，`type X as S` 在此核验
// 并注入默认方法，随后 spec 与遵从列表对下游不可见。方法调用仍走既有的静态
// 方法解析——不需要 witness/dyn 表示，也不引入第二套语义。
// ---------------------------------------------------------------------------

/// Spec declarations are checked here. The ordinary surface can produce
/// concrete dispatch variants from the verified conformance declarations.
struct SpecDecl {
    name: Ident,
    span: Span,
    /// PLAN-963（T-03）：通用 spec 的类型参数（`spec Storage<T>`）；实例化
    /// 发生在 `as Storage<int>` 处，注入方法已是具体签名。
    type_params: Vec<Ident>,
    /// PLAN-963（T-05）：`spec Tool: Send + Sync` 的 super-spec（书写序）。
    /// 遵从 `Tool` 即须同时满足其 super-spec 的方法义务（传递闭包）。
    supers: Vec<Ident>,
    /// PLAN-963（T-05）：关联类型名（`type Item`）——实现者须在遵从列表
    /// 以命名实参 `as Container<Item=int>` 绑定。
    assoc: Vec<Ident>,
    methods: Vec<SpecMethodDecl>,
}

/// 投影侧 `ext Type { .. }`（不进 AST；方法在单态化/遵从检查前合入该类型）。
struct ExtDecl {
    ty: Ident,
    methods: Vec<FnItem>,
}

/// PLAN-963（T-03）：把扩展块的方法合入已有类型；未知类型/重复方法具名拒。
fn merge_ext_blocks(items: &mut Vec<Item>, exts: &[ExtDecl]) -> Vec<Diagnostic> {
    if exts.is_empty() {
        return Vec::new();
    }
    let mut diags: Vec<Diagnostic> = Vec::new();
    for ext in exts {
        let idx = items
            .iter()
            .position(|i| matches!(i, Item::Type(t) if t.name.name == ext.ty.name));
        let Some(idx) = idx else {
            let sp = ext.ty.span;
            let n = ext.ty.name.clone();
            diags.push(Diagnostic::new(
                Stage::Text,
                "source.unsupported",
                format!("unknown type `{n}` in an `ext` block"),
                sp,
            ));
            continue;
        };
        let Item::Type(t) = &mut items[idx] else {
            continue;
        };
        let owner = t.name.name.clone();
        for m in &ext.methods {
            if t.methods
                .iter()
                .any(|f| f.name.name == m.name.name && f.receiver == m.receiver)
            {
                let sp = m.name.span;
                let mn = m.name.name.clone();
                diags.push(Diagnostic::new(
                    Stage::Text,
                    "source.duplicate",
                    format!("`fn {mn}` is declared more than once for type `{owner}`"),
                    sp,
                ));
                continue;
            }
            t.methods.push(m.clone());
        }
    }
    diags
}

/// spec 方法签名：抽象（无体）或默认（有体，可被实现者的显式方法覆盖）。
struct SpecMethodDecl {
    name: Ident,
    params: Vec<Param>,
    ret: TypeAnn,
    body: Option<Block>,
    span: Span,
}

/// PLAN-963（T-03）：核验每个声明遵从的类型，并把默认方法注入到**具体**
/// 记录（单态化之后，故合成实例同样被核验）。返回具名诊断，不静默通过。
/// PLAN-963（T-03）：把 spec 方法的签名/默认体里的类型参数替换为实参。
/// `subst` 为空（普通 spec）时原样返回——零行为改动。
fn instantiate_spec_method(
    m: &SpecMethodDecl,
    subst: &BTreeMap<String, TypeAnn>,
    diags: &mut Vec<Diagnostic>,
) -> (Vec<Param>, TypeAnn, Option<Block>) {
    if subst.is_empty() {
        return (m.params.clone(), m.ret.clone(), m.body.clone());
    }
    let mut st = TypeSubst {
        map: subst.clone(),
        diags: Vec::new(),
        depth: 0,
        cycle_code: "source.unsupported",
        cycle_msg: "instantiating this spec does not terminate",
    };
    let mut params = m.params.clone();
    for p in params.iter_mut() {
        p.ty = st.ann(&p.ty);
    }
    let ret = st.ann(&m.ret);
    let mut body = m.body.clone();
    if let Some(b) = body.as_mut() {
        st.rewrite_block(b);
    }
    diags.append(&mut st.diags);
    (params, ret, body)
}

fn apply_spec_conformance(
    items: &mut [Item],
    specs: &[SpecDecl],
    ordinary_auto: bool,
) -> Vec<Diagnostic> {
    let any_conform = items
        .iter()
        .any(|i| matches!(i, Item::Type(t) if !t.conforms.is_empty()));
    if specs.is_empty() && !any_conform {
        return Vec::new();
    }
    let mut diags: Vec<Diagnostic> = Vec::new();
    let mut by_name: BTreeMap<&str, &SpecDecl> = BTreeMap::new();
    for s in specs {
        if by_name.contains_key(s.name.name.as_str()) {
            let sp = s.name.span;
            let n = s.name.name.clone();
            diags.push(Diagnostic::new(
                Stage::Text,
                "source.duplicate",
                format!("spec `{n}` is declared more than once"),
                sp,
            ));
            continue;
        }
        by_name.insert(s.name.name.as_str(), s);
    }
    for item in items.iter_mut() {
        let Item::Type(t) = item else { continue };
        if t.conforms.is_empty() {
            continue;
        }
        let owner = t.name.name.clone();
        let owner_span = t.name.span;
        let mut seen: Vec<String> = Vec::new();
        // 每条遵从声明 →（被遵从 spec、方法义务闭包、具体化替换表）。
        let mut claims: Vec<(&SpecDecl, Vec<&SpecDecl>, BTreeMap<String, TypeAnn>)> = Vec::new();
        // 关联类型名 → 绑定类型（跨 spec 汇总；实现者方法/字段里的 assoc 名按此替换）。
        let mut assoc_bound: BTreeMap<String, TypeAnn> = BTreeMap::new();
        for conform in t.conforms.iter() {
            let spec_ident = &conform.name;
            if seen.contains(&spec_ident.name) {
                let sp = spec_ident.span;
                let n = spec_ident.name.clone();
                diags.push(Diagnostic::new(
                    Stage::Text,
                    "source.duplicate",
                    format!("spec `{n}` is listed more than once for type `{owner}`"),
                    sp,
                ));
                continue;
            }
            seen.push(spec_ident.name.clone());
            let Some(spec) = by_name.get(spec_ident.name.as_str()).copied() else {
                let sp = spec_ident.span;
                let n = spec_ident.name.clone();
                diags.push(Diagnostic::new(
                    Stage::Text,
                    "source.unsupported",
                    format!("unknown spec `{n}`"),
                    sp,
                ));
                continue;
            };
            // PLAN-963（T-03）：通用 spec 实例化——实参 arity 必须与声明一致；
            // 随后把 spec 方法的类型参数替换成实参（默认体与签名一起具体化）。
            if spec.type_params.len() != conform.args.len() {
                let sp = spec_ident.span;
                let n = spec_ident.name.clone();
                diags.push(Diagnostic::new(
                    Stage::Text,
                    "source.unsupported",
                    format!(
                        "spec `{n}` expects {} type argument(s), found {}",
                        spec.type_params.len(),
                        conform.args.len()
                    ),
                    sp,
                ));
                continue;
            }
            // PLAN-963（T-05）：方法义务 = 自身 + super-spec 传递闭包。
            let closure = spec_obligations(spec, &by_name, &mut diags);
            // 关联类型名（闭包内去重，声明序）。
            let mut assoc_names: Vec<Ident> = Vec::new();
            for s in &closure {
                for a in &s.assoc {
                    if !assoc_names.iter().any(|x| x.name == a.name) {
                        assoc_names.push(a.clone());
                    }
                }
            }
            let spec_name = spec.name.name.clone();
            let mut bound: BTreeMap<String, TypeAnn> = BTreeMap::new();
            // 绑定本身不合法时不再做该方法义务核验：否则会以未替换的 `Item`
            // 签名报“签名不符”，把真正的绑定错误挤掉（first_diag 只回一条）。
            let mut binding_ok = true;
            for (k, v) in conform.assoc.iter() {
                if !assoc_names.iter().any(|a| a.name == k.name) {
                    binding_ok = false;
                    let sp = k.span;
                    let kn = k.name.clone();
                    let sn = spec_name.clone();
                    diags.push(Diagnostic::new(
                        Stage::Text,
                        "source.unsupported",
                        format!("spec `{sn}` has no associated type `{kn}`"),
                        sp,
                    ));
                    continue;
                }
                if bound.contains_key(&k.name) {
                    binding_ok = false;
                    let sp = k.span;
                    let kn = k.name.clone();
                    let sn = spec_name.clone();
                    diags.push(Diagnostic::new(
                        Stage::Text,
                        "source.duplicate",
                        format!("associated type `{kn}` is bound more than once for spec `{sn}`"),
                        sp,
                    ));
                    continue;
                }
                bound.insert(k.name.clone(), v.clone());
            }
            for a in &assoc_names {
                if !bound.contains_key(&a.name) {
                    binding_ok = false;
                    let sp = spec_ident.span;
                    let an = a.name.clone();
                    let sn = spec_name.clone();
                    diags.push(Diagnostic::new(
                        Stage::Text,
                        "source.unsupported",
                        format!(
                            "type `{owner}` does not bind associated type `{an}` required by spec `{sn}`"
                        ),
                        sp,
                    ));
                }
            }
            for (k, v) in bound.iter() {
                match assoc_bound.get(k) {
                    Some(prev) if prev.kind != v.kind => {
                        diags.push(Diagnostic::new(
                            Stage::Text,
                            "source.unsupported",
                            format!(
                                "associated type `{k}` is bound to different types by the declared \
                                 specs of type `{owner}`"
                            ),
                            v.span,
                        ));
                    }
                    Some(_) => {}
                    None => {
                        assoc_bound.insert(k.clone(), v.clone());
                    }
                }
            }
            let mut subst: BTreeMap<String, TypeAnn> = spec
                .type_params
                .iter()
                .zip(conform.args.iter())
                .map(|(p, a)| (p.name.clone(), a.clone()))
                .collect();
            for (k, v) in bound {
                subst.insert(k, v);
            }
            if binding_ok {
                claims.push((spec, closure, subst));
            }
        }
        // PLAN-963（T-05）：实现者自身的签名/体/字段里保留关联类型名时（旧 F1
        // 行为），按绑定替换成具体类型；无关联类型绑定时零行为改动。
        if !assoc_bound.is_empty() {
            let mut st = TypeSubst {
                map: assoc_bound.clone(),
                diags: Vec::new(),
                depth: 0,
                cycle_code: "source.unsupported",
                cycle_msg: "expanding an associated type does not terminate",
            };
            for f in t.fields.iter_mut() {
                f.ty = st.ann(&f.ty);
            }
            for f in t.methods.iter_mut() {
                st.rewrite_fn(f);
            }
            diags.append(&mut st.diags);
        }
        let mut injected: Vec<FnItem> = Vec::new();
        // 同一 spec 的方法义务只处理一次（直接遵从与经 super-spec 到达同一 spec）。
        let mut done: Vec<String> = Vec::new();
        for (_, closure, subst) in claims {
            for spec in closure {
                if done.iter().any(|d| d == &spec.name.name) {
                    continue;
                }
                done.push(spec.name.name.clone());
                for m in &spec.methods {
                    let (m_params, m_ret, m_body) = instantiate_spec_method(m, &subst, &mut diags);
                    let own = t.methods.iter().find(|f| {
                        f.name.name == m.name.name && f.receiver == Receiver::View
                    });
                    let default = injected.iter().find(|f| f.name.name == m.name.name);
                    if ordinary_auto && own.is_none() && default.is_some() && m_body.is_some() {
                        diags.push(Diagnostic::new(
                            Stage::Bind,
                            "ambiguous-candidates",
                            format!(
                                "type `{owner}` inherits more than one default for `fn {}`",
                                m.name.name
                            ),
                            m.name.span,
                        ));
                        continue;
                    }
                    if let Some(f) = own.or(default) {
                        // 显式实现优先于默认体；签名必须与 spec 一致（按类型
                        // kind 比较，span 不参与语义身份）。
                        let same = f.params.len() == m_params.len()
                            && f.ret.kind == m_ret.kind
                            && f.params
                                .iter()
                                .zip(m_params.iter())
                                .all(|(a, b)| a.ty.kind == b.ty.kind);
                        if !same {
                            let sp = m.name.span;
                            let mn = m.name.name.clone();
                            let sn = spec.name.name.clone();
                            diags.push(Diagnostic::new(
                                Stage::Text,
                                "source.unsupported",
                                format!(
                                    "`fn {mn}` in type `{owner}` does not match the signature \
                                     required by spec `{sn}`"
                                ),
                                sp,
                            ));
                        }
                        continue;
                    }
                    match m_body {
                        Some(body) => injected.push(FnItem {
                            name: m.name.clone(),
                            type_params: Vec::new(),
                            params: m_params,
                            ret: m_ret,
                            body,
                            span: m.span,
                            receiver: Receiver::View,
                            owner: Some(Ident {
                                name: owner.clone(),
                                span: owner_span,
                            }),
                            script: false,
                            proto: false,
                            test_attribute: None,
                        }),
                        None => {
                            let sp = m.name.span;
                            let mn = m.name.name.clone();
                            let sn = spec.name.name.clone();
                            diags.push(Diagnostic::new(
                                Stage::Text,
                                "source.unsupported",
                                format!(
                                    "type `{owner}` does not implement `fn {mn}` required by spec `{sn}`"
                                ),
                                sp,
                            ));
                        }
                    }
                }
            }
        }
        t.methods.extend(injected);
        if ordinary_auto {
            t.spec_memberships = done;
        }
    }
    diags
}

fn materialize_spec_values(items: &mut Vec<Item>, specs: &[SpecDecl]) -> Vec<Diagnostic> {
    let by_name: BTreeMap<_, _> = specs.iter().map(|s| (s.name.name.as_str(), s)).collect();
    let mut diags = Vec::new();
    let mut witnesses: BTreeMap<&str, Vec<Ident>> = BTreeMap::new();
    for record in items.iter().filter_map(|item| match item {
        Item::Type(record) if record.type_params.is_empty() => Some(record),
        _ => None,
    }) {
        let mut seen = BTreeSet::new();
        for claim in &record.conforms {
            let Some(spec) = by_name.get(claim.name.name.as_str()) else {
                continue;
            };
            for inherited in spec_obligations(spec, &by_name, &mut diags) {
                if seen.insert(inherited.name.name.as_str())
                    && spec_obligations(inherited, &by_name, &mut diags)
                        .iter()
                        .all(|s| s.type_params.is_empty() && s.assoc.is_empty())
                {
                    witnesses
                        .entry(inherited.name.name.as_str())
                        .or_default()
                        .push(record.name.clone());
                }
            }
        }
    }
    for spec in specs {
        let Some(records) = witnesses.get(spec.name.name.as_str()) else {
            continue;
        };
        let arms = records
            .iter()
            .map(|record| EnumArm {
                name: record.clone(),
                payload: Some(TypeAnn {
                    span: record.span,
                    kind: TypeRef::Record(record.clone()),
                }),
                fields: Vec::new(),
                disc: None,
                span: record.span,
            })
            .collect();
        let mut seen = BTreeSet::new();
        let methods = spec_obligations(spec, &by_name, &mut diags)
            .into_iter()
            .flat_map(|obligation| &obligation.methods)
            .filter(|method| seen.insert(method.name.name.as_str()))
            .map(|method| spec_dispatch_method(spec, method, records))
            .collect();
        items.push(Item::Enum(EnumItem {
            name: spec.name.clone(),
            type_params: Vec::new(),
            instance: None,
            spec_witnesses: true,
            arms,
            methods,
            span: spec.span,
        }));
    }
    diags
}

fn spec_dispatch_method(spec: &SpecDecl, method: &SpecMethodDecl, records: &[Ident]) -> FnItem {
    let span = method.span;
    let binding = Ident {
        name: "$spec_receiver".to_owned(),
        span,
    };
    let arms = records
        .iter()
        .map(|record| {
            let args = method
                .params
                .iter()
                .map(|param| {
                    let value = Expr {
                        kind: ExprKind::Var(param.name.clone()),
                        span: param.name.span,
                    };
                    if param.mode == ParamMode::Move {
                        Arg::MovePos(value)
                    } else {
                        Arg::Pos(value)
                    }
                })
                .collect();
            let value = Expr {
                kind: ExprKind::MethodCall {
                    recv: Box::new(Expr {
                        kind: ExprKind::Var(binding.clone()),
                        span,
                    }),
                    name: method.name.clone(),
                    args,
                    instance: None,
                },
                span,
            };
            AggMatchArm {
                pat: MatchPattern::Variant {
                    qual: Some(spec.name.clone()),
                    name: record.clone(),
                    bindings: vec![binding.clone()],
                },
                body: AggArmBody::Block(Block {
                    stmts: vec![Stmt::Return { value, span }],
                    span,
                }),
                guard: None,
                span,
            }
        })
        .collect();
    FnItem {
        name: method.name.clone(),
        type_params: Vec::new(),
        params: method.params.clone(),
        ret: TypeAnn {
            kind: match &method.ret.kind {
                TypeRef::Builtin(TypeWord::Str) => TypeRef::Builtin(TypeWord::StrOwned),
                kind => kind.clone(),
            },
            span: method.ret.span,
        },
        body: Block {
            stmts: vec![Stmt::AggIsMatch {
                scrutinee: Expr {
                    kind: ExprKind::Var(Ident {
                        name: "self".to_owned(),
                        span,
                    }),
                    span,
                },
                arms,
                span,
            }],
            span,
        },
        span,
        receiver: Receiver::View,
        owner: Some(spec.name.clone()),
        script: false,
        proto: false,
        test_attribute: None,
    }
}

/// PLAN-963（T-05）：spec 及其 super-spec 的方法义务闭包（自身在前，随后按
/// 书写序深度优先，重复名去重）。未知 super-spec / 通用 super-spec / 成环均
/// 具名拒——不把未知 bound 当“无约束”静默放行。
fn spec_obligations<'a>(
    spec: &'a SpecDecl,
    by_name: &BTreeMap<&str, &'a SpecDecl>,
    diags: &mut Vec<Diagnostic>,
) -> Vec<&'a SpecDecl> {
    fn walk<'a>(
        s: &'a SpecDecl,
        by_name: &BTreeMap<&str, &'a SpecDecl>,
        out: &mut Vec<&'a SpecDecl>,
        path: &mut Vec<String>,
        diags: &mut Vec<Diagnostic>,
    ) {
        if out.iter().any(|o| o.name.name == s.name.name) {
            return;
        }
        out.push(s);
        path.push(s.name.name.clone());
        for sup in &s.supers {
            if path.iter().any(|p| p == &sup.name) {
                let sp = sup.span;
                let a = sup.name.clone();
                let b = s.name.name.clone();
                diags.push(Diagnostic::new(
                    Stage::Text,
                    "source.unsupported",
                    format!("super-spec cycle: `{b}` requires `{a}`"),
                    sp,
                ));
                continue;
            }
            let Some(next) = by_name.get(sup.name.as_str()).copied() else {
                let sp = sup.span;
                let a = sup.name.clone();
                let b = s.name.name.clone();
                diags.push(Diagnostic::new(
                    Stage::Text,
                    "source.unsupported",
                    format!("unknown super-spec `{a}` of spec `{b}`"),
                    sp,
                ));
                continue;
            };
            if !next.type_params.is_empty() {
                let sp = sup.span;
                let a = sup.name.clone();
                let b = s.name.name.clone();
                diags.push(Diagnostic::new(
                    Stage::Text,
                    "source.unsupported",
                    format!(
                        "super-spec `{a}` of spec `{b}` is generic; super-spec bounds take no \
                         type arguments"
                    ),
                    sp,
                ));
                continue;
            }
            walk(next, by_name, out, path, diags);
        }
        path.pop();
    }
    let mut out: Vec<&SpecDecl> = Vec::new();
    let mut path: Vec<String> = Vec::new();
    walk(spec, by_name, &mut out, &mut path, diags);
    out
}

// ---------------------------------------------------------------------------
// PLAN-963（T-02）：用户泛型记录类型的语法期单态化
//
// 每个使用位都显式写出类型实参（`Point<int>`），因此不需要语义推理：按
// 声明序替换字段/方法注解，合成具体 `TypeItem`（`Point_int`），并移除模板。
// 下游 resolve/HIR/布局/四端只见具体 nominal 记录——不引入第二套类型系统，
// 也不让模板 TyVar 抵达 resolve（那会是具名拒绝而非静默回退）。
// ---------------------------------------------------------------------------

/// 单态化用户泛型记录；无泛型声明时零行为改动。
fn expand_user_generic_types(
    items: &mut Vec<Item>,
    specs: &[SpecDecl],
    ordinary: bool,
) -> Vec<Diagnostic> {
    let keep_templates = ordinary && has_dependent_record_uses(items);
    expand_user_generic_types_with_origins(items, specs, keep_templates, ordinary).0
}

fn has_dependent_record_uses(items: &[Item]) -> bool {
    items.iter().any(|item| match item {
        Item::Fn(f) => !f.type_params.is_empty(),
        Item::Type(t) => t.methods.iter().any(|m| !m.type_params.is_empty()),
        Item::Enum(e) => {
            !e.type_params.is_empty() || e.methods.iter().any(|m| !m.type_params.is_empty())
        }
        _ => false,
    })
}

/// PLAN-963（V-G06）：包模块扁平程序的记录模板单态化入口。返回（诊断、
/// 实例符号 → 模板符号），后者供绑定层补 `type_origins` 并把诊断归到模块。
pub(crate) fn monomorphize_package_program(
    items: &mut Vec<Item>,
    ordinary: bool,
) -> (Vec<Diagnostic>, BTreeMap<String, String>) {
    // 包模块面无 spec（`spec` 不在 packages-modules 裁剪内），遵从核验空转。
    let specs: Vec<SpecDecl> = Vec::new();
    let keep_templates = ordinary && has_dependent_record_uses(items);
    expand_user_generic_types_with_origins(items, &specs, keep_templates, ordinary)
}

fn expand_user_generic_types_with_origins(
    items: &mut Vec<Item>,
    specs: &[SpecDecl],
    keep_templates: bool,
    ordinary: bool,
) -> (Vec<Diagnostic>, BTreeMap<String, String>) {
    // PLAN-963（T-03）：`<T has S>` 的核验面——bound 名必须是已声明 spec，
    // 实参必须是遵从该 spec 的记录（合成实例的遵从列表随模板复制）。
    let spec_names: BTreeSet<String> = specs.iter().map(|s| s.name.name.clone()).collect();
    let mut record_specs: BTreeMap<String, Vec<ConformRef>> = BTreeMap::new();
    for item in items.iter() {
        if let Item::Type(t) = item {
            record_specs.insert(t.name.name.clone(), t.conforms.clone());
        }
    }
    let mut templates: BTreeMap<String, TypeItem> = BTreeMap::new();
    for item in items.iter() {
        if let Item::Type(t) = item {
            if !t.type_params.is_empty() {
                templates.insert(t.name.name.clone(), t.clone());
            }
        }
    }
    if templates.is_empty() {
        // 没有泛型模板仍要拒绝显式类型实参构造（`Plain<int> { .. }`）——不
        // 静默丢实参；重写遍为空，只跑构造检查。
        let mut gt = GenericExpand {
            templates,
            keep_templates,
            ordinary,
            spec_names,
            record_specs,
            ..GenericExpand::default()
        };
        for it in items.iter_mut() {
            gt.resolve_typed_ctors_in_item(it);
        }
        return (gt.diags, gt.origin);
    }
    let taken: BTreeSet<String> = items
        .iter()
        .filter_map(|i| match i {
            Item::Type(t) => Some(t.name.name.clone()),
            _ => None,
        })
        .collect();
    let mut gt = GenericExpand {
        templates,
        taken,
        keep_templates,
        ordinary,
        spec_names,
        record_specs,
        ..GenericExpand::default()
    };
    gt.rewrite_program(items);
    // PLAN-963（T-02）：显式类型实参构造（`Point<int> { .. }`）覆盖注解
    // 换不到的位置；本遍可能合成新实例（其方法体同样要过这一遍），按批
    // 循环直到没有新增。
    for it in items.iter_mut() {
        gt.resolve_typed_ctors_in_item(it);
    }
    gt.finish(items);
    items.retain(|i| match i {
        Item::Type(t) => keep_templates || t.type_params.is_empty(),
        _ => true,
    });
    (gt.diags, gt.origin)
}

pub fn materialize_auto_record_instances(
    program: &mut Program,
    templates: &BTreeMap<String, TypeItem>,
) -> Result<BTreeMap<String, String>, Vec<Diagnostic>> {
    if templates.is_empty() {
        return Ok(BTreeMap::new());
    }
    let mut gt = GenericExpand {
        templates: templates.clone(),
        ordinary: true,
        ..GenericExpand::default()
    };
    gt.taken.extend(templates.keys().cloned());
    for item in &program.items {
        let Item::Type(t) = item else { continue };
        gt.taken.insert(t.name.name.clone());
        gt.spec_names.extend(t.spec_memberships.iter().cloned());
        gt.record_specs
            .insert(t.name.name.clone(), t.conforms.clone());
        if let Some(instance) = &t.instance {
            let spellings: Option<Vec<_>> = instance.arguments.iter().map(type_spelling).collect();
            if let Some(spellings) = spellings {
                gt.by_key.insert(
                    format!("{}<{}>", instance.template, spellings.join(",")),
                    t.name.name.clone(),
                );
                gt.origin
                    .insert(t.name.name.clone(), instance.template.clone());
            }
        }
    }
    gt.rewrite_program(&mut program.items);
    for item in &mut program.items {
        gt.resolve_typed_ctors_in_item(item);
    }
    gt.finish(&mut program.items);
    if gt.diags.is_empty() {
        Ok(gt.origin)
    } else {
        Err(gt.diags)
    }
}

impl GenericExpand {
    fn finish(&mut self, items: &mut Vec<Item>) {
        let mut made = std::mem::take(&mut self.made);
        let mut done: Vec<TypeItem> = Vec::new();
        while !made.is_empty() {
            for mut t in made.drain(..) {
                for m in &mut t.methods {
                    self.resolve_typed_ctors_in_block(&mut m.body);
                }
                done.push(t);
            }
            made = std::mem::take(&mut self.made);
        }
        items.extend(done.into_iter().map(Item::Type));
    }
}

// ---------------------------------------------------------------------------
// PLAN-963（T-03）：类型别名（`type Name = T`，投影期展开）
//
// 别名不是运行期对象：投影期收进侧表，随后把**每个类型位置**的别名引用替换为
// 目标（字段/参数/返回/局部注解/常量与 store 注解；目标内部的类型实参位同样
// 递归）。别名可在使用点之后声明；展开先于泛型记录单态化，故别名目标里的
// `Point<int>` 落在使用位后再被换名。重复名与环具名拒，不静默取其一。
// ---------------------------------------------------------------------------

/// 投影侧 `type Name = T`（不进 AST；展开后程序里只剩具体类型）。
struct TypeAliasDecl {
    name: Ident,
    target: TypeAnn,
}

fn expand_type_aliases(
    items: &mut Vec<Item>,
    aliases: &[TypeAliasDecl],
    explicit_type_arguments: bool,
) -> Vec<Diagnostic> {
    if aliases.is_empty() {
        return Vec::new();
    }
    let mut diags: Vec<Diagnostic> = Vec::new();
    let declared_types: BTreeSet<String> = items
        .iter()
        .filter_map(|i| match i {
            Item::Type(t) => Some(t.name.name.clone()),
            _ => None,
        })
        .collect();
    let mut map: BTreeMap<String, TypeAnn> = BTreeMap::new();
    for a in aliases {
        let taken = declared_types.contains(&a.name.name) || map.contains_key(&a.name.name);
        let span = a.name.span;
        let name = a.name.name.clone();
        if taken {
            diags.push(Diagnostic::new(
                Stage::Text,
                "source.duplicate",
                format!("`{name}` is declared more than once"),
                span,
            ));
            continue;
        }
        map.insert(name, a.target.clone());
    }
    if !diags.is_empty() {
        return diags;
    }
    let mut ex = TypeSubst {
        map,
        diags: Vec::new(),
        depth: 0,
        cycle_code: "source.alias-cycle",
        cycle_msg: "type alias cycle: expanding the alias chain does not terminate",
    };
    ex.rewrite_program(items);
    if explicit_type_arguments {
        // The expression walk used for generic types also visits explicit
        // type arguments. Expand aliases before materializing any records.
        let mut walk = GenericExpand {
            aliases: Some(ex),
            ..GenericExpand::default()
        };
        for item in items {
            walk.resolve_typed_ctors_in_item(item);
        }
        ex = walk.aliases.expect("alias substitution");
    }
    diags.append(&mut ex.diags);
    diags
}

struct TypeSubst {
    map: BTreeMap<String, TypeAnn>,
    diags: Vec<Diagnostic>,
    depth: usize,
    /// 环诊断（别名展开与通用 spec 实例化共用这一个替换器）。
    cycle_code: &'static str,
    cycle_msg: &'static str,
}

impl TypeSubst {
    /// 把一处注解里的名字引用展开成目标（递归；链与目标内部实参都覆盖）。
    fn ann(&mut self, a: &TypeAnn) -> TypeAnn {
        if self.depth > 32 {
            // 替换链过深（成环）：具名拒，不无限展开。
            self.diags.push(Diagnostic::new(
                Stage::Text,
                self.cycle_code,
                self.cycle_msg,
                a.span,
            ));
            return a.clone();
        }
        match &a.kind {
            // PLAN-1080：Inferred 无别名面（推断标记不在此解析）。
            TypeRef::Inferred => a.clone(),
            TypeRef::Record(id) => match self.map.get(&id.name).cloned() {
                Some(target) => {
                    self.depth += 1;
                    let out = self.ann(&target);
                    self.depth -= 1;
                    // 使用位保留自己的 span（诊断指向使用处而不是别名声明）。
                    TypeAnn {
                        kind: out.kind,
                        span: a.span,
                    }
                }
                None => a.clone(),
            },
            TypeRef::Optional(inner) => {
                let inner = self.ann(inner);
                TypeAnn {
                    kind: TypeRef::Optional(Box::new(inner)),
                    span: a.span,
                }
            }
            TypeRef::ResultBang(inner) => {
                let inner = self.ann(inner);
                TypeAnn {
                    kind: TypeRef::ResultBang(Box::new(inner)),
                    span: a.span,
                }
            }
            TypeRef::Stream(inner) => {
                let inner = self.ann(inner);
                TypeAnn {
                    kind: TypeRef::Stream(Box::new(inner)),
                    span: a.span,
                }
            }
            TypeRef::Tuple(xs) => {
                let xs = xs.iter().map(|x| self.ann(x)).collect();
                TypeAnn {
                    kind: TypeRef::Tuple(xs),
                    span: a.span,
                }
            }
            TypeRef::Generic { name, args } => {
                let args = args.iter().map(|x| self.ann(x)).collect();
                TypeAnn {
                    kind: TypeRef::Generic {
                        name: name.clone(),
                        args,
                    },
                    span: a.span,
                }
            }
            TypeRef::Function { params, result } => {
                let params = params.iter().map(|x| self.ann(x)).collect();
                let result = Box::new(self.ann(result));
                TypeAnn {
                    kind: TypeRef::Function { params, result },
                    span: a.span,
                }
            }
            TypeRef::Builtin(_) | TypeRef::TyVar(_) => a.clone(),
        }
    }

    fn rewrite_program(&mut self, items: &mut Vec<Item>) {
        for item in items.iter_mut() {
            match item {
                Item::Fn(f) => self.rewrite_fn(f),
                Item::Type(t) => {
                    for field in t.fields.iter_mut() {
                        field.ty = self.ann(&field.ty);
                    }
                    for m in t.methods.iter_mut() {
                        self.rewrite_fn(m);
                    }
                }
                Item::Const(c) => c.ty = self.ann(&c.ty),
                Item::Store(s) => s.ty = self.ann(&s.ty),
                _ => {}
            }
        }
    }

    fn rewrite_fn(&mut self, f: &mut FnItem) {
        for p in f.params.iter_mut() {
            p.ty = self.ann(&p.ty);
        }
        f.ret = self.ann(&f.ret);
        self.rewrite_block(&mut f.body);
    }

    fn rewrite_block(&mut self, b: &mut Block) {
        for s in b.stmts.iter_mut() {
            match s {
                Stmt::Let { ann, .. } => {
                    if let Some(a) = ann {
                        *a = self.ann(a);
                    }
                }
                Stmt::If { then, els, .. } => {
                    self.rewrite_block(then);
                    if let Some(e) = els {
                        self.rewrite_block(e);
                    }
                }
                Stmt::While { body, .. }
                | Stmt::Loop { body, .. }
                | Stmt::ForIn { body, .. }
                | Stmt::Hold { body, .. } => self.rewrite_block(body),
                Stmt::IsMatch { arms, .. } => {
                    for arm in arms.iter_mut() {
                        self.rewrite_block(&mut arm.body);
                    }
                }
                _ => {}
            }
        }
    }
}

#[derive(Default)]
struct GenericExpand {
    aliases: Option<TypeSubst>,
    keep_templates: bool,
    ordinary: bool,
    templates: BTreeMap<String, TypeItem>,
    taken: BTreeSet<String>,
    by_key: BTreeMap<String, String>,
    /// 合成记录名 → 来源模板名（静态方法接收者换名时校验归属，避免把
    /// 别的模板的静态调用换成不相干实例）。
    origin: BTreeMap<String, String>,
    made: Vec<TypeItem>,
    diags: Vec<Diagnostic>,
    depth: usize,
    /// PLAN-963（T-03）：已声明 spec 名（`<T has S>` 的 bound 必须在此集合内）。
    spec_names: BTreeSet<String>,
    /// 记录名 → 遵从列表（含模板与已合成实例；bound 核验的实参侧事实）。
    record_specs: BTreeMap<String, Vec<ConformRef>>,
}

impl GenericExpand {
    fn rewrite_program(&mut self, items: &mut Vec<Item>) {
        for item in items.iter_mut() {
            match item {
                Item::Fn(f) => self.rewrite_fn(f, false),
                Item::Type(t) if t.type_params.is_empty() => {
                    for field in t.fields.iter_mut() {
                        field.ty = self.ann(&field.ty);
                    }
                    for m in t.methods.iter_mut() {
                        self.rewrite_fn(m, true);
                    }
                }
                Item::Const(c) => {
                    c.ty = self.ann(&c.ty);
                    let ty = c.ty.clone();
                    rename_ctor_for_ann(
                        &mut c.value,
                        &ty,
                        &self.templates,
                        &self.made,
                        &self.origin,
                    );
                }
                Item::Store(s) => {
                    s.ty = self.ann(&s.ty);
                    let ty = s.ty.clone();
                    rename_ctor_for_ann(
                        &mut s.value,
                        &ty,
                        &self.templates,
                        &self.made,
                        &self.origin,
                    );
                }
                _ => {}
            }
        }
    }

    fn rewrite_fn(&mut self, f: &mut FnItem, method: bool) {
        let _ = method;
        for p in f.params.iter_mut() {
            p.ty = self.ann(&p.ty);
        }
        f.ret = self.ann(&f.ret);
        let ret = f.ret.clone();
        rename_ctor_in_block_returns(&mut f.body, &ret, &self.templates, &self.made, &self.origin);
        self.rewrite_block(&mut f.body);
    }

    fn rewrite_block(&mut self, b: &mut Block) {
        for s in b.stmts.iter_mut() {
            match s {
                Stmt::Let { ann, value, .. } => {
                    if let Some(a) = ann {
                        let na = self.ann(a);
                        *a = na;
                        let ty = a.clone();
                        rename_ctor_for_ann(value, &ty, &self.templates, &self.made, &self.origin);
                    }
                }
                Stmt::If {
                    cond: _, then, els, ..
                } => {
                    self.rewrite_block(then);
                    if let Some(e) = els {
                        self.rewrite_block(e);
                    }
                }
                Stmt::While { cond: _, body, .. } => self.rewrite_block(body),
                Stmt::Loop { body, .. } => self.rewrite_block(body),
                Stmt::ForIn { iter: _, body, .. } => self.rewrite_block(body),
                Stmt::Hold { body, .. } => self.rewrite_block(body),
                Stmt::IsMatch { arms, .. } => {
                    for arm in arms.iter_mut() {
                        self.rewrite_block(&mut arm.body);
                    }
                }
                _ => {}
            }
        }
    }

    /// PLAN-963（T-02）：把显式类型实参构造 `Point<int> { .. }` 解析为具体
    /// 记录名（并登记/合成实例）。注解驱动的换名只覆盖带注解的位置，本遍
    /// 覆盖任意表达式位置（调用实参、字段值、返回、条件……）。
    fn resolve_typed_ctor(&mut self, e: &mut Expr) {
        let ExprKind::CtorNamed { ty, args, .. } = &mut e.kind else {
            return;
        };
        if args.is_empty() {
            return;
        }
        if let Some(aliases) = self.aliases.as_mut() {
            for arg in args {
                *arg = aliases.ann(arg);
            }
            return;
        }
        if self.keep_templates && args.iter().any(type_contains_variable) {
            return;
        }
        let span = e.span;
        let name = ty.clone();
        let taken = std::mem::take(args);
        // 非泛型/实参个数不符：具名拒（不静默丢实参）。
        match self.templates.get(&name.name) {
            None => {
                self.diags.push(Diagnostic::new(
                    Stage::Text,
                    "source.unsupported",
                    format!("type `{}` takes no type arguments", name.name),
                    span,
                ));
                return;
            }
            Some(tpl) if tpl.type_params.len() != taken.len() => {
                self.diags.push(Diagnostic::new(
                    Stage::Text,
                    "source.unsupported",
                    format!(
                        "generic type `{}` takes {} type argument(s), got {}",
                        name.name,
                        tpl.type_params.len(),
                        taken.len()
                    ),
                    span,
                ));
                return;
            }
            Some(_) => {}
        }
        let resolved = self.ann(&TypeAnn {
            kind: TypeRef::Generic { name, args: taken },
            span,
        });
        let TypeRef::Record(id) = resolved.kind else {
            return;
        };
        if let ExprKind::CtorNamed { ty, .. } = &mut e.kind {
            ty.name = id.name;
            ty.span = id.span;
        }
    }

    fn resolve_typed_ctors_in_expr(&mut self, mut e: &mut Expr) {
        // Prefix chains have no depth bound. A unary node has no
        // constructor of its own, so descend without a Rust stack frame.
        let e = loop {
            match e {
                Expr {
                    kind: ExprKind::Unary { operand, .. },
                    ..
                } => e = operand,
                other => break other,
            }
        };
        self.resolve_typed_ctor(e);
        match &mut e.kind {
            ExprKind::TupleLit(xs) | ExprKind::ArrayLit(xs) => {
                for x in xs.iter_mut() {
                    self.resolve_typed_ctors_in_expr(x);
                }
            }
            ExprKind::GridLit { rows, .. } => {
                for row in rows.iter_mut() {
                    for x in row.iter_mut() {
                        self.resolve_typed_ctors_in_expr(x);
                    }
                }
            }
            ExprKind::Index { base, index } => {
                self.resolve_typed_ctors_in_expr(base);
                self.resolve_typed_ctors_in_expr(index);
            }
            ExprKind::Range { start, end, .. } => {
                self.resolve_typed_ctors_in_expr(start);
                self.resolve_typed_ctors_in_expr(end);
            }
            ExprKind::MethodCall {
                recv,
                args,
                instance: Some(instance),
                ..
            } => {
                for ty in &mut instance.types {
                    *ty = self.ann(ty);
                }
                self.resolve_typed_ctors_in_expr(recv);
                for a in args.iter_mut() {
                    self.resolve_typed_ctors_in_arg(a);
                }
            }
            ExprKind::Method { recv, args, .. } | ExprKind::MethodCall { recv, args, .. } => {
                self.resolve_typed_ctors_in_expr(recv);
                for a in args.iter_mut() {
                    self.resolve_typed_ctors_in_arg(a);
                }
            }
            ExprKind::ConstCall { types, args, .. } => {
                for ty in types {
                    *ty = self.ann(ty);
                }
                for a in args.iter_mut() {
                    self.resolve_typed_ctors_in_arg(a);
                }
            }
            ExprKind::GenericType { args, .. } if self.aliases.is_some() => {
                for ty in args {
                    *ty = self.ann(ty);
                }
            }
            ExprKind::Call { args, .. } => {
                for a in args.iter_mut() {
                    self.resolve_typed_ctors_in_arg(a);
                }
            }
            ExprKind::CallValue { callee, args } => {
                self.resolve_typed_ctors_in_expr(callee);
                for a in args.iter_mut() {
                    self.resolve_typed_ctors_in_arg(a);
                }
            }
            ExprKind::Binary { lhs, rhs, .. } => {
                // Preserve source order while keeping binary spines off the host stack.
                let mut pending = vec![rhs.as_mut(), lhs.as_mut()];
                while let Some(child) = pending.pop() {
                    match child {
                        Expr {
                            kind: ExprKind::Binary { lhs, rhs, .. },
                            ..
                        } => {
                            pending.push(rhs);
                            pending.push(lhs);
                        }
                        Expr {
                            kind: ExprKind::Unary { operand, .. },
                            ..
                        } => pending.push(operand),
                        other => self.resolve_typed_ctors_in_expr(other),
                    }
                }
            }
            ExprKind::CtorNamed { fields, .. } => {
                for f in fields.iter_mut() {
                    self.resolve_typed_ctors_in_expr(&mut f.value);
                }
            }
            ExprKind::Field { base, .. } => self.resolve_typed_ctors_in_expr(base),
            ExprKind::HashExpr { expr } => self.resolve_typed_ctors_in_expr(expr),
            ExprKind::Closure { params, result, body, .. } => {
                if let Some(aliases) = self.aliases.as_mut() {
                    for param in params {
                        param.ty = aliases.ann(&param.ty);
                    }
                    if let Some(result) = result {
                        *result = aliases.ann(result);
                    }
                }
                match body.as_mut() {
                    ClosureBody::Expr(x) => self.resolve_typed_ctors_in_expr(x),
                    ClosureBody::Block(b) => self.resolve_typed_ctors_in_block(b),
                }
            }
            _ => {}
        }
    }

    fn resolve_typed_ctors_in_arg(&mut self, a: &mut Arg) {
        match a {
            Arg::Pos(x) | Arg::MovePos(x) => self.resolve_typed_ctors_in_expr(x),
            Arg::Named { value, .. } => self.resolve_typed_ctors_in_expr(value),
        }
    }

    fn resolve_typed_ctors_in_block(&mut self, b: &mut Block) {
        for s in b.stmts.iter_mut() {
            match s {
                Stmt::ExprStmt { value, .. }
                | Stmt::Let { value, .. }
                | Stmt::Assign { value, .. }
                | Stmt::LetDestruct { value, .. }
                | Stmt::Return { value, .. } => self.resolve_typed_ctors_in_expr(value),
                Stmt::IndexAssign(ia) => {
                    self.resolve_typed_ctors_in_expr(&mut ia.index);
                    self.resolve_typed_ctors_in_expr(&mut ia.value);
                }
                Stmt::PlaceAssign(pa) => {
                    for st in &mut pa.steps {
                        if let PlaceStep::Index(ix) = st {
                            self.resolve_typed_ctors_in_expr(ix);
                        }
                    }
                    self.resolve_typed_ctors_in_expr(&mut pa.value);
                }
                Stmt::NestedFn { item } => self.resolve_typed_ctors_in_block(&mut item.body),
                Stmt::Hold { body, .. } => self.resolve_typed_ctors_in_block(body),
                Stmt::With { expr, body, .. } => {
                    self.resolve_typed_ctors_in_expr(expr);
                    self.resolve_typed_ctors_in_block(body);
                }
                Stmt::If {
                    cond, then, els, ..
                } => {
                    self.resolve_typed_ctors_in_expr(cond);
                    self.resolve_typed_ctors_in_block(then);
                    if let Some(e) = els {
                        self.resolve_typed_ctors_in_block(e);
                    }
                }
                Stmt::While { cond, body, .. } => {
                    self.resolve_typed_ctors_in_expr(cond);
                    self.resolve_typed_ctors_in_block(body);
                }
                Stmt::Loop { body, .. } => self.resolve_typed_ctors_in_block(body),
                Stmt::IsMatch { arms, .. } => {
                    for arm in arms.iter_mut() {
                        self.resolve_typed_ctors_in_block(&mut arm.body);
                    }
                }
                Stmt::ForIn { iter, body, .. } => {
                    self.resolve_typed_ctors_in_expr(iter);
                    self.resolve_typed_ctors_in_block(body);
                }
                Stmt::Break { .. } | Stmt::Continue { .. } => {}
                Stmt::Yield { value, .. } => self.resolve_typed_ctors_in_expr(value),
                Stmt::AggIsMatch {
                    scrutinee, arms, ..
                } => {
                    self.resolve_typed_ctors_in_expr(scrutinee);
                    for arm in arms.iter_mut() {
                        match &mut arm.body {
                            AggArmBody::Block(b) => self.resolve_typed_ctors_in_block(b),
                            AggArmBody::Expr(e) => self.resolve_typed_ctors_in_expr(e),
                        }
                    }
                }
                Stmt::Try {
                    body,
                    catch,
                    finally,
                    ..
                } => {
                    self.resolve_typed_ctors_in_block(body);
                    if let Some(c) = catch {
                        self.resolve_typed_ctors_in_block(&mut c.body);
                    }
                    if let Some(f) = finally {
                        self.resolve_typed_ctors_in_block(f);
                    }
                }
                // PLAN-966（T-02）：语句级 CT——两臂/体内表达式同样做具名构造
                // 重写（选中臂之后才参与绑定，重写对未选臂无害）。
                Stmt::HashIf { then, els, .. } => {
                    self.resolve_typed_ctors_in_block(then);
                    if let Some(e) = els {
                        self.resolve_typed_ctors_in_block(e);
                    }
                }
                Stmt::HashFor { body, .. } => self.resolve_typed_ctors_in_block(body),
                Stmt::HashIs { arms, .. } => {
                    for a in arms.iter_mut() {
                        self.resolve_typed_ctors_in_block(&mut a.body);
                    }
                }
            }
        }
    }

    fn resolve_typed_ctors_in_item(&mut self, it: &mut Item) {
        match it {
            Item::Fn(f) => self.resolve_typed_ctors_in_block(&mut f.body),
            Item::Const(c) => self.resolve_typed_ctors_in_expr(&mut c.value),
            Item::Store(s) => self.resolve_typed_ctors_in_expr(&mut s.value),
            Item::Type(t) => {
                for m in t.methods.iter_mut() {
                    self.resolve_typed_ctors_in_block(&mut m.body);
                }
            }
            Item::Enum(e) if self.aliases.is_some() => {
                for method in &mut e.methods {
                    self.resolve_typed_ctors_in_block(&mut method.body);
                }
            }
            Item::Enum(_) => {}
            // PLAN-967 F0: the import frontdoor carries no expressions.
            Item::UseRs(_) => {}
            // PLAN-1026: the py frontdoor carries no expressions either
            // (member imports are hoisted py_import calls in the script).
            Item::UsePy(_) => {}
            // PLAN-968: task declarations carry expressions in field
            // defaults, hook bodies and handler arms.
            Item::Task(t) => {
                for f in &t.fields {
                    self.resolve_typed_ctors_in_expr(&mut f.default.clone());
                }
                for h in t.start.iter().chain(t.stop.iter()) {
                    self.resolve_typed_ctors_in_block(&mut h.body.clone());
                }
                for a in &mut t.arms {
                    self.resolve_typed_ctors_in_block(&mut a.body.clone());                }
            }
            // PLAN-1095: the node declaration body was consumed at parse
            // time; it carries no ordinary expressions to rewrite.
            Item::Node(_) => {}
        }
    }

    /// 重写注解：用户泛型引用 → 具体记录名；内建泛型只递归其参数位。
    fn ann(&mut self, a: &TypeAnn) -> TypeAnn {
        if let Some(aliases) = self.aliases.as_mut() {
            return aliases.ann(a);
        }
        match &a.kind {
            TypeRef::Generic { name, args } => {
                if self.templates.contains_key(&name.name)
                    && !(self.keep_templates && args.iter().any(type_contains_variable))
                {
                    self.instance(name, args, a.span)
                } else {
                    TypeAnn {
                        kind: TypeRef::Generic {
                            name: name.clone(),
                            args: args.iter().map(|x| self.ann(x)).collect(),
                        },
                        span: a.span,
                    }
                }
            }
            TypeRef::Optional(inner) => TypeAnn {
                kind: TypeRef::Optional(Box::new(self.ann(inner))),
                span: a.span,
            },
            TypeRef::Stream(inner) => TypeAnn {
                kind: TypeRef::Stream(Box::new(self.ann(inner))),
                span: a.span,
            },
            TypeRef::Tuple(elems) => TypeAnn {
                kind: TypeRef::Tuple(elems.iter().map(|x| self.ann(x)).collect()),
                span: a.span,
            },
            TypeRef::Function { params, result } => TypeAnn {
                kind: TypeRef::Function {
                    params: params.iter().map(|x| self.ann(x)).collect(),
                    result: Box::new(self.ann(result)),
                },
                span: a.span,
            },
            _ => a.clone(),
        }
    }

    fn instance(&mut self, name: &Ident, args: &[TypeAnn], span: Span) -> TypeAnn {
        let Some(tpl) = self.templates.get(&name.name).cloned() else {
            return TypeAnn {
                kind: TypeRef::Record(name.clone()),
                span,
            };
        };
        if tpl.type_params.len() != args.len() {
            self.diags.push(Diagnostic::new(
                Stage::Text,
                "source.unsupported",
                format!(
                    "generic type `{}` takes {} type argument(s), got {}",
                    name.name,
                    tpl.type_params.len(),
                    args.len()
                ),
                span,
            ));
            return TypeAnn {
                kind: TypeRef::Record(name.clone()),
                span,
            };
        }
        let mut concrete = Vec::with_capacity(args.len());
        let mut spellings = Vec::with_capacity(args.len());
        for a in args {
            let c = self.ann(a);
            match type_spelling(&c) {
                Some(s) => {
                    spellings.push(s);
                    concrete.push(c);
                }
                None => {
                    self.diags.push(Diagnostic::new(
                        Stage::Text,
                        "source.unsupported",
                        "generic type arguments must be `int`/`i32`, `bool`, `str`/`Str` or a declared record type",
                        a.span,
                    ));
                    return TypeAnn {
                        kind: TypeRef::Record(name.clone()),
                        span,
                    };
                }
            }
        }
        let key = format!("{}<{}>", name.name, spellings.join(","));
        // PLAN-963（T-03）：`<T has S>` 约束——语法期按实参的遵从列表核验。
        // 不满足即具名拒，不合成“无约束”的实例（那会让 wrapper 静默丢失
        // 约束，正是 V-G03 的反控项）。
        for (param, concrete_ann) in tpl.type_params.iter().zip(concrete.iter()) {
            let Some(bound) = &param.bound else { continue };
            if !self.spec_names.contains(&bound.name) {
                let sp = bound.span;
                let (b, t) = (bound.name.clone(), name.name.clone());
                self.diags.push(Diagnostic::new(
                    Stage::Text,
                    "source.unsupported",
                    format!("unknown spec `{b}` in the bound of `{t}`'s type parameter"),
                    sp,
                ));
                continue;
            }
            let satisfies = match &concrete_ann.kind {
                TypeRef::Record(id) => self
                    .record_specs
                    .get(&id.name)
                    .is_some_and(|cs| cs.iter().any(|s| s.name.name == bound.name)),
                _ => false,
            };
            if !satisfies {
                let sp = concrete_ann.span;
                let (b, t, p) = (
                    bound.name.clone(),
                    name.name.clone(),
                    param.name.name.clone(),
                );
                let arg = type_spelling(concrete_ann).unwrap_or_else(|| "that type".into());
                self.diags.push(Diagnostic::new(
                    Stage::Text,
                    "source.unsupported",
                    format!(
                        "type argument `{arg}` does not satisfy `{b}` (bound of `{p}` in `{t}`)"
                    ),
                    sp,
                ));
            }
        }
        let mangled = match self.by_key.get(&key) {
            Some(m) => m.clone(),
            None => {
                if self.depth > 8 {
                    self.diags.push(Diagnostic::new(
                        Stage::Text,
                        "source.unsupported",
                        format!(
                            "recursive generic type expansion for `{}` exceeds its bound",
                            name.name
                        ),
                        span,
                    ));
                    return TypeAnn {
                        kind: TypeRef::Record(name.clone()),
                        span,
                    };
                }
                let base = format!("{}_{}", name.name, spellings.join("_"));
                let mut candidate = base.clone();
                let mut n = 2usize;
                while self.taken.contains(&candidate) {
                    candidate = format!("{base}__{n}");
                    n += 1;
                }
                self.taken.insert(candidate.clone());
                self.by_key.insert(key, candidate.clone());
                self.origin.insert(candidate.clone(), name.name.clone());
                // PLAN-963（T-03）：合成实例的遵从列表随模板复制，登记后供
                // 外层 `Box<Pair<int>>` 之类嵌套实参的 bound 核验使用。
                self.record_specs
                    .insert(candidate.clone(), tpl.conforms.clone());
                let subst: BTreeMap<String, TypeAnn> = tpl
                    .type_params
                    .iter()
                    .zip(concrete.iter())
                    .map(|(p, t)| (p.name.name.clone(), t.clone()))
                    .collect();
                self.depth += 1;
                let mut fields = Vec::with_capacity(tpl.fields.len());
                for f in &tpl.fields {
                    let sub = subst_ty(&f.ty, &subst);
                    fields.push(FieldDecl {
                        name: f.name.clone(),
                        ty: self.ann(&sub),
                        span: f.span,
                    });
                }
                let mut methods = Vec::with_capacity(tpl.methods.len());
                for m in &tpl.methods {
                    let mut g = m.clone();
                    if self.ordinary {
                        if let Some(owner) = g.owner.as_mut() {
                            owner.name = candidate.clone();
                        }
                    }
                    // 先按声明序替换（参数/返回/局部注解），再走与普通函数
                    // 相同的注解解析+构造换名：方法体内的 `Point { .. }`
                    // （返回类型已具体化）同样必须换成合成实例名。
                    for p in g.params.iter_mut() {
                        p.ty = subst_ty(&p.ty, &subst);
                    }
                    g.ret = subst_ty(&g.ret, &subst);
                    subst_block_lets(&mut g.body, &subst);
                    self.rewrite_fn(&mut g, true);
                    methods.push(g);
                }
                self.depth -= 1;
                self.made.push(TypeItem {
                    name: Ident {
                        name: candidate.clone(),
                        span,
                    },
                    type_params: Vec::new(),
                    instance: self.ordinary.then(|| RecordInstance {
                        template: name.name.clone(),
                        arguments: concrete,
                    }),
                    conforms: tpl.conforms.clone(),
                    spec_memberships: tpl.spec_memberships.clone(),
                    fields,
                    methods,
                    span: tpl.span,
                });
                candidate
            }
        };
        TypeAnn {
            kind: TypeRef::Record(Ident {
                name: mangled,
                span,
            }),
            span,
        }
    }
}

fn type_contains_variable(annotation: &TypeAnn) -> bool {
    match &annotation.kind {
        TypeRef::TyVar(_) => true,
        TypeRef::Generic { args, .. } | TypeRef::Tuple(args) => {
            args.iter().any(type_contains_variable)
        }
        TypeRef::Function { params, result } => {
            params.iter().any(type_contains_variable) || type_contains_variable(result)
        }
        TypeRef::Optional(inner) | TypeRef::ResultBang(inner) | TypeRef::Stream(inner) => {
            type_contains_variable(inner)
        }
        _ => false,
    }
}

fn type_spelling(a: &TypeAnn) -> Option<String> {
    match &a.kind {
        TypeRef::Builtin(TypeWord::Int) | TypeRef::Builtin(TypeWord::I32) => Some("int".into()),
        TypeRef::Builtin(TypeWord::Bool) => Some("bool".into()),
        // PLAN-963（T-08，V-08）：资源（文本）实参也参与模板实例身份；
        // `str` 是借用视图、`Str` 是 owned，两者拼写不同即不同实例。
        TypeRef::Builtin(TypeWord::Str) => Some("str".into()),
        TypeRef::Builtin(TypeWord::StrOwned) => Some("Str".into()),
        TypeRef::Record(id) => Some(id.name.clone()),
        _ => None,
    }
}

/// 声明序替换：类型参数名 → 具体类型（递归全部类型形状）。
fn subst_ty(a: &TypeAnn, subst: &BTreeMap<String, TypeAnn>) -> TypeAnn {
    match &a.kind {
        TypeRef::TyVar(id) => match subst.get(&id.name) {
            Some(t) => t.clone(),
            None => a.clone(),
        },
        TypeRef::Optional(inner) => TypeAnn {
            kind: TypeRef::Optional(Box::new(subst_ty(inner, subst))),
            span: a.span,
        },
        TypeRef::Generic { name, args } => TypeAnn {
            kind: TypeRef::Generic {
                name: name.clone(),
                args: args.iter().map(|x| subst_ty(x, subst)).collect(),
            },
            span: a.span,
        },
        TypeRef::Stream(inner) => TypeAnn {
            kind: TypeRef::Stream(Box::new(subst_ty(inner, subst))),
            span: a.span,
        },
        TypeRef::Tuple(elems) => TypeAnn {
            kind: TypeRef::Tuple(elems.iter().map(|x| subst_ty(x, subst)).collect()),
            span: a.span,
        },
        TypeRef::Function { params, result } => TypeAnn {
            kind: TypeRef::Function {
                params: params.iter().map(|x| subst_ty(x, subst)).collect(),
                result: Box::new(subst_ty(result, subst)),
            },
            span: a.span,
        },
        _ => a.clone(),
    }
}

fn subst_block_lets(b: &mut Block, subst: &BTreeMap<String, TypeAnn>) {
    for s in b.stmts.iter_mut() {
        match s {
            Stmt::Let { ann, .. } => {
                if let Some(a) = ann {
                    *a = subst_ty(a, subst);
                }
            }
            Stmt::If { then, els, .. } => {
                subst_block_lets(then, subst);
                if let Some(e) = els {
                    subst_block_lets(e, subst);
                }
            }
            Stmt::While { body, .. } => subst_block_lets(body, subst),
            Stmt::Loop { body, .. } => subst_block_lets(body, subst),
            Stmt::ForIn { body, .. } => subst_block_lets(body, subst),
            Stmt::Hold { body, .. } => subst_block_lets(body, subst),
            Stmt::IsMatch { arms, .. } => {
                for arm in arms.iter_mut() {
                    subst_block_lets(&mut arm.body, subst);
                }
            }
            _ => {}
        }
    }
}

/// 注解已具体化为 `Record` 时，把该位置的直接构造 `Point { .. }` 重命名为
/// 具体实例名（模板名在单态化后不存在）；`Point.mk(..)` 这类模板**静态方法**
/// 接收者同样按注解换成同一实例（仅当该实例确由该模板合成）。字段值再按
/// 具体记录声明的字段类型递归处理（`Pair { a: 1, b: Box { .. } }` 的内层
/// 构造同样要换名）。
fn rename_ctor_for_ann(
    value: &mut Expr,
    ann: &TypeAnn,
    templates: &BTreeMap<String, TypeItem>,
    made: &[TypeItem],
    origin: &BTreeMap<String, String>,
) {
    let TypeRef::Record(id) = &ann.kind else {
        return;
    };
    match &mut value.kind {
        ExprKind::CtorNamed { ty, args, fields } => {
            // An explicit constructor can infer a different instance than the
            // annotation. Leave it intact for the explicit-instantiation pass.
            if args.is_empty()
                && origin.get(&id.name).map(String::as_str) == Some(ty.name.as_str())
            {
                ty.name = id.name.clone();
                ty.span = id.span;
            }
            if ty.name != id.name {
                return;
            }
            let Some(item) = made.iter().find(|t| t.name.name == id.name) else {
                return;
            };
            for init in fields.iter_mut() {
                if let Some(decl) = item.fields.iter().find(|f| f.name.name == init.name.name) {
                    let decl_ty = decl.ty.clone();
                    rename_ctor_for_ann(&mut init.value, &decl_ty, templates, made, origin);
                }
            }
        }
        ExprKind::Method { recv, .. } | ExprKind::MethodCall { recv, .. } => {
            let ExprKind::Var(v) = &mut recv.kind else {
                return;
            };
            if !templates.contains_key(&v.name) {
                return;
            }
            if origin.get(&id.name).map(String::as_str) != Some(v.name.as_str()) {
                return;
            }
            v.name = id.name.clone();
            v.span = id.span;
        }
        _ => {}
    }
}

fn rename_ctor_in_block_returns(
    b: &mut Block,
    ret: &TypeAnn,
    templates: &BTreeMap<String, TypeItem>,
    made: &[TypeItem],
    origin: &BTreeMap<String, String>,
) {
    for s in b.stmts.iter_mut() {
        match s {
            Stmt::Return { value, .. } => rename_ctor_for_ann(value, ret, templates, made, origin),
            Stmt::If { then, els, .. } => {
                rename_ctor_in_block_returns(then, ret, templates, made, origin);
                if let Some(e) = els {
                    rename_ctor_in_block_returns(e, ret, templates, made, origin);
                }
            }
            Stmt::While { body, .. } => {
                rename_ctor_in_block_returns(body, ret, templates, made, origin)
            }
            Stmt::Loop { body, .. } => {
                rename_ctor_in_block_returns(body, ret, templates, made, origin)
            }
            Stmt::ForIn { body, .. } => {
                rename_ctor_in_block_returns(body, ret, templates, made, origin)
            }
            Stmt::Hold { body, .. } => {
                rename_ctor_in_block_returns(body, ret, templates, made, origin)
            }
            Stmt::IsMatch { arms, .. } => {
                for arm in arms.iter_mut() {
                    rename_ctor_in_block_returns(&mut arm.body, ret, templates, made, origin);
                }
            }
            _ => {}
        }
    }
}

/// PLAN-1043（T-03）：packages-ct 面 `#in` 块内声明的子树投影入口。
///
/// 与顶层投影同一引擎/预算/诊断纪律（不建第二投影器）；只服务 in-stmt 的
/// fn/type/const/var 项与 `pub spec fn` 需求（pub 壳由 packages 投影剥开
/// 后传入内层节点）。产物由 packages 投影按选中结果落位，未选分支的
/// 声明不进任何 items。
pub(crate) struct InDeclProjector<'a> {
    eng: Engine<'a>,
}

pub(crate) fn in_decl_projector<'a>(
    p: &'a GrammarProgram,
    k: &'a SourceKinds,
    tree: &'a CstTree,
    snap: &'a Snapshot,
    profile: SourceProfile,
) -> InDeclProjector<'a> {
    InDeclProjector {
        eng: Engine::new(
            p,
            k,
            tree,
            snap,
            profile,
            false,
            profile.is_auto_language(),
            false,
        ),
    }
}

impl<'a> InDeclProjector<'a> {
    pub fn fn_item(&mut self, node: u32) -> FnItem {
        self.eng.run_fn_item(node, None, Receiver::Free)
    }
    pub fn type_item(&mut self, node: u32) -> TypeItem {
        self.eng.run_type_item(node)
    }
    pub fn global_item(&mut self, node: u32, mutable: bool) -> Item {
        self.eng.run_global_item(node, mutable)
    }
    /// `pub spec fn` 需求签名（name/params/ret）。
    pub fn require(&mut self, node: u32) -> (Ident, Vec<Param>, TypeAnn) {
        self.eng.run_require_item(node)
    }
    /// type 别名形（`type X = T`）在 #in 内不支持（本轮面）；调用方具名拒。
    pub fn is_type_alias(&mut self, node: u32) -> bool {
        self.eng.is_type_alias(node)
    }
    /// 首个投影诊断（有则调用方整体失败——与顶层首事实序一致）。
    pub fn first_diag(&self) -> Option<Diagnostic> {
        self.eng.cx.first_diag()
    }
}

fn async_future(mut body: Block, result: Option<TypeAnn>, span: Span) -> Expr {
    let void = result
        .as_ref()
        .is_some_and(|ty| ty.kind == TypeRef::Builtin(TypeWord::Void));
    if !void {
        if let Some(Stmt::ExprStmt { value, span }) = body.stmts.last_mut() {
            let terminal = matches!(&value.kind, ExprKind::Call { callee, .. }
                if matches!(callee.name.as_str(), "fail" | "rethrow"));
            if !terminal {
                let span = *span;
                let value = std::mem::replace(
                    value,
                    Expr {
                        span,
                        kind: ExprKind::IntLit(0),
                    },
                );
                *body.stmts.last_mut().expect("checked tail") = Stmt::Return { value, span };
            }
        }
    }
    let thunk = Expr {
        span,
        kind: ExprKind::Closure {
            move_capture: true,
            params: Vec::new(),
            result,
            body: Box::new(ClosureBody::Block(body)),
        },
    };
    Expr {
        span,
        kind: ExprKind::Call {
            callee: Ident {
                name: "__sched_future".into(),
                span,
            },
            args: vec![Arg::Pos(thunk)],
        },
    }
}
