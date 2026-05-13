//! Agent Instance
//!
//! Combines Soul + Profession + Model into a runnable agent identity.
//! Handles context assembly and system prompt rendering.

use crate::forge::ai::{ChatMessage, ToolChatRequest};
use crate::forge::tools::ToolDefinition;
use crate::relay::profession::Profession;
use crate::relay::soul::SoulConfig;
use serde::{Deserialize, Serialize};

/// Cognitive substrate configuration for an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub provider: Provider,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: u32,
    pub reasoning_budget: Option<u32>,
    /// Ordered list of fallback model names if the primary fails.
    pub fallback_chain: Vec<String>,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            provider: Provider::Anthropic,
            model: String::from("claude-3-5-sonnet-20241022"),
            temperature: 0.3,
            max_tokens: 4096,
            reasoning_budget: None,
            fallback_chain: vec![
                String::from("claude-3-5-sonnet-20241022"),
                String::from("gpt-4o"),
            ],
        }
    }
}

impl ModelConfig {
    pub fn cheap() -> Self {
        Self {
            provider: Provider::Anthropic,
            model: String::from("claude-3-5-haiku-20241022"),
            temperature: 0.2,
            max_tokens: 2048,
            reasoning_budget: None,
            fallback_chain: vec![String::from("claude-3-5-haiku-20241022")],
        }
    }

    pub fn standard() -> Self {
        Self {
            provider: Provider::Anthropic,
            model: String::from("claude-3-5-sonnet-20241022"),
            temperature: 0.3,
            max_tokens: 4096,
            reasoning_budget: None,
            fallback_chain: vec![
                String::from("claude-3-5-sonnet-20241022"),
                String::from("gpt-4o"),
            ],
        }
    }

    pub fn strong() -> Self {
        Self {
            provider: Provider::Anthropic,
            model: String::from("claude-3-opus-20240229"),
            temperature: 0.2,
            max_tokens: 8192,
            reasoning_budget: Some(4000),
            fallback_chain: vec![
                String::from("claude-3-opus-20240229"),
                String::from("claude-3-5-sonnet-20241022"),
                String::from("gpt-4o"),
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Provider {
    Anthropic,
    OpenAI,
    Local { url: String },
}

/// Per-turn mutable state for an agent.
#[derive(Debug, Clone, Default)]
pub struct AgentContext {
    /// Tokens consumed this step.
    pub budget_used: u64,
    /// Number of LLM turns taken this step.
    pub turns_taken: u32,
    /// Files touched during this step.
    pub files_touched: Vec<String>,
    /// Decisions made during this step.
    pub decisions: Vec<String>,
    /// Open questions raised during this step.
    pub open_questions: Vec<String>,
}

/// A runnable agent instance — ephemeral, spawned per step.
pub struct AgentInstance {
    pub id: String,
    pub profession: Profession,
    pub soul: SoulConfig,
    pub model: ModelConfig,
    pub context: AgentContext,
}

impl AgentInstance {
    /// Spawn a new agent for a pipeline step.
    pub fn spawn(
        profession: Profession,
        soul: SoulConfig,
        model: ModelConfig,
    ) -> Self {
        Self {
            id: format!("agent-{}", uuid::Uuid::new_v4()),
            profession,
            soul,
            model,
            context: AgentContext::default(),
        }
    }

    /// Render the system prompt from Soul + Profession + constraints.
    pub fn render_system_prompt(&self) -> String {
        let mut parts = Vec::new();

        // Soul identity
        parts.push(self.soul.render());

        // Profession scope
        parts.push(format!(
            "## Profession: {}\n\nYou are a {}. Your phase is {}.\n",
            self.profession.name,
            self.profession.name,
            self.profession.phase.as_str()
        ));

        if !self.profession.owned_sections.is_empty() {
            let sections: Vec<String> = self.profession.owned_sections.iter()
                .map(|s| s.as_str().to_string())
                .collect();
            parts.push(format!(
                "You OWN these spec sections and may write to them: {}\n",
                sections.join(", ")
            ));
        }

        if !self.profession.readable_sections.is_empty() {
            let sections: Vec<String> = self.profession.readable_sections.iter()
                .map(|s| s.as_str().to_string())
                .collect();
            parts.push(format!(
                "You may READ these spec sections for context: {}\n",
                sections.join(", ")
            ));
        }

        if !self.profession.allowed_tools.is_empty() {
            parts.push(format!(
                "You may use these tools: {}\n",
                self.profession.allowed_tools.join(", ")
            ));
        }

        // Constraints
        parts.push(format!(
            "\n## Constraints\n- Max turns before handoff: {}\n- Token budget: {}\n",
            self.profession.max_turns,
            self.profession.token_budget
        ));

        parts.join("\n")
    }

    /// Build the initial user message from handoff + relevant specs.
    pub fn render_user_message(&self, handoff_summary: &str, spec_summary: &str) -> Vec<ChatMessage> {
        let mut content = String::new();
        if !handoff_summary.is_empty() {
            content.push_str("## Previous Agent's Handoff\n\n");
            content.push_str(handoff_summary);
            content.push_str("\n\n---\n\n");
        }
        if !spec_summary.is_empty() {
            content.push_str("## Relevant Specs\n\n");
            content.push_str(spec_summary);
            content.push_str("\n\n---\n\n");
        }
        content.push_str("Begin your work now. When you are ready to hand off, call the `handoff` tool.");

        vec![ChatMessage::user(&content)]
    }

    /// Build a complete ToolChatRequest for this agent's turn.
    pub fn build_chat_request(
        &self,
        tools: Vec<ToolDefinition>,
        handoff_summary: &str,
        spec_summary: &str,
    ) -> ToolChatRequest {
        let system = self.render_system_prompt();
        let messages = self.render_user_message(handoff_summary, spec_summary);
        ToolChatRequest {
            messages,
            tools,
            system_prompt: Some(system),
        }
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::relay::profession::{ForgePhase, ProfessionRegistry};
    use crate::relay::soul::SoulConfig;

    #[test]
    fn test_agent_spawn() {
        let profession = ProfessionRegistry::new().get("planner").unwrap().clone();
        let soul = SoulConfig::parse("planner", "# Soul of the Planner\n\n## Core Values\n- Careful planning\n").unwrap();
        let model = ModelConfig::standard();
        let agent = AgentInstance::spawn(profession, soul, model);
        assert!(agent.id.starts_with("agent-"));
        assert_eq!(agent.profession.id, "planner");
    }

    #[test]
    fn test_render_system_prompt() {
        let profession = ProfessionRegistry::new().get("planner").unwrap().clone();
        let soul = SoulConfig::parse("planner", "# Soul of the Planner\n\n## Core Values\n- Careful planning\n").unwrap();
        let model = ModelConfig::standard();
        let agent = AgentInstance::spawn(profession, soul, model);
        let prompt = agent.render_system_prompt();
        assert!(prompt.contains("Soul of the Planner"));
        assert!(prompt.contains("Profession: Planner"));
        assert!(prompt.contains("You OWN these spec sections"));
        assert!(prompt.contains("goals"));
        assert!(prompt.contains("plans"));
    }

    #[test]
    fn test_model_tiers() {
        let cheap = ModelConfig::cheap();
        assert!(cheap.model.contains("haiku"));

        let standard = ModelConfig::standard();
        assert!(standard.model.contains("sonnet"));

        let strong = ModelConfig::strong();
        assert!(strong.model.contains("opus"));
    }
}
