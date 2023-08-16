#![allow(non_camel_case_types)]

use crate::coordinate::Coordinate;
use crate::group::Group;

pub const BOARD_SIZE: usize = 19; // for now

pub type position = (usize, usize); // (x, y)
pub type index = usize; // index of the 1d vector


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

#[derive(Clone, Debug, PartialEq)]
pub struct Board {
    pub size: usize, // most typically 19x19 - will start smaller for ai
    grid: Vec<Colour>, // 1D vector that stores the colours of the stones on the board
    pub groups: Vec<Group>, // list of groups on the board
    white_points: usize, // how many stones white has captured
    black_points: usize, // how many stones black has captured
}

impl Board {
    pub fn new(size: usize) -> Self {
        Board {
            size,
            grid: vec![Colour::Empty; size * size],
            groups: Vec::new(),
            white_points: 0,
            black_points: 0,
        }
    }

    pub fn get(&self, coordinate: Coordinate) -> Colour {
        match coordinate.into_index() {
            Coordinate::Position(_) => { 
                //println!("Error: Position not supported");
                Colour::Empty
            }, // not reachable
            Coordinate::Index(value) => { 
                let colour = self.grid[value];
                match colour {
                    Colour::Black => return Colour::Black,
                    Colour::White => return Colour::White,
                    Colour::Empty => return Colour::Empty,
                }
            
            }
        }
    }

    pub fn add_stone(&mut self, coordinate: Coordinate, colour: Colour) {
        // adds a stone to the position WITHOUT CHECKING FOR VALIDITY

        if self.get(coordinate) == Colour::Empty { // can't place on a stone 
            let mut new_stone = Group::new(colour, coordinate); // 1.A new group is created
            self.grid[coordinate.get_index()] = colour; // 1.B the colour is added to the grid
            let mergeable_groups = new_stone.find_mergeable_groups(self);// 2. Merge groups if needed:
            self.merge_groups(mergeable_groups);

            // decrease the liberties of adjacent opposite colour groups
            let adjacent_groups_positions = Board::get_adjacent_indices(coordinate);
            let mut groups_to_remove: Vec<Group> = Vec::new();
            for group_position in adjacent_groups_positions {
                if self.get(group_position) != colour {
                    let group = self.find_group(group_position, colour.swap_turn());
                    if let Some(group) = group {
                        let count = group.decrease_and_get_liberties();

                        if count < 1 {
                            groups_to_remove.push(group.clone());
                        }
                    }
                }
            }          
            for group in groups_to_remove {
                self.remove_group(group);
            }
            //let new_super_group = self.find_group(coordinate, colour);
            
            let group_count = self.groups.len();
            for i in 0..group_count {
                self.groups[i].calculate_liberties(&self.grid);
            }
        }
    }

    pub fn find_group(&mut self, position: Coordinate, colour: Colour) -> Option<&mut Group>{
        // finds the group that has a stone at the given position

        for group in &mut self.groups {
            if group.contains(position, colour) {
                return Some(group);
            }
        }

        None
    }

    pub fn remove_group(&mut self, group: Group) {
        // removes a group from the board
        // this is used when a group is captured
        let points = group.get_points();
        //println!("POINTS TO BE REMOVED: {:?}", points);
        let mut count = 0;

        for position in points {
            let index = position.get_index();
            self.grid[index] = Colour::Empty;
            count += 1;
        }

        self.groups.retain(|x| x != &group);

        match group.get_colour() {
            Colour::Black => self.white_points += count,
            Colour::White => self.black_points += count,
            _ => {},
        }
    }

    pub fn get_adjacent_indices(position: Coordinate) -> Vec<Coordinate> {
        let (x, y) = position.get_position();
        let mut indices = Vec::new();

        if x > 0 {
            indices.push(Coordinate::Index((x - 1) * BOARD_SIZE + y)); // Top
        }
        if y > 0 {
            indices.push(Coordinate::Index(x * BOARD_SIZE + y - 1)); // Left
        }
        if x < BOARD_SIZE - 1 {
            indices.push(Coordinate::Index((x + 1) * BOARD_SIZE + y)); // Bottom
        }
        if y < BOARD_SIZE - 1 {
            indices.push(Coordinate::Index(x * BOARD_SIZE + y + 1)); // Right
        }
        indices
    }

    pub fn merge_groups(&mut self, groups: Vec<Group>) {
        // replace the old groups with a merged group
        let new_group = Group::merge_groups(&groups);
        // replace the old groups.
        self.groups.retain(|x| !groups.contains(&x));
        self.groups.push(new_group);
    }

    pub fn get_grid(&self) -> &Vec<Colour> {
        &self.grid
    }

    pub fn display_board(&self) {
        // print out the board in rows of size BOARD_SIZE
        // black is x, white is o and empty is .
        for row in 0..self.size {
            for col in 0..self.size {
                let index = row * self.size + col;
                match self.grid[index] {
                    Colour::Black => print!("x  "),
                    Colour::White => print!("o  "),
                    Colour::Empty => print!(".  "),
                }
            }
            println!(); // Move to the next line after each row
        }
    }
}

