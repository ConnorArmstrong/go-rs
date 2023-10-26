// a purely function way to represent the board

use std::collections::{HashMap, HashSet};

use crate::{colour::{Colour, self}, new_group::NewGroup, zobrist::ZobristTable, coordinate::{self, Coordinate}, group, fails::TurnErrors, board::BOARD_SIZE};

#[derive(Clone, Debug)]
pub struct BoardState {
    pub grid: Vec<Colour>,
    pub size: usize,
    pub groups: Vec<Option<usize>>,
    pub group_map: HashMap<usize, NewGroup>,
    pub group_counter: usize,
    pub zobrist_table: ZobristTable,
}


impl BoardState {
    pub fn new(size: usize) -> Self {
        BoardState {
            grid: vec![Colour::Empty; size * size],
            size, 
            groups: vec![None; size * size],
            group_map: HashMap::new(),
            group_counter: 0,
            zobrist_table: ZobristTable::new(),
        }
    }

    pub fn generate_grid_from_groups(groups: &Vec<Option<usize>>, map: &HashMap<usize, NewGroup>, size: usize) -> Vec<Colour> {
        let mut grid = vec![Colour::Empty; size * size];

        for (i, group) in groups.iter().enumerate() {
            if let Some(group_id) = group {
                let group = map.get(group_id).unwrap();
                for position in group.get_points().iter() {
                    grid[position.get_index()] = group.colour;
                }
            }
        }

        grid
    }

    /// takes a Vec<Option<usize>> and Vec<usize> and sets the value of the first Vec to None if that value appears in the second vector
    pub fn remove_groups(groups: &Vec<Option<usize>>, groups_to_remove: Vec<usize>) -> Vec<Option<usize>> {
        // Create a HashSet for quicker lookup
        let group_ids_to_remove: HashSet<usize> = groups_to_remove.into_iter().collect();
        
        // Create a new vector with updated groups
        let new_groups: Vec<Option<usize>> = groups.iter().map(|&x| {
            if let Some(id) = x {
                if group_ids_to_remove.contains(&id) {
                    return None;
                }
                return Some(id);
            }
            x
        }).collect();
    
        new_groups
    }
    /// places a stone onto the board
    /// acts as the play move for the local board instance
    pub fn add_stone(&self, coordinate: Coordinate, colour: Colour) -> Result<BoardState, TurnErrors> {
        let mut new_group_map = self.group_map.clone();
        let mut new_groups = self.groups.clone();
        let mut new_zobrist_table = self.zobrist_table.clone();
        // initial error checking:
        if coordinate.get_index() >= self.groups.len() {
            return Err(TurnErrors::OutofBounds);
        }

        if !self.check_empty(coordinate) {
            return Err(TurnErrors::AlreadyPlaced);
        }

        let new_group = NewGroup::new(self.group_counter, colour, coordinate); // create the group
        new_group_map.insert(self.group_counter, new_group.clone());
        new_groups[coordinate.get_index()] = Some(self.group_counter); // this can be changed later.
        let mut surrounding_groups = BoardState::find_adjacent_groups(&new_groups, &new_group_map, coordinate, colour, self.size); // get all surrounding groups - 

        if !surrounding_groups.is_empty() { // ie there exist groups to merge
            surrounding_groups.push(&new_group);
            let merged_group = NewGroup::merge_groups(self.group_counter, &surrounding_groups);

            for point in merged_group.get_positions() { // update group list
                new_groups[point.get_index()] = Some(self.group_counter);
            }
            new_group_map.insert(self.group_counter, merged_group);
        }

        let adjacent_groups = BoardState::find_adjacent_groups(&new_groups, &new_group_map, coordinate, colour.swap_turn(), self.size);
        // find and check that the opposite colour groups to our new group have liberties > 0: else remove them from the board
        let (new_groups, flag) = BoardState::check_opposing_groups_liberties(&new_groups, &new_group_map, self.size, adjacent_groups);

        let final_grid = &BoardState::generate_grid_from_groups(&new_groups, &new_group_map, self.size);

        if flag { // means I did not remove any groups
            let final_group = new_group_map.get(&new_groups[coordinate.get_index()].unwrap_or_else(|| panic!("Failed to unwrap new_groups at index {}", coordinate.get_index()))).unwrap_or_else(|| panic!("Failed to unwrap new_group_map"));            
            if !final_group.check_liberties(final_grid) { //final merged group has no liberties and didnt remove any other group
                return Err(TurnErrors::Suicide);
            }
        }

        if self.zobrist_table.contains_position(&final_grid) { // check for repeated position/ko
            return Err(TurnErrors::Ko);
        }

        new_zobrist_table.insert_position(&final_grid);

        Ok(BoardState {
            grid: final_grid.to_owned(),
            size: self.size,
            groups: new_groups.to_owned(),
            group_map: new_group_map.to_owned(),
            group_counter: self.group_counter + 1,
            zobrist_table: new_zobrist_table,
        })  
    }    
}


// pure state
impl BoardState {
    pub fn get(&self, coordinate: Coordinate) -> Colour {
        let index = coordinate.get_index();
        self.grid[index]
    }

