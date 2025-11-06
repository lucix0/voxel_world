use crate::rendering::texture::Texture;

pub struct SharedResources {
    pub voxel_texture: Texture,
    pub voxel_bind_group: wgpu::BindGroup,
}

impl SharedResources {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let diffuse_bytes = include_bytes!("../../resources/textures/voxel_textures.png");
        let voxel_texture =
            Texture::from_bytes(&device, &queue, diffuse_bytes, "happy-tree.png").unwrap();

        let voxel_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&voxel_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&voxel_texture.sampler),
                    }
                ],
                label: Some("diffuse_bind_group"),
            }
        );

        Self {
            voxel_texture,
            voxel_bind_group
        }
    }
}