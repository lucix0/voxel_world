pub mod app;
mod game;
mod input;
mod rendering;
mod ui;

use std::sync::Arc;
use egui_wgpu::ScreenDescriptor;
use wgpu::util::DeviceExt;
use winit::{event_loop::ActiveEventLoop, event::DeviceEvent, keyboard::KeyCode, window::Window};
use winit::window::CursorGrabMode;
use game::camera::Camera;
use game::chunk::ChunkPos;
use game::world::World;

use input::player_controller::PlayerController;

use rendering::projection::Projection;
use crate::game::chunk::VoxelType;
use crate::game::player::Player;
use crate::game::{raycast_voxel, RaycastHit};
use crate::rendering::geometry_renderer::GeometryRenderer;
use crate::rendering::gpu_context::GpuContext;
use crate::rendering::SharedResources;
use crate::ui::debug_ui::DebugUi;
use crate::ui::panels;

pub struct State {
    // GPU Resources
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    gpu_context: GpuContext,
    config: wgpu::SurfaceConfiguration,

    // Game State
    world: World,
    player: Player,
    camera: Camera,
    selected_block: Option<RaycastHit>,
    held_block_type: VoxelType,

    // Input state
    player_controller: PlayerController,
    cursor_grabbed: bool,
    mouse_pressed: bool,

    // UI state
    debug_ui: DebugUi,

    // Rendering state
    projection: Projection,
    geometry_renderer: GeometryRenderer,

    // Render pipeline and resources
    shared_resources: SharedResources,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    // Timing
    last_render_time: std::time::Instant,
    is_surface_configured: bool,
}

impl State {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let (gpu_context, surface) = GpuContext::new(window.clone()).await?;

        let size = window.inner_size();

        let surface_caps = surface.get_capabilities(&gpu_context.adapter);

        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let texture_bind_group_layout =
            gpu_context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let shared_resources = SharedResources::new(&gpu_context.device, &gpu_context.queue, &texture_bind_group_layout);

        /*
            Setup Game State
        */
        let camera = Camera::new(
            (0.0, 16.0, 32.0).into(),
            -std::f32::consts::FRAC_PI_2,
            0.0,
        );

        let projection = Projection::new(config.width, config.height);
        let player_controller = PlayerController::new(0.003);

        let mut world = World::new();
        world.load_chunk(ChunkPos::new(0, 1, 0));
        world.load_chunk(ChunkPos::new(0, 0, 0));
        world.load_chunk(ChunkPos::new(0, -1, 0));

        let player = Player::new((0.0, 32.0, 16.0).into());

