#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    // Log to stdout (if you run with `RUST_LOG=debug`).

    use egui::Vec2;
    tracing_subscriber::fmt::init();

    eframe::run_native(
        "epomo",
        eframe::NativeOptions {
            initial_window_size: Some(Vec2::new(200.0, 225.0)),
            resizable: false,
            ..Default::default()
        },
        Box::new(|cc| Box::new(epomo::EpomoApp::new(cc))),
    )
}
