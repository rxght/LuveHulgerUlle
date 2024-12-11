pub mod bindable;
pub mod camera;
pub mod drawable;
pub mod pipeline;
pub mod shaders;
pub mod utils;

use egui_winit_vulkano::{Gui, GuiConfig};
use smallvec::smallvec;
use std::array;
use std::cmp::min;
use std::collections::HashMap;
use std::panic::Location;
use std::sync::{Arc, OnceLock, RwLock, Weak};
use vulkano::command_buffer::{
    PrimaryAutoCommandBuffer, PrimaryCommandBufferAbstract, RenderPassBeginInfo, SubpassBeginInfo,
    SubpassContents, SubpassEndInfo,
};
use vulkano::descriptor_set::allocator::{
    StandardDescriptorSetAllocator, StandardDescriptorSetAllocatorCreateInfo,
};
use vulkano::format::{ClearValue, FormatFeatures};
use vulkano::image::sampler::ComponentMapping;
use vulkano::image::view::ImageViewType;
use vulkano::image::{Image, ImageCreateInfo, ImageTiling, ImageType};
use vulkano::instance::InstanceCreateFlags;
use vulkano::memory::allocator::AllocationCreateInfo;
use vulkano::render_pass::Subpass;
use vulkano::swapchain::SurfaceInfo;
use vulkano::sync::future::FenceSignalFuture;
use vulkano::{Validated, VulkanError};

use self::drawable::{Drawable, DrawableSharedPart};
use vulkano::{
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
    },
    device::{
        physical::{PhysicalDevice, PhysicalDeviceType},
        Device, DeviceCreateInfo, DeviceExtensions, Features, Queue, QueueCreateInfo, QueueFlags,
    },
    format::Format,
    image::{
        view::{ImageView, ImageViewCreateInfo},
        ImageAspects, ImageSubresourceRange, ImageUsage, SampleCount,
    },
    instance::{debug::ValidationFeatureEnable, Instance, InstanceCreateInfo, InstanceExtensions},
    memory::allocator::StandardMemoryAllocator,
    pipeline::graphics::viewport::Viewport,
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass},
    swapchain::{
        acquire_next_image, ColorSpace, CompositeAlpha, Surface, Swapchain, SwapchainCreateInfo,
        SwapchainPresentInfo,
    },
    sync::{GpuFuture, Sharing},
    Version, VulkanLibrary,
};

use winit::{event_loop::EventLoop, window::Window};

const IN_FLIGHT_COUNT: usize = 2;

const DEVICE_EXTENSIONS: DeviceExtensions = DeviceExtensions {
    khr_swapchain: true,
    ..DeviceExtensions::empty()
};

const INSTANCE_EXTENSIONS: InstanceExtensions = InstanceExtensions {
    #[cfg(debug_assertions)]
    ext_validation_features: true,
    #[cfg(debug_assertions)]
    ext_debug_utils: true,
    ..InstanceExtensions::empty()
};

const ENABLED_FEATURES: Features = Features {
    dynamic_rendering: true,
    ..Features::empty()
};

const ENABLED_VALIDATION_FEATURES: &[ValidationFeatureEnable] = &[
    #[cfg(debug_assertions)]
    ValidationFeatureEnable::BestPractices,
];

const VALIDATION_LAYERS: &[&str] = &[
    #[cfg(debug_assertions)]
    "VK_LAYER_KHRONOS_validation",
];

#[derive(Default)]
struct Queues {
    graphics_queue: Option<Arc<Queue>>,
    present_queue: Option<Arc<Queue>>,
    transfer_queue: Option<Arc<Queue>>,
}

#[derive(Default)]
struct QueueIndices {
    graphics_queue: Option<u32>,
    present_queue: Option<u32>,
    transfer_queue: Option<u32>,
}

impl QueueIndices {
    fn is_complete(&self) -> bool {
        self.graphics_queue.is_some()
            && self.present_queue.is_some()
            && self.transfer_queue.is_some()
    }
}

pub struct Graphics {
    surface: Arc<Surface>,
    window: Arc<Window>,
    device: Arc<Device>,
    queues: Queues,

    allocator: Arc<StandardMemoryAllocator>,
    cmd_allocator: Arc<StandardCommandBufferAllocator>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,

    swapchain: Arc<Swapchain>,
    main_render_pass: Arc<RenderPass>,
    framebuffers: Vec<Arc<Framebuffer>>,

