use std::{
    cell::RefCell,
    fmt::Result,
    mem::{align_of, MaybeUninit},
    sync::Arc,
};

use vulkano::{
    buffer::{Buffer, BufferContents, BufferCreateFlags, BufferCreateInfo, BufferUsage, Subbuffer},
    command_buffer::{
        AutoCommandBufferBuilder, BufferCopy, CommandBufferUsage, CopyBufferInfo,
        CopyBufferInfoTyped, PrimaryAutoCommandBuffer, PrimaryCommandBufferAbstract,
    },
    device::DeviceOwned,
    memory::allocator::{
        AllocationCreateInfo, DeviceLayout, MemoryAllocatePreference, MemoryTypeFilter,
    },
    pipeline::{graphics::vertex_input::Vertex, PipelineLayout},
    sync::GpuFuture,
};

use crate::graphics::{pipeline::PipelineBuilder, Graphics};

use super::{Bindable, CommandBufferBuilder};
pub struct VertexBuffer<T>
where
    T: Vertex + BufferContents,
{
    subbuffer: Subbuffer<[T]>,
}

impl<T> Bindable for VertexBuffer<T>
where
    T: Vertex + BufferContents,
{
    fn bind_to_pipeline(&self, builder: &mut PipelineBuilder) {
        builder.vertex_buffer_description = Some(T::per_vertex());
    }

    fn bind(&self, _gfx: &Graphics, builder: &mut CommandBufferBuilder, _: Arc<PipelineLayout>) {
        builder
            .bind_vertex_buffers(0, self.subbuffer.clone())
            .unwrap();
    }
}

