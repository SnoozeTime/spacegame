use crate::assets::sprite::SpriteAsset;
use crate::assets::{AssetManager, Handle};
use crate::core::colors::RgbaColor;
use crate::core::curve::Curve;
use crate::core::transform::Transform;
use crate::event::GameEvent;
use crate::resources::Resources;
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
use rand::Rng;
use serde_derive::{Deserialize, Serialize};
use shrev::EventChannel;
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ParticleScale {
    Constant(glam::Vec2),
    Random(glam::Vec2, glam::Vec2),
}

#[derive(Debug, Clone, Default)]
struct Particle {
    life: u32,
    initial_life: u32,
    position: glam::Vec2,
    velocity: glam::Vec2,
    colors: Curve<RgbaColor>,
    scale: glam::Vec2,
    scale_over_lifetime: Option<Curve<f32>>,

    rotation: f32,
}

impl Particle {
    fn respawn(
        &mut self,
        life: u32,
        origin: glam::Vec2,
        velocity: glam::Vec2,
        scale: glam::Vec2,
        scale_over_lifetime: Option<Curve<f32>>,
        rotation: f32,
    ) {
        self.life = life;
        self.position = origin;
        self.velocity = velocity;
        self.scale = scale;
        self.scale_over_lifetime = scale_over_lifetime;
        self.initial_life = life;
        self.rotation = rotation;
    }

    /// return true if the particle is still alive
    fn alive(&self) -> bool {
        self.life > 0
    }

    fn update(&mut self, dt: f32) {
        self.position += self.velocity * dt;
        self.life -= 1; // one frame.
    }

    fn t(&self) -> f32 {
        1.0 - self.life as f32 / self.initial_life as f32
    }

    fn color(&self) -> RgbaColor {
        let t = self.t();
        self.colors.y(t)
    }

