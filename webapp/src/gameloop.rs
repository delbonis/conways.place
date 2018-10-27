
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
        make_rpentomino((64, 64), &mut w);
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

        // Figure out the next world state.
        let prev;
        let current;
        let sessions;
        {
            let mut state = st.lock().unwrap();
            prev = state.world.as_ref().clone();

            // Compute the next step.
            let next_step = Arc::new(prev.step());
            //println!("{}", next_step); // TODO Report this somewhere else.

            // Update.
            state.world = next_step.clone(); // should move this clone outside the lock region?
            current = state.world.clone();

            // Cache these for the next step.
            sessions = state.sessions.clone();
        };

        // This looks scary, but basically just makes a vec of all the cell changes across states.
        let diffs = prev.cells().iter()
            .enumerate()
            .zip(current.cells().iter())
            .map(|((i, p), c)| (i, p, c))
            .filter_map(|(i, p, c)| if *p != *c {
                Some((i, c))
            } else {
                None
            })
            .map(|(i, c)| (world::index_to_cartesean(current.dims(), i).unwrap(), c))
            .map(|(pos, c)| messages::UpdateCellMessage {
                pos: pos,
                state: *c
            })
            .collect();

        // Send the state updates to all the player sessions.
        let msg = messages::NetMessage::UpdateCells(diffs);
        for s in sessions.iter() { // idk why I have to .iter() this
            s.1.queue_message(msg.clone());
        }

        // Sleep for a second.
        // TODO Make it so it sleeps until 250ms has passed *since the last sleep finished*.
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
