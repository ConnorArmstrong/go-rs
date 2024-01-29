use std::cell::RefCell;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use rand::Rng;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rayon::iter::{IntoParallelRefIterator, IntoParallelIterator};
use rayon::prelude::ParallelIterator;

use crate::colour::Outcome;
use crate::group_state::GroupState;
use crate::{board_state::BoardState, colour::Colour, tree::GameTree, coordinate::Coordinate, fails::TurnErrors, turn::Turn};

pub const AUTO_PLAY: bool = false; // play against the MCTS
pub const _SELF_PLAY: bool = false; // play against itself
pub const _WEAK_PLAY: bool = false; // have the MCTS play against a random opponent

pub const KOMI: f32 = 0.5;

pub struct GameState {
    pub board_state: BoardState,
    pub turn: Colour,
    pub game_tree: GameTree,
    rng: RefCell<ThreadRng>,
    pub size: usize,
    mcts: Arc<Mutex<MonteCarloSearch>>, // for the persistent mcts
}

impl GameState {
    pub fn new(board_size: usize) -> Self {
        let mcts = Arc::new(Mutex::new(MonteCarloSearch::new(BoardState::new(board_size), Colour::Black)));
        let mut search = mcts.lock().unwrap();
        search.expand(0);

        println!("{}", search.nodes.len());


        GameState {
            board_state: BoardState::new(board_size),
            turn: Colour::Black,
            game_tree: GameTree::new(board_size),
            rng: RefCell::new(rand::thread_rng()),
            size: board_size,
            mcts: mcts.clone(),
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
                let turn_number = self.game_tree.get_length();
                let turn_colour: String = self.turn.get_string();
                match error {
                    TurnErrors::AlreadyPlaced => println!("Error on Move {}: {} Stone already in the position at {:?}", turn_number, turn_colour, coordinate),
                    TurnErrors::Ko => println!("Error on Move {}: {} can't place due to Ko", turn_number, turn_colour),
                    TurnErrors::Suicide => println!("Error on Move {}: {} can't place due to suicide at {:?}", turn_number, turn_colour, coordinate),
                    TurnErrors::OutofBounds => println!("Error on Move {}: {} can't place due to out of bounds at {:?}", turn_number, turn_colour, coordinate),
                }

                return false;
            }
        }

        /* TODO:
                if AUTO_PLAY { // handle mcts tree pruning
            let mut mcts = self.mcts.lock().unwrap();
            mcts.expand(0); 

            let root_node = &mcts.nodes[mcts.root];

            let played_move_node_index = root_node.children.iter()
                .find(|&&child_index| mcts.nodes[child_index].game_move == Some(coordinate))
                .cloned()
                .expect("Didn't find the move");
            mcts.prune(played_move_node_index);
        }
        */

