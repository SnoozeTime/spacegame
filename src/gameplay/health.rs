use crate::core::timer::Timer;
use crate::event::GameEvent;
use crate::gameplay::player::Player;
use crate::render::sprite::Blink;
use crate::resources::Resources;
use log::{debug, trace};
use shrev::{EventChannel, ReaderId};
use std::time::Duration;

#[derive(Debug)]
pub struct Health {
    max: u32,
    current: u32,

    /// true if can hit the component.
    hittable: bool,
    invulnerability_timer: Timer,
}

impl Health {
    pub fn new(health: u32, timer: Timer) -> Self {
        Self {
            max: health,
            current: health,
            hittable: true,
            invulnerability_timer: timer,
        }
    }

    fn is_dead(&self) -> bool {
        self.current == 0
    }
}

pub struct HealthSystem {
    rdr_id: ReaderId<GameEvent>,
}

impl HealthSystem {
    pub fn new(resources: &mut Resources) -> Self {
        let mut chan = resources.fetch_mut::<EventChannel<GameEvent>>().unwrap();
        let rdr_id = chan.register_reader();
        Self { rdr_id }
    }

    pub fn update(&mut self, world: &mut hecs::World, resources: &Resources, dt: Duration) {
        trace!("Update HealthSystem");
        let mut chan = resources.fetch_mut::<EventChannel<GameEvent>>().unwrap();
        let mut death_events = vec![];

        // FIRST, PROCESS ALL EVENTS TO SEE IF ANYBODY GOT HIT
        // ----------------------------------------------------
        for ev in chan.read(&mut self.rdr_id) {
            if let GameEvent::Hit(e) = ev {
                debug!("Process HIT event for {:?}", e);

                let mut insert_blink = false;
                {
                    let health = world.get_mut::<Health>(*e);
                    if let Ok(mut health) = health {
                        if !health.hittable {
                            continue;
                        }

                        health.current -= 1;
                        if health.is_dead() {
                            debug!("{:?} is dead ({:?}", e, *health);
                            if world.get::<Player>(*e).is_ok() {
                                death_events.push(GameEvent::GameOver);
                            } else {
                                death_events.push(GameEvent::Delete(*e));
                            }
                        } else {
                            // start invulnerability frames.
                            health.hittable = false;
                            health.invulnerability_timer.reset();
                            health.invulnerability_timer.start();
                            insert_blink = true;
                        }
                    } else {
                        log::error!("Entity that has been Hit should have a health component");
                    }
                }

                if insert_blink {
                    debug!("WIll insert blink");
                    world
                        .insert(
                            *e,
                            (Blink {
                                color: [1.0, 0.0, 0.0, 1.0],
                                amplitude: 10.0,
                            },),
                        )
                        .unwrap();
                }
            }
        }

        if !death_events.is_empty() {
            debug!("WIll publish {:?}", death_events);
            chan.drain_vec_write(&mut death_events);
        }

        // THEN, UPDATE INVULNERABILY TIMERS.
        // ----------------------------------------------------
        let mut remove_blink = vec![];
        for (e, h) in world.query::<&mut Health>().iter() {
            h.invulnerability_timer.tick(dt);
            if !h.hittable && h.invulnerability_timer.finished() {
                h.hittable = true;
                h.invulnerability_timer.reset();
                h.invulnerability_timer.stop();
                remove_blink.push(e);
            }
        }
        remove_blink.drain(..).for_each(|e| {
            if let Err(e) = world.remove_one::<Blink>(e) {
                log::error!("Cannot remove blink components = {:?}", e);
            }
        });

        trace!("Finished updating HealthSystem");
    }
}
