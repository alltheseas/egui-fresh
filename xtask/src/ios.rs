use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

use crate::{DynError, utils};

const SCRIPT_PATH: &str = "scripts/build_ios_runner.sh";
const XCFRAMEWORK_SRC: &str = "target/ios/EframeIos.xcframework";
const HEADER_SRC: &str = "target/ios/eframe_ios_runner.h";
const HOST_PROJECT: &str = "ios/runner-smoke/RunnerSmoke.xcodeproj";
const HOST_SCHEME: &str = "RunnerSmoke";
const HOST_FRAMEWORK_DST: &str = "ios/runner-smoke/Frameworks/EframeIos.xcframework";
const HOST_HEADER_DST: &str = "ios/runner-smoke/Generated/eframe_ios_runner.h";
const DEFAULT_DESTINATION: &str = "platform=iOS Simulator,name=iPhone 16";

pub(crate) fn sim_smoke() -> Result<(), DynError> {
    ensure_macos()?;

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .expect("xtask workspace layout changed");

    run_build_script(&repo_root)?;
    sync_runner_artifacts(&repo_root)?;
    run_xcodebuild(&repo_root)?;

    println!("iOS simulator smoke build finished successfully.");
    Ok(())
}

fn ensure_macos() -> Result<(), DynError> {
    if cfg!(target_os = "macos") {
        Ok(())
    } else {
        Err("iOS smoke tests require macOS with Xcode command-line tools".into())
    }
}

fn run_build_script(repo_root: &Path) -> Result<(), DynError> {
    let script = repo_root.join(SCRIPT_PATH);
    if !script.exists() {
        return Err(format!("missing helper script at {}", script.display()).into());
    }

    let mut cmd = Command::new("bash");
    cmd.arg(script);
    cmd.current_dir(repo_root);

    utils::print_cmd(&cmd);
    let status = cmd.status()?;
    if !status.success() {
        return Err("failed to build eframe-ios-runner artifacts".into());
    }
    Ok(())
}

fn sync_runner_artifacts(repo_root: &Path) -> Result<(), DynError> {
    let xcframework_src = repo_root.join(XCFRAMEWORK_SRC);
    let header_src = repo_root.join(HEADER_SRC);
    if !xcframework_src.exists() || !header_src.exists() {
        return Err("run ios runner build script first to generate artifacts".into());
    }

    let framework_dst = repo_root.join(HOST_FRAMEWORK_DST);
    let header_dst = repo_root.join(HOST_HEADER_DST);

    if framework_dst.exists() {
        fs::remove_dir_all(&framework_dst)?;
    }
    if let Some(parent) = framework_dst.parent() {
        fs::create_dir_all(parent)?;
    }
    copy_dir(&xcframework_src, &framework_dst)?;

    if let Some(parent) = header_dst.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(&header_src, &header_dst)?;

    Ok(())
}

fn run_xcodebuild(repo_root: &Path) -> Result<(), DynError> {
    let project = repo_root.join(HOST_PROJECT);
    if !project.exists() {
        return Err(format!("host Xcode project missing at {}", project.display()).into());
    }

    let mut cmd = Command::new("xcodebuild");
    cmd.arg("-project")
        .arg(project)
        .arg("-scheme")
        .arg(HOST_SCHEME)
        .arg("-configuration")
        .arg("Debug")
        .arg("-destination")
        .arg(env::var("IOS_SMOKE_DESTINATION").unwrap_or_else(|_| DEFAULT_DESTINATION.to_string()))
        .arg("BUILD_DIR=target/ios-smoke-build")
        .current_dir(repo_root);

    utils::print_cmd(&cmd);
    let status = cmd.status()?;
    if !status.success() {
        return Err("xcodebuild failed for RunnerSmoke host app".into());
    }
    Ok(())
}

fn copy_dir(src: &Path, dst: &Path) -> Result<(), DynError> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let entry_path = entry.path();
        let target_path = dst.join(entry.file_name());
        if entry_path.is_dir() {
            copy_dir(&entry_path, &target_path)?;
        } else {
            fs::copy(&entry_path, &target_path)?;
        }
    }
    Ok(())
}
