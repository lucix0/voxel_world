use cgmath::prelude::*;



pub struct Camera {
    pub position: cgmath::Point3<f32>,
    pub yaw: f32,
    pub pitch: f32,
}

impl Camera {
    pub fn new(position: cgmath::Point3<f32>, yaw: f32, pitch: f32) -> Self {
        Self {
            position,
            yaw,
            pitch,
        }
    }

    pub fn get_view_matrix(&self) -> cgmath::Matrix4<f32> {
        let direction = self.get_direction();
        let target = self.position + direction;
        cgmath::Matrix4::look_at_rh(self.position, target, cgmath::Vector3::unit_y())
    }

    pub fn get_direction(&self) -> cgmath::Vector3<f32> {
        cgmath::Vector3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        ).normalize()
    }

    pub fn get_forward_horizontal(&self) -> cgmath::Vector3<f32> {
        cgmath::Vector3::new(self.yaw.cos(), 0.0, self.yaw.sin()).normalize()
    }

    pub fn get_up(&self) -> cgmath::Vector3<f32> {
        cgmath::Vector3::unit_y()
    }

    pub fn get_right(&self) -> cgmath::Vector3<f32> {
        self.get_forward_horizontal()
            .cross(cgmath::Vector3::unit_y())
            .normalize()
    }
}
