#![allow(unused)]
//#![windows_subsystem = "windows"]

use crate::coordinate::Coordinate;
use crate::colour::Colour;
use pancurses::{initscr, endwin, Input, noecho, Window};

mod coordinate;

mod graphics;
mod colour;
mod fails;
mod tree;
mod zobrist;
mod board_state;
mod game_state;
mod group_state;
mod turn;


const BOARD_SIZE: usize = 9;

fn main() {
    println!("running...");
    graphics::run().unwrap();
}

/*
fn print_board(window: &Window, board: &NewBoard, pointer_pos: &(usize, usize)) {
    for y in 0..BOARD_SIZE{
        for x in 0..BOARD_SIZE {
            let coordinate = Coordinate::Position((x, y));
            match board.get(coordinate) {
                Colour::Empty => window.addch('.'),
                Colour::Black => window.addch('x'),
                Colour::White => window.addch('o'),
            };
            window.addch(' ');
        }
        window.addch('\n');
    }
    window.mv(pointer_pos.1 as i32, (pointer_pos.0 * 2) as i32);
    window.refresh();
}


pub fn run_printed() {
    println!("Hello, world!");
    let mut board = NewBoard::new();
    board.add_stone(Coordinate::Position((10, 10)), Colour::Black);
    board.add_stone(Coordinate::Position((10, 11)), Colour::White);
    board.add_stone(Coordinate::Position((11, 11)), Colour::Black);
    
    //board.display_board();
    
    let mut pointer_pos: (usize, usize) = (9, 9);
    let mut colour: Colour = Colour::Black;

    let window = initscr();
    window.refresh();
    window.keypad(true);
    noecho();

    print_board(&window, &board, &pointer_pos);
    
    //window.getch();  // Wait for a key press
    //endwin();
    loop {
        window.clear(); // Clear previous frame
        print_board(&window, &board, &pointer_pos);
        match window.getch() {
            Some(Input::KeyRight) => pointer_pos.0 = (pointer_pos.0 + 1).min(BOARD_SIZE - 1),
            Some(Input::KeyLeft) => pointer_pos.0 = (pointer_pos.0 as isize - 1).max(0) as usize,
            Some(Input::KeyUp) => pointer_pos.1 = (pointer_pos.1 as isize - 1).max(0) as usize,
            Some(Input::KeyDown) => pointer_pos.1 = (pointer_pos.1 + 1).min(BOARD_SIZE - 1),
            Some(Input::Character(' ')) => {board.add_stone(Coordinate::Position(pointer_pos), colour);
                colour = colour.swap_turn();
                println!("NEW TURN.")},
            Some(Input::Character('q')) => break, // Exit loop if 'q' is pressed
            _ => {}
        }
    }
    
    endwin();
}
*/