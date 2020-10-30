use crate::core::camera::Camera;
use crate::core::colors::RgbColor;
use crate::core::timer::Timer;
use crate::core::transform::Transform;
use crate::event::GameEvent;
use crate::gameplay::collision::{BoundingBox, CollisionLayer};
use crate::gameplay::enemy::{EnemyType, ProtoShip};
use crate::gameplay::health::{Health, HealthSystem};
use crate::gameplay::level::{LevelInstruction, LevelSystem};
use crate::gameplay::player::{Player, Weapon};
use crate::gameplay::{bullet, collision, enemy, player};
use crate::render::sprite::Sprite;
use crate::render::text::Text;
use crate::resources::Resources;
use hecs::World;
use log::info;
use std::time::Duration;

/// The stack will keep track of the states in the game.
/// The top of the stack will be used for the update loop. The states below
/// are still kept in memory so to go back to a previous state, you just have
/// to pop the stack.
#[derive(Default)]
pub struct SceneStack {
    states: Vec<Box<dyn Scene>>,
}

pub enum SceneResult {
    ReplaceScene(Box<dyn Scene>),
    Push(Box<dyn Scene>),
    Pop,
    Noop,
}

impl SceneStack {
    pub fn apply_result(
        &mut self,
        res: SceneResult,
        world: &mut hecs::World,
        resources: &mut Resources,
    ) {
        match res {
            SceneResult::ReplaceScene(state) => self.replace(state, world, resources),
            SceneResult::Push(state) => self.push(state, world, resources),
            SceneResult::Pop => {
                self.pop(world);
            }
            SceneResult::Noop => (),
        }
    }

    /// Add a state to the game. Will be used for updating.
    ///
    /// The callback on_enter will be executed for the new state.
    pub fn push(
        &mut self,
        state: Box<dyn Scene>,
        world: &mut hecs::World,
        resources: &mut Resources,
    ) {
        if let Some(current) = self.states.last_mut() {
            current.on_exit();
        }

        self.states.push(state);
        if let Some(current) = self.states.last_mut() {
            current.on_create(world, resources);
        }
    }

    /// Remove the current state and execute its exit callback.
    pub fn pop(&mut self, world: &mut hecs::World) -> Option<Box<dyn Scene>> {
        if let Some(mut s) = self.states.pop() {
            s.on_destroy(world);
            if let Some(current) = self.states.last() {
                current.on_enter();
            }
            Some(s)
        } else {
            None
        }
    }

    /// Replace the current state.
    pub fn replace(
        &mut self,
        state: Box<dyn Scene>,
        world: &mut hecs::World,
        resources: &mut Resources,
    ) {
        if let Some(mut s) = self.states.pop() {
            s.on_destroy(world);
        }
        self.states.push(state);
        if let Some(current) = self.states.last_mut() {
            current.on_create(world, resources);
        }
    }

    /// Get the current state as a mut reference.
    #[allow(clippy::borrowed_box)]
    pub fn current_mut(&mut self) -> Option<&mut Box<dyn Scene>> {
        self.states.last_mut()
    }
}

pub trait Scene {
    /// WIll be called when the state is added to the state stack.
    fn on_create(&mut self, _world: &mut hecs::World, _resources: &mut Resources) {
        info!("Create state");
    }

    /// Will be called when the state is removed from the state stack.
    fn on_destroy(&mut self, _world: &mut hecs::World) {
        info!("Destroy state");
    }

    /// Will be called when the state becomes active. This is called
    /// on stack.pop
    ///
    /// Careful, this is not call on stack.push. Use the on_create callback instead.
    fn on_enter(&self) {
        info!("Enter state");
    }

    /// Will be called when the state becomes inactive. This is called on
    /// stack.push
    fn on_exit(&self) {
        info!("Exit state");
    }

    //fn on_new_world(&mut self);

    /// Update gameplay systems.
    fn update(&mut self, dt: Duration, world: &mut World, resources: &Resources) -> SceneResult;

    /// React to game events.
    fn process_event(&mut self, ev: GameEvent);
}

