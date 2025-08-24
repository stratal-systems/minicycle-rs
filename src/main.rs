use warp::Filter;
use std::process::exit;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};
use tokio::time::{sleep, Duration};

mod cfg;
mod appstate;
use crate::appstate::AppState;

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

    let mut busy = state.busy.lock().await;
    if *busy {
        return Ok(
            warp::reply::with_status("busy!", warp::http::StatusCode::SERVICE_UNAVAILABLE)
        );
    }
    *busy = true;
    drop(busy);

    if !verify_hmac(payload, signature).await {

        let mut busy = state.busy.lock().await;
        if !*busy { eprintln!("Huh!!??"); exit(1); }
        *busy = false;
        drop(busy);

        return Ok(
            warp::reply::with_status("not allowed", warp::http::StatusCode::FORBIDDEN)
        );
    }

    sleep(Duration::from_millis(5000)).await;

    let mut busy = state.busy.lock().await;
    if !*busy { eprintln!("Huh!!??"); exit(1); }
    *busy = false;
    drop(busy);

    return Ok(
        warp::reply::with_status("allowed", warp::http::StatusCode::OK)
    );
}


async fn hello() -> Result<String, warp::Rejection> {
    return Ok(
        "hello".to_string()
    );
}


#[tokio::main]
async fn main() {

    let state = AppState {
        cfg: cfg::read_config(),
        busy: Mutex::new(false),
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

