#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use crate::chess::STAR_VALUE;

// hide console window on Windows in release
use super::chess::{self, LiBoard, MovePiece};
use super::egui_widgets::progress_bar::ProgressBar;
use eframe::{
    egui::{self, Sense, TextBuffer, TextureOptions, Ui},
    emath::{Pos2, Rect},
    epaint::{Color32, TextureHandle},
};
use egui::{Painter, PointerButton, Stroke, Vec2};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{collections::HashMap, time::Duration};

// store main app state here?...
// egui has dragging implemented already !
pub struct MyApp {
    // pictures and animations
    textures: HashMap<i8, Option<egui::TextureHandle>>, // piece -> texture mapping
    // game state
    moves_played_so_far: Vec<MovePiece>,
    arrows_to_draw: Vec<ArrowMove>,
    board: LiBoard,
    cur_move_cnt: i8,
    optimal_move_cnt: i8,
    choice_piece: i8,
    star_cnt: i8,
    board_light_sq_color: Color32,
    board_dark_sq_color: Color32,
    window_bg_color: Color32,
    arrow_color: Color32,
    side_panel_color: Color32,
    arrow_thickness: f32,
    auto_play: bool,
    in_game: bool,
    // timer things
    timed: bool, // see how many rounds you can complete in X minutes
    starting_timer: u64,
    timer: u64, // using frames as ref
    in_timed_round: bool,
    cur_timed_num_wins: i32,
    last_timed_game: Option<i32>,
    // stats
    points: u64,
    streak: u64,
    // ui sizing
    board_width: Option<f32>,
    // Manual click drag tracking. egui doesn't support figuring out what button a widget was released by.
    secondary_clicked: bool,
    primary_clicked: bool,
}

// Captures drawing an arrow from (start_i, start_j) to (end_i, end_j).
// Arrows are drawn by right click.
#[derive(Debug)]
struct ArrowMove {
    start_x: f32,
    start_y: f32,
    x: f32,
    y: f32,
}

enum PieceStates {
    Dragged(Rect, i8),             // where to draw image and what image to draw
    DragReleased(Rect, MovePiece), // draw the image just before releasing
    ArrowDragReleased(ArrowMove),  // where to draw arrows
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
            moves_played_so_far: Vec::new(),
            arrows_to_draw: Vec::new(),
            cur_move_cnt: 0,
            choice_piece: chess::QUEEN_WHITE,
            star_cnt: 5,
            board_light_sq_color: Color32::WHITE,
            board_dark_sq_color: Color32::BLACK,
            auto_play: false,
            window_bg_color: Color32::BLACK,
            arrow_color: Color32::YELLOW,
            side_panel_color: Color32::WHITE,
            arrow_thickness: 4.0,
            // timers
            timed: false,
            timer: 0,
            in_timed_round: false,
            cur_timed_num_wins: 0,
            last_timed_game: None,
            starting_timer: 2000,
            streak: 0,
            points: 0,
            in_game: true,
            board_width: None,
            secondary_clicked: false,
            primary_clicked: false,
        }
    }
}

// piece IMAGES
static IMAGES: [&[u8]; 6] = [
    include_bytes!("../images/star.png").as_slice(),
    include_bytes!("../images/icon.png").as_slice(),
    include_bytes!("../images/white_rook.png").as_slice(),
    include_bytes!("../images/white_knight.png").as_slice(),
    include_bytes!("../images/white_queen.png").as_slice(),
    include_bytes!("../images/fire.png").as_slice(),
];

// piece AUDIO
static AUDIO: [&[u8]; 3] = [
    include_bytes!("../sounds/move.wav").as_slice(),
    include_bytes!("../sounds/win.wav").as_slice(),
    include_bytes!("../sounds/capture.wav").as_slice(),
];

fn img_id_map(i: i8) -> usize {
    match i {
        chess::QUEEN_WHITE => 4,
        chess::KNIGHT_WHITE => 3,
        chess::ROOK_WHITE => 2,
        _ => panic!("invalid Image request"),
    }
}

#[cfg(target_arch = "wasm32")]
pub fn now_sec() -> u64 {
    (eframe::web_sys::window()
        .expect("should have a Window")
        .performance()
        .expect("should have a Performance")
        .now()
        / 1000.0) as u64
}

