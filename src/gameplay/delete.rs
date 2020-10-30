//! Clean entities the right way. Done at the end of a frame.

use crate::event::GameEvent;
use crate::resources::Resources;
use log::{debug, info};
use shrev::{EventChannel, ReaderId};

/// ahahaha what a confusing name.
pub struct GarbageCollector {
    rdr_id: ReaderId<GameEvent>,
}

impl GarbageCollector {
    pub fn new(resources: &mut Resources) -> Self {
        let mut chan = resources.fetch_mut::<EventChannel<GameEvent>>().unwrap();
        let rdr_id = chan.register_reader();
        Self { rdr_id }
    }

    pub fn collect(&mut self, world: &mut hecs::World, resources: &Resources) {
        let chan = resources.fetch::<EventChannel<GameEvent>>().unwrap();
        for ev in chan.read(&mut self.rdr_id) {
            if let GameEvent::Delete(e) = ev {
                log::debug!("Will delete {:?}", e);
                // remove from world
                if let Err(e) = world.despawn(*e) {
                    info!("Entity was already deleted (or does not exist?) = {}", e);
                } else {
                    debug!("Entity successfully deleted.");
                }
            }
        }
    }
}
