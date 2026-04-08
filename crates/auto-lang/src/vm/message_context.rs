//! Plan 125: Phase 3.4 - MessageContext Runtime
//!
//! Message context for task message handling with reply capability.
//!
//! ## Overview
//!
//! When a task receives a message via `ask()`, the handler can reply using
//! the context parameter:
//!
//! ```auto
//! on(ctx) {
//!     "ping" => { ctx.reply("pong") }
//!     amount int if amount > 10000 => { ctx.reply("need_approval") }
//!     amount int => { ctx.reply("approved") }
//! }
//! ```
//!
//! The MessageContext provides:
//! - `sender_id`: Optional ID of the sending task
//! - `trace_id`: Distributed tracing ID
//! - `is_ask`: Whether this is an ask (request-response) message
//! - `reply()`: Send a response back to the caller (only available in ask mode)

use auto_val::Value;
use std::sync::mpsc::Sender;
use std::fmt;

/// Message context - runtime representation for task message handlers
///
/// This struct is passed to task message handlers when the `on(ctx)` syntax
/// is used. It provides access to message metadata and the reply channel.
#[derive(Debug)]
pub struct MessageContext {
    /// Sender task ID (may be None for external messages)
    pub sender_id: Option<u64>,
    /// Distributed tracing ID for correlation
    pub trace_id: String,
    /// Whether this is an ask (request-response) message
    pub is_ask: bool,
    /// Reply channel (only set for ask mode)
    /// Uses std::sync::mpsc for synchronous reply (works without tokio runtime)
    reply_tx: Option<Sender<Value>>,
}

impl MessageContext {
    /// Create a new context for send (fire-and-forget) mode
    ///
    /// In send mode, `reply()` will return an error since there's no
    /// reply channel available.
    pub fn new(sender_id: Option<u64>, trace_id: String) -> Self {
        Self {
            sender_id,
            trace_id,
            is_ask: false,
            reply_tx: None,
        }
    }

    /// Create a context for ask (request-response) mode
    ///
    /// In ask mode, `reply()` can be called exactly once to send a
    /// response back to the caller.
    pub fn for_ask(
        sender_id: Option<u64>,
        trace_id: String,
        reply_tx: Sender<Value>,
    ) -> Self {
        Self {
            sender_id,
            trace_id,
            is_ask: true,
            reply_tx: Some(reply_tx),
        }
    }

    /// Create a context with a pre-built reply channel
    ///
    /// This is useful for testing or when the reply channel is created
    /// externally.
    pub fn with_reply_channel(
        sender_id: Option<u64>,
        trace_id: impl Into<String>,
        reply_tx: Option<Sender<Value>>,
    ) -> Self {
        Self {
            sender_id,
            trace_id: trace_id.into(),
            is_ask: reply_tx.is_some(),
            reply_tx,
        }
    }

    /// Reply to the sender (only available in ask mode)
    ///
    /// # Arguments
    ///
    /// * `payload` - The value to send back to the caller
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Reply was sent successfully
    /// * `Err(String)` - Reply failed (no channel or channel closed)
    ///
    /// # Example
    ///
    /// ```auto
    /// on(ctx) {
    ///     "ping" => { ctx.reply("pong") }
    /// }
    /// ```
    pub fn reply(&self, payload: Value) -> Result<(), String> {
        if let Some(tx) = &self.reply_tx {
            tx.send(payload).map_err(|_| "Reply channel closed".to_string())
        } else {
            Err("No reply channel available (not in ask mode)".to_string())
        }
    }

    /// Check if this context can send a reply
    ///
    /// Returns `true` if this is an ask message and the reply channel
    /// is still available.
    pub fn can_reply(&self) -> bool {
        self.reply_tx.is_some()
    }

    /// Check if this is an ask (request-response) message
    pub fn is_ask(&self) -> bool {
        self.is_ask
    }

    /// Get the sender task ID
    pub fn sender_id(&self) -> Option<u64> {
        self.sender_id
    }

