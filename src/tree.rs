use crate::board_state::BoardState;
use crate::colour::Colour;
use crate::coordinate::Coordinate;
use crate::fails::TreeErrors;
use crate::turn::Turn;

pub struct GameTree {
    board_states: Vec<(Turn, BoardState)>,
    pointer: usize,
}


impl GameTree {
    pub fn new(size: usize) -> Self {
        let mut board_states = Vec::new();
        board_states.push((Turn::Pass, BoardState::new(size)));

        GameTree {
            board_states,
            pointer: 0,
        }   
    }

    pub fn move_back(&mut self) -> Result<&(Turn, BoardState), TreeErrors> {
        if self.pointer == 0 {
            return Err(TreeErrors::BelowZero);
        }
        
        self.pointer -= 1;

        Ok(&self.board_states[self.pointer])
    }

    pub fn move_forward(&mut self) -> Result<&(Turn, BoardState), TreeErrors> {
        if self.pointer == self.board_states.len() - 1 {
            return Err(TreeErrors::AboveMax);
        }

        self.pointer += 1;

        Ok(&self.board_states[self.pointer])
    }

    
    pub fn _jump(&mut self, index: usize) -> Result<&(Turn, BoardState), TreeErrors> {
        if index >= self.board_states.len() {
            return Err(TreeErrors::AboveMax);
        }

        self.pointer = index;
        return Ok(&self.board_states[self.pointer]);
    }

    pub fn _latest(&mut self) -> &(Turn, BoardState) {
        self.pointer = self.board_states.len() - 1;

        return &self.board_states[self.pointer];
    }

    pub fn reset(&mut self) {
        self.pointer = self.board_states.len() - 1;
    }

    pub fn add_move(&mut self, turn: Turn, board: BoardState) {
        self.reset();
        self.board_states.push((turn, board));
        self.pointer += 1;
    }

    pub fn get_pointer(&self) -> usize {
        self.pointer
    }

    pub fn get_length(&self) -> usize {
        self.board_states.len() - 1
    }

    /// returns true if the game is over by resignation or agreement
    pub fn check_end(&self) -> bool {
        if self.board_states.len() < 3 {
            return false;
        }
    
        match self.board_states.last().unwrap() {
            (Turn::Resign, _) => true,
            _ => {
                if let Some((first_last, _)) = self.board_states.get(self.board_states.len() - 2) {
                    if let Some((first_second_last, _)) = self.board_states.last() {
                        match (first_last, first_second_last) {
                            (Turn::Pass, Turn::Pass) => true,
                            _ => false,
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
        }
    }

    /// returns true if the pointer is on the current position
    pub fn up_to_date(&self) -> bool {
        self.pointer == self.board_states.len() - 1
    }

    pub fn get_board(&self) -> (Colour, BoardState) {
        let (_, board) = &self.board_states[self.pointer];

        let colour;

        if self.pointer % 2 == 0 {
            colour = Colour::Black;
        } else {
            colour = Colour::White;
        }

        (colour, board.clone())
    }

    /// Returns the last move if it is not a pass or resignation
    pub fn get_last_move(&self) -> Option<Coordinate> {
        let data = self.board_states[self.pointer].0;

        match data {
            Turn::Move(coordinate) => Some(coordinate),
            _ => None
        }
    }
}