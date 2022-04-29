#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] //

mod chess;
use chess::{LiBoard, MovePiece, QUEEN_WHITE};
use eframe::{
    egui::{self, Sense, Ui},
    emath::{Numeric, Pos2, Rect, Vec2},
    epaint::{Color32, ColorImage, TextureHandle},
};
use std::{collections::HashMap, hash::Hash, ptr::NonNull};
use std::{path::Path, thread};
use tokio::io::{stdin, AsyncReadExt};

fn main() {
    let options = eframe::NativeOptions {
        // Let's show off that we support transparent windows
        transparent: false,
        drag_and_drop_support: true,
        min_window_size: Some(Vec2 {
            x: 1000.0,
            y: 600.0,
        }),
        ..Default::default()
    };

    eframe::run_native(
        "LiLearn",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    );
}

// store main app state here?...
// egui has dragging implemented already !
struct MyApp {
    textures: HashMap<i32, Option<egui::TextureHandle>>, // piece -> texture mapping
    board: LiBoard,
    cur_move_cnt: i32,
    optimal_move_cnt: i32,
    choice_piece: i32,
    star_cnt: i32,
    board_light_sq_color: Color32,
    board_dark_sq_color: Color32,
    auto_play: bool,
}

enum PieceStates {
    Dragged(Rect, i32),            // where to draw image and what image to draw
    DragReleased(Rect, MovePiece), // draw the image just before releasing
    NoDrag,
}

impl Default for MyApp {
    fn default() -> Self {
        let b = chess::LiBoard::new(5, chess::QUEEN_WHITE);
        let opt_cnt = b.num_optimal_moves_to_star();
        Self {
            textures: HashMap::new(),
            board: b,
            optimal_move_cnt: opt_cnt,
            cur_move_cnt: 0,
            choice_piece: QUEEN_WHITE,
            star_cnt: 5,
            board_light_sq_color: Color32::WHITE,
            board_dark_sq_color: Color32::DARK_BLUE,
            auto_play: false,
        }
    }
}

// piece paths (legacy code)
static paths: [&str; 15] = [
    "",
    "images/black_pawn.png",
    "images/white_pawn.png",
    "images/black_bishop.png",
    "images/black_knight.png",
    "images/black_rook.png",
    "images/black_queen.png",
    "images/black_king.png",
    "",
    "",
    "images/white_rook.png",
    "images/white_knight.png",
    "images/white_bishop.png",
    "images/white_queen.png",
    "images/white_king.png",
];

fn play_sound(path_to_file: &'static str) {
    thread::spawn(move || {
        use rodio::{source::Source, Decoder, OutputStream};
        use std::fs::File;
        use std::io::BufReader;
        // Get a output stream handle to the default physical sound device
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        // Load a sound from a file, using a path relative to Cargo.toml
        let file = BufReader::new(File::open(path_to_file).unwrap());
        // Decode that sound file into a source
        let source = Decoder::new(file).unwrap();
        // Play the sound directly on the device
        stream_handle.play_raw(source.convert_samples());

        // The sound plays in a separate audio thread,
        // so we need to keep the main thread alive while it's playing.
        std::thread::sleep(std::time::Duration::from_millis(700));
    });
}

fn load_image_from_path(path: &std::path::Path) -> Result<egui::ColorImage, image::ImageError> {
    let image = image::io::Reader::open(path)?.decode()?;
    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();
    Ok(egui::ColorImage::from_rgba_unmultiplied(
        size,
        pixels.as_slice(),
    ))
}

