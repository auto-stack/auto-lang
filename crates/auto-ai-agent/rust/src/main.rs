//! Minimal ReAct Q&A entry point (plan 013 — "make it run" milestone).
//!
//! Constructs an Agent over a builtin role (Assistant) + the real AiClient
//! (talking to the running aaid daemon), runs one turn, and prints the answer.
//! No tools registered — this proves the plain-text Q&A path end to end.

use auto_ai_agent_a2r::agent::Agent;
use auto_ai_agent_a2r::builtin_roles::Assistant;
use auto_ai_client::AiClient;

fn daemon_url() -> String {
    // The aaid daemon listens here by default (see daemon.at). Honor $AAID_URL
    // if set, else use the default.
    std::env::var("AAID_URL").unwrap_or_else(|_| "http://127.0.0.1:17654".into())
}

#[tokio::main]
async fn main() {
    let url = daemon_url();
    println!("[react] talking to daemon at {url}");
    let client = AiClient::with_url(&url);
    let role = Assistant {};
    let mut agent = Agent::new_shared(Box::new(role), Box::new(client));

    let task = "你好，请用一句话介绍你自己。";
    println!("[react] task: {task}");
    match agent.run(task).await {
        Ok(result) => {
            println!("[react] turns: {}", result.turns);
            println!("[react] answer: {}", result.output);
        }
        Err(e) => {
            eprintln!("[react] error: {}", e.message());
            std::process::exit(1);
        }
    }
}