    shared_data_map: RwLock<HashMap<Location<'static>, Weak<DrawableSharedPart>>>,
    draw_queue: Vec<Arc<Drawable>>,

    utils: OnceLock<utils::Utils>,
    gui_system: Gui,

    main_command_buffer: Option<Arc<PrimaryAutoCommandBuffer<Arc<StandardCommandBufferAllocator>>>>,
    futures: [FenceSignalFuture<Box<dyn GpuFuture>>; IN_FLIGHT_COUNT],
    inflight_index: u32,
    framebuffer_index: u32,
}

impl Graphics {
    pub fn new(window: Window, event_loop: &EventLoop<()>) -> Graphics {
        let window = Arc::new(window);

        let library = VulkanLibrary::new().expect("Vulkan library is not installed.");

        let instance = create_instance(library.clone(), event_loop);

        let surface = Surface::from_window(instance.clone(), window.clone()).unwrap();

        let physical_device = create_physical_device(instance.clone(), surface.clone());

        let (device, queues) = create_logical_device(physical_device.clone(), surface.clone());

        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

        let cmd_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            Default::default(),
        ));

        let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            device.clone(),
            StandardDescriptorSetAllocatorCreateInfo::default(),
        ));

        let (swapchain, swapchain_images) = create_swapchain(device.clone(), surface.clone());

        println!("Swapchain is using {:?} images.", swapchain.image_count());

        let swapchain_image_views = create_image_views(&swapchain_images, swapchain.clone());

        let (depth_buffers, depth_format) =
            create_depth_buffers(device.clone(), swapchain.clone(), memory_allocator.clone());

        let main_render_pass =
            create_main_render_pass(device.clone(), swapchain.image_format(), depth_format);

        let framebuffers = create_framebuffers(
            swapchain_image_views,
            main_render_pass.clone(),
            depth_buffers,
        );

        let futures = array::from_fn(|_| {
            let empty_command_buffer = AutoCommandBufferBuilder::primary(
                &cmd_allocator,
                queues.graphics_queue.as_ref().unwrap().queue_family_index(),
                CommandBufferUsage::OneTimeSubmit,
            )
            .unwrap();
            let future = empty_command_buffer
                .build()
                .unwrap()
                .execute(queues.graphics_queue.as_ref().unwrap().clone())
                .unwrap();
            future.boxed().then_signal_fence_and_flush().unwrap()
        });

        let gui_system = Gui::new_with_subpass(
            event_loop,
            surface.clone(),
            queues.graphics_queue.clone().unwrap(),
            Subpass::from(main_render_pass.clone(), 1).unwrap(),
            swapchain.image_format(),
            GuiConfig {
                allow_srgb_render_target: true,
                is_overlay: true,
                ..Default::default()
            },
        );

        #[allow(unused_mut)]
        let mut gfx = Graphics {
            surface,
            window,
            device,
            queues,

            allocator: memory_allocator,
            cmd_allocator,
            descriptor_set_allocator,

            swapchain,
            main_render_pass,
            framebuffers,

            shared_data_map: RwLock::new(HashMap::new()),
            draw_queue: Vec::new(),

            utils: OnceLock::new(),
            gui_system,

            main_command_buffer: None,
            futures,
            inflight_index: 0,
            framebuffer_index: 0,
        };

        _ = gfx.utils.set(utils::Utils::new(&gfx));

        gfx
    }

    pub fn get_device(&self) -> Arc<Device> {
        self.device.clone()
    }
    pub fn get_main_render_pass(&self) -> Arc<RenderPass> {
        self.main_render_pass.clone()
    }
    pub fn get_allocator(&self) -> Arc<StandardMemoryAllocator> {
        self.allocator.clone()
    }
    pub fn get_shared_data(&self, id: &Location<'static>) -> Option<Arc<DrawableSharedPart>> {
        self.shared_data_map
            .read()
            .unwrap()
            .get(id)
            .and_then(Weak::upgrade)
    }
    pub fn get_swapchain_format(&self) -> Format {
        self.swapchain.image_format()
    }
    pub fn get_descriptor_set_allocator(&self) -> &StandardDescriptorSetAllocator {
        &self.descriptor_set_allocator
    }
    pub fn get_window(&self) -> Arc<Window> {
        self.window.clone()
    }
    pub fn graphics_queue(&self) -> Arc<Queue> {
        self.queues.graphics_queue.clone().unwrap()
    }
    pub fn get_cmd_allocator(&self) -> &StandardCommandBufferAllocator {
        &self.cmd_allocator
    }
    pub const fn get_in_flight_count(&self) -> usize {
        IN_FLIGHT_COUNT
    }
    pub fn get_in_flight_index(&self) -> usize {
        self.inflight_index as usize
    }

    pub fn recreate_command_buffer(&mut self) {
        let mut builder = AutoCommandBufferBuilder::primary(
            &self.cmd_allocator,
            self.queues
                .graphics_queue
                .as_ref()
                .unwrap()
                .queue_family_index(),
            CommandBufferUsage::SimultaneousUse,
        )
        .unwrap();

        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: self.swapchain.image_extent().map(|int| int as f32),
            depth_range: 0.0..=1.0,
        };

        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    render_pass: self.main_render_pass.clone(),
                    clear_values: vec![
                        Some(ClearValue::Float([0.0, 0.0, 0.0, 1.0])),
                        Some(ClearValue::Depth(1.0)),
                    ],
                    ..RenderPassBeginInfo::framebuffer(
                        self.framebuffers[self.framebuffer_index as usize].clone(),
                    )
                },
                SubpassBeginInfo {
                    contents: SubpassContents::Inline,
                    ..Default::default()
                },
            )
            .unwrap()
            .set_viewport(0, smallvec![viewport])
            .unwrap();

        for drawable in self.draw_queue.iter() {
            for bindable in drawable.get_bindables() {
                bindable.bind(&self, &mut builder, drawable.get_pipeline_layout());
            }

            for bindable in drawable.get_shared_bindables() {
                bindable.bind(&self, &mut builder, drawable.get_pipeline_layout());
            }

            builder
                .bind_pipeline_graphics(drawable.get_pipeline())
                .unwrap();
            builder
                .draw_indexed(drawable.get_index_count(), 1, 0, 0, 0)
                .unwrap();
        }

        self.draw_queue.truncate(0);

        // move on to gui subpass
        builder
            .next_subpass(
                SubpassEndInfo::default(),
                SubpassBeginInfo {
                    contents: SubpassContents::SecondaryCommandBuffers,
                    ..Default::default()
                },
            )
            .unwrap();

        //render gui
        let cb = self
            .gui_system
            .draw_on_subpass_image(self.swapchain.image_extent());
        builder.execute_commands(cb).unwrap();

        builder.end_render_pass(SubpassEndInfo::default()).unwrap();
        self.main_command_buffer = Some(builder.build().unwrap());
    }

    pub fn is_drawable(&mut self) -> bool {
        let window_size = self.window.inner_size();

        let is_minimized = self.window.is_minimized().unwrap_or(false);
        let is_visible = self.window.is_visible().unwrap_or(true);

        if is_minimized || !is_visible || window_size.width == 0 || window_size.height == 0 {
            return false;
        }
        return true;
    }

    pub fn draw_frame(&mut self) {
        if self.is_swapchain_bad() {
            self.recreate_swapchain();
        }

        let future = &self.futures[self.inflight_index as usize];
        future.wait(None).unwrap();

        let (image_index, suboptimal, acquire_future) =
            acquire_next_image(self.swapchain.clone(), None).unwrap();

        self.framebuffer_index = image_index;

        match self.main_command_buffer.as_ref() {
            None => self.recreate_command_buffer(),
            _ => {}
        };

        let new_future_result = acquire_future
            .then_execute(
                self.queues.graphics_queue.clone().unwrap(),
                self.main_command_buffer.take().unwrap(),
            )
            .unwrap()
            .then_swapchain_present(
                self.queues.graphics_queue.clone().unwrap(),
                SwapchainPresentInfo::swapchain_image_index(self.swapchain.clone(), image_index),
            )
            .boxed()
            .then_signal_fence_and_flush();

        self.futures[self.inflight_index as usize] = match new_future_result {
            Ok(v) => v,
            Err(Validated::Error(VulkanError::OutOfDate)) => {
                self.recreate_swapchain();
                return;
            }
            Err(e) => panic!("Failed to flush future: {e:?}"),
        };

        if suboptimal {
            self.recreate_swapchain();
        }
        self.inflight_index = (self.inflight_index + 1) % IN_FLIGHT_COUNT as u32;
    }

    pub fn queue_drawable(&mut self, drawable: Arc<Drawable>) {
        self.draw_queue.push(drawable);
    }

    pub fn recreate_swapchain(&mut self) {
        let capabilities = self
            .device
            .physical_device()
            .surface_capabilities(&self.surface, SurfaceInfo::default())
            .unwrap();

        let extent: [u32; 2] = self.window.inner_size().into();
        let extent = extent.clamp(capabilities.min_image_extent, capabilities.max_image_extent);

        let create_info = SwapchainCreateInfo {
            image_extent: extent,
            ..self.swapchain.create_info()
        };

        let (swapchain, swapchain_images) = match self.swapchain.recreate(create_info).ok() {
            Some(v) => v,
            None => return,
        };

        let image_views = create_image_views(&swapchain_images, swapchain.clone());

        let (depth_buffers, _) = create_depth_buffers(
            self.device.clone(),
            swapchain.clone(),
            self.allocator.clone(),
        );

        let framebuffers =
            create_framebuffers(image_views, self.main_render_pass.clone(), depth_buffers);

        self.swapchain = swapchain;
        self.framebuffers = framebuffers;

        self.utils.get().unwrap().recreate(&self);
    }

    pub fn cache_drawable_shared_part(
        &self,
        shared_id: &Location<'static>,
        shared_part: Arc<DrawableSharedPart>,
    ) {
        self.shared_data_map
            .write()
            .unwrap()
            .insert(*shared_id, Arc::downgrade(&shared_part));
    }

    pub fn utils(&self) -> &utils::Utils {
        self.utils.get().unwrap()
    }

    pub fn gui(&mut self) -> &mut Gui {
        &mut self.gui_system
    }

    fn is_swapchain_bad(&self) -> bool {
        let window_size: [u32; 2] = self.window.inner_size().into();
        println!("window_size: {window_size:?}");
        if self.swapchain.image_extent() != window_size {
            return true;
        }
        return false;
    }
}

