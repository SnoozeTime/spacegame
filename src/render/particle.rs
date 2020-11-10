use crate::assets::sprite::SpriteAsset;
use crate::assets::{AssetManager, Handle};
use crate::core::colors::{interpolate_between_three, RgbColor, RgbaColor};
use crate::event::GameEvent;
use crate::gameplay::trail::Trail;
use crate::resources::Resources;
use bitflags::_core::iter::Enumerate;
use bitflags::_core::slice::IterMut;
use downcast_rs::__std::collections::HashSet;
use hecs::World;
use luminance::blending::{Blending, Equation, Factor};
use luminance::context::GraphicsContext;
use luminance::pipeline::{Pipeline, PipelineError, TextureBinding};
use luminance::pixel::NormUnsigned;
use luminance::render_state::RenderState;
use luminance::shader::{Program, Uniform};
use luminance::shading_gate::ShadingGate;
use luminance::tess::{Mode, Tess};
use luminance::texture::Dim2;
use luminance_derive::UniformInterface;
use luminance_gl::GL33;
use rand::{random, Rng};
use serde_derive::{Deserialize, Serialize};
use shrev::EventChannel;
use std::time::Duration;

#[derive(Debug, Clone, Copy)]
struct Particle {
    life: u32,
    initial_life: u32,
    position: glam::Vec2,
    velocity: glam::Vec2,
    start_color: RgbaColor,
    mid_color: RgbaColor,
    end_color: RgbaColor,
    scale: f32,
}

impl Particle {
    /// Create a new particle at the given position with the given velocity.
    fn new(
        life: u32,
        origin: glam::Vec2,
        velocity: glam::Vec2,
        start_color: RgbaColor,
        mid_color: RgbaColor,
        end_color: RgbaColor,
    ) -> Self {
        let mut particle = Particle {
            life,
            initial_life: life,
            position: glam::Vec2::zero(),
            velocity: glam::Vec2::zero(),
            start_color,
            mid_color,
            end_color,
            scale: 1.0,
        };
        particle.respawn(life, origin, velocity, 1.0);
        particle
    }

    fn respawn(&mut self, life: u32, origin: glam::Vec2, velocity: glam::Vec2, scale: f32) {
        self.life = life;
        self.position = origin;
        self.velocity = velocity;
        self.scale = scale;
        self.initial_life = life;
    }

    /// return true if the particle is still alive
    fn alive(&self) -> bool {
        self.life > 0
    }

    fn update(&mut self, gravity: f32, dt: f32) {
        //self.velocity -= gravity * glam::Vec2::unit_y() * dt;
        self.position += self.velocity * dt;
        self.life -= 1; // one frame.
    }

    fn color(&self) -> RgbaColor {
        let t = 1.0 - self.life as f32 / self.initial_life as f32;
        interpolate_between_three(self.start_color, self.mid_color, self.end_color, t)
    }
}

#[derive(Debug, Clone)]
struct ParticlePool {
    particles: Vec<Particle>,
    free: Vec<usize>,
}

impl Default for ParticlePool {
    fn default() -> Self {
        Self {
            particles: vec![],
            free: vec![],
        }
    }
}

impl ParticlePool {
    /// Initiate a bunch of dead particles.
    fn of_size(nb: usize) -> Self {
        Self {
            particles: (0..nb)
                .map(|_| {
                    Particle::new(
                        0,
                        glam::Vec2::zero(),
                        glam::Vec2::zero(),
                        RgbaColor::new(255, 0, 0, 255),
                        RgbaColor::new(255, 0, 0, 255),
                        RgbaColor::new(255, 0, 0, 0),
                    )
                })
                .collect(),
            free: (0..nb).collect(),
        }
    }

    /// Return the first available particle.
    fn get_available(&mut self) -> Option<&mut Particle> {
        let mut particles = &mut self.particles;
        self.free
            .pop()
            .map(move |idx| unsafe { particles.get_unchecked_mut(idx) })
    }

    fn particles_mut(&mut self) -> impl Iterator<Item = (usize, &mut Particle)> {
        self.particles.iter_mut().enumerate()
    }

    fn len(&self) -> usize {
        self.particles.len()
    }

