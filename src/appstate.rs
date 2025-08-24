use tokio::sync::Mutex;
use std::sync::Arc;
use crate::cfg;

pub struct AppState {
    pub cfg: cfg::Cfg,
    pub busy: Mutex<bool>,
}

