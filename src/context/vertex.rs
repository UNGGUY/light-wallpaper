use cgmath::vec2;
use vulkanalia::vk::{self, HasBuilder};

type Vec2 = cgmath::Vector2<f32>;
pub const INDICES: &[u16] = &[0, 1, 2, 2, 3, 0];
pub static VERTICES: [Vertex; 4] = [
    Vertex::new(vec2(1.0, -1.0), vec2(1.0, 0.0)),
    Vertex::new(vec2(-1.0, -1.0), vec2(0.0, 0.0)),
    Vertex::new(vec2(-1.0, 1.0), vec2(0.0, 1.0)),
    Vertex::new(vec2(1.0, 1.0), vec2(1.0, 1.0)),
];

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pos: Vec2,
    tex_coord: Vec2,
}

impl Vertex {
    const fn new(pos: Vec2, tex_coord: Vec2) -> Self {
        Self { pos, tex_coord }
    }

    pub fn binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::builder()
            .binding(0)
            .input_rate(vk::VertexInputRate::VERTEX)
            .stride(size_of::<Vertex>() as u32)
            .build()
    }

    pub fn attribute_description() -> [vk::VertexInputAttributeDescription; 2] {
        let pos = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(0)
            .format(vk::Format::R32G32_SFLOAT)
            .offset(0)
            .build();

        let tex_coord = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(1)
            .format(vk::Format::R32G32_SFLOAT)
            .offset(size_of::<Vec2>() as u32)
            .build();

        [pos, tex_coord]
    }
}
