//! PLAN-738：stdlib 装配契约——纯模型、装载、provider 目录与校验。
//!
//! 职责边界（计划 §2）：本模块是**装配事实的单源模型层**——
//! - `model`：AssemblyPlan/Manifest/Inventory 的类型（serde 稳定 JSON）；
//! - `loader`：磁盘扫描 + 实际语言 parser 装载，产出 `StdlibInventory`；
//! - `providers`：`stdlib/assembly-providers.json` 声明目录（不作第二份
//!   "声称已实现"的独立表——由真实注册/发射面机器对照校验）；
//! - `provider_selection`（PLAN-947）：四轴（后端/宿主/目标/provider
//!   来源）装配选择合同——目标平台轴决定候选、默认自有 Auto、显式
//!   外库点名、平台 ABI 冒充 AutoABI 为审计反例；
//! - `validate`：诊断码与覆盖检查（核心符号/native 绑定门在 T-04 充实）。
//!
//! 不依赖 Axum/Tokio/UI；解析使用实际 parser（计划 §3），不用 regex 数
//! fn/contains 类型充当契约。验证等级语义见 `model::VerificationLevel`：
//! 文件存在/名称登记最多 resolved/bound，不得自称 executed。

pub mod ace;
pub mod contracts;
pub mod emission;
pub mod host;
pub mod loader;
pub mod manifest;
pub mod model;
pub mod plan;
pub mod provider_selection;
pub mod providers;
pub mod reference;
pub mod validate;
