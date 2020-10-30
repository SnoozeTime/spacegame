use crate::event::GameEvent;
use crate::resources::Resources;
use shrev::{EventChannel, ReaderId};

pub struct GameOver {
    rdr_id: ReaderId<GameEvent>,
}

impl GameOver {
    pub fn new(resources: &mut Resources) -> Self {
        let mut chan = resources.fetch_mut::<EventChannel<GameEvent>>().unwrap();
        let rdr_id = chan.register_reader();
        Self { rdr_id }
    }

    pub fn game_over(&mut self, resources: &Resources) -> bool {
        let chan = resources.fetch::<EventChannel<GameEvent>>().unwrap();
        for ev in chan.read(&mut self.rdr_id) {
            if let GameEvent::GameOver = ev {
                return true;
            }
        }
        return false;
    }
}
