use eframe::{egui, App, Frame, NativeOptions};
use egui::{Rect, Id};
use pancurses::reset_prog_mode;

use crate::{board::{self, Colour}, coordinate::Coordinate};
use crate::new_board::NewBoard;

struct MyApp {
    board: NewBoard,
    turn_to_play: Colour,
}

impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        let MyApp {board, turn_to_play} = self;

        let grid_state = self.board.get_grid().clone();
        let move_string = self.turn_to_play.get_string() + " to play.";


        egui::CentralPanel::default().show(ctx, |ui| {
            

            //ui.heading(&move_string);

            // Calculate the size of each cell in the grid
            let cell_size = ui.available_size_before_wrap().x / 18.5;
            let go_board_rect = ui.min_rect().expand(2.0);
            let response = ui.interact(go_board_rect, Id::new("go board"), egui::Sense::click());

            // Draw the Go board
            ui.allocate_ui(egui::vec2(cell_size * 19.0, cell_size * 19.0), |ui| {
                let mut shapes = Vec::new();

                // Draw grid lines
                for i in 0..19 {
                    let x = i as f32 * cell_size + cell_size / 2.0;
                    let y = i as f32 * cell_size + cell_size / 2.0;

                    shapes.push(egui::Shape::line_segment(
                        [egui::pos2(x, cell_size / 2.0), egui::pos2(x, cell_size * 18.5)],
                        egui::Stroke::new(1.0, egui::Color32::BLACK),
                    ));

                    shapes.push(egui::Shape::line_segment(
                        [egui::pos2(cell_size / 2.0, y), egui::pos2(cell_size * 18.5, y)],
                        egui::Stroke::new(1.0, egui::Color32::BLACK),
                    ));
                }

                

                // Draw stones here!
                for i in 0..19 {
                    for j in 0..19 {
                        let color = match &grid_state[Coordinate::Position((i, j)).get_index()] {
                            board::Colour::White => egui::Color32::WHITE,
                            board::Colour::Black => egui::Color32::BLACK,
                            board::Colour::Empty => continue, // Skip empty positions
                        };
                
                        let center = egui::pos2(
                            j as f32 * cell_size + cell_size / 2.0,
                            i as f32 * cell_size + cell_size / 2.0,
                        );
                        shapes.push(egui::Shape::circle_stroke(center, cell_size / 2.2, egui::Stroke::new(1.5, egui::Color32::BLACK))); // black outline
                        shapes.push(egui::Shape::circle_filled(center, cell_size / 2.25, color));
                    }
                }
                //

                ui.painter().extend(shapes);
            });

            if response.clicked() {

                let y_pos = (response.interact_pointer_pos().unwrap().y - response.rect.min.y).max(0.0);
                let x_pos = (response.interact_pointer_pos().unwrap().x - response.rect.min.x).max(0.0);
   
                let i = (y_pos / cell_size).floor() as usize;
                let j = (x_pos / cell_size).floor() as usize;

                println!("---    Clicked at position {} {}    ---", i, j);
            
                let coords = Coordinate::Position((i, j));

                // Now you can update the state of the board at the clicked cell (i, j):
                self.board.add_stone(coords, Colour::Black);
                //self.turn_to_play = self.turn_to_play.swap_turn();
            }

            if response.secondary_clicked() {
                let y_pos = (response.interact_pointer_pos().unwrap().y - response.rect.min.y).max(0.0);
                let x_pos = (response.interact_pointer_pos().unwrap().x - response.rect.min.x).max(0.0);
   
                let i = (y_pos / cell_size).floor() as usize;
                let j = (x_pos / cell_size).floor() as usize;

                println!("Clicked at position {} {}", i, j);
            
                let coords = Coordinate::Position((i, j));

                // Now you can update the state of the board at the clicked cell (i, j):
                self.board.add_stone(coords, Colour::White);
            }

            
            ui.heading(move_string);
        });
    }
}

pub fn run() -> Result<(), eframe::Error> {
    let app = MyApp {
        board: NewBoard::new(),
        turn_to_play: Colour::Black,
    };

    let native_options = NativeOptions {
        initial_window_size: Some(egui::vec2(450.0, 450.0)),
        ..Default::default()
    };
    
    eframe::run_native("Go.", native_options, Box::new(|cc| Box::new(app)))
}