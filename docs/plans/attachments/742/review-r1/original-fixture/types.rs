use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct NoteDocument {
    pub schema_version: i64,
    pub note_id: String,
    pub title: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct NoteView {
    pub schema_version: i64,
    pub note_id: String,
    pub title: String,
}
