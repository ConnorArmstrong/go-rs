use crate::{Colour, coordinate::{self, Coordinate}, Board};
use std::{collections::HashSet, hash::Hash};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Group {
    // struct to store groups of stones on the board for ease of calculation
    colour: Colour, // colour of the group
    liberties: usize, // amount of liberties the group has
    points: Vec<Coordinate>, // the locations of the stones in the group
}

/*
When a stone is placed on the board:
    1. A new group is created
    2. Merge groups if needed:
        check the adjacent positions (keep in mind the edge)
        if there are stones of the same colour, merge them
    3. update liberties:
        decrease the liberty count of any adjacent opposing colour groups
        if any opposing colour groups liberties become 0, remove that group and add its stone count to the player
    4. update current group - update the liberties of the newly formed group or the updated group after it merges
*/

impl Group {
    pub fn new(colour: Colour, position: Coordinate) -> Group {
        // creates a simple group with a single stone that is then checked
        Group {
            colour,
            liberties: 4, // updated later
            points: vec![position], // potentially updated later
        }
    }

    pub fn calculate_liberties(&mut self, board: &Board) {
        let mut liberties: HashSet<Coordinate> = HashSet::new();

        for position in &self.points {
            for adjacent in board.get_adjacent_indices(*position)  {
                if (board.get(adjacent) == Colour::Empty) {
                    liberties.insert(adjacent);
                }
            }
        }
        self.liberties = liberties.len()
    }

    pub fn get_positions(&self) -> &Vec<Coordinate> {
        &self.points
    }

    pub fn get_colour(&self) -> Colour {
        self.colour
    }

    pub fn contains(&self, position: Coordinate, colour: Colour) -> bool {
        // if the group contains the a stone of specified colour at a certain colour
        // might be useful for checking a merged group
        if (self.colour == colour) {
            if (self.points.contains(&position)) {
                return true;
            }
        }
        false
    }

    pub fn merge_groups(groups: Vec<Group>, board: &Board) -> Group {
        let colour = groups[0].colour;
        let mut points: HashSet<Coordinate> = HashSet::new();
        for group in groups {
            points.extend(group.points);
        }

        let mut group = Group {
            colour,
            liberties: 0,
            points: points.into_iter().collect(),
        };
        group.calculate_liberties(board);

        assert_ne!(group.liberties, 0); // this would be a problem

        group
    }
}
