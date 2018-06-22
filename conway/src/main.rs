extern crate actix;
extern crate futures;
#[macro_use] extern crate clap;

use std::fs;
use std::io::{self, Read};

use actix::prelude::*;

use futures::Future;

mod sim;
mod world;

fn main() {

    let matches = clap_app!(conwayserv =>
        (version: "0.1.0")
        (author: "treyzania <treyzania@gmail.com>")
        (about: "The Conway's Game of Life server for Game of Light.")
        (@arg state: "File to read state from.")
        (@arg steps: "Number of iterations to run.  Default: 1")
        (@arg dest: "File to write final state to.  Default: stdout"))
        .get_matches();

    let src_file_str = matches.value_of("state").unwrap();
    let steps_str = matches.value_of("steps").unwrap();
    let _out_file_str = matches.value_of("dest").unwrap();

    let w = file_to_world(fs::File::open(src_file_str).unwrap()).unwrap();
    let steps: u16 = steps_str.parse().unwrap();

    println!("in:\n{}", &w);
    let system = System::new("live");
    let addr: Addr<Unsync, _> = sim::WorldSim::with_world("world".into(), w).start();

    for _ in 0..steps {
        let res = addr.send(sim::MsgRunForTicks::new(1));
        Arbiter::handle().spawn(
            res.map(|r| {
                println!("{}", r.world())
            })
            .map_err(|e| {
                println!("Fatal Error: {}", e)
            })
        );
    }

    system.run();
}

fn file_to_world(mut f: fs::File) -> Result<world::World, io::Error> {

    let mut w = 0;
    loop {
        let mut rb = [0; 1];
        f.read_exact(&mut rb)?;
        if rb[0] != 0xA { // newline
            w += 1;
        } else {
            break
        }
    }

    let raw_states: Vec<bool> = f.bytes()
        .map(|b| b.unwrap())
        .filter(|c| *c != 0xA)
        .map(|c| match c {
            0x20 => false,
            0x2e => false,
            _ => true
        })
        .collect();

    println!("states found: {}", raw_states.len());

    let actual_len = raw_states.len() - (raw_states.len() % w);
    let mut living = vec![false; actual_len];
    for i in 0..actual_len {
        living[i] = raw_states[i];
    }

    println!("actual states: {} -> {}", actual_len, living.len());

    Ok(world::World::from_bools((w, actual_len / w), living).unwrap())

}