    fn scale(&self) -> glam::Vec2 {
        if let Some(curve) = &self.scale_over_lifetime {
            self.scale * curve.y(self.t())
        } else {
            self.scale
        }
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
            particles: (0..nb).map(|_| Particle::default()).collect(),
            free: (0..nb).collect(),
        }
    }

    /// Return the first available particle.
    fn get_available(&mut self) -> Option<&mut Particle> {
        let particles = &mut self.particles;
        self.free
            .pop()
            .map(move |idx| unsafe { particles.get_unchecked_mut(idx) })
    }

    fn all_dead(&self) -> bool {
        self.particles.len() == self.free.len()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmitterSource {
    /// Spawn particle from this point
    Point,

    /// Spawn particle randomly on this line
    /// Line relative to emitter's transform, so first point will be transform + v1, next point will be
    /// transform + v2
    Line(glam::Vec2, glam::Vec2),
}

impl EmitterSource {
    fn spawn_position<R: Rng>(&self, emitter_position: glam::Vec2, rand: &mut R) -> glam::Vec2 {
        match self {
            Self::Point => emitter_position,
            Self::Line(p1, p2) => {
                (emitter_position - *p1).lerp(emitter_position + *p2, rand.gen_range(0.0, 1.0f32))
            }
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
    enabled: bool,

    #[serde(skip)]
    particles: ParticlePool,
    pub source: EmitterSource,
    pub shape: ParticleShape,

    pub velocity_range: (f32, f32),
    pub angle_range: (f32, f32),

    pub scale: ParticleScale,
    pub scale_over_lifetime: Option<Curve<f32>>,

    /// Particle per frame to emit.
    particle_number: f32,

    /// when particle_number < 1, we need to know when we should spawn a particle.
    #[serde(skip)]
    nb_accumulator: f32,

    /// Color of the particle
    pub colors: Curve<RgbaColor>,

    /// How long does the particle (in frames)
    #[serde(default)]
    particle_life: u32,

    /// Offset applied to a particle position on spawn.
    #[serde(default)]
    pub position_offset: glam::Vec2,

    /// If true, only spawn stuff once
    #[serde(default)]
    pub burst: bool,
}

impl Default for ParticleEmitter {
    fn default() -> Self {
        Self {
            enabled: true,
            particles: Default::default(),
            source: EmitterSource::Point,
            shape: ParticleShape::Quad,
            velocity_range: (0.0, 10.0),
            angle_range: (0.0, 2.0 * std::f32::consts::PI),
            scale: ParticleScale::Constant(glam::vec2(5.0, 5.0)),
            scale_over_lifetime: None,
            particle_number: 1.0,
            nb_accumulator: 0.0,
            colors: Default::default(),
            particle_life: 10,
            position_offset: Default::default(),
            burst: false,
        }
    }
}

impl ParticleEmitter {
    pub fn load_from_path<P: AsRef<Path>>(p: P) -> Result<Self, anyhow::Error> {
        let content = std::fs::read_to_string(p)?;
        let mut emitter: Self = serde_json::from_str(&content)?;
        emitter.init_pool();
        Ok(emitter)
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Necessary when getting the emitter from a file.
    pub fn init_pool(&mut self) {
        let frame_needed = if self.burst {
            1
        } else {
            self.particle_life as usize + 1
        };
        self.particles = ParticlePool::of_size(self.particle_number.ceil() as usize * frame_needed);
    }

    /// Update the position and velocity of all particles. If a particle is dead, respawn it :)
    /// Return true if should despawn the particle emitter.
    fn update(&mut self, position: glam::Vec2, dt: f32) -> bool {
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
            if self.enabled {
                for _ in 0..entire_nb {
                    if let Some(particle) = self.particles.get_available() {
                        trace!("Emit particle");

                        let angle = rng.gen_range(self.angle_range.0, self.angle_range.1);
                        let rotation = glam::Mat2::from_angle(angle);
                        let speed = rng.gen_range(self.velocity_range.0, self.velocity_range.1);

                        // PARTICLE SCALE. -> initial scale.
                        let scale = match self.scale {
                            ParticleScale::Constant(s) => s,
                            ParticleScale::Random(low, high) => {
                                let x = rng.gen_range(low.x, high.x);
                                let y = rng.gen_range(low.y, high.y);
                                glam::vec2(x, y)
                            }
                        };

                        particle.respawn(
                            self.particle_life,
                            self.source.spawn_position(position, &mut rng) + self.position_offset,
                            rotation * (speed * glam::Vec2::unit_x()),
                            scale,
                            self.scale_over_lifetime.clone(),
                            angle,
                        );
                        particle.colors = self.colors.clone();
                        trace!("{:?}", particle);
                    }
                }
            }
            self.nb_accumulator -= self.nb_accumulator.floor();
        }

        // update existing particles.
        for (idx, p) in self.particles.particles.iter_mut().enumerate() {
            if p.alive() {
                p.update(dt);
            } else {
                if !self.particles.free.contains(&idx) {
                    self.particles.free.push(idx);
                }
            }
        }

        if self.burst {
            self.disable();

            if self.particles.all_dead() {
                return false;
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
        let mut remove_events = vec![];
        for (e, (t, emitter)) in world.query::<(&Transform, &mut ParticleEmitter)>().iter() {
            if !emitter.update(t.translation, dt.as_secs_f32()) {
                chan.single_write(GameEvent::Delete(e));
            }
        }
        chan.drain_vec_write(&mut remove_events);
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
            .set_blending(Blending {
                equation: Equation::Additive,
                src: Factor::One,
                dst: Factor::SrcAlphaComplement,
            });
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
                                    p.scale().extend(1.0),
                                    glam::Quat::from_rotation_z(p.rotation),
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
                                                p.scale().extend(1.0),
                                                glam::Quat::from_rotation_z(p.rotation),
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
                    } else {
                        debug!("Texture is not loaded {}", id);
                        textures.load(id.clone());
                    }
                }
            }
        }

        Ok(())
    }
}
