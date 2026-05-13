//! Agent Turn Engine
//!
//! Extracts the ReAct loop from the Forge chat handler into a reusable,
//! parameterized component. Each AgentTurn runs one agent's step in the
//! relay pipeline: it holds the baton, executes tools, and produces a
//! result that can be turned into a HandoffDocument.

use crate::forge::ai::{ChatMessage, ContentBlock, ToolChatEvent, ToolChatRequest, ToolClaudeProvider};
use crate::forge::tools::{ToolDefinition, ToolRegistry};
use crate::relay::agent::AgentInstance;
use crate::relay::budget::{BudgetAction, BudgetTracker};
use crate::relay::handoff::HandoffDocument;
use serde_json::Value;

/// Events emitted during an agent turn.
#[derive(Debug, Clone)]
pub enum TurnEvent {
    /// A text delta from the LLM.
    TextDelta { text: String },
    /// The agent wants to use a tool.
    ToolCall { id: String, name: String, arguments: Value },
    /// Tool execution completed.
    ToolResult { id: String, result: String },
    /// The turn completed normally (no more tool calls).
    Complete,
    /// An error occurred.
    Error { message: String },
    /// Budget warning fired.
    BudgetWarning { remaining: u64 },
    /// Budget hard-stopped the turn.
    BudgetExceeded,
}

/// Result of a completed agent turn.
#[derive(Debug, Clone)]
pub struct TurnResult {
    /// Full assistant text produced during the turn.
    pub assistant_text: String,
    /// Tool calls made during the turn.
    pub tool_calls: Vec<ToolCallRecord>,
    /// Total tokens consumed this turn.
    pub tokens_used: u64,
    /// Whether the agent explicitly called the `handoff` tool.
    pub handoff_requested: bool,
    /// Decisions extracted from the turn text.
    pub decisions: Vec<String>,
    /// Open questions extracted from the turn text.
    pub open_questions: Vec<String>,
    /// Files touched during this turn.
    pub files_touched: Vec<String>,
}

/// Record of a single tool invocation.
#[derive(Debug, Clone)]
pub struct ToolCallRecord {
    pub id: String,
    pub name: String,
    pub arguments: Value,
    pub result: String,
}

/// Reusable agent turn engine.
pub struct AgentTurn {
    pub agent: AgentInstance,
    /// Filtered tool definitions for this profession.
    pub tool_definitions: Vec<ToolDefinition>,
    /// Full tool registry for execution.
    pub tool_registry: ToolRegistry,
    /// Conversation history (mutable during the turn).
    pub messages: Vec<ChatMessage>,
    /// Max LLM turns.
    pub max_turns: u32,
    /// Budget tracker for this run.
    pub budget_tracker: Option<BudgetTracker>,
}

impl AgentTurn {
    /// Create a new AgentTurn from an agent instance.
    /// Filters tools to only those the profession is allowed to use.
    pub fn new(
        agent: AgentInstance,
        registry: ToolRegistry,
        messages: Vec<ChatMessage>,
    ) -> Self {
        let allowed: Vec<String> = agent.profession.allowed_tools.clone();
        let tool_definitions: Vec<ToolDefinition> = if allowed.is_empty() {
            // If no tools are explicitly allowed, allow none (intaker/documenter)
            Vec::new()
        } else {
            registry
                .definitions()
                .into_iter()
                .filter(|d| allowed.contains(&d.name))
                .collect()
        };

        Self {
            agent,
            tool_definitions,
            tool_registry: registry,
            messages,
            max_turns: 10,
            budget_tracker: None,
        }
    }

