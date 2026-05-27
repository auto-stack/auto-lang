// MCP (Model Context Protocol) Server for AutoVM
//
// Plan 265: AI-first VM interaction protocol.
// Allows AI agents to create isolated VM sessions, execute code,
// and receive structured diagnostics via JSON-RPC over stdio.

pub mod protocol;
pub mod server;
pub mod session_manager;

pub use server::McpServer;

#[cfg(test)]
mod tests;
