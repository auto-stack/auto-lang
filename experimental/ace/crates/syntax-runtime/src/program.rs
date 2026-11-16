//! 语法程序 IR：编译器（ac-syntax-grammar）的输出，运行时的唯一驱动源。
//!
//! 规则程序 = 明确优先级的 ordered-choice（PLAN-787 §3）：
//! rule := alt | alt | ...（按声明序尝试）；alt := step*；
//! step := atom [field] [repeat]。原子 = terminal / rule 引用 / Pratt 表达式入口。
//! 表达式走配置驱动的 Pratt 引擎（binding power/结合性来自语法包），
//! 结构（声明/语句/块）走规则程序——单一运行时，禁止按语言 ID 分派。
//!
//! 禁止（生成期拒绝，见 ac-syntax-grammar::check）：左递归、nullable repeat、
//! 未知引用、无法确定前进的 scanner 规则。

use crate::identity::{GrammarId, PackageIdentity};

/// 语法程序 schema 版本（.at 配置的可读面）。
pub const GRAMMAR_SCHEMA_VERSION: u32 = 2;
/// 程序规范二进制序列化格式版本（语言包装载格式，实验 Schema，非正式 Batom）。
/// 5：PLAN-979 增加规则诊断标签表（构造级 "unexpected token in …" 诊断）；
/// 4 已增加上下文名称表与 `Atom::CtxName`（槽位限定拼写识别）。
pub const PROGRAM_CODEC_VERSION: u16 = 5;

pub type RuleId = u32;
pub type TokenId = u32;
pub type PrattId = u32;
/// 上下文名称规则 id（PLAN-979 T-H0-I）。
pub type CtxNameId = u32;

/// ASCII 位集 + 非 ASCII 许可（core 拒绝非 ASCII 名；mini 亦仅 ASCII）。
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CharSet {
    pub bits: [u64; 2],
    pub non_ascii: bool,
}

impl CharSet {
    pub fn empty() -> Self {
        CharSet {
            bits: [0; 2],
            non_ascii: false,
        }
    }
    pub fn from_ranges(spec: &[(u8, u8)]) -> Self {
        let mut cs = CharSet::empty();
        for &(a, b) in spec {
            for c in a..=b {
                cs.add(c);
            }
        }
        cs
    }
    pub fn add(&mut self, c: u8) {
        self.bits[(c >> 6) as usize] |= 1u64 << (c & 63);
    }
    /// ASCII 位图命中；字节 ≥128 归入 non_ascii 语义（ASCII-only 语言面）。
    pub fn contains(&self, c: u8) -> bool {
        if c >= 128 {
            return self.non_ascii;
        }
        self.bits[(c >> 6) as usize] & (1u64 << (c & 63)) != 0
    }
    pub fn is_empty(&self) -> bool {
        self.bits == [0, 0] && !self.non_ascii
    }
}