    /// Run the ReAct loop until completion, error, or budget exhaustion.
    /// Events are sent via `tx` so callers can observe progress in real time.
    pub async fn run(
        &mut self,
        provider: &ToolClaudeProvider,
        tx: tokio::sync::mpsc::UnboundedSender<TurnEvent>,
    ) -> TurnResult {
        let mut result = TurnResult {
            assistant_text: String::new(),
            tool_calls: Vec::new(),
            tokens_used: 0,
            handoff_requested: false,
            decisions: Vec::new(),
            open_questions: Vec::new(),
            files_touched: Vec::new(),
        };

        let system_prompt = self.agent.render_system_prompt();
        let mut turn_count = 0;

        while turn_count < self.max_turns {
            turn_count += 1;
            self.agent.context.turns_taken = turn_count;

            // Budget check before turn
            if let Some(ref tracker) = self.budget_tracker {
                match tracker.check(&self.agent.profession.id) {
                    BudgetAction::Warning { remaining } => {
                        let _ = tx.send(TurnEvent::BudgetWarning { remaining });
                    }
                    BudgetAction::HardStop => {
                        let _ = tx.send(TurnEvent::BudgetExceeded);
                        break;
                    }
                    _ => {}
                }
            }

            let request = ToolChatRequest {
                messages: self.messages.clone(),
                tools: self.tool_definitions.clone(),
                system_prompt: Some(system_prompt.clone()),
            };

            let (turn_tx, mut turn_rx) = tokio::sync::mpsc::unbounded_channel::<ToolChatEvent>();
            let turn_task = provider.chat_turn(request, turn_tx);

            let mut got_tool_use = false;
            let mut turn_text = String::new();
            let mut turn_tools: Vec<ToolCallRecord> = Vec::new();

            while let Some(event) = turn_rx.recv().await {
                match event {
                    ToolChatEvent::TextDelta { text } => {
                        turn_text.push_str(&text);
                        let _ = tx.send(TurnEvent::TextDelta { text: text.clone() });
                    }
                    ToolChatEvent::ToolUse { id, name, input } => {
                        got_tool_use = true;
                        let _ = tx.send(TurnEvent::ToolCall {
                            id: id.clone(),
                            name: name.clone(),
                            arguments: input.clone(),
                        });

                        // Execute the tool
                        let exec_result = if let Some(tool) = self.tool_registry.get(&name) {
                            match tool.execute(input.clone()) {
                                Ok(r) => r,
                                Err(e) => format!("Error: {}", e),
                            }
                        } else {
                            format!("Tool '{}' not found or not allowed for profession '{}'", name, self.agent.profession.id)
                        };

                        let _ = tx.send(TurnEvent::ToolResult {
                            id: id.clone(),
                            result: exec_result.clone(),
                        });

                        // Track special tools
                        if name == "handoff" {
                            result.handoff_requested = true;
                        }
                        if name == "read_file" || name == "write_file" || name == "edit_file" {
                            if let Some(path) = input.get("path").and_then(|v| v.as_str()) {
                                if !result.files_touched.contains(&path.to_string()) {
                                    result.files_touched.push(path.to_string());
                                }
                            }
                        }

                        turn_tools.push(ToolCallRecord {
                            id: id.clone(),
                            name: name.clone(),
                            arguments: input,
                            result: exec_result.clone(),
                        });

                        self.messages.push(ChatMessage::tool_result(&id, &exec_result));
                    }
                    ToolChatEvent::Done => break,
                    ToolChatEvent::Error { message } => {
                        let _ = tx.send(TurnEvent::Error { message: message.clone() });
                        result.assistant_text = turn_text;
                        result.tool_calls = turn_tools;
                        return result;
                    }
                }
            }

            // Check for turn-level errors from the provider
            if let Some(err) = turn_task.await {
                let _ = tx.send(TurnEvent::Error { message: err });
                result.assistant_text = turn_text;
                result.tool_calls = turn_tools;
                return result;
            }

            // Persist assistant message for next turn
            if !turn_text.is_empty() || !turn_tools.is_empty() {
                if got_tool_use {
                    let mut blocks = vec![ContentBlock::text(&turn_text)];
                    for call in &turn_tools {
                        blocks.push(ContentBlock::ToolUse {
                            id: call.id.clone(),
                            name: call.name.clone(),
                            input: call.arguments.clone(),
                        });
                    }
                    self.messages.push(ChatMessage {
                        role: "assistant".to_string(),
                        content: blocks,
                    });
                } else {
                    self.messages.push(ChatMessage::assistant_text(&turn_text));
                }
            }

            result.assistant_text.push_str(&turn_text);
            result.tool_calls.extend(turn_tools);

            // If no tool_use was requested, we're done
            if !got_tool_use {
                break;
            }

            // If handoff was explicitly requested, stop
            if result.handoff_requested {
                break;
            }
        }

        // Extract decisions and questions from text (simple heuristics)
        result.decisions = extract_section(&result.assistant_text, "Decisions Made");
        result.open_questions = extract_section(&result.assistant_text, "Open Questions");

        let _ = tx.send(TurnEvent::Complete);
        result
    }

