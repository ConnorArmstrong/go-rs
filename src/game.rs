#![allow(unused)]

use std::collections::HashSet;

use crate::{board::{Board, Colour}, coordinate::Coordinate};

pub enum Turn {
    Move(Coordinate), // move a stone (coordinate could either be the position or index)
    Pass, // 2 passes and the game is over
    Resign, // resign the game
}

pub struct Move {
    // might change this later to better allow for a move tree to be made
    move_number: usize, // what turn it is ie, move 1, move 2, et
    colour: Colour, // which player it is
    turn: Turn, // the actual move that is played
}


pub struct Game { // the actual game logic required
    board: Board, 
    turn: Colour, // swapping Black -> White -> Black etc
    visited_positions: HashSet<Board>, // keep track of visited positions for ko's -> SEE ZOBRIST HASHING!
}