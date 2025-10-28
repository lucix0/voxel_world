use crate::game::chunk::VoxelType;

pub struct TextureAtlas {
    tile_size: f32,
}

impl TextureAtlas {
    pub fn new(atlas_size: u32, tile_count: u32) -> Self {
        Self {
            tile_size: 1.0 / tile_count as f32,
        }
    }

    pub fn get_uvs(&self, voxel: VoxelType, face: FaceDirection) -> [[f32; 2]; 6] {
        let (u, v) = self.get_tile_coords(voxel, face);

        let u_min = u * self.tile_size;
        let u_max = (u + 1.0) * self.tile_size;
        let v_min = v * self.tile_size;
        let v_max = (v + 1.0) * self.tile_size;

        // 6 vertices (2 triangles) for the quad
        [
            [u_min, v_max], // Bottom-left
            [u_max, v_max], // Bottom-right
            [u_max, v_min], // Top-right
            [u_min, v_max], // Bottom-left
            [u_max, v_min], // Top-right
            [u_min, v_min], // Top-left
        ]
    }

    // Get tile position in the atlas (in tiles, not UV coords)
    fn get_tile_coords(&self, voxel: VoxelType, face: FaceDirection) -> (f32, f32) {
        match voxel {
            VoxelType::Air => (0.0, 0.0), // Shouldn't be rendered
            VoxelType::Stone => (2.0, 0.0),
            VoxelType::Dirt => (1.0, 0.0),
            VoxelType::Grass => {
                match face {
                    FaceDirection::Top => (0.0, 1.0),    // Grass top
                    FaceDirection::Bottom => (1.0, 0.0), // Dirt bottom
                    _ => (0.0, 0.0), // Grass side
                }
            },
        }
    }
}

#[derive(Copy, Clone)]
pub enum FaceDirection {
    North,
    South,
    East,
    West,
    Top,
    Bottom,
}