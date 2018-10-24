
use std::sync::*;
use std::time;
use std::thread;

use tokio_core::reactor::Handle;

use conway::world;

use session;
use messages;

pub struct GameState {
    pub sessions: Vec<session::Session>,
    pub world: Arc<world::World>
}

impl GameState {
    pub fn new(mut w: world::World) -> GameState {
        make_rpentomino((20, 20), &mut w);
        GameState {
            sessions: vec![],
            world: Arc::new(w)
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
                s.queue_message(messages::NetMessage::new_world_state_message(next_step.clone()))
            }
        };

        // Sleep for a second.
        // TODO Make it so it sleeps until a second has passed *since the last sleep finished*.
        thread::sleep(time::Duration::from_millis(100));

    }

}

// This should really be all contained.
pub fn handle_message(msg: messages::NetMessage, _handle: Handle, gs: Arc<Mutex<GameState>>) -> messages::NetMessage {
    use messages::NetMessage::*;
    match msg {
        Log(m) => {
            println!("remote: {}", m);
            messages::NetMessage::new_log_msg("ok")
        }
        SubmitTiles(stm) => {
            let mut s = gs.lock().unwrap();
            let nw = apply_changes_to_world(&s.world, stm.updates);
            s.world = Arc::new(nw);
            messages::NetMessage::new_log_msg("updates applied")
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EditWindow {
    xpos: usize,
    ypos: usize,

    width: usize,
    height: usize
}
