use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::process::exit;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use warp::Filter;
use warp::http::StatusCode;
use warp::reply::with_status;
use tracing::{info, warn, error};
use tracing_subscriber;
use git2::Repository;

mod cfg;
mod appstate;
use crate::appstate::AppState;
use crate::cfg::Repo;

#[derive(Deserialize, Serialize)]
struct Payload {
    // "ref" is a keyword so need to escape it!
    r#ref: String
    // TODO what data do we need??
}


async fn verify_hmac(payload: Payload, signature: String) -> bool {
    if signature.starts_with("sha256=") {
        // yeah looks good enough
        return true;
    }
    return false;
}

async fn hook(
        state: Arc<AppState>,
        name: String,
        payload: Payload,
        signature: String,
        ) -> Result<impl warp::Reply, Infallible> {

    let Ok(_guard) = state.busy.try_lock()
    else {
        return Ok(with_status("busy!".into(), StatusCode::SERVICE_UNAVAILABLE));
    };

    if !verify_hmac(payload, signature).await {
        return Ok(with_status("not allowed".into(), StatusCode::FORBIDDEN));
    }

    let Some(repo) = state.cfg.repos.get(&name)
    else {
        return Ok(with_status("repo not found".into(), StatusCode::NOT_FOUND));
    };

    info!("Bumping repo `{}`...", name.clone());

    match bump_repo(name.clone(), repo).await {
        Ok(_) => {},
        Err(err) => {
            let errmsg = format!(
                "Failed to bump repo `{}`! Git error: {}",
                name,
                err,
                );

            error!("{}", errmsg);
            return Ok(with_status(
                errmsg,
                StatusCode::INTERNAL_SERVER_ERROR,
                ));
        },

    }

    return Ok(with_status("allowed".into(), StatusCode::OK));
}

async fn bump_repo(
        name: String,
        repo: &Repo,
    ) -> Result<(), git2::Error> {

    let git = Repository::open(repo.path.clone())?;
    // TODO what is the git2 error type??

    info!("Opened repo!");

    return Ok(());

}


async fn hello() -> Result<String, warp::Rejection> {
    return Ok(
        "hello".to_string()
    );
}


#[tokio::main]
async fn main() {

    // Incantation to get logging to work
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    info!("Starting minicycle-rs!!");

    let state = AppState {
        cfg: cfg::read_config(),
        busy: Mutex::new(()),
    };

    let state_ptr = Arc::new(state);

    warp::serve(
        warp::path("hello")
            .and(warp::path::end())
            .and(warp::post())
            .and_then(hello)
            .or(
                warp::path!("hook" / String)
                    .and(warp::post())
                    .and(warp::body::json())
                    .and(warp::header::<String>("X-Hub-Signature-256"))
                    .and_then(
                        move |name: String, payload: Payload, signature: String| {
                            hook(state_ptr.clone(), name, payload, signature)
                        }
                    )
            )

        )
        .run(([127, 0, 0, 1], 3030))
        .await;
}

