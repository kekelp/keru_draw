// Ellipse primitive module

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct EllipseData {
    pub center: [f32; 2],
    pub size: [f32; 2],
    pub color: [f32; 3],
    pub _padding: f32,
}

pub struct Ellipses {
    primitives: Vec<EllipseData>,
    buffer: wgpu::Buffer,
    buffer_capacity: usize,
    pub bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl Ellipses {
    pub fn new(device: &wgpu::Device) -> Self {
        let initial_capacity = 64;

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Ellipse Buffer"),
            size: (std::mem::size_of::<EllipseData>() * initial_capacity) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = Self::create_bind_group_layout(device);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("EllipseResources Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                },
            ],
        });

        Self {
            primitives: Vec::new(),
            buffer,
            buffer_capacity: initial_capacity,
            bind_group,
            bind_group_layout,
        }
    }

    pub fn clear(&mut self) {
        self.primitives.clear();
    }

    pub fn push(&mut self, primitive: EllipseData) {
        self.primitives.push(primitive);
    }

    pub fn len(&self) -> usize {
        self.primitives.len()
    }

    pub fn is_empty(&self) -> bool {
        self.primitives.is_empty()
    }

    pub fn upload(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        if self.primitives.is_empty() {
            return;
        }

        // Grow buffer if needed
        if self.primitives.len() > self.buffer_capacity {
            let new_capacity = self.primitives.len().next_power_of_two();
            self.buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Ellipse Buffer"),
                size: (std::mem::size_of::<EllipseData>() * new_capacity) as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.buffer_capacity = new_capacity;

            // Recreate bind group with new buffer
            self.bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("EllipseResources Bind Group"),
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: self.buffer.as_entire_binding(),
                    },
                ],
            });
        }

        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&self.primitives));
    }

    pub fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("EllipseResources Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
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