    /// Generate a HandoffDocument from this turn's result.
    pub fn to_handoff(
        &self,
        result: &TurnResult,
        to_profession: &str,
        run_id: &str,
        checkpoint_id: u64,
    ) -> HandoffDocument {
        let mut handoff = HandoffDocument::new(
            &self.agent.profession.id,
            to_profession,
            run_id,
            checkpoint_id,
        );
        handoff.summary = format!(
            "{} completed their work in {} turns. Produced {} tool calls.",
            self.agent.profession.name,
            self.agent.context.turns_taken,
            result.tool_calls.len()
        );
        for d in &result.decisions {
            handoff.decisions.push(crate::relay::handoff::Decision {
                id: format!("D-{}", handoff.decisions.len() + 1),
                title: d.clone(),
                status: "made".to_string(),
                rationale: String::new(),
            });
        }
        for q in &result.open_questions {
            handoff.open_questions.push(crate::relay::handoff::Question {
                id: format!("Q-{}", handoff.open_questions.len() + 1),
                text: q.clone(),
                status: "open".to_string(),
                assigned_to: None,
            });
        }
        for f in &result.files_touched {
            handoff.work_product.push(crate::relay::handoff::WorkProduct {
                path: f.clone(),
                description: String::new(),
                lines: None,
            });
        }
        handoff.token_usage = crate::relay::handoff::TokenUsage {
            step_input: result.tokens_used / 2, // estimate
            step_output: result.tokens_used / 2,
            cumulative: result.tokens_used,
            budget_remaining: self
                .agent
                .profession
                .token_budget
                .saturating_sub(result.tokens_used),
        };
        handoff
    }
}

/// Simple heuristic: extract bullet items under a heading.
fn extract_section(text: &str, heading: &str) -> Vec<String> {
    let mut results = Vec::new();
    let mut in_section = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.eq_ignore_ascii_case(&format!("## {}", heading))
            || trimmed.eq_ignore_ascii_case(&format!("### {}", heading))
        {
            in_section = true;
            continue;
        }
        if in_section {
            if trimmed.starts_with("## ") || trimmed.starts_with("### ") {
                break;
            }
            if trimmed.starts_with("-") || trimmed.starts_with("*") {
                results.push(trimmed[1..].trim().to_string());
            }
        }
    }
    results
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::relay::agent::AgentInstance;
    use crate::relay::profession::{ForgePhase, Profession};
    use crate::relay::soul::SoulConfig;

    fn make_test_agent() -> AgentInstance {
        let profession = Profession {
            id: "tester".to_string(),
            name: "Tester".to_string(),
            phase: ForgePhase::Execution,
            owned_sections: vec![],
            readable_sections: vec![],
            allowed_tools: vec!["read_file".to_string()],
            handoff_to: vec![],
            approval_gates: vec![],
            max_turns: 5,
            token_budget: 10_000,
        };
        let soul = SoulConfig::parse("tester", "# Soul of the Tester\n\n## Core Values\n- Test everything\n").unwrap();
        AgentInstance::spawn(profession, soul, crate::relay::agent::ModelConfig::cheap())
    }

    #[test]
    fn test_agent_turn_filters_tools() {
        let agent = make_test_agent();
        let registry = ToolRegistry::new();
        let turn = AgentTurn::new(agent, registry, vec![]);

        // Only read_file is allowed for the tester profession
        let names: Vec<String> = turn.tool_definitions.iter().map(|d| d.name.clone()).collect();
        assert!(names.contains(&"read_file".to_string()));
        assert!(!names.contains(&"write_file".to_string()));
    }

    #[test]
    fn test_extract_section() {
        let text = r#"## Decisions Made
- Use JWT instead of sessions
- Add refresh token rotation

## Open Questions
- Should we support OAuth1?
"#;
        let decisions = extract_section(text, "Decisions Made");
        assert_eq!(decisions.len(), 2);
        assert!(decisions[0].contains("JWT"));

        let questions = extract_section(text, "Open Questions");
        assert_eq!(questions.len(), 1);
        assert!(questions[0].contains("OAuth1"));
    }
}
