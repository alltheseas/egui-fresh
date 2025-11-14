# eframe-ios-runner

Minimal helpers for embedding `eframe` apps inside a native iOS host.

The current entry point exports a single C function, `eframe_ios_run_demo`,
which launches the built-in `egui_demo_app`. It is meant as a smoke test
for integrating the Rust event loop with a Swift/Objective‑C wrapper.

Future revisions will allow configuring custom apps from Swift/Objective‑C.
