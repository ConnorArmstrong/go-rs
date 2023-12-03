use eframe::{egui, App, Frame, NativeOptions};
use egui::Id;

use crate::{colour, BOARD_SIZE};
use crate:: coordinate::Coordinate;
use crate::game_state::GameState;
use crate::turn::Turn;

struct MyApp {
    game: GameState,
}

impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        let MyApp {game: _} = self;

        let (turn, boardstate) = self.game.game_tree.get_board();

        let grid_state = boardstate.get_grid().clone();

        let move_string = turn.get_string() + " to play.";
        let turn_string = format!("{}/{}", self.game.game_tree.get_pointer(), self.game.game_tree.get_length());

        egui::CentralPanel::default().show(ctx, |ui| {
            // Calculate the size of each cell in the grid
            let cell_size = ui.available_size_before_wrap().x / self.game.size as f32;
            let go_board_rect = ui.min_rect().expand(2.0);
            let response = ui.interact(go_board_rect, Id::new("go board"), egui::Sense::click());

            // Draw the Go board
            ui.allocate_ui(egui::vec2(cell_size * self.game.size as f32, cell_size * self.game.size as f32), |ui| {
                let mut shapes = Vec::new();

                // Draw grid lines
                for i in 0..self.game.size {
                    let x = i as f32 * cell_size + cell_size / 2.0;
                    let y = i as f32 * cell_size + cell_size / 2.0;
                
                    // Horizontal lines
                    shapes.push(egui::Shape::line_segment(
                        [egui::pos2(cell_size / 2.0, y), egui::pos2(cell_size * (self.game.size as f32 - 0.5), y)],
                        egui::Stroke::new(1.0, egui::Color32::BLACK),
                    ));
                
                    // Vertical lines
                    shapes.push(egui::Shape::line_segment(
                        [egui::pos2(x, cell_size / 2.0), egui::pos2(x, cell_size * (self.game.size as f32 - 0.5))],
                        egui::Stroke::new(1.0, egui::Color32::BLACK),
                    ));
                }

                // Draw stones here!
                for i in 0..self.game.size {
                    for j in 0..self.game.size {
                        let color = match &grid_state[Coordinate::Position((i, j)).get_index()] {
                            colour::Colour::White => egui::Color32::WHITE,
                            colour::Colour::Black => egui::Color32::BLACK,
                            colour::Colour::Empty => continue, // Skip empty positions
                        };
                
                        let center = egui::pos2(
                            j as f32 * cell_size + cell_size / 2.0,
                            i as f32 * cell_size + cell_size / 2.0,
                        );
                        shapes.push(egui::Shape::circle_stroke(center, cell_size / 2.2, egui::Stroke::new(1.5, egui::Color32::BLACK))); // black outline
                        shapes.push(egui::Shape::circle_filled(center, cell_size / 2.25, color));
                    }
                }

                ui.painter().extend(shapes);
            });

            if response.clicked() {

                let y_pos = (response.interact_pointer_pos().unwrap().y - response.rect.min.y).max(0.0);
                let x_pos = (response.interact_pointer_pos().unwrap().x - response.rect.min.x).max(0.0);
   
                let i = (y_pos / cell_size).floor() as usize;
                let j = (x_pos / cell_size).floor() as usize;

                let coords = self.game.clamp_coordinate(i, j);

                self.game.play_move(coords);
            }

            if response.secondary_clicked() {
                let y_pos = (response.interact_pointer_pos().unwrap().y - response.rect.min.y).max(0.0);
                let x_pos = (response.interact_pointer_pos().unwrap().x - response.rect.min.x).max(0.0);
   
                let i = (y_pos / cell_size).floor() as usize;
                let j = (x_pos / cell_size).floor() as usize;
            
                let coords = self.game.clamp_coordinate(i, j);

                self.game.play_move(coords);
                self.game.random_game();
            }

            if response.middle_clicked() {
                let y_pos = (response.interact_pointer_pos().unwrap().y - response.rect.min.y).max(0.0);
                let x_pos = (response.interact_pointer_pos().unwrap().x - response.rect.min.x).max(0.0);
   
                let i = (y_pos / cell_size).floor() as usize;
                let j = (x_pos / cell_size).floor() as usize;
            
                let coords = self.game.clamp_coordinate(i, j);

                self.game.board_state.debug_selection(coords);
            }

                       
            ui.heading(move_string);
            ui.weak(turn_string);

        });

        ctx.input(|i| {
            let direction = i.scroll_delta.y; // check for vertical scroll

            if direction > 0.0 {
                self.game.jump_back();
            } else if direction < 0.0 {
                self.game.jump_forward();
            }

            if i.key_pressed(egui::Key::A) {
                self.game.play_turn(Turn::Pass);
            }

            if i.key_pressed(egui::Key::R) {
                self.game.play_turn(Turn::Resign);
            }

            if i.key_pressed(egui::Key::P) {
                let value = self.game.calculate_total_completed_score();
                println!("{:?}", value);
            }

            if i.key_pressed(egui::Key::Space) {
                println!("{:?}", self.game.board_state.check_all_important_points_played());
            }
        });
    }
}

pub fn run() -> Result<(), eframe::Error> {
    let app = MyApp {
        game: GameState::new(BOARD_SIZE),
    };

    let native_options = NativeOptions {
        initial_window_size: Some(egui::vec2(450.0, 450.0)),
        ..Default::default()
    };
    eframe::run_native("Go.", native_options, Box::new(|_cc| Box::new(app)))
}
