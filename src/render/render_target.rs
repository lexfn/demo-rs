use super::renderer::Renderer;
use super::texture::Texture;
use super::texture::TextureSize;

pub struct RenderTarget {
    color_tex: Texture,
    depth_tex: Texture,
}

impl RenderTarget {
    pub fn new(rr: &Renderer, size: Option<TextureSize>) -> Self {
        let size = size.unwrap_or(rr.surface_size().into());
        let color_tex = Texture::new_render_attachment(rr, rr.surface_texture_format(), size);
        let depth_tex = Texture::new_depth(rr, rr.depth_texture_format(), size);

        Self {
            color_tex,
            depth_tex,
        }
    }

    pub fn color_texture(&self) -> &Texture {
        &self.color_tex
    }

    pub fn depth_texture(&self) -> &Texture {
        &self.depth_tex
    }

    pub fn resize(&mut self, new_size: TextureSize, rr: &Renderer) {
        *self = RenderTarget::new(rr, Some(new_size));
    }
}
