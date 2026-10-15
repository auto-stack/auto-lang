//! ac-syntax-rt — 通用语法运行时（PLAN-787）。
//!
//! 依赖方向（AGENTS/Design39）：snapshot → grammar-program/scanner → CST → 结构查询。
//! 本 crate 不依赖 ac-core / Cranelift / auto-cache / Auto 语义环境；
//! 语法程序（`program::GrammarProgram`）由 ac-syntax-grammar 从 Atom 配置编译而来。
//!
//! 所有多树遍历/构建/序列化路径均为显式栈迭代实现（V-12 深度约束）。

pub mod identity;
pub mod program;
pub mod program_codec;
pub mod snapshot;
pub mod position;
pub mod handle;
pub mod lexer;
pub mod cst;
pub mod parse;
pub mod recovery;
pub mod edit;
pub mod outline;

/// 测试/探针用最小语法程序构造器（doc(hidden)：非产品 API）。
#[doc(hidden)]
pub mod testkit;

/// 语法配置 schema 版本字符串（参与 GrammarId）。
pub const GRAMMAR_SCHEMA_VERSION_STR: &str = "ac787/grammar-schema/1";

/// 运行时（执行器）版本，参与 A01/A02/A03 产物 key。
pub const RUNTIME_VERSION: &str = "ac-syntax-rt/1";

/// 词法 scanner ABI 版本（checkpoint 布局语义），参与 A02 key。
pub const SCANNER_ABI_VERSION: &str = "ac-syntax-scanner/1";

/// 解析器 ABI 版本（规则程序语义），参与 A03 key。
pub const PARSER_ABI_VERSION: &str = "ac-syntax-parser/1";
