use std::sync::*;

use serde_json;

use conway::world;

use gameloop;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "body")]
pub enum NetMessage {
    Alert(String),
    Log(String),
    NewWorldState(NewWorldStateMessage),
    UpdateCells(UpdateCellsMessage),
    RequestEditWindow,
    UpdateEditWindow(gameloop::EditWindow),
    SubmitTiles(SubmitTilesMessage),
    Invoice(String, String), // ("id", "body")
    InvoicePaid(String) // "id"
}

impl NetMessage {
    pub fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    pub fn from_string(m: &String) -> Result<Self, String> { // FIXME Make errors smarter.
        serde_json::from_str(m.as_ref()).map_err(|e| format!("{:?}", e))
    }

    pub fn new_alert_msg(txt: &str) -> NetMessage {
        NetMessage::Alert(String::from(txt))
    }

    pub fn new_log_msg(txt: &str) -> NetMessage {
        NetMessage::Log(String::from(txt))
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubmitTilesMessage {
    pub updates: Vec<TileState>
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct TileState {
    pub x: usize,
    pub y: usize,
    pub live: bool
}
