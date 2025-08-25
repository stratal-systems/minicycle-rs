use std::process::Command;
use std::io;

pub fn check_git() -> Result<bool, io::Error> {
    let output = Command::new("git")
        .arg("--version")
        .output()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.trim_start().starts_with("git version"))
    } else {
        Ok(false)
    }
}

pub fn status(path: &str) -> Result<bool, io::Error> {
    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .arg("status")
        .output()?;

    Ok(output.status.success())
}

pub fn clone(path: &str, url: &str) -> Result<bool, io::Error> {
    let output = Command::new("git")
        .arg("clone")
        .arg(url)
        .arg(path)
        .output()?;

    Ok(output.status.success())
}