fn create_instance(library: Arc<VulkanLibrary>, event_loop: &EventLoop<()>) -> Arc<Instance> {
    let required_extensions = Surface::required_extensions(event_loop);

    let create_info = InstanceCreateInfo {
        application_name: Some(String::from("Rosten")),
        flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
        enabled_extensions: required_extensions.union(&INSTANCE_EXTENSIONS),
        enabled_layers: VALIDATION_LAYERS.iter().map(|p| String::from(*p)).collect(),
        max_api_version: None,
        enabled_validation_features: Vec::from(ENABLED_VALIDATION_FEATURES),
        ..InstanceCreateInfo::default()
    };

    Instance::new(library.clone(), create_info).expect("Failed to create instance!")
}

fn create_physical_device(instance: Arc<Instance>, surface: Arc<Surface>) -> Arc<PhysicalDevice> {
    let physical_device = instance
        .enumerate_physical_devices()
        .expect("No appropriate physical device found!")
        .filter(|p| is_device_suitable(p.clone(), surface.clone()))
        .min_by_key(|p| {
            // We assign a lower score to device types that are likely to be faster/better.
            match p.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                PhysicalDeviceType::Other => 4,
                _ => 5,
            }
        })
        .expect("no suitable physical device found");

    // Some little debug infos.
    println!(
        "Using device: {} (type: {:?})",
        physical_device.properties().device_name,
        physical_device.properties().device_type,
    );

    physical_device
}

