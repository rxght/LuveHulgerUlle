use std::{
    cell::{Cell, UnsafeCell},
    sync::Arc,
};

use vulkano::{
    buffer::BufferContents, pipeline::graphics::vertex_input::Vertex, shader::ShaderStages,
};

use crate::graphics::{
    bindable::{self, Texture, TextureBinding, UniformBuffer, UniformBufferBinding},
    drawable::Drawable,
    shaders::{frag_textured, vert_ui_textured},
    ui::{Rectangle, UiElement, UiLayout},
    Graphics,
};

pub struct UiImage {
    layout: UnsafeCell<UiLayout>,
    descriptor: Cell<Rectangle<f32>>,
    drawable: Arc<Drawable>,
    layout_data: Arc<UniformBuffer<vert_ui_textured::LayoutData>>,
}

impl UiImage {
    pub fn new(gfx: &mut Graphics, texture: Arc<Texture>, layout: UiLayout) -> Arc<Self> {
        let window_size: [u32; 2] = gfx.get_window().inner_size().into();
        let descriptor = layout.to_normalized(window_size);

        let layout_data = UniformBuffer::new(
            gfx,
            0,
            vert_ui_textured::LayoutData {
                position: [descriptor.x, descriptor.y],
                dimensions: [descriptor.width, descriptor.height],
            },
            ShaderStages::VERTEX,
        );

        let drawable = Drawable::new(
            gfx,
            vec![
                UniformBufferBinding::new(layout_data.clone(), 0),
                TextureBinding::new(texture.clone(), 1),
            ],
            || {
                #[derive(BufferContents, Vertex)]
                #[repr(C)]
                struct Vertex {
                    #[format(R32G32_SFLOAT)]
                    pos: [f32; 2],
                }

                let vertices = vec![
                    Vertex { pos: [0.0, 0.0] },
                    Vertex { pos: [1.0, 0.0] },
                    Vertex { pos: [0.0, 1.0] },
                    Vertex { pos: [1.0, 1.0] },
                ];

                let indices: Vec<u32> = vec![0, 1, 3, 0, 3, 2];

                vec![
                    bindable::VertexBuffer::new(gfx, vertices),
                    bindable::IndexBuffer::new(gfx, indices),
                    bindable::VertexShader::from_module(
                        vert_ui_textured::load(gfx.get_device()).unwrap(),
                    ),
                    bindable::FragmentShader::from_module(
                        frag_textured::load(gfx.get_device()).unwrap(),
                    ),
                ]
            },
            6,
        );

        Arc::new(Self {
            layout: UnsafeCell::new(layout),
            descriptor: Cell::new(descriptor),
            drawable,
            layout_data,
        })
    }

    pub fn layout(&self) -> &UiLayout {
        unsafe { &*self.layout.get() }
    }

    pub fn layout_mut(&self) -> &mut UiLayout {
        unsafe { &mut *self.layout.get() }
    }
}

impl UiElement for UiImage {
    fn handle_resize(&self, new_size: [u32; 2]) {
        let descriptor = self.layout().to_normalized(new_size);
        self.layout_data.access_data(|data| {
            data.position = [descriptor.x, descriptor.y];
            data.dimensions = [descriptor.width, descriptor.height];
        });
        self.descriptor.set(descriptor);
    }
    fn get_drawable(&self) -> Arc<Drawable> {
        self.drawable.clone()
    }
    fn get_layout(&self) -> Rectangle<f32> {
        self.descriptor.get()
    }
}
