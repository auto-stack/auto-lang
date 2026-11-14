//! PLAN-967 T-08 (AC-06): the neutral ASYNC COMPLETION PORT contract.
//!
//! The package delivers registration/completion/cancellation PORTS plus a
//! controlled real provider — the language-side `await` that consumes
//! them belongs to PLAN-968; 967 never depends on 968. The port model:
//!
//! * a port registers at the CURRENT session generation and pins it;
//! * a completion publishes ONLY when the port is open (registered, not
//!   cancelled) AND its pinned generation is still current — a late
//!   completion (session moved on / teardown) is DROPPED, never
//!   published and never touches the closed session;
//! * a cancel marks the port cancelled; a completion racing in after the
//!   cancel is dropped the same way; the provider callback stays pinned
//!   until the port settles (the host releases it last);
//! * every state change is an observable [`PortEvent`] — the consumer
//!   (968's await) judges from events, never from absent values.
//!
//! The contract is backend-free and versioned by [`PORT_FORMAT`]; the
//! host-side registry lives in `ace-av-core` (`foreign_exec`), the
//! controlled real provider in `ace-abi`'s fixtures.

/// The completion-port contract identity (version = the string).
pub const PORT_FORMAT: &str = "autoabi.contract.completion-port/1";

/// The result lane a completion may carry (the F0-shaped value lanes;
/// wider lanes join with the resource domain, never by silent widening).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortValue {
    I32(i32),
    /// A structured failure: a named negative status (the error-model
    /// table stays the naming authority).
    Status(i32),
}

impl PortValue {
    pub fn spell(&self) -> String {
        match self {
            PortValue::I32(v) => format!("i32:{v}"),
            PortValue::Status(s) => format!("status:{s}"),
        }
    }
}

/// One observable port state change, in causal order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PortEvent {
    /// `register(port, symbol)` at the pinned generation.
    Registered { port: u64, symbol: String },
    /// A completion PUBLISHED (open port, current generation).
    Published { port: u64, value: PortValue },
    /// `cancel(port)` — the port refuses every later completion.
    Cancelled { port: u64 },
    /// A completion that arrived but was dropped: the port was cancelled
    /// or the session generation moved (teardown/rebind). The value is
    /// carried for observability, NEVER published.
    LateDropped { port: u64, value: PortValue },
    /// The pinned provider callback released (the port settled; this is
    /// the "原方 destroy 一次" discipline at the port granularity).
    Released { port: u64 },
}

/// The frozen port-lifecycle rule table: given (registered?, cancelled?,
/// pinned_generation == current), what a completion does. Single source
/// for the host registry, the contract tests, and 968's consumer.
pub fn completion_disposition(
    registered: bool,
    cancelled: bool,
    pinned_generation: u64,
    current_generation: u64,
) -> CompletionDisposition {
    match (registered, cancelled) {
        (false, _) => CompletionDisposition::UnknownPort,
        (true, false) if pinned_generation == current_generation => CompletionDisposition::Publish,
        (true, _) => CompletionDisposition::DropLate,
    }
}

/// What the host does with an arriving completion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompletionDisposition {
    /// Publish the value (open port, current generation).
    Publish,
    /// Drop it — cancelled or stale generation; never publish.
    DropLate,
    /// The port id was never registered (named rejection, not a drop).
    UnknownPort,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_frozen_rule_table_decides_every_shape() {
        // open + current -> publish
        assert_eq!(
            completion_disposition(true, false, 3, 3),
            CompletionDisposition::Publish
        );
        // cancelled -> drop
        assert_eq!(
            completion_disposition(true, true, 3, 3),
            CompletionDisposition::DropLate
        );
        // stale generation (teardown/rebind moved on) -> drop
        assert_eq!(
            completion_disposition(true, false, 3, 4),
            CompletionDisposition::DropLate
        );
        // never registered -> named unknown (no silent drop)
        assert_eq!(
            completion_disposition(false, false, 3, 3),
            CompletionDisposition::UnknownPort
        );
    }

    #[test]
    fn value_lanes_spell_stably() {
        assert_eq!(PortValue::I32(9).spell(), "i32:9");
        assert_eq!(PortValue::Status(-1).spell(), "status:-1");
        assert_ne!(PortValue::I32(0), PortValue::Status(0));
    }
}
