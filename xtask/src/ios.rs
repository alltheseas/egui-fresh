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
const DEFAULT_DEVICE: &str = "iPhone 16";
const BUILD_DIR: &str = "target/ios-smoke-build";
const APP_BUNDLE_ID: &str = "com.egui.RunnerSmoke";

pub(crate) fn sim_smoke() -> Result<(), DynError> {
    ensure_macos()?;

    let repo_root = workspace_root();
    prepare_ios_build(&repo_root)?;

    println!("iOS simulator smoke build finished successfully.");
    Ok(())
}

pub(crate) fn sim_launch() -> Result<(), DynError> {
    ensure_macos()?;

    let repo_root = workspace_root();
    let app_bundle = prepare_ios_build(&repo_root)?;

    let device = env::var("IOS_SMOKE_DEVICE").unwrap_or_else(|_| DEFAULT_DEVICE.to_string());

    boot_simulator(&device)?;
    install_app(&device, &app_bundle)?;
    launch_app(&device)?;

    Ok(())
}

pub(crate) fn bundle_run(args: &[&str]) -> Result<(), DynError> {
    ensure_macos()?;
    let manifest =
        env::var("IOS_BUNDLE_MANIFEST").unwrap_or_else(|_| "apps/eframe-ios-app/Cargo.toml".into());

    let mut cmd = Command::new("python3.11");
    cmd.arg("ios-cargo")
        .arg("--manifest-path")
        .arg(&manifest)
        .arg("run")
        .arg("--sim");
    cmd.args(args);
    cmd.current_dir(workspace_root());

    utils::print_cmd(&cmd);
    let status = cmd.status()?;
    if !status.success() {
        return Err("ios-cargo run failed".into());
    }
    Ok(())
}

fn workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .expect("xtask workspace layout changed")
        .to_path_buf()
}

fn ensure_macos() -> Result<(), DynError> {
    if cfg!(target_os = "macos") {
        Ok(())
    } else {
        Err("iOS smoke tests require macOS with Xcode command-line tools".into())
    }
}

fn prepare_ios_build(repo_root: &Path) -> Result<PathBuf, DynError> {
    run_build_script(repo_root)?;
    sync_runner_artifacts(repo_root)?;
    let build_dir = repo_root.join(BUILD_DIR);
    if build_dir.exists() {
        fs::remove_dir_all(&build_dir)?;
    }
    run_xcodebuild(repo_root, &build_dir)?;

    let app_path = build_dir.join("Debug-iphonesimulator/RunnerSmoke.app");
    if !app_path.exists() {
        return Err(format!("app bundle missing at {}", app_path.display()).into());
    }
    Ok(app_path)
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

fn run_xcodebuild(repo_root: &Path, build_dir: &Path) -> Result<(), DynError> {
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
        .arg(format!("BUILD_DIR={}", build_dir.display()))
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

fn boot_simulator(device: &str) -> Result<(), DynError> {
    let mut boot_cmd = simctl_cmd();
    boot_cmd.arg("boot").arg(device);
    utils::print_cmd(&boot_cmd);
    let boot_status = boot_cmd.status()?;
    if !boot_status.success() {
        eprintln!(
            "simctl boot exited with status {boot_status}. Assuming {device} is already booted."
        );
    }

    let mut wait_cmd = simctl_cmd();
    wait_cmd.arg("bootstatus").arg(device).arg("-b");
    utils::print_cmd(&wait_cmd);
    let status = wait_cmd.status()?;
    if !status.success() {
        return Err("failed to wait for simulator boot completion".into());
    }
    Ok(())
}

fn install_app(device: &str, app_bundle: &Path) -> Result<(), DynError> {
    if !app_bundle.exists() {
        return Err(format!("app bundle missing at {}", app_bundle.display()).into());
    }

    let mut cmd = simctl_cmd();
    cmd.arg("uninstall").arg(device).arg(APP_BUNDLE_ID);
    utils::print_cmd(&cmd);
    let _ = cmd.status();

    let mut install_cmd = simctl_cmd();
    install_cmd.arg("install").arg(device).arg(app_bundle);
    utils::print_cmd(&install_cmd);
    let status = install_cmd.status()?;
    if !status.success() {
        return Err("failed to install RunnerSmoke into the simulator".into());
    }
    Ok(())
}

fn launch_app(device: &str) -> Result<(), DynError> {
    println!(
        "Launching RunnerSmoke on {device}. Focus the simulator window, open an egui text field, and type to emit IME events."
    );
    println!("Press Ctrl+C when you are done capturing logs.");

    let mut cmd = simctl_cmd();
    cmd.arg("launch")
        .arg("--console")
        .arg(device)
        .arg(APP_BUNDLE_ID);
    utils::print_cmd(&cmd);
    let status = cmd.status()?;
    if !status.success() {
        return Err("failed to launch RunnerSmoke in the simulator".into());
    }
    Ok(())
}

fn simctl_cmd() -> Command {
    let mut cmd = Command::new("xcrun");
    cmd.arg("simctl");
    cmd
}
