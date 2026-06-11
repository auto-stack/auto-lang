//! Bridge between legacy PipelineData and AtomPipeline
//!
//! Provides bidirectional conversion so that the shell can gradually
//! migrate from `PipelineData` to `AtomPipeline` without breaking
//! existing commands.

use ash_core::pipeline::{Atom, AtomPipeline, AtomType, convert::infer_atom_type};
use auto_val::Value;
use super::pipeline_data::PipelineData;

/// Convert legacy PipelineData → AtomPipeline
///
/// Uses `infer_atom_type()` to assign a semantic type tag when the
/// input carries a structured Value. Text and empty data map directly.
pub fn pipeline_data_to_atom(data: PipelineData) -> AtomPipeline {
    match data {
        PipelineData::Value(v) => {
            let atom_type = infer_atom_type(&v);
            AtomPipeline::from_atom(Atom::new(v, atom_type))
        }
        PipelineData::Text(s) => {
            if s.is_empty() {
                AtomPipeline::Empty
            } else {
                AtomPipeline::text(s)
            }
        }
    }
}

/// Convert AtomPipeline → legacy PipelineData
///
/// Flattens typed data back to the untyped PipelineData enum.
/// Atom/Stream → Value, Text → Text, Empty → empty Text.
pub fn atom_to_pipeline_data(atom: AtomPipeline) -> PipelineData {
    match atom {
        AtomPipeline::Atom(a) => PipelineData::Value(a.value),
        AtomPipeline::Stream(s) => {
            // Collect stream into a list Value
            let values: Vec<Value> = s.items.iter().map(|a| a.value.clone()).collect();
            PipelineData::Value(Value::Array(auto_val::Array::from(values)))
        }
        AtomPipeline::ExternalStream(es) => {
            // Read external stream output into text
            PipelineData::Text(es.read_all().unwrap_or_default())
        }
        AtomPipeline::Text(s) => PipelineData::Text(s),
        AtomPipeline::Empty => PipelineData::empty(),
    }
}

/// Convert Option<PipelineData> → Option<AtomPipeline>
pub fn opt_pipeline_to_atom(data: Option<PipelineData>) -> Option<AtomPipeline> {
    data.map(pipeline_data_to_atom)
}

/// Convert Option<AtomPipeline> → Option<PipelineData>
pub fn opt_atom_to_pipeline(data: Option<AtomPipeline>) -> Option<PipelineData> {
    data.map(atom_to_pipeline_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use auto_val::Obj;

    #[test]
    fn test_value_to_atom() {
        let pd = PipelineData::from_value(Value::Int(42));
        let atom = pipeline_data_to_atom(pd);
        assert!(atom.is_atom());
        assert_eq!(atom.atom_type(), AtomType::Nothing); // Int → Nothing
    }

    #[test]
    fn test_string_to_atom() {
        let pd = PipelineData::from_text("hello".into());
        let atom = pipeline_data_to_atom(pd);
        assert!(atom.is_text());
        assert_eq!(atom.as_text(), "hello");
    }

    #[test]
    fn test_empty_text_to_atom() {
        let pd = PipelineData::from_text("".into());
        let atom = pipeline_data_to_atom(pd);
        assert!(atom.is_empty());
    }

    #[test]
    fn test_file_list_roundtrip() {
        let mut obj = Obj::new();
        obj.set("name", Value::str("test.txt"));
        obj.set("type", Value::str("file"));
        let arr = auto_val::Array::from(vec![Value::Obj(obj)]);
        let pd = PipelineData::from_value(Value::Array(arr));

        let atom = pipeline_data_to_atom(pd);
        assert_eq!(atom.atom_type(), AtomType::FileList);

        let back = atom_to_pipeline_data(atom);
        assert!(back.is_value());
    }

    #[test]
    fn test_atom_to_pipeline_text() {
        let atom = AtomPipeline::text("world");
        let pd = atom_to_pipeline_data(atom);
        assert!(pd.is_text());
        assert_eq!(pd.into_text(), "world");
    }

    #[test]
    fn test_atom_to_pipeline_empty() {
        let atom = AtomPipeline::empty();
        let pd = atom_to_pipeline_data(atom);
        assert!(pd.is_empty());
    }

    #[test]
    fn test_opt_conversions() {
        let pd = Some(PipelineData::from_value(Value::Int(1)));
        let atom = opt_pipeline_to_atom(pd);
        assert!(atom.is_some());

        let back = opt_atom_to_pipeline(atom);
        assert!(back.is_some());
    }

    #[test]
    fn test_opt_none() {
        let pd: Option<PipelineData> = None;
        let atom = opt_pipeline_to_atom(pd);
        assert!(atom.is_none());
    }
}
