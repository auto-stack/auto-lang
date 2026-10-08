//! PLAN-738 §5.1：装配清单纯模型。serde 形状即工具输出契约（stable JSON）：
//! 模块/层/符号均排序输出；便携清单只用 stdlib 相对源 ID，不携带机器绝对
//! 路径或运行文件根（local 诊断另行携带，与便携 fingerprint 分开）。

use serde::Serialize;

pub const INVENTORY_SCHEMA_VERSION: u32 = 2;

/// 装配执行/发射目标（计划 §5.2：目标与环境分别记录；Vue 前端不改变后端目标）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AssemblyTarget {
    #[default]
    Vm,
    Rust,
    C,
}

impl AssemblyTarget {
    /// 目标层后缀（parser.rs::get_file_extensions 的 dest→后缀映射由此收编——
    /// 原 dead_code helper 不再是装载事实源，见 backend-assembly.md 装配规则）。
    pub fn context_extension(self) -> &'static str {
        match self {
            AssemblyTarget::Vm => ".vm.at",
            AssemblyTarget::Rust => ".rs.at",
            AssemblyTarget::C => ".c.at",
        }
    }
}

/// 运行环境（与目标正交；Browser 能力门在 T-04 核心 symbol 校验消费）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Environment {
    #[default]
    Native,
    Browser,
}

/// 装配上下文：显式目标 + 环境。默认 Vm×Native（现行为零漂移）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AssemblyContext {
    pub target: AssemblyTarget,
    pub environment: Environment,
}

/// 单模块层选择记录（AssemblyManifest v0 seam；T-06 CLI/生成收据消费）。
/// `public_file`/`context_file`/`candidate_files` 为解析期实际路径（local
/// 诊断面）；便携清单由消费方另行映射 stdlib 相对 ID。
#[derive(Debug, Clone, serde::Serialize)]
pub struct LayerSelection {
    pub module: String,
    pub target: AssemblyTarget,
    pub public_file: String,
    /// 实际合并的目标层（仅 VM 目标；Rust/C 目标为零装载面——决策报告 E1③④）
    pub context_file: Option<String>,
    /// 同目录存在但未选的目标层（candidate——未消费层只能报 candidate）
    pub candidate_files: Vec<String>,
    /// 合并源中目标层段的起始字节（源段错误归因；None=未合并）
    pub context_byte_boundary: Option<usize>,
}

/// 验证等级（计划 §5.1，从弱到强）。
///
/// 文件存在/名称登记最多到 `Resolved`/`Bound`，不得升为 `Executed`；
/// `SignatureChecked` 要求独立逻辑签名核对（不能复制公共 AST 充当两侧）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationLevel {
    Declared,
    Resolved,
    Bound,
    SignatureChecked,
    Executed,
}

/// 目标层类别。`Other` 保留给未来后缀（当前 .at/.vm.at/.rs.at/.c.at）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LayerKind {
    Public,
    Vm,
    Rust,
    C,
    Other,
}

