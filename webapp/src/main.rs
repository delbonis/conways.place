#[allow(unused)]

#[macro_use] extern crate clap;
extern crate conway;
extern crate websocket;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate tokio_core;
extern crate tokio_channel;
extern crate futures;

use std::fmt::Debug;
use std::iter::*;
use std::thread;
use std::sync::*;

use websocket::async::Server;
use websocket::message::{Message, OwnedMessage};
use websocket::server::InvalidConnection;

use futures::{Future, Sink, Stream};
use tokio_core::reactor::{Core, Handle};

mod messages;
mod session;

const PROTO_NAME: &'static str = "gameoflight";

/*
 * Some of this was "borrowed" from:
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

    // TODO
    let st = Arc::new(Mutex::new(
        GameState {
            sessions: vec![]
        }
    ));

    // This is where the actual game runs.
    thread::spawn(|| game_sim_thread(st));

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
            let (_session, _recv) = session::Session::new();

			// accept the request to be a ws connection if it does
			let f = upgrade
				.use_protocol(PROTO_NAME)
				.accept()
				// send a greeting!
				.and_then(|(s, _)| s.send(Message::text("Hello!").into()))
				// simple echo server impl
				.and_then(|s| {
					let (sink, stream) = s.split();
                    // TODO Add session to GameState
					stream
						.take_while(|m| Ok(!m.is_close()))
						.filter_map(|m| {
							println!("Action: {:?}", m);
							match m {
                                OwnedMessage::Text(t) => Some(OwnedMessage::Text(handle_str_packet(t))),
								OwnedMessage::Ping(p) => Some(OwnedMessage::Pong(p)),
                                OwnedMessage::Pong(_) => None,
								_ => None,
							}
						})
                        // TODO .select(recv.map(...)) here?
                        .forward(sink)
						.and_then(|(_, sink)| sink.send(OwnedMessage::Close(None)))
                        // TODO After it's done then just remove it from the GameState.
				});

			spawn_future(f, "Client Status", &handle);
			Ok(())
		});

    core.run(clients).unwrap();

}

struct GameState {
    sessions: Vec<session::Session>
}

fn spawn_future<F, I, E>(f: F, desc: &'static str, handle: &Handle)
where
	F: Future<Item = I, Error = E> + 'static,
	E: Debug,
{
	handle.spawn(
		f.map_err(move |e| println!("{}: '{:?}'", desc, e))
			.map(move |_| println!("{}: Finished.", desc)),
	);
}

fn handle_str_packet(data: String) -> String {
    data // TODO
}

fn game_sim_thread(_st: Arc<Mutex<GameState>>) {
    // TODO
}
