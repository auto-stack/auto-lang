//! A simple echo tool for testing the ReAct loop's tool-calling path.
//! Returns "ECHO: <input>" — lets the model practice calling a tool and
//! seeing its result fed back. Hand-written (no .at source).

use crate::tool::Tool;
use crate::wire::JsonValue;
use crate::error::ToolError;
use std::pin::Pin;
use std::future::Future;

pub struct EchoTool;

impl Tool for EchoTool {
    fn name(&self) -> String {
        "echo".into()
    }

    fn description(&self) -> String {
        "Echoes back the input message. Use this to test tool calling.".into()
    }

    fn parameters(&self) -> JsonValue {
        a2r_std::json::parse("{\"type\":\"object\",\"properties\":{\"message\":{\"type\":\"string\",\"description\":\"The message to echo back\"}},\"required\":[\"message\"]}")
    }

    fn execute<'a>(&'a self, args: JsonValue) -> Pin<Box<dyn Future<Output = Result<String, ToolError>> + Send + 'a>> {
        Box::pin(async move {
            let msg = args.get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            Ok(format!("ECHO: {}", msg))
        })
    }
}
