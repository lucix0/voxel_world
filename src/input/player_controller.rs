use cgmath::{InnerSpace, Zero};
use winit::keyboard::KeyCode;
use crate::game::camera::Camera;
use crate::game::player::Player;

const JUMP_STRENGTH: f32 = 7.0;

pub struct PlayerController {
    // Keyboard input.
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_up_pressed: bool,
    is_down_pressed: bool,

    // Mouse input.
    mouse_sensitivity: f32,
    pub mouse_delta: (f32, f32),
    is_mouse_captured: bool,
}

impl PlayerController {
    pub fn new(mouse_sensitivity: f32) -> Self {
        Self {
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
    }

    pub fn update_velocity(&self, player: &mut Player, camera: &mut Camera, dt: f32) {
        let move_speed = 10.0;

        let mut move_direction = cgmath::Vector3::zero();

        if self.is_forward_pressed {
            move_direction += camera.get_forward_horizontal();
        }
        if self.is_backward_pressed {
            move_direction -= camera.get_forward_horizontal();
        }
        if self.is_right_pressed {
            move_direction += camera.get_right();
        }
        if self.is_left_pressed {
            move_direction -= camera.get_right();
        }

        let horizontal_velocity = if !move_direction.is_zero() {
            move_direction.normalize() * move_speed
        } else {
            cgmath::Vector3::zero()
        };

        let mut vertical_velocity = player.velocity.y;
        if self.is_up_pressed && player.is_on_ground {
            vertical_velocity = JUMP_STRENGTH;
        }

        player.velocity.x = horizontal_velocity.x;
        player.velocity.y = vertical_velocity;
        player.velocity.z = horizontal_velocity.z;
    }
}