    pub fn get_adjacent_indices(size: usize, coordinate: Coordinate) -> Vec<Coordinate> {
        let (x, y) = coordinate.get_position();
        let mut indices = Vec::new();

        if x > 0 {
            indices.push(Coordinate::Index((x - 1) * size + y)); // Top
        }
        if y > 0 {
            indices.push(Coordinate::Index(x * size + y - 1)); // Left
        }
        if x < size - 1 {
            indices.push(Coordinate::Index((x + 1) * size + y)); // Bottom
        }
        if y < size - 1 {
            indices.push(Coordinate::Index(x * size + y + 1)); // Right
        }
        indices
    }

    pub fn get_grid(&self) -> &Vec<Colour> {
        &self.grid
    }

    /// return a list of all current groups actually on the board
    pub fn get_current_groups(&self) -> Vec<NewGroup> {
        let current_groups: HashSet<NewGroup> = self.groups.clone()
                                                    .into_iter()
                                                    .filter_map(|x| x.and_then(|key| self.group_map.get(&key)))
                                                    .cloned()
                                                    .collect();

        return current_groups.into_iter().collect();
    }

    /// return if the group is currently on the board
    pub fn contains(&self, group: &NewGroup) -> bool {
        let current_groups: HashSet<NewGroup> = self.groups.clone()
                                                    .into_iter()
                                                    .filter_map(|x| x.and_then(|key| self.group_map.get(&key)))
                                                    .cloned()
                                                    .collect();
        current_groups.contains(&group)
    }

    /// returns the group at the given coordinate
    pub fn find_group(&self, coordinate: Coordinate) -> Option<&NewGroup> {
        let index = coordinate.get_index();

        if let Some(group_id) = self.groups[index] {
            return self.group_map.get(&group_id);
        }
        None
    }

    /// returns true if the coordinate is empty
    pub fn check_empty(&self, coordinate: Coordinate) -> bool {
        self.groups[coordinate.get_index()].is_none()
    }

    /// returns a list of all adjacent same colour group's group_id
    pub fn find_groups_to_merge(&self, coordinate: Coordinate, colour: Colour) -> Vec<usize> {
        
        if colour == Colour::Empty {return vec![]}; // shouldnt be called on empty - if it is nothing happens as there is a length check
        let mut id_set: HashSet<usize> = HashSet::new();

        for point in BoardState::get_adjacent_indices(self.size, coordinate) {
            if let Some(group_id) = self.groups[point.get_index()] {
                if self.group_map.get(&group_id).unwrap().get_colour() == colour {
                    id_set.insert(group_id);
                }
            }
        }

        id_set.into_iter().collect()
    }

    /// return a list of adjacent same colour groups given a specific coordinate
    pub fn find_adjacent_groups<'a>(groups: &'a Vec<Option<usize>>, group_map: &'a HashMap<usize, NewGroup>, coordinate: Coordinate, group_colour: Colour, size: usize) -> Vec<&'a NewGroup> {
        // return a list of adjacent same colour groups given a specific coordinate 
        let group = group_map.get(&groups[coordinate.get_index()].unwrap()).unwrap();
        let mut positions: HashSet<Coordinate> = HashSet::new();

        for point in group.get_positions() { // for all points that are of the same colour adjacent to points in the group
            for adjacent_point in BoardState::get_adjacent_indices(size, point) {
                if !(group.get_points().contains(&adjacent_point)) {
                    if let Some(index) = groups[adjacent_point.get_index()] {
                        if let Some(group_data) = group_map.get(&index) {
                            if group_data.get_colour() == group_colour {
                                positions.insert(adjacent_point);
                            }
                        }
                    }
                }
            }
        }

        let mut surrounding_groups: HashSet<&NewGroup> = HashSet::new();
        for position in positions { // get all matching groups for said adjacent positions
            let potential_group = group_map.get(&groups[position.get_index()].unwrap()).unwrap();
            if potential_group.get_colour() == group.get_colour() {
                surrounding_groups.insert(potential_group);
            }
        }

        surrounding_groups.into_iter().collect()
    }

    /// return the new list of groups after checking to remove adjacent groups with 0 liberties
    pub fn check_opposing_groups_liberties(total_groups: &Vec<Option<usize>>, map: &HashMap<usize, NewGroup>, size: usize, groups: Vec<&NewGroup>) -> (Vec<Option<usize>>, bool) {
        // for each group in groups
        // update and check that the liberties are above 0 else remove them
        let mut didnt_remove_groups: bool = true; // exists so i can check for suicide
        println!("Groups: {:?}", groups);

        let grid = BoardState::generate_grid_from_groups(total_groups, map, size);
        let mut ids_to_remove: Vec<usize> = Vec::new();

        for group in groups {
            if !group.check_liberties(&grid) {
                println!("removing group with id {}", group.get_id());
                ids_to_remove.push(group.id);
                didnt_remove_groups = false;
            }
        }

        (BoardState::remove_groups(total_groups, ids_to_remove), didnt_remove_groups)
    }

}