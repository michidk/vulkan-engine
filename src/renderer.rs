use std::{
    collections::{BTreeMap, HashMap},
    ffi::{CStr, CString},
    ops::Deref,
};

use ash::{
    extensions::{ext::DebugUtils, khr},
    version::{DeviceV1_0, EntryV1_0, InstanceV1_0},
    vk,
};

use winit::{
    error::OsError,
    event_loop::EventLoopWindowTarget,
    window::{Window, WindowBuilder},
};

use crate::{
    color::Color,
    debug::{
        get_debug_create_info, get_layer_names, has_validation_layers_support,
        startup_debug_severity, startup_debug_type, DebugMessenger, ENABLE_VALIDATION_LAYERS,
    },
};
use math::prelude::*;

#[derive(thiserror::Error, Debug)]
pub enum RendererError {
    #[error("Unknown error")]
    Unknown,
    #[error("Vulkan error: {0}")]
    VkError(#[from] vk::Result),
    #[error("VulkanMemory error: {0}")]
    VkMemError(#[from] vk_mem::error::Error),
    #[error("No suitable gpu found")]
    NoSuitableGpu,
    #[error("No suitable queue family found")]
    NoSuitableQueueFamily,
    #[error("Invalid handle")]
    InvalidHandle,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AppInfo {
    width: u32,
    height: u32,
    title: &'static str,
}

impl AppInfo {
    pub fn into_window<T: 'static>(
        self,
        window_target: &EventLoopWindowTarget<T>,
    ) -> Result<Window, OsError> {
        WindowBuilder::new()
            .with_title(self.title)
            .with_inner_size(winit::dpi::LogicalSize::new(
                f64::from(self.width),
                f64::from(self.height),
            ))
            .build(window_target)
    }
}

pub const DEFAULT_WINDOW_INFO: AppInfo = AppInfo {
    width: 1920,
    height: 1080,
    title: "VulkanTriangle",
};

#[derive(Debug)]
pub struct Camera {
    view_matrix: Mat4<f32>,
    position: Vec3<f32>,
    view_direction: Unit<Vec3<f32>>,
    down_direction: Unit<Vec3<f32>>,
    fovy: f32,
    aspect: f32,
    near: f32,
    far: f32,
    projection_matrix: Mat4<f32>,
}

impl Camera {
    pub fn update_buffer(&self, allocator: &vk_mem::Allocator, buffer: &mut BufferWrapper) {
        let data: [[[f32; 4]; 4]; 2] = [self.view_matrix.into(), self.projection_matrix.into()];
        buffer.fill(allocator, &data).unwrap();
    }

    fn update_projection_matrix(&mut self) {
        let d = 1.0 / (0.5 * self.fovy).tan();
        self.projection_matrix = Mat4::new(
            d / self.aspect,
            0.0,
            0.0,
            0.0,
            0.0,
            d,
            0.0,
            0.0,
            0.0,
            0.0,
            self.far / (self.far - self.near),
            -self.near * self.far / (self.far - self.near),
            0.0,
            0.0,
            1.0,
            0.0,
        );
    }

    fn update_view_matrix(&mut self) {
        // TODO: Unit
        let right = Unit::new_normalize(self.down_direction.cross_product(&self.view_direction));
        let m: Mat4<f32> = Mat4::new(
            *right.x(),
            *right.y(),
            *right.z(),
            -right.dot_product(&self.position),
            //
            *self.down_direction.x(),
            *self.down_direction.y(),
            *self.down_direction.z(),
            -self.down_direction.dot_product(&self.position),
            //
            *self.view_direction.x(),
            *self.view_direction.y(),
            *self.view_direction.z(),
            -self.view_direction.dot_product(&self.position),
            //
            0.0,
            0.0,
            0.0,
            1.0,
        );
        log::debug!("C: {:#?}", m);
        self.view_matrix = m;
    }

    pub fn move_forward(&mut self, distance: f32) {
        log::debug!("B: {:#?}", self.position);
        self.position += self.view_direction.as_ref() * distance;
        log::debug!("A: {:#?}", self.position);
        self.update_view_matrix();
    }

    pub fn move_backward(&mut self, distance: f32) {
        self.move_forward(-distance);
    }

    pub fn turn_right(&mut self, angle: Angle<f32>) {
        let rotation = Mat3::from_axis_angle(&self.down_direction, angle);
        self.view_direction = Unit::new_normalize(&rotation * self.view_direction.as_ref());
        self.update_view_matrix();
    }

    pub fn turn_left(&mut self, angle: Angle<f32>) {
        self.turn_right(-angle);
    }

    pub fn turn_up(&mut self, angle: Angle<f32>) {
        let right = Unit::new_normalize(self.down_direction.cross_product(&self.view_direction));
        let rotation = Mat3::from_axis_angle(&right, angle);
        self.view_direction = Unit::new_normalize(&rotation * self.view_direction.as_ref());
        self.down_direction = Unit::new_normalize(&rotation * self.down_direction.as_ref());
        self.update_view_matrix();
    }

    pub fn turn_down(&mut self, angle: Angle<f32>) {
        self.turn_up(-angle);
    }

    pub fn builder() -> CameraBuilder {
        CameraBuilder {
            position: Vec3::new(0.0, -3.0, -3.0),
            view_direction: Unit::new_normalize(Vec3::new(0.0, 1.0, 1.0)),
            down_direction: Unit::new_normalize(Vec3::new(0.0, 1.0, -1.0)),
            fovy: std::f32::consts::FRAC_PI_3,
            aspect: 800.0 / 600.0,
            near: 0.1,
            far: 100.0,
        }
    }
}

pub struct CameraBuilder {
    position: Vec3<f32>,
    view_direction: Unit<Vec3<f32>>,
    down_direction: Unit<Vec3<f32>>,
    fovy: f32,
    aspect: f32,
    near: f32,
    far: f32,
}

impl CameraBuilder {
    pub fn position(&mut self, pos: Vec3<f32>) -> &mut Self {
        self.position = pos;
        self
    }

    pub fn view_direction(&mut self, direction: Vec3<f32>) -> &mut Self {
        self.view_direction = Unit::new_normalize(direction);
        self
    }

    pub fn down_direction(&mut self, direction: Vec3<f32>) -> &mut Self {
        self.down_direction = Unit::new_normalize(direction);
        self
    }

    pub fn fovy(&mut self, fovy: Angle<f32>) -> &mut Self {
        let fovy = fovy.to_rad();
        const MIN: f32 = 0.01;
        const MAX: f32 = std::f32::consts::PI - 0.01;

        self.fovy = fovy.max(MIN).min(MAX);
        if self.fovy != fovy {
            log::warn!("Fovy out of bounds: {} <= `{}` <= {}", MIN, fovy, MAX);
        }
        self
    }

    pub fn aspect(&mut self, aspect: f32) -> &mut Self {
        self.aspect = aspect;
        self
    }

    pub fn near(&mut self, near: f32) -> &mut Self {
        if near <= 0.0 {
            log::warn!("Near is negative: `{}`", near);
        }
        self.near = near;
        self
    }

    pub fn far(&mut self, far: f32) -> &mut Self {
        if far <= 0.0 {
            log::warn!("Far is negative: `{}`", far);
        }
        self.far = far;
        self
    }

    pub fn build(&mut self) -> Camera {
        if self.far < self.near {
            log::warn!("Far is closer than near: `{}` `{}`", self.far, self.near);
        }
        let down = self.down_direction.as_ref();
        let view = self.view_direction.as_ref();

        let dv = view * down.dot_product(view);
        let ds = down - &dv;

        let mut cam = Camera {
            position: self.position,
            view_direction: self.view_direction,
            down_direction: Unit::new_normalize(ds),
            fovy: self.fovy,
            aspect: self.aspect,
            near: self.near,
            far: self.far,
            view_matrix: Mat4::identity(),
            projection_matrix: Mat4::identity(),
        };
        cam.update_projection_matrix();
        cam.update_view_matrix();
        cam
    }
}

pub struct Model<V, I> {
    vertices: Vec<V>,
    handle_to_index: HashMap<usize, usize>,
    handles: Vec<usize>,
    instances: Vec<I>,
    fist_invisible: usize,
    next_handle: usize,
    vertex_buffer: Option<BufferWrapper>,
    instance_buffer: Option<BufferWrapper>,
}

#[allow(dead_code)]
impl<V, I> Model<V, I> {
    fn get(&self, handle: usize) -> Option<&I> {
        self.instances.get(*self.handle_to_index.get(&handle)?)
    }

    pub fn get_mut(&mut self, handle: usize) -> Option<&mut I> {
        self.instances.get_mut(*self.handle_to_index.get(&handle)?)
    }

    fn swap_by_handle(&mut self, handle1: usize, handle2: usize) -> Result<(), RendererError> {
        if handle1 == handle2 {
            return Ok(());
        }

        if let (Some(&index1), Some(&index2)) = (
            self.handle_to_index.get(&handle1),
            self.handle_to_index.get(&handle2),
        ) {
            self.handles.swap(index1, index2);
            self.instances.swap(index1, index2);
            self.handle_to_index.insert(index1, handle2);
            self.handle_to_index.insert(index2, handle1);
            Ok(())
        } else {
            Err(RendererError::InvalidHandle)
        }
    }

    fn swap_by_index(&mut self, index1: usize, index2: usize) {
        if index1 == index2 {
            return;
        }
        let handle1 = self.handles[index1];
        let handle2 = self.handles[index2];
        self.handles.swap(index1, index2);
        self.instances.swap(index1, index2);
        self.handle_to_index.insert(index1, handle2);
        self.handle_to_index.insert(index2, handle1);
    }

    fn is_visible(&self, handle: usize) -> Result<bool, RendererError> {
        Ok(self
            .handle_to_index
            .get(&handle)
            .ok_or(RendererError::InvalidHandle)?
            < &self.fist_invisible)
    }

    fn make_visible(&mut self, handle: usize) -> Result<(), RendererError> {
        if let Some(&index) = self.handle_to_index.get(&handle) {
            // if already visible to nothing
            if index < self.fist_invisible {
                return Ok(());
            }
            // else: move to position first_invisible and increase value of first_invisible
            self.swap_by_index(index, self.fist_invisible);
            self.fist_invisible += 1;
            Ok(())
        } else {
            Err(RendererError::InvalidHandle)
        }
    }

    fn make_invisible(&mut self, handle: usize) -> Result<(), RendererError> {
        if let Some(&index) = self.handle_to_index.get(&handle) {
            // if already invisible to nothing
            if index >= self.fist_invisible {
                return Ok(());
            }
            // else: move to position first_invisible and increase value of first_invisible
            self.swap_by_index(index, self.fist_invisible - 1);
            self.fist_invisible -= 1;
            Ok(())
        } else {
            Err(RendererError::InvalidHandle)
        }
    }

    fn insert(&mut self, element: I) -> usize {
        let handle = self.next_handle;
        self.next_handle += 1;
        let index = self.instances.len();
        self.instances.push(element);
        self.handles.push(handle);
        self.handle_to_index.insert(handle, index);
        handle
    }

    pub fn insert_visibly(&mut self, element: I) -> usize {
        let new_handle = self.insert(element);
        self.make_visible(new_handle)
            .expect("Failed to make newly inserted handle visible");
        new_handle
    }

    fn remove(&mut self, handle: usize) -> Result<I, RendererError> {
        if let Some(&index) = self.handle_to_index.get(&handle) {
            if index < self.fist_invisible {
                self.swap_by_index(index, self.fist_invisible - 1);
                self.fist_invisible -= 1;
            }
            self.swap_by_index(self.fist_invisible, self.instances.len() - 1);
            self.handles.pop();
            self.handle_to_index.remove(&handle);
            Ok(self.instances.pop().expect("Failed to pop instance"))
        } else {
            Err(RendererError::InvalidHandle)
        }
    }

    pub fn update_vertex_buffer(
        &mut self,
        allocator: &vk_mem::Allocator,
    ) -> Result<(), vk_mem::error::Error> {
        if let Some(buffer) = &mut self.vertex_buffer {
            buffer.fill(allocator, &self.vertices)?;
            Ok(())
        } else {
            let bytes = (self.vertices.len() * std::mem::size_of::<V>()) as u64;
            let mut buffer = BufferWrapper::new(
                &allocator,
                bytes,
                vk::BufferUsageFlags::VERTEX_BUFFER,
                vk_mem::MemoryUsage::CpuToGpu,
            )?;
            buffer.fill(allocator, &self.vertices)?;
            self.vertex_buffer = Some(buffer);
            Ok(())
        }
    }

    pub fn update_instance_buffer(
        &mut self,
        allocator: &vk_mem::Allocator,
    ) -> Result<(), vk_mem::error::Error> {
        if let Some(buffer) = &mut self.instance_buffer {
            buffer.fill(allocator, &self.instances[0..self.fist_invisible])?;
            Ok(())
        } else {
            let bytes = (self.fist_invisible * std::mem::size_of::<I>()) as u64;
            let mut buffer = BufferWrapper::new(
                &allocator,
                bytes,
                vk::BufferUsageFlags::VERTEX_BUFFER,
                vk_mem::MemoryUsage::CpuToGpu,
            )?;
            buffer.fill(allocator, &self.instances[0..self.fist_invisible])?;
            self.instance_buffer = Some(buffer);
            Ok(())
        }
    }

    fn draw(&self, logical_device: &ash::Device, command_buffer: vk::CommandBuffer) {
        if let Some(vertex_buffer) = &self.vertex_buffer {
            if let Some(instance_buffer) = &self.instance_buffer {
                if self.fist_invisible > 0 {
                    unsafe {
                        logical_device.cmd_bind_vertex_buffers(
                            command_buffer,
                            0,
                            &[vertex_buffer.buffer],
                            &[0],
                        );
                        logical_device.cmd_bind_vertex_buffers(
                            command_buffer,
                            1,
                            &[instance_buffer.buffer],
                            &[0],
                        );
                        logical_device.cmd_draw(
                            command_buffer,
                            self.vertices.len() as u32,
                            self.fist_invisible as u32,
                            0,
                            0,
                        );
                    }
                }
            }
        }
    }

    fn cleanup(&mut self, allocator: &vk_mem::Allocator) {
        if let Some(buffer) = &mut self.vertex_buffer {
            buffer.cleanup(allocator)
        }

        if let Some(buffer) = &mut self.instance_buffer {
            buffer.cleanup(allocator)
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct InstanceData {
    pub position: Mat4<f32>,
    pub color: Color,
}

pub type DefaultModel = Model<Vec3<f32>, InstanceData>;

impl DefaultModel {
    pub fn cube() -> Self {
        // lbf: left bottom front
        let lbf = Vec3::new(-1.0, 1.0, 0.0);
        let lbb = Vec3::new(-1.0, 1.0, 1.0);
        let ltf = Vec3::new(-1.0, -1.0, 0.0);
        let ltb = Vec3::new(-1.0, -1.0, 1.0);
        let rbf = Vec3::new(1.0, 1.0, 0.0);
        let rbb = Vec3::new(1.0, 1.0, 1.0);
        let rtf = Vec3::new(1.0, -1.0, 0.0);
        let rtb = Vec3::new(1.0, -1.0, 1.0);

        Model {
            vertices: vec![
                lbf, lbb, rbb, lbf, rbb, rbf, //bottom
                ltf, rtb, ltb, ltf, rtf, rtb, //top
                lbf, rtf, ltf, lbf, rbf, rtf, //front
                lbb, ltb, rtb, lbb, rtb, rbb, //back
                lbf, ltf, lbb, lbb, ltf, ltb, //left
                rbf, rbb, rtf, rbb, rtb, rtf, //right
            ],
            handle_to_index: HashMap::new(),
            handles: Vec::new(),
            instances: Vec::new(),
            fist_invisible: 0,
            next_handle: 0,
            vertex_buffer: None,
            instance_buffer: None,
        }
    }
}

fn init_instance(window: &Window, entry: &ash::Entry) -> Result<ash::Instance, ash::InstanceError> {
    let app_name = CString::new(DEFAULT_WINDOW_INFO.title).unwrap();

    // // https://hoj-senna.github.io/ashen-engine/text/002_Beginnings.html
    let app_info = vk::ApplicationInfo::builder()
        .application_name(&app_name)
        .application_version(vk::make_version(0, 0, 1))
        .engine_name(&app_name)
        .engine_version(vk::make_version(0, 42, 0))
        .api_version(vk::make_version(1, 0, 106));

    // sooo, we need to use display extensions as well
    // let extension_name_pointers: Vec<*const i8> =
    //     vec![ash::extensions::ext::DebugUtils::name().as_ptr()];
    // but let's do it the cool way
    // https://hoj-senna.github.io/ashen-engine/text/006_Window.html

    let surface_extensions = ash_window::enumerate_required_extensions(window).unwrap();
    let mut extension_names_raw = surface_extensions
        .iter()
        .map(|ext| ext.as_ptr())
        .collect::<Vec<_>>();
    extension_names_raw.push(DebugUtils::name().as_ptr()); // still wanna use the debug extensions

    let mut instance_create_info = vk::InstanceCreateInfo::builder()
        .application_info(&app_info)
        .enabled_extension_names(&extension_names_raw);

    // handle validation layers
    let startup_debug_severity = startup_debug_severity();
    let startup_debug_type = startup_debug_type();
    let debug_create_info = &mut get_debug_create_info(startup_debug_severity, startup_debug_type);

    let layer_names = get_layer_names();
    if ENABLE_VALIDATION_LAYERS && has_validation_layers_support(&entry) {
        instance_create_info = instance_create_info
            .push_next(debug_create_info)
            .enabled_layer_names(&layer_names);
    }

    unsafe { entry.create_instance(&instance_create_info, None) }
}

struct SurfaceWrapper {
    surface: vk::SurfaceKHR,
    surface_loader: khr::Surface,
}

impl SurfaceWrapper {
    fn init(window: &Window, entry: &ash::Entry, instance: &ash::Instance) -> SurfaceWrapper {
        // load the surface
        // handles x11 or whatever OS specific drivers
        // this shit is terrible and nobody wants to do it, so lets use ash-window
        let surface = unsafe { ash_window::create_surface(entry, instance, window, None).unwrap() };
        let surface_loader = khr::Surface::new(entry, instance);

        SurfaceWrapper {
            surface,
            surface_loader,
        }
    }

    fn get_capabilities(
        &self,
        physical_device: vk::PhysicalDevice,
    ) -> Result<vk::SurfaceCapabilitiesKHR, vk::Result> {
        unsafe {
            self.surface_loader
                .get_physical_device_surface_capabilities(physical_device, self.surface)
        }
    }

    // fn get_present_modes(
    //     &self,
    //     physical_device: vk::PhysicalDevice,
    // ) -> Result<Vec<vk::PresentModeKHR>, vk::Result> {
    //     unsafe {
    //         self.surface_loader
    //             .get_physical_device_surface_present_modes(physical_device, self.surface)
    //     }
    // }

    fn get_formats(
        &self,
        physical_device: vk::PhysicalDevice,
    ) -> Result<Vec<vk::SurfaceFormatKHR>, vk::Result> {
        unsafe {
            self.surface_loader
                .get_physical_device_surface_formats(physical_device, self.surface)
        }
    }

    fn get_physical_device_surface_support(
        &self,
        physical_device: vk::PhysicalDevice,
        queuefamilyindex: usize,
    ) -> Result<bool, vk::Result> {
        unsafe {
            self.surface_loader.get_physical_device_surface_support(
                physical_device,
                queuefamilyindex as u32,
                self.surface,
            )
        }
    }
}

impl Drop for SurfaceWrapper {
    fn drop(&mut self) {
        unsafe {
            self.surface_loader.destroy_surface(self.surface, None);
        }
    }
}

// choose gpu
// https://hoj-senna.github.io/ashen-engine/text/004_Physical_device.html
// https://vulkan-tutorial.com/Drawing_a_triangle/Setup/Physical_devices_and_queue_families
fn init_physical_device_and_properties(
    instance: &ash::Instance,
) -> Result<
    (
        vk::PhysicalDevice,
        vk::PhysicalDeviceProperties,
        vk::PhysicalDeviceFeatures,
    ),
    RendererError,
> {
    let phys_devs = unsafe { instance.enumerate_physical_devices() }?;
    let mut candidates: BTreeMap<
        u32,
        (
            vk::PhysicalDevice,
            vk::PhysicalDeviceProperties,
            vk::PhysicalDeviceFeatures,
        ),
    > = BTreeMap::new();

    for device in phys_devs {
        let properties = unsafe { instance.get_physical_device_properties(device) };
        let features = unsafe { instance.get_physical_device_features(device) };

        let mut score: u32 = 0;

        // prefere discrete gpu
        if properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
            score += 1000;
        }

        // possible texture size affects graphics quality
        score += properties.limits.max_image_dimension2_d;

        // require geometry shader
        if features.geometry_shader == vk::FALSE {
            score = 0;
        }

        candidates.insert(score, (device, properties, features));

        #[cfg(debug_assertions)]
        {
            let name = String::from(
                unsafe { CStr::from_ptr(properties.device_name.as_ptr()) }
                    .to_str()
                    .unwrap(),
            );
            println!("GPU detected: {}", name);
        }
    }

    if candidates.is_empty() {
        return Err(RendererError::NoSuitableGpu);
    }

    Ok(candidates.pop_first().unwrap().1)
}

struct QueueFamilies {
    graphics_q_index: u32,
}

impl QueueFamilies {
    fn init(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        surfaces: &SurfaceWrapper,
    ) -> Result<QueueFamilies, RendererError> {
        Ok(QueueFamilies {
            graphics_q_index: QueueFamilies::find_suiltable_queue_family(
                instance,
                physical_device,
                surfaces,
            )?,
        })
    }

    fn find_suiltable_queue_family(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        surfaces: &SurfaceWrapper,
    ) -> Result<u32, RendererError> {
        let queuefamilyproperties =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

        let mut found_graphics_q_index = None;
        for (index, qfam) in queuefamilyproperties.iter().enumerate() {
            if qfam.queue_count > 0
                && qfam.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                && surfaces.get_physical_device_surface_support(physical_device, index)?
            {
                found_graphics_q_index = Some(index as u32);
                break;
            }
        }

        found_graphics_q_index.ok_or(RendererError::NoSuitableQueueFamily)
    }
}

pub struct Queues {
    pub graphics_queue: vk::Queue,
}

fn init_device_and_queues(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    queue_families: &QueueFamilies,
) -> Result<(ash::Device, Queues), vk::Result> {
    // select queues
    // https://hoj-senna.github.io/ashen-engine/text/005_Queues.html
    // in this case we only want one queue for now
    let queue_family_index = queue_families.graphics_q_index;
    let device_extension_names_raw = [khr::Swapchain::name().as_ptr()];
    // https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkPhysicalDeviceFeatures.html
    // required for wireframe fill mode
    let features = vk::PhysicalDeviceFeatures::builder().fill_mode_non_solid(true);
    let priorities = [1.0];

    let queue_info = [vk::DeviceQueueCreateInfo::builder()
        .queue_family_index(queue_family_index)
        .queue_priorities(&priorities)
        .build()];

    let device_create_info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_info)
        .enabled_extension_names(&device_extension_names_raw)
        .enabled_features(&features);

    let logical_device: ash::Device =
        unsafe { instance.create_device(physical_device, &device_create_info, None) }?;

    let present_queue = unsafe { logical_device.get_device_queue(queue_family_index as u32, 0) };

    Ok((
        logical_device,
        Queues {
            graphics_queue: present_queue,
        },
    ))
}

#[allow(dead_code)]
pub struct SwapchainWrapper {
    pub swapchain_loader: ash::extensions::khr::Swapchain,
    pub swapchain: vk::SwapchainKHR,
    pub images: Vec<vk::Image>,
    pub imageviews: Vec<vk::ImageView>,
    pub depth_image: vk::Image,
    pub depth_image_allocation: vk_mem::Allocation,
    pub depth_image_allocation_info: vk_mem::AllocationInfo,
    pub depth_imageview: vk::ImageView,
    pub framebuffers: Vec<vk::Framebuffer>,
    pub surface_format: vk::SurfaceFormatKHR,
    pub extent: vk::Extent2D,
    pub image_available: Vec<vk::Semaphore>,
    pub rendering_finished: Vec<vk::Semaphore>,
    pub may_begin_drawing: Vec<vk::Fence>,
    pub amount_of_images: u32,
    pub current_image: usize,
}

impl SwapchainWrapper {
    fn init(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        logical_device: &ash::Device,
        surfaces: &SurfaceWrapper,
        queue_families: &QueueFamilies,
        allocator: &vk_mem::Allocator,
    ) -> Result<SwapchainWrapper, RendererError> {
        let surface_capabilities = surfaces.get_capabilities(physical_device)?;
        let extent = surface_capabilities.current_extent;
        let surface_format = *surfaces.get_formats(physical_device)?.first().unwrap();
        let queuefamilies = [queue_families.graphics_q_index];
        let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surfaces.surface)
            .min_image_count(
                3.max(surface_capabilities.min_image_count)
                    .min(surface_capabilities.max_image_count),
            )
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .queue_family_indices(&queuefamilies)
            .pre_transform(surface_capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(vk::PresentModeKHR::IMMEDIATE);
        let swapchain_loader = ash::extensions::khr::Swapchain::new(instance, logical_device);
        let swapchain = unsafe { swapchain_loader.create_swapchain(&swapchain_create_info, None)? };
        let swapchain_images = unsafe { swapchain_loader.get_swapchain_images(swapchain)? };
        let amount_of_images = swapchain_images.len() as u32;
        let mut swapchain_imageviews = Vec::with_capacity(swapchain_images.len());
        for image in &swapchain_images {
            let subresource_range = vk::ImageSubresourceRange::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1);
            let imageview_create_info = vk::ImageViewCreateInfo::builder()
                .image(*image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(vk::Format::B8G8R8A8_UNORM)
                .subresource_range(*subresource_range);
            let imageview =
                unsafe { logical_device.create_image_view(&imageview_create_info, None) }?;
            swapchain_imageviews.push(imageview);
        }
        let extend_3d = vk::Extent3D {
            width: extent.width,
            height: extent.height,
            depth: 1,
        };
        let depth_image_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            // TODO: maybe optimize wit D24 bit instead
            .format(vk::Format::D32_SFLOAT)
            .extent(extend_3d)
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .queue_family_indices(&queuefamilies);
        let allocation_info = vk_mem::AllocationCreateInfo {
            usage: vk_mem::MemoryUsage::GpuOnly,
            ..Default::default()
        };
        let (depth_image, depth_image_allocation, depth_image_allocation_info) =
            allocator.create_image(&depth_image_info, &allocation_info)?;
        let subresource_range = vk::ImageSubresourceRange::builder()
            .aspect_mask(vk::ImageAspectFlags::DEPTH)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1);
        let imageview_create_info = vk::ImageViewCreateInfo::builder()
            .image(depth_image)
            .view_type(vk::ImageViewType::TYPE_2D)
            // TODO: maybe optimize wit D24 bit instead
            .format(vk::Format::D32_SFLOAT)
            .subresource_range(*subresource_range);
        let depth_imageview =
            unsafe { logical_device.create_image_view(&imageview_create_info, None) }?;
        let mut image_available = vec![];
        let mut rendering_finished = vec![];
        let mut may_begin_drawing = vec![];
        let semaphoreinfo = vk::SemaphoreCreateInfo::builder();
        let fenceinfo = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);
        for _ in 0..amount_of_images {
            let semaphore_available =
                unsafe { logical_device.create_semaphore(&semaphoreinfo, None) }?;
            let semaphore_finished =
                unsafe { logical_device.create_semaphore(&semaphoreinfo, None) }?;
            image_available.push(semaphore_available);
            rendering_finished.push(semaphore_finished);
            let fence = unsafe { logical_device.create_fence(&fenceinfo, None) }?;
            may_begin_drawing.push(fence);
        }

        Ok(SwapchainWrapper {
            swapchain_loader,
            swapchain,
            images: swapchain_images,
            imageviews: swapchain_imageviews,
            depth_image,
            depth_image_allocation,
            depth_image_allocation_info,
            depth_imageview,
            framebuffers: vec![],
            surface_format,
            extent,
            amount_of_images,
            current_image: 0,
            image_available,
            rendering_finished,
            may_begin_drawing,
        })
    }

    fn create_framebuffers(
        &mut self,
        logical_device: &ash::Device,
        renderpass: vk::RenderPass,
    ) -> Result<(), vk::Result> {
        for iv in &self.imageviews {
            let iview = [*iv, self.depth_imageview];
            let framebuffer_info = vk::FramebufferCreateInfo::builder()
                .render_pass(renderpass)
                .attachments(&iview)
                .width(self.extent.width)
                .height(self.extent.height)
                .layers(1);
            let fb = unsafe { logical_device.create_framebuffer(&framebuffer_info, None) }?;
            self.framebuffers.push(fb);
        }
        Ok(())
    }

    unsafe fn cleanup(&mut self, logical_device: &ash::Device, allocator: &vk_mem::Allocator) {
        logical_device.destroy_image_view(self.depth_imageview, None);
        allocator.destroy_image(self.depth_image, &self.depth_image_allocation);

        for fence in &self.may_begin_drawing {
            logical_device.destroy_fence(*fence, None);
        }
        for semaphore in &self.image_available {
            logical_device.destroy_semaphore(*semaphore, None);
        }
        for semaphore in &self.rendering_finished {
            logical_device.destroy_semaphore(*semaphore, None);
        }
        for fb in &self.framebuffers {
            logical_device.destroy_framebuffer(*fb, None);
        }
        for iv in &self.imageviews {
            logical_device.destroy_image_view(*iv, None);
        }
        self.swapchain_loader
            .destroy_swapchain(self.swapchain, None)
    }
}

fn init_renderpass(
    logical_device: &ash::Device,
    format: vk::Format,
) -> Result<vk::RenderPass, vk::Result> {
    let attachments = [
        vk::AttachmentDescription::builder()
            .format(format)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .samples(vk::SampleCountFlags::TYPE_1)
            .build(),
        vk::AttachmentDescription::builder()
            .format(vk::Format::D32_SFLOAT)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::DONT_CARE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .samples(vk::SampleCountFlags::TYPE_1)
            .build(),
    ];
    let color_attachment_references = [vk::AttachmentReference {
        attachment: 0,
        layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
    }];
    let depth_attachment_references = vk::AttachmentReference {
        attachment: 1,
        layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
    };
    let subpasses = [vk::SubpassDescription::builder()
        .color_attachments(&color_attachment_references)
        .depth_stencil_attachment(&depth_attachment_references)
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .build()];
    let subpass_dependencies = [vk::SubpassDependency::builder()
        .src_subpass(vk::SUBPASS_EXTERNAL)
        .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .dst_subpass(0)
        .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .dst_access_mask(
            vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
        )
        .build()];
    let renderpass_info = vk::RenderPassCreateInfo::builder()
        .attachments(&attachments)
        .subpasses(&subpasses)
        .dependencies(&subpass_dependencies);
    let renderpass = unsafe { logical_device.create_render_pass(&renderpass_info, None)? };
    Ok(renderpass)
}

struct Pipeline {
    pipeline: vk::Pipeline,
    layout: vk::PipelineLayout,
    descriptor_set_layouts: Vec<vk::DescriptorSetLayout>,
}

impl Pipeline {
    fn init(
        logical_device: &ash::Device,
        swapchain: &SwapchainWrapper,
        renderpass: &vk::RenderPass,
    ) -> Result<Pipeline, vk::Result> {
        let vertexshader_createinfo = vk::ShaderModuleCreateInfo::builder().code(
            vk_shader_macros::include_glsl!("shaders/triangle.vert", kind: vert),
        );
        let vertexshader_module =
            unsafe { logical_device.create_shader_module(&vertexshader_createinfo, None)? };
        let fragmentshader_createinfo = vk::ShaderModuleCreateInfo::builder()
            .code(vk_shader_macros::include_glsl!("shaders/triangle.frag"));
        let fragmentshader_module =
            unsafe { logical_device.create_shader_module(&fragmentshader_createinfo, None)? };
        let mainfunctionname = std::ffi::CString::new("main").unwrap();
        let vertexshader_stage = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vertexshader_module)
            .name(&mainfunctionname);
        let fragmentshader_stage = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(fragmentshader_module)
            .name(&mainfunctionname);
        let shader_stages = vec![vertexshader_stage.build(), fragmentshader_stage.build()];

