use crate::WgpuManager;
use wgpu::{CommandEncoder, TextureView, SurfaceConfiguration};

pub trait RenderPass {
    fn render(
        &mut self,
        wgpu_manager: &WgpuManager,
        encoder: &mut CommandEncoder,
        view: &TextureView,
        config: &SurfaceConfiguration,
    );
}

impl<T> RenderPass for T
where
    T: FnMut(&WgpuManager, &mut CommandEncoder, &TextureView, &SurfaceConfiguration),
{
    fn render(
        &mut self,
        wgpu_manager: &WgpuManager,
        encoder: &mut CommandEncoder,
        view: &TextureView,
        config: &SurfaceConfiguration,
    ) {
        self(wgpu_manager, encoder, view, config)
    }
}

