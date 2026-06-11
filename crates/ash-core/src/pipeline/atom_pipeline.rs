//! AtomPipeline — the typed pipeline data enum
//!
//! Replaces `PipelineData(Value|Text)` with a richer enum that carries
//! semantic type information. Every variant preserves the ability to
//! convert back to plain text for display or legacy compatibility.

use auto_val::Value;
use super::atom::{Atom, AtomType};
use super::atom_stream::AtomStream;

/// Typed pipeline data flowing between shell commands.
///
/// | Variant | Purpose |
/// |---------|---------|
/// | `Atom` | Single typed value (most common) |
/// | `Stream` | Lazy iteration for large datasets |
/// | `Text` | Plain text (external commands, legacy) |
/// | `Empty` | No output (side-effect commands) |
#[derive(Debug, Clone)]
pub enum AtomPipeline {
    /// Single typed value
    Atom(Atom),
    /// Lazy stream of typed values
    Stream(AtomStream),
    /// Plain text (external commands, legacy compatibility)
    Text(String),
    /// No data
    Empty,
}

impl AtomPipeline {
    // ── Constructors ─────────────────────────────────────

    /// Create an AtomPipeline from a Value and explicit type tag.
    pub fn atom(value: Value, atom_type: AtomType) -> Self {
        AtomPipeline::Atom(Atom::new(value, atom_type))
    }

    /// Create a FileList pipeline.
    pub fn file_list(value: Value) -> Self {
        AtomPipeline::Atom(Atom::file_list(value))
    }

    /// Create a ProcessList pipeline.
    pub fn process_list(value: Value) -> Self {
        AtomPipeline::Atom(Atom::process_list(value))
    }

    /// Create a plain-text pipeline.
    pub fn text(s: impl Into<String>) -> Self {
        AtomPipeline::Text(s.into())
    }

    /// Create an empty pipeline (no data).
    pub fn empty() -> Self {
        AtomPipeline::Empty
    }

    /// Create from an existing Atom.
    pub fn from_atom(atom: Atom) -> Self {
        AtomPipeline::Atom(atom)
    }

    /// Create from a stream of Atoms.
    pub fn from_stream(stream: AtomStream) -> Self {
        AtomPipeline::Stream(stream)
    }

    // ── Query methods ────────────────────────────────────

    /// Get a reference to the inner Atom, if this is the Atom variant.
    pub fn as_atom(&self) -> Option<&Atom> {
        match self {
            AtomPipeline::Atom(a) => Some(a),
            _ => None,
        }
    }

    /// Get the type tag (Nothing for Text/Empty/Stream).
    pub fn atom_type(&self) -> AtomType {
        match self {
            AtomPipeline::Atom(a) => a.atom_type(),
            AtomPipeline::Stream(_) => AtomType::Nothing, // streams don't have a single type
            AtomPipeline::Text(_) => AtomType::Text,
            AtomPipeline::Empty => AtomType::Nothing,
        }
    }

    /// Check if this is an Atom variant.
    pub fn is_atom(&self) -> bool {
        matches!(self, AtomPipeline::Atom(_))
    }

    /// Check if this is a Stream variant.
    pub fn is_stream(&self) -> bool {
        matches!(self, AtomPipeline::Stream(_))
    }

    /// Check if this is plain text.
    pub fn is_text(&self) -> bool {
        matches!(self, AtomPipeline::Text(_))
    }

    /// Check if this is empty.
    pub fn is_empty(&self) -> bool {
        match self {
            AtomPipeline::Empty => true,
            AtomPipeline::Atom(a) => a.is_empty(),
            AtomPipeline::Text(s) => s.is_empty(),
            AtomPipeline::Stream(s) => s.total_count() == 0,
        }
    }

    /// Check if this carries structured (typed) data.
    pub fn is_structured(&self) -> bool {
        match self {
            AtomPipeline::Atom(a) => a.is_structured(),
            AtomPipeline::Stream(_) => true,
            _ => false,
        }
    }

    // ── Conversion ───────────────────────────────────────

    /// Convert to display text (consumes self).
    pub fn into_text(self) -> String {
        match self {
            AtomPipeline::Atom(a) => a.into_text(),
            AtomPipeline::Stream(mut s) => {
                let items: Vec<String> = s.collect_remaining().iter().map(|a| a.as_text()).collect();
                items.join("\n")
            }
            AtomPipeline::Text(s) => s,
            AtomPipeline::Empty => String::new(),
        }
    }

    /// Get display text without consuming.
    pub fn as_text(&self) -> String {
        match self {
            AtomPipeline::Atom(a) => a.as_text(),
            AtomPipeline::Stream(s) => {
                let items: Vec<String> = s.items.iter().map(|a: &Atom| a.as_text()).collect();
                items.join("\n")
            }
            AtomPipeline::Text(s) => s.clone(),
            AtomPipeline::Empty => String::new(),
        }
    }

    /// Extract the inner Value, if this is an Atom variant.
    pub fn into_value(self) -> Option<Value> {
        match self {
            AtomPipeline::Atom(a) => Some(a.value),
            AtomPipeline::Stream(s) => Some(s.into_atom_list().value),
            AtomPipeline::Text(s) => Some(Value::str(&s)),
            AtomPipeline::Empty => None,
        }
    }

    /// Collect a Stream variant into an Atom (no-op for other variants).
    pub fn collect_stream(self) -> Self {
        match self {
            AtomPipeline::Stream(s) => AtomPipeline::Atom(s.into_atom_list()),
            other => other,
        }
    }

    // ── Batom binary serialization ─────────────────────

    /// Serialize this pipeline to Batom binary format.
    pub fn to_batom(&self) -> Result<Vec<u8>, super::batom::BatomError> {
        super::batom::encode_pipeline(self)
    }

    /// Deserialize a Batom binary blob into an AtomPipeline.
    pub fn from_batom(data: &[u8]) -> Result<Self, super::batom::BatomError> {
        super::batom::decode_pipeline(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atom_pipeline_atom() {
        let p = AtomPipeline::atom(Value::Int(42), AtomType::CountResult);
        assert!(p.is_atom());
        assert!(p.is_structured());
        assert!(!p.is_empty());
        assert_eq!(p.atom_type(), AtomType::CountResult);
    }

    #[test]
    fn test_atom_pipeline_text() {
        let p = AtomPipeline::text("hello");
        assert!(p.is_text());
        assert!(!p.is_structured());
        assert_eq!(p.as_text(), "hello");
    }

    #[test]
    fn test_atom_pipeline_empty() {
        let p = AtomPipeline::empty();
        assert!(p.is_empty());
        assert_eq!(p.as_text(), "");
    }

    #[test]
    fn test_atom_pipeline_file_list() {
        let p = AtomPipeline::file_list(Value::Void);
        assert_eq!(p.atom_type(), AtomType::FileList);
        assert!(p.is_structured());
    }

    #[test]
    fn test_atom_pipeline_from_atom() {
        let atom = Atom::path("/tmp");
        let p = AtomPipeline::from_atom(atom);
        assert_eq!(p.atom_type(), AtomType::Path);
    }

    #[test]
    fn test_atom_pipeline_into_text() {
        let p = AtomPipeline::text("world");
        assert_eq!(p.into_text(), "world");
    }

    #[test]
    fn test_atom_pipeline_into_value() {
        let p = AtomPipeline::atom(Value::Int(99), AtomType::Nothing);
        let v = p.into_value();
        assert!(v.is_some());
    }

    #[test]
    fn test_atom_pipeline_empty_into_value() {
        let p = AtomPipeline::empty();
        assert!(p.into_value().is_none());
    }
}
