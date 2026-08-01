//! auto-ai-agent — assembled from a2r-transpiled `.a2r.rs` files.
//!
//! Assembly (plan 013, option A — proven by auto-coder/coder/rust/):
//! - ALL modules are FLAT at the crate root (a2r assumes this: subdirectory
//!   files like `orchestration/budget.at` transpile to `use crate::budget::`,
//!   i.e. crate-root, not `use crate::orchestration::budget`). So config/,
//!   orchestration/, builtin_roles/ files are hoisted to root (renamed to
//!   avoid clashes), and `config.rs`/`orchestration.rs` are hand-written
//!   aggregators re-exporting the hoisted modules.
//! - a2r emits `use crate::<extern>::{...}` for extern crates; the
//!   `pub mod <name> { pub use ::<name>::*; }` shims resolve them.
//! - `JsonValue` is aliased to serde_json::Value (a2r uses it as a generic
//!   JSON blob type; wire.rs doesn't define it).
//! - `client_impl` is hand-written `impl Client for AiClient`.

// ── extern-crate shims (a2r emits `use crate::<these>::...`) ────────────────
pub mod auto_ai_client {
    pub use ::auto_ai_client::*;
}
pub mod ai_config {
    pub use ::ai_config::*;
}
pub mod wire {
    pub use ::ai_config::wire::*;
    // a2r references `crate::wire::JsonValue` but wire.rs has no JsonValue.
    // It's the generic JSON blob type = serde_json::Value.
    pub type JsonValue = serde_json::Value;
}

// ── hand-written glue ───────────────────────────────────────────────────────
pub mod client_impl;
pub mod echo_tool;

// ── a2r-transpiled modules (FLAT at crate root) ─────────────────────────────
pub mod agent;
pub mod error;
pub mod memory;
pub mod relay;
pub mod role_def;
pub mod role_config;
pub mod roles;
pub mod skill;
pub mod tool;
pub mod validate;
pub mod workflow;
pub mod workflow_validator;

// orchestration (hoisted) + aggregator
pub mod budget;
pub mod driver;
pub mod flow;
pub mod handoff;
pub mod pipeline;
pub mod orchestration; // aggregator: re-exports the above as crate::orchestration::*

// config aggregator (re-exports role_config as crate::config::*)
pub mod config;

// builtin roles (hoisted; builtin_roles.rs is the mod.at aggregator with
// load_builtin/builtin_names; the per-role files are builtin_role_<name>.rs)
pub mod builtin_roles;
pub mod builtin_role_advisor;
pub mod builtin_role_architect;
pub mod builtin_role_assistant;
pub mod builtin_role_coder;
pub mod builtin_role_documenter;
pub mod builtin_role_gofer;
pub mod builtin_role_planner;
pub mod builtin_role_reviewer;
pub mod builtin_role_runner;
pub mod builtin_role_super_advisor;
pub mod builtin_role_super_coder;
pub mod builtin_role_super_tester;
pub mod builtin_role_tester;
pub mod builtin_role_translator;

// ── crate-root re-exports (mirrors lib.at) ──────────────────────────────────
pub use agent::{Agent, AgentResult, Client, StreamEvent, ToolCallRecord};
pub use role_config::{ConfigRole, RoleConfig, config_role_new, config_role_with_base, load_role, parse_at_role, parse_tier_field, serialize_at_role};
pub use error::{AgentError, ToolError};
pub use memory::Memory;
pub use role_def::Role;
pub use ai_config::ModelTier;
pub use builtin_roles::{load_builtin, builtin_names, Assistant, Architect, Coder, Documenter, Reviewer, Runner, Tester, Translator};
pub use roles::{RoleDetail, RoleRegistry, RoleSummary};
pub use skill::{Skill, SkillRegistry, SkillTool};
pub use tool::{Tool, ToolRegistry};
pub use validate::{load_client_config, validate_role_model};
pub use orchestration::{
    BudgetAction, BudgetStrategy, BudgetTracker, TokenBudget,
    ContextPointers, Decision, HandoffDocument, Question, TokenUsage, WorkProduct,
    FlowSpec, FlowStep, GateType, ExitRouting,
    PipelineEngine, PipelineStatus, PipelineMode, AdvanceResult, StepRecord,
    AgentFactory, PipelineDriver, PipelineEvent,
};
