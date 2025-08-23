use tokio::sync::Mutex;
use std::sync::Arc;
use crate::cfg;

// TODO don't cfg but share it!! It's readonly!
#[derive(Clone)]
pub struct AppState {
    pub cfg: cfg::Cfg,
    pub busy: Arc<Mutex<bool>>,
}