        let vertex_attrib_descs = [
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 0,
                offset: 0,
                format: vk::Format::R32G32B32_SFLOAT,
            },
            vk::VertexInputAttributeDescription {
                binding: 1,
                location: 1,
                offset: 0,
                format: vk::Format::R32G32B32A32_SFLOAT,
            },
            vk::VertexInputAttributeDescription {
                binding: 1,
                location: 2,
                offset: 16,
                format: vk::Format::R32G32B32A32_SFLOAT,
            },
            vk::VertexInputAttributeDescription {
                binding: 1,
                location: 3,
                offset: 32,
                format: vk::Format::R32G32B32A32_SFLOAT,
            },
            vk::VertexInputAttributeDescription {
                binding: 1,
                location: 4,
                offset: 48,
                format: vk::Format::R32G32B32A32_SFLOAT,
            },
            vk::VertexInputAttributeDescription {
                binding: 1,
                location: 5,
                offset: 64,
                format: vk::Format::R32G32B32A32_SFLOAT,
            },
        ];
        let vertex_binding_descs = [
            vk::VertexInputBindingDescription {
                binding: 0,
                stride: 12,
                input_rate: vk::VertexInputRate::VERTEX,
            },
            vk::VertexInputBindingDescription {
                binding: 1,
                stride: 80,
                input_rate: vk::VertexInputRate::INSTANCE,
            },
        ];
        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_attribute_descriptions(&vertex_attrib_descs)
            .vertex_binding_descriptions(&vertex_binding_descs);

