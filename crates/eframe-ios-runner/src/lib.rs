//! iOS helper entry points for running `eframe` apps from Swift/Objective‑C.
//!
//! The initial API intentionally keeps the surface area tiny: a single
//! demo launcher that hosts `egui_demo_app`. This gives iOS experimenters a
//! known-good target before wiring up their own applications.

#![cfg_attr(not(target_os = "ios"), allow(dead_code))]

use eframe::{NativeOptions, egui};

/// Run the bundled `egui_demo_app` inside an iOS process.
///
/// Exported with C ABI so Swift / Objective‑C can call it directly once the
/// crate is compiled into an `.a` or `.xcframework`.
#[no_mangle]
pub extern "C" fn eframe_ios_run_demo() {
    #[cfg(target_os = "ios")]
    {
        run_demo_app();
    }

    #[cfg(not(target_os = "ios"))]
    {
        // Intentionally log instead of panicking so this helper can still be
        // referenced from other targets during development.
        log::warn!("eframe_ios_run_demo() called on non-iOS target; no-op for safety");
    }
}

#[cfg(target_os = "ios")]
fn run_demo_app() {
    // Keep the window borderless and maximized; UIKit owns the frame.
    let native_options = NativeOptions {
        viewport: egui::ViewportBuilder::default().with_fullscreen(true),
        ..Default::default()
    };

    if let Err(err) = eframe::run_native(
        "egui_ios_demo",
        native_options,
        Box::new(|cc| Ok(Box::new(egui_demo_app::DemoApp::new(cc)))),
    ) {
        log::error!("eframe_ios_run_demo failed: {err}");
    }
}
