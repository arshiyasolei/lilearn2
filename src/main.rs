#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod app;
use eframe::{
    egui::{self, CollapsingHeader, Sense, Ui},
    emath::{Numeric, Pos2, Rect, Vec2},
    epaint::{Color32, ColorImage, TextureHandle},
};
use std::{collections::HashMap, hash::Hash, ptr::NonNull};
use std::{path::Path, thread};
use std::time::{SystemTime, UNIX_EPOCH};


// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let options = eframe::NativeOptions {
        // Let's show off that we support transparent windows
        transparent: false,
        drag_and_drop_support: true,
        initial_window_size: Some(Vec2 { x: 730.0, y: 550.0 }),
        vsync: false,
        icon_data: Some(load_icon().unwrap()), 
        ..Default::default()
    };

    eframe::run_native(
        "LiLearn",
        options,
        Box::new(|_cc| Box::new(app::MyApp::default())),
    );
}

fn load_icon() -> Result< eframe::IconData , image::ImageError>{
    let image = image::io::Reader::open(Path::new("./images/icon.png"))?.decode()?;
    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();
    Ok(eframe::IconData {
        rgba: image_buffer.to_vec(),
        width: size[0],
        height: size[1]
    })
}


