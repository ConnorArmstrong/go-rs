use crate::new_game::{Turn, Game};
use crate::new_board::NewBoard;
use crate::fails::TreeErrors;
use std::collections::HashMap;

pub struct GameTree {
    board_states: Vec<(Turn, NewBoard)>,
    pointer: usize,
}


impl GameTree {
    pub fn new() -> Self {
        let mut board_states = Vec::new();
        board_states.push((Turn::Pass, NewBoard::new()));

        GameTree {
            board_states,
            pointer: 0,
        }   
    }

    pub fn move_back(&mut self) -> Result<&(Turn, NewBoard), TreeErrors> {
        if self.pointer == 0 {
            return Err(TreeErrors::BelowZero);
        }
        
        self.pointer -= 1;

        Ok(&self.board_states[self.pointer])
    }

    pub fn move_forward(&mut self) -> Result<&(Turn, NewBoard), TreeErrors> {
        if self.pointer == self.board_states.len() - 1 {
            return Err(TreeErrors::AboveMax);
        }

        self.pointer += 1;

        Ok(&self.board_states[self.pointer])
    }

    
    pub fn jump(&mut self, index: usize) -> Result<&(Turn, NewBoard), TreeErrors> {
        if index >= self.board_states.len() {
            return Err(TreeErrors::AboveMax);
        }

        self.pointer = index;
        return Ok(&self.board_states[self.pointer]);
    }

    pub fn latest(&mut self) -> &(Turn, NewBoard) {
        self.pointer = self.board_states.len() - 1;

        return &self.board_states[self.pointer];
    }

    pub fn reset(&mut self) {
        self.pointer = self.board_states.len() - 1;
    }

    pub fn add_move(&mut self, turn: Turn, board: NewBoard) {
        self.reset();
        self.board_states.push((turn, board));
        self.pointer += 1;
    }

}