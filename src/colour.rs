
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

    pub fn get_string(&self) -> String {
        match self {
            Colour::Black => String::from("Black"),
            Colour::White => String::from("White"),
            Colour::Empty => String::from("Empty"),
        }
    }
}