use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use std::thread::{Thread, self};
use std::time::Duration;

use rand::Rng;
use rand::rngs::ThreadRng;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};

use crate::{board_state::BoardState, colour::Colour, tree::GameTree, coordinate::Coordinate, fails::TurnErrors, turn::Turn};

pub const AUTO_PLAY: bool = false;

pub struct GameState {
    pub board_state: BoardState,
    pub turn: Colour,
    pub game_tree: GameTree,
    rng: RefCell<ThreadRng>,
    pub size: usize,
}

impl GameState {
    pub fn new(board_size: usize) -> Self {
        GameState {
            board_state: BoardState::new(board_size),
            turn: Colour::Black,
            game_tree: GameTree::new(board_size),
            rng: RefCell::new(rand::thread_rng()),
            size: board_size,
        }
    }


    /// Swap the colour
    pub fn swap_turn(&mut self) {
        self.turn = self.turn.swap_turn();
    }

    /// Attempts to play a move at the specified position and returns whether it was successful
    pub fn play_move(&mut self, coordinate: Coordinate) -> bool {
        let new_position = self.board_state.add_stone(coordinate, self.turn);
        if !self.game_tree.up_to_date() {
            println!("Please reach the current position before playing a move");
            return false;
        }


        match new_position {
            Ok(state) => {
                self.board_state = state;
                self.swap_turn();
                //self.game_tree.add_move(coordinate, self.turn);
            }

            Err(error) =>{
                match error {
                    TurnErrors::AlreadyPlaced => println!("Stone already in the position"),
                    TurnErrors::Ko => println!("Can't place due to Ko"),
                    TurnErrors::Suicide => println!("Can't place due to suicide"),
                    TurnErrors::OutofBounds => println!("Can't place due to out of bounds"),
                }

                return false;
            }
        }

        if AUTO_PLAY {
            self.auto_move();
        }

        return true;
    }

    /// handles all Turn Enum Arms: Move, Pass and Resign
    pub fn play_turn(&mut self, turn: Turn) {
        match turn {
            Turn::Move(coordinate) => {
                if self.play_move(coordinate) {
                    self.game_tree.add_move(turn, self.board_state.clone());
                }
            },
            Turn::Pass => {
                self.swap_turn();
                self.game_tree.add_move(turn, self.board_state.clone());
            },
            Turn::Resign => {
                self.game_tree.add_move(turn, self.board_state.clone())
            }
        }
    }

    /// returns a list of all possible moves for a given colour via brute force
    pub fn get_all_possible_moves(&self, colour: Colour) -> Vec<Coordinate> {
        let possible_moves = Arc::new(Mutex::new(Vec::new()));
        let current_state = self.board_state.clone();

        (0..self.size * self.size).into_par_iter().for_each(|i| {
            let coordinate = Coordinate::Index(i);
            let new_position = current_state.add_stone(coordinate, colour);

            match new_position {
                Ok(_state) => {
                    let mut moves = possible_moves.lock().unwrap();
                    moves.push(coordinate);
                }
                Err(_) => {} // an error placing in that position so skip
            }
        });

        Arc::try_unwrap(possible_moves).unwrap().into_inner().unwrap()
    }

    /// Chooses a random coordinate from the possible coordinate
    pub fn play_random_move(&self, possible_moves: &Vec<Coordinate>) -> Coordinate {
        if possible_moves.is_empty() {
            println!("No possible moves");
            return Coordinate::Index(self.size * self.size + 1);
        }

        let mut rng = self.rng.borrow_mut();
        let index: usize = rng.gen_range(0..possible_moves.len()); // chosen position
        possible_moves[index]
    }

    /// Returns true if either black or white have possible moves to play
    pub fn moves_to_play(&self) -> bool {
        let black_moves = self.get_all_possible_moves(Colour::Black).len() > 0;
        let white_moves = self.get_all_possible_moves(Colour::White).len() > 0;

        black_moves || white_moves

    }

    pub fn random_game(&mut self) {
        println!("Playing random game");
        while !self.check_end() {
            let possible_moves = self.get_all_possible_moves(self.turn);
            if possible_moves.len() < 5 {
                self.play_turn(Turn::Pass);
            }
            else  {
                self.play_turn(Turn::Move(self.play_random_move(&possible_moves)));
            }
        }

        println!("Game over");
    }

    /// Plays a random move if possible
    pub fn auto_move(&mut self) {
        let possible_moves = self.get_all_possible_moves(self.turn);
        if possible_moves.is_empty() {
            return;
        }

        let random_move = self.play_random_move(&possible_moves);
        self.play_move(random_move);
    }

    /// clamps the coordinate to be within the max size of the board
    pub fn clamp_coordinate(&self, x: usize, y: usize) -> Coordinate {
        let mut new_x = x;
        let mut new_y = y;
    
        if new_x > self.size {
            new_x = self.size;
        }
    
        if new_y > self.size {
            new_y = self.size;
        }
    
        Coordinate::Position((new_x, new_y))
    }

    /// return true if the game is over by resignation or passing
    pub fn check_end(&self) -> bool {
        self.game_tree.check_end()
    }

    /// moves the game tree pointer forward one (called when mousewheel is scrolled down)
    pub fn jump_forward(&mut self) {
        if self.game_tree.up_to_date() {
            return; // no need to change the board position
        }

        self.game_tree.move_forward();
    }

    /// moves the game tree pointer forward one (called when mousewheel is scrolled down)
    pub fn jump_back(&mut self) {
        if self.game_tree.get_pointer() == 0 {
            return; // no need to change the board position
        }

        self.game_tree.move_back();
    }

    /// calculate the score of the game assuming there are no dead stones
    pub fn calculate_total_completed_score(&self) -> (Colour, usize) {
        // ie this just finds empty spots, assigns them to a big group
        // matches the groups to a colour and then sums up the empty spots
        // chinese scoring is empty spots + number of stones on the board

        let grid = self.board_state.get_grid(); // the board state
        
        let (black_stone_count, white_stone_count): (usize, usize) = grid.iter()
            .map(|&colour| match colour {
                Colour::Black => (1, 0),
                Colour::White => (0, 1),
                Colour::Empty => (0, 0),
            })
        .fold((0, 0), |acc, counts| (acc.0 + counts.0, acc.1 + counts.1));

        // Count the number of empty intersections surrounded by each colour
        let black_area = self.board_state.get_surrounded_area(&grid, Colour::Black);
        let white_area = self.board_state.get_surrounded_area(&grid, Colour::White);

        // Chinese scoring: empty spots + number of stones on the board
        let black_score = black_area + black_stone_count;
        let white_score = white_area + white_stone_count;

        println!("Black Score: {}", black_score);
        println!("White Score: {}", white_score);

        // Return the total scores for both colours
        if black_score > white_score {
            (Colour::Black, black_score - white_score)
        } else {
            (Colour::White, white_score - black_score)
        }
    }
}