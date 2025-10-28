use std::collections::{HashMap, HashSet};
use crate::game::chunk::{Chunk, ChunkPos, VoxelType, CHUNK_SIZE};

pub struct World {
    chunks: HashMap<ChunkPos, Chunk>,
    dirty_chunks: HashSet<ChunkPos>,
}

impl World {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
            dirty_chunks: HashSet::new(),
        }
    }

    pub fn get_chunk(&self, pos: ChunkPos) -> Option<&Chunk> {
        self.chunks.get(&pos)
    }

    pub fn get_chunk_mut(&mut self, pos: ChunkPos) -> Option<&mut Chunk> {
        self.chunks.get_mut(&pos)
    }

    pub fn load_chunk(&mut self, pos: ChunkPos) {
        if !self.chunks.contains_key(&pos) {
            let chunk = self.generate_chunk(pos);
            self.chunks.insert(pos, chunk);
            self.dirty_chunks.insert(pos);
        }
    }

    fn unload_chunk(&mut self, pos: ChunkPos) {
        self.chunks.remove(&pos);
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

    pub fn set_voxel(&mut self, wx: i32, wy: i32, wz: i32, voxel: VoxelType) {
        let chunk_pos = ChunkPos::new(
            wx.div_euclid(CHUNK_SIZE as i32),
            wy.div_euclid(CHUNK_SIZE as i32),
            wz.div_euclid(CHUNK_SIZE as i32),
        );

        let local_x = wx.rem_euclid(CHUNK_SIZE as i32) as usize;
        let local_y = wy.rem_euclid(CHUNK_SIZE as i32) as usize;
        let local_z = wz.rem_euclid(CHUNK_SIZE as i32) as usize;

        self.load_chunk(chunk_pos);

        if let Some(chunk) = self.chunks.get_mut(&chunk_pos) {
            chunk.set_voxel(local_x, local_y, local_z, voxel);
            self.dirty_chunks.insert(chunk_pos);
        }
    }

    pub fn take_dirty_chunks(&mut self) -> impl Iterator<Item = ChunkPos> + '_ {
        self.dirty_chunks.drain()
    }
}