fn play_sound(name: &'static str) {
    #[cfg(target_arch = "wasm32")]
    {
        use web_sys::HtmlAudioElement;
        let sample = match name.as_str() {
            "move" => "move.wav",
            "win" => "win.wav",
            "capture" => "capture.wav",
            _ => panic!("wrong type of sound?"),
        };
        let mut player = web_sys::HtmlAudioElement::new_with_src(sample).unwrap();
        player.play();
    }
    #[cfg(not(target_arch = "wasm32"))]
    thread::spawn(move || {
        use rodio::{source::Source, Decoder, OutputStream};
        use std::io::Cursor;
        let sample = match name.as_str() {
            "move" => AUDIO[0],
            "win" => AUDIO[1],
            "capture" => AUDIO[2],
            _ => panic!("wrong type of sound?"),
        };
        // Get a output stream handle to the default physical sound device
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        // using cursor to load it in memory
        let file = Cursor::new(sample);
        // Decode that sound file into a source
        let source = Decoder::new(file).unwrap();
        // Play the sound directly on the device
        if stream_handle.play_raw(source.convert_samples()).is_ok() {
            std::thread::sleep(std::time::Duration::from_millis(1400));
        }
    });
}

pub fn load_image(img: &[u8]) -> Result<egui::ColorImage, image::ImageError> {
    let image = image::load_from_memory_with_format(img, image::ImageFormat::Png).unwrap();
    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();
    Ok(egui::ColorImage::from_rgba_unmultiplied(
        size,
        pixels.as_slice(),
    ))
}