        let input_assembly_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST);
        let viewports = [vk::Viewport {
            x: 0.,
            y: 0.,
            width: swapchain.extent.width as f32,
            height: swapchain.extent.height as f32,
            min_depth: 0.,
            max_depth: 1.,
        }];
        let scissors = [vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: swapchain.extent,
        }];

        let viewport_info = vk::PipelineViewportStateCreateInfo::builder()
            .viewports(&viewports)
            .scissors(&scissors);
        let rasterizer_info = vk::PipelineRasterizationStateCreateInfo::builder()
            .line_width(1.0)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .cull_mode(vk::CullModeFlags::FRONT)
            .polygon_mode(vk::PolygonMode::FILL);
        let multisampler_info = vk::PipelineMultisampleStateCreateInfo::builder()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);
        let colourblend_attachments = [vk::PipelineColorBlendAttachmentState::builder()
            .blend_enable(true)
            .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .alpha_blend_op(vk::BlendOp::ADD)
            .color_write_mask(
                vk::ColorComponentFlags::R
                    | vk::ColorComponentFlags::G
                    | vk::ColorComponentFlags::B
                    | vk::ColorComponentFlags::A,
            )
            .build()];
        let colourblend_info =
            vk::PipelineColorBlendStateCreateInfo::builder().attachments(&colourblend_attachments);

        let descriptorset_layout_binding_descs = [vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .build()];
        let descriptorset_layout_info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(&descriptorset_layout_binding_descs);
        let descriptorset_layout = unsafe {
            logical_device.create_descriptor_set_layout(&descriptorset_layout_info, None)
        }?;
        let desclayouts = vec![descriptorset_layout];

        let pipelinelayout_info = vk::PipelineLayoutCreateInfo::builder().set_layouts(&desclayouts);
        let pipelinelayout =
            unsafe { logical_device.create_pipeline_layout(&pipelinelayout_info, None) }?;
        let depth_stencil_info = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL);
        let pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly_info)
            .viewport_state(&viewport_info)
            .rasterization_state(&rasterizer_info)
            .multisample_state(&multisampler_info)
            .depth_stencil_state(&depth_stencil_info)
            .color_blend_state(&colourblend_info)
            .layout(pipelinelayout)
            .render_pass(*renderpass)
            .subpass(0);
        let graphicspipeline = unsafe {
            logical_device
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    &[pipeline_info.build()],
                    None,
                )
                .expect("A problem with the pipeline creation")
        }[0];
        unsafe {
            logical_device.destroy_shader_module(fragmentshader_module, None);
            logical_device.destroy_shader_module(vertexshader_module, None);
        }
        Ok(Pipeline {
            pipeline: graphicspipeline,
            layout: pipelinelayout,
            descriptor_set_layouts: desclayouts,
        })
    }

    fn cleanup(&self, logical_device: &ash::Device) {
        unsafe {
            for dsl in &self.descriptor_set_layouts {
                logical_device.destroy_descriptor_set_layout(*dsl, None);
            }
            logical_device.destroy_pipeline(self.pipeline, None);
            logical_device.destroy_pipeline_layout(self.layout, None);
        }
    }
}

