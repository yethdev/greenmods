use anyhow::Result;
use std::process::Command;

fn get_version() -> Result<String> {
    let ver = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()?;
    let str = String::from_utf8(ver.stdout)?;

    Ok(str)
}

fn get_origin() -> Result<String> {
    let ver = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .output()?;
    let str = String::from_utf8(ver.stdout)?;

    Ok(str)
}

fn main() {
    let ver = get_version().unwrap_or("".into());
    let origin = get_origin().unwrap_or("".into());

    println!("cargo::rustc-env=__MH_GIT_COMMIT={ver}");
    println!("cargo::rustc-env=__MH_GIT_ORIGIN={origin}");
}