pub struct MainScene {
    level_system: LevelSystem,
    health_system: Option<HealthSystem>,
}

fn load_level() -> LevelSystem {
    let instructions = vec![
        LevelInstruction::SpawnEnemy {
            health: 2,
            pos: glam::Vec2::new(200.0, 400.0),
            enemy_type: EnemyType::ProtoShip(ProtoShip {
                current_target: glam::Vec2::new(400.0, 400.0),
                speed: 2.0,
                wait_time: 2.0,
                current_timed_waited: 0.0,
                elapsed_from_beginning: 0.0,
            }),
        },
        LevelInstruction::Wait {
            elapsed: 0.0,
            to_wait: 5.0,
        },
        LevelInstruction::SpawnEnemy {
            health: 15,
            pos: glam::Vec2::new(200.0, 800.0),
            enemy_type: EnemyType::Spammer {
                path: enemy::Path {
                    current: 0,
                    path: vec![
                        glam::Vec2::new(200.0, 600.0),
                        glam::Vec2::new(600.0, 600.0),
                        glam::Vec2::new(400.0, 600.0),
                    ],
                },
                shoot_timer: Timer::of_seconds(2.0),
                bullet_timeout: Timer::of_seconds(0.5),
                shooting: false,
            },
        },
        LevelInstruction::SpawnMultiple {
            health: 1,
            time_between: 0.5,
            elapsed: 0.0,
            nb: 5,
            spawned: 0,
            pos: glam::Vec2::new(400.0, 800.0),
            enemy_type: EnemyType::KamikazeRandom(enemy::Path {
                current: 0,
                path: vec![
                    glam::Vec2::new(400.0, 600.0),
                    glam::Vec2::new(300.0, 400.0),
                    glam::Vec2::new(400.0, 200.0),
                ],
            }),
        },
    ];
    LevelSystem::new(instructions)
}

impl MainScene {
    pub fn new() -> Self {
        Self {
            level_system: load_level(),
            health_system: None, //Option<HealthSystem::new(resources)>,
        }
    }
}

impl Scene for MainScene {
    fn on_create(&mut self, world: &mut hecs::World, resources: &mut Resources) {
        info!("Create MainScene");
        self.health_system = Some(HealthSystem::new(resources));

        world.spawn((Camera::new(),));
        world.spawn((
            Transform {
                translation: glam::Vec2::new(100.0, 100.0),
                scale: glam::Vec2::new(50.0, 50.0),
                rotation: 0.0,
            },
            Sprite {
                id: "p-blue-a".to_string(),
            },
            Player {
                weapon: Weapon::Simple,
            },
            BoundingBox {
                collision_layer: CollisionLayer::PLAYER,
                collision_mask: CollisionLayer::ENEMY_BULLET | CollisionLayer::ENEMY,
                half_extend: glam::vec2(20.0, 20.0),
            },
            Health::new(5, Timer::of_seconds(1.0)),
        ));

        world.spawn((
            Transform {
                translation: glam::Vec2::new(100.0, 100.0),
                scale: glam::Vec2::new(50.0, 50.0),
                rotation: 0.0,
            },
            Text {
                content: "BENOIT".to_lowercase(),
                font_size: 16.0,
            },
            RgbColor { r: 255, g: 0, b: 0 },
        ));
    }

    fn update(&mut self, dt: Duration, world: &mut World, resources: &Resources) -> SceneResult {
        log::debug!("UPDATE SYSTEMS");
        self.level_system.update(world, dt);
        player::update_player(world, &resources);
        enemy::update_enemies(world, &resources, dt);
        bullet::process_bullets(world);
        let collisions = collision::find_collisions(world);
        collision::process_collisions(world, collisions, &resources);
        if let Some(hs) = self.health_system.as_mut() {
            hs.update(world, &resources, dt);
        }

        SceneResult::Noop
    }

    fn process_event(&mut self, ev: GameEvent) {
        if let GameEvent::GameOver = ev {
            std::process::exit(0); // TODO Replace that.
        }
    }
}
