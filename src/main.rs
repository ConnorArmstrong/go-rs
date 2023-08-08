#![allow(unused)]

use crate::board::{Board, BOARD_SIZE, Colour};
use crate::coordinate::Coordinate;

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
    board.display_board();
    
    for group in board.groups {
        println!("{:?}", group.get_liberties());
    }
}