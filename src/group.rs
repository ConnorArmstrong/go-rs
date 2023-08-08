use crate::{Colour, coordinate::Coordinate, Board};
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Group {
    // struct to store groups of stones on the board for ease of calculation
    colour: Colour, // colour of the group
    liberties: usize, // amount of liberties the group has
    points: HashSet<Coordinate>, // the locations of the stones in the group
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
            points: HashSet::from_iter(vec![position].into_iter()), // potentially updated later
        }
    }

    pub fn calculate_liberties(&mut self, board_grid: &Vec<Colour>) {
        let mut liberties1: HashSet<Coordinate> = HashSet::new();

        for position in &self.points {
            for adjacent in Board::get_adjacent_indices(*position)  {
                if board_grid[adjacent.get_index()] == Colour::Empty {
                    liberties1.insert(adjacent);
                }
            }
        }
        let x = liberties1.len();

        println!("{liberties1:?}, {x:} liberties");
        
        self.liberties = liberties1.len();
        assert_eq!(self.liberties, x);
    }

    pub fn get_positions(&self) -> Vec<&Coordinate> {
        //convert the points hashset into a vector:
        Vec::from_iter(self.points.iter())
    }

    pub fn get_points(&self) -> &HashSet<Coordinate>  {
        &self.points
    }

    pub fn get_colour(&self) -> Colour {
        self.colour
    }

    pub fn get_liberties(&self) -> &usize {
        &self.liberties
    }

    pub fn decrease_liberties(&mut self) {
        self.liberties -= 1;
    }

    pub fn decrease_and_get_liberties(&mut self) -> usize {
        self.liberties -= 1;
        self.liberties
    }

    pub fn contains(&self, position: Coordinate, colour: Colour) -> bool {
        // if the group contains the a stone of specified colour at a certain colour
        // might be useful for checking a merged group
        if self.colour == colour {
            if self.points.contains(&position) {
                return true;
            }
        }
        false
    }

    pub fn merge_groups(groups: &Vec<Group>) -> Group {
        let colour = groups[0].colour;
        let mut points: HashSet<Coordinate> = HashSet::new();

        for group in groups {
            for point in group.get_positions() {
                points.insert(*point);
            }
        }

        let group = Group {
            colour,
            liberties: 0,
            points: points.into_iter().collect(),
        };
        //group.calculate_liberties(board);
        // CURRENTLY COMMENTED OUT AS THIS IS NOW DONE IN THE BOARD::ADD_STONE() FUNCTION
        //assert!(0 != group.liberties); // this would be a problem
        
        group
    }

    pub fn find_mergeable_groups(&self, board: &mut Board) -> Vec<Group> {
        let location = *self.points.iter().next().expect("No points in group");
        let positions = Board::get_adjacent_indices(location);
        let mut groups: Vec<Group> = vec![self.clone()];

        for position in positions {
            if board.get(position) == self.colour { 
                // if the adjacent position is the same colour as the current group
                if let Some(group) = board.find_group(position, self.colour) {
                    groups.push(group.clone());
                }
            }
        }

        groups
    }
}
