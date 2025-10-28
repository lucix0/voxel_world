use wgpu::BindingType::Texture;
use wgpu::util::DeviceExt;
use crate::game::chunk::{Chunk, ChunkPos, VoxelType, CHUNK_SIZE};
use crate::rendering::texture_atlas::{TextureAtlas, FaceDirection};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
}

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

pub struct ChunkMesh {
    pub vertices: Vec<Vertex>,
}

impl ChunkMesh {
    pub fn new() -> Self {
        Self { vertices: Vec::new(), }
    }

    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
    }

    pub fn vertex_count(&self) -> u32 {
        self.vertices.len() as u32
    }
}

impl FaceDirection {
    pub fn normal(&self) -> [f32; 3] {
        match self {
            FaceDirection::North => [0.0, 0.0, 1.0],
            FaceDirection::South => [0.0, 0.0, -1.0],
            FaceDirection::East => [1.0, 0.0, 0.0],
            FaceDirection::West => [-1.0, 0.0, 0.0],
            FaceDirection::Top => [0.0, 1.0, 0.0],
            FaceDirection::Bottom => [0.0, -1.0, 0.0],
        }
    }

    pub fn vertices(&self, x: f32, y: f32, z: f32) -> [[f32; 3]; 6] {
        match self {
            FaceDirection::North => [
                // Triangle 1
                [x, y, z + 1.0],           // Bottom-left
                [x + 1.0, y, z + 1.0],     // Bottom-right
                [x + 1.0, y + 1.0, z + 1.0], // Top-right
                // Triangle 2
                [x, y, z + 1.0],           // Bottom-left
                [x + 1.0, y + 1.0, z + 1.0], // Top-right
                [x, y + 1.0, z + 1.0],     // Top-left
            ],
            FaceDirection::South => [
                // Triangle 1
                [x + 1.0, y, z],
                [x, y, z],
                [x, y + 1.0, z],
                // Triangle 2
                [x + 1.0, y, z],
                [x, y + 1.0, z],
                [x + 1.0, y + 1.0, z],
            ],
            FaceDirection::East => [
                // Triangle 1
                [x + 1.0, y, z + 1.0],
                [x + 1.0, y, z],
                [x + 1.0, y + 1.0, z],
                // Triangle 2
                [x + 1.0, y, z + 1.0],
                [x + 1.0, y + 1.0, z],
                [x + 1.0, y + 1.0, z + 1.0],
            ],
            FaceDirection::West => [
                // Triangle 1
                [x, y, z],
                [x, y, z + 1.0],
                [x, y + 1.0, z + 1.0],
                // Triangle 2
                [x, y, z],
                [x, y + 1.0, z + 1.0],
                [x, y + 1.0, z],
            ],
            FaceDirection::Top => [
                // Triangle 1
                [x, y + 1.0, z + 1.0],
                [x + 1.0, y + 1.0, z + 1.0],
                [x + 1.0, y + 1.0, z],
                // Triangle 2
                [x, y + 1.0, z + 1.0],
                [x + 1.0, y + 1.0, z],
                [x, y + 1.0, z],
            ],
            FaceDirection::Bottom => [
                // Triangle 1
                [x, y, z],
                [x + 1.0, y, z],
                [x + 1.0, y, z + 1.0],
                // Triangle 2
                [x, y, z],
                [x + 1.0, y, z + 1.0],
                [x, y, z + 1.0],
            ],
        }
    }
}

pub struct ChunkMesher {
    texture_atlas: TextureAtlas,
}

impl ChunkMesher {
    pub fn new() -> Self {
        Self {
            texture_atlas: TextureAtlas::new(256, 16),
        }
    }
    pub fn generate_mesh(&self, chunk: &Chunk, chunk_pos: ChunkPos) -> ChunkMesh {
        let mut mesh = ChunkMesh::new();

        let offset_x = (chunk_pos.x * CHUNK_SIZE as i32) as f32;
        let offset_y = (chunk_pos.y * CHUNK_SIZE as i32) as f32;
        let offset_z = (chunk_pos.z * CHUNK_SIZE as i32) as f32;

        for z in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    if let Some(voxel) = chunk.get_voxel(x, y, z) {
                        if let VoxelType::Air = voxel {
                            continue;
                        }

                        self.add_voxel_faces(
                            &mut mesh,
                            chunk,
                            x,
                            y,
                            z,
                            voxel,
                            offset_x,
                            offset_y,
                            offset_z,
                        );
                    }
                }
            }
        }

        mesh
    }

    fn add_voxel_faces(
        &self,
        mesh: &mut ChunkMesh,
        chunk: &Chunk,
        x: usize,
        y: usize,
        z: usize,
        voxel: VoxelType,
        offset_x: f32,
        offset_y: f32,
        offset_z: f32,
    ) {
        let faces = [
            (FaceDirection::North, (x, y, z + 1)),
            (FaceDirection::South, (x, y, z.wrapping_sub(1))),
            (FaceDirection::East, (x + 1, y, z)),
            (FaceDirection::West, (x.wrapping_sub(1), y, z)),
            (FaceDirection::Top, (x, y + 1, z)),
            (FaceDirection::Bottom, (x, y.wrapping_sub(1), z)),
        ];

        for (direction, neighbor_pos) in faces {
            if Self::should_render_face(chunk, neighbor_pos) {
                self.add_face(
                    mesh,
                    x as f32 + offset_x,
                    y as f32 + offset_y,
                    z as f32 + offset_z,
                    direction,
                    voxel,
                );
            }
        }
    }

    fn should_render_face(chunk: &Chunk, neighbor_pos: (usize, usize, usize)) -> bool {
        let (x, y, z) = neighbor_pos;

        if x >= CHUNK_SIZE || y >= CHUNK_SIZE || z >= CHUNK_SIZE {
            return true;
        }

        match chunk.get_voxel(x, y, z) {
            Some(VoxelType::Air) => true,
            None => true,
            _ => false,
        }
    }

    fn add_face(
        &self,
        mesh: &mut ChunkMesh,
        x: f32,
        y: f32,
        z: f32,
        direction: FaceDirection,
        voxel: VoxelType,
    ) {
        let positions = direction.vertices(x, y, z);
        let uvs = self.texture_atlas.get_uvs(voxel, direction);
        let normal = direction.normal();

        for i in 0..6 {
            mesh.vertices.push(Vertex {
                position: positions[i],
                tex_coords: uvs[i],
                normal,
            });
        }
    }
}

pub struct ChunkMeshBuffer {
    pub vertex_buffer: wgpu::Buffer,
    pub vertex_count: u32,
}

impl ChunkMeshBuffer {
    pub fn from_mesh(device: &wgpu::Device, mesh: &ChunkMesh) -> Option<Self> {
        if mesh.is_empty() {
            return None;
        }

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Chunk Vertex Buffer"),
            contents: bytemuck::cast_slice(&mesh.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Some(Self {
            vertex_buffer,
            vertex_count: mesh.vertex_count(),
        })
    }

    pub fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..self.vertex_count, 0..1);
    }
}