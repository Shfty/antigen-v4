use crate::{RenderPass, RenderPassId, components::SurfaceComponent};
use std::{cell::{Ref, RefCell, RefMut}, collections::{BTreeMap, HashMap}, sync::Arc};

use legion::{Entity, World};
use parking_lot::RwLock;
use wgpu::{Adapter, Device, Instance, PresentMode, Queue, Surface, SurfaceConfiguration};
use winit::{dpi::PhysicalSize, window::Window};

pub struct WgpuManager {
    instance: Arc<Instance>,
    adapter: Arc<Adapter>,
    device: Arc<Device>,
    queue: Arc<Queue>,

    surface_configurations: HashMap<Entity, Arc<RwLock<SurfaceConfiguration>>>,
    surfaces: HashMap<Entity, Surface>,

    render_passes: RefCell<BTreeMap<RenderPassId, Box<dyn RenderPass>>>,
    entity_render_passes: RefCell<HashMap<Entity, Vec<RenderPassId>>>,
}

impl std::ops::Deref for WgpuManager {
    type Target = RefCell<BTreeMap<RenderPassId, Box<dyn RenderPass>>>;

    fn deref(&self) -> &Self::Target {
        &self.render_passes
    }
}

impl WgpuManager {
    pub fn new(instance: Instance, adapter: Adapter, device: Device, queue: Queue) -> Self {
        WgpuManager {
            instance: instance.into(),
            adapter: adapter.into(),
            device: device.into(),
            queue: queue.into(),
            surface_configurations: Default::default(),
            surfaces: Default::default(),
            render_passes: Default::default(),
            entity_render_passes: Default::default(),
        }
    }

    pub fn instance(&self) -> Arc<Instance> {
        self.instance.clone()
    }

    pub fn adapter(&self) -> Arc<Adapter> {
        self.adapter.clone()
    }

    pub fn device(&self) -> Arc<Device> {
        self.device.clone()
    }

    pub fn queue(&self) -> Arc<Queue> {
        self.queue.clone()
    }

    pub fn surface_configuration(
        &self,
        entity: &Entity,
    ) -> Option<&Arc<RwLock<SurfaceConfiguration>>> {
        self.surface_configurations.get(entity)
    }

    pub fn surface(&self, entity: &Entity) -> Option<&Surface> {
        self.surfaces.get(entity)
    }

    pub fn create_surface_for(
        &mut self,
        entity: Entity,
        window: &Window,
    ) -> Arc<RwLock<SurfaceConfiguration>> {
        let size = window.inner_size();
        let surface = unsafe { self.instance.create_surface(window) };
        let swapchain_format = surface.get_preferred_format(&self.adapter).unwrap();

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Mailbox,
        };

        surface.configure(&self.device, &surface_config);

        self.surfaces.insert(entity, surface);

        let surface_config = Arc::new(RwLock::new(surface_config));
        self.surface_configurations
            .insert(entity, surface_config.clone());
        surface_config
    }

    pub fn try_resize_surface(&mut self, entity: &Entity, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }

        let surface = self.surfaces.get(entity).unwrap();
        let mut surface_config = self.surface_configurations.get_mut(entity).unwrap().write();
        surface_config.width = size.width;
        surface_config.height = size.height;
        surface.configure(&self.device, &surface_config);
    }

    pub fn destroy_surface(&mut self, world: &mut World, entity: &Entity) {
        self.surfaces
            .remove(entity)
            .unwrap_or_else(|| panic!("No surface with ID {:?}", entity));

        self.surface_configurations
            .remove(entity)
            .unwrap_or_else(|| panic!("No surface configuration with ID {:?}", entity));

        if let Some(mut entry) = world.entry(*entity) {
            if let Ok(surface) = entry.get_component_mut::<SurfaceComponent>() {
                surface.set_destroyed()
            }
        }
    }

    pub fn add_render_pass(&self, constructor: Box<dyn RenderPass>) -> RenderPassId {
        let id = RenderPassId::next();
        self.render_passes.borrow_mut().insert(id, constructor);
        id
    }

    pub fn render_pass(&self, id: &RenderPassId) -> Option<Ref<'_, Box<dyn RenderPass>>> {
        let render_pass_constructors = self.render_passes.borrow();
        if render_pass_constructors.contains_key(id) {
            Some(Ref::map(render_pass_constructors, |v| v.get(id).unwrap()))
        } else {
            None
        }
    }

    pub fn entity_render_passes(&self, entity: &Entity) -> Option<Ref<'_, Vec<RenderPassId>>> {
        let entity_render_passes = self.entity_render_passes.borrow();
        if entity_render_passes.contains_key(entity) {
            Some(Ref::map(entity_render_passes, |v| v.get(entity).unwrap()))
        } else {
            None
        }
    }

    pub fn render_passes(&self) -> RefMut<'_, BTreeMap<RenderPassId, Box<dyn RenderPass>>> {
        self.render_passes.borrow_mut()
    }

    pub fn register_render_pass_for_entity(&mut self, render_pass: &RenderPassId, entity: &Entity) {
        self.entity_render_passes
            .borrow_mut()
            .entry(*entity)
            .or_default()
            .push(*render_pass)
    }

    pub fn unregister_render_pass_for_entity(
        &mut self,
        render_pass: &RenderPassId,
        entity: &Entity,
    ) {
        let mut entity_render_passes = self.entity_render_passes.borrow_mut();
        let entity_passes = entity_render_passes.entry(*entity).or_default();
        entity_passes.remove(
            entity_passes
                .iter()
                .position(|pass| *pass == *render_pass)
                .unwrap(),
        );
    }

    /// Get a mutable reference to the wgpu manager's entity render passes.
    pub fn entity_render_passes_mut(&mut self) -> &mut RefCell<HashMap<Entity, Vec<RenderPassId>>> {
        &mut self.entity_render_passes
    }

    /// Get a mutable reference to the wgpu manager's render passes.
    pub fn render_passes_mut(
        &mut self,
    ) -> &mut RefCell<BTreeMap<RenderPassId, Box<dyn RenderPass>>> {
        &mut self.render_passes
    }
}
