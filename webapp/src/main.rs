#![allow(unused)]

#[macro_use] extern crate clap;
extern crate conway;
extern crate websocket;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_json;

use std::iter::*;

use std::net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::thread;

use websocket::OwnedMessage;
use websocket::server::upgrade::sync::Buffer;
use websocket::server::upgrade::WsUpgrade;
use websocket::server::{NoTlsAcceptor, WsServer};
use websocket::sync::Server;

mod messages;
mod session;

fn main() {

    let matches = clap_app!(gameoflightinst =>
        (version: "0.1.0")
        (author: "treyzania <treyzania@gmail.com>")
        (about: "The Conway's Game of Life instance web server.")
        (@arg wsport: --wp +takes_value "Port to host the websocket HTTP server on.  Default: 7908"))
        .get_matches();

    let ws_port: u16 = matches.value_of("wsport").unwrap_or("7908").parse().unwrap();

    let sock_addr = SocketAddr::new("0.0.0.0".parse().unwrap(), ws_port); // TODO Select port.
    let server = Server::bind(sock_addr).unwrap();

    println!("Websocket server listening on port {}", ws_port);
    for request in server.filter_map(Result::ok) {
        if !request.protocols().contains(&"gameoflight".to_string()) {
            println!("not support for gol proto, go away");
            request.reject().unwrap();
            continue;
        }
        let mut client = request.use_protocol("gameoflight").accept().unwrap();
        let ip = client.peer_addr().unwrap();
        println!("Websocket connection from {}", ip);

        let message = OwnedMessage::Text("Hello".to_string());
        client.send_message(&message).unwrap();

        let (mut receiver, mut sender) = client.split().unwrap();
        for message in receiver.incoming_messages() {
            if message.is_err() {
                println!("whoops! {:?}", message.unwrap_err());
                break;
            }

            let msg = message.unwrap();
            match msg {
                OwnedMessage::Close(_) => {
                    let m = OwnedMessage::Close(None);
                    sender.send_message(&m).unwrap();
                    println!("Client {} disconnected", ip);
                    continue;
                },
                OwnedMessage::Ping(ping) => {
                    let m = OwnedMessage::Pong(ping);
                    sender.send_message(&m).unwrap();
                },
                _ => sender.send_message(&msg).unwrap()
            }
        }
    }

}
