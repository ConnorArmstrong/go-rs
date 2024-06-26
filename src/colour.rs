
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

    pub fn into_usize(&self) -> usize {
        match self {
            Colour::Black => 1,
            Colour::White => 2,
            Colour::Empty => 0,
        }
    }
}

pub enum Outcome {
    BlackWin,
    WhiteWin,
    _Draw, // Not possible with komi
}

impl Outcome {
    pub fn _get_string(&self) -> String {
        match self {
            Outcome::BlackWin => String::from("Black Wins"),
            Outcome::WhiteWin => String::from("White Wins"),
            Outcome::_Draw => String::from("Draw"),
        }
    }

    pub fn _determine(black_score: usize, white_score: usize) -> Outcome {
        if black_score > white_score {
            Outcome::BlackWin
        } else if white_score > black_score {
            Outcome::WhiteWin
        } else {
            Outcome::_Draw
        }
    }

    pub fn into_colour(&self) -> Colour {
        match self {
            Outcome::BlackWin => Colour::Black,
            Outcome::WhiteWin => Colour::White,
            Outcome::_Draw => Colour::Empty,
        }
    }
}