fn get_texture<'a>(app: &'a mut MyApp, ui: &'a mut Ui, img_id: i8) -> &'a TextureHandle {
    // where to draw currently dragged image
    // insert id if it isn't there
    app.textures.entry(img_id).or_insert(None);

    app.textures
        .get_mut(&img_id)
        .unwrap()
        .get_or_insert_with(|| {
            let img;
            let name;
            if img_id == chess::STAR_VALUE {
                // load star
                img = load_image(IMAGES[0]).unwrap();
                name = "star_img";
            } else if img_id == 24 {
                // TODO remove magic nums
                img = load_image(IMAGES[5]).unwrap();
                name = "fire";
            } else {
                img = load_image(IMAGES[img_id_map(img_id)]).unwrap();
                name = "others"; // TODO fix
            }
            ui.ctx().load_texture(name, img, TextureOptions::default())
        });

    app.textures[&img_id].as_ref().unwrap()
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Controls styles
        let mut visuals = egui::Visuals::light();
        visuals.window_shadow = egui::epaint::Shadow::small_dark();
        visuals.widgets.noninteractive.bg_stroke.width = 0.0;
        ctx.tessellation_options().feathering_size_in_pixels = 0.3;
        let mut style: egui::Style = (*ctx.style()).clone();
        style.spacing.indent = 11.0;
        ctx.set_style(style);
        ctx.set_visuals(visuals.clone());
        // TODO add new fonts
        egui::containers::SidePanel::left("Controls")
            .resizable(true)
            .frame(egui::containers::Frame {
                inner_margin: egui::style::Margin::from(15.0),
                fill: self.side_panel_color,
                ..Default::default()
            })
            .show(ctx, |ui| {

                ui.add_space(5.0);
                ui.vertical_centered_justified(|ui| {
                    ui.heading("LiLearn");
                });
                ui.add_space(25.0);


                ui.label(format!("Points: {}",self.points));

                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(format!("🔥: {}",self.streak))
                    .color(Color32::RED)
                    );
                });


                // show win msgs
                if !self.in_timed_round && self.board.num_star_cnt == 0 && !self.auto_play && !self.in_game {
                    let mut msg = "You were close!";
                    let mut msg_color = Color32::DARK_GREEN;
                    if self.cur_move_cnt == self.optimal_move_cnt {
                        msg = "Excellent! 🔥🔥🔥";
                        msg_color = Color32::from_rgb(91, 33, 50)
                    }
                    ui.label(
                        egui::RichText::new(msg)
                            .color(msg_color)
                            ,
                    );
                }

                match self.last_timed_game {
                    None => (),
                    Some(v) => {
                        ui.add_space(10.0);
                        ui.label(
                            egui::RichText::new(format!(
                                "You won {} round(s) in your last timed game",
                                v
                            ))
                            .color(Color32::DARK_GREEN)
                        );
                    }
                }

                ui.collapsing(" How to play:", |ui| {
                    ui.label("Try to collect all the stars with as few moves as possible! There's also a timed mode if you are up for the challenge! The timer is set in seconds.");
                    ui.add_space(2.0);
                });
                
                if !self.in_timed_round {
                    ui.horizontal(|ui| {
                        ui.radio_value(&mut self.choice_piece, chess::QUEEN_WHITE, "Queen");
                        ui.radio_value(&mut self.choice_piece, chess::KNIGHT_WHITE, "Knight");
                        ui.radio_value(&mut self.choice_piece, chess::ROOK_WHITE, "Rook");
                    });

                    ui.horizontal(|ui| {
                        ui.label("Number of stars: ");
                        ui.add(egui::Slider::new(&mut self.star_cnt, 1..=18));
                    }); 
                    ui.horizontal(|ui| {
                        ui.label("Arrow thickness: ");
                        ui.add(egui::Slider::new(&mut self.arrow_thickness, 1.0..=30.0));
                    });
                }
                egui::Grid::new("my_grid")
                .num_columns(2)
                .spacing([23.0, 4.0])
                .show(ui, |ui| {
                    // pick board colors

                    ui.label("Dark square color: ");
                    ui.color_edit_button_srgba(&mut self.board_dark_sq_color);
                    ui.end_row();

                    ui.label("Light square color: ");
                    ui.color_edit_button_srgba(&mut self.board_light_sq_color);
                    ui.end_row();

                    ui.label("Window background color: ");
                    ui.color_edit_button_srgba(&mut self.window_bg_color);
                    ui.end_row();

                    ui.label("Arrow color: ");
                    ui.color_edit_button_srgba(&mut self.arrow_color);
                    ui.end_row();

                    ui.label("Side panel color: ");
                    ui.color_edit_button_srgba(&mut self.side_panel_color);
                    ui.end_row();
                });
                ui.horizontal(|ui| {
                    if !self.timed {
                        ui.checkbox(&mut self.auto_play, "Auto play");
                    }
                    ui.checkbox(&mut self.timed, "Timed rounds");
                });

                ui.add_space(2.0);
                ui.vertical_centered_justified(|ui| {
                    if !self.in_timed_round {
                        ui.menu_button("Timer", |ui| {
                            ui.horizontal(|ui| {
                                ui.label("Set timer: ");
                                ui.add(egui::Slider::new(&mut self.starting_timer, 1..=500));
                            });

                            if ui.button("Close").clicked() {
                                ui.close_menu();
                            }
                        });
                    }

                    ui.add_space(2.0);
                    let new_round_btn = egui::Button::new("New round");

                    if ui.add(new_round_btn).clicked() {
                        self.last_timed_game = None;
                        self.cur_timed_num_wins = 0;
                        
                        if self.timed {
                            self.auto_play = true;
                            self.in_timed_round = true;
                            #[cfg(not(target_arch = "wasm32"))]
                            let cur_time = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .expect("Time went backwards")
                                .as_secs();
                            
                            #[cfg(target_arch = "wasm32")]
                            let cur_time =  now_sec();

                            self.timer = cur_time;
                        } else {
                            self.timed = false;
                            self.in_timed_round = false;
                        }

                        self.in_game = true;
                        self.board = LiBoard::new(self.star_cnt as i8, self.choice_piece);
                        self.cur_move_cnt = 0;
                        self.optimal_move_cnt = self.board.num_optimal_moves_to_star();
                    }

                    if self.auto_play && self.board.num_star_cnt == 0 {
                        self.in_game = true;
                        self.board = LiBoard::new(self.star_cnt as i8, self.choice_piece);
                        self.cur_move_cnt = 0;
                        self.optimal_move_cnt = self.board.num_optimal_moves_to_star();
                    }
                });
            });

        // set window colors
        visuals = egui::Visuals::dark();
        visuals.override_text_color = Some(Color32::from_gray(255));
        visuals.widgets.noninteractive.bg_fill = Color32::BLACK;
        visuals.selection.bg_fill = Color32::RED;
        visuals.extreme_bg_color = Color32::from_gray(0); // ProgressBar bg color... :/
        ctx.set_visuals(visuals);

        egui::containers::CentralPanel::default()
            .frame(egui::containers::Frame {
                inner_margin: egui::style::Margin::from(25.0),
                fill: self.window_bg_color,
                ..Default::default()
            })
            .show(ctx, |ui| {
                let mut show_progress_bar = false;
                // get cur time and compare
                #[cfg(not(target_arch = "wasm32"))]
                let cur_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs();

                #[cfg(target_arch = "wasm32")]
                let cur_time = now_sec();
                if self.in_timed_round {
                    if self.timer + self.starting_timer <= cur_time {
                        self.in_timed_round = false;
                        self.last_timed_game = Some(self.cur_timed_num_wins);
                        self.cur_timed_num_wins = 0;
                        self.cur_move_cnt = 0;
                        self.timer = cur_time;
                        // restart and create a new game
                        self.in_game = true;
                        self.board = LiBoard::new(self.star_cnt as i8, self.choice_piece);
                        self.optimal_move_cnt = self.board.num_optimal_moves_to_star();
                    } else {
                        show_progress_bar = true;
                    }
                }
                ui.label("Number of current moves: ".to_owned() + &self.cur_move_cnt.to_string());
                ui.add_space(3.0);
                ui.label("Optimal: ".to_owned() + &self.optimal_move_cnt.to_string());
                ui.add_space(3.0);
                if show_progress_bar {
                    ui.label(format!(
                        "Time left: {}",
                        self.starting_timer - (cur_time - self.timer)
                    ));
                    ui.add_space(3.0);
                    let ratio_f = (cur_time - self.timer) as f32 / self.starting_timer as f32;
                    match self.board_width {
                        Some(v) => ui.add(
                            ProgressBar::new(ratio_f)
                                .desired_width(v)
                                .desired_rounding(2.5),
                        ),
                        None => ui.add(ProgressBar::new(ratio_f).desired_rounding(2.5)),
                    };
                    ui.add_space(3.0);
                }
                ui.add_space(4.0);
                let (r, _) = ui.allocate_at_least(ui.available_size(), Sense::click());
                let size = ((r.max.x - r.min.x) / 8.0).min((r.max.y - r.min.y) / 8.0); // width of square
                self.board_width = Some(size * 8.0);
                let mut piece_state = PieceStates::NoDrag;
                ui.add_space(5.0);
                for i in 0..8 {
                    for j in 0..8 {
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
                        } else if i % 2 == 1 {
                            temp_color = self.board_light_sq_color;
                        };
                        let piece_resp = ui.allocate_rect(sq, Sense::drag());

                        let cur_input_pos = ctx.input().pointer.interact_pos();
                        let piece_being_moved = self.board.board[i as usize][j as usize];
                        // paint squares
                        ui.painter().rect_filled(sq, 0.0, temp_color);

                        // Handle arrow drags
                        if piece_resp.dragged_by(PointerButton::Secondary) {
                            self.secondary_clicked = true;
                            // paint image
                            if piece_being_moved != 0 {
                                let texture = get_texture(self, ui, piece_being_moved);
                                // Show the image:
                                egui::Image::new(texture, texture.size_vec2()).paint_at(ui, sq);
                            }
                        } else if piece_resp.dragged_by(PointerButton::Primary)
                            && piece_being_moved != 0
                        {
                            self.primary_clicked = true;
                            // currently dragging.. draw the texture at current mouse pos
                            if cur_input_pos.is_some() && piece_being_moved != 0 {
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
                        }
                        // arrow drag released
                        else if self.secondary_clicked && piece_resp.drag_released() {
                            self.secondary_clicked = false;
                            let a = ctx.input().pointer.interact_pos();
                            if a.is_some() && r.contains(a.unwrap()) {
                                let a = a.unwrap();
                                let goal_j = (a.x - r.min.x) / size;
                                let goal_i = (a.y - r.min.y) / size;
                                let start_x = (j as i8) as f32 * size + r.min.x + size / 2.0;
                                let start_y = (i as i8) as f32 * size + r.min.y + size / 2.0;
                                piece_state = PieceStates::ArrowDragReleased(ArrowMove {
                                    start_x,
                                    start_y,
                                    x: (goal_j as i8) as f32 * size + r.min.x + size / 2.0
                                        - start_x,
                                    y: (goal_i as i8) as f32 * size + r.min.y + size / 2.0
                                        - start_y,
                                });
                            }
                        }
                        // Handle primary button drags
                        else if self.primary_clicked
                            && piece_resp.drag_released()
                            && piece_being_moved != STAR_VALUE
                        {
                            self.primary_clicked = false;
                            // done dragging here.. potentially update board state for next frame
                            let a = ctx.input().pointer.interact_pos();
                            if a.is_some() && r.contains(a.unwrap()) {
                                let a = a.unwrap();
                                let goal_j = (a.x - r.min.x) / size;
                                let goal_i = (a.y - r.min.y) / size;
                                let image_rect = Rect {
                                    min: Pos2 {
                                        x: (goal_j as i8) as f32 * size + r.min.x,
                                        y: (goal_i as i8) as f32 * size + r.min.y,
                                    },
                                    max: Pos2 {
                                        x: (goal_j as i8) as f32 * size + size + r.min.x,
                                        y: (goal_i as i8) as f32 * size + size + r.min.y,
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
                        } else {
                            // paint image
                            if piece_being_moved != 0 {
                                let texture = get_texture(self, ui, piece_being_moved);
                                // Show the image:
                                egui::Image::new(texture, texture.size_vec2()).paint_at(ui, sq);
                            }
                        }
                    }
                }

                // Draw the "dragged piece"
                match piece_state {
                    PieceStates::Dragged(piece_rect, img_id) => {
                        let texture = get_texture(self, ui, img_id);

                        // Show the image:
                        egui::Image::new(texture, texture.size_vec2()).paint_at(ui, piece_rect);
                    }
                    PieceStates::DragReleased(piece_rect, move_piece) => {
                        if self.board.validate_move(&move_piece).is_valid() {
                            if self.board.board[move_piece.goal_i][move_piece.goal_j]
                                == chess::STAR_VALUE
                            {
                                play_sound("capture");
                                self.board.num_star_cnt -= 1;
                            } else {
                                play_sound("move");
                            }
                            self.board.update_board(&move_piece);
                            // let img_id = self.board.board[move_piece.goal_i][move_piece.goal_j];
                            // let texture = get_texture(self,ui,img_id);
                            // Show the image:
                            // egui::Image::new(texture, texture.size_vec2()).paint_at(ui, piece_rect);
                            if self.in_game {
                                self.cur_move_cnt += 1;
                            }
                            if self.board.num_star_cnt == 0 && self.in_game {
                                play_sound("win");
                            }
                        }
                        // validate goali and j so they are within bounds
                        if !(move_piece.goal_i >= 8 || move_piece.goal_j >= 8) {
                            let img_id = self.board.board[move_piece.goal_i][move_piece.goal_j];
                            if img_id != 0 {
                                let texture = get_texture(self, ui, img_id);
                                egui::Image::new(texture, texture.size_vec2())
                                    .paint_at(ui, piece_rect);
                            }
                        }
                    }
                    PieceStates::ArrowDragReleased(arrow_move) => {
                        self.arrows_to_draw.push(arrow_move)
                    }
                    _ => (),
                }

                // Draw arrows
                for ArrowMove {
                    start_x,
                    start_y,
                    x,
                    y,
                } in &self.arrows_to_draw
                {
                    arrow(
                        ui.painter(),
                        Pos2::new(*start_x, *start_y),
                        Vec2::new(*x, *y),
                        Stroke::new(self.arrow_thickness, self.arrow_color),
                    );
                }

                // Update game stats when all the stars are collected
                if self.board.num_star_cnt == 0 && self.in_game {
                    // clear arrow drawings
                    self.arrows_to_draw.clear();
                    self.in_game = false;
                    match (self.cur_move_cnt - self.optimal_move_cnt).abs() {
                        // handle point system 100 : perfect , 10, off by 1
                        0 => {
                            self.cur_timed_num_wins += 1;
                            self.points += 100;
                            self.streak += 1;
                        }
                        1 => {
                            self.points += 10;
                            self.streak = 0;
                        }
                        _ => self.streak = 0,
                    }
                }
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button("Undo Drawing").clicked() {
                        self.arrows_to_draw.pop();
                    }

                    if ui.button("Clear Drawing").clicked() {
                        self.arrows_to_draw.clear()
                    }
                });
                // slow mode for debugging
                // let mut i = i8::MAX;
                // while i > 0  { i -= 20;}
            });

        // If a timed round is happening, repaint every second.
        if self.in_timed_round {
            ctx.request_repaint_after(Duration::from_secs(1));
        }
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> egui::Rgba {
        // sets window bg color
        self.window_bg_color.into()
    }
}

pub fn arrow(painter: &Painter, origin: Pos2, vec: Vec2, stroke: Stroke) {
    use egui::emath::*;
    let rot = Rot2::from_angle(std::f32::consts::TAU / 6.0);
    let tip_length = 10.0 + stroke.width;
    let tip = origin + vec;
    let dir = vec.normalized();
    painter.line_segment([origin, tip], stroke);
    painter.circle_filled(tip, stroke.width / 2.0, stroke.color);
    painter.line_segment([tip, tip - tip_length * (rot * dir)], stroke);
    painter.line_segment([tip, tip - tip_length * (rot.inverse() * dir)], stroke);
}
