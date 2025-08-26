use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::process::exit;

#[derive(Deserialize, Serialize, Debug)]
pub struct Repo {
    pub path: String,
    pub entrypoint: String
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Cfg {
    pub repos: HashMap<String, Repo>,

    pub hmac_key: String,

    #[serde(default = "default_enforce_signatures")]
    pub enforce_signatures: bool,

    #[serde(default = "default_report_dir")]
    pub report_dir: String,
}

// TODO is this necessary!?!?
fn default_enforce_signatures() -> bool {
    return true;
}

fn default_report_dir() -> String {
    return "./reports".into();
}

pub fn read_config() -> Cfg {
    let contents = match fs::read_to_string("minicycle.toml") {
        Ok(c) => c,
        Err(_) => {
            eprintln!("Could not read toml");
            exit(1);
        }
    };

    let cfg: Cfg = match toml::from_str(&contents) {
        Ok(d) => d,
        Err(err) => {
            eprintln!("Could not decode toml: {err}");
            exit(1);
        }
    };

    return cfg;
}

