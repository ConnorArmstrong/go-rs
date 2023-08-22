#![allow(non_camel_case_types)]

use crate::coordinate::{Coordinate, self};
use crate::group::Group;
use crate::board::Colour;
use crate::new_group::{NewGroup, self};
use std::collections::{HashMap, HashSet};

pub const BOARD_SIZE: usize = 19; // for now

pub type position = (usize, usize); // (x, y)
pub type index = usize; // index of the 1d vector



pub struct NewBoard {
    grid: Vec<Colour>,
    group_map: HashMap<usize, NewGroup>, // maps each id to a group
    groups: Vec<Option<usize>>, // the groups currently on the board - groups.len == grid.len
    group_counter: usize, // the current number of groups tracked - should be equal to the most recent id
    colour_to_play: Colour, // the colour to play next
}


// logic functions
impl NewBoard {
    pub fn new() -> Self {
        NewBoard {
            grid: vec![Colour::Empty; BOARD_SIZE * BOARD_SIZE],
            group_map: HashMap::new(),
            groups: vec![None; BOARD_SIZE * BOARD_SIZE],
            group_counter: 0,
            colour_to_play: Colour::Black,
        }
    }

    pub fn remove_group(&mut self, group: NewGroup) {
        // get group id
        let group_id = group.get_id();
        let mut _count = 0;

        self.groups.iter_mut().for_each(|x| { // set matching id's to None
            if let Some(id) = x {
                if *id == *group_id {
                    *x = None;
                    _count += 1;
                }
            }
        });
    }

    pub fn add_stone(&mut self, coordinate: Coordinate, colour: Colour) {
        if !self.check_empty(coordinate) { // cannot place on a non-empty space
            return;
        }

        // 1. create a new group and stone
        let mut new_group = NewGroup::new(self.group_counter, colour, coordinate);
        let ids = self.find_groups_to_merge(coordinate, colour); // get all groups that need to be merged

    }
}


// state functions
impl NewBoard {
    pub fn get(&self, coordinate: Coordinate) -> Colour {
        let index = coordinate.get_index();
        self.grid[index]
    }

    pub fn get_adjacent_indices(coordinate: Coordinate) -> Vec<Coordinate> {
        let (x, y) = coordinate.get_position();
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

    pub fn get_grid(&self) -> &Vec<Colour> {
        &self.grid
    }

    pub fn get_current_groups(&self, coordinate: Coordinate) -> Vec<NewGroup> {
        let current_groups: HashSet<NewGroup> = self.groups.clone() // returns all the groups currently on the board
                                                    .into_iter()
                                                    .filter_map(|x| x.and_then(|key| self.group_map.get(&key)))
                                                    .cloned()
                                                    .collect();

        let groups: Vec<NewGroup> = current_groups.into_iter().collect();

        groups
    }

    pub fn contains(&self, group: NewGroup) -> bool {
        let current_groups: HashSet<NewGroup> = self.groups.clone() // returns all the groups currently on the board
                                                    .into_iter()
                                                    .filter_map(|x| x.and_then(|key| self.group_map.get(&key)))
                                                    .cloned()
                                                    .collect();

        current_groups.contains(&group)
    }

    pub fn find_group(&self, coordinate: Coordinate) -> Option<&NewGroup> {
        let index = coordinate.get_index();
        
        if let Some(group_id) = self.groups[index] {
            return self.group_map.get(&group_id)
        }

        None
    }

    pub fn clean_map(&mut self) {
        // deletes all old groups from the map
        let current_groups: HashSet<usize> = self.groups.clone().into_iter().filter_map(|x| x).collect();
        self.group_map.retain(|key, _| current_groups.contains(key));
    }

    pub fn check_empty(&self, coordinate: Coordinate) -> bool {
        self.grid[coordinate.get_index()] == Colour::Empty
    }

    pub fn find_groups_to_merge(&self, coordinate: Coordinate, colour: Colour) -> Vec<usize> {
        // returns a list of all adjacent same colour group's group_id
        if colour == Colour::Empty {panic!("EMPTY COLOUR TO MERGE")}; // shouldnt be called on empty.
        let mut id_set: HashSet<usize> = HashSet::new();

        for point in Self::get_adjacent_indices(coordinate) {
            if let Some(group_id) = self.groups[point.get_index()] {
                if self.group_map.get(&group_id).unwrap().get_colour() == colour {
                    id_set.insert(group_id);
                }
            }
        }

        id_set.into_iter().collect()
    }
}

