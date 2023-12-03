use crate::coordinate::Coordinate;

pub enum Turn {
    Move(Coordinate), // move a stone (coordinate could either be the position or index)
    Pass, // 2 passes and the game is over
    Resign, // resign the game
}

impl Turn {
    pub fn handle_turn(turn: Turn) -> Option<Coordinate> {
        match turn {
            Turn::Move(coordinate) => Some(coordinate),
            _ => None,
        }
    }

    pub fn into_turn(coordinate: Coordinate) -> Turn {
        Turn::Move(coordinate)
    }
}
