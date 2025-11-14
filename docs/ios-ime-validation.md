# iOS IME validation checklist

Until we have automated simulator coverage, run the following steps manually
whenever we need to confirm that winit/egui still receive `WindowEvent::Ime`
events on iOS.

## 1. Build the runner

```
./scripts/build_ios_runner.sh
```

This produces:

* `target/ios/EframeIos.xcframework`
* `target/ios/eframe_ios_runner.h`

Both artefacts are required for the Xcode project in the next step.

## 2. Create or reuse a SwiftUI shell

1. In Xcode, create a new *App* project named `EguiImeTest`.
2. Add `EframeIos.xcframework` to the project (drag it into the Project
   navigator, select “Copy items if needed”).
3. Add a bridging header (e.g. `EguiImeTest-Bridging-Header.h`) with
   `#include "eframe_ios_runner.h"`.
4. Update the build settings of the app target:
   * Set `Objective-C Bridging Header` to the header path.
   * Ensure the app links against the xcframework (Xcode normally takes care of
     this when you add it to “Frameworks, Libraries, and Embedded Content”).

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
