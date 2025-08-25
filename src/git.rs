use std::io;
use std::process::Command;
use tracing::{info, warn, error, debug, instrument};

#[instrument]
pub fn check_git() -> Result<bool, io::Error> {
    let output = Command::new("git")
        .arg("--version")
        .output()?;

    debug!("{:#?}", output);

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.trim_start().starts_with("git version"))
    } else {
        Ok(false)
    }
}

#[instrument]
pub fn status(path: &str) -> Result<bool, io::Error> {
    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .arg("status")
        .output()?;

    debug!("{:#?}", output);

    Ok(output.status.success())
}

#[instrument]
pub fn clone(path: &str, url: &str) -> Result<bool, io::Error> {
    let output = Command::new("git")
        .arg("clone")
        .arg(url)
        .arg(path)
        .output()?;

    debug!("{:#?}", output);

    Ok(output.status.success())
}

#[instrument]
pub fn pull(path: &str, r#ref: &str) -> Result<bool, io::Error> {
    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .arg("pull")
        .arg("origin")
        .arg(r#ref)
        .output()?;

    debug!("{:#?}", output);

    Ok(output.status.success())
}


#[instrument]
pub fn verify_commit(path: &str, r#ref: &str) -> Result<bool, io::Error> {
    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .arg("verify-commit")
        .arg(r#ref)
        .output()?;

    debug!("{:#?}", output);

    Ok(output.status.success())
}

