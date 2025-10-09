use bytes;
use hex;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::convert::Infallible;
use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{exit, Command};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
use tracing::{info, warn, error, debug, instrument};
use tracing_subscriber;
use warp::Filter;
use warp::http::Response;
use warp::http::StatusCode;
use warp::reply::with_status;

mod cfg;
mod appstate;
mod payload;
mod git;
mod report;
mod force_symlink;
use crate::payload::Payload;
use crate::appstate::AppState;
use crate::cfg::Repo;
use crate::report::Report;

type HmacSha256 = Hmac<Sha256>;

async fn verify_hmac(
        config: &cfg::Cfg,
        payload_bytes: &bytes::Bytes,
        signature: String
        ) -> bool {

    let mut mac = HmacSha256::new_from_slice(config.hmac_key.as_bytes()).unwrap();
    mac.update(payload_bytes);
    // TODO clone??
    let result = mac.clone().finalize();

    debug!("Computed HMAC is: {:x}", result.into_bytes());

    // TODO is this constant-time comparison?
    // I think it is!
    // TODO unwraps are a mess fix it!!
    return match mac.verify_slice(&hex::decode(signature.strip_prefix("sha256=").unwrap()).unwrap()) {
        Ok(_) => true,
        _ => false,
    };
}

async fn get_latest_report(
        state: Arc<AppState>,
        ) -> Result<impl warp::Reply, Infallible> {

    let latest_path = Path::new(&state.cfg.report_dir).join("latest.json");
    // TODO copypasta!!

    return match fs::read_to_string(latest_path) {
        Ok(content) => Ok(Response::builder()
            .header("Content-Type", "application/json")
            .status(StatusCode::OK)
            .body(content)),
        Err(_) => Ok(Response::builder()
            .header("Content-Type", "text/plain")
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body("Failed to get latest report".into())),
    };
    // TODO json error response?

}

async fn hook(
        state: Arc<AppState>,
        name: String,
        //payload: Payload,
        payload_bytes: bytes::Bytes,
        signature: String,
        ) -> Result<impl warp::Reply, Infallible> {

    let Ok(_guard) = state.busy.try_lock()
    else {
        return Ok(with_status("busy!", StatusCode::SERVICE_UNAVAILABLE));
    };

    if !verify_hmac(&state.cfg, &payload_bytes, signature.clone()).await {
        return Ok(with_status("not allowed".into(), StatusCode::FORBIDDEN));
    }

    let state_cloned = state.clone();
    let signature_cloned = signature.clone();
    // FIXME this is spaghett

    tokio::spawn( async {
        run_hook(state_cloned, name, payload_bytes, signature_cloned)
    } );

    return Ok(with_status("task spawned.".into(), StatusCode::OK));
}

async fn run_hook(
        state: Arc<AppState>,
        name: String,
        //payload: Payload,
        payload_bytes: bytes::Bytes,
        signature: String,
) {

    let payload: Payload = match serde_json::from_slice(&payload_bytes) {
        Ok(p) => p,
        Err(_) => { return (); },
        //Err(_) => { return Ok(warp::reply::with_status(
        //    "Invalid JSON".into(),
        //    StatusCode::BAD_REQUEST
        //)); },
    };
    // TODO proper panic

    let Some(repo) = state.cfg.repos.get(&name)
    else {
        return ();
        //return Ok(with_status("repo not found".into(), StatusCode::NOT_FOUND));
    };


    'branch_check: {

        // TODO spaghett
        for branch in &*repo.branches {
            if payload.r#ref.ends_with(&*branch) {
                break 'branch_check;
            }
        }

        return ();
        //return Ok(with_status(
        //        "Not listening on this branch".into(),
        //        StatusCode::OK
        //        ));
    }

    info!("Bumping repo `{}`...", name.clone());

    match bump_repo(&state.cfg, repo, &payload).await {
        Ok(_) => {},
        Err(err) => {
            let errmsg = format!(
                "Failed to bump repo `{}`: {}",
                name,
                err,
                );

            error!("{}", errmsg);
            //return Ok(with_status(
            //    errmsg,
            //    StatusCode::INTERNAL_SERVER_ERROR,
            //    ));
        },

    }

    match run_entrypoint(&state.cfg, name.clone(), repo, &payload).await {
        Ok(_) => {},
        Err(err) => {
            let errmsg = format!(
                "Failed to run entrypoint for repo `{}`: {}",
                name,
                err,
                );

            error!("{}", errmsg);
            //return Ok(with_status(
            //    errmsg,
            //    StatusCode::INTERNAL_SERVER_ERROR,
            //    ));
        },

    }
}

