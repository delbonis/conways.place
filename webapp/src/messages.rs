
use conway::world;

#[derive(Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    Alert(String),
    NewWorldState(NewWorldStateMessage),
    UpdateCells(UpdateCellsMessage)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct NewWorldStateMessage {
    world: world::World
}

#[derive(Clone, Serialize, Deserialize)]
pub struct UpdateCellsMessage {
    pos: (usize, usize),
    state: world::Tile
}
