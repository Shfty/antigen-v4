use antigen_winit::{WindowComponent, WindowState};
use atomic_id::*;
use legion::{Entity, World};
use parking_lot::RwLock;
use remote_channel::*;
use serde::ser::SerializeStruct;
use std::{
    cell::{Ref, RefCell, RefMut},
    collections::{BTreeMap, HashMap},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use wgpu::{
    Adapter, Buffer, CommandEncoder, Device, Instance, PresentMode, Queue, Surface,
    SurfaceConfiguration, TextureView,
};
use winit::{dpi::PhysicalSize, window::Window};

pub type WgpuRequester = RemoteRequester<WgpuManager, (), World>;
pub type WgpuResponder = RemoteResponder<WgpuManager, (), World>;

atomic_id!(NEXT_RENDER_PASS_ID, RenderPassId);
atomic_id!(NEXT_BUFFER_ID, BufferId);

#[derive(Debug)]
pub enum SurfaceState {
    Invalid,
    Pending,
    Valid(Arc<RwLock<SurfaceConfiguration>>),
    Destroyed,
}

impl Default for SurfaceState {
    fn default() -> Self {
        SurfaceState::Invalid
    }
}

impl serde::Serialize for SurfaceState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            SurfaceState::Invalid => serializer.serialize_str("Invalid"),
            SurfaceState::Pending => serializer.serialize_str("Pending"),
            SurfaceState::Valid(_) => serializer.serialize_str("Valid"),
            SurfaceState::Destroyed => serializer.serialize_str("Destroyed"),
        }
    }
}

impl<'de> serde::Deserialize<'de> for SurfaceState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        unimplemented!()
    }
}

#[derive(Debug)]
pub struct SurfaceComponent {
    state: SurfaceState,
    present_mode: PresentMode,
}

impl Default for SurfaceComponent {
    fn default() -> Self {
        SurfaceComponent {
            state: Default::default(),
            present_mode: PresentMode::Mailbox,
        }
    }
}

impl SurfaceComponent {
    pub fn new(present_mode: PresentMode) -> Self {
        SurfaceComponent {
            present_mode,
            ..Default::default()
        }
    }

    pub fn state(&self) -> &SurfaceState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut SurfaceState {
        &mut self.state
    }

    pub fn set_invalid(&mut self) {
        self.state = SurfaceState::Invalid;
    }

    pub fn set_pending(&mut self) {
        self.state = SurfaceState::Pending;
    }

    pub fn set_valid(&mut self, config: Arc<RwLock<SurfaceConfiguration>>) {
        self.state = SurfaceState::Valid(config);
    }

    pub fn set_destroyed(&mut self) {
        self.state = SurfaceState::Destroyed;
    }
}

impl serde::Serialize for SurfaceComponent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("SurfaceComponent", 2)?;
        s.serialize_field("state", &self.state)?;
        s.serialize_field(
            "present_mode",
            match self.present_mode {
                PresentMode::Immediate => "Immediate",
                PresentMode::Mailbox => "Mailbox",
                PresentMode::Fifo => "Fifo",
            },
        )?;
        s.end()
    }
}

impl<'de> serde::Deserialize<'de> for SurfaceComponent {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        unimplemented!()
    }
}

pub struct WgpuManager {
    instance: Arc<Instance>,
    adapter: Arc<Adapter>,
    device: Arc<Device>,
    queue: Arc<Queue>,

    surface_configurations: HashMap<Entity, Arc<RwLock<SurfaceConfiguration>>>,
    surfaces: HashMap<Entity, Surface>,

    render_passes: RefCell<BTreeMap<RenderPassId, Box<dyn RenderPass>>>,
    entity_render_passes: RefCell<HashMap<Entity, Vec<RenderPassId>>>,

    buffers: RefCell<BTreeMap<BufferId, Buffer>>,
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
            buffers: Default::default(),
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

    pub fn surface_configuration(&self, entity: &Entity) -> Option<&Arc<RwLock<SurfaceConfiguration>>> {
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

    pub fn try_resize_surface(
        &mut self,
        entity: &Entity,
        size: PhysicalSize<u32>,
    ) {
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

    pub fn add_buffer(&self, buffer: Buffer) -> BufferId {
        let id = BufferId::next();
        self.buffers.borrow_mut().insert(id, buffer);
        id
    }

    pub fn buffer(&self, buffer: &BufferId) -> Option<Ref<'_, Buffer>> {
        let buffers = self.buffers.borrow();
        if buffers.contains_key(buffer) {
            Some(Ref::map(buffers, |v| v.get(buffer).unwrap()))
        } else {
            None
        }
    }
}

#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub enum RenderPassState {
    Invalid,
    Pending,
    Registered,
    Unregistered,
}

impl Default for RenderPassState {
    fn default() -> Self {
        RenderPassState::Invalid
    }
}

#[derive(Debug, Clone, Default)]
pub struct RenderPassComponent {
    passes: Vec<(RenderPassState, RenderPassId)>,
}

impl serde::Serialize for RenderPassComponent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("RenderPassComponent", 2)?;
        s.serialize_field("passes", &self.passes)?;
        s.end()
    }
}

impl<'de> serde::Deserialize<'de> for RenderPassComponent {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        unimplemented!()
    }
}

impl RenderPassComponent {
    pub fn add_render_pass(&mut self, render_pass: RenderPassId) {
        self.passes.push((Default::default(), render_pass))
    }

    pub fn remove_render_pass(&mut self, render_pass: RenderPassId) {
        self.passes
            .iter_mut()
            .find(|(_, pass)| *pass == render_pass)
            .unwrap()
            .0 = RenderPassState::Unregistered;
    }
}

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

#[legion::system(par_for_each)]
pub fn create_surfaces(
    entity: &Entity,
    window: &WindowComponent,
    surface: &mut SurfaceComponent,
    #[resource] wgpu_requester: &WgpuRequester,
) {
    let entity = *entity;
    if let WindowState::Valid(window) = window.state() {
        if let SurfaceState::Invalid = surface.state() {
            surface.set_pending();

            let window = window.clone();
            wgpu_requester.send_request(Box::new(move |wgpu_manager, _| {
                let config = wgpu_manager.create_surface_for(entity, &window);
                window.request_redraw();

                Box::new(move |world: &mut World| {
                    if let Some(mut entry) = world.entry(entity) {
                        if let Ok(surface) = entry.get_component_mut::<SurfaceComponent>() {
                            surface.set_valid(config);
                        }
                    }
                })
            }))
        }
    }
}

#[legion::system(par_for_each)]
pub fn register_render_passes(
    entity: &Entity,
    render_passes: &mut RenderPassComponent,
    #[resource] wgpu_requester: &WgpuRequester,
) {
    let entity = *entity;

    for (state, render_pass) in render_passes.passes.iter_mut() {
        let render_pass = *render_pass;

        if let RenderPassState::Invalid = state {
            *state = RenderPassState::Pending;
            wgpu_requester.send_request(Box::new(move |wgpu_manager, _| {
                wgpu_manager.register_render_pass_for_entity(&render_pass, &entity);
                Box::new(move |world: &mut World| {
                    if let Some(mut entry) = world.entry(entity) {
                        if let Ok(render_passes) = entry.get_component_mut::<RenderPassComponent>()
                        {
                            render_passes
                                .passes
                                .iter_mut()
                                .find(|(_, pass)| *pass == render_pass)
                                .unwrap()
                                .0 = RenderPassState::Registered;
                        }
                    }
                })
            }));
        }
    }
}
