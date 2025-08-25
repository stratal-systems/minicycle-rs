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

mod cfg;
mod appstate;
mod payload;
mod git;
use crate::payload::Payload;
use crate::appstate::AppState;
use crate::cfg::Repo;

async fn verify_hmac(payload: &Payload, signature: String) -> bool {
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

    if !verify_hmac(&payload, signature).await {
        return Ok(with_status("not allowed".into(), StatusCode::FORBIDDEN));
    }

    let Some(repo) = state.cfg.repos.get(&name)
    else {
        return Ok(with_status("repo not found".into(), StatusCode::NOT_FOUND));
    };

    info!("Bumping repo `{}`...", name.clone());

    match bump_repo(&state.cfg, name.clone(), repo, &payload).await {
        Ok(_) => {},
        Err(err) => {
            let errmsg = format!(
                "Failed to bump repo `{}`: {}",
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
        config: &cfg::Cfg,
        name: String,
        repo: &Repo,
        payload: &Payload,
    ) -> Result<(), String> {

    match git::status(repo.path.as_str()) {
        Ok(true) => { info!("repo OK"); },
        Ok(false) => {
            info!("repo not OK, trying clone");

            match git::clone(
                    repo.path.as_str(),
                    payload.repository.clone_url.as_str()
                    ) {
                Ok(true) => { info!("Clone OK"); },
                Ok(false) => {
                    return Err("Error while cloning repo.".into());
                },
                Err(err) => {
                    return Err(format!("Error while cloning repo: {}", err));
                },
            };
        },
        Err(err) => {
            return Err(format!("Error while trying to check repo: {}", err));
        },
    };

    match git::pull(repo.path.as_str(), payload.r#ref.as_str()) {
        Ok(true) => { info!("Pull OK") },
        Ok(false) => { return Err("Error pulling repo.".into()) },
        Err(err) => {
            return Err(format!("Error while cloning repo: {}", err));
        },
    };

    if config.enforce_signatures {
        match git::verify_commit(repo.path.as_str(), payload.r#ref.as_str()) {
            Ok(true) => { info!("Signature verification OK") },
            Ok(false) => { return Err("Error verifying signature.".into()) },
            Err(err) => {
                return Err(format!("Error verifying signature: {}", err));
            },
        };
    } else {
        warn!("Skipping signature verification!!!!!");
    }


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

    match git::check_git() {
        Ok(true) => { info!("Found `git` command.") },
        Ok(false) => {
            error!("Could not find a suitable `git`, aborting.");
            exit(1);
        },
        Err(err) => {
            error!("Error while checking for `git`, aborting: {}", err);
            exit(1);
        },
    }

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

