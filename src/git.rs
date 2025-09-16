use std::io;
use std::process::Command;
use tracing::{warn, debug, instrument};

// TODO very confusing error reporting/handling here,
// think about it for a long time and fix it!!!!

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
pub fn fetch_and_checkout(path: &str, r#ref: &str) -> Result<bool, io::Error> {
    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .arg("fetch")
        .arg("origin")
        .arg(r#ref)
        .output()?;

    debug!("{:#?}", output);

    if !output.status.success() {
        return Ok(false);
    }

    // TODO will break on slash in branch name??
    // TODO spaghett!!
    let mut parts = r#ref.split('/');
    parts.next().unwrap();
    parts.next().unwrap();
    let branch_name = parts.next().unwrap();

    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .arg("checkout")
        .arg(branch_name)
        .output()?;

    debug!("{:#?}", output);

    if !output.status.success() {
        return Ok(false);
    }

    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .arg("pull")
        .output()?;

    debug!("{:#?}", output);

    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .arg("restore")
        .arg(".")
        .output()?;

    debug!("{:#?}", output);

    if !output.status.success() {
        return Ok(false);
    }

    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .arg("reset")
        .arg("--hard")
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

