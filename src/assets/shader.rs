use crate::assets::{Asset, AssetError, AssetManager, Loader};
use crate::render::mesh::{ShaderUniform, VertexSemantics};
use crate::render::Context;
use luminance_front::context::GraphicsContext;
use luminance_front::shader::Program;
use std::path::{Path, PathBuf};
/// Load with this handle. Filenames for the vertex and fragment shaders
pub type ShaderHandle = (String, String);

pub type ShaderManager = AssetManager<ShaderAsset, ShaderHandle>;

/// Content of the shaders
pub struct ShaderAsset {
    pub vertex_shader: String,
    pub fragment_shader: String,
    pub shader: Option<Program<VertexSemantics, (), ShaderUniform>>,
}

impl Default for ShaderAsset {
    fn default() -> Self {
        Self {
            vertex_shader: "".to_string(),
            fragment_shader: "".to_string(),
            shader: None,
        }
    }
}

pub struct ShaderLoader {
    base_path: PathBuf,
}

impl ShaderLoader {
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        println!(
            "WILL CREATE SHADER LOADER WITH PATH {:?}",
            base_path.as_ref().display()
        );
        let base_path = base_path.as_ref();
        Self {
            base_path: base_path.to_path_buf(),
        }
    }
}

impl Loader<ShaderAsset, ShaderHandle> for ShaderLoader {
    fn load(&mut self, asset_name: (String, String)) -> Asset<ShaderAsset> {
        info!("Will load {:?}", asset_name);
        let vertex_shader_filename = self.base_path.join(asset_name.0);
        let fragment_shader_filename = self.base_path.join(asset_name.1);

        let mut asset = Asset::new();

        match (
            std::fs::read_to_string(&vertex_shader_filename),
            std::fs::read_to_string(&fragment_shader_filename),
        ) {
            (Ok(vertex_shader), Ok(fragment_shader)) => {
                info!("Ok loading shader");
                asset.set_loaded(ShaderAsset {
                    vertex_shader,
                    fragment_shader,
                    shader: None,
                })
            }
            (Err(e), _) | (_, Err(e)) => {
                error!(
                    "Error while loading shader({}/{}) = {:?}",
                    vertex_shader_filename.display(),
                    fragment_shader_filename.display(),
                    e
                );
                asset.set_error(e.into());
            }
        }
        asset
    }

    fn upload_to_gpu(&self, ctx: &mut Context, inner: &mut ShaderAsset) -> Result<(), AssetError> {
        let shader = ctx
            .new_shader_program::<VertexSemantics, (), ShaderUniform>()
            .from_strings(&inner.vertex_shader, None, None, &inner.fragment_shader)?
            .ignore_warnings();
        inner.shader = Some(shader);
        Ok(())
    }
}
