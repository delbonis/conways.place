
use std::sync::*;
use std::time;

use tokio_core::reactor::Handle;

use conway::world;

use session;
use messages;

pub struct GameState {
    pub sessions: Vec<session::Session>,
    pub world: world::World
}

impl GameState {
    pub fn new(w: world::World) -> GameState {
        GameState {
            sessions: vec![],
            world: w
        }
    }
}

pub fn game_sim_thread(_st: Arc<Mutex<GameState>>) {
    // TODO
}

// This should really be all contained.
pub fn handle_message(msg: messages::NetMessage, handle: Handle, gs: Arc<Mutex<GameState>>) -> messages::NetMessage {
    unimplemented!()
}
