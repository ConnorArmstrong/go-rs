use rand::Rng;
use std::collections::{HashMap, HashSet};

use crate::colour::Colour;


#[derive(Debug, Clone)]
pub struct ZobristTable {
    map: HashMap<(usize, Colour), u64>,
    visited: HashSet<u64>,
}

impl ZobristTable {
    pub fn new(size: usize) -> Self {
        let mut rng = rand::thread_rng();
        let mut table = HashMap::new();

        for i in 0..(size * size) {
            for &colour in &[Colour::Black, Colour::White] {
                let random_value: u64 = rng.gen();
                table.insert((i, colour), random_value);
            }
        }

        ZobristTable { map: table, visited: HashSet::new()  }
    }

    /// Compute the Zobrist hash of a given board
    fn zobrist_hash(&self, board: &[Colour]) -> u64 { 
        let mut hash_value: u64 = 0;
    
        for (i, &colour) in board.iter().enumerate() {
            if colour != Colour::Empty {
                hash_value ^= self.map.get(&(i, colour)).unwrap();
            }
        }
    
        hash_value
    }

    /// Get the zobrist hash and insert it
    pub fn insert_position(&mut self, board: &[Colour]) { 
        let hash_value = self.zobrist_hash(board);
        self.visited.insert(hash_value);
    }

    /// Check if the board's zobrist hash exists
    pub fn contains_position(&self, board: &[Colour]) -> bool { 
        let hash_value = self.zobrist_hash(board);
        self.visited.contains(&hash_value)
    }
}
