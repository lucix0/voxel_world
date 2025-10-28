use std::collections::HashMap;
use crate::chunk::{Chunk, ChunkPos, VoxelType, CHUNK_SIZE};
use crate::mesh::{ChunkMeshBuffer, ChunkMesher};

pub struct World {
    chunks: HashMap<ChunkPos, Chunk>,
    pub chunk_buffers: HashMap<ChunkPos, ChunkMeshBuffer>,
}

impl World {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
            chunk_buffers: HashMap::new(),
        }
    }

    pub fn get_chunk(&self, pos: ChunkPos) -> Option<&Chunk> {
        self.chunks.get(&pos)
    }

    pub fn get_chunk_mut(&mut self, pos: ChunkPos) -> Option<&mut Chunk> {
        self.chunks.get_mut(&pos)
    }

    pub fn load_chunk(&mut self, device: &wgpu::Device, pos: ChunkPos) {
        // Later on, chunk will either be freshly generated or loaded from disk.
        // For now, just generate it.
        if !self.chunks.contains_key(&pos) {
            let chunk = self.generate_chunk(pos);
            let chunk_mesh = ChunkMesher::generate_mesh(&chunk, pos);

            // Don't add a buffer if the chunk is all air.
            if let Some(chunk_buffer) = ChunkMeshBuffer::from_mesh(device, &chunk_mesh) {
                self.chunk_buffers.insert(pos, chunk_buffer);
            }

            self.chunks.insert(pos, chunk);
        }
    }

    fn unload_chunk(&mut self, pos: ChunkPos) {
        self.chunks.remove(&pos);
        self.chunk_buffers.remove(&pos);
    }

    fn generate_chunk(&mut self, pos: ChunkPos) -> Chunk {
        let mut chunk = Chunk::new();

        for z in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let wx = pos.x * CHUNK_SIZE as i32 + x as i32;
                    let wy = pos.y * CHUNK_SIZE as i32 + y as i32;
                    let wz = pos.z * CHUNK_SIZE as i32 + z as i32;

                    let voxel =
                        if wy < -3 {
                            VoxelType::Stone
                        } else if wy < 0 {
                            VoxelType::Dirt
                        } else if wy == 0 {
                            VoxelType::Grass
                        } else {
                            VoxelType::Air
                        };

                    chunk.set_voxel(x, y, z, voxel);
                }
            }
        }

        chunk
    }

    pub fn get_voxel(&self, wx: i32, wy: i32, wz: i32) -> Option<VoxelType> {
        let chunk_pos = ChunkPos::new(
            wx.div_euclid(CHUNK_SIZE as i32),
            wy.div_euclid(CHUNK_SIZE as i32),
            wz.div_euclid(CHUNK_SIZE as i32),
        );

        let local_x = wx.rem_euclid(CHUNK_SIZE as i32) as usize;
        let local_y = wy.rem_euclid(CHUNK_SIZE as i32) as usize;
        let local_z = wz.rem_euclid(CHUNK_SIZE as i32) as usize;

        self.chunks.get(&chunk_pos)?.get_voxel(local_x, local_y, local_z)
    }

    pub fn set_voxel(&mut self, device: &wgpu::Device, wx: i32, wy: i32, wz: i32, voxel: VoxelType) {
        let chunk_pos = ChunkPos::new(
            wx.div_euclid(CHUNK_SIZE as i32),
            wy.div_euclid(CHUNK_SIZE as i32),
            wz.div_euclid(CHUNK_SIZE as i32),
        );

        let local_x = wx.rem_euclid(CHUNK_SIZE as i32) as usize;
        let local_y = wy.rem_euclid(CHUNK_SIZE as i32) as usize;
        let local_z = wz.rem_euclid(CHUNK_SIZE as i32) as usize;

        self.load_chunk(device, chunk_pos);

        if let Some(chunk) = self.chunks.get_mut(&chunk_pos) {
            chunk.set_voxel(local_x, local_y, local_z, voxel);
        }
    }
}