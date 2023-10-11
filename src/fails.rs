
#[derive(Debug)]
pub enum TurnErrors { // errors regarding making moves
    AlreadyPlaced,
    Ko,
    Suicide,
    OutofBounds,
}

pub enum TreeErrors { // errors regarding tree navigation
    BelowZero,
    AboveMax,
}