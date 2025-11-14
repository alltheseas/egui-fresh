use eframe::NativeOptions;

fn main() -> eframe::Result {
    #[cfg(not(target_os = "android"))]
    {
        let _ = env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Warn)
            .try_init();
    }

    #[cfg(target_os = "ios")]
    unsafe {
        std::env::set_var("EGUI_IOS_LOG_IME", "1");
    }

    let mut native_options = NativeOptions::default();
    #[cfg(target_os = "ios")]
    {
        native_options.run_and_return = false;
    }
    eframe::run_native(
        "egui iOS demo",
        native_options,
        Box::new(|cc| Ok(Box::new(egui_demo_app::WrapApp::new(cc)))),
    )
}
