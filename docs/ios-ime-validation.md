# iOS IME validation checklist

Until we have automated simulator coverage, run the following steps manually
whenever we need to confirm that winit/egui still receive `WindowEvent::Ime`
events on iOS.

## 0. Automated smoke test

Run the headless validation command:

```
cargo xtask ios-sim-smoke
```

What this does:

1. Executes `scripts/build_ios_runner.sh` to compile `eframe-ios-runner` for the device + simulator targets.
2. Copies the generated `EframeIos.xcframework` and `eframe_ios_runner.h` into the SwiftUI harness under `examples/ios/runner-smoke/`.
3. Invokes `xcodebuild` against the shared `RunnerSmoke` scheme for the iOS simulator destination (defaults to `platform=iOS Simulator,name=iPhone 16`; override with `IOS_SMOKE_DESTINATION`).

The command succeeds only if both the Rust build and the Swift host build succeed, so it is safe to wire up in CI. Use the manual steps below only when you need to inspect or tweak the harness directly.

## 1. Build the runner

```
./scripts/build_ios_runner.sh
```

This produces:

* `target/ios/EframeIos.xcframework`
* `target/ios/eframe_ios_runner.h`

Both artefacts are required for the Xcode project in the next step.

## 2. Create or reuse a SwiftUI shell

The repository already contains a ready-to-run harness under
`ios/runner-smoke/RunnerSmoke.xcodeproj`. Open it in Xcode after
running the smoke command if you want to iterate interactively—the project
expects the xcframework + header to live under `Frameworks/` and `Generated/`,
which the xtask copies automatically.

If you prefer to wire up your own host from scratch, follow these steps:

1. In Xcode, create a new *App* project.
2. Add `target/ios/EframeIos.xcframework` to the project (drag it into the
   Project navigator, select “Copy items if needed”).
3. Add a bridging header (e.g. `EguiImeTest-Bridging-Header.h`) containing
   `#include "eframe_ios_runner.h"`.
4. Update build settings:
   * `Framework Search Paths`: add the folder containing the xcframework (the
     harness uses `$(PROJECT_DIR)/Frameworks`).
   * `Header Search Paths`: add the folder containing the copied header (the
     harness uses `$(PROJECT_DIR)/Generated`).
   * `Objective-C Bridging Header`: point to the header you just created.
   * Ensure the xcframework is set to “Embed & Sign” (or “Do Not Embed” if you
     prefer to handle it manually).

Replace the SwiftUI `App` body with:

```swift
import SwiftUI

@main
struct EguiImeTestApp: App {
    init() {
        eframe_ios_run_demo()
    }

    var body: some Scene {
        WindowGroup {
            // Empty view: Rust renders everything.
            Color.clear
        }
    }
}
```

## 3. Simulator validation

1. Select an iPhone simulator (e.g. iPhone 16, iOS 18).
2. Run the app from Xcode.
3. Inside the egui demo window:
   * Open “Widgets → Text Edit” and tap the multiline text field.
   * The software keyboard must appear.
   * Type characters and verify they show up immediately inside the egui text
     edit. This confirms that `WindowEvent::Ime` events reach egui.
4. Repeat the test on “Widgets → Text Edit (Singleline)” if you need extra
   coverage.

If the keyboard fails to appear or characters don’t show up, capture the Xcode
console log and file an issue referencing the egui and winit versions used.