#[instrument]
async fn run_entrypoint(
        config: &cfg::Cfg,
        name: String,
        repo: &Repo,
        payload: &Payload,
    ) -> Result<(), String> {

    let path_repo = Path::new(&repo.path);
    let path_rel = Path::new(&repo.entrypoint);
    let path_joined = path_repo.join(path_rel);

    let time_start = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();


    let report_path = Path::new(&config.report_dir).join(format!("{}.json", time_start));
    let latest_path = Path::new(&config.report_dir).join("latest.json");

    let artifacts_name: String = format!("{}-{}", name, time_start);
    let artifacts_path = env::current_dir().unwrap().join(&config.artifact_dir).join(&artifacts_name);
    let output_path = artifacts_path.join("output");
    fs::create_dir_all(&artifacts_path).unwrap();

    let mut report = report::Report {
        artifacts: artifacts_name.clone(),
        message: payload.head_commit.message.clone(),
        r#ref: payload.r#ref.clone(),
        start: report::Start {
            time: time_start,
        },
        finish: None
    };
    // TODO into_os_string? encoding issues??

    let report_str = serde_json::to_string(&report).unwrap();
    let mut file = fs::File::create(&report_path).unwrap();
    write!(file, "{}", report_str).unwrap();
    // TODO very ugly code like `report_path` being re-created every time
    // FIX MEE!!!
    force_symlink::force_symlink(format!("{}.json", time_start), &latest_path).unwrap();

    let output_file = fs::File::create(&output_path).unwrap();
    let output_file_clone = output_file.try_clone().unwrap();

    let output = match Command::new(&path_rel)
            .env("MINICYCLE_ARTIFACTS", artifacts_path.into_os_string().into_string().unwrap())  // TODO copypasta
            .current_dir(path_repo)
            .stderr(output_file)
            .stdout(output_file_clone)
            .output()
            {
        Err(err) => { return Err(format!("{}", err)); },
        Ok(output) => output,
    };

    debug!("{:#?}", output);

    let time_finish = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    report.finish = Some(report::Finish {
        time: time_finish,
        ok: output.status.success(),
    });

    let report_str = serde_json::to_string(&report).unwrap();
    let mut file = fs::File::create(&report_path).unwrap();
    write!(file, "{}", report_str).unwrap();
    force_symlink::force_symlink(format!("{}.json", time_start), &latest_path).unwrap();

    if output.status.success() {
        info!("entrypoint executed successfully");
        return Ok(());
    } else {
        error!("entrypoint exited with status: {}", output.status);
        return Err(format!("entrypoint exited with status: {}", output.status));
        // TODO
    }
}

async fn bump_repo(
        config: &cfg::Cfg,
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

    match git::fetch_and_checkout(repo.path.as_str(), payload.r#ref.as_str()) {
        Ok(true) => { info!("checkout OK") },
        Ok(false) => { return Err("Error checking out repo.".into()) },
        Err(err) => {
            return Err(format!("Error while checking out repo: {}", err));
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
    if (
        env!("CARGO_PKG_LICENSE").is_empty()
        ||
        env!("CARGO_PKG_REPOSITORY").is_empty()
    ) {
        return Ok(
            format!(
                "Hello! I am {} version {}. The person who compiled \
                me is violating the terms of the GNU Affero General Public \
                License by hiding my source code from you!!",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION"),
            ).to_string()
        );
    }
    return Ok(
        format!(
            "Hello! I am {} version {}. I am licensed under {}, \
            and my source code is at {}.",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
            env!("CARGO_PKG_LICENSE"),
            env!("CARGO_PKG_REPOSITORY"),
        ).to_string()
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

    // TODO also check GPG!!!!
    
    info!("creating report dir at {}", state.cfg.report_dir);
    fs::create_dir_all(state.cfg.report_dir.clone()).unwrap();

    let state_ptr = Arc::new(state);
    let foo = state_ptr.clone();
    let bar = state_ptr.clone();
    let baz = state_ptr.clone();
    // TODO fix thiiiss!!

    warp::serve(
        warp::path::end()
            .and_then(hello)
            .or(
                warp::path!("hook" / String)
                    .and(warp::body::content_length_limit(1024 * 2048))
                    .and(warp::post())
                    .and(warp::body::bytes())
                    .and(warp::header::<String>("X-Hub-Signature-256"))
                    .and_then(
                        move |name: String, payload_bytes: bytes::Bytes, signature: String| {
                            hook(foo.clone(), name, payload_bytes, signature)
                        }
                    //.and_then(
                    //    move |name: String, payload: Payload, signature: String| {
                    //        hook(state_ptr.clone(), name, payload, signature)
                    //    }
                    )
            )
            .or(
                warp::path("report-latest")
                    .and(warp::get())
                    .and(warp::path::end())
                    .and_then(
                        move || {
                            get_latest_report(bar.clone())
                        }
                    )
            )
            .or(
                warp::path("artifacts")
                .and(warp::fs::dir(baz.cfg.artifact_dir.clone()))
            )

        )
        //.run(([127, 0, 0, 1], 3030))
        .run(([0, 0, 0, 0], 3030))
        .await;
}

