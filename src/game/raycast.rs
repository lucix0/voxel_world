use cgmath::prelude::*;
use crate::game::chunk::VoxelType;
use crate::game::world::World;

pub struct RaycastHit {
    pub position: (i32, i32, i32),
    pub normal: (i32, i32, i32),
    pub distance: f32,
}

pub fn raycast_voxel(
    world: &World,
    origin: cgmath::Point3<f32>,
    direction: cgmath::Vector3<f32>,
    max_distance: f32,
) -> Option<RaycastHit> {
    let mut current_pos = origin;

    // Step size for each axis
    let step_x = if direction.x > 0.0 { 1 } else { -1 };
    let step_y = if direction.y > 0.0 { 1 } else { -1 };
    let step_z = if direction.z > 0.0 { 1 } else { -1 };

    // Distance to travel along ray to cross one voxel boundary on each axis
    let t_delta_x = if direction.x != 0.0 { (1.0 / direction.x).abs() } else { f32::MAX };
    let t_delta_y = if direction.y != 0.0 { (1.0 / direction.y).abs() } else { f32::MAX };
    let t_delta_z = if direction.z != 0.0 { (1.0 / direction.z).abs() } else { f32::MAX };

    // Current voxel position
    let mut voxel_x = current_pos.x.floor() as i32;
    let mut voxel_y = current_pos.y.floor() as i32;
    let mut voxel_z = current_pos.z.floor() as i32;

    let mut t_max_x = if direction.x > 0.0 {
        ((voxel_x + 1) as f32 - current_pos.x) / direction.x
    } else if direction.x < 0.0 {
        (current_pos.x - voxel_x as f32) / -direction.x
    } else {
        f32::MAX
    };

    let mut t_max_y = if direction.y > 0.0 {
        ((voxel_y + 1) as f32 - current_pos.y) / direction.y
    } else if direction.y < 0.0 {
        (current_pos.y - voxel_y as f32) / -direction.y
    } else {
        f32::MAX
    };

    let mut t_max_z = if direction.z > 0.0 {
        ((voxel_z + 1) as f32 - current_pos.z) / direction.z
    } else if direction.z < 0.0 {
        (current_pos.z - voxel_z as f32) / -direction.z
    } else {
        f32::MAX
    };

    let mut last_normal = (0, 0, 0);
    let mut distance = 0.0;
    
    // Step through voxels
    for _ in 0..100 {
        // Check if current voxel is solid
        if let Some(voxel) = world.get_voxel(voxel_x, voxel_y, voxel_z) {
            if !matches!(voxel, VoxelType::Air) {
                return Some(RaycastHit {
                    position: (voxel_x, voxel_y, voxel_z),
                    normal: last_normal,
                    distance,
                });
            }
        }

        // Step to next voxel boundary
        if t_max_x < t_max_y && t_max_x < t_max_z {
            voxel_x += step_x;
            distance = t_max_x;
            t_max_x += t_delta_x;
            last_normal = (-step_x, 0, 0);
        } else if t_max_y < t_max_z {
            voxel_y += step_y;
            distance = t_max_y;
            t_max_y += t_delta_y;
            last_normal = (0, -step_y, 0);
        } else {
            voxel_z += step_z;
            distance = t_max_z;
            t_max_z += t_delta_z;
            last_normal = (0, 0, -step_z);
        }
        
        if distance > max_distance {
            break;
        }
    }
    
    None
}