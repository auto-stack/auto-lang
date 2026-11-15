// MCP (Model Context Protocol) Server for AutoVM
//
// Plan 265: AI-first VM interaction protocol.
// Allows AI agents to create isolated VM sessions, execute code,
// and receive structured diagnostics via JSON-RPC over stdio.

/// PLAN-970: the explicit-ACE session MCP server (the legacy server above
/// is untouched; retirement belongs to PLAN-987).
pub mod ace_server;
pub mod protocol;
pub mod server;
pub mod session_manager;

pub use ace_server::AceMcpServer;
pub use server::McpServer;
