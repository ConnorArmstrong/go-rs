use crate::board::BOARD_SIZE;
use crate::board::{index, position};
use std::hash::Hash;


#[derive(Clone, Copy, Debug)]
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

    pub fn new_index(index: usize) -> Option<Coordinate> {
        // returns an index if it is inbounds
        if index < (BOARD_SIZE * BOARD_SIZE) {
           return Some(Coordinate::Index(index))
        }
        return None
    }

    pub fn new_position(position: position) -> Option<Coordinate> {
        if position.0 < BOARD_SIZE && position.1 < BOARD_SIZE {
            return Some(Coordinate::Position(position))
        }
        return None
    }

    pub fn get_index(&self) -> index {
        let new = self.into_index();
        match new {
            Coordinate::Index(value) => {value},
            Coordinate::Position(_) => 0,
        }
    }

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

