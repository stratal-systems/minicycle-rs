use tokio::sync::Mutex;
use crate::cfg;

pub struct AppState {
    pub cfg: cfg::Cfg,
    pub busy: Mutex<bool>,
}