        /*
            Setup Camera Uniform
        */
        let camera_buffer = gpu_context.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[projection.get_view_projection_matrix(&camera)]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let camera_bind_group_layout = gpu_context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("camera_bind_group_layout"),
        });

        let camera_bind_group = gpu_context.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }
            ],
            label: Some("camera_bind_group"),
        });

        let geometry_renderer = GeometryRenderer::new(
            &gpu_context.device,
            &config,
            &texture_bind_group_layout,
            &camera_bind_group_layout,
        );

        let debug_ui = DebugUi::new(
            &gpu_context.device,
            surface_format,
            None,
            1,
            &window,
        );

        Self::set_cursor_grabbed(&window, true);

        Ok(Self {
            surface,
            config,
            is_surface_configured: false,
            window,
            gpu_context,
            shared_resources,
            camera,
            player_controller,
            projection,
            camera_buffer,
            camera_bind_group,
            world,
            player,
            debug_ui,
            cursor_grabbed: true,
            selected_block: None,
            held_block_type: VoxelType::Stone,
            last_render_time: std::time::Instant::now(),
            mouse_pressed: false,
            geometry_renderer,
        })
    }

    fn set_cursor_grabbed(window: &Window, grabbed: bool) {
        if grabbed {
            // Hide cursor
            window.set_cursor_visible(false);

            // Capture/lock cursor
            window.set_cursor_grab(CursorGrabMode::Confined)
                .or_else(|_| window.set_cursor_grab(CursorGrabMode::Locked))
                .unwrap_or_else(|e| log::warn!("Failed to grab cursor: {}", e));
        } else {
            // Show cursor
            window.set_cursor_visible(true);

            // Release cursor
            window.set_cursor_grab(CursorGrabMode::None)
                .unwrap_or_else(|e| log::warn!("Failed to release cursor: {}", e));
        }
    }

    /*
        Window Events
    */
    pub fn resize(&mut self, _width: u32, _height: u32) {
        if _width > 0 && _height > 0 {
            self.config.width = _width;
            self.config.height = _height;
            self.surface.configure(&self.gpu_context.device, &self.config);
            self.is_surface_configured = true;
        }

        self.geometry_renderer.recreate_depth_texture(&self.gpu_context.device, &self.config);
    }

    fn handle_key(&mut self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        if code == KeyCode::Escape && is_pressed {
            self.cursor_grabbed = !self.cursor_grabbed;
            Self::set_cursor_grabbed(&self.window, self.cursor_grabbed);
        } else {
            self.player_controller.handle_key(code, is_pressed);
        }
    }

    pub fn device_input(&mut self, event: &DeviceEvent) {
        match event {
            DeviceEvent::MouseMotion { delta } => {
                self.player_controller.handle_mouse(delta.0, delta.1, &mut self.camera);
            }
            _ => {}
        }
    }

    fn break_block(&mut self) {
        if let Some(hit) = &self.selected_block {
            let (x, y, z) = hit.position;
            self.world.set_voxel(x, y, z, VoxelType::Air);
        }
    }

    fn place_block(&mut self) {
        if let Some(hit) = &self.selected_block {
            let (x, y, z) = hit.position;
            let (nx, ny, nz) = hit.normal;

            let place_x = x + nx;
            let place_y = y + ny;
            let place_z = z + nz;

            if !self.is_position_inside_player(place_x, place_y, place_z) {
                self.world.set_voxel(place_x, place_y, place_z, self.held_block_type);
            }
        }
    }

    fn is_position_inside_player(&self, x: i32, y: i32, z: i32) -> bool {
        // Check if block would intersect with player's collision box
        let block_min_x = x as f32;
        let block_max_x = (x + 1) as f32;
        let block_min_y = y as f32;
        let block_max_y = (y + 1) as f32;
        let block_min_z = z as f32;
        let block_max_z = (z + 1) as f32;

        let player_min_x = self.player.position.x - self.player.width / 2.0;
        let player_max_x = self.player.position.x + self.player.width / 2.0;
        let player_min_y = self.player.position.y - self.player.height / 2.0;
        let player_max_y = self.player.position.y + self.player.height / 2.0;
        let player_min_z = self.player.position.z - self.player.width / 2.0;
        let player_max_z = self.player.position.z + self.player.width / 2.0;

        // AABB intersection test
        block_min_x < player_max_x
            && block_max_x > player_min_x
            && block_min_y < player_max_y
            && block_max_y > player_min_y
            && block_min_z < player_max_z
            && block_max_z > player_min_z
    }

    /*
        Game Loop
    */
    fn update(&mut self) {
        // Calculate delta time
        let now = std::time::Instant::now();
        let mut dt = now.duration_since(self.last_render_time).as_secs_f32();
        self.last_render_time = now;

        dt = dt.min(0.1);

        // Update camera
        self.player_controller.update_velocity(&mut self.player, &mut self.camera, dt);
        self.gpu_context.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.projection.get_view_projection_matrix(&self.camera)]));

        self.player.update(&mut self.world, dt);
        self.camera.position = self.player.position + cgmath::vec3(0.0, 0.8, 0.0);

        // Raycast to find selected block
        let ray_origin = self.camera.position;
        let ray_direction = self.camera.get_direction();
        self.selected_block = raycast_voxel(
            &self.world,
            ray_origin,
            ray_direction,
            5.0,
        );

        // Remesh chunks if necessary
        self.geometry_renderer.update_chunk_renderer(&mut self.world, &self.gpu_context.device);
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.window.request_redraw();

        if !self.is_surface_configured {
            return Ok(());
        }

        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.gpu_context.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        self.geometry_renderer.render(&view, &mut encoder, &self.shared_resources.voxel_bind_group, &self.camera_bind_group);

        // UI rendering
        let surface_view = output
            .texture.create_view(&wgpu::TextureViewDescriptor::default());

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [self.config.width, self.config.height],
            pixels_per_point: self.window.as_ref().scale_factor() as f32,
        };

        self.debug_ui.begin_frame(&self.window);

        egui::Window::new("Debug Panel 1")
            .frame(egui::Frame {
                shadow: egui::epaint::Shadow::NONE,
                fill: egui::Color32::from_black_alpha(200),
                corner_radius: egui::CornerRadius::same(0),
                ..Default::default()
            })
            .title_bar(false)
            .resizable(false)
            .fixed_pos(egui::pos2(0.0, 0.0))
            .vscroll(false)
            .default_open(true)
            .collapsible(false)
            .movable(false)
            .show(self.debug_ui.context(), |ui| {
                panels::PlayerInfoPanel::show(
                    ui,
                    self.player.position,
                    self.player.velocity
                );
            });

        self.debug_ui.end_frame_and_draw(
            &self.gpu_context.device,
            &self.gpu_context.queue,
            &mut encoder,
            &self.window,
            &surface_view,
            screen_descriptor,
        );


        self.gpu_context.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}