# iOS IME validation checklist

Until we have automated simulator coverage, follow the steps below whenever we need to confirm that
`WindowEvent::Ime` events still reach egui on iOS.

## 0. Prerequisites

1. Install Python 3.11 or newer (`python3 --version`).
2. Install `cargo-bundle` (currently requires nightly):
   ```bash
   rustup install nightly
   cargo +nightly install cargo-bundle
   ```
3. Install the required Rust targets:
   ```bash
   rustup target add aarch64-apple-ios
   rustup target add aarch64-apple-ios-sim
   ```
4. Optional: export `CARGO_TARGET_DIR` if you use a custom target directory.

## 1. Automated smoke test (`ios-cargo`)

We ship a small binary crate (`apps/eframe-ios-app`) that launches `egui_demo_app`. Build, bundle,
install, and launch it in the simulator with:

```bash
./ios-cargo run --sim --manifest-path apps/eframe-ios-app/Cargo.toml
```

What happens:

1. `cargo bundle --target aarch64-apple-ios-sim --manifest-path apps/eframe-ios-app/Cargo.toml`
   produces `target/aarch64-apple-ios-sim/debug/bundle/ios/egui-ios-demo.app`.
2. The script installs the bundle into the booted simulator (or boots the newest iPhone if none are running).
3. The app is launched via `xcrun simctl launch --console …`, so stdout/stderr (including
   `[egui-ios-ime] …` logs) stream to your terminal.

Tips:

- Pass `--device <udid>` to target a specific simulator.
- Use `--release` (and/or `--ipad`) when you need IPA archives.

## 2. IME validation steps

1. Once the simulator displays the egui demo, open “Widgets → Text Edit”.
2. Tap inside both the multi-line and single-line text fields. The software keyboard must appear.
3. Type a short phrase. Characters must show up immediately in egui **and** the terminal that ran
   `ios-cargo` should log entries such as:
   ```
   [egui-ios-ime] WindowEvent::Ime: Commit("hello")
   ```
4. If the keyboard does not appear or the logs stay silent:
   - Toggle “I/O → Keyboard → Toggle Software Keyboard” (⇧⌘K) in the simulator.
   - Capture `xcrun simctl spawn <udid> log show --style syslog --last 2m --predicate 'process == "egui-ios-demo"'`
     and attach it to the issue you file.

## 3. Manual fallback (deprecated harness)

The previous Swift/Xcode harness still lives in `ios/runner-smoke/RunnerSmoke.xcodeproj` for debugging
custom UIKit code. Keep it only if you need to inspect Swift-side behaviour manually; the
cargo-bundle flow above is now the canonical way to package + validate egui on iOS. When using the old
harness, remember to run `scripts/build_ios_runner.sh`, copy the resulting xcframework/header into
`Frameworks/` and `Generated/`, then drive it via `xcodebuild` or Xcode UI.
