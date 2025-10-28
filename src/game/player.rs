use crate::game::chunk::VoxelType;
use crate::game::world::World;

pub struct Player {
    pub position: cgmath::Point3<f32>,
    width: f32,
    height: f32,
}

impl Player {
    pub fn new(position: cgmath::Point3<f32>) -> Self {
        Self {
            position,
            width: 0.5,
            height: 1.8,
        }
    }

    pub fn update(&mut self, world: &mut World) {
        // First, calculate what voxels the player may be inside.
        let p_min_x = self.position.x - (self.width / 2.0);
        let p_max_x = self.position.x + (self.width / 2.0);
        let p_min_y = self.position.y - (self.height / 2.0);
        let p_max_y = self.position.y + (self.height / 2.0);
        let p_min_z = self.position.z - (self.width / 2.0);
        let p_max_z = self.position.z + (self.width / 2.0);

        // The range of voxels the player may be in.
        let min_x_voxel = (self.position.x - (self.width / 2.0)).floor() as i32;
        let max_x_voxel = (self.position.x + (self.width / 2.0)).floor() as i32;
        let min_y_voxel = (self.position.y - (self.height / 2.0)).floor() as i32;
        let max_y_voxel = (self.position.y + (self.height / 2.0)).floor() as i32;
        let min_z_voxel = (self.position.z - (self.width / 2.0)).floor() as i32;
        let max_z_voxel = (self.position.z + (self.width / 2.0)).floor() as i32;

        let mut collided_block_pos: Option<(i32, i32, i32)> = None;

        'collision_check: for z in min_z_voxel..=max_z_voxel {
            for y in min_y_voxel..=max_y_voxel {
                for x in min_x_voxel..=max_x_voxel {
                    if matches!(world.get_voxel(x, y, z), Some(VoxelType::Air) | None) {
                        continue;
                    }

                    let v_min_x = x as f32;
                    let v_max_x = (x + 1) as f32;
                    let v_min_y = y as f32;
                    let v_max_y = (y + 1) as f32;
                    let v_min_z = z as f32;
                    let v_max_z = (z + 1) as f32;

                    let x_overlap = p_min_x < v_max_x && p_max_x > v_min_x;
                    let y_overlap = p_min_y < v_max_y && p_max_y > v_min_y;
                    let z_overlap = p_min_z < v_max_z && p_max_z > v_min_z;

                    if x_overlap && y_overlap && z_overlap {
                        collided_block_pos = Some((x, y, z));
                        break 'collision_check; // Exit all 3 loops
                    }
                }
            }
        }

        if let Some((collided_x, collided_y, collided_z)) = collided_block_pos {
            self.position.x = collided_x as f32 + 2.0;
            self.position.y = collided_y as f32 + 2.0;
            self.position.z = collided_z as f32 + 2.0;

        }
    }
}