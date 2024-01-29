//#![allow(unused)]

mod coordinate;

mod graphics;
mod colour;
mod fails;
mod tree;
mod zobrist;
mod board_state;
mod game_state;
mod group_state;
mod turn;

const BOARD_SIZE: usize = 5;

// potentially define MCTS parameters here:
// ie iterations/time to iterate for

fn main() {
    println!("running...");
    graphics::run(BOARD_SIZE).unwrap();
}
