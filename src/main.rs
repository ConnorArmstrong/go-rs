#![allow(unused)]

use crate::board::{Board, BOARD_SIZE, Colour};
use crate::coordinate::Coordinate;
use pancurses::{initscr, endwin, Input, noecho, Window};

mod board;
mod coordinate;
mod group;
mod game;

fn main() {
    println!("Hello, world!");
    let mut board = Board::new(BOARD_SIZE);
    board.add_stone(Coordinate::Position((10, 10)), Colour::Black);
    board.add_stone(Coordinate::Position((10, 11)), Colour::White);
    board.add_stone(Coordinate::Position((11, 11)), Colour::Black);
    //board.display_board();
    
    for group in &board.groups {
        println!("{:?}", group.get_liberties());
    }

    let mut pointer_pos: (usize, usize) = (9, 9);

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
            Some(Input::Character(' ')) => {board.add_stone(Coordinate::Position(pointer_pos), Colour::Black);
                                            println!("{:?}", board.groups.len())},
            Some(Input::Character('q')) => break, // Exit loop if 'q' is pressed
            _ => {}
        }
    }
    
    endwin();
}


fn print_board(window: &Window, board: &Board, pointer_pos: &(usize, usize)) {
    for y in 0..board.size {
        for x in 0..board.size {
            let coordinate = Coordinate::Position((x, y));
            match board.get(coordinate) {
                Colour::Empty => window.addch('.'),
                Colour::Black => window.addch('x'),
                Colour::White => window.addch('o'),
            };
            //window.addch(' ');
        }
        window.addch('\n');
    }
    window.mv(pointer_pos.1 as i32, pointer_pos.0 as i32);
    window.refresh();
}