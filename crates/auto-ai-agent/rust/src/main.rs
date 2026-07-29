//! Auto-ported ReAct agent REPL (plan 013 — G2+G3: tools + interactive loop).
//!
//! Constructs an Agent over Assistant role + AiClient, registers the EchoTool,
//! then loops reading stdin questions and printing answers (+ tool-call info).
//! /exit quits; empty lines are skipped.

use std::io::{self, BufRead, Write};
use std::sync::Arc;
use auto_ai_agent_a2r::agent::Agent;
use auto_ai_agent_a2r::builtin_roles::Assistant;
use auto_ai_agent_a2r::echo_tool::EchoTool;
use auto_ai_client::AiClient;

fn daemon_url() -> String {
    std::env::var("AAID_URL").unwrap_or_else(|_| "http://127.0.0.1:17654".into())
}

#[tokio::main]
async fn main() {
    let url = daemon_url();
    eprintln!("[react] daemon at {url}");

    let client = AiClient::with_url(&url);
    let role = Assistant {};
    let mut agent = Agent::new_shared(Box::new(role), Box::new(client));

    // Register the echo tool so the model can invoke it.
    agent.register_shared(Arc::new(Box::new(EchoTool {})));

    eprintln!("[react] ready.  Type a question (or /exit).");
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        // prompt
        write!(stdout, "> ").unwrap();
        stdout.flush().unwrap();

        let mut line = String::new();
        match stdin.lock().read_line(&mut line) {
            Ok(0) => break,                   // EOF
            Err(e) => {
                eprintln!("[react] read error: {e}");
                break;
            }
            _ => {}
        }

        let input = line.trim().to_string();
        if input.is_empty() {
            continue;
        }
        if input == "/exit" {
            eprintln!("[react] bye.");
            break;
        }

        match agent.run(&input).await {
            Ok(result) => {
                if !result.tool_calls.is_empty() {
                    eprintln!("[react] tool calls this turn:");
                    for tc in &result.tool_calls {
                        eprintln!("  • {} : {}", tc.tool, tc.result);
                    }
                }
                println!("{}", result.output);
                eprintln!("  ({} turn{}, {} tokens)",
                    result.turns,
                    if result.turns == 1 { "" } else { "s" },
                    result.total_tokens);
            }
            Err(e) => {
                eprintln!("[react] error: {}", e.message());
                // keep running — don't exit on one bad turn
            }
        }
    }
}
