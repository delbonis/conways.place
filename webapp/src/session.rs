use tokio_channel::mpsc;

use messages;

#[derive(Clone, Debug, Hash)]
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

#[derive(Clone)]
pub struct Session {
    id: u64,
    outqueue: mpsc::UnboundedSender<messages::NetMessage>
}

impl Session {
    pub fn new(id: u64) -> (Session, mpsc::Receiver<messages::NetMessage>) {
        let (s, r) = mpsc::unbounded();
        let session = Session {
            id: id,
            outqueue: s
        };
        (session, r)
    }

    pub fn queue_message(&self, m: messages::NetMessage) {
        self.outqueue.unbounded_send(m);
    }

    pub fn id(&self) -> u64 {
        self.id
    }
}

impl PartialEq for Session {
    fn eq(&self, o: &Self) -> bool {
        self.id == o.id
    }
}
