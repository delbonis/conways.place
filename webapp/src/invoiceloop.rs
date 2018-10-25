use std::sync::*;
use std::path::PathBuf;

use clightningrpc;
use clightningrpc::lightningrpc::LightningRPC;

use gameloop;
use messages;

#[derive(Clone)]
pub struct InvoiceRequest {
    sat: i64,
    label: String, // the one in pending_updates in GameState.  this should be inserted before calling this function
    desc: String,

    dest: u64
}

impl InvoiceRequest {
    pub fn new(dest: u64, sat: i64, label: String, desc: String) -> InvoiceRequest {
        InvoiceRequest {
            sat,
            label,
            desc,
            dest
        }
    }
}

pub fn invoice_proc_thread(reqs: mpsc::Receiver<InvoiceRequest>, gs: Arc<Mutex<gameloop::GameState>>, sock: PathBuf) {

    let mut c = LightningRPC::new(&sock);

    loop {

        // Get the invoice info.
        let got = reqs.recv();

        match got {
            Ok(ir) => {

                let label = ir.label.clone();

                // Create the invoice info.
                match create_invoice(&ir, &mut c) {
                    Ok(iv) => {
                        println!("created invoice {} for tile update {}", iv.bolt11, label);

                        let st = gs.lock().unwrap();
                        match st.sessions.get(&ir.dest) {
                            Some(sess) => {
                                let nm = messages::NetMessage::Invoice(label, iv.bolt11);
                                sess.queue_message(nm);
                            },
                            None => {
                                println!("user ID {} not present anymore, ignoring", ir.dest);
                                continue;
                            }
                        }
                    },
                    Err(e) => {
                        let st = gs.lock().unwrap();
                        match st.sessions.get(&ir.dest) {
                            Some(sess) => {
                                let msg = format!("Could not create invoice, try refreshing the page.\nError:\n{:?}", e);
                                let nm = messages::NetMessage::Alert(msg);
                                sess.queue_message(nm);
                            },
                            None => {
                                println!("user ID {} not present anymore, ignoring", ir.dest);
                                continue;
                            }
                        }
                    }
                }
            },
            Err(e) => {
                println!("error processing invoice setup: {:?}", e);
                break;
            }
        }

    }

}

fn create_invoice(req: &InvoiceRequest, client: &mut LightningRPC) -> Result<clightningrpc::responses::Invoice, String> {
    let rc = req.clone();
    client.invoice(rc.sat * 1000, rc.label, rc.desc, None)
        .map_err(|e| {
            println!("error creating invoice: {:?}", e);
            format!("{:?}", e) // this is going to users anyways so formatting it as a string is fine.
        })
}
