pub mod app;
mod game;
mod input;
mod rendering;

use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::{event_loop::ActiveEventLoop, event::DeviceEvent, keyboard::KeyCode, window::Window};
use winit::window::CursorGrabMode;
use game::camera::Camera;
use game::chunk::ChunkPos;
use game::world::World;

use input::player_controller::PlayerController;

use rendering::texture;
use rendering::chunk_renderer::ChunkRenderer;
use rendering::projection::Projection;
use crate::game::chunk::VoxelType;
use crate::game::player::Player;
use crate::game::{raycast_voxel, RaycastHit};

pub struct State {
    // GPU Resources
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
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

    // Rendering state
    projection: Projection,
    chunk_renderer: ChunkRenderer,

    // Render pipeline and resources
    render_pipeline: wgpu::RenderPipeline,
    diffuse_bind_group: wgpu::BindGroup,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    depth_texture: texture::Texture,

    // Timing
    last_render_time: std::time::Instant,
    is_surface_configured: bool,
}

impl State {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        /*
            GPU Setup
        */
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
           backends: wgpu::Backends::PRIMARY,
           ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }).await?;

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            label: None,
            required_features: wgpu::Features::empty(),
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
            required_limits: wgpu::Limits::default(),
            memory_hints: Default::default(),
            trace: wgpu::Trace::Off,
        }).await?;

        let surface_caps = surface.get_capabilities(&adapter);

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

        /*
            Load Resources
        */
        let diffuse_bytes = include_bytes!("../resources/textures/voxel_textures.png");
        let diffuse_texture = texture::Texture::from_bytes(&device, &queue, diffuse_bytes, "happy-tree.png").unwrap();

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let diffuse_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&diffuse_texture.view), // CHANGED!
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler), // CHANGED!
                    }
                ],
                label: Some("diffuse_bind_group"),
            }
        );

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
        let camera_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[projection.get_view_projection_matrix(&camera)]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }
            ],
            label: Some("camera_bind_group"),
        });

        /*
            Create Render Pipeline
        */
        let depth_texture = texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../resources/shaders/shader.wgsl").into()),
        });


        let render_pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &texture_bind_group_layout,
                    &camera_bind_group_layout,
                ],
                push_constant_ranges: &[],
            }
        );

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[
                    rendering::mesh::Vertex::desc(),
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        let chunk_renderer = ChunkRenderer::new();

        Self::set_cursor_grabbed(&window, true);

        Ok(Self {
            surface,
            device,
            queue,
            config,
            is_surface_configured: false,
            window,
            render_pipeline,
            diffuse_bind_group,
            camera,
            player_controller,
            projection,
            camera_buffer,
            camera_bind_group,
            depth_texture,
            world,
            player,
            cursor_grabbed: true,
            selected_block: None,
            held_block_type: VoxelType::Stone,
            last_render_time: std::time::Instant::now(),
            mouse_pressed: false,
            chunk_renderer,
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
            self.surface.configure(&self.device, &self.config);
            self.is_surface_configured = true;
        }

        self.depth_texture = texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
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
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.projection.get_view_projection_matrix(&self.camera)]));

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
        self.chunk_renderer.update(&mut self.world, &self.device);
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.window.request_redraw();

        if !self.is_surface_configured {
            return Ok(());
        }

        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
        render_pass.set_bind_group(1, &self.camera_bind_group, &[]);

        self.chunk_renderer.render(&mut render_pass);

        drop(render_pass);

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}