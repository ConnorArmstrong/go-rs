// a purely function way to represent the board
use std::collections::{HashMap, HashSet};

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{colour::Colour, zobrist::ZobristTable, coordinate::Coordinate, fails::TurnErrors, group_state::GroupState};

#[derive(Clone, Debug)]
pub struct BoardState {
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

    /// allows for the generate_grid_from_groups to be called from outside the struct
    pub fn get_grid(&self) -> Vec<Colour> {
        BoardState::generate_grid_from_groups(&self.groups, &self.group_map, self.size)
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

        let (new_groups, flag) = BoardState::check_opposing_adjacent_liberties(&new_groups, &new_group_map, self.size, coordinate);

        let final_grid = &BoardState::generate_grid_from_groups(&new_groups, &new_group_map, self.size);

        if flag { // no groups were removed 
            let final_group = new_group_map.get(&new_groups[coordinate.get_index()].unwrap_or_else(|| panic!("Failed to unwrap new_groups at index {}", coordinate.get_index()))).unwrap_or_else(|| panic!("Failed to unwrap new_group_map"));
            //println!("liberties: {:?}", final_group.clone().calculate_liberties(final_grid, self.size));            
            if !final_group.check_liberties(final_grid, self.size) { // final merged group has no liberties and didnt remove any other group
                return Err(TurnErrors::Suicide);
            }
        }

        if self.zobrist_table.contains_position(&final_grid) { // check for repeated position/ko
            return Err(TurnErrors::Ko);
        }

        new_zobrist_table.insert_position(&final_grid);
        let final_group_map = BoardState::clean_map(new_group_map, &new_groups);

        Ok(BoardState {
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

    /// return a list of all current groups actually on the board
    pub fn _get_current_groups(&self) -> Vec<GroupState> {
        let current_groups: HashSet<GroupState> = self.groups.clone()
                                                    .into_iter()
                                                    .filter_map(|x| x.and_then(|key| self.group_map.get(&key)))
                                                    .cloned()
                                                    .collect();

        return current_groups.into_iter().collect();
    }

    /// return if the group is currently on the board
    pub fn _contains(&self, group: &GroupState) -> bool {
        let current_groups: HashSet<GroupState> = self.groups.clone()
                                                    .into_iter()
                                                    .filter_map(|x| x.and_then(|key| self.group_map.get(&key)))
                                                    .cloned()
                                                    .collect();
        current_groups.contains(&group)
    }

    /// returns the group at the given coordinate
    pub fn _find_group(&self, coordinate: Coordinate) -> Option<&GroupState> {
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

    /// return a list of adjacent same colour groups given a specific coordinate
/// return a list of adjacent same colour groups given a specific coordinate
    pub fn find_adjacent_groups<'a>(groups: &'a Vec<Option<usize>>, group_map: &'a HashMap<usize, GroupState>, coordinate: Coordinate, group_colour: Colour, size: usize) -> Vec<&'a GroupState> {
        let group_id = groups[coordinate.get_index()].expect("Group ID cannot be None");
        let group = group_map.get(&group_id).expect("Group not found in group_map");

        let surrounding_groups: HashSet<&GroupState> = group
            .get_positions()
            .iter()
            .flat_map(|point| BoardState::get_adjacent_indices(size, *point))
            .filter(|&adjacent_point| !group.get_points().contains(&adjacent_point))
            .filter_map(|adjacent_point| {
                let index = groups[adjacent_point.get_index()]?;
                let group_data = group_map.get(&index)?;
                if group_data.get_colour() == group_colour {
                    Some(group_data)
                } else {
                    None
                }
            })
            .collect();

        surrounding_groups.into_iter().collect()
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
                didnt_remove_groups = false;
            }
        }

        (BoardState::remove_groups(total_groups, groups_to_remove), didnt_remove_groups)
    }

    pub fn _debug_groups(&self, groups: &Vec<Option<usize>>, map: &HashMap<usize, GroupState>) {
        let grid = BoardState::generate_grid_from_groups(groups, map, self.size);
        println!("-- Total Groups --");
        for (id, group) in map {
            let points = group.get_positions();
            let liberties = group.calculate_liberties(&grid, self.size);
            let colour = group.colour;
            println!("Group {}: {:#?} with {:} positions: {:?} has {} liberties", id, colour, points.len(), points, liberties);
        }
    }

    pub fn debug_selection(&self, coordinate: Coordinate) {
        if self.groups[coordinate.get_index()] == None {
            println!("Error! This is an Empty Square");
            return;
        }

        let id = self.groups[coordinate.get_index()].expect("Didnt find an ID");
        let group = self.group_map.get(&id).unwrap();

        println!("-- GROUP INFO --");
        let points = group.get_positions();
        let liberties = group.calculate_liberties(&self.get_grid(), self.size);
        let colour = group.colour;
        println!("Group {}: {:#?} with positions: {:?} has {} liberties", id, colour, points, liberties);
    }

    /// Assuming game finished (and no dead stones) this will return how many empty spaces are enclosed by a particular colour
    pub fn get_surrounded_area(&self, grid: &Vec<Colour>, colour: Colour) -> usize {
        let mut empty_locations: HashSet<Coordinate> = HashSet::new();
        let groups: Vec<&GroupState> = self.group_map.values().filter(|&group| group.colour == colour).collect();
    
        let mut queue: Vec<Coordinate> = groups
            .iter()
            .flat_map(|group| group.get_positions())
            .flat_map(|coordinate| BoardState::get_adjacent_indices(self.size, coordinate))
            .collect();
    
        queue.retain(|&adjacent_coord| {
            let index = adjacent_coord.get_index();
            grid.get(index) == Some(&Colour::Empty)
        });
    
        while let Some(position) = queue.pop() {
            if grid[position.get_index()] == Colour::Empty && empty_locations.insert(position) {
                let neighbours: Vec<Coordinate> = BoardState::get_adjacent_indices(self.size, position)
                    .iter()
                    .filter(|position| grid[position.get_index()] == Colour::Empty)
                    .copied()
                    .collect();
                queue.extend(neighbours);
            }
        }
    
        empty_locations.len()
    }

    pub fn get_colour_territory(&self, grid: &Vec<Colour>) -> (usize, usize) {
        let mut empty_locations: [HashSet<Coordinate>; 2] = [HashSet::new(), HashSet::new()];
        let groups: Vec<&GroupState> = self.group_map.values().collect();
    
        let mut queue: Vec<(Colour, Coordinate)> = groups
            .iter()
            .flat_map(|group| group.get_positions().into_iter().map(move |pos| (group.colour, pos)))
            .flat_map(|(colour, coordinate)| BoardState::get_adjacent_indices(self.size, coordinate).into_iter().map(move |pos| (colour, pos)))
            .collect();
    
        queue.retain(|&(colour, adjacent_coord)| {
            let index = adjacent_coord.get_index();
            grid.get(index) == Some(&Colour::Empty)
        });
    
        while let Some((colour, position)) = queue.pop() {
            if grid[position.get_index()] == Colour::Empty && empty_locations[colour.into_usize() - 1].insert(position) {
                let neighbours: Vec<(Colour, Coordinate)> = BoardState::get_adjacent_indices(self.size, position)
                    .iter()
                    .filter(|position| grid[position.get_index()] == Colour::Empty)
                    .map(|&position| (colour, position))
                    .collect();
                queue.extend(neighbours);
            }
        }
    
        (empty_locations[0].len(), empty_locations[1].len())
    }

    /// returns a list of all the empty territory surrounded by the given colour (in the form of a vec of GroupState)
    pub fn get_territory(&self, grid: &[Colour], colour: Colour) -> Vec<GroupState> {
        let mut empty_groups: HashSet<GroupState> = HashSet::new();

        let empty_points: Vec<Coordinate> = grid // collects all the empty coordinates
            .iter()
            .enumerate()
            .filter(|(_, value)| value == &&Colour::Empty)
            .map(|(index, _)| Coordinate::Index(index))
            .collect();

        for position in empty_points {
            empty_groups.insert(self.create_empty_group(&grid, position)); // inefficient implementation - this goes through every empty point and constructs a group from it's surrounding neighbours
        } // after this we now have every empty group on the board

        let mut surrounded_territory: Vec<GroupState> = Vec::new(); // surrounded territory of a particular colour

        for empty_group in empty_groups {
            let mut adjacents: HashSet<Colour> = HashSet::new();

            for empty_point in empty_group.get_positions() {
                let a = BoardState::get_adjacent_indices(self.size, empty_point);
                adjacents.extend(a.iter().map(|&coord| grid[coord.get_index()]));
            }

            if adjacents.contains(&colour.swap_turn()) {
                continue;
            } else {
                surrounded_territory.push(empty_group);
            }
        }
        surrounded_territory
    }

    /// check if all empty spaces are only surrounded by one colour -> once this is true i can force the end of the game
    pub fn check_all_important_points_played(&self) -> bool {
        let grid = self.get_grid();
        let mut non_empty_count = 0;
        let mut empty_points = HashSet::new();
        let mut visited: HashSet<Coordinate> = HashSet::new();

        for (index, colour) in grid.iter().enumerate() {
            match colour {
                Colour::Empty => {
                    if !visited.contains(&Coordinate::Index(index)) {
                        let empty_group = self.create_empty_group(&grid, Coordinate::Index(index));
                        visited.extend(empty_group.get_positions());
                        empty_points.insert(empty_group);
                    }
                }
                _ => non_empty_count += 1,
            }
        }

        if non_empty_count < 5 { // the game wont finish in 5 moves so this can be decided confidently
            return false;
        }

        let is_all_played = empty_points.par_iter().all(|empty_group| {
            let adjacents: HashSet<Colour> = empty_group.get_positions()
                .into_iter()
                .flat_map(|empty_point| BoardState::get_adjacent_indices(self.size, empty_point))
                .map(|coord| grid[coord.get_index()])
                .collect();
    
            !(adjacents.contains(&Colour::Black) && adjacents.contains(&Colour::White))
        });
    
        is_all_played
    }

    /// Goes through all adjacent points to create a group of empty "territory"
    /// 
    /// This is a helper function for check_all_important_points_played() in order to build up the empty groups
    pub fn create_empty_group(&self, grid: &[Colour], coordinate: Coordinate) -> GroupState {
        let mut points: HashSet<Coordinate> = HashSet::new();
        let mut queue = vec![coordinate];

        while let Some(position) = queue.pop() {
            if !points.contains(&position) {
                let adjacent_empty_points: Vec<Coordinate> = BoardState::get_adjacent_indices(self.size, position).iter().filter(|p| grid[p.get_index()] == Colour::Empty)
                    .map(|c| c.to_owned()).collect();

                queue.extend(adjacent_empty_points);
                points.insert(position);
            }
        }
        GroupState::from_empty_points(points)
    }

    /// Recreates a board state given a slice of colours by sequentially playing a given move.
    /// 
    /// This will only ever be called for scoring so there is no need to worry about ko's and previous board
    /// iterations.
    pub fn from_colours(colours: &[Colour], size: usize) -> Self {
        let mut board_state = BoardState::new(size);

        for (i, &colour) in colours.iter().enumerate() {
            if colour != Colour::Empty {
                let coordinate = Coordinate::Index(i);
                board_state = board_state.add_stone(coordinate, colour).unwrap();
            }
        }

        board_state
    }
}