fn get_texture<'a>(app: &'a mut MyApp, ui: &'a mut Ui, img_id: i32) -> &'a TextureHandle {
    // where to draw currently dragged image
    // insert id if it isn't there
    if !app.textures.contains_key(&img_id) {
        app.textures.insert(img_id, None);
    }

    app.textures
        .get_mut(&img_id)
        .unwrap()
        .get_or_insert_with(|| {
            let mut img;
            let mut name;
            if img_id == 99 {
                // load star
                img = load_image_from_path(Path::new("./images/star.png")).unwrap();
                name = "star_img";
            } else {
                img = load_image_from_path(Path::new(paths[img_id as usize])).unwrap();
                name = paths[img_id as usize];
            }

            ui.ctx().load_texture(name, img)
        });

    let texture = app.textures[&img_id].as_ref().unwrap();
    texture
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let mut visuals = egui::Visuals::dark();
        visuals.override_text_color = Some(Color32::from_gray(200));
        visuals.window_shadow = egui::epaint::Shadow::small_dark();
        ctx.set_visuals(visuals);
        egui::containers::Window::new("controls")
            // .default_size(Vec2 {x: 400.0, y: 400.0})
            .resizable(true)
            .show(ctx, |ui| {
                /*
                centers
                ui.columns(5, |col| {
                    col[1].radio_value(&mut self.choice_piece, chess::QUEEN_WHITE, "Queen");
                    col[2].radio_value(&mut self.choice_piece, chess::KNIGHT_WHITE, "Knight");
                    col[3].radio_value(&mut self.choice_piece, chess::ROOK_WHITE, "Rook");
                });
                */
                ui.horizontal(|ui| {
                    ui.radio_value(&mut self.choice_piece, chess::QUEEN_WHITE, "Queen");
                    ui.radio_value(&mut self.choice_piece, chess::KNIGHT_WHITE, "Knight");
                    ui.radio_value(&mut self.choice_piece, chess::ROOK_WHITE, "Rook");
                });
                ui.add(egui::Slider::new(&mut self.star_cnt, 1..=13));

                // pick board colors
                ui.horizontal(|ui| {
                    ui.label("dark square color picker: ");
                    ui.color_edit_button_srgba(&mut self.board_light_sq_color);
                });
                ui.horizontal(|ui| {
                    ui.label("light square color picker: ");
                    ui.color_edit_button_srgba(&mut self.board_dark_sq_color);
                });
                ui.separator();
                ui.checkbox(&mut self.auto_play, "Auto play");
                ui.separator();
                let new_round_btn = egui::Button::new("new round");

                if ui.add(new_round_btn).clicked()
                    || (self.auto_play && self.board.num_star_cnt == 0)
                {
                    self.board = LiBoard::new(self.star_cnt as u8, self.choice_piece);
                    self.cur_move_cnt = 0;
                    self.optimal_move_cnt = self.board.num_optimal_moves_to_star();
                }
            });

        egui::containers::Window::new("chess window")
            // .default_size(Vec2 {x: 400.0, y: 400.0})
            .resizable(true)
            .show(ctx, |ui| {
                ui.label("Number of current moves: ".to_owned() + &self.cur_move_cnt.to_string());
                ui.label("Optimal: ".to_owned() + &self.optimal_move_cnt.to_string());
                ui.add_space(5.0);
                // println!("{}",self.board.num_optimal_moves_to_star());
                let (r, _) = ui.allocate_at_least(ui.available_size(), Sense::click());
                let mut piece_state = PieceStates::NoDrag;

                for i in 0..8 {
                    for j in 0..8 {
                        let size = ((r.max.x - r.min.x) / 8.0).min((r.max.y - r.min.y) / 8.0);
                        let sq = Rect {
                            min: Pos2 {
                                x: j as f32 * size + r.min.x,
                                y: i as f32 * size + r.min.y,
                            },
                            max: Pos2 {
                                x: j as f32 * size + size + r.min.x,
                                y: i as f32 * size + size + r.min.y,
                            },
                        };
                        let mut temp_color = self.board_dark_sq_color;
                        if j % 2 == 0 {
                            if i % 2 == 0 {
                                temp_color = self.board_light_sq_color;
                            }
                        } else {
                            if i % 2 == 1 {
                                temp_color = self.board_light_sq_color;
                            }
                        };
                        let piece_resp = ui.allocate_rect(sq, Sense::drag());

                        let cur_input_pos = ctx.input().pointer.interact_pos();

                        if piece_resp.drag_released() {
                            // done dragging here.. potentially update board state for next frame
                            assert!(!piece_resp.dragged());
                            let a = ctx.input().pointer.interact_pos();
                            if a.is_some() && r.contains(a.unwrap()) {
                                let a = a.unwrap();
                                let goal_j = (a.x - r.min.x) / size;
                                let goal_i = (a.y - r.min.y) / size;
                                let image_rect = Rect {
                                    min: Pos2 {
                                        x: (goal_j as i32) as f32 * size + r.min.x,
                                        y: (goal_i as i32) as f32 * size + r.min.y,
                                    },
                                    max: Pos2 {
                                        x: (goal_j as i32) as f32 * size + size + r.min.x,
                                        y: (goal_i as i32) as f32 * size + size + r.min.y,
                                    },
                                };
                                piece_state = PieceStates::DragReleased(
                                    image_rect,
                                    MovePiece {
                                        i: i as usize,
                                        j: j as usize,
                                        goal_i: goal_i as usize,
                                        goal_j: goal_j as usize,
                                    },
                                );
                            }

                            ui.painter().rect_filled(sq, 0.0, temp_color);
                        } else if piece_resp.dragged() {
                            // println!("{:?} {:?} {:?} {:?} {:?}",r,cur_input_pos, (i,j), sq, piece_resp.rect);
                            // currently dragging.. draw the texture at current mouse pos
                            let piece_being_moved = self.board.board[i as usize][j as usize];
                            if !cur_input_pos.is_none() && piece_being_moved != 0 {
                                let cur_input_pos = cur_input_pos.unwrap();
                                // draw at the center of mouse when grabbed
                                let start_of_rec = Pos2 {
                                    x: cur_input_pos.x - size / 2.0,
                                    y: cur_input_pos.y - size / 2.0,
                                };
                                let end_of_rec = Pos2 {
                                    x: start_of_rec.x + size,
                                    y: start_of_rec.y + size,
                                };
                                let image_rect = Rect {
                                    min: start_of_rec,
                                    max: end_of_rec,
                                };

                                piece_state = PieceStates::Dragged(image_rect, piece_being_moved);
                            }
                            ui.painter().rect_filled(sq, 0.0, temp_color);
                        } else if !piece_resp.dragged() && !piece_resp.drag_released() {
                            ui.painter().rect_filled(sq, 0.0, temp_color);
                            // paint image
                            let piece_being_moved = self.board.board[i as usize][j as usize];
                            if piece_being_moved != 0 {
                                let texture = get_texture(self, ui, piece_being_moved);
                                // Show the image:
                                egui::Image::new(texture, texture.size_vec2()).paint_at(ui, sq);
                            }
                        } else {
                            ui.painter().rect_filled(sq, 0.0, temp_color);
                        }
                    }
                }

                // draw the "dragged piece" here
                match piece_state {
                    PieceStates::Dragged(piece_rect, img_id) => {
                        let texture = get_texture(self, ui, img_id);

                        // Show the image:
                        egui::Image::new(texture, texture.size_vec2()).paint_at(ui, piece_rect);
                    }
                    PieceStates::DragReleased(piece_rect, move_piece) => {
                        if self.board.validate_move(&move_piece) != 0 {
                            if self.board.board[move_piece.goal_i][move_piece.goal_j]
                                == chess::STAR_VALUE
                            {
                                play_sound("./sounds/capture.wav");
                                self.board.num_star_cnt -= 1;
                            } else {
                                play_sound("./sounds/move.wav");
                            }
                            self.board.update_board(&move_piece);
                            // let img_id = self.board.board[move_piece.goal_i][move_piece.goal_j];
                            // let texture = get_texture(self,ui,img_id);
                            // Show the image:
                            // egui::Image::new(texture, texture.size_vec2()).paint_at(ui, piece_rect);
                            if self.board.num_star_cnt == 0 {
                                play_sound("./sounds/win.wav");
                            }
                            self.cur_move_cnt += 1;
                        }
                        // validate goali and j so they are within bounds
                        if !( move_piece.goal_i >= 8
                            || move_piece.goal_j >= 8)
                        {
                            let img_id = self.board.board[move_piece.goal_i][move_piece.goal_j];
                            if img_id != 0 {
                                let texture = get_texture(self, ui, img_id);
                                egui::Image::new(texture, texture.size_vec2())
                                    .paint_at(ui, piece_rect);
                            }
                        }
                    }
                    _ => (),
                }

                if self.board.num_star_cnt == 0 {
                    ui.vertical_centered_justified(|ui| {
                        ui.add_space(5.0);
                        ui.label(
                            egui::RichText::new("You finished!")
                                .color(Color32::LIGHT_GREEN)
                                .size(25.0),
                        );
                    });
                }

                /*
                //slow mode for debugging
                 let mut i = i32::MAX;
                 while i > 0  { i -= 20;}
                 */
            });

        // Resize the native window to be just the size we need it to be:
        // frame.set_window_size(ctx.used_size());
    }
}
