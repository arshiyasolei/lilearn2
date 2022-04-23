#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::{egui::{self, Sense}, epaint::Color32, emath::{Rect, Pos2, Vec2, Numeric}};

mod chess;

fn main() {
    let options = eframe::NativeOptions {
        // Let's show off that we support transparent windows
        transparent: true,
        drag_and_drop_support: true,
        ..Default::default()
    };


    eframe::run_native(
        "My app",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    );
}

// store main app state here?...
struct MyApp {
    name: String,
    age: u32,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            name: "Arshi".to_owned(),
            age: 42,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // ctx.set_visuals(egui::Visuals::dark());        
        egui::containers::Window::new("chess window")
            // .default_size(Vec2 {x: 400.0, y: 400.0})
            .resizable(true)
            .show(ctx, |ui| {
                let (r , res) = ui.allocate_at_least(ui.available_size(), Sense::click());
            
                for i in 0..8 {
                    for j in 0..8 {
                        let size = ((r.max.x - r.min.x) / 8.0).min((r.max.y - r.min.y) / 8.0);
                        let sq = Rect {
                            min: Pos2 {
                                x: j as f32 *size + r.min.x,
                                y: i as f32 *size + r.min.y
                            }, 
                            max: Pos2 {
                                x: j as f32 *size + size + r.min.x,
                                y: i as f32 *size + size + r.min.y
                            }
                        };
                        let mut temp_color = Color32::BLACK;
                        if j % 2 == 0 {
                            if i % 2 == 0 {
                                temp_color = Color32::WHITE;
                            }
                        } else {
                            if i % 2 == 1 {
                                temp_color = Color32::WHITE;
                            }
                        };
                        ui.painter()
                            .rect_filled(sq, 0.0, temp_color);
                    }
                }
            });
        
        return;
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My pog Application");
            
            ui.painter()
                .rect_filled(Rect {min: Pos2 {x: 10.0, y: 50.0}, max: Pos2 {x: 100.0, y: 80.9}}, 0.1, Color32::YELLOW);
                        
            // ui.image(texture_id, size)
            let small_btn = egui::Button::new("smol").sense(Sense::drag());
            let poggers_resp = ui.add(small_btn);

            let small_btn = egui::Button::new("smol").sense(Sense::drag());
            
            let btn_resp = ui.add(small_btn);
            

            if poggers_resp.dragged() {
                println!("what");
            }
            
            if btn_resp.dragged() {
                
                let Vec2 {x,y} = btn_resp.drag_delta();
                println!("{} {}",x,y);
                let cur_mouse_pos = ctx.input().pointer.interact_pos().unwrap();
                let end_of_rec = Pos2 {x: cur_mouse_pos.x + 5.0, y: cur_mouse_pos.y + 5.0};
                // paint the original button we are holding to some other color?
                ui.painter()
                .rect_filled(btn_resp.rect, 1.0, Color32::RED);

                ui.painter()
                .rect_filled(Rect {min: cur_mouse_pos, max: end_of_rec}, 1.0, Color32::RED);
            } else {
                println!("not dragged");
            }
        });

        // Resize the native window to be just the size we need it to be:
        frame.set_window_size(ctx.used_size());
    }
}