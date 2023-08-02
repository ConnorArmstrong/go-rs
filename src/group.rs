use crate::{Colour, Coordinate};

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
            liberties: 4,
            points: vec![position],
        }
    }
}