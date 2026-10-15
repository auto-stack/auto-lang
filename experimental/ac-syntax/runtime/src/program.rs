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
pub const GRAMMAR_SCHEMA_VERSION: u32 = 1;
/// 程序规范二进制序列化格式版本（语言包装载格式，实验 Schema，非正式 Batom）。
pub const PROGRAM_CODEC_VERSION: u16 = 1;

pub type RuleId = u32;
pub type TokenId = u32;
pub type PrattId = u32;

/// ASCII 位集 + 非 ASCII 许可（core 拒绝非 ASCII 名；mini 亦仅 ASCII）。
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CharSet {
    pub bits: [u64; 2],
    pub non_ascii: bool,
}

impl CharSet {
    pub fn empty() -> Self {
        CharSet { bits: [0; 2], non_ascii: false }
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
    Charset { first: CharSet, cont: CharSet, min_len: u32 },
    /// 行注释：start 字面量到（不含）换行。
    LineComment(String),
    /// 块注释：start..end，可嵌套（interface-mini 的 /* */ 嵌套）。
    BlockComment { start: String, end: String, nested: bool },
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
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Atom {
    Tok(TokenId),
    Rule(RuleId),
    Pratt(PrattId),
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
        self.field_names.iter().position(|f| f == name).map(|i| i as u32)
    }
    /// 规范二进制（身份计算与持久化共用同一布局，PROGRAM_CODEC_VERSION 前缀）。
    pub fn canonical_bytes(&self) -> Vec<u8> {
        crate::program_codec::encode_canonical(self)
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
        Budget { max_parse_depth: 1 << 20, work_bytes: 64 << 20 }
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