struct Pools {
    commandpool_graphics: vk::CommandPool,
}

impl Pools {
    fn init(
        logical_device: &ash::Device,
        queue_families: &QueueFamilies,
    ) -> Result<Pools, vk::Result> {
        let graphics_commandpool_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(queue_families.graphics_q_index)
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
        let commandpool_graphics =
            unsafe { logical_device.create_command_pool(&graphics_commandpool_info, None) }?;
        Ok(Pools {
            commandpool_graphics,
        })
    }

    fn cleanup(&self, logical_device: &ash::Device) {
        unsafe {
            logical_device.destroy_command_pool(self.commandpool_graphics, None);
        }
    }
}

fn create_commandbuffers(
    logical_device: &ash::Device,
    pools: &Pools,
    amount: usize,
) -> Result<Vec<vk::CommandBuffer>, vk::Result> {
    let commandbuf_allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(pools.commandpool_graphics)
        .command_buffer_count(amount as u32);
    unsafe { logical_device.allocate_command_buffers(&commandbuf_allocate_info) }
}

#[allow(dead_code)]
pub struct BufferWrapper {
    buffer: vk::Buffer,
    allocation: vk_mem::Allocation,
    allocation_info: vk_mem::AllocationInfo,
    size_in_bytes: u64,
    buffer_usage: vk::BufferUsageFlags,
    memory_usage: vk_mem::MemoryUsage,
}

