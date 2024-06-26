use crate::{colour::Colour, coordinate::Coordinate, board_state::BoardState};
use std::{collections::HashSet, hash::Hash};

#[derive(Clone, Debug)]
pub struct GroupState {
    id: usize,
    pub colour: Colour,
    pub points: HashSet<Coordinate>,
}

impl GroupState {
    /// creates a new groupstate given an id, position and colour of size 1
    pub fn new(id: usize, colour: Colour, position: Coordinate) -> Self {
        GroupState { id, colour, points: HashSet::from_iter(vec![position].into_iter()) }
    }

    /// merges multiple groupstates into a single group
    pub fn merge_groups(id: usize, groups: &Vec<&GroupState>) -> GroupState {
        let colour = groups[0].colour; // make a more robust check in the future

        let points: HashSet<Coordinate> = HashSet::from_iter(groups.iter().flat_map(|group| group.points.iter().cloned()));

        GroupState {
            id,
            colour, 
            points,
        }
    }

    pub fn combine_groups(groups: &Vec<GroupState>) -> HashSet<Coordinate> {
        let points: HashSet<Coordinate> = HashSet::from_iter(groups.iter().flat_map(|group| group.points.iter().cloned()));

        points
    }

    /// returns true if there are more than 0 liberties (calculated every time)
    pub fn check_liberties(&self, grid: &Vec<Colour>, board_size: usize) -> bool {
        let mut liberties: HashSet<Coordinate> = HashSet::new();

        for position in &self.points {
            for adjacent in BoardState::get_adjacent_indices(board_size, *position)  {
                if grid[adjacent.get_index()] == Colour::Empty {
                    liberties.insert(adjacent);
                }
            }
        }
        return liberties.len() > 0;
    }

    /// debug function to check how many liberties a group has
    pub fn calculate_liberties(&self, grid: &Vec<Colour>, board_size: usize) -> usize {
        let mut liberties: HashSet<Coordinate> = HashSet::new();

        for position in &self.points {
            for adjacent in BoardState::get_adjacent_indices(board_size, *position)  {
                if grid[adjacent.get_index()] == Colour::Empty {
                    liberties.insert(adjacent);
                }
            }
        }
        return liberties.len()
    }

    /// returns true if the group contains the given position
    pub fn _contains(&self, position: Coordinate) -> bool {
        self.points.contains(&position)
    }

    pub fn get_points(&self) -> &HashSet<Coordinate> {
        &self.points
    }

    pub fn get_positions(&self) -> Vec<Coordinate> {
        return self.points.iter().cloned().collect();
    }

    pub fn get_colour(&self) -> Colour {
        self.colour
    }

    pub fn _get_id(&self) -> usize {
        self.id
    }

    pub fn from_empty_points(points: HashSet<Coordinate>) -> Self {
        GroupState {
            id: 777, // making number - shoot me
            colour: Colour::Empty,
            points,
        }
    }
}


// easiest way to handle it assuming all goes well with my board implementation
// as long as the id is always unique for each subsequent new group and i remove the old ones from
// the board struct the id of the group is the only thing needed
impl PartialEq for GroupState {
    fn eq(&self, other: &Self) -> bool {
        return self.id == other.id
    }
}

impl Eq for GroupState {}

impl Hash for GroupState {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.points.len().hash(state);
        self.points.iter().map(|coordinate| coordinate.get_index()).sum::<usize>().hash(state);
    }
}