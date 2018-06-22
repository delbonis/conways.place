#![allow(unused)]

use actix::prelude::*;
use actix::dev::*;

use world;

pub struct WorldSim {
    name: String,
    w: world::World
}

impl WorldSim {

    pub fn new(name: String, dim: (usize, usize)) -> WorldSim {
        WorldSim {
            name: name,
            w: world::World::new(dim)
        }
    }

    pub fn with_world(name: String, w: world::World) -> WorldSim {
        WorldSim {
            name: name,
            w: w
        }
    }

}

impl Actor for WorldSim {
    type Context = Context<Self>;
}

pub struct WorldResponse(world::World);

impl WorldResponse {
    pub fn world(&self) -> &world::World {
        &self.0
    }
}

impl<A, M> MessageResponse<A, M> for WorldResponse
where
    A: Actor,
    M: Message<Result = WorldResponse> {

        fn handle<R: ResponseChannel<M>>(self, _: &mut A::Context, tx: Option<R>) {
            if let Some(tx) = tx {
                tx.send(self)
            }
        }

}

pub struct MsgRunForTicks(u16);

impl MsgRunForTicks {
    pub fn new(ticks: u16) -> MsgRunForTicks {
        MsgRunForTicks(ticks)
    }
}

impl Message for MsgRunForTicks {
    type Result = WorldResponse;
}

impl Handler<MsgRunForTicks> for WorldSim {
    type Result = WorldResponse;
    fn handle(&mut self, msg: MsgRunForTicks, _ctx: &mut Context<Self>) -> Self::Result {

        println!("Requested {} ticks for world {}", msg.0, self.name);

        for _ in 0..msg.0 {
            self.w = self.w.step();
        }

        println!("Done!");
        WorldResponse(self.w.clone())

    }
}

pub struct MsgSetTileState((usize, usize), bool);

impl Message for MsgSetTileState {
    type Result = WorldResponse;
}

impl Handler<MsgSetTileState> for WorldSim {
    type Result = WorldResponse;
    fn handle(&mut self, msg: MsgSetTileState, _ctx: &mut Context<Self>) -> Self::Result {
        println!("Setting tile ({}, {}) to {}", msg .0 .0, msg .0 .1, msg.1);
        match self.w.cell_at_mut(msg.0) {
            Some(t) => t.live = msg.1,
            None => {}
        }
        WorldResponse(self.w.clone())
    }
}