fn is_device_suitable(physical_device: Arc<PhysicalDevice>, surface: Arc<Surface>) -> bool {
    (physical_device.api_version() >= Version::V1_3
        || physical_device.supported_extensions().khr_dynamic_rendering)
        && physical_device
            .supported_extensions()
            .contains(&DEVICE_EXTENSIONS)
        && {
            let indices = find_queue_indices(physical_device.clone(), surface.clone());
            indices.graphics_queue.is_some() && indices.present_queue.is_some()
        }
}

fn find_queue_indices(physical_device: Arc<PhysicalDevice>, surface: Arc<Surface>) -> QueueIndices {
    let mut indices = QueueIndices::default();

    for (i, properties) in physical_device.queue_family_properties().iter().enumerate() {
        let flags = properties.queue_flags;
        if indices.graphics_queue.is_none() && flags.contains(QueueFlags::GRAPHICS) {
            indices.graphics_queue = Some(i as u32);
        }
        if indices.present_queue.is_none()
            && physical_device
                .surface_support(i as u32, &surface)
                .unwrap_or(false)
        {
            indices.present_queue = Some(i as u32);
        }
        if indices.transfer_queue.is_none()
            && flags.contains(QueueFlags::TRANSFER)
            && !(flags.contains(QueueFlags::GRAPHICS))
        {
            indices.transfer_queue = Some(i as u32);
        }
        if indices.is_complete() {
            break;
        }
    }
    indices
}

