use crate::core::timer::Timer;
use crate::event::GameEvent;
use crate::gameplay::enemy::Enemy;
use crate::gameplay::player::Player;
use crate::render::sprite::Blink;
use crate::resources::Resources;
use log::{debug, trace};
use serde_derive::{Deserialize, Serialize};
use shrev::{EventChannel, ReaderId};
use std::time::Duration;

/// Health/Hull is the health points of an entity. When those reach 0, then the entity dies.
/// It does not refill over time, so the player will need to refill it with some money.
#[derive(Debug, Serialize, Deserialize)]
pub struct Health {
    /// Maximum amount of health.
    pub max: u32,

    /// Current amount of health.
    pub current: u32,

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

/// Shield will be reduced at first when the player is hit. It will replenish when the player hasn't
/// been hit for some time.
#[derive(Debug, Serialize, Deserialize)]
pub struct Shield {
    /// max amount of shield
    pub max: f32,

    /// current amount of shield
    pub current: f32,

    /// Timer until the shield will start to refill
    timer_until_replenish: Timer,

    /// Amount of shield entity get back per second.
    replenish_rate: f32,
}

impl Shield {
    pub fn new(amt: f32, down_time: f32, replenish_rate: f32) -> Self {
        Self {
            max: amt,
            current: amt,
            timer_until_replenish: Timer::of_seconds(down_time),
            replenish_rate,
        }
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
                    let shield = world.get_mut::<Shield>(*e);

                    let is_enemy = world.get::<Enemy>(*e).is_ok();

                    let has_shield = if let Ok(mut shield) = shield {
                        // reset shield timer. Shield cannot recharge until elapsed.
                        shield.timer_until_replenish.reset();
                        shield.timer_until_replenish.start();
                        if shield.current == 0.0 {
                            false
                        } else {
                            shield.current = (shield.current - 1.0).max(0.0);
                            true
                        }
                    } else {
                        false
                    };

                    // if no shield, then we can hit the health.
                    if !has_shield {
                        if let Ok(mut health) = health {
                            if !health.hittable {
                                continue;
                            }

                            health.current -= 1;
                            if health.is_dead() {
                                debug!("{:?} is dead ({:?}", e, *health);
                                Self::add_death_events(&mut death_events, world, *e, is_enemy);
                            } else {
                                // start invulnerability frames.
                                health.hittable = false;
                                health.invulnerability_timer.reset();
                                health.invulnerability_timer.start();
                                insert_blink = true;
                            }
                        } else {
                            // no shield, no health,  you're dead boy.
                            Self::add_death_events(&mut death_events, world, *e, is_enemy);
                        }
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

        // Then, update shields
        // ----------------------------------------------------
        for (_e, shield) in world.query::<&mut Shield>().iter() {
            shield.timer_until_replenish.tick(dt);
            if shield.timer_until_replenish.finished() {
                shield.current =
                    (shield.current + shield.replenish_rate * dt.as_secs_f32()).min(shield.max);
            }
        }
        trace!("Finished updating HealthSystem");
    }

    fn add_death_events(
        death_events: &mut Vec<GameEvent>,
        world: &hecs::World,
        entity: hecs::Entity,
        is_enemy: bool,
    ) {
        // no shield, no health,  you're dead boy.
        if world.get::<Player>(entity).is_ok() {
            death_events.push(GameEvent::GameOver);
        } else {
            death_events.push(GameEvent::Delete(entity));
        }

        if is_enemy {
            death_events.push(GameEvent::EnemyDied);
        }
    }
}
