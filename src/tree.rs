
use std::rc::Rc;
use std::cell::RefCell;

use crate::Move;

pub struct MoveTree {
    turn_number: usize,
    turn: Move,
    children: Vec<Rc<RefCell<MoveTree>>>,
}

impl MoveTree {
    pub fn new(turn: Move) -> Self {
        MoveTree {
            turn_number: turn.move_number,
            turn,
            children: Vec::new(),
        }
    }

    pub fn add_child(&mut self, child: Rc<RefCell<MoveTree>>) {
        self.children.push(child);
    }
}