    /// Get the trace ID for distributed tracing
    pub fn trace_id(&self) -> &str {
        &self.trace_id
    }

    /// Convert to a Value representation for VM
    ///
    /// This creates an object value that can be used in the VM:
    /// ```text
    /// {
    ///     sender_id: ?i64,
    ///     trace_id: string,
    ///     is_ask: bool,
    ///     can_reply: bool,
    /// }
    /// ```
    pub fn to_value(&self) -> Value {
        use auto_val::Obj;
        use auto_val::AutoStr;

        let mut obj = Obj::new();
        obj.set(AutoStr::from("sender_id"), self.sender_id.map_or(Value::Nil, |id| Value::I64(id as i64)));
        obj.set(AutoStr::from("trace_id"), Value::str(&self.trace_id));
        obj.set(AutoStr::from("is_ask"), Value::Bool(self.is_ask));
        obj.set(AutoStr::from("can_reply"), Value::Bool(self.can_reply()));
        Value::Obj(obj)
    }
}

impl fmt::Display for MessageContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MessageContext(sender={:?}, trace={}, is_ask={})",
            self.sender_id, self.trace_id, self.is_ask
        )
    }
}

/// Builder for creating MessageContext instances
///
/// Useful for test code and complex context construction.
pub struct MessageContextBuilder {
    sender_id: Option<u64>,
    trace_id: String,
    is_ask: bool,
    reply_tx: Option<Sender<Value>>,
}

impl MessageContextBuilder {
    /// Create a new builder with default values
    pub fn new() -> Self {
        Self {
            sender_id: None,
            trace_id: String::new(),
            is_ask: false,
            reply_tx: None,
        }
    }

    /// Set the sender ID
    pub fn sender_id(mut self, id: u64) -> Self {
        self.sender_id = Some(id);
        self
    }

    /// Set the trace ID
    pub fn trace_id(mut self, id: impl Into<String>) -> Self {
        self.trace_id = id.into();
        self
    }

    /// Enable ask mode with a reply channel
    pub fn with_reply(mut self, tx: Sender<Value>) -> Self {
        self.is_ask = true;
        self.reply_tx = Some(tx);
        self
    }

    /// Build the MessageContext
    pub fn build(self) -> MessageContext {
        MessageContext {
            sender_id: self.sender_id,
            trace_id: self.trace_id,
            is_ask: self.is_ask,
            reply_tx: self.reply_tx,
        }
    }
}

