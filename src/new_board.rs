#![allow(non_camel_case_types)]

use crate::coordinate::Coordinate;
use crate::board::Colour;
use crate::new_group::NewGroup;
use std::collections::{HashMap, HashSet};


pub const BOARD_SIZE: usize = 19; // for now

pub type position = (usize, usize); // (x, y)
pub type index = usize; // index of the 1d vector


pub struct NewBoard {
    grid: Vec<Colour>,
    group_map: HashMap<usize, NewGroup>, // maps each id to a group
    groups: Vec<Option<usize>>, // the groups currently on the board - groups.len == grid.len
    group_counter: usize, // the current number of groups tracked - should be equal to the most recent id
}


// logic functions
impl NewBoard {
    pub fn new() -> Self {
        NewBoard {
            grid: vec![Colour::Empty; BOARD_SIZE * BOARD_SIZE],
            group_map: HashMap::new(),
            groups: vec![None; BOARD_SIZE * BOARD_SIZE],
            group_counter: 0,
        }
    }

    pub fn remove_group(&mut self, group: NewGroup) { // removes a group from self.groups
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

    pub fn remove_groups(&mut self, groups: Vec<usize>) {
        // removes the groups from the self.groups vector

        for group_id in groups {
            let id = self.group_map.get(&group_id);
            match id {
                Some(group) => self.remove_group(group.clone()),
                None => continue,
            }
        }
    }

    pub fn add_stone(&mut self, coordinate: Coordinate, colour: Colour) {
        
        if !self.check_empty(coordinate) { // cannot place on a non-empty space
            println!("Cannot place on a non-empty space");
            return;
        }
        // steps: 1 create new group and add stone. 2: merge groups if necessary. 3: update adjacent groups liberties. 4: update new_groups liberties. Remove groups when necessary

        // 1. create a new group and stone
        let new_group = NewGroup::new(self.group_counter, colour, coordinate);

        self.group_map.insert(self.group_counter, new_group.clone()); // this can get overwritten later.
        self.groups[coordinate.get_index()] = Some(self.group_counter); // changed later too


        let mut surrounding_groups = self.find_adjacent_groups(coordinate); // get all surrounding groups


        if surrounding_groups.len() > 0 { // there are now groups to merge
            surrounding_groups.push(&new_group);
            let merged_group = NewGroup::merge_groups(self.group_counter, &surrounding_groups);
 

            for point in merged_group.get_positions() { //update group list
                self.groups[point.get_index()] = Some(self.group_counter);
            }
            self.group_map.insert(self.group_counter, merged_group);
        }

        self.group_counter += 1;
        self.reflect_group_to_board();


        // 3. update adjacent groups liberties
        // get list of adjacent opponant groups by finding the opposite colour to merge
        let adjacent_groups = self.find_groups_to_merge(coordinate, colour.swap_turn());
        self.recalculate_liberties(adjacent_groups.clone());
        let groups_to_remove = self.check_liberties(adjacent_groups);
        self.remove_groups(groups_to_remove);
        
        let total_groups = self.group_map.len();
        self.clean_map();
        println!("Total Groups in the map: {}", total_groups);

        // 4. check new groups liberties.
        println!("  Groups:");
        for group in self.group_map.values() {
            println!("Group: {:?}", group);
        }

        // TODO:
        self.reflect_group_to_board();

    }

    pub fn recalculate_liberties(&mut self, groups: Vec<usize>) {
        // step 3 of adding a stone - update adjacent groups liberties
        for group_id in groups {
            self.group_map.get_mut(&group_id).unwrap().calculate_liberties(&self.grid);
        }
    }
    
    pub fn check_liberties(&self, ids: Vec<usize>) -> Vec<usize> {
        // return the groups that have 0 liberties
        let mut groups = Vec::new();
        for group_id in ids {
            if self.group_map.get(&group_id).unwrap().get_liberties() < &((1 as usize)) { // if the group has 0 liberties - NOTE: cannot do if usize == 0
                groups.push(group_id);
            }
        }

        groups
    }

    pub fn reflect_group_to_board(&mut self) { // makes the self.grid match the self.groups
        // iterate over the grid and make sure each value matches the groups
        for (index, group_id) in self.groups.iter().enumerate() {
            if let Some(id) = group_id {
                let group = self.group_map.get(id).unwrap();
                self.grid[index] = group.get_colour();
            } else {
                self.grid[index] = Colour::Empty;
            }
        }

        // print a hashset of the group ids
        let mut groups: HashSet<usize> = HashSet::new();
        for group in &self.groups {
            if let Some(id) = group {
                groups.insert(*id);
            }
        }
        println!("Number of groups: {}", groups.len()); 
        println!("Groups on the board: {:?}", groups);
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
        //self.grid[coordinate.get_index()] == Colour::Empty || 
        self.groups[coordinate.get_index()] == None
    }

    pub fn find_groups_to_merge(&self, coordinate: Coordinate, colour: Colour) -> Vec<usize> {
        // returns a list of all adjacent same colour group's group_id
        if colour == Colour::Empty {return vec![]}; // shouldnt be called on empty - if it is nothing happens as there is a length check
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

    pub fn find_adjacent_groups(&self, coordinate: Coordinate) -> Vec<&NewGroup> { // return a list of adjacent same colour groups given a specific coordinate
        let group = self.find_group(coordinate).unwrap();
        let group_colour = group.get_colour();
        let mut positions: HashSet<Coordinate> = HashSet::new();

        for point in group.get_positions() { // get all positions adjacent to the group at the given coordinate
            for adjacent_point in Self::get_adjacent_indices(point) {
                if !(group.get_points().contains(&adjacent_point)) && self.grid[adjacent_point.get_index()] == group_colour {
                    positions.insert(adjacent_point);
                }
            }
        }

        let mut surrounding_groups: HashSet<&NewGroup> = HashSet::new();
        for position in positions { // get all matching groups for said adjacent positions
            let potential_group = self.find_group(position).unwrap();
            if potential_group.get_colour() == group.get_colour() {
                surrounding_groups.insert(potential_group);
            }
        }

        surrounding_groups.into_iter().collect()
    }

    pub fn get_groups_to_merge<'a>(&'a self, initial_group: &'a NewGroup, ids: Vec<usize>) -> Vec<&'a NewGroup> {
        // get a vector of the initial group + the surrounding groups to be merged
        let mut groups: Vec<&NewGroup> = vec![&initial_group];

        for id in ids {
            groups.push(self.group_map.get(&id).unwrap());
        }

        groups
    }
}

