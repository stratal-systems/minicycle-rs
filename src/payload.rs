use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Payload {
    // "ref" is a keyword so need to escape it!
    pub r#ref: String,
    pub repository: Repository,
}

#[derive(Deserialize, Serialize)]
pub struct Repository {
    pub clone_url: String,
}

