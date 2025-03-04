use std::{fmt::Result, mem::align_of, sync::Arc};

use vulkano::{
    buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer},
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferUsage, CopyBufferInfoTyped,
        PrimaryCommandBufferAbstract,
    },
    memory::allocator::{AllocationCreateInfo, DeviceLayout, MemoryTypeFilter},
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
    T: Vertex + BufferContents,
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
        .expect("Failed to create vertex buffer.");

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
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();

        fence.wait(None).unwrap();

        Arc::new(Self {
            subbuffer: main_subbuffer,
        })
    }

    pub fn write(
        &self,
    ) -> core::result::Result<
        vulkano::buffer::BufferWriteGuard<'_, [T]>,
        vulkano::sync::HostAccessError,
    > {
        self.subbuffer.write()
    }

    pub fn read(
        &self,
    ) -> core::result::Result<
        vulkano::buffer::BufferReadGuard<'_, [T]>,
        vulkano::sync::HostAccessError,
    > {
        self.subbuffer.read()
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
        .expect("Failed to create index buffer.");

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
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();

        fence.wait(None).unwrap();

        Arc::new(Self {
            subbuffer: main_subbuffer,
        })
    }
}
