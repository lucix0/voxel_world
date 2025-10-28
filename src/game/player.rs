use cgmath::Zero;
use crate::game::chunk::VoxelType;
use crate::game::world::World;

enum Axis {
    X,
    Y,
    Z,
}

pub struct Player {
    pub position: cgmath::Point3<f32>,
    pub velocity: cgmath::Vector3<f32>,
    pub width: f32,
    pub height: f32,
    pub is_on_ground: bool,
}

impl Player {
    pub fn new(position: cgmath::Point3<f32>) -> Self {
        Self {
            position,
            velocity: cgmath::Vector3::zero(),
            width: 0.5,
            height: 1.8,
            is_on_ground: false,
        }
    }

    fn resolve_collisions(&mut self, world: &World, axis: Axis) {
        let min_x_voxel = (self.position.x - (self.width / 2.0)).floor() as i32;
        let max_x_voxel = (self.position.x + (self.width / 2.0)).floor() as i32;
        let min_y_voxel = (self.position.y - (self.height / 2.0)).floor() as i32;
        let max_y_voxel = (self.position.y + (self.height / 2.0)).floor() as i32;
        let min_z_voxel = (self.position.z - (self.width / 2.0)).floor() as i32;
        let max_z_voxel = (self.position.z + (self.width / 2.0)).floor() as i32;

        const EPSILON: f32 = 0.001;

        for z in min_z_voxel..=max_z_voxel {
            for y in min_y_voxel..=max_y_voxel {
                for x in min_x_voxel..=max_x_voxel {
                    if matches!(world.get_voxel(x, y, z), Some(VoxelType::Air) | None) {
                        continue;
                    }

                    let mut p_min_x = self.position.x - (self.width / 2.0);
                    let mut p_max_x = self.position.x + (self.width / 2.0);
                    let mut p_min_y = self.position.y - (self.height / 2.0);
                    let mut p_max_y = self.position.y + (self.height / 2.0);
                    let mut p_min_z = self.position.z - (self.width / 2.0);
                    let mut p_max_z = self.position.z + (self.width / 2.0);

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
                        let pen_x = (p_max_x - v_min_x).min(v_max_x - p_min_x);
                        let pen_y = (p_max_y - v_min_y).min(v_max_y - p_min_y);
                        let pen_z = (p_max_z - v_min_z).min(v_max_z - p_min_z);

                        match axis {
                            Axis::X => {
                                if pen_x <= pen_y && pen_x <= pen_z {
                                    let pen_from_right = p_max_x - v_min_x; // Overlap on block's left face
                                    let pen_from_left = v_max_x - p_min_x;  // Overlap on block's right face

                                    // Push back from the side with the smallest overlap
                                    if pen_from_right < pen_from_left {
                                        self.position.x -= pen_from_right + EPSILON; // Push left
                                    } else {
                                        self.position.x += pen_from_left + EPSILON;  // Push right
                                    }

                                    self.velocity.x = 0.0;

                                    // Recalculate AABB for next check in loop
                                    p_min_x = self.position.x - (self.width / 2.0);
                                    p_max_x = self.position.x + (self.width / 2.0);
                                }
                            }
                            Axis::Y => {
                                if pen_y <= pen_x && pen_y <= pen_z {
                                    let pen_from_top = p_max_y - v_min_y;
                                    let pen_from_bottom = v_max_y - p_min_y;

                                    if pen_from_top < pen_from_bottom {
                                        self.position.y -= pen_from_top + EPSILON; // Push down
                                    } else {
                                        self.position.y += pen_from_bottom + EPSILON; // Push up
                                        self.is_on_ground = true;
                                    }

                                    self.velocity.y = 0.0;

                                    p_min_y = self.position.y - (self.height / 2.0);
                                    p_max_y = self.position.y + (self.height / 2.0);
                                }
                            }
                            Axis::Z => {
                                if pen_z <= pen_x && pen_z <= pen_y {
                                    let pen_from_front = p_max_z - v_min_z;
                                    let pen_from_back = v_max_z - p_min_z;

                                    if pen_from_front < pen_from_back {
                                        self.position.z -= pen_from_front + EPSILON; // Push "back"
                                    } else {
                                        self.position.z += pen_from_back + EPSILON;  // Push "forward"
                                    }

                                    self.velocity.z = 0.0;

                                    p_min_z = self.position.z - (self.width / 2.0);
                                    p_max_z = self.position.z + (self.width / 2.0);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn update(&mut self, world: &mut World, dt: f32) {
        const GRAVITY: f32 = -9.81;
        self.is_on_ground = false;
        self.velocity.y += GRAVITY * dt;

        let desired_movement = self.velocity * dt;

        self.position.x += desired_movement.x;
        self.resolve_collisions(world, Axis::X);
        self.position.y += desired_movement.y;
        self.resolve_collisions(world, Axis::Y);
        self.position.z += desired_movement.z;
        self.resolve_collisions(world, Axis::Z);
    }
}