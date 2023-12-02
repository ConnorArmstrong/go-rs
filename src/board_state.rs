// a purely function way to represent the board

use std::collections::{HashMap, HashSet};

use eframe::glow::LINES_ADJACENCY;

use crate::{colour::Colour, zobrist::ZobristTable, coordinate::Coordinate, fails::TurnErrors, group_state::GroupState};

#[derive(Clone, Debug)]
pub struct BoardState {
    pub grid: Vec<Colour>,
    pub size: usize,
    pub groups: Vec<Option<usize>>,
    pub group_map: HashMap<usize, GroupState>,
    pub group_counter: usize,
    pub zobrist_table: ZobristTable,
}


impl BoardState {
    /// Creates a new board of a specified size
    pub fn new(size: usize) -> Self {
        BoardState {
            grid: vec![Colour::Empty; size * size],
            size,
            groups: vec![None; size * size],
            group_map: HashMap::new(),
            group_counter: 0,
            zobrist_table: ZobristTable::new(size),
        }
    }

    /// Converts the list of groups into a more displayable list of colours
    pub fn generate_grid_from_groups(groups: &Vec<Option<usize>>, map: &HashMap<usize, GroupState>, size: usize) -> Vec<Colour> {
        let mut grid = vec![Colour::Empty; size * size];

        for group in groups {
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
        if groups_to_remove.is_empty() {
            return groups.to_owned()
        }

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
        let mut new_zobrist_table = self.zobrist_table.clone(); // these all get passed to the new board

        // initial error checking:
        if coordinate.get_index() >= self.groups.len() {
            return Err(TurnErrors::OutofBounds);
        }

        if !self.check_empty(coordinate) {
            return Err(TurnErrors::AlreadyPlaced);
        }

        let new_group = GroupState::new(self.group_counter, colour, coordinate); // create the group
        new_group_map.insert(self.group_counter, new_group.clone());
        new_groups[coordinate.get_index()] = Some(self.group_counter); // this can be changed later.
        let mut surrounding_groups = BoardState::find_adjacent_groups(&new_groups, &new_group_map, coordinate, colour, self.size); // get all surrounding groups - 

        if !surrounding_groups.is_empty() { // ie there exist groups to merge
            surrounding_groups.push(&new_group);
            let merged_group = GroupState::merge_groups(self.group_counter, &surrounding_groups);

            for point in merged_group.get_positions() { // update group list
                new_groups[point.get_index()] = Some(self.group_counter);
            }
            new_group_map.insert(self.group_counter, merged_group);
        }

        // find opposite colour groups adjacent to the new group
        let adjacent_groups = BoardState::find_adjacent_groups(&new_groups, &new_group_map, coordinate, colour.swap_turn(), self.size);

        // find and check that the opposite colour groups to our new group have liberties > 0: else remove them from the board
        //let (new_groups, flag) = BoardState::check_opposing_groups_liberties(&new_groups, &new_group_map, self.size, &adjacent_groups);
        let (new_groups, flag) = BoardState::check_opposing_adjacent_liberties(&new_groups, &new_group_map, self.size, coordinate);

        let final_grid = &BoardState::generate_grid_from_groups(&new_groups, &new_group_map, self.size);

        if flag { // no groups were removed 
            let final_group = new_group_map.get(&new_groups[coordinate.get_index()].unwrap_or_else(|| panic!("Failed to unwrap new_groups at index {}", coordinate.get_index()))).unwrap_or_else(|| panic!("Failed to unwrap new_group_map"));
            //println!("liberties: {:?}", final_group.clone().calculate_liberties(final_grid, self.size));            
            if !final_group.check_liberties(final_grid, self.size) { // final merged group has no liberties and didnt remove any other group
                return Err(TurnErrors::Suicide);
            }
        }
        
        //self.debug_groups(&new_groups, &new_group_map);

        if self.zobrist_table.contains_position(&final_grid) { // check for repeated position/ko
            return Err(TurnErrors::Ko);
        }

        new_zobrist_table.insert_position(&final_grid);
        let final_group_map = BoardState::clean_map(new_group_map, &new_groups);

        Ok(BoardState {
            grid: final_grid.to_owned(),
            size: self.size,
            groups: new_groups.to_owned(),
            group_map: final_group_map,
            group_counter: self.group_counter + 1,
            zobrist_table: new_zobrist_table,
        })
    }

    pub fn clean_map(group_map: HashMap<usize, GroupState>, groups: &Vec<Option<usize>>) -> HashMap<usize, GroupState> {
        let current_groups: HashSet<usize> = groups.clone().into_iter().filter_map(|x| x).collect();
        let new = group_map.clone().into_iter().filter(|(key, _)| current_groups.contains(key)).collect();
        return new;
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
    pub fn get_current_groups(&self) -> Vec<GroupState> {
        let current_groups: HashSet<GroupState> = self.groups.clone()
                                                    .into_iter()
                                                    .filter_map(|x| x.and_then(|key| self.group_map.get(&key)))
                                                    .cloned()
                                                    .collect();

        return current_groups.into_iter().collect();
    }

    /// return if the group is currently on the board
    pub fn contains(&self, group: &GroupState) -> bool {
        let current_groups: HashSet<GroupState> = self.groups.clone()
                                                    .into_iter()
                                                    .filter_map(|x| x.and_then(|key| self.group_map.get(&key)))
                                                    .cloned()
                                                    .collect();
        current_groups.contains(&group)
    }

    /// returns the group at the given coordinate
    pub fn find_group(&self, coordinate: Coordinate) -> Option<&GroupState> {
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
                if self.group_map.get(&group_id).unwrap().colour == colour {
                    id_set.insert(group_id);
                }
            }
        }

        id_set.into_iter().collect()
    }

