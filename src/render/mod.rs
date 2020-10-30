use crate::assets::SpriteCache;
use crate::event::GameEvent;
use crate::render::background::{BackgroundRenderer, BackgroundUniform};
use crate::render::sprite::{ShaderUniform, SpriteRenderer};
use crate::render::text::TextRenderer;
use crate::resources::Resources;
use glyph_brush::{GlyphBrush, GlyphBrushBuilder};
use luminance::context::GraphicsContext;
use luminance::framebuffer::Framebuffer;
use luminance::pipeline::{PipelineError, PipelineState, Render};
use luminance::shader::Program;
use luminance::texture::Dim2;
use luminance_gl::GL33;
use shrev::{EventChannel, ReaderId};

pub mod background;
pub mod particle;
pub mod sprite;
pub mod text;

const FONT_DATA: &'static [u8] = include_bytes!("../../assets/fonts/FFFFORWA.TTF");

pub struct Renderer<S>
where
    S: GraphicsContext<Backend = GL33>,
{
    /// listen for text updates
    rdr_id: ReaderId<GameEvent>,
    /// Render sprites on screen.
    sprite_renderer: SpriteRenderer<S>,
    /// Render the moving space background.
    background_renderer: BackgroundRenderer<S>,

    fonts: GlyphBrush<'static, text::Instance>,
    /// render text on the screen.
    text_renderer: TextRenderer<S>,

    background_shader: Program<S::Backend, (), (), BackgroundUniform>,
    text_shader: Program<S::Backend, text::VertexSemantics, (), text::ShaderInterface>,
}

impl<S> Renderer<S>
where
    S: GraphicsContext<Backend = GL33> + 'static,
{
    pub fn new(surface: &mut S, resources: &mut Resources) -> Renderer<S> {
        let sprite_renderer = sprite::SpriteRenderer::new(surface);
        let background_renderer = background::BackgroundRenderer::new(surface);
        let background_shader = background::new_shader(surface);
        let mut fonts = GlyphBrushBuilder::using_font_bytes(FONT_DATA).build();
        let text_renderer = TextRenderer::new(surface, &mut fonts);

        let mut chan = resources.fetch_mut::<EventChannel<GameEvent>>().unwrap();
        let rdr_id = chan.register_reader();
        Self {
            rdr_id,
            fonts,
            sprite_renderer,
            background_renderer,
            background_shader,
            text_renderer,
            text_shader: text::new_shader(surface),
        }
    }

    pub fn render(
        &mut self,
        surface: &mut S,
        back_buffer: &mut Framebuffer<S::Backend, Dim2, (), ()>,
        world: &hecs::World,
        resources: &Resources,
    ) -> Render<PipelineError> {
        let mut textures = resources.fetch_mut::<SpriteCache<S>>().unwrap();
        surface
            .new_pipeline_gate()
            .pipeline(
                back_buffer,
                &PipelineState::default().set_clear_color([0.0, 0.0, 0.0, 1.0]),
                |pipeline, mut shd_gate| {
                    self.background_renderer.render(
                        &pipeline,
                        &mut shd_gate,
                        &mut self.background_shader,
                    )?;

                    self.sprite_renderer.render(
                        &pipeline,
                        &mut shd_gate,
                        &world,
                        &mut *textures,
                    )?;

                    self.text_renderer
                        .render(&pipeline, &mut shd_gate, &mut self.text_shader)
                },
            )
            .assume()
    }

    pub fn update(&mut self, surface: &mut S, world: &hecs::World, resources: &Resources) {
        let mut should_update_text = false;
        let chan = resources.fetch_mut::<EventChannel<GameEvent>>().unwrap();
        for ev in chan.read(&mut self.rdr_id) {
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
