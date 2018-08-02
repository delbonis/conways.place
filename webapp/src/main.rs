#![allow(unused)]
#![feature(plugin)]
#![plugin(rocket_codegen)]

#[macro_use] extern crate clap;
extern crate rocket;
extern crate websocket;

use std::iter::*;

use std::net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::thread;

use rocket::config::{Config, Environment};
use rocket::response::NamedFile;

use websocket::OwnedMessage;
use websocket::server::upgrade::sync::Buffer;
use websocket::server::upgrade::WsUpgrade;
use websocket::server::{NoTlsAcceptor, WsServer};
use websocket::sync::Server;

fn main() {

    let matches = clap_app!(gameoflightinst =>
        (version: "0.1.0")
        (author: "treyzania <treyzania@gmail.com>")
        (about: "The Conway's Game of Life instance web server.")
        (@arg resport: --rp +takes_value "Port to host the primary HTTP server on.  Default: 8801")
        (@arg wsport: --wp +takes_value "Port to host the websocket HTTP server on.  Default: 8802"))
        .get_matches();

    let res_port: u16 = matches.value_of("resport").unwrap_or("8801").parse().unwrap();
    let ws_port: u16 = matches.value_of("wsport").unwrap_or("8802").parse().unwrap();

    // Start the normal web server.
    thread::spawn(move || {
        let config = rocket::Config::build(Environment::Staging)
            .address("127.0.0.1")
            .port(res_port)
            .finalize()
            .unwrap();
        rocket::custom(config, true)
            .mount("/", routes![http_main, http_filename])
            .launch();
    });

    // Start the websocket listener.
    thread::spawn(move || {
        let sock_addr = SocketAddr::from(([127, 0, 0, 1], ws_port));
        let server = Server::bind(sock_addr).unwrap();
        println!("Websocket server listening on port {}", ws_port);
        for request in server.filter_map(Result::ok) {
            if !request.protocols().contains(&"gameoflight".to_string()) {
                request.reject().unwrap();
                return;
            }
            let mut client = request.use_protocol("gameoflight").accept().unwrap();
            let ip = client.peer_addr().unwrap();
            println!("Websocket connection from {}", ip);

            let message = OwnedMessage::Text("Hello".to_string());
            client.send_message(&message).unwrap();

            let (mut receiver, mut sender) = client.split().unwrap();
            for message in receiver.incoming_messages() {
                let msg = message.unwrap();
                match msg {
                    OwnedMessage::Close(_) => {
                        let m = OwnedMessage::Close(None);
                        sender.send_message(&m).unwrap();
                        println!("Client {} disconnected", ip);
                        return;
                    },
                    OwnedMessage::Ping(ping) => {
                        let m = OwnedMessage::Pong(ping);
                        sender.send_message(&m).unwrap();
                    },
                    _ => sender.send_message(&msg).unwrap()
                }
            }
        }
    });

    loop {
        #[allow(deprecated)]
        thread::sleep_ms(1000u32);
    }

}

fn get_static_file_path(name: &str) -> PathBuf {
    let mut buf = PathBuf::new();
    buf.push("static");
    buf.push(name);
    buf
}

#[get("/")]
fn http_main() -> NamedFile {
    NamedFile::open(get_static_file_path("index.html")).unwrap()
}

#[get("/<filename>")]
fn http_filename(filename: String) -> NamedFile {
    NamedFile::open(get_static_file_path(filename.as_ref())).unwrap()
}
