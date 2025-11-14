use std::{env, path::PathBuf, process::Command};

use crate::DynError;

const DEFAULT_MANIFEST: &str = "apps/eframe-ios-app/Cargo.toml";

pub(crate) fn bundle_run(extra_args: &[&str]) -> Result<(), DynError> {
    ensure_macos()?;
    let manifest = env::var("IOS_BUNDLE_MANIFEST").unwrap_or_else(|_| DEFAULT_MANIFEST.into());

    let mut cmd = Command::new("python3.11");
    cmd.arg("ios-cargo")
        .arg("--manifest-path")
        .arg(&manifest)
        .arg("run")
        .arg("--sim");
    cmd.args(extra_args);
    cmd.current_dir(workspace_root());

    print_cmd(&cmd);
    let status = cmd.status()?;
    if !status.success() {
        return Err("ios-cargo run failed".into());
    }
    Ok(())
}

fn ensure_macos() -> Result<(), DynError> {
    if cfg!(target_os = "macos") {
        Ok(())
    } else {
        Err("iOS bundle commands require macOS with Xcode tools installed".into())
    }
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask workspace layout changed")
        .to_path_buf()
}

fn print_cmd(cmd: &Command) {
    print!("{} ", cmd.get_program().to_string_lossy());
    for arg in cmd.get_args() {
        print!("{} ", arg.to_string_lossy());
    }
    println!();
}
