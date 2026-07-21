use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Note {
    pub id: i64,
    pub title: String,
    pub body: String,
    pub time: String,
    pub pinned: bool,
    pub tags: Vec<String>,
    pub folder: String,
}
