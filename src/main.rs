#![allow(unused, dead_code, non_camel_case_types)]

use std::collections::HashSet;
use coordinate::Coordinate;
use group::Group;

mod coordinate;
mod group;
mod tree;
//

pub const BOARD_SIZE: usize = 19; // for now

fn main() {
    println!("Hello, world!");
    let mut board = Board::new(BOARD_SIZE);
}

type position = (usize, usize); // (x, y)
type index = usize; // index of the 1d vector


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
    white_points: usize, // how many stones white has captured
    black_points: usize, // how many stones black has captured
}

impl Board {
    pub fn new(size: usize) -> Self {
        Board {
            size,
            grid: vec![Colour::Empty; size * size],
            groups: Vec::new(),
            white_points: 0,
            black_points: 0,
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

    pub fn add_stone(&mut self, coordinate: Coordinate, colour: Colour) {
        // adds a stone to the position WITHOUT CHECKING FOR VALIDITY

    }

    pub fn remove_group(&mut self, group: Group) {
        // removes a group from the board
        // this is used when a group is captured
        let positions = group.get_positions();
        let mut count = 0;

        for position in positions {
            let index = position.get_index();
            self.grid[index] = Colour::Empty;
            count += 1;
        }

        match group.get_colour() {
            Colour::Black => self.white_points += count,
            Colour::White => self.black_points += count,
            _ => {},
        }
    }

    pub fn get_adjacent_indices(&self, position: Coordinate) -> Vec<Coordinate> {
        let (x, y) = position.get_position();
        let mut indices = Vec::new();

        if x > 0 {
            indices.push(Coordinate::Index((x - 1) * self.size + y)); // Top
        }
        if y > 0 {
            indices.push(Coordinate::Index(x * self.size + y - 1)); // Left
        }
        if x < self.size - 1 {
            indices.push(Coordinate::Index((x + 1) * self.size + y)); // Bottom
        }
        if y < self.size - 1 {
            indices.push(Coordinate::Index(x * self.size + y + 1)); // Right
        }
        indices
    }
}

pub struct Game { // the actual game logic required
    board: Board, 
    turn: Colour, // swapping Black -> White -> Black etc
    visited_positions: HashSet<Board>, // keep track of visited positions for ko's -> SEE ZOBRIST HASHING!

}