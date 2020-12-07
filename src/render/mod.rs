use crate::assets::shader::ShaderManager;
use crate::assets::sprite::SpriteAsset;
use crate::assets::AssetManager;
use crate::core::camera::ProjectionMatrix;
use crate::render::mesh::MeshRenderer;
use crate::render::particle::ParticleSystem;
use crate::render::path::PathRenderer;
use crate::render::sprite::SpriteRenderer;
use crate::render::ui::{text, Gui, GuiContext, UiRenderer};
use crate::resources::Resources;
use glyph_brush::GlyphBrush;
use luminance::context::GraphicsContext;
use luminance::pipeline::{PipelineError, PipelineState, Render};
use luminance::texture::Dim2;
use luminance_front::framebuffer::Framebuffer;
use std::time::Duration;

pub mod mesh;
pub mod particle;
pub mod path;
pub mod sprite;
pub mod ui;

/// Build for desktop will use opengl
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub type Backend = luminance_gl::GL33;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub type Context = luminance_glfw::GlfwSurface;

/// Build for web (wasm) will use webgl
#[cfg(target_arch = "wasm32")]
pub type Backend = luminance_webgl::webgl2::WebGL2;
#[cfg(target_arch = "wasm32")]
pub type Context = luminance_web_sys::WebSysWebGL2Surface;

pub struct Renderer {
    /// Render sprites on screen.
    sprite_renderer: SpriteRenderer,
    mesh_renderer: MeshRenderer,
    /// particles :)
    particle_renderer: ParticleSystem,
    ui_renderer: UiRenderer,
    path_renderer: PathRenderer,
}

impl Renderer {
    pub fn new(surface: &mut Context, gui_context: &GuiContext) -> Renderer {
        let sprite_renderer = sprite::SpriteRenderer::new(surface);
        let particle_renderer = ParticleSystem::new(surface);
        let ui_renderer = UiRenderer::new(surface, gui_context);
        let path_renderer = PathRenderer::new(surface);
        let mesh_renderer = MeshRenderer::new(surface);
        Self {
            sprite_renderer,
            mesh_renderer,
            particle_renderer,
            ui_renderer,
            path_renderer,
        }
    }

    pub fn prepare_ui(
        &mut self,
        surface: &mut Context,
        gui: Option<Gui>,
        resources: &Resources,
        fonts: &mut GlyphBrush<'static, text::Instance>,
    ) {
        self.ui_renderer.prepare(surface, gui, resources, fonts);
        self.path_renderer.prepare(surface, resources);
    }

    pub fn render(
        &mut self,
        surface: &mut Context,
        back_buffer: &mut Framebuffer<Dim2, (), ()>,
        world: &hecs::World,
        resources: &Resources,
    ) -> Render<PipelineError> {
        let projection_matrix = resources.fetch::<ProjectionMatrix>().unwrap().0;
        let view = crate::core::camera::get_view_matrix(world).unwrap();

        let mut textures = resources.fetch_mut::<AssetManager<SpriteAsset>>().unwrap();
        let mut shaders = resources.fetch_mut::<ShaderManager>().unwrap();
        surface
            .new_pipeline_gate()
            .pipeline(
                back_buffer,
                &PipelineState::default().set_clear_color([0.0, 0.0, 0.0, 1.0]),
                |pipeline, mut shd_gate| {
                    self.sprite_renderer.render(
                        &pipeline,
                        &mut shd_gate,
                        &projection_matrix,
                        &view,
                        &world,
                        &mut *textures,
                    )?;

                    self.mesh_renderer.render(
                        &pipeline,
                        &mut shd_gate,
                        &projection_matrix,
                        &view,
                        &world,
                        &mut *shaders,
                    )?;

                    self.particle_renderer.render(
                        &pipeline,
                        &mut shd_gate,
                        &projection_matrix,
                        &view,
                        world,
                        &mut *textures,
                    )?;

                    self.ui_renderer.render(&pipeline, &mut shd_gate)?;
                    self.path_renderer
                        .render(&projection_matrix, &view, &mut shd_gate)
                },
            )
            .assume()
    }

    pub fn update(
        &mut self,
        _surface: &mut Context,
        world: &hecs::World,
        dt: Duration,
        resources: &Resources,
    ) {
        // update particle systems.
        self.particle_renderer.update(world, dt, resources);
    }
}
