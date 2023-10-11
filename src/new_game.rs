#![allow(unused)]

use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use std::thread::{Thread, self};
use std::time::Duration;

use rand::rngs::ThreadRng;
use rand::{thread_rng, Rng};

use rayon::prelude::*;

use crate::coordinate::Coordinate;
use crate::fails::TurnErrors;
use crate::new_board::NewBoard;
use crate::colour::Colour;
use crate::tree::GameTree;
use crate::zobrist::ZobristTable;

pub const BOARD_SIZE: usize = 19;
pub const AUTO_PLAY: bool = true;

pub enum Turn {
    Move(Coordinate), // move a stone (coordinate could either be the position or index)
    Pass, // 2 passes and the game is over
    Resign, // resign the game
}

pub struct Game { // the actual game logic required
    pub board: NewBoard, 
    pub turn: Colour, // swapping Black -> White -> Black etc
    pub zobrist: ZobristTable,
    pub game_tree: GameTree,
    rng: RefCell<ThreadRng>, // in case randomness is needed
}


impl Game {
    pub fn new() -> Self {
        Game {
            board: NewBoard::new(),
            turn: Colour::Black,
            zobrist: ZobristTable::new(),
            game_tree: GameTree::new(),
            rng: RefCell::new(thread_rng())
        }
    }

    pub fn swap_turn(&mut self) {
        self.turn = self.turn.swap_turn();
    }

    pub fn play_move(&mut self, coordinate: Coordinate) -> bool {
        let original = self.board.clone(); // "save" the currrent state
        let mut state = self.board.add_stone(coordinate, self.turn); // update the new state

        if self.zobrist.contains_position(&self.board.get_grid()) && state.is_ok() { // results in a previous board position
            state = Err(TurnErrors::Ko); //TODO: somehow include this in the board position - maybe Some(ZobristHash?)
        }

        //println!("{:?}", state);

        match state { // see if the new state results in an error
            Ok(_state) => {
                self.swap_turn();
            }
            Err(error) => {
                self.board = original;

                match error {
                    TurnErrors::AlreadyPlaced => println!("Stone already in the position"),
                    TurnErrors::Ko => println!("Can't place due to Ko"),
                    TurnErrors::Suicide => println!("Can't place due to suicide"),
                    TurnErrors::OutofBounds => println!("Can't place due to out of bounds"),
                }

                return false;
            }
        }

        self.zobrist.insert_position(&self.board.get_grid());
        
        if AUTO_PLAY {
            self.auto_move();
        }

        true
    }

    pub fn play_turn(&mut self, turn: Turn) {
        match turn {
            Turn::Move(coordinate) => {
                self.play_move(coordinate);
            },
            Turn::Pass => {
                self.swap_turn();
                self.game_tree.add_move(turn, self.board.clone())
            },
            Turn::Resign => {
                self.swap_turn(); // for now
            },
        }
    }

    pub fn get_all_possible_moves(current_position: &NewBoard, colour: Colour) -> Vec<Coordinate> {
        // return all possible moves for a given colour
        // brute force: for every position check if making a move results in a valid state
        // if it does, add it to a vector
        let possible_moves = Arc::new(Mutex::new(Vec::new()));

        (0..BOARD_SIZE * BOARD_SIZE).into_par_iter().for_each(|i| { // rayon for loop paralellisation
            let mut original_position = current_position.clone();
            let coordinate = Coordinate::Index(i);

            let state = original_position.add_stone(coordinate, colour);

            match state {
                Ok(_state) => {
                    let mut moves = possible_moves.lock().unwrap();
                    moves.push(coordinate);
                }
                Err(_error) => {}, // cannot for whatever reason (suicide, out of bounds, non-empty) so we skip
            }
        });
        Arc::try_unwrap(possible_moves).unwrap().into_inner().unwrap()
    }

    pub fn play_random_move(&self, possible_moves: &Vec<Coordinate>) -> Coordinate {
        if possible_moves.len() == 0 {
            println!("No possible moves!");
            return Coordinate::Index(BOARD_SIZE * BOARD_SIZE + 1); // OOB
        }
        let mut rng = self.rng.borrow_mut();
        let index: usize = rng.gen_range(0..possible_moves.len()); // chosen position
        possible_moves[index]
    }

    pub fn moves_to_play(board_state: &NewBoard) -> bool {
        let black_moves = Game::get_all_possible_moves(board_state, Colour::Black).len() > 0;
        let white_moves = Game::get_all_possible_moves(board_state, Colour::White).len() > 0;

        black_moves || white_moves
    }

    pub fn random_game(&mut self) {
        println!("Playing random game...");
        while Game::moves_to_play(&self.board) {
            let possible_moves = Game::get_all_possible_moves(&self.board, self.turn);
            if possible_moves.len() == 0 {
                self.play_turn(Turn::Pass);
            }
            else if possible_moves.len() > 5 { // play a move
                self.play_turn(Turn::Move(self.play_random_move(&possible_moves)));
            } else {
                break;
            }
        }

        println!("Game over!");
    }

    pub fn auto_move(&mut self) { // plays a random move if possible
        let possible_moves = Game::get_all_possible_moves(&self.board, self.turn);
        if possible_moves.len() == 0 { // no longer possible to play a move
            return;
        }

        let random_move = self.play_random_move(&possible_moves);
        let _ = self.board.add_stone(random_move, self.turn);
        self.zobrist.insert_position(&self.board.get_grid());
        self.swap_turn();
    }
}

impl Game { // scary functions 0.o
    pub fn _get_count(&self) -> (f64, f64) {
        todo!()
    }

    pub fn _get_winner(&self) -> Colour {
        todo!()
    }

    pub fn _remove_dead_groups(&mut self) {
        todo!()
    }
}

