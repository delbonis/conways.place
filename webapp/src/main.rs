#![allow(unused)]
#![feature(integer_atomics)]

#[macro_use] extern crate clap;
extern crate conway;
extern crate websocket;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate tokio_core;
extern crate tokio_channel;
extern crate tokio_timer;
extern crate futures;
extern crate clightningrpc;
extern crate rand;

use std::env;
use std::fmt::Debug;
use std::iter::*;
use std::thread;
use std::sync::*;
use std::sync::atomic::AtomicU64;
use std::path::PathBuf;

use websocket::async::Server;
use websocket::message::{Message, OwnedMessage};
use websocket::server::InvalidConnection;
use futures::{Future, Sink, Stream};
use tokio_core::reactor::{Core, Handle};
use clightningrpc::LightningRPC;

use conway::world;

mod messages;
mod session;
mod gameloop;
mod invoiceloop;
mod payloop;

const PROTO_NAME: &'static str = "gameoflight";
const GAME_TICK_MILLIS: u64 = 500;
const WORLD_SIZE: usize = 256;

/*
 * A lot of this connection code was "borrowed" from:
 * https://github.com/websockets-rs/rust-websocket/blob/master/examples/async-server.rs
 */

fn main() {

    let matches = clap_app!(gameoflightinst =>
        (version: "0.1.0")
        (author: "treyzania <treyzania@gmail.com>")
        (about: "The Conway's Game of Life instance game server.")
        (@arg wsport: --port -p +takes_value "Port to host the websocket server on.  Default: 7908")
        (@arg clsock: --clsocket -s +takes_value "Socket path for c-lightning.  Default: default")
        (@arg testnet: --testnet "Switch to using testnet stuff."))
        .get_matches();

    let ws_port: u16 = matches.value_of("wsport").unwrap_or("7908").parse().unwrap();

    // Figure out the full path to the socket for c-lightning.
    #[allow(deprecated)]
    let cl_sock = matches.value_of("clsock")
        .map(PathBuf::from)
        .map(|r| env::current_dir()
            .unwrap()
            .join(r))
        .unwrap_or(
            env::home_dir()
                .unwrap()
                .join(".lightning/lightning-rpc"));
    println!("c-lightning socket: {}", cl_sock.display());

    let testnet_huh: bool = matches.value_of("testnet").unwrap_or("false").parse().unwrap(); // testnet?

    // Set up event loop and listening port.
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let listener = Server::bind(format!("0.0.0.0:{}", ws_port).as_ref() as &str, &handle).unwrap();

    // Just create a new empty world.
    let st = Arc::new(Mutex::new(
        gameloop::GameState::new(world::World::new((WORLD_SIZE, WORLD_SIZE)))
    ));

    // threads and shit.
    let (ir_send, ir_recv) = mpsc::channel();
    let ist = st.clone();
    let isock = cl_sock.clone();
    thread::spawn(move || invoiceloop::invoice_proc_thread(ir_recv, ist, isock));
    let pst = st.clone();
    let psock = cl_sock.clone();
    thread::spawn(move || payloop::payment_proc_thread(pst, psock));

    // This is where the actual game runs.
    let lst = st.clone();
    thread::spawn(|| gameloop::game_sim_thread(lst));

    // Session ID counter.
    let next_sid = Arc::new(AtomicU64::new(0));

    println!("Websocket server listening on port {}", ws_port);
    let clients = listener
		.incoming()
		// we don't wanna save the stream if it drops
		.map_err(|InvalidConnection { error, .. }| error)
		.for_each(|(upgrade, addr)| {
			println!("Got a connection from: {}", addr);

			// check if it has the protocol we want
			if !upgrade.protocols().iter().any(|s| s == PROTO_NAME) {
				// reject it if it doesn't
				spawn_future(upgrade.reject(), "frik off m8", &handle);
				return Ok(());
			}

            // Make the new session?
            let sid = next_sid.fetch_add(1, atomic::Ordering::AcqRel);
            let (session, recv) = session::Session::new(sid);

            // Clone the event look handle so we can spawn futures in the message handler(s).
            let hclone = handle.clone();
            let wref = st.clone();
            let wref2 = st.clone();
            let irs2 = ir_send.clone();

            // Add the session to the table.
            let current_world; // make a copy of the world state so we can send it to the player
            {
                let mut gs: MutexGuard<gameloop::GameState> = wref.lock().unwrap();
                gs.sessions.insert(sid, session);
                current_world = gs.world.clone();
            }

			// accept the request to be a ws connection if it does
			let f = upgrade
				.use_protocol(PROTO_NAME)
				.accept()
				.and_then(move |(s, _)| s.send(OwnedMessage::Text(messages::NetMessage::new_world_state_message(current_world).to_string()).into())) // this doesn't compile if I remove it
				.and_then(move |s| {
					let (sink, stream) = s.split();
					stream
						.take_while(|m| Ok(!m.is_close()))
						.filter_map(move |m| {
                            let hc = hclone.clone();
							match m {
                                OwnedMessage::Text(t) => Some(OwnedMessage::Text({
                                    handle_str_packet(sid, hc, t, wref2.clone(), irs2.clone()) // not sure why we have to clone these again...
                                })),
								OwnedMessage::Ping(p) => Some(OwnedMessage::Pong(p)),
                                OwnedMessage::Pong(_) => None,
								_ => None,
							}
						})
                        // Also send messages coming in from the queue from the other thread(s).
                        .select(recv
                            .map(|m| {
                                let raw = m.to_string();
                                OwnedMessage::Text(raw)
                            })
                            .map_err(|_| websocket::WebSocketError::NoDataAvailable))
                        .forward(sink)
						.and_then(move |(_, sink)| {
                            // Remove the session from the table.
                            let mut gs: MutexGuard<gameloop::GameState> = wref.lock().unwrap();
                            gs.sessions.remove(&sid);

                            // Send the close message.
                            sink.send(OwnedMessage::Close(None))
                        })
				});

            println!("spawned future");
			spawn_future(f, "Client Status", &handle);
			Ok(())
		});

    core.run(clients).unwrap();

}

fn spawn_future<F, I, E>(f: F, desc: &'static str, handle: &Handle)
where
	F: Future<Item = I, Error = E> + 'static,
	E: Debug,
{
	handle.spawn(f
			.map(move |_| println!("{}: Finished.", desc))
            .map_err(move |e| println!("{}: '{:?}'", desc, e)),
	);
}

fn handle_str_packet(sid: u64, handle: Handle, data: String, gs: Arc<Mutex<gameloop::GameState>>, ir_chan: mpsc::Sender<invoiceloop::InvoiceRequest>) -> String {

    let nm = messages::NetMessage::from_string(&data);
    if let Err(m) = nm {
        return m; // TODO Make this better.
    }

    let res = gameloop::handle_message(sid, nm.unwrap(), handle, gs, ir_chan);
    res.to_string()

}