/// scanner 匹配器。配置terminal支持 literal、受限字符集合/重复与声明的
/// mode 规则；不执行任意宿主代码、不支持任意 regex。
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Matcher {
    /// 固定字面量（关键字/运算符）。
    Literal(String),
    /// 字符集合串（标识符/数字）：首字符集 + 后续字符集，最少匹配数。
    Charset {
        first: CharSet,
        cont: CharSet,
        min_len: u32,
    },
    /// 行注释：start 字面量到（不含）换行。
    LineComment(String),
    /// 块注释：start..end，可嵌套（interface-mini 的 /* */ 嵌套）。
    BlockComment {
        start: String,
        end: String,
        nested: bool,
    },
    /// 字符串字面量：引号字节、是否支持 `\` 转义（跨块/跨行状态恢复）。
    StringLit { quote: u8, escape: bool },
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ScanRule {
    pub mode: u32,
    pub matcher: Matcher,
    /// 命中产生的 token（trivia 也占 token id，标志 trivia）。
    pub token: TokenId,
    pub trivia: bool,
    /// 命中后切换到的 mode（自环 = 自身）。
    pub next_mode: u32,
}

/// 多字符 token 的字面量集合（同一首字节下取最长匹配、同长取声明序）。
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TokenSpec {
    pub name: String,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ScannerProgram {
    /// mode 0 = 初始 mode。
    pub mode_names: Vec<String>,
    pub rules: Vec<ScanRule>,
}

/// 结构规则。
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RuleSpec {
    pub name: String,
    pub alts: Vec<Alt>,
    /// 失败恢复：所有 alt 失败且已消费 ≥1 token 时，吃进 error 叶到同步点
    /// （sync ∪ first(rule)），产出 missing + error 子节点后"成功"返回
    /// （PLAN-787 §5.3 声明级恢复）。
    pub recover: bool,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Alt {
    pub steps: Vec<Step>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Repeat {
    Once,
    Optional,
    ZeroOrMore,
    /// 列表：项间跳过 `stmt_end` 分隔 token（`;`）。
    ListPlain,
    /// 换行敏感列表（newline.significant 时）：项间除分隔 token 外还接受
    /// 换行（下一 token 起始行 > 上一消费 token 行）作为语句分隔（core 语义）。
    ListNewline,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Step {
    pub atom: Atom,
    pub field: Option<u32>,
    pub repeat: Repeat,
    /// 换行敏感（core）：序列内该 token 必须与前一消费 token 同行
    /// （仅未括号表达式上下文强制；用于 `-`/字面量同行的有符号字面量规则）。
    pub same_line: bool,
    /// PLAN-927 cut 步：该 token 消费后，本 alt 失败**不再回退到后续备选**
    /// （配合 rule.recover 走恢复，或无 recover 时硬失败上抛）。复刻被替代
    /// 手写 parser 的"提交点"语义（如 record ctor `ident {` 之后不回溯）。
    pub cut: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Atom {
    Tok(TokenId),
    Rule(RuleId),
    Pratt(PrattId),
    /// PLAN-979 T-H0-I：上下文名称槽（槽位限定拼写识别，不进入保留字表）。
    CtxName(CtxNameId),
}

/// 上下文名称规则：**槽位**决定一个拼写是结构名还是普通标识符。
///
/// 词法面不变（拼写仍按通用 token 词法切分，如 `ident`）；只有语法程序里
/// 声明 `CtxName` 的槽位才要求逐字节等于 `spelling`。因此 `view` 在
/// `view { ... }` 节点头位置是结构名，在 `let view = 1` 里仍是普通绑定。
/// 角色（`role`）承载领域语义：换拼写（别名/隔离测试 Schema）不改角色，
/// 投影按角色归属，绝不按拼写猜领域。
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CtxNameSpec {
    /// 配置中的名字（规则步 `name(of: ...)` 的引用键）。
    pub name: String,
    /// 槽位要求的源码拼写（逐字节相等）。
    pub spelling: String,
    /// 该拼写经 scanner 产生的 token（first 集与诊断用）。
    pub token: TokenId,
    /// 语义角色（投影/领域归属按角色，不按拼写）。
    pub role: String,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Assoc {
    Left,
    Right,
}

/// Pratt 运算符表项：operator token、binding power、结合性。
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PrattOp {
    pub op: TokenId,
    /// 0=禁用（保留），值越大绑定越紧。core：`<`=1 `+`=2 `*`=3。
    pub binding: u8,
    pub assoc: Assoc,
}

/// Pratt 表达式入口：主规则 + 运算符表（表可为空 = 纯 primary）。
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PrattSpec {
    pub primary: RuleId,
    pub ops: Vec<PrattOp>,
    /// `(` 分组开始 token（None = 无分组）。
    pub group_open: Option<TokenId>,
    pub group_close: Option<TokenId>,
    /// 调用后缀：callee token 行规则（core：`(` 必须与 callee 同行）。
    pub call_paren: Option<CallParenRule>,
    /// 调用实参列表规则（call `(` args `)`）。
    pub call_args: Option<RuleId>,
    /// 点后缀续读规则（如 `postfix-tail { many { postfix-step } }`）：调用
    /// 收尾后操作数还可继续消费 `.name` 步（`f().len()`）；None = 无续读。
    pub postfix_tail: Option<RuleId>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CallParenRule {
    pub open: TokenId,
    pub close: TokenId,
    /// 括号内换行是否自由（core：括号内换行纯空白）。
    pub newline_free_inside: bool,
    /// 未括号上下文中 callee 与 `(` 是否必须同行（core=true）。
    pub callee_same_line: bool,
}

/// 换行敏感策略（generic 旋钮，来自语法包；mini 声明 significant=false）。
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct NewlinePolicy {
    pub significant: bool,
    /// 语句边界 token（`;` `}`）：边界检查用（`;`/`}`/换行 皆可为界）。
    /// **不消费** `}`——它是块的结束 token，由块规则消费。
    pub stmt_end: Vec<TokenId>,
    /// 项间实际**消费**的分隔 token（仅 `;`）。
    pub sep_consume: Vec<TokenId>,
}

/// 恢复规则：同步点 token 集合（在 grammar 中声明）。
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RecoverySpec {
    pub sync: Vec<TokenId>,
}

/// outline 抽取规则（声明规则名 + 名字/参数字段名）。
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct OutlineSpec {
    pub decl_rule: String,
    pub name_field: String,
    pub params_field: Option<String>,
}

/// 编译产物：完整语法程序。所有 kind 命名空间：
/// token kind = `TOKEN_KIND_BASE + token_id`；rule kind = `RULE_KIND_BASE + rule_id`；
/// 元节点（ERROR/MISSING）在高位。
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct GrammarProgram {
    pub name: String,
    pub revision: u64,
    pub schema_version: u32,
    pub scanner: ScannerProgram,
    pub tokens: Vec<TokenSpec>,
    pub rules: Vec<RuleSpec>,
    pub pratt: Vec<PrattSpec>,
    /// 字段名表（Step.field 的 id → 名；outline 按 name 解析 id）。
    pub field_names: Vec<String>,
    /// 上下文名称规则表（PLAN-979；无该表 = 无槽位限定拼写）。
    pub ctx_names: Vec<CtxNameSpec>,
    /// 规则诊断标签（PLAN-979；与 `rules` 平行，`Some` = 该规则失败时报告
    /// "unexpected token in <label>" 并指向当前 token 的原 span）。这是语法
    /// 数据，不是运行时领域知识。
    pub rule_expects: Vec<Option<String>>,
    pub start: RuleId,
    pub newline: NewlinePolicy,
    pub recovery: RecoverySpec,
    pub outline: Option<OutlineSpec>,
    pub identity: PackageIdentity,
}

pub const TOKEN_KIND_BASE: u32 = 1 << 20;
pub const RULE_KIND_BASE: u32 = 1 << 21;
pub const KIND_ERROR: u32 = u32::MAX - 1;
pub const KIND_MISSING: u32 = u32::MAX - 2;
/// 调用后缀节点（Pratt 引擎构造；孩子 = callee/`(`/args/`)`）。
pub const KIND_CALL: u32 = u32::MAX - 3;
/// 调用后点后缀包装节点（Pratt 引擎构造；孩子 = 操作数 + postfix-tail 规则节点，
/// 如 `f().len()` 调用收尾后再接 `.name` 步——旧 parser 的后缀链语义）。
pub const KIND_POSTFIX: u32 = u32::MAX - 4;

impl GrammarProgram {
    pub fn token_kind(&self, t: TokenId) -> u32 {
        TOKEN_KIND_BASE + t
    }
    pub fn rule_kind(&self, r: RuleId) -> u32 {
        RULE_KIND_BASE + r
    }
    pub fn token_name(&self, t: TokenId) -> &str {
        self.tokens[t as usize].name.as_str()
    }
    pub fn rule_name(&self, r: RuleId) -> &str {
        self.rules[r as usize].name.as_str()
    }
    pub fn grammar_id(&self) -> &GrammarId {
        &self.identity.grammar_id
    }
    /// 字段名 → id（首次出现顺序分配由编译器保证；重复名取首个）。
    pub fn field_id(&self, name: &str) -> Option<u32> {
        self.field_names
            .iter()
            .position(|f| f == name)
            .map(|i| i as u32)
    }
    /// 上下文名称规则（PLAN-979）。
    pub fn ctx_name(&self, id: CtxNameId) -> Option<&CtxNameSpec> {
        self.ctx_names.get(id as usize)
    }
    /// 规则诊断标签（PLAN-979）：规则失败时的构造描述。
    pub fn rule_expect(&self, id: RuleId) -> Option<&str> {
        self.rule_expects
            .get(id as usize)
            .and_then(|e| e.as_deref())
    }
    /// 该角色声明的全部上下文名称（别名/拼写变体共用同一角色）。
    pub fn ctx_names_with_role<'s>(
        &'s self,
        role: &'s str,
    ) -> impl Iterator<Item = &'s CtxNameSpec> {
        self.ctx_names.iter().filter(move |n| n.role == role)
    }
    /// 规范二进制（身份计算与持久化共用同一布局，PROGRAM_CODEC_VERSION 前缀）。
    pub fn canonical_bytes(&self) -> Vec<u8> {
        crate::program_codec::encode_canonical(self)
    }

    /// 引用命名空间准入（PLAN-1029 ADM-02）：在授信/FirstSets/parse 消费前
    /// 按字段语义核对每个数值引用的归属表——TokenId 与 RuleId 是**不同
    /// 命名空间**（`Pratt.call_args/postfix_tail` 是 RuleId，不得按 token 表
    /// 校验），optional None 合法。任何越界 = `InvalidProgram`，绝不带着坏
    /// 引用进入 `first[prim]`/Machine 的直接索引（坏 primary 此前会 panic）。
    /// 合法递归 grammar 照常通过（不新增无环/深度限制）。
    pub fn validate_references(&self) -> Result<(), RtError> {
        let n_tok = self.tokens.len();
        let n_rule = self.rules.len();
        let n_mode = self.scanner.mode_names.len();
        for r in &self.scanner.rules {
            if (r.token as usize) >= n_tok {
                return Err(RtError::InvalidProgram(format!(
                    "scan rule references unknown token {} ({} tokens)",
                    r.token, n_tok
                )));
            }
            if (r.mode as usize) >= n_mode || (r.next_mode as usize) >= n_mode {
                return Err(RtError::InvalidProgram(format!(
                    "scan rule references unknown mode {}/{} ({} modes)",
                    r.mode, r.next_mode, n_mode
                )));
            }
        }
        if (self.start as usize) >= n_rule {
            return Err(RtError::InvalidProgram(format!(
                "start rule {} out of range ({} rules)",
                self.start, n_rule
            )));
        }
        if !self.rule_expects.is_empty() && self.rule_expects.len() != n_rule {
            return Err(RtError::InvalidProgram(format!(
                "rule-expect table length {} != rules {}",
                self.rule_expects.len(),
                n_rule
            )));
        }
        for n in &self.ctx_names {
            if (n.token as usize) >= n_tok {
                return Err(RtError::InvalidProgram(format!(
                    "ctx-name `{}` references unknown token {} ({} tokens)",
                    n.name, n.token, n_tok
                )));
            }
            if n.spelling.is_empty() || n.role.is_empty() || n.name.is_empty() {
                return Err(RtError::InvalidProgram(format!(
                    "ctx-name `{}` has empty name/spelling/role",
                    n.name
                )));
            }
        }
        let bad_tok = |what: &str, t: TokenId| {
            RtError::InvalidProgram(format!(
                "{what} references unknown token {t} ({n_tok} tokens)"
            ))
        };
        let bad_rule = |what: &str, r: RuleId| {
            RtError::InvalidProgram(format!(
                "{what} references unknown rule {r} ({n_rule} rules)"
            ))
        };
        for rule in &self.rules {
            for alt in &rule.alts {
                for s in &alt.steps {
                    match s.atom {
                        Atom::Tok(t) => {
                            if (t as usize) >= n_tok {
                                return Err(bad_tok(&format!("rule `{}` step", rule.name), t));
                            }
                        }
                        Atom::Rule(r) => {
                            if (r as usize) >= n_rule {
                                return Err(bad_rule(&format!("rule `{}` step", rule.name), r));
                            }
                        }
                        Atom::Pratt(x) => {
                            if (x as usize) >= self.pratt.len() {
                                return Err(RtError::InvalidProgram(format!(
                                    "rule `{}` step references unknown pratt {x} ({} tables)",
                                    rule.name,
                                    self.pratt.len()
                                )));
                            }
                        }
                        Atom::CtxName(c) => {
                            if (c as usize) >= self.ctx_names.len() {
                                return Err(RtError::InvalidProgram(format!(
                                    "rule `{}` step references unknown ctx-name {c} ({} entries)",
                                    rule.name,
                                    self.ctx_names.len()
                                )));
                            }
                        }
                    }
                    if let Some(f) = s.field {
                        if (f as usize) >= self.field_names.len() {
                            return Err(RtError::InvalidProgram(format!(
                                "rule `{}` step field {f} outside field table ({} names)",
                                rule.name,
                                self.field_names.len()
                            )));
                        }
                    }
                }
            }
        }
        for t in &self.pratt {
            if (t.primary as usize) >= n_rule {
                return Err(RtError::InvalidProgram(format!(
                    "pratt primary rule {} out of range ({} rules)",
                    t.primary, n_rule
                )));
            }
            for op in &t.ops {
                if (op.op as usize) >= n_tok {
                    return Err(bad_tok("pratt op", op.op));
                }
            }
            if let Some(o) = t.group_open {
                if (o as usize) >= n_tok {
                    return Err(bad_tok("pratt group_open", o));
                }
            }
            if let Some(c) = t.group_close {
                if (c as usize) >= n_tok {
                    return Err(bad_tok("pratt group_close", c));
                }
            }
            if let Some(cp) = &t.call_paren {
                if (cp.open as usize) >= n_tok {
                    return Err(bad_tok("pratt call_paren.open", cp.open));
                }
                if (cp.close as usize) >= n_tok {
                    return Err(bad_tok("pratt call_paren.close", cp.close));
                }
            }
            // RuleId 命名空间（R-1029-01）：调用实参/点后缀续读是规则引用。
            if let Some(r) = t.call_args {
                if (r as usize) >= n_rule {
                    return Err(bad_rule("pratt call_args", r));
                }
            }
            if let Some(r) = t.postfix_tail {
                if (r as usize) >= n_rule {
                    return Err(bad_rule("pratt postfix_tail", r));
                }
            }
        }
        for t in &self.newline.stmt_end {
            if (*t as usize) >= n_tok {
                return Err(bad_tok("newline stmt_end", *t));
            }
        }
        for t in &self.newline.sep_consume {
            if (*t as usize) >= n_tok {
                return Err(bad_tok("newline sep_consume", *t));
            }
        }
        for t in &self.recovery.sync {
            if (*t as usize) >= n_tok {
                return Err(bad_tok("recovery sync", *t));
            }
        }
        Ok(())
    }
}

/// 资源预算（V-11/V-12：显式工作栈 + 可配置预算，无固定嵌套上限）。
#[derive(Clone, Copy, Debug)]
pub struct Budget {
    /// 显式解析工作栈最大深度（无固定语言上限；超出 = ResourceExhausted）。
    pub max_parse_depth: usize,
    /// 单次查询/构建最大工作字节数。
    pub work_bytes: usize,
}

impl Default for Budget {
    fn default() -> Self {
        Budget {
            max_parse_depth: 1 << 20,
            work_bytes: 256 << 20,
        }
    }
}

/// 运行时受控错误（不 panic、不静默截断）。
#[derive(Clone, Debug, PartialEq)]
pub enum RtError {
    /// 语法程序非法（运行时防御性检查；生成期已拒绝的形态不应到达这里）。
    InvalidProgram(String),
    /// 预算耗尽（巨声明/深嵌套）。
    ResourceExhausted { what: &'static str, limit: usize },
    /// 句柄跨树/换代使用。
    BadHandle(String),
    /// 编辑请求非法（错 base/重叠/非 UTF-8 边界）。
    BadEdit(String),
    /// 缺页/坏 hash/错 owner/grammar 不匹配（持久层，adapter 转 miss/重算）。
    Storage(String),
}

impl std::fmt::Display for RtError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RtError::InvalidProgram(m) => write!(f, "invalid grammar program: {m}"),
            RtError::ResourceExhausted { what, limit } => {
                write!(f, "resource exhausted: {what} (limit {limit})")
            }
            RtError::BadHandle(m) => write!(f, "bad handle: {m}"),
            RtError::BadEdit(m) => write!(f, "bad edit: {m}"),
            RtError::Storage(m) => write!(f, "storage: {m}"),
        }
    }
}

impl std::error::Error for RtError {}
