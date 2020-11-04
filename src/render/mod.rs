use crate::assets::sprite::SpriteAsset;
use crate::assets::AssetManager;
use crate::event::GameEvent;
use crate::render::background::BackgroundRenderer;
use crate::render::particle::ParticleSystem;
use crate::render::sprite::SpriteRenderer;
use crate::render::text::TextRenderer;
use crate::resources::Resources;
use crate::{HEIGHT, WIDTH};
use glyph_brush::{GlyphBrush, GlyphBrushBuilder};
use luminance::context::GraphicsContext;
use luminance::framebuffer::Framebuffer;
use luminance::pipeline::{PipelineError, PipelineState, Render};
use luminance::texture::Dim2;
use luminance_gl::GL33;
use shrev::{EventChannel, ReaderId};
use std::time::Duration;

pub mod background;
pub mod particle;
pub mod sprite;
pub mod text;

const FONT_DATA: &'static [u8] = include_bytes!("../../assets/fonts/FFFFORWA.TTF");

pub struct Renderer<S>
where
    S: GraphicsContext<Backend = GL33>,
{
    projection: glam::Mat4,

    /// listen for text updates
    rdr_id: ReaderId<GameEvent>,
    /// Render sprites on screen.
    sprite_renderer: SpriteRenderer<S>,
    /// Render the moving space background.
    background_renderer: BackgroundRenderer<S>,

    fonts: GlyphBrush<'static, text::Instance>,
    /// render text on the screen.
    text_renderer: TextRenderer<S>,
    /// particles :)
    particle_renderer: ParticleSystem<S>,
}

impl<S> Renderer<S>
where
    S: GraphicsContext<Backend = GL33> + 'static,
{
    pub fn new(surface: &mut S, resources: &mut Resources) -> Renderer<S> {
        let sprite_renderer = sprite::SpriteRenderer::new(surface);
        let background_renderer = background::BackgroundRenderer::new(surface);
        let mut fonts = GlyphBrushBuilder::using_font_bytes(FONT_DATA).build();
        let text_renderer = TextRenderer::new(surface, &mut fonts);

        let mut channel = resources.fetch_mut::<EventChannel<GameEvent>>().unwrap();
        let rdr_id = channel.register_reader();

        let particle_renderer = ParticleSystem::new(surface);

        let projection_matrix =
            glam::Mat4::orthographic_rh_gl(0.0, WIDTH as f32, 0.0, HEIGHT as f32, -1.0, 10.0);
        Self {
            projection: projection_matrix,
            rdr_id,
            fonts,
            sprite_renderer,
            background_renderer,
            text_renderer,
            particle_renderer,
        }
    }

    pub fn render(
        &mut self,
        surface: &mut S,
        back_buffer: &mut Framebuffer<S::Backend, Dim2, (), ()>,
        world: &hecs::World,
        resources: &Resources,
    ) -> Render<PipelineError> {
        let view = crate::core::camera::get_view_matrix(world).unwrap();

        let mut textures = resources
            .fetch_mut::<AssetManager<S, SpriteAsset<S>>>()
            .unwrap();
        surface
            .new_pipeline_gate()
            .pipeline(
                back_buffer,
                &PipelineState::default().set_clear_color([0.0, 0.0, 0.0, 1.0]),
                |pipeline, mut shd_gate| {
                    self.background_renderer
                        .render(&pipeline, &mut shd_gate, &mut *textures)?;

                    self.sprite_renderer.render(
                        &pipeline,
                        &mut shd_gate,
                        &self.projection,
                        &view,
                        &world,
                        &mut *textures,
                    )?;

                    self.particle_renderer
                        .render(&mut shd_gate, &self.projection, &view, world)?;

                    self.text_renderer.render(&pipeline, &mut shd_gate)
                },
            )
            .assume()
    }

    pub fn update(
        &mut self,
        surface: &mut S,
        world: &hecs::World,
        dt: Duration,
        resources: &Resources,
    ) {
        self.background_renderer.update(world);
        // update particle systems.
        self.particle_renderer.update(world, dt, resources);

        // Update text if needed.
        let mut should_update_text = false;
        let channel = resources.fetch_mut::<EventChannel<GameEvent>>().unwrap();
        for ev in channel.read(&mut self.rdr_id) {
            if let GameEvent::TextUpdated = ev {
                should_update_text = true;
            }
        }

        if should_update_text {
            self.update_text(surface, world);
        }
    }

    /// Recreate the texture
    pub fn update_text(&mut self, surface: &mut S, world: &hecs::World) {
        self.text_renderer
            .update_text(surface, world, &mut self.fonts);
    }
}
