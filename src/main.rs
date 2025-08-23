use warp::Filter;
use warp::reply::Json;
use std::convert::Infallible;
use warp::http::StatusCode;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use hmac::{Hmac, Mac};
use hex_literal::hex;
use std::process::exit;
use std::fs;

mod cfg;

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
        name: String,
        payload: Payload,
        signature: String,
        ) -> Result<impl warp::Reply, Infallible> {

    if !verify_hmac(payload, signature).await {
        return Ok(
            warp::reply::with_status("not allowed", warp::http::StatusCode::FORBIDDEN)
        );
    }
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

    let config = cfg::read_config();

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
                    .and_then(hook)
            )

        )
        .run(([127, 0, 0, 1], 3030))
        .await;
}