fn create_logical_device(
    physical_device: Arc<PhysicalDevice>,
    surface: Arc<Surface>,
) -> (Arc<Device>, Queues) {
    let mut extensions = DEVICE_EXTENSIONS.clone();

    if physical_device.api_version() < Version::V1_3 {
        extensions.khr_dynamic_rendering = true;
    }

    let indices = find_queue_indices(physical_device.clone(), surface.clone());
    let mut index_set = vec![indices.graphics_queue.unwrap()];

    if !index_set.contains(&indices.present_queue.unwrap()) {
        index_set.push(indices.present_queue.unwrap());
    }

    if indices.transfer_queue.is_some() && !index_set.contains(&indices.transfer_queue.unwrap()) {
        index_set.push(indices.transfer_queue.unwrap());
    }

    let create_info = DeviceCreateInfo {
        enabled_extensions: extensions,
        enabled_features: ENABLED_FEATURES,
        queue_create_infos: index_set
            .iter()
            .map(|p| QueueCreateInfo {
                queue_family_index: *p,
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    };

    let (device, mut queue_iter) = Device::new(physical_device.clone(), create_info)
        .expect("Failed to create logical device!");

    let mut queues = Queues::default();

    queues.graphics_queue = queue_iter.next();

    if !index_set.contains(&indices.present_queue.unwrap()) {
        dbg!("Forced to use a dedicated present queue,");
        queues.present_queue = queue_iter.next();
    } else {
        queues.present_queue = queues.graphics_queue.clone();
    }

    if indices.transfer_queue.is_some() && !index_set.contains(&indices.transfer_queue.unwrap()) {
        dbg!("Found support for dedicated transfer queue.");
        queues.transfer_queue = queue_iter.next();
    } else {
        queues.transfer_queue = queues.graphics_queue.clone();
    }

    (device, queues)
}

fn create_swapchain(
    device: Arc<Device>,
    surface: Arc<Surface>,
) -> (Arc<Swapchain>, Vec<Arc<Image>>) {
    let capabilities = device
        .physical_device()
        .surface_capabilities(surface.as_ref(), Default::default())
        .unwrap();

    let formats = device
        .physical_device()
        .surface_formats(surface.as_ref(), Default::default())
        .unwrap();

    let present_modes = device
        .physical_device()
        .surface_present_modes(surface.as_ref(), SurfaceInfo::default())
        .unwrap();

    let surface_format = formats
        .iter()
        .find(|(format, color_space)| {
            *format == Format::R8G8B8A8_SRGB && *color_space == ColorSpace::SrgbNonLinear
        })
        .unwrap_or(formats.first().unwrap());

    let extent: [u32; 2] = match capabilities.current_extent {
        Some(current) => current,
        None => {
            let window: &Window = surface.object().unwrap().downcast_ref().unwrap();
            let framebuffer_extent = window.inner_size();
            let width = framebuffer_extent.width;
            let height = framebuffer_extent.height;
            [
                width.clamp(
                    capabilities.min_image_extent[0],
                    capabilities.max_image_extent[0],
                ),
                height.clamp(
                    capabilities.min_image_extent[1],
                    capabilities.max_image_extent[1],
                ),
            ]
        }
    };

    // if vsync is enabled then PresentMode::Immediate should be filtered out here
    let present_mode: vulkano::swapchain::PresentMode = present_modes
        .min_by_key(|p| (*p as u32).wrapping_sub(1))
        .unwrap();

    println!("Using present mode: {present_mode:?}");

    let indices = find_queue_indices(device.physical_device().clone(), surface.clone());
    let image_sharing = if indices.graphics_queue == indices.present_queue {
        Sharing::Exclusive
    } else {
        Sharing::Concurrent(smallvec::smallvec![
            indices.graphics_queue.unwrap(),
            indices.present_queue.unwrap()
        ])
    };

    let create_info = SwapchainCreateInfo {
        min_image_count: match capabilities.max_image_count {
            Some(max) => min(capabilities.min_image_count + 1, max),
            None => capabilities.min_image_count + 1,
        },
        image_format: surface_format.0,
        image_color_space: surface_format.1,
        image_extent: extent,
        image_array_layers: 1,
        image_usage: ImageUsage::COLOR_ATTACHMENT,
        image_sharing: image_sharing,
        pre_transform: capabilities.current_transform,
        composite_alpha: capabilities
            .supported_composite_alpha
            .into_iter()
            .min_by_key(|p| match *p {
                CompositeAlpha::Opaque => 0,
                _ => 1,
            })
            .unwrap(),
        present_mode: present_mode,
        clipped: true,
        ..Default::default()
    };

    Swapchain::new(device.clone(), surface.clone(), create_info)
        .expect("Failed to create Swapchain!")
}

fn create_image_views(images: &Vec<Arc<Image>>, swapchain: Arc<Swapchain>) -> Vec<Arc<ImageView>> {
    images
        .iter()
        .map(|image| {
            ImageView::new(
                image.clone(),
                ImageViewCreateInfo {
                    view_type: ImageViewType::Dim2d,
                    format: swapchain.image_format(),
                    component_mapping: ComponentMapping::identity(),
                    subresource_range: ImageSubresourceRange {
                        aspects: ImageAspects::COLOR,
                        mip_levels: 0..1,
                        array_layers: 0..1,
                    },
                    usage: ImageUsage::COLOR_ATTACHMENT,
                    ..Default::default()
                },
            )
            .unwrap()
        })
        .collect()
}

fn create_main_render_pass(
    device: Arc<Device>,
    swapchain_format: Format,
    depth_format: Format,
) -> Arc<RenderPass> {
    vulkano::ordered_passes_renderpass!(
        device,
        attachments: {
            color: {
                format: swapchain_format,
                samples: SampleCount::Sample1,
                load_op: Clear,
                store_op: Store,
            },
            depth: {
                format: depth_format,
                samples: SampleCount::Sample1,
                load_op: Clear,
                store_op: Store,
            }
        },
        passes: [
            { color: [color], depth_stencil: {depth}, input: [] },
            { color: [color], depth_stencil: {}, input: [] } // gui render pass
        ]
    )
    .unwrap()
}

fn create_framebuffers(
    image_views: Vec<Arc<ImageView>>,
    render_pass: Arc<RenderPass>,
    depth_buffers: Vec<Arc<ImageView>>,
) -> Vec<Arc<Framebuffer>> {
    image_views
        .into_iter()
        .zip(depth_buffers.into_iter())
        .map(|(image, depth_buffer)| {
            Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![image, depth_buffer],
                    extent: [0, 0],
                    layers: 1,
                    ..Default::default()
                },
            )
            .unwrap()
        })
        .collect()
}

