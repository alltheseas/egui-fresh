import SwiftUI
import Darwin

@main
struct RunnerSmokeApp: App {
    init() {
        setenv("EGUI_IOS_LOG_IME", "1", 1)
        eframe_ios_run_demo()
    }

    var body: some Scene {
        WindowGroup {
            Color.clear
        }
    }
}
