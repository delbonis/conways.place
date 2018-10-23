use std::sync::*;

use serde_json;

use conway::world;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "body")]
pub enum NetMessage {
    Alert(String),
    NewWorldState(NewWorldStateMessage),
    UpdateCells(UpdateCellsMessage)
}

impl NetMessage {
    pub fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    pub fn from_string(m: &String) -> Result<Self, String> { // FIXME Make errors smarter.
        serde_json::from_str(m.as_ref()).map_err(|e| format!("{:?}", e))
    }

    pub fn new_world_state_message(w: Arc<world::World>) -> NetMessage {
        NetMessage::NewWorldState(NewWorldStateMessage {
            world: w
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewWorldStateMessage {
    world: Arc<world::World>
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateCellsMessage {
    pos: (usize, usize),
    state: world::Tile
}
