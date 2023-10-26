use crate::{Colour, coordinate::Coordinate, Board, board_state::BoardState};
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

    /// returns true if the group contains the given position
    pub fn contains(&self, position: Coordinate) -> bool {
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

    pub fn get_id(&self) -> usize {
        self.id
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
    }
}