impl LayerKind {
    pub fn from_file_name(name: &str) -> Option<(LayerKind, String)> {
        // 返回 (层类别, 剥层后的基名)；非 .at 结尾 → None
        let (kind, base) = if let Some(b) = name.strip_suffix(".vm.at") {
            (LayerKind::Vm, b)
        } else if let Some(b) = name.strip_suffix(".rs.at") {
            (LayerKind::Rust, b)
        } else if let Some(b) = name.strip_suffix(".c.at") {
            (LayerKind::C, b)
        } else if let Some(b) = name.strip_suffix(".at") {
            (LayerKind::Public, b)
        } else {
            return None;
        };
        Some((kind, base.to_string()))
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ParseStatus {
    Parsed { symbol_count: usize },
    Failed { error: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SymbolKind {
    Fn,
    Method,
    Type,
    Field,
}

/// Logical source contract. Host producers supply their own signature; an
/// inventory declaration alone can never establish SignatureChecked.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LogicalSignature {
    pub parameters: Vec<String>,
    pub parameter_modes: Vec<String>,
    pub returns: String,
    pub is_static: bool,
    pub has_self: bool,
    pub generics: Vec<String>,
    pub attributes: Vec<String>,
}

/// 单符号登记。`name` 为归一身份：方法/类型限定形式 `Type.name`
/// （parser 的 `parent` 字段或 ext target 归一），顶层 fn 用裸名。
#[derive(Debug, Clone, Serialize)]
pub struct SymbolEntry {
    pub name: String,
    pub kind: SymbolKind,
    pub is_pub: bool,
    /// `#[vm]` 声明（FnKind::VmFunction）——VM 层 provider 的 producer 标记；
    /// 公共层出现属 legacy 事实（io.at read_line / sse.at parse_sse）。
    pub is_vm_decl: bool,
    pub arity: usize,
    pub signature: Option<LogicalSignature>,
    pub has_body: bool,
    pub source_span: Option<(usize, usize)>,
    pub verification: VerificationLevel,
}

/// 一层的清点。`file` 为便携源 ID（stdlib 相对、正斜杠）。
#[derive(Debug, Clone, Serialize)]
pub struct LayerInventory {
    pub kind: LayerKind,
    pub file: String,
    /// FNV-1a 64 内容指纹（与 auto-man generation.json 指纹同族）
    pub content_hash: u64,
    #[serde(flatten)]
    pub parse: ParseStatus,
    pub symbols: Vec<SymbolEntry>,
}

/// 一个模块（dot 路径，如 `io`、`encoding.base64`）的全部层。
#[derive(Debug, Clone, Serialize)]
pub struct ModuleInventory {
    pub module: String,
    /// 按 kind 排序（Public 优先）
    pub layers: Vec<LayerInventory>,
}

/// 装配诊断（machine-readable；码表见 `validate::code`）。
#[derive(Debug, Clone, serde::Deserialize, Serialize)]
pub struct AssemblyDiagnostic {
    pub code: String,
    pub module: String,
    pub file: Option<String>,
    pub message: String,
}

/// 全库清点（计划 §5.1：全库 inventory ≠ 全库 strict；解析失败不从总分母
/// 删除，逐项 diagnostics）。
#[derive(Debug, Clone, Serialize)]
pub struct StdlibInventory {
    pub schema_version: u32,
    /// stdlib 来源身份（便携惯例值，如 "stdlib/auto"）
    pub root: String,
    /// 扫描到的 .at 文件总数（公共+目标层合计，分母）
    pub files_total: usize,
    pub modules: Vec<ModuleInventory>,
    pub diagnostics: Vec<AssemblyDiagnostic>,
}

impl StdlibInventory {
    pub fn module(&self, name: &str) -> Option<&ModuleInventory> {
        self.modules.iter().find(|m| m.module == name)
    }

    /// 稳定性自检：模块/层/符号全序（loader 已排序；JSON 输出前断言）。
    pub fn assert_sorted(&self) {
        assert!(self.modules.windows(2).all(|w| w[0].module < w[1].module));
        for m in &self.modules {
            assert!(m.layers.windows(2).all(|w| w[0].kind <= w[1].kind));
            for l in &m.layers {
                assert!(
                    l.symbols.windows(2).all(|w| w[0].name <= w[1].name),
                    "symbols unsorted in {}",
                    l.file
                );
            }
        }
        assert!(self
            .diagnostics
            .windows(2)
            .all(|w| diag_key(&w[0]) <= diag_key(&w[1])));
    }
}

fn diag_key(d: &AssemblyDiagnostic) -> String {
    format!(
        "{}|{}|{}",
        d.code,
        d.module,
        d.file.as_deref().unwrap_or("")
    )
}

/// FNV-1a 64（与 crates/auto-man api_gen 指纹同族，跨面可对照）。
pub fn fnv1a64(data: &str) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in data.as_bytes() {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}
