pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

pub fn get_chunk_index(x: usize, y: usize, z: usize) -> usize {
    x + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE
}

#[derive(Copy, Clone, Eq, Hash, PartialEq)]
pub struct ChunkPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl ChunkPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    // Convert world coordinates to the chunk position they reside in
    pub fn from_world_pos(wx: f32, wy: f32, wz: f32) -> Self {
        Self {
            x: (wx / CHUNK_SIZE as f32).floor() as i32,
            y: (wy / CHUNK_SIZE as f32).floor() as i32,
            z: (wz / CHUNK_SIZE as f32).floor() as i32,
        }
    }
}

#[derive(Copy, Clone)]
pub enum VoxelType {
    Air,
    Grass,
    Dirt,
    Stone,
}

pub struct Chunk {
    voxels: [VoxelType; CHUNK_VOLUME],
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            voxels: [VoxelType::Air; CHUNK_VOLUME],
        }
    }

    pub fn set_voxel(&mut self, x: usize, y: usize, z: usize, voxel_type: VoxelType) {
        self.voxels[get_chunk_index(x, y, z)] = voxel_type;
    }

    pub fn get_voxel(&self, x: usize, y: usize, z: usize) -> Option<VoxelType> {
        Some(self.voxels[get_chunk_index(x, y, z)])
    }
}
