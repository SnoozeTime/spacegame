use crate::core::input::Input;
use crate::core::random::RandomGenerator;
use crate::core::timer::Timer;
use crate::core::transform::Transform;
use crate::event::GameEvent;
use crate::gameplay::collision::{aabb_intersection, BoundingBox, CollisionLayer};
use crate::gameplay::health::{Health, Invulnerable, Shield};
use crate::gameplay::inventory::Inventory;
use crate::gameplay::physics::DynamicBody;
use crate::gameplay::player::Player;
use crate::gameplay::Action;
use crate::render::sprite::Sprite;
use crate::resources::Resources;
use rand::distributions::{Distribution, Standard};
use rand::Rng;
use serde_derive::{Deserialize, Serialize};
use shrev::EventChannel;

pub struct Pickup {
    pub item: Items,
}

pub fn spawn_pickup(
    world: &mut hecs::World,
    pos: glam::Vec2,
    random: &mut RandomGenerator,
) -> hecs::Entity {
    let item: Items = random.rng().gen();
    world.spawn((
        Invulnerable,
        Transform {
            translation: pos,
            scale: glam::vec2(20.0, 20.0),
            rotation: 0.0,
            dirty: false,
        },
        Sprite {
            id: "capsule.png".to_string(),
        },
        Pickup { item },
        BoundingBox {
            half_extend: glam::vec2(10.0, 10.0),
            collision_layer: CollisionLayer::PICKUP,
            collision_mask: None,
        },
    ))
}

pub fn is_pickup(world: &hecs::World, entity: hecs::Entity) -> bool {
    world.get::<Pickup>(entity).is_ok()
}

pub fn process_pickups(world: &mut hecs::World, resources: &Resources) {
    let mut channel = resources.fetch_mut::<EventChannel<GameEvent>>().unwrap();
    let input = resources.fetch::<Input<Action>>().unwrap();
    let mut inventory = resources.fetch_mut::<Inventory>().unwrap();
    let mut to_delete = vec![];

    let maybe_player = {
        let mut query = world.query::<(&Transform, &BoundingBox, &Player)>();
        let maybe = query.iter().next().map(|(e, (t, bb, _p))| (e, *t, *bb));
        maybe
    };
    if let Some((player_entity, pos, bounding_box)) = maybe_player {
        let mut picked_up = vec![];
        for (e, (pickup, t, bb)) in world.query::<(&Pickup, &Transform, &BoundingBox)>().iter() {
            if input.is_just_pressed(Action::Pickup)
                && aabb_intersection(&pos, &bounding_box, t, bb)
            {
                if let Ok(()) = inventory.remove_scratch(50) {
                    //
                    channel.single_write(GameEvent::PlaySound("sounds/powerUp2.mp3".to_string()));
                    to_delete.push(GameEvent::Delete(e));
                    to_delete.push(GameEvent::InfoText(pickup.item.info_text()));
                    picked_up.push(pickup.item);
                }
            }
        }

        for item in picked_up {
            item.process(world, player_entity);
        }
    }

    channel.drain_vec_write(&mut to_delete);
}

pub fn aabb_intersection2(
    transform1: &Transform,
    bb1: &BoundingBox,
    transform2: &Transform,
    bb2: &BoundingBox,
) -> bool {
    dbg!(transform1.translation - bb1.half_extend);
    dbg!(transform1.translation + bb1.half_extend);
    dbg!(transform2.translation - bb2.half_extend);
    dbg!(transform2.translation + bb2.half_extend);
    transform1.translation.x() - bb1.half_extend.x()
        < transform2.translation.x() + bb2.half_extend.x()
        && transform1.translation.x() + bb1.half_extend.x()
            > transform2.translation.x() - bb2.half_extend.x()
        && transform1.translation.y() - bb1.half_extend.y()
            < transform2.translation.y() + bb2.half_extend.y()
        && transform1.translation.y() + bb1.half_extend.y()
            > transform2.translation.y() - bb2.half_extend.y()
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub enum Items {
    /// Speed +50%
    SpeedBonus,
    /// Crit chance +10%
    CritChance,
    /// Crit dmg +0.10 (10%)
    CritDmg,
    /// Base Damage +10%
    BaseDmg,
    /// Shield +1
    ShieldUp,
    /// Health +1
    HealthUp,
    /// Shoot random Missiles when firing
    Missile,
}

impl Items {
    pub fn process(&self, world: &mut hecs::World, player: hecs::Entity) {
        match self {
            Items::SpeedBonus => {
                let mut dynamic_body = world.get_mut::<DynamicBody>(player).unwrap();
                dynamic_body.velocity *= 1.5;
            }
            Items::CritChance => {
                let mut player = world.get_mut::<Player>(player).unwrap();
                player.stats.crit_percent += 10;
            }
            Items::CritDmg => {
                let mut player = world.get_mut::<Player>(player).unwrap();
                player.stats.crit_multiplier += 0.10;
            }

            Items::BaseDmg => {
                let mut player = world.get_mut::<Player>(player).unwrap();
                player.stats.dmg *= 1.1;
            }
            Items::HealthUp => {
                let mut should_add_health = false;
                if let Ok(mut health) = world.get_mut::<Health>(player) {
                    health.max += 5.0;
                    health.current = health.max;
                } else {
                    should_add_health = true;
                }

                if should_add_health {
                    world
                        .insert_one(player, Health::new(1.0, Timer::of_seconds(1.0)))
                        .expect("Cannot add health component");
                }
            }
            Items::ShieldUp => {
                let mut should_add_shield = false;
                if let Ok(mut shield) = world.get_mut::<Shield>(player) {
                    shield.max += 2.0;
                    shield.current = shield.max;
                } else {
                    should_add_shield = true;
                }

                if should_add_shield {
                    world
                        .insert_one(player, Shield::new(1.0, 5.0, 0.15))
                        .expect("Cannot add shield component");
                }
            }
            Items::Missile => {
                let mut player = world.get_mut::<Player>(player).unwrap();
                player.stats.missile_percent += 10;
            }
        }
    }

    pub fn info_text(&self) -> String {
        match self {
            Items::BaseDmg => "Base Damage Up",
            Items::SpeedBonus => "Speed +30%",
            Items::CritChance => "Crit Chance +5%",
            Items::CritDmg => "Crit Damage +5%",
            Items::ShieldUp => "Shield Up",
            Items::HealthUp => "Health Up",
            Items::Missile => "Missile Spray",
        }
        .to_string()
    }
}

impl Distribution<Items> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Items {
        match rng.gen_range(0, 7) {
            0 => Items::SpeedBonus,
            1 => Items::CritChance,
            2 => Items::CritDmg,
            3 => Items::Missile,
            4 => Items::ShieldUp,
            5 => Items::BaseDmg,
            _ => Items::HealthUp,
        }
    }
}
