use std::collections::HashMap;
use crate::game::{world::World, chunk::ChunkPos};
use crate::rendering::mesh::{ChunkMeshBuffer, ChunkMesher};

pub struct ChunkRenderer {
    mesher: ChunkMesher,
    buffers: HashMap<ChunkPos, ChunkMeshBuffer>
}

impl ChunkRenderer {
    pub fn new() -> Self {
        Self {
            mesher: ChunkMesher::new(),
            buffers: HashMap::new(),
        }
    }

    pub fn update(&mut self, world: &mut World, device: &wgpu::Device) {
        let dirty_chunks = world.take_dirty_chunks().collect::<Vec<_>>();

        for pos in dirty_chunks {
            self.remesh_chunk(world, pos, device);
        }
    }

    fn remesh_chunk(&mut self, world: &World, pos: ChunkPos, device: &wgpu::Device) {
        if let Some(chunk) = world.get_chunk(pos) {
            let mesh = self.mesher.generate_mesh(chunk, pos);

            if !mesh.is_empty() {
                if let Some(buffer) = ChunkMeshBuffer::from_mesh(device, &mesh) {
                    self.buffers.insert(pos, buffer);
                }
            } else {
                self.buffers.remove(&pos);
            }
        }
    }

    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        for buffers in self.buffers.values() {
            buffers.draw(render_pass);
        }
    }
}