impl Default for MessageContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    #[test]
    fn test_message_context_new() {
        let ctx = MessageContext::new(Some(1), "trace-123".to_string());
        assert_eq!(ctx.sender_id, Some(1));
        assert_eq!(ctx.trace_id, "trace-123");
        assert!(!ctx.is_ask);
        assert!(!ctx.can_reply());
    }

    #[test]
    fn test_message_context_for_ask() {
        let (tx, _rx) = mpsc::channel();
        let ctx = MessageContext::for_ask(Some(1), "trace-456".to_string(), tx);
        assert_eq!(ctx.sender_id, Some(1));
        assert_eq!(ctx.trace_id, "trace-456");
        assert!(ctx.is_ask);
        assert!(ctx.can_reply());
    }

    #[test]
    fn test_reply_success() {
        let (tx, rx) = mpsc::channel();
        let ctx = MessageContext::for_ask(Some(1), "trace-789".to_string(), tx);

        let result = ctx.reply(Value::str("response"));
        assert!(result.is_ok());

        let received = rx.try_recv();
        assert!(received.is_ok());
        assert_eq!(received.unwrap(), Value::str("response"));
    }

    #[test]
    fn test_reply_no_channel() {
        let ctx = MessageContext::new(Some(1), "trace-000".to_string());

        let result = ctx.reply(Value::str("response"));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No reply channel"));
    }

    #[test]
    fn test_can_reply() {
        let (tx, _rx) = mpsc::channel();

        let ctx_ask = MessageContext::for_ask(Some(1), "trace".to_string(), tx);
        assert!(ctx_ask.can_reply());

        let ctx_send = MessageContext::new(Some(1), "trace".to_string());
        assert!(!ctx_send.can_reply());
    }

    #[test]
    fn test_to_value() {
        let (tx, _rx) = mpsc::channel();
        let ctx = MessageContext::for_ask(Some(42), "trace-abc".to_string(), tx);

        let value = ctx.to_value();

        // Check it's an object (Obj variant)
        assert!(matches!(value, Value::Obj(_)));

        // Verify the value contains the right data by checking its string representation
        let str_repr = format!("{:?}", value);
        assert!(str_repr.contains("sender_id"));
        assert!(str_repr.contains("42"));
        assert!(str_repr.contains("trace_id"));
        assert!(str_repr.contains("trace-abc"));
        assert!(str_repr.contains("is_ask"));
        assert!(str_repr.contains("can_reply"));
    }

    #[test]
    fn test_to_value_no_sender() {
        let ctx = MessageContext::new(None, "trace".to_string());
        let value = ctx.to_value();

        assert!(matches!(value, Value::Obj(_)));

        // Verify Nil for sender_id
        let str_repr = format!("{:?}", value);
        assert!(str_repr.contains("sender_id"));
        assert!(str_repr.contains("Nil"));
    }

    #[test]
    fn test_display() {
        let ctx = MessageContext::new(Some(1), "trace-123".to_string());
        let display = format!("{}", ctx);
        assert!(display.contains("sender=Some(1)"));
        assert!(display.contains("trace=trace-123"));
        assert!(display.contains("is_ask=false"));
    }

    #[test]
    fn test_builder() {
        let (tx, _rx) = mpsc::channel();

        let ctx = MessageContextBuilder::new()
            .sender_id(100)
            .trace_id("builder-trace")
            .with_reply(tx)
            .build();

        assert_eq!(ctx.sender_id, Some(100));
        assert_eq!(ctx.trace_id, "builder-trace");
        assert!(ctx.is_ask);
        assert!(ctx.can_reply());
    }

    #[test]
    fn test_builder_default() {
        let ctx = MessageContextBuilder::default().build();
        assert_eq!(ctx.sender_id, None);
        assert_eq!(ctx.trace_id, "");
        assert!(!ctx.is_ask);
        assert!(!ctx.can_reply());
    }

    #[test]
    fn test_is_ask_method() {
        let (tx, _rx) = mpsc::channel();

        let ctx_ask = MessageContext::for_ask(None, "trace".to_string(), tx);
        assert!(ctx_ask.is_ask());

        let ctx_send = MessageContext::new(None, "trace".to_string());
        assert!(!ctx_send.is_ask());
    }

    #[test]
    fn test_sender_id_method() {
        let ctx = MessageContext::new(Some(123), "trace".to_string());
        assert_eq!(ctx.sender_id(), Some(123));
    }

    #[test]
    fn test_trace_id_method() {
        let ctx = MessageContext::new(None, "my-trace-id".to_string());
        assert_eq!(ctx.trace_id(), "my-trace-id");
    }

    #[test]
    fn test_with_reply_channel() {
        let (tx, _rx) = mpsc::channel();

        let ctx = MessageContext::with_reply_channel(
            Some(1),
            "custom-trace",
            Some(tx),
        );

        assert_eq!(ctx.sender_id, Some(1));
        assert_eq!(ctx.trace_id, "custom-trace");
        assert!(ctx.is_ask);
        assert!(ctx.can_reply());
    }

    #[test]
    fn test_with_reply_channel_none() {
        let ctx = MessageContext::with_reply_channel(
            None,
            "no-reply-trace",
            None,
        );

        assert_eq!(ctx.sender_id, None);
        assert_eq!(ctx.trace_id, "no-reply-trace");
        assert!(!ctx.is_ask);
        assert!(!ctx.can_reply());
    }
}
