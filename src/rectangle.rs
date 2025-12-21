// Rectangle primitive module

use crate::aabb::Aabb;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RectangleData {
    pub top_left: [f32; 2],
    pub size: [f32; 2],
    pub clip: Aabb,
    pub color: [f32; 3],
    pub corner_radius: f32,
}

pub struct Rectangles {
    pub rectangles: Vec<RectangleData>,
    buffer: wgpu::Buffer,
    buffer_capacity: usize,
    pub bind_group: wgpu::BindGroup,
}

impl Rectangles {
    pub fn new(device: &wgpu::Device) -> Self {
        let initial_capacity = 64;

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Quad Buffer"),
            size: (std::mem::size_of::<RectangleData>() * initial_capacity) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = Self::bind_group_layout(device);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Rectangles Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                },
            ],
        });

        Self {
            rectangles: Vec::new(),
            buffer,
            buffer_capacity: initial_capacity,
            bind_group,
        }
    }

    pub fn clear(&mut self) {
        self.rectangles.clear();
    }

    pub fn push(&mut self, primitive: RectangleData) -> usize {
        let index = self.rectangles.len();
        self.rectangles.push(primitive);
        index
    }

    pub fn len(&self) -> usize {
        self.rectangles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rectangles.is_empty()
    }

    pub fn upload(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        if self.rectangles.is_empty() {
            return;
        }

        // Grow buffer if needed
        if self.rectangles.len() > self.buffer_capacity {
            let new_capacity = self.rectangles.len().next_power_of_two();
            self.buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Quad Buffer"),
                size: (std::mem::size_of::<RectangleData>() * new_capacity) as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.buffer_capacity = new_capacity;

            // Recreate bind group with new buffer
            self.bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Rectangles Bind Group"),
                layout: &Self::bind_group_layout(device),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: self.buffer.as_entire_binding(),
                    },
                ],
            });
        }

        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&self.rectangles));
    }

    pub fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Rectangles Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        })
    }
}
