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

use std::fmt::Debug;
use std::iter::*;
use std::thread;
use std::sync::*;
use std::sync::atomic::AtomicU64;

use websocket::async::Server;
use websocket::message::{Message, OwnedMessage};
use websocket::server::InvalidConnection;

use futures::{Future, Sink, Stream};
use tokio_core::reactor::{Core, Handle};

use conway::world;

mod messages;
mod session;
mod gameloop;

const PROTO_NAME: &'static str = "gameoflight";
const GAME_TICK_MILLIS: u64 = 500;
const WORLD_SIZE: usize = 64;

/*
 * A lot of this connection code was "borrowed" from:
 * https://github.com/websockets-rs/rust-websocket/blob/master/examples/async-server.rs
 */

fn main() {

    let matches = clap_app!(gameoflightinst =>
        (version: "0.1.0")
        (author: "treyzania <treyzania@gmail.com>")
        (about: "The Conway's Game of Life instance game server.")
        (@arg wsport: --wp +takes_value "Port to host the websocket server on.  Default: 7908"))
        .get_matches();

    let ws_port: u16 = matches.value_of("wsport").unwrap_or("7908").parse().unwrap();

    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let listener = Server::bind(format!("0.0.0.0:{}", ws_port).as_ref() as &str, &handle).unwrap();

    // Just create a new empty world.
    let st = Arc::new(Mutex::new(
        gameloop::GameState::new(world::World::new((WORLD_SIZE, WORLD_SIZE)))
    ));

    // This is where the actual game runs.
    let tst = st.clone();
    thread::spawn(move || gameloop::game_sim_thread(tst));

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

            // Add the session to the table.
            {
                let mut gs: MutexGuard<gameloop::GameState> = wref.lock().unwrap();
                gs.sessions.push(session);
            }

			// accept the request to be a ws connection if it does
			let f = upgrade
				.use_protocol(PROTO_NAME)
				.accept()
				.and_then(move |(s, _)| s.send(Message::text("Hello!").into())) // this doesn't compile if I remove it
				.and_then(move |s| {
					let (sink, stream) = s.split();
					stream
						.take_while(|m| Ok(!m.is_close()))
						.filter_map(move |m| {
                            let hc = hclone.clone();
							match m {
                                OwnedMessage::Text(t) => Some(OwnedMessage::Text(handle_str_packet(hc, t, wref2.clone()))), // idk why we have to clone this again
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
                            gs.sessions = gs.sessions.iter() // TODO Make this better!
                                .cloned()
                                .filter(|i| i.id() != sid)
                                .collect();

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

fn handle_str_packet(handle: Handle, data: String, gs: Arc<Mutex<gameloop::GameState>>) -> String {

    let nm = messages::NetMessage::from_string(&data);
    if let Err(m) = nm {
        return m; // TODO Make this better.
    }

    let res = gameloop::handle_message(nm.unwrap(), handle, gs);
    res.to_string()

}
