#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(clippy::suspicious_else_formatting)]
#![allow(clippy::unnecessary_unwrap)]

mod app;
mod chess;
use eframe::emath::Vec2;
mod egui_widgets;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let options = eframe::NativeOptions {
        // Let's show off that we support transparent windows
        transparent: false,
        drag_and_drop_support: true,
        initial_window_size: Some(Vec2 { x: 730.0, y: 550.0 }),
        ..Default::default()
    };

    eframe::run_native(
        "LiLearn",
        options,
        Box::new(|_cc| Box::new(app::MyApp::default())),
    );
}

// ----------------------------------------------------------------------------
// When compiling for web:
#[cfg(target_arch = "wasm32")]
pub fn main() {
    let web_options = eframe::WebOptions::default();
    use lib::MyApp;
    eframe::start_web(
        "lilearn_id",
        web_options,
        Box::new(|cc| Box::new(MyApp::default())),
    );
}
