#![allow(non_camel_case_types)]

use std::hash::Hash;

use crate::BOARD_SIZE;


pub type position = (usize, usize); // (x, y)
pub type index = usize; // index of the 1d vector

#[derive(Clone, Copy, Debug)]
pub enum Coordinate { // to make it easier to handle the position of the board and transfer between the two formats
    Position(position),
    Index(index),
}

impl Coordinate {
    /// convert position to index, or index to position
    pub fn into(&self) -> Coordinate {
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
            Coordinate::Index(_value) => {*self},
            Coordinate::Position((_x, _y)) => {self.into()}
        }
    }

    pub fn into_position(&self) -> Coordinate {
        match self {
            Coordinate::Index(_value) => {self.into()},
            Coordinate::Position((_x, _y)) => {*self}
        }
    }

    /// returns an index if it is inbounds
    pub fn _new_index(index: usize) -> Option<Coordinate> {

        if index < (BOARD_SIZE * BOARD_SIZE) {
           return Some(Coordinate::Index(index))
        }
        return None
    }

    /// returns a Position if it is inbounds
    pub fn _new_position(position: position) -> Option<Coordinate> {
        if position.0 < BOARD_SIZE && position.1 < BOARD_SIZE {
            return Some(Coordinate::Position(position))
        }
        return None
    }

    /// returns the Index of the Coordinate
    pub fn get_index(&self) -> index {
        let new = self.into_index();
        match new {
            Coordinate::Index(value) => {value},
            Coordinate::Position(_) => 0,
        }
    }

    /// Returns the Position of the Coordinate
    pub fn get_position(&self) -> position {
        let new = self.into_position();
        match new  {
            Coordinate::Index(_) => {(0, 0)},
            Coordinate::Position(value) => {value},
        }
    }
}

// position and index are equivalent if they point to the same location in a 1d vector
impl PartialEq for Coordinate {
    fn eq(&self, other: &Self) -> bool {
        return self.get_index() == other.get_index()
    }
}

impl Eq for Coordinate {}

impl Hash for Coordinate {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.get_index().hash(state);
    }
}