        return true;
    }

    /// handles all Turn Enum Arms: Move, Pass and Resign
    pub fn play_turn(&mut self, turn: Turn) {
        match turn {
            Turn::Move(coordinate) => {
                if self.play_move(coordinate) {
                    self.game_tree.add_move(turn, self.board_state.clone());
                }

                if AUTO_PLAY {
                    if self.turn == Colour::White {
                        self.auto_move();
                    }
                }

                
            },
            Turn::Pass => {
                println!("{} has Passed.", self.turn.get_string());
                self.swap_turn();
                self.game_tree.add_move(turn, self.board_state.clone());
            },
            Turn::Resign => {
                self.game_tree.add_move(turn, self.board_state.clone())
            }
        }
    }

    /// get all possible moves for the current board state and colour through brute force
    pub fn get_all_possible_moves(&self, colour: Colour) -> Vec<Coordinate> {
        let mut possible_moves = Vec::new();
        let current_state = self.board_state.clone();

        (0..self.size * self.size).into_iter().for_each(|i| {
            let coordinate = Coordinate::Index(i);
            let new_position = current_state.add_stone(coordinate, colour);

            if new_position.is_ok() {
                possible_moves.push(coordinate);
            }
        });

        possible_moves
    }

    /// Get all possible moves for a given board state and colour using brute force and parallelization
    pub fn get_all_possible_moves_for_board(board: &BoardState, colour: Colour) -> Vec<Coordinate> {
        (0..board.size * board.size).into_par_iter().filter_map(|i| {
            let coordinate = Coordinate::Index(i);
            let new_position = board.add_stone(coordinate, colour);

            if new_position.is_ok() {
                Some(coordinate)
            } else {
                None
            }
        }).collect()
    }

    /// Chooses a random coordinate from the possible coordinate
    pub fn play_random_move(&self, possible_moves: &[Coordinate]) -> Option<Coordinate> {
        if possible_moves.is_empty() {
           return None;
        }

        let mut rng = self.rng.borrow_mut();
        let index: usize = rng.gen_range(0..possible_moves.len()); // chosen position
        Some(possible_moves[index])
    }

    /// Returns true if either black or white have possible moves to play
    pub fn _moves_to_play(&self) -> bool {
        let black_moves = self.get_all_possible_moves(Colour::Black).len() > 0;
        let white_moves = self.get_all_possible_moves(Colour::White).len() > 0;

        black_moves || white_moves
    }

    pub fn count_possible_moves(&self) {
        println!("{}", self.get_all_possible_moves(self.turn).len());
    }

    pub fn random_completed_game(&mut self) {
        println!("Playing random game (but like better than the other one)");

        while !self.check_end() {
            if self.board_state.check_all_important_points_played() {
                // all important points are played for the given turn
                self.play_turn(Turn::Pass);
            } 

            // else we need to play a random move

            let possible_moves = self.get_all_possible_moves(self.turn);

            if possible_moves.len() < 3 {
                if possible_moves.is_empty() {
                    self.play_turn(Turn::Resign);
                } else {
                    self.play_turn(Turn::Pass);
                }
            }

            let random_move = self.play_random_move(&possible_moves);

            match random_move {
                Some(m) => self.play_turn(Turn::Move(m)),
                None => self.play_turn(Turn::Pass),
            }

        }

        println!("Game over - and should be like fairly reasonable.");
        self.calculate_total_completed_score();
    }


    /// return true if the only moves to play are within a player's own territory
    /// also return false if there are dame points still left
    pub fn check_useful_points_played(board: &BoardState, colour: Colour) -> bool {
        let grid = board.get_grid();
        
        let all_possible_moves = GameState::get_all_possible_moves_for_board(board, colour); // generate all possible moves for this colour
        let territory = board.get_territory(&grid, colour); // a vector of empty territoy groups
        let coords: HashSet<Coordinate> = GroupState::combine_groups(&territory); // a hashset of coordinates that are in the territory
    
        let useful_moves = all_possible_moves.iter().filter(|&move_to_play| coords.contains(move_to_play)).count(); // the number of moves that are in the territory
    
        //println!("Useful Moves: {} | All possible moves: {}", useful_moves, all_possible_moves.len());
    
        return useful_moves == all_possible_moves.len();
    }


    /// Plays a random move if possible
    pub fn auto_move(&mut self) {
        if self.game_tree.get_length() < 30 {
            // generate a random move instead
            //let possible_moves = self.get_all_possible_moves(self.turn);
            //self.play_turn(Turn::Move(possible_moves.last().unwrap().clone())); 
            //self.play_turn(Turn::Move(self.play_random_move(&possible_moves).unwrap()));
            //return;
        }
        let coordinate = self.decide_next_move(self.turn);
        self.play_turn(Turn::Move(coordinate));
    }

    pub fn weak_play(&mut self) {
        // Black is Random, White is MCTS
        println!("Random vs MCTS starting");
        let mut counter = 0;
        loop {
            if self.board_state.check_all_important_points_played() {
                // all important points are played for the given turn
                self.play_turn(Turn::Pass);
            } 

            // else we need to play a random move

            if self.turn == Colour::Black {
                let possible_moves = self.get_all_possible_moves(self.turn);
                let random_move = self.play_random_move(&possible_moves);

                match random_move {
                    Some(m) => self.play_turn(Turn::Move(m)),
                    None => self.play_turn(Turn::Pass),
                }
            } else {
                self.auto_move();
            }

            if self.check_end() {
                break;
            }
            counter += 1;

            if counter > 30 {
                break;
            }
        }

        println!("Done after {} moves", counter);
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

        let _ = self.game_tree.move_forward();
    }

    /// moves the game tree pointer forward one (called when mousewheel is scrolled down)
    pub fn jump_back(&mut self) {
        if self.game_tree.get_pointer() == 0 {
            return; // no need to change the board position
        }

        let _ = self.game_tree.move_back();
    }

    /// calculate the score of the game assuming there are no dead stones
    pub fn calculate_total_completed_score(&self) -> (Colour, f32) {
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
        let (black_area, white_area) = self.board_state.get_colour_territory(&grid);

        // Chinese scoring: empty spots + number of stones on the board
        let black_score = (black_area + black_stone_count) as f32;
        let white_score = (white_area + white_stone_count) as f32 + KOMI; // komi is always some 0.5 value

        // Return the total scores for both colours
        if black_score > white_score {
            (Colour::Black, black_score - white_score)
        } else {
            (Colour::White, white_score - black_score)
        }
    }

    /// Let the MCTS know if the game is over
    pub fn is_game_over(board: &BoardState) -> bool {
        /* 
        // returns true if therre are still points that dont belong to either territory
        //let first = board.check_all_important_points_played();

        // this function returns true if the only moves to play are within a player's own territory 
        let black = GameState::check_useful_points_played(board, Colour::Black);
        let white = GameState::check_useful_points_played(board, Colour::White);

        black && white */
        board.check_all_important_points_played()
    }

    /// Let the MCTS know the outcome of the game
    pub fn determine_outcome(board: &BoardState) -> Outcome {
        // ie this just finds empty spots, assigns them to a big group
        // matches the groups to a colour and then sums up the empty spots
        // chinese scoring is empty spots + number of stones on the board

        let grid = board.get_grid(); // the board state
        
        let (black_stone_count, white_stone_count): (usize, usize) = grid.iter()
            .map(|&colour| match colour {
                Colour::Black => (1, 0),
                Colour::White => (0, 1),
                Colour::Empty => (0, 0),
            })
        .fold((0, 0), |acc, counts| (acc.0 + counts.0, acc.1 + counts.1));

        // Count the number of empty intersections surrounded by each colour
        let black_area = board.get_surrounded_area(&grid, Colour::Black);
        let white_area = board.get_surrounded_area(&grid, Colour::White);

        // Chinese scoring: empty spots + number of stones on the board
        let black_score = (black_area + black_stone_count) as f32;
        let white_score = (white_area + white_stone_count) as f32 + KOMI;

        if black_score > white_score {
            return Outcome::BlackWin;
        } else {
            return Outcome::WhiteWin;
        }
    }
        
    /// Use the MCTS to return a coordinate to play.
    /// 
    /// This is only called if it is determined there should be a move to play
    /// Passes and resignations are handled elsewhere
    pub fn decide_next_move(&self, colour: Colour) -> Coordinate {
        let mcts = Arc::new(Mutex::new(MonteCarloSearch::new(self.board_state.clone(), colour)));

        // Parameters:
        let max_time = Duration::from_millis(5000);
        let max_iterations = 25000;
        let num_threads = 2;
        
        let start = std::time::Instant::now();
        let iterations = Arc::new(Mutex::new(0));

        println!("Starting MCTS with max_time: {:?}, max_iterations: {}", max_time, max_iterations);

        while start.elapsed() < max_time && *iterations.lock().unwrap() < max_iterations {
            let mcts_clone = mcts.clone();
            let iterations_clone = iterations.clone();

            // Parallelize the Simulation phase
            (0..num_threads).into_par_iter().for_each(|_| {
                let mut mcts_locked = mcts_clone.lock().unwrap();

                // Selection
                let leaf_index = mcts_locked.select_leaf(mcts_locked.root);

                // Expansion
                mcts_locked.expand(leaf_index);

                // Simulation
                let outcome = mcts_locked.simulate(leaf_index);

                // Backpropagation
                mcts_locked.backpropagate(leaf_index, outcome);

                // Increment iterations in a thread-safe way
                let mut iter_locked = iterations_clone.lock().unwrap();
                *iter_locked += 1;

                //println!("Thread {:?} completed!", thread::current().id());
            });
            //println!("Completed {} iterations", *iterations.lock().unwrap());
        }

        println!("FINAL NUMBER OF ITERATIONS: {} in {:?}", *iterations.lock().unwrap(), start.elapsed());

        let mcts = mcts.lock().unwrap();

        // After the MCTS loop
        let best_child_index = mcts.nodes[mcts.root].children.par_iter()
        .max_by(|&a, &b| {
            let win_ratio_a = mcts.nodes[*a].wins as f64 / mcts.nodes[*a].visits as f64;
            let win_ratio_b = mcts.nodes[*b].wins as f64 / mcts.nodes[*b].visits as f64;
            win_ratio_a.partial_cmp(&win_ratio_b).unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap_or_else(|| {
            panic!("No children in the current node of the MCTS tree");
        });

        // Return the best move
        println!("Decided on Move at: {:?} with winrate: {} after {} visits", mcts.nodes[*best_child_index].game_move.unwrap(), mcts.nodes[*best_child_index].wins as f64 / mcts.nodes[*best_child_index].visits as f64, mcts.nodes[*best_child_index].visits);
        mcts.nodes[*best_child_index].game_move.unwrap()
    }
}


pub struct MonteCarloNode { // maybe?
    pub state: Arc<BoardState>, // the actual position of the board
    pub parent: Option<usize>,
    pub children: Vec<usize>, // A list of the children's ids
    pub wins: usize, // how many wins this node leads to
    pub visits: usize, // how many times has this node been visited
    pub colour: Colour, // Turn to play
    pub id: usize, // the index in the node list
    pub game_move: Option<Coordinate>, // the move that led to this node
}

pub struct MonteCarloSearch {
    pub nodes: Vec<MonteCarloNode>, // Where each index is the id of the node
    pub root: usize, // the starting position -> either an empty board or the current board
}


impl MonteCarloSearch {
    pub fn new(board: BoardState, colour: Colour) -> Self {
        let root_node = MonteCarloNode {
            state: board.into(),
            parent: None,
            children: Vec::new(),
            wins: 0,
            visits: 0,
            colour,
            id: 0,
            game_move: None,
        };

        MonteCarloSearch {
            nodes: vec![root_node],
            root: 0,
        }
    }

    pub fn _get_root(&self) -> &MonteCarloNode {
        &self.nodes[self.root]
    }

    /// Selection phase of the MCTS
    fn select_leaf(&self, node_index: usize) -> usize {
        let node = &self.nodes[node_index];
    
        // If the node has no children, return it
        if node.children.is_empty() {
            return node_index;
        }
    
        // Select the child with the highest UCT value
        let log_parent_visits = (node.visits as f64).ln();
        let best_child_index = node.children.par_iter()
            .max_by(|&a, &b| {
                let uct_a = self.calculate_uct(*a, log_parent_visits);
                let uct_b = self.calculate_uct(*b, log_parent_visits);
                uct_a.partial_cmp(&uct_b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap_or_else(|| {
                panic!("No children in the current node of the MCTS tree");
            });
    
        // Recursively select the best child
        self.select_leaf(*best_child_index)
    }

    /// Calculate the UCT for the given node
    fn calculate_uct(&self, node_index: usize, log_parent_visits: f64) -> f64 {
        let node = &self.nodes[node_index];

        if node.visits == 0 { // make sure all nodes get explored at least once
            return f64::MAX;
        }

        let win_ratio = node.wins as f64 / node.visits as f64;
        let exploration = 2.0 * (log_parent_visits / node.visits as f64).sqrt();
        win_ratio + (exploration/1.1)
    }

    /// Expansion phase of the MCTS
    fn expand(&mut self, node_index: usize) {
        let node_state = Arc::clone(&self.nodes[node_index].state);
        let node_colour = self.nodes[node_index].colour;

        // Generate all possible moves from the current state
        let possible_moves = GameState::get_all_possible_moves_for_board(&node_state, node_colour);
        //possible_moves.shuffle(&mut *self.rng.lock().unwrap()); // for now - could implement a heuristic

        // For each move, create a new node and add it to the tree
        for game_move in possible_moves {
            let new_state = Arc::new(node_state.add_stone(game_move, node_colour).unwrap());
            let new_node = MonteCarloNode {
                state: new_state,
                parent: Some(node_index),
                children: Vec::new(),
                wins: 0,
                visits: 0,
                colour: node_colour.swap_turn(),
                id: self.nodes.len(),
                game_move: Some(game_move),
            };
            self.nodes.push(new_node);
        }

        // Update the children of the node
        let children_ids: Vec<usize> = (node_index+1..self.nodes.len()).collect();
        self.nodes[node_index].children = children_ids;
    }

    fn simulate(&self, node_index: usize) -> Outcome {
        let mut current_state = self.nodes[node_index].state.clone();
        let mut colour = self.nodes[node_index].colour;
        let mut consecutive_passes = 0;
    
        // Loop until the game is over
        while !GameState::is_game_over(&current_state) {
            // Select a move using the default policy (in this case, a random move)
            let possible_moves = GameState::get_all_possible_moves_for_board(&current_state, colour);
            
            let game_move = if GameState::check_useful_points_played(&current_state, colour) {
                // If there are no possible moves, simulate a pass
                consecutive_passes += 1;
                None
            } else {
                consecutive_passes = 0;
                Some(self.select_random_move(&possible_moves))
            };
    
            // Apply the move to get the new state
            if let Some(game_move) = game_move {
                current_state = current_state.add_stone(game_move, colour).unwrap().into();
            }
    
            // If there are two consecutive passes, the game is over
            if consecutive_passes >= 2 {
                break;
            }
    
            colour = colour.swap_turn();
        }
    
        // Return the outcome of the game
        GameState::determine_outcome(&current_state)
    }

    fn select_random_move(&self, possible_moves: &[Coordinate]) -> Coordinate {
        let mut rng = rand::thread_rng();
        possible_moves.choose(&mut rng).unwrap().clone()
    }

    fn backpropagate(&mut self, leaf_index: usize, outcome: Outcome) {
        let mut current_index = leaf_index;

        // Loop until the root node is reached
        while let Some(parent_index) = self.nodes[current_index].parent {
            // Update the visits and wins of the parent node
            self.nodes[parent_index].visits += 1;
            if self.nodes[current_index].colour == outcome.into_colour() {
                self.nodes[parent_index].wins += 1;
            }

            // Move to the parent node
            current_index = parent_index;
        }

        // Update the visits and wins of the root node
        self.nodes[current_index].visits += 1;
        if self.nodes[current_index].colour == outcome.into_colour() {
            self.nodes[current_index].wins += 1;
        }
    }

    /// Modify the MCTS tree to move the root to some new node
    /// 
    /// This also adjusts every child node's id, parent, and children
    pub fn prune(&mut self, new_root: usize) {
        if new_root >= self.nodes.len() {
            panic!("Error: Attempted to prune with a non-existent node index: {}", new_root);
        }
    
        // Adjust the ids, parents, and children of the nodes
        for node in &mut self.nodes {
            node.id -= new_root;
            if let Some(parent) = node.parent {
                node.parent = Some(parent - new_root);
            }
            node.children = node.children.iter().map(|&child| child - new_root).collect();
        }
    
        // Keep the nodes from new_root onwards and discard the rest
        self.nodes = self.nodes.split_off(new_root);
    
        // Set the root to the new_root (which is now at index 0)
        self.root = 0;
    }
}

