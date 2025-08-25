use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Report {
    pub time: u64,
    pub ok: bool,
    pub message: String,
}