impl BufferWrapper {
    fn new(
        allocator: &vk_mem::Allocator,
        size_in_bytes: u64,
        buffer_usage: vk::BufferUsageFlags,
        memory_usage: vk_mem::MemoryUsage,
    ) -> Result<Self, vk_mem::error::Error> {
        let allocation_create_info = vk_mem::AllocationCreateInfo {
            usage: memory_usage,
            ..Default::default()
        };

        let (buffer, allocation, allocation_info) = allocator.create_buffer(
            &vk::BufferCreateInfo::builder()
                .size(size_in_bytes)
                .usage(buffer_usage)
                .build(),
            &allocation_create_info,
        )?;

        Ok(Self {
            buffer,
            allocation,
            allocation_info,
            size_in_bytes,
            buffer_usage,
            memory_usage,
        })
    }

    fn fill<T: Sized>(
        &mut self,
        allocator: &vk_mem::Allocator,
        data: &[T],
    ) -> Result<(), vk_mem::error::Error> {
        let bytes_to_write = (data.len() * std::mem::size_of::<T>()) as u64;
        if bytes_to_write > self.size_in_bytes {
            log::warn!("Not enough memory allocated in buffer; Resizing");
            self.resize(allocator, bytes_to_write)?;
        }

        let data_ptr = allocator.map_memory(&self.allocation)? as *mut T;
        unsafe {
            data_ptr.copy_from_nonoverlapping(data.as_ptr(), data.len());
        };
        allocator.unmap_memory(&self.allocation);
        Ok(())
    }

