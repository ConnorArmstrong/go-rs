use crate::{Colour, coordinate::Coordinate, new_board::NewBoard};
use std::{collections::HashSet, hash::Hash};

#[derive(Clone, Debug)]
pub struct NewGroup {
    pub id: usize,
    pub liberties: usize,
    pub colour: Colour,
    points: HashSet<Coordinate>
}

impl NewGroup {
    pub fn new(id: usize, colour: Colour, position: Coordinate) -> Self {
        let mut points = HashSet::new();
        points.insert(position);

        NewGroup {
            id,
            liberties: 4, // temporary 
            colour,
            points,
        }
    }

    pub fn calculate_liberties(&mut self, grid: &Vec<Colour>) {
        let mut liberties: HashSet<Coordinate> = HashSet::new();

        for position in &self.points {
            for adjacent in NewBoard::get_adjacent_indices(*position)  {
                if grid[adjacent.get_index()] == Colour::Empty {
                    liberties.insert(adjacent);
                }
            }
        self.liberties = liberties.len();
        }
    }

    pub fn check_liberties(&self, grid: &Vec<Colour>) -> bool {
        // returns true if there are more than 0 liberties
        let mut liberties: HashSet<Coordinate> = HashSet::new();

        for position in &self.points {
            for adjacent in NewBoard::get_adjacent_indices(*position)  {
                if grid[adjacent.get_index()] == Colour::Empty {
                    liberties.insert(adjacent);
                }
            }
        }
        return liberties.len() > 0;
    }

    pub fn get_id(&self) -> &usize {
        &self.id
    }

    pub fn get_liberties(&self) -> &usize {
        &self.liberties
    }

    pub fn get_points(&self) -> &HashSet<Coordinate> {
        &self.points
    }

    pub fn get_positions(&self) -> Vec<Coordinate> {
        return self.points.iter().cloned().collect();
    }

    pub fn get_colour(&self) -> Colour {
        match self.colour {
            Colour::Black => Colour::Black,
            Colour::Empty => Colour::Empty,
            Colour::White => Colour::White,
        }
    }

    pub fn decrease_liberties(&mut self) {
        self.liberties -= 1;
    }

    pub fn decrease_and_get_liberties(&mut self) -> &usize {
        self.liberties -= 1;
        &self.liberties
    }

    pub fn contains(&self, coordinate: &Coordinate, colour: &Colour) -> bool {
        // return true if the coordinate is in the hashset of points
        return self.points.contains(coordinate) && self.colour == *colour;
    }

    pub fn merge_groups(id: usize, groups: &Vec<&NewGroup>) -> NewGroup {
        let total_points: HashSet<Coordinate> = groups.iter().flat_map(|group| group.get_points()).cloned().collect();
        let colour = groups[0].get_colour();

        let new_group = NewGroup {
            id,
            colour,
            liberties: 4,
            points: total_points,
        };
        // might need new_group.calculate_liberties();
        new_group
    }
}

// easiest way to handle it assuming all goes well with my board implementation
// as long as the id is always unique for each subsequent new group and i remove the old ones from
// the board struct the id of the group is the only thing needed
impl PartialEq for NewGroup {
    fn eq(&self, other: &Self) -> bool {
        return self.id == other.id
    }
}

impl Eq for NewGroup {}

impl Hash for NewGroup {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.points.len().hash(state);
    }
}