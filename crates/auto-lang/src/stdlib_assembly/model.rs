//! PLAN-738 §5.1：装配清单纯模型。serde 形状即工具输出契约（stable JSON）：
//! 模块/层/符号均排序输出；便携清单只用 stdlib 相对源 ID，不携带机器绝对
//! 路径或运行文件根（local 诊断另行携带，与便携 fingerprint 分开）。

use serde::Serialize;

pub const INVENTORY_SCHEMA_VERSION: u32 = 1;

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
    Parsed {
        symbol_count: usize,
    },
    Failed {
        error: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SymbolKind {
    Fn,
    Method,
    Type,
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
        assert!(self.diagnostics.windows(2).all(|w| diag_key(&w[0]) <= diag_key(&w[1])));
    }
}

fn diag_key(d: &AssemblyDiagnostic) -> String {
    format!("{}|{}|{}", d.code, d.module, d.file.as_deref().unwrap_or(""))
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