impl<T> VertexBuffer<T>
where
    T: Vertex + BufferContents + Clone,
{
    pub fn new(gfx: &Graphics, vertices: Vec<T>) -> Arc<Self>
    where
        T: Vertex + BufferContents,
    {
        let staging_buffer = Buffer::from_iter(
            gfx.get_allocator(),
            BufferCreateInfo {
                usage: BufferUsage::TRANSFER_SRC,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            vertices.into_iter(),
        )
        .expect("Failed to create staging buffer.");

        let main_buffer = Buffer::new(
            gfx.get_allocator(),
            BufferCreateInfo {
                usage: BufferUsage::TRANSFER_DST | BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
                ..Default::default()
            },
            DeviceLayout::from_size_alignment(staging_buffer.size(), align_of::<T>() as u64)
                .unwrap(),
        )
        .expect("Failed to create vertex buffer.");

        let main_subbuffer = Subbuffer::new(main_buffer).cast_aligned();

        let mut builder = AutoCommandBufferBuilder::primary(
            gfx.get_cmd_allocator(),
            gfx.graphics_queue().queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        builder
            .copy_buffer(CopyBufferInfoTyped::buffers(
                staging_buffer,
                main_subbuffer.clone(),
            ))
            .unwrap();

        let fence = builder
            .build()
            .unwrap()
            .execute(gfx.graphics_queue())
            .expect("Failed to execute command buffer.")
            .then_signal_fence_and_flush()
            .expect("Failed to flush command buffer.");
        
        fence.wait(None).unwrap();

        Arc::new(Self {
            subbuffer: main_subbuffer,
        })
    }
}

pub struct IndexBuffer {
    subbuffer: Subbuffer<[u32]>,
}

impl Bindable for IndexBuffer {
    fn bind(&self, _gfx: &Graphics, builder: &mut CommandBufferBuilder, _: Arc<PipelineLayout>) {
        builder.bind_index_buffer(self.subbuffer.clone()).unwrap();
    }
    fn bind_to_pipeline(&self, _builder: &mut PipelineBuilder) {}
}

impl IndexBuffer {
    pub fn new(gfx: &Graphics, indices: Vec<u32>) -> Arc<Self> {
        let staging_buffer = Buffer::from_iter(
            gfx.get_allocator(),
            BufferCreateInfo {
                usage: BufferUsage::TRANSFER_SRC,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            indices.into_iter(),
        )
        .expect("Failed to create staging buffer.");

        let main_buffer = Buffer::new(
            gfx.get_allocator(),
            BufferCreateInfo {
                usage: BufferUsage::TRANSFER_DST | BufferUsage::INDEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
                ..Default::default()
            },
            DeviceLayout::from_size_alignment(staging_buffer.size(), align_of::<u32>() as u64)
                .unwrap(),
        )
        .expect("Failed to create index buffer.");

        let main_subbuffer = Subbuffer::new(main_buffer).cast_aligned();

        let mut builder = AutoCommandBufferBuilder::primary(
            gfx.get_cmd_allocator(),
            gfx.graphics_queue().queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        builder
            .copy_buffer(CopyBufferInfoTyped::buffers(
                staging_buffer,
                main_subbuffer.clone(),
            ))
            .unwrap();

        let fence = builder
            .build()
            .unwrap()
            .execute(gfx.graphics_queue())
            .expect("Failed to execute command buffer.")
            .then_signal_fence_and_flush()
            .expect("Failed to flush command buffer.");

        fence.wait(None).unwrap();

        Arc::new(Self {
            subbuffer: main_subbuffer,
        })
    }
}

pub struct VertexBufferMut<T>
where
    T: Vertex + BufferContents,
{
    main_buffer: Subbuffer<[T]>,
    staging_buffer: Subbuffer<[T]>,

    upload_command_buffer: Arc<PrimaryAutoCommandBuffer>,
    upload_queue: Arc<vulkano::device::Queue>,
}

impl<T> Bindable for VertexBufferMut<T>
where
    T: Vertex + BufferContents,
{
    fn bind_to_pipeline(&self, builder: &mut PipelineBuilder) {
        builder.vertex_buffer_description = Some(T::per_vertex());
    }

    fn bind(&self, _gfx: &Graphics, builder: &mut CommandBufferBuilder, _: Arc<PipelineLayout>) {
        builder
            .bind_vertex_buffers(0, self.main_buffer.clone())
            .unwrap();
    }
}

impl<T> VertexBufferMut<T>
where
    T: Vertex + BufferContents + Clone,
{
    pub fn new(gfx: &Graphics, vertices: Vec<T>) -> Arc<Self>
    where
        T: Vertex + BufferContents,
    {
        let staging_buffer = Buffer::from_iter(
            gfx.get_allocator(),
            BufferCreateInfo {
                usage: BufferUsage::TRANSFER_SRC,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            vertices.into_iter(),
        )
        .expect("Failed to create staging buffer.");

        let main_buffer = Subbuffer::new(
            Buffer::new(
                gfx.get_allocator(),
                BufferCreateInfo {
                    usage: BufferUsage::TRANSFER_DST | BufferUsage::VERTEX_BUFFER,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
                    ..Default::default()
                },
                DeviceLayout::from_size_alignment(staging_buffer.size(), align_of::<T>() as u64)
                    .unwrap(),
            )
            .expect("Failed to create vertex buffer."),
        )
        .cast_aligned();

        let upload_queue = gfx.graphics_queue();
        let upload_command_buffer =
            Self::record_upload_commands(gfx, staging_buffer.clone(), main_buffer.clone());

        upload_command_buffer
            .clone()
            .execute(upload_queue.clone())
            .expect("Failed to execute command buffer.")
            .flush()
            .expect("Failed to flush command buffer.");

        Arc::new(Self {
            upload_command_buffer,
            upload_queue,
            main_buffer,
            staging_buffer,
        })
    }

    fn record_upload_commands(
        gfx: &Graphics,
        staging_buffer: Subbuffer<[T]>,
        main_buffer: Subbuffer<[T]>,
    ) -> Arc<PrimaryAutoCommandBuffer> {
        let mut builder = AutoCommandBufferBuilder::primary(
            gfx.get_cmd_allocator(),
            gfx.graphics_queue().queue_family_index(),
            CommandBufferUsage::MultipleSubmit,
        )
        .unwrap();

        builder
            .copy_buffer(CopyBufferInfo::buffers(staging_buffer, main_buffer))
            .unwrap();

        builder.build().unwrap()
    }

    pub fn write(&self, accessor: impl FnOnce(&mut [T])) {
        let mut write_guard = match self.staging_buffer.write() {
            Ok(guard) => guard,
            Err(e) => {
                println!("Failed to write to staging buffer. {:?}", e);
                return;
            }
        };
        accessor(write_guard.as_mut());
        drop(write_guard);

        let exec = match self
            .upload_command_buffer
            .clone()
            .execute(self.upload_queue.clone())
        {
            Ok(future) => future,
            Err(e) => {
                println!("Failed to execute upload command buffer. {:?}", e);
                return;
            }
        };

        match exec.then_signal_fence_and_flush() {
            Ok(fence) => fence.wait(None).unwrap(),
            Err(e) => println!("Failed to signal flush command buffer. {:?}", e),
        };

    }
}
