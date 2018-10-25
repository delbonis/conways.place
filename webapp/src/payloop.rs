use std::path::PathBuf;
use std::sync::*;

use clightningrpc;
use clightningrpc::lightningrpc::LightningRPC;

use gameloop;
use messages;

pub fn payment_proc_thread(gs: Arc<Mutex<gameloop::GameState>>, sock: PathBuf) {

    let mut c = LightningRPC::new(&sock);
    let mut last = None;

    loop {

        let got = c.waitanyinvoice(last);

        match got {
            Ok(v) => {

                // First-off, update the last pay index so that we can advance forward in processing.
                last = v.pay_index;

                let label = &v.label;

                let mut st = gs.lock().unwrap(); // FIXME
                match st.pending_updates.remove(label) {
                    Some((sid, updates)) => {

                        println!("invoice paid: {}", label);

                        // First, apply the updates we want.
                        let mut nw = st.world.as_ref().clone();
                        updates.iter()
                            .for_each(|u| nw.set_tile_liveness((u.x, u.y), u.live));

                        // Replace the world in the game state.
                        st.world = Arc::new(nw);

                        // Now send the notification to the user.
                        let nm = messages::NetMessage::InvoicePaid(label.clone());
                        match st.sessions.get(&sid) {
                            Some(sess) => sess.queue_message(nm),
                            None => println!("couldn't find seession with newly-paid invoice, they may have left")
                        }

                    },
                    None => println!("got payment on invoice we don't know about: {}", label)
                }

            },
            Err(e) => println!("error awaiting next payment: {}", e)
        }

    }

}