    fn resize(
        &mut self,
        allocator: &vk_mem::Allocator,
        bytes_to_write: u64,
    ) -> Result<(), vk_mem::error::Error> {
        allocator.destroy_buffer(self.buffer, &self.allocation);
        let new_buffer = BufferWrapper::new(
            allocator,
            bytes_to_write,
            self.buffer_usage,
            self.memory_usage,
        )?;
        *self = new_buffer;
        Ok(())
    }

    fn cleanup(&mut self, allocator: &vk_mem::Allocator) {
        allocator.destroy_buffer(self.buffer, &self.allocation)
    }
}

impl Deref for BufferWrapper {
    type Target = vk::Buffer;

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

#[allow(dead_code)]
pub struct Renderer {
    pub window: winit::window::Window,
    entry: ash::Entry,
    instance: ash::Instance,
    debug: std::mem::ManuallyDrop<DebugMessenger>,
    surfaces: std::mem::ManuallyDrop<SurfaceWrapper>,
    physical_device: vk::PhysicalDevice,
    physical_device_properties: vk::PhysicalDeviceProperties,
    queue_families: QueueFamilies,
    pub queues: Queues,
    pub device: ash::Device,
    pub swapchain: SwapchainWrapper,
    renderpass: vk::RenderPass,
    pipeline: Pipeline,
    pools: Pools,
    pub commandbuffers: Vec<vk::CommandBuffer>,
    pub allocator: vk_mem::Allocator,
    pub models: Vec<DefaultModel>,
    pub uniform_buffer: BufferWrapper,
    descriptor_pool: vk::DescriptorPool,
    descriptor_sets: Vec<vk::DescriptorSet>,
}

impl Renderer {
    pub fn init(window: winit::window::Window) -> Result<Renderer, Box<dyn std::error::Error>> {
        let entry = ash::Entry::new()?;

        let instance = init_instance(&window, &entry)?;
        let debug = DebugMessenger::init(&entry, &instance)?;
        let surfaces = SurfaceWrapper::init(&window, &entry, &instance);

        let (physical_device, physical_device_properties, _physical_device_features) =
            init_physical_device_and_properties(&instance)?;

        let queue_families = QueueFamilies::init(&instance, physical_device, &surfaces)?;

        let (logical_device, queues) =
            init_device_and_queues(&instance, physical_device, &queue_families)?;

        let allocator_create_info = vk_mem::AllocatorCreateInfo {
            physical_device,
            device: logical_device.clone(),
            instance: instance.clone(),
            ..Default::default()
        };
        let allocator = vk_mem::Allocator::new(&allocator_create_info)?;

        let mut swapchain = SwapchainWrapper::init(
            &instance,
            physical_device,
            &logical_device,
            &surfaces,
            &queue_families,
            &allocator,
        )?;
        let format = surfaces
            .get_formats(physical_device)?
            .first()
            .unwrap()
            .format;
        let renderpass = init_renderpass(&logical_device, format)?;
        swapchain.create_framebuffers(&logical_device, renderpass)?;
        let pipeline = Pipeline::init(&logical_device, &swapchain, &renderpass)?;
        let pools = Pools::init(&logical_device, &queue_families)?;

        let commandbuffers =
            create_commandbuffers(&logical_device, &pools, swapchain.framebuffers.len())?;

        let mut uniform_buffer = BufferWrapper::new(
            &allocator,
            128,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk_mem::MemoryUsage::CpuToGpu,
        )?;
        let camera_transform: [[[f32; 4]; 4]; 2] =
            [Mat4::identity().into(), Mat4::identity().into()];
        uniform_buffer.fill(&allocator, &camera_transform)?;

        let pool_size = [vk::DescriptorPoolSize {
            ty: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: swapchain.amount_of_images,
        }];
        let descriptor_pool_info = vk::DescriptorPoolCreateInfo::builder()
            .max_sets(swapchain.amount_of_images)
            .pool_sizes(&pool_size);
        let descriptor_pool =
            unsafe { logical_device.create_descriptor_pool(&descriptor_pool_info, None) }?;
        let desc_layouts =
            vec![pipeline.descriptor_set_layouts[0]; swapchain.amount_of_images as usize];
        let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&desc_layouts);
        let descriptor_sets =
            unsafe { logical_device.allocate_descriptor_sets(&descriptor_set_allocate_info) }?;
        for descset in &descriptor_sets {
            let buffer_infos = [vk::DescriptorBufferInfo {
                buffer: uniform_buffer.buffer,
                offset: 0,
                range: 128,
            }];
            let desc_set_write = [vk::WriteDescriptorSet::builder()
                .dst_set(*descset)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(&buffer_infos)
                .build()];
            unsafe { logical_device.update_descriptor_sets(&desc_set_write, &[]) };
        }

