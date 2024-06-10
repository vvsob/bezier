#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 2],
}

impl Vertex {
    pub fn new(position: [f32; 2]) -> Vertex {
        Vertex { position }
    }

    pub fn new_f64(position: [f64; 2]) -> Vertex {
        Vertex {
            position: position.map(|x| x as f32),
        }
    }

    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x3];

    pub const fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub struct RenderData {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl RenderData {
    pub fn new() -> RenderData {
        RenderData {
            vertices: vec![],
            indices: vec![],
        }
    }

    pub fn merge(self: RenderData, other: RenderData) -> RenderData {
        let vertices_len = self.vertices.len() as u32;
        RenderData {
            vertices: self.vertices.into_iter().chain(other.vertices).collect(),
            indices: self
                .indices
                .into_iter()
                .chain(other.indices.into_iter().map(|i| i + vertices_len))
                .collect(),
        }
    }
}
