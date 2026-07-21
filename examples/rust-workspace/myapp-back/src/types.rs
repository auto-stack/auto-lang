use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Status {
    pub ok: bool,
    pub message: String,
}