        Ok(Renderer {
            window,
            entry,
            instance,
            debug: std::mem::ManuallyDrop::new(debug),
            surfaces: std::mem::ManuallyDrop::new(surfaces),
            physical_device,
            physical_device_properties,
            queue_families,
            queues,
            device: logical_device,
            swapchain,
            renderpass,
            pipeline,
            pools,
            commandbuffers,
            allocator,
            models: Vec::new(),
            uniform_buffer,
            descriptor_pool,
            descriptor_sets,
        })
    }

    pub fn update_commandbuffer(&mut self, index: usize) -> Result<(), vk::Result> {
        let commandbuffer = self.commandbuffers[index];
        let commandbuffer_begininfo = vk::CommandBufferBeginInfo::builder();
        unsafe {
            self.device
                .begin_command_buffer(commandbuffer, &commandbuffer_begininfo)?;
        }

        let clearvalues = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.08, 1.0],
                },
            },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0,
                },
            },
        ];
        let renderpass_begininfo = vk::RenderPassBeginInfo::builder()
            .render_pass(self.renderpass)
            .framebuffer(self.swapchain.framebuffers[index])
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain.extent,
            })
            .clear_values(&clearvalues);
        unsafe {
            self.device.cmd_begin_render_pass(
                commandbuffer,
                &renderpass_begininfo,
                vk::SubpassContents::INLINE,
            );
            self.device.cmd_bind_pipeline(
                commandbuffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.pipeline,
            );
            self.device.cmd_bind_descriptor_sets(
                commandbuffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.layout,
                0,
                &[self.descriptor_sets[index]],
                &[],
            );
            for m in &self.models {
                m.draw(&self.device, commandbuffer);
            }
            self.device.cmd_end_render_pass(commandbuffer);
            self.device.end_command_buffer(commandbuffer)?;
        }
        Ok(())
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            self.device
                .device_wait_idle()
                .expect("something wrong while waiting");

            self.device
                .destroy_descriptor_pool(self.descriptor_pool, None);
            self.uniform_buffer.cleanup(&self.allocator);

            // if we fail to destroy the buffer continue to destory as many things
            // as possible
            for m in &mut self.models {
                m.cleanup(&self.allocator);
            }

            self.pools.cleanup(&self.device);
            self.pipeline.cleanup(&self.device);
            self.device.destroy_render_pass(self.renderpass, None);
            // --segfault
            self.swapchain.cleanup(&self.device, &self.allocator);
            self.allocator.destroy();
            self.device.destroy_device(None);
            // --segfault
            std::mem::ManuallyDrop::drop(&mut self.surfaces);
            std::mem::ManuallyDrop::drop(&mut self.debug);
            self.instance.destroy_instance(None)
        };
    }
}
