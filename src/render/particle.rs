use crate::core::colors::RgbColor;
use crate::event::GameEvent;
use crate::resources::Resources;
use hecs::World;
use luminance::context::GraphicsContext;
use luminance::pipeline::PipelineError;
use luminance::render_state::RenderState;
use luminance::shader::{Program, Uniform};
use luminance::shading_gate::ShadingGate;
use luminance::tess::{Mode, Tess};
use luminance_derive::UniformInterface;
use luminance_gl::GL33;
use rand::Rng;
use serde_derive::{Deserialize, Serialize};
use shrev::EventChannel;
use std::time::Duration;

#[derive(Debug, Clone, Copy)]
struct Particle {
    life: f32,
    position: glam::Vec2,
    velocity: glam::Vec2,
    color: RgbColor,
}

impl Particle {
    /// Create a new particle at the given position with the given velocity.
    fn new(origin: glam::Vec2, velocity: glam::Vec2, color: RgbColor) -> Self {
        let mut particle = Particle {
            life: 0.0,
            position: glam::Vec2::zero(),
            velocity: glam::Vec2::zero(),
            color,
        };
        particle.respawn(origin, velocity);
        particle
    }

    fn respawn(&mut self, origin: glam::Vec2, velocity: glam::Vec2) {
        self.life = 1.0;
        self.position = origin;
        self.velocity = velocity;
    }

    /// return true if the particle is still alive
    fn alive(&self) -> bool {
        self.life > 0.0
    }

    fn update(&mut self, gravity: f32, dt: f32) {
        self.velocity -= gravity * glam::Vec2::unit_y() * dt;
        self.position += self.velocity * dt;
        self.life -= dt;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleEmitter {
    #[serde(skip)]
    particles: Vec<Particle>,
    pub position: glam::Vec2,
    pub velocity: glam::Vec2,

    /// maximum number of particle to emit.
    particle_number: usize,

    /// Color of the particle
    color: RgbColor,

    /// How long does the emitter live (in seconds)
    #[serde(default)]
    life: Option<f32>,
}

impl ParticleEmitter {
    pub fn new(
        position: glam::Vec2,
        velocity: glam::Vec2,
        particle_number: usize,
        color: RgbColor,
        life: Option<f32>,
    ) -> Self {
        Self {
            particles: vec![],
            position,
            velocity,
            particle_number,
            color,
            life,
        }
    }

    /// Update the position and velocity of all particles. If a particle is dead, respawn it :)
    /// Return true if should despawn the particle emitter.
    fn update(&mut self, dt: f32) -> bool {
        let mut rng = rand::thread_rng();

        for p in &mut self.particles {
            if p.alive() {
                p.update(9.8, dt);
            } else {
                let pos_offset: f32 = rng.gen_range(-1.0, 1.0);

                let vel_offset =
                    glam::Vec2::new(rng.gen_range(-1.0, 1.0), rng.gen_range(-1.0, 1.0));

                p.respawn(
                    self.position + pos_offset * self.velocity.normalize(),
                    self.velocity + vel_offset,
                );
            }
        }

        if self.particles.len() < self.particle_number {
            self.particles
                .reserve(self.particle_number - self.particles.len());
            let pos_offset: f32 = rng.gen_range(-1.0, 1.0);
            let vel_offset = glam::Vec2::new(rng.gen_range(-1.0, 1.0), rng.gen_range(-1.0, 1.0));

            self.particles.push(Particle::new(
                self.position + pos_offset * self.velocity.normalize(),
                self.velocity + vel_offset,
                self.color,
            ));
        }

        // update life of emitter.
        if let Some(life) = self.life.as_mut() {
            *life -= dt;
            *life > 0.0
        } else {
            true
        }
    }
}

const VS: &'static str = include_str!("particle-vs.glsl");
const FS: &'static str = include_str!("particle-fs.glsl");

pub fn new_shader<B>(surface: &mut B) -> Program<GL33, (), (), ParticleShaderInterface>
where
    B: GraphicsContext<Backend = GL33>,
{
    surface
        .new_shader_program::<(), (), ParticleShaderInterface>()
        .from_strings(VS, None, None, FS)
        .expect("Program creation")
        .ignore_warnings()
}

#[derive(UniformInterface)]
pub struct ParticleShaderInterface {
    pub projection: Uniform<[[f32; 4]; 4]>,
    #[uniform(unbound)]
    pub view: Uniform<[[f32; 4]; 4]>,
    pub model: Uniform<[[f32; 4]; 4]>,
    pub color: Uniform<[f32; 3]>,
}

pub struct ParticleSystem<S>
where
    S: GraphicsContext<Backend = GL33>,
{
    tess: Tess<S::Backend, ()>,
    shader: Program<S::Backend, (), (), ParticleShaderInterface>,
}

impl<S> ParticleSystem<S>
where
    S: GraphicsContext<Backend = GL33>,
{
    pub fn new(surface: &mut S) -> Self {
        let tess = surface
            .new_tess()
            .set_vertex_nb(4)
            .set_mode(Mode::TriangleFan)
            .build()
            .expect("Tess creation");
        Self {
            tess,
            shader: new_shader(surface),
        }
    }

    pub fn update(&mut self, world: &World, dt: Duration, resources: &Resources) {
        let mut chan = resources.fetch_mut::<EventChannel<GameEvent>>().unwrap();
        for (e, emitter) in world.query::<&mut ParticleEmitter>().iter() {
            if !emitter.update(dt.as_secs_f32()) {
                chan.single_write(GameEvent::Delete(e));
            }
        }
    }

    pub fn render(
        &mut self,
        shd_gate: &mut ShadingGate<S::Backend>,
        projection: &glam::Mat4,
        view: &glam::Mat4,
        world: &World,
    ) -> Result<(), PipelineError> {
        let tess = &self.tess;
        shd_gate.shade(&mut self.shader, |mut iface, uni, mut rdr_gate| {
            for (_, emitter) in world.query::<&mut ParticleEmitter>().iter() {
                iface.set(&uni.projection, projection.to_cols_array_2d());
                iface.set(&uni.view, view.to_cols_array_2d());

                for p in &emitter.particles {
                    iface.set(&uni.color, p.color.to_normalized());
                    iface.set(
                        &uni.model,
                        glam::Mat4::from_scale_rotation_translation(
                            glam::vec3(1.0, 1.0, 1.0),
                            glam::Quat::identity(),
                            p.position.extend(0.0),
                        )
                        .to_cols_array_2d(),
                    );

                    rdr_gate.render(&RenderState::default(), |mut tess_gate| {
                        tess_gate.render(tess)
                    })?;
                }
            }
            Ok(())
        })
    }
}
