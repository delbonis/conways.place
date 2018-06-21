extern crate actix;
extern crate futures;

use actix::prelude::*;

use futures::Future;

mod sim;
mod world;

fn main() {
    let system = System::new("test");
    let addr: Addr<Unsync, _> = sim::WorldSim::new("test".into(), (1000, 1000)).start();
    let res = addr.send(sim::MsgRunForTicks::new(1));
    Arbiter::handle().spawn(
        res.map(|r| {
            println!("Simulation done!")
        })
        .map_err(|e| {
            println!("Fatal Error: {}", e)
        })
    );
    system.run();
}
