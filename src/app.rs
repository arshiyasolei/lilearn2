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
use egui::{Button, Painter, PointerButton, RichText, Stroke, Vec2};
use rpds::Vector;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{collections::HashMap, time::Duration};

// store main app state here?...
// egui has dragging implemented already !
pub struct MyApp {
    // pictures and animations
    textures: HashMap<i8, Option<egui::TextureHandle>>, // piece -> texture mapping
    // game state
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
    side_panel_dark_mode: bool,
    auto_play: bool,
    in_game: bool,
    show_side_panel: bool,
    solution_path: chess::SolutionPath,
    show_solution: bool,
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
#[derive(Debug, Clone)]
struct ArrowMove {
    start_i: usize,
    start_j: usize,
    end_i: usize,
    end_j: usize,
}

impl From<MovePiece> for ArrowMove {
    fn from(m: MovePiece) -> Self {
        let MovePiece { i, j, goal_i, goal_j } = m;
        Self {
            start_i: i,
            start_j: j,
            end_i: goal_i,
            end_j: goal_j,
        }
    }
}

enum PieceStates {
    Dragged(Rect, i8),             // where to draw image and what image to draw
    ArrowDragged(ArrowMove),       // where to draw arrows as it's being dragged
    DragReleased(Rect, MovePiece), // draw the image just before releasing
    ArrowDragReleased(ArrowMove),  // where to draw arrows
    NoDrag,
}

impl MyApp {
    fn draw_arrow(&self, arrow_move: ArrowMove, painter: &Painter, size: f32, board_rect: Rect) {
        let ArrowMove { start_i, start_j, end_i, end_j } = arrow_move;
        let start_x = (start_j as i8) as f32 * size + board_rect.min.x + size / 2.0;
        let start_y = (start_i as i8) as f32 * size + board_rect.min.y + size / 2.0;

        let x = (end_j as i8) as f32 * size + board_rect.min.x + size / 2.0 - start_x;
        let y = (end_i as i8) as f32 * size + board_rect.min.y + size / 2.0 - start_y;
        arrow(painter, Pos2::new(start_x, start_y), Vec2::new(x, y), Stroke::new(size / 5.0, self.arrow_color));
    }
}

impl Default for MyApp {
    fn default() -> Self {
        let b = chess::LiBoard::new(5, chess::QUEEN_WHITE);
        let (opt_cnt, path) = b.num_optimal_moves_to_star();
        Self {
            textures: HashMap::new(),
            board: b.clone(),
            show_side_panel: true,
            optimal_move_cnt: opt_cnt,
            solution_path: path,
            arrows_to_draw: Vec::new(),
            cur_move_cnt: 0,
            choice_piece: chess::QUEEN_WHITE,
            star_cnt: 5,
            board_light_sq_color: Color32::from_rgba_premultiplied(213, 213, 213, 170),
            board_dark_sq_color: Color32::BLACK,
            auto_play: false,
            window_bg_color: Color32::BLACK,
            arrow_color: Color32::from_rgba_premultiplied(81, 171, 0, 104),
            side_panel_dark_mode: false,
            show_solution: false,
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
    (eframe::web_sys::window().expect("should have a Window").performance().expect("should have a Performance").now() / 1000.0) as u64
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
    Ok(egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice()))
}

fn get_texture<'a>(app: &'a mut MyApp, ui: &'a mut Ui, img_id: i8) -> &'a TextureHandle {
    // where to draw currently dragged image
    // insert id if it isn't there
    app.textures.entry(img_id).or_insert(None);

    app.textures.get_mut(&img_id).unwrap().get_or_insert_with(|| {
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
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Controls styles
        let mut visuals = if !self.side_panel_dark_mode {
            let mut v = egui::Visuals::light();
            v.window_shadow = egui::epaint::Shadow::small_dark();
            v
        } else {
            let mut v = egui::Visuals::dark();
            v.override_text_color = Some(Color32::from_gray(245));
            v.widgets.inactive.bg_fill = Color32::BLACK;
            v.widgets.inactive.bg_stroke = Stroke::new(0.8, Color32::WHITE);
            v
        };
        visuals.widgets.noninteractive.bg_stroke.width = 0.0;
        ctx.tessellation_options().feathering_size_in_pixels = 0.3;
        let mut style: egui::Style = (*ctx.style()).clone();
        style.spacing.indent = 11.0;
        ctx.set_style(style);
        ctx.set_visuals(visuals.clone());
        // TODO add new fonts
        if self.show_side_panel {
            egui::containers::SidePanel::left("Controls")
                .resizable(true)
                .frame(egui::containers::Frame {
                    inner_margin: egui::style::Margin::from(15.0),
                    fill: if !self.side_panel_dark_mode { Color32::WHITE } else { Color32::BLACK },
                    ..Default::default()
                })
                .show(ctx, |ui| {
                    ui.add_space(5.0);
                    ui.vertical_centered_justified(|ui| {
                        ui.heading("LiLearn");
                    });
                    ui.add_space(25.0);

                    ui.label(format!("Points: {}", self.points));

                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(format!("🔥: {}", self.streak)).color(Color32::RED).strong());
                    });

                    // show win msgs
                    if !self.in_timed_round && self.board.num_star_cnt == 0 && !self.auto_play && !self.in_game {
                        let mut msg = "You were close!";
                        let msg_color = Color32::RED;
                        if self.cur_move_cnt == self.optimal_move_cnt {
                            msg = "Excellent! 🔥🔥🔥";
                        }
                        ui.label(egui::RichText::new(msg).color(msg_color));
                    }

                    match self.last_timed_game {
                        None => (),
                        Some(v) => {
                            ui.add_space(10.0);
                            ui.label(egui::RichText::new(format!("You won {} round(s) in your last timed game", v)).color(Color32::DARK_GREEN));
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
                    }
                    egui::Grid::new("my_grid").num_columns(2).spacing([23.0, 4.0]).show(ui, |ui| {
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

                        ui.label("Side panel dark mode: ");
                        ui.checkbox(&mut self.side_panel_dark_mode, "");
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
                                let cur_time = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs();

                                #[cfg(target_arch = "wasm32")]
                                let cur_time = now_sec();

                                self.timer = cur_time;
                            } else {
                                self.timed = false;
                                self.in_timed_round = false;
                            }

                            self.in_game = true;
                            self.board = LiBoard::new(self.star_cnt as i8, self.choice_piece);
                            self.cur_move_cnt = 0;
                            (self.optimal_move_cnt, self.solution_path) = self.board.num_optimal_moves_to_star();
                            self.arrows_to_draw.clear();
                        }
                        ui.add_space(2.0);

                        if ui.button("Undo Drawing").clicked() {
                            self.arrows_to_draw.pop();
                        }
                        ui.add_space(2.0);
                        if ui.button("Clear Drawing").clicked() {
                            self.arrows_to_draw.clear()
                        }
                        ui.add_space(2.0);
                        if ui
                            .add(
                                Button::new(if self.show_solution {
                                    RichText::new("Hide Solution Path").color(Color32::WHITE).strong()
                                } else {
                                    RichText::new("Show Solution Path").color(Color32::WHITE)
                                })
                                .fill(if self.side_panel_dark_mode { Color32::DARK_RED } else { Color32::RED }),
                            )
                            .clicked()
                        {
                            self.show_solution ^= true;
                        }

                        if self.auto_play && self.board.num_star_cnt == 0 {
                            self.in_game = true;
                            self.board = LiBoard::new(self.star_cnt as i8, self.choice_piece);
                            self.cur_move_cnt = 0;
                            (self.optimal_move_cnt, self.solution_path) = self.board.num_optimal_moves_to_star();
                            self.arrows_to_draw.clear();
                        }
                    });
                });
        } else {
            let Vec2 { x, .. } = frame.info().window_info.size;
            egui::containers::SidePanel::left("Phantom")
                .exact_width(x / 6.0)
                .resizable(false)
                .frame(egui::containers::Frame {
                    inner_margin: egui::style::Margin::from(15.0),
                    fill: Color32::BLACK,
                    ..Default::default()
                })
                .show(ctx, |_ui| {});

            egui::containers::SidePanel::right("AnotherPhantom")
                .exact_width(x / 6.0)
                .resizable(false)
                .frame(egui::containers::Frame {
                    inner_margin: egui::style::Margin::from(15.0),
                    fill: Color32::BLACK,
                    ..Default::default()
                })
                .show(ctx, |_ui| {});
        }
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
                let cur_time = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs();

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
                        (self.optimal_move_cnt, self.solution_path) = self.board.num_optimal_moves_to_star();
                        self.arrows_to_draw.clear();
                    } else {
                        show_progress_bar = true;
                    }
                }
                if ui
                    .add(
                        Button::new(RichText::new(if self.show_side_panel { "Zen Mode" } else { "Menu" }))
                            .fill(self.window_bg_color)
                            .stroke(Stroke::new(0.8, Color32::WHITE)),
                    )
                    .clicked()
                {
                    self.show_side_panel ^= true;
                }

                ui.add_space(3.0);
                ui.label("Number of current moves: ".to_owned() + &self.cur_move_cnt.to_string());
                ui.add_space(3.0);
                ui.label("Optimal: ".to_owned() + &self.optimal_move_cnt.to_string());
                ui.add_space(3.0);
                if show_progress_bar {
                    ui.label(format!("Time left: {}", self.starting_timer - (cur_time - self.timer)));
                    ui.add_space(3.0);
                    let ratio_f = (cur_time - self.timer) as f32 / self.starting_timer as f32;
                    match self.board_width {
                        Some(v) => ui.add(ProgressBar::new(ratio_f).desired_width(v).desired_rounding(2.5)),
                        None => ui.add(ProgressBar::new(ratio_f).desired_rounding(2.5)),
                    };
                    ui.add_space(3.0);
                }
                ui.add_space(4.0);
                // leave some space for controls on the bottom
                let Vec2 { x, y } = ui.available_size();
                let (board_rect, _) = ui.allocate_at_least(Vec2::new(x, y - 50.0), Sense::click());
                let size = ((board_rect.max.x - board_rect.min.x) / 8.0).min((board_rect.max.y - board_rect.min.y) / 8.0); // width of square
                self.board_width = Some(size * 8.0);
                let mut piece_state = PieceStates::NoDrag;
                ui.add_space(5.0);
                for i in 0..8 {
                    for j in 0..8 {
                        let sq = Rect {
                            min: Pos2 {
                                x: j as f32 * size + board_rect.min.x,
                                y: i as f32 * size + board_rect.min.y,
                            },
                            max: Pos2 {
                                x: j as f32 * size + size + board_rect.min.x,
                                y: i as f32 * size + size + board_rect.min.y,
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
                            if cur_input_pos.is_some() && board_rect.contains(cur_input_pos.unwrap()) {
                                let a = cur_input_pos.unwrap();
                                let goal_j = (a.x - board_rect.min.x) / size;
                                let goal_i = (a.y - board_rect.min.y) / size;
                                piece_state = PieceStates::ArrowDragged(ArrowMove {
                                    start_i: i,
                                    start_j: j,
                                    end_i: goal_i as usize,
                                    end_j: goal_j as usize,
                                });
                            }
                            if piece_being_moved != 0 {
                                let texture = get_texture(self, ui, piece_being_moved);
                                // Show the image:
                                egui::Image::new(texture, texture.size_vec2()).paint_at(ui, sq);
                            }
                        } else if piece_resp.dragged_by(PointerButton::Primary) && piece_being_moved != 0 {
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
                                let image_rect = Rect { min: start_of_rec, max: end_of_rec };

                                piece_state = PieceStates::Dragged(image_rect, piece_being_moved);
                            }
                        }
                        // arrow drag released
                        else if self.secondary_clicked && piece_resp.drag_released() {
                            self.secondary_clicked = false;
                            let a = ctx.input().pointer.interact_pos();
                            if a.is_some() && board_rect.contains(a.unwrap()) {
                                let a = a.unwrap();
                                let goal_j = (a.x - board_rect.min.x) / size;
                                let goal_i = (a.y - board_rect.min.y) / size;
                                piece_state = PieceStates::ArrowDragReleased(ArrowMove {
                                    start_i: i,
                                    start_j: j,
                                    end_i: goal_i as usize,
                                    end_j: goal_j as usize,
                                });
                            }
                            if piece_being_moved != 0 {
                                let texture = get_texture(self, ui, piece_being_moved);
                                // Show the image:
                                egui::Image::new(texture, texture.size_vec2()).paint_at(ui, sq);
                            }
                        }
                        // Handle primary button drags
                        else if self.primary_clicked && piece_resp.drag_released() && piece_being_moved != STAR_VALUE {
                            self.primary_clicked = false;
                            // done dragging here.. potentially update board state for next frame
                            let a = ctx.input().pointer.interact_pos();
                            if a.is_some() && board_rect.contains(a.unwrap()) {
                                let a = a.unwrap();
                                let goal_j = (a.x - board_rect.min.x) / size;
                                let goal_i = (a.y - board_rect.min.y) / size;
                                let image_rect = Rect {
                                    min: Pos2 {
                                        x: (goal_j as i8) as f32 * size + board_rect.min.x,
                                        y: (goal_i as i8) as f32 * size + board_rect.min.y,
                                    },
                                    max: Pos2 {
                                        x: (goal_j as i8) as f32 * size + size + board_rect.min.x,
                                        y: (goal_i as i8) as f32 * size + size + board_rect.min.y,
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
                            if self.board.board[move_piece.goal_i][move_piece.goal_j] == chess::STAR_VALUE {
                                play_sound("capture");
                                self.board.num_star_cnt -= 1;
                            } else {
                                play_sound("move");
                            }
                            self.board.update_board(&move_piece);
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
                                egui::Image::new(texture, texture.size_vec2()).paint_at(ui, piece_rect);
                            }
                        }
                    }
                    PieceStates::ArrowDragReleased(arrow_move) => {
                        self.arrows_to_draw.push(arrow_move);
                    }
                    PieceStates::ArrowDragged(arrow_move) => {
                        self.draw_arrow(arrow_move.clone(), ui.painter(), size, board_rect);
                    }
                    _ => (),
                }

                // Draw arrows
                for arrow_move in &self.arrows_to_draw {
                    self.draw_arrow(arrow_move.clone(), ui.painter(), size, board_rect);
                }

                if self.show_solution {
                    for move_piece in &self.solution_path {
                        self.draw_arrow(move_piece.clone().into(), ui.painter(), size, board_rect);
                    }
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