    /// return a list of adjacent same colour groups given a specific coordinate
    pub fn find_adjacent_groups<'a>(groups: &'a Vec<Option<usize>>, group_map: &'a HashMap<usize, GroupState>, coordinate: Coordinate, group_colour: Colour, size: usize) -> Vec<&'a GroupState> {
        let group_id = groups[coordinate.get_index()].expect("Group ID cannot be None");
        let group = group_map.get(&group_id).expect("Group not found in group_map");
    
        let positions: HashSet<Coordinate> = group
            .get_positions()
            .iter()
            .flat_map(|point| BoardState::get_adjacent_indices(size, *point))
            .filter(|&adjacent_point| !group.get_points().contains(&adjacent_point))
            .filter_map(|adjacent_point| {
                let index = groups[adjacent_point.get_index()]?;
                let group_data = group_map.get(&index)?;
                if group_data.get_colour() == group_colour {
                    Some(adjacent_point)
                } else {
                    None
                }
            })
            .collect();
    
        let surrounding_groups: HashSet<&GroupState> = positions
            .iter()
            .flat_map(|&position| {
                let potential_group_id = groups[position.get_index()].expect("Group ID cannot be None");
                let potential_group = group_map.get(&potential_group_id).expect("Group not found in group_map");
                if potential_group.get_colour() == group.get_colour() {
                    Some(potential_group)
                } else {
                    None
                }
            })
            .collect();
    
        surrounding_groups.into_iter().collect()
    }

    /// return the new list of groups after checking to remove adjacent groups with 0 liberties
    pub fn check_opposing_groups_liberties(total_groups: &Vec<Option<usize>>, map: &HashMap<usize, GroupState>, size: usize, groups: &Vec<&GroupState>) -> (Vec<Option<usize>>, bool) {
        // for each group in groups
        // update and check that the liberties are above 0 else remove them
        let mut didnt_remove_groups: bool = true; // exists so i can check for suicide

        let grid = BoardState::generate_grid_from_groups(total_groups, map, size);
        let mut ids_to_remove: Vec<usize> = Vec::new();

        for group in groups {
            if !group.check_liberties(&grid, size) {
                println!("removing group with id {}", group.get_id());
                ids_to_remove.push(group.get_id());
                didnt_remove_groups = false;
            }
        }

        (BoardState::remove_groups(total_groups, ids_to_remove), didnt_remove_groups)
    }

    /// check and remove the opposing adjacent 0 liberty groups
    pub fn check_opposing_adjacent_liberties(total_groups: &Vec<Option<usize>>, map: &HashMap<usize, GroupState>, size: usize, position: Coordinate) -> (Vec<Option<usize>>, bool) {
        let id = total_groups[position.get_index()].expect("Wrong position given");
        let group = map.get(&id).expect("Wrong id provided");
        let group_colour = group.get_colour();
        let grid = BoardState::generate_grid_from_groups(&total_groups, &map, size);

        // we need to get adjacent positions
        // check if a group with the opposite colour is there
        // add that group in to be checked
        // if that checked group is at 0 liberties call the remove

        let mut groups_to_check: Vec<usize> = Vec::new();
        let mut groups_to_remove: Vec<usize> = Vec::new();
        let mut didnt_remove_groups = true;

        // for each position in the main group
        // check that position doesnt contain a group of the opposite colour

        for point in group.get_positions() {
            let adjacent = BoardState::get_adjacent_indices(size, point);
            for adjacent_point in adjacent {
                if let Some(group_id) = total_groups[adjacent_point.get_index()] {
                    if map.get(&group_id).expect("Group not in Map!").get_colour() != group_colour { // this group is the opposite colour to our new group
                        groups_to_check.push(group_id);
                    }
                }
            }
        }

        for group_id in groups_to_check { // find the groups that need to be removed
            if !map.get(&group_id).unwrap().check_liberties(&grid, size) { // if a checked group has 0 liberties
                groups_to_remove.push(group_id);
            }
        }

        (BoardState::remove_groups(total_groups, groups_to_remove), didnt_remove_groups)
    }

    pub fn debug_groups(&self, groups: &Vec<Option<usize>>, map: &HashMap<usize, GroupState>) {
        let grid = BoardState::generate_grid_from_groups(groups, map, self.size);
        println!("-- Total Groups --");
        for (id, group) in map {
            let points = group.get_positions();
            let liberties = group.calculate_liberties(&grid, self.size);
            let colour = group.colour;
            println!("Group {}: {:#?} with positions: {:?} has {} liberties", id, colour, points, liberties);
        }
    }
}