    fn remove(&mut self, index: usize) {
        self.particles.get_mut(index).unwrap().life = 0;

        if !self.free.contains(&index) {
            self.free.push(index);
        } else {
            error!("Tried to free the same particle twice");
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmitterSource {
    /// Spawn particle from this point
    Point(glam::Vec2),

    /// Spawn particle randomly on this line
    Line(glam::Vec2, glam::Vec2),
}

impl EmitterSource {
    fn spawn_position<R: Rng>(&self, rand: &mut R) -> glam::Vec2 {
        match self {
            Self::Point(p) => *p,
            Self::Line(p1, p2) => p1.lerp(*p2, rand.gen_range(0.0, 1.0f32)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParticleShape {
    Quad,
    Texture(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleEmitter {
    #[serde(skip)]
    particles: ParticlePool,
    pub source: EmitterSource,
    pub shape: ParticleShape,

    pub velocity_range: (f32, f32),
    pub angle_range: (f32, f32),
    pub scale_range: (f32, f32),

    /// Particle per frame to emit.
    particle_number: f32,

    /// when particle_number < 1, we need to know when we should spawn a particle.
    #[serde(skip)]
    nb_accumulator: f32,

    /// Color of the particle
    pub colors: (RgbaColor, RgbaColor, RgbaColor),

    /// How long does the particle (in frames)
    #[serde(default)]
    particle_life: u32,
}

impl ParticleEmitter {
    pub fn new(
        source: EmitterSource,
        shape: ParticleShape,
        velocity_range: (f32, f32),
        angle_range: (f32, f32),
        scale_range: (f32, f32),
        particle_number: f32,
        colors: (RgbaColor, RgbaColor, RgbaColor),
        life: u32,
    ) -> Self {
        Self {
            particles: ParticlePool::of_size(particle_number.ceil() as usize * (life as usize + 1)),
            source,
            shape,
            velocity_range,
            angle_range,
            scale_range,
            particle_number,
            nb_accumulator: 0.0,
            colors,
            particle_life: life,
        }
    }

    /// Necessary when getting the emitter from a file.
    pub fn init_pool(&mut self) {
        self.particles = ParticlePool::of_size(
            self.particle_number.ceil() as usize * (self.particle_life as usize + 1),
        );
    }

    /// Update the position and velocity of all particles. If a particle is dead, respawn it :)
    /// Return true if should despawn the particle emitter.
    fn update(&mut self, dt: f32) -> bool {
        let mut rng = rand::thread_rng();

        // emit particles.
        trace!(
            "Will emit {} particles (Acc = {}",
            self.particle_number,
            self.nb_accumulator
        );
        self.nb_accumulator += self.particle_number;

        let entire_nb = self.nb_accumulator.floor() as u32;
        if entire_nb > 0 {
            for _ in 0..entire_nb {
                if let Some(particle) = self.particles.get_available() {
                    trace!("Emit particle");

                    let rotation = glam::Mat2::from_angle(
                        rng.gen_range(self.angle_range.0, self.angle_range.1),
                    );
                    let speed = rng.gen_range(self.velocity_range.0, self.velocity_range.1);
                    let scale = rng.gen_range(self.scale_range.0, self.scale_range.1);
                    particle.respawn(
                        self.particle_life,
                        self.source.spawn_position(&mut rng),
                        rotation * (speed * glam::Vec2::unit_x()),
                        scale,
                    );
                    particle.start_color = self.colors.0;
                    particle.mid_color = self.colors.1;
                    particle.end_color = self.colors.2;
                    trace!("{:?}", particle);
                }
            }
            self.nb_accumulator -= self.nb_accumulator.floor();
        }

        // update existing particles.
        for (idx, p) in self.particles.particles.iter_mut().enumerate() {
            if p.alive() {
                p.update(9.8, dt);
            } else {
                if !self.particles.free.contains(&idx) {
                    self.particles.free.push(idx);
                }
            }
        }

        true
    }
}

const VS: &'static str = include_str!("particle-vs.glsl");
const FS: &'static str = include_str!("particle-fs.glsl");
const FS_TEXTURE: &'static str = include_str!("particle-texture-fs.glsl");

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

pub fn new_texture_shader<B>(
    surface: &mut B,
) -> Program<GL33, (), (), TextureParticleShaderInterface>
where
    B: GraphicsContext<Backend = GL33>,
{
    surface
        .new_shader_program::<(), (), TextureParticleShaderInterface>()
        .from_strings(VS, None, None, FS_TEXTURE)
        .expect("Program creation")
        .ignore_warnings()
}

#[derive(UniformInterface)]
pub struct ParticleShaderInterface {
    pub projection: Uniform<[[f32; 4]; 4]>,
    #[uniform(unbound)]
    pub view: Uniform<[[f32; 4]; 4]>,
    pub model: Uniform<[[f32; 4]; 4]>,
    pub color: Uniform<[f32; 4]>,
}

#[derive(UniformInterface)]
pub struct TextureParticleShaderInterface {
    pub projection: Uniform<[[f32; 4]; 4]>,
    #[uniform(unbound)]
    pub view: Uniform<[[f32; 4]; 4]>,
    pub model: Uniform<[[f32; 4]; 4]>,
    pub color: Uniform<[f32; 4]>,

    /// Texture for the sprite.
    tex: Uniform<TextureBinding<Dim2, NormUnsigned>>,
}

pub struct ParticleSystem<S>
where
    S: GraphicsContext<Backend = GL33>,
{
    tess: Tess<S::Backend, ()>,
    shader: Program<S::Backend, (), (), ParticleShaderInterface>,
    texture_shader: Program<S::Backend, (), (), TextureParticleShaderInterface>,
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
            texture_shader: new_texture_shader(surface),
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
        pipeline: &Pipeline<S::Backend>,
        shd_gate: &mut ShadingGate<S::Backend>,
        projection: &glam::Mat4,
        view: &glam::Mat4,
        world: &World,

        textures: &mut AssetManager<S, SpriteAsset<S>>,
    ) -> Result<(), PipelineError> {
        let tess = &self.tess;
        let render_st = RenderState::default()
            .set_depth_test(None)
            .set_blending_separate(
                Blending {
                    equation: Equation::Additive,
                    src: Factor::SrcAlpha,
                    dst: Factor::SrcAlphaComplement,
                },
                Blending {
                    equation: Equation::Additive,
                    src: Factor::One,
                    dst: Factor::Zero,
                },
            );
        for (_, emitter) in world.query::<&mut ParticleEmitter>().iter() {
            match &emitter.shape {
                ParticleShape::Quad => {
                    shd_gate.shade(&mut self.shader, |mut iface, uni, mut rdr_gate| {
                        iface.set(&uni.projection, projection.to_cols_array_2d());
                        iface.set(&uni.view, view.to_cols_array_2d());

                        for p in &emitter.particles.particles {
                            if !p.alive() {
                                continue;
                            }

                            iface.set(&uni.color, p.color().to_normalized());
                            iface.set(
                                &uni.model,
                                glam::Mat4::from_scale_rotation_translation(
                                    glam::vec3(p.scale, p.scale, 1.0),
                                    glam::Quat::identity(),
                                    p.position.extend(0.0),
                                )
                                .to_cols_array_2d(),
                            );

                            rdr_gate.render(&render_st, |mut tess_gate| tess_gate.render(tess))?;
                        }

                        Ok(())
                    })?;
                }
                ParticleShape::Texture(id) => {
                    if let Some(tex) = textures.get_mut(&Handle(id.clone())) {
                        let mut res = Ok(());
                        let shader = &mut self.texture_shader;
                        tex.execute_mut(|asset| {
                            if let Some(tex) = asset.texture() {
                                let bound_tex = pipeline.bind_texture(tex).unwrap();
                                res = shd_gate.shade(shader, |mut iface, uni, mut rdr_gate| {
                                    iface.set(&uni.projection, projection.to_cols_array_2d());
                                    iface.set(&uni.view, view.to_cols_array_2d());
                                    iface.set(&uni.tex, bound_tex.binding());
                                    for p in &emitter.particles.particles {
                                        if !p.alive() {
                                            continue;
                                        }

                                        iface.set(&uni.color, p.color().to_normalized());
                                        iface.set(
                                            &uni.model,
                                            glam::Mat4::from_scale_rotation_translation(
                                                glam::vec3(p.scale, p.scale, 1.0),
                                                glam::Quat::identity(),
                                                p.position.extend(0.0),
                                            )
                                            .to_cols_array_2d(),
                                        );

                                        rdr_gate.render(&render_st, |mut tess_gate| {
                                            tess_gate.render(tess)
                                        })?;
                                    }

                                    Ok(())
                                });
                            }
                        });

                        res?;
                    };
                }
            }
        }

        Ok(())
    }
}
