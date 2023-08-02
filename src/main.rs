#![allow(unused, dead_code, non_camel_case_types)]

use std::collections::HashSet;
use group::Group;

mod group;
//

pub const BOARD_SIZE: usize = 19; // for now

fn main() {
    println!("Hello, world!");
    let board = Board::new(BOARD_SIZE);
}

type position = (usize, usize); // (x, y)
type index = usize; // index of the 1d vector



#[derive(Clone, Copy, Debug, PartialEq, Hash, Eq)]
pub enum Coordinate { // to make it easier to handle the position of the board and transfer between the two formats
    Position(position),
    Index(index),
}

impl Coordinate {
    pub fn into(&self) -> Coordinate {
        // convert position to index, or index to position
        match self {
            Coordinate::Index(value) => {
                // return position
               Coordinate::Position((value / BOARD_SIZE, value % BOARD_SIZE))
            }
            Coordinate::Position((x, y)) => {
                // return index
                Coordinate::Index(x * BOARD_SIZE + y)
            }
        }
    }

    pub fn into_index(&self) -> Coordinate {
        match self {
            Coordinate::Index(value) => {*self},
            Coordinate::Position((x, y)) => {self.into()}
        }
    }

    pub fn new_index(index: usize) -> Option<Coordinate> {
        // returns an index if it is inbounds
        if (index < (BOARD_SIZE * BOARD_SIZE)) {
           return Some(Coordinate::Index(index))
        }
        return None
    }

    pub fn new_position(position: position) -> Option<Coordinate> {
        if (position.0 < BOARD_SIZE && position.1 < BOARD_SIZE) {
            return Some(Coordinate::Position(position))
        }
        return None
    }
}

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


#[derive(Clone, Copy, Debug, PartialEq, Hash, Eq)]
pub enum Colour {
    Empty, // empty space on the board
    Black,
    White,
}

impl Colour {
    pub fn swap_turn(&self) -> Colour {
        // swap the turn of the player
        match self {
            Colour::Black => Colour::White,
            Colour::White => Colour::Black,
            Colour::Empty => Colour::Empty, // realistically shouldnt happen
        }
    }
}


#[derive(Clone, Debug, PartialEq)]
pub struct Board {
    size: usize, // most typically 19x19 - will start smaller for ai
    grid: Vec<Colour>, // 1D vector that stores the colours of the stones on the board
    groups: Vec<Group>, // list of groups on the board
}

impl Board {
    pub fn new(size: usize) -> Self {
        Board {
            size,
            grid: vec![Colour::Empty; size * size],
            groups: Vec::new(),
        }
    }

    pub fn get(&self, coordinate: Coordinate) -> Colour {
        match coordinate.into_index() {
            Coordinate::Position(_) => { 
                println!("Error: Position not supported");
                Colour::Empty
            }, // not reachable
            Coordinate::Index(value) => { self.grid[value]}
        }
    }
}

pub struct Game { // the actual game logic required
    board: Board, 
    turn: Colour, // swapping Black -> White -> Black etc
    visited_positions: HashSet<Board>, // keep track of visited positions for ko's -> SEE ZOBRIST HASHING!
    white_points: usize, // how many stones white has captured
    black_points: usize, // how many stones black has captured
}