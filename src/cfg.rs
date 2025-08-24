use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::process::exit;

#[derive(Deserialize, Serialize, Debug)]
pub struct Repo {
    pub name: String,
    pub path: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Cfg {
    pub repos: HashMap<String, Repo>
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

