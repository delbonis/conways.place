
use std::sync::*;
use std::time;
use std::thread;
use std::collections::HashMap;

use tokio_core::reactor::Handle;
use rand::{self, Rng, RngCore};

use conway::world;

use session;
use messages;
use invoiceloop;

pub struct GameState {
    pub sessions: HashMap<u64, session::Session>,
    pub world: Arc<world::World>,

    pub pending_updates: HashMap<String, (u64, Vec<messages::TileState>)>
}

impl GameState {
    pub fn new(mut w: world::World) -> GameState {
        let pos = vec![
            (100, 100),
            (85, 85),
            (50, 50),
            (50, 150),
            (150, 50),
            (150, 150),
        ];
        for p in pos {
            make_rpentomino(p, &mut w);
        }
        GameState {
            sessions: HashMap::new(),
            world: Arc::new(w),
            pending_updates: HashMap::new()
        }
    }
}

/// Makes an R-pentomino at the position specified.  This should go for a really long time.
fn make_rpentomino(pos: (usize, usize), w: &mut world::World) {

    /*
     * Should look like:
     *  ##
     * ##
     *  #
     */
    let set = vec![
        (1, 0),
        (2, 0),
        (0, 1),
        (1, 1),
        (1, 2)
    ];

    // Just loop through the positions and make them live.
    for p in set {
        w.set_tile_liveness((pos.0 + p.0, pos.1 + p.1), true);
    }
}

pub fn game_sim_thread(st: Arc<Mutex<GameState>>) {

    loop {

        // Get the current world state.
        {
            let mut state = st.lock().unwrap();
            let prev_step = state.world.as_ref().clone();

            // Compute the next step.
            let next_step = Arc::new(prev_step.step());
            //println!("{}", next_step); // TODO Report this somewhere else.

            // Update.
            state.world = next_step.clone(); // should move this clone outside the lock region?

            // Send the new state to all the player sessions.
            // TODO Make diffs instead of just sending the whole world to players.
            // TODO Move this somewhere else.  Should we put an Arc or Rc around worlds?
            for s in state.sessions.iter() { // idk why I have to .iter() this
                s.1.queue_message(messages::NetMessage::new_world_state_message(next_step.clone()))
            }
        };

        // Sleep for a second.
        // TODO Make it so it sleeps until a second has passed *since the last sleep finished*.
        thread::sleep(time::Duration::from_millis(250));

    }

}

// This should really be all contained.
// TODO Move the ir channel somewhere else.  This is a baaad place to pass it in.
pub fn handle_message(sid: u64, msg: messages::NetMessage, _handle: Handle, gs: Arc<Mutex<GameState>>, ir_chan: mpsc::Sender<invoiceloop::InvoiceRequest>) -> messages::NetMessage {
    use messages::NetMessage::*;
    match msg {
        Log(m) => {
            println!("remote: {}", m);
            messages::NetMessage::new_log_msg("ok")
        }
        SubmitTiles(stm) => {
            let label = random_label();
            let mut s = gs.lock().unwrap();

            // Insert the pending update into our cache.
            s.pending_updates.insert(label.clone(), (sid, stm.updates.clone()));

            // Now actually set up to create the invoice to send to the user.
            let cnt = stm.updates.len();
            let desc = format!("payment to set liveness of {} tiles (update label: {})", cnt, label);
            let ir = invoiceloop::InvoiceRequest::new(
                sid,
                cnt as i64 * 10,
                label,
                desc
            );
            ir_chan.send(ir);

            // Return that the invoice will come later.
            messages::NetMessage::new_log_msg("creating invoice...")
        },
        _ => messages::NetMessage::new_alert_msg("lol you can't send that, noob")
    }
}

fn apply_changes_to_world(world: &world::World, chgs: Vec<messages::TileState>) -> world::World {
    let mut nw = world.clone();
    for chg in chgs {
        nw.set_tile_liveness((chg.x, chg.y), chg.live);
    }
    nw
}

const LABEL_LEN: usize = 16;
const LABEL_CHARS: &str = "1234567890abcdef";
const LABEL_PREFIX: &str = "conway-";

fn random_label() -> String {
    // FIXME This shouldn't be as hard as it is.

    let tmp = String::from(LABEL_CHARS); // FIXME NO NO NO NO THIS IS SO BAD.

    let mut rng = rand::thread_rng();
    let mut buf = String::with_capacity(LABEL_LEN + LABEL_PREFIX.len());
    buf.push_str(LABEL_PREFIX);
    for _ in 0..LABEL_LEN {
        buf.push(tmp.chars().nth(rng.next_u32() as usize % tmp.len()).unwrap());
    }

    buf
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EditWindow {
    xpos: usize,
    ypos: usize,

    width: usize,
    height: usize
}