fn select_image_format(
    device: Arc<Device>,
    tiling: ImageTiling,
    features: FormatFeatures,
    candidates: &[Format],
) -> Option<Format> {
    for format in candidates {
        let props = device.physical_device().format_properties(*format).unwrap();
        if tiling == ImageTiling::Optimal && props.optimal_tiling_features.contains(features) {
            return Some(*format);
        }
        if tiling == ImageTiling::Linear && props.linear_tiling_features.contains(features) {
            return Some(*format);
        }
    }
    None
}

fn create_depth_buffers(
    device: Arc<Device>,
    swapchain: Arc<Swapchain>,
    allocator: Arc<StandardMemoryAllocator>,
) -> (Vec<Arc<ImageView>>, Format) {
    let format_candidates = [
        Format::D16_UNORM,
        Format::D32_SFLOAT,
        Format::D16_UNORM_S8_UINT,
        Format::D24_UNORM_S8_UINT,
        Format::D32_SFLOAT_S8_UINT,
    ];

    let format = select_image_format(
        device.clone(),
        ImageTiling::Optimal,
        FormatFeatures::DEPTH_STENCIL_ATTACHMENT,
        &format_candidates,
    )
    .unwrap();

    let [x, y] = swapchain.image_extent();
    let extent = [x, y, 1];

    let depth_buffers = (0..swapchain.image_count())
        .map(|_| {
            ImageView::new_default(
                Image::new(
                    allocator.clone(),
                    ImageCreateInfo {
                        image_type: ImageType::Dim2d,
                        format: Format::D16_UNORM,
                        extent,
                        usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT
                            | ImageUsage::TRANSIENT_ATTACHMENT,
                        ..Default::default()
                    },
                    AllocationCreateInfo::default(),
                )
                .unwrap(),
            )
            .unwrap()
        })
        .collect();

    (depth_buffers, format)
}
