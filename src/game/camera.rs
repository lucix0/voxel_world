use cgmath::InnerSpace;
use winit::keyboard::KeyCode;

#[rustfmt::skip]
const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::from_cols(
    cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0),
);

pub struct Camera {
    pub eye: cgmath::Point3<f32>,
    pub target: cgmath::Point3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,

    pub yaw: f32,
    pub pitch: f32,
}

impl Camera {
    pub fn new(
        position: cgmath::Point3<f32>,
        yaw: f32,
        pitch: f32,
        aspect: f32,
    ) -> Self {
        Self {
            eye: position,
            target: position + cgmath::Vector3::new(yaw.cos(), 0.0, yaw.sin()),
            up: cgmath::Vector3::unit_y(),
            aspect,
            fovy: 45.0,
            znear: 0.1,
            zfar: 1000.0,
            yaw,
            pitch,
        }
    }

    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
        return OPENGL_TO_WGPU_MATRIX * proj * view;
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}

pub struct CameraController {
    speed: f32,

    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_up_pressed: bool,
    is_down_pressed: bool,

    mouse_sensitivity: f32,
    pub mouse_delta: (f32, f32),
    is_mouse_captured: bool,
}

impl CameraController {
    pub fn new(speed: f32, mouse_sensitivity: f32) -> Self {
        Self {
            speed,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_up_pressed: false,
            is_down_pressed: false,
            mouse_sensitivity,
            mouse_delta: (0.0, 0.0),
            is_mouse_captured: true,
        }
    }

    pub fn handle_key(&mut self, code: KeyCode, is_pressed: bool) -> bool {
        match code {
            KeyCode::KeyW | KeyCode::ArrowUp => {
                self.is_forward_pressed = is_pressed;
                true
            }
            KeyCode::KeyA | KeyCode::ArrowLeft => {
                self.is_left_pressed = is_pressed;
                true
            }
            KeyCode::KeyS | KeyCode::ArrowDown => {
                self.is_backward_pressed = is_pressed;
                true
            }
            KeyCode::KeyD | KeyCode::ArrowRight => {
                self.is_right_pressed = is_pressed;
                true
            }
            KeyCode::Space => {
                self.is_up_pressed = is_pressed;
                true
            }
            KeyCode::ShiftLeft => {
                self.is_down_pressed = is_pressed;
                true
            }
            _ => false,
        }
    }

    pub fn handle_mouse(&mut self, delta_x: f64, delta_y: f64, camera: &mut Camera) {
        // Update yaw (horizontal rotation)
        camera.yaw += delta_x as f32 * self.mouse_sensitivity;

        // Update pitch (vertical rotation) with clamping
        camera.pitch -= delta_y as f32 * self.mouse_sensitivity;
        camera.pitch = camera.pitch.clamp(-89.0_f32.to_radians(), 89.0_f32.to_radians());

        // Update target based on new rotation
        let direction = cgmath::Vector3::new(
            camera.yaw.cos() * camera.pitch.cos(),
            camera.pitch.sin(),
            camera.yaw.sin() * camera.pitch.cos(),
        ).normalize();

        camera.target = camera.eye + direction;
    }

    pub fn update_camera(&self, camera: &mut Camera, dt: f32) {
        use cgmath::InnerSpace;

        let direction = cgmath::Vector3::new(
            camera.yaw.cos() * camera.pitch.cos(),
            camera.pitch.sin(),
            camera.yaw.sin() * camera.pitch.cos(),
        ).normalize();

        let forward_horizontal = cgmath::Vector3::new(
            camera.yaw.cos(),
            0.0,
            camera.yaw.sin(),
        ).normalize();

        let right = forward_horizontal.cross(camera.up).normalize();

        let move_speed = self.speed * dt;

        if self.is_forward_pressed {
            camera.eye += forward_horizontal * move_speed;
        }
        if self.is_backward_pressed {
            camera.eye -= forward_horizontal * move_speed;
        }
        if self.is_right_pressed {
            camera.eye += right * move_speed;
        }
        if self.is_left_pressed {
            camera.eye -= right * move_speed;
        }
        if self.is_up_pressed {
            camera.eye += camera.up * move_speed;
        }
        if self.is_down_pressed {
            camera.eye -= camera.up * move_speed;
        }

        camera.target = camera.eye + direction;
    }
}