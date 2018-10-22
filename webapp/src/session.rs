use tokio_channel::mpsc;

use conway::world;

use messages;

pub struct User {
    id: u32,
    token: String,
    camera_pos: Option<(i32, i32)>
}

impl User {
    pub fn new(id: u32, token: &str) -> User {
        User {
            id: id,
            token: String::from(token),
            camera_pos: None
        }
    }
    pub fn set_pos(&mut self, pos: (i32, i32)) {
        self.camera_pos = Some(pos)
    }
    pub fn clear_pos(&mut self) {
        self.camera_pos = None
    }
    pub fn pos(&self) -> Option<(i32, i32)> {
        self.camera_pos
    }
}

pub struct Session {
    pub world_updates: mpsc::Sender<world::World>
}

impl Session {
    pub fn new() -> (Session, mpsc::Receiver<world::World>) {
        let (s, r) = mpsc::channel(8);
        let session = Session {
            world_updates: s // is 8 enough?  I think it should be.
        };
        (session, r)
    }
}
