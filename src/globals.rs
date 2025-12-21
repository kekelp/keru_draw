#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GlobalsData {
    pub resolution: [f32; 2],
    pub time: f32,
    pub _padding: f32,
}

pub struct Globals {
    pub data: GlobalsData,
    buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl Globals {
    pub fn new(device: &wgpu::Device) -> Self {
        let data = GlobalsData {
            resolution: [0.0, 0.0],
            time: 0.0,
            _padding: 0.0,
        };

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Globals Buffer"),
            size: std::mem::size_of::<GlobalsData>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = Self::bind_group_layout(device);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Globals Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                },
            ],
        });

        Self {
            data,
            buffer,
            bind_group,
            bind_group_layout,
        }
    }

    pub fn set_resolution(&mut self, width: f32, height: f32) {
        self.data.resolution = [width, height];
    }

    pub fn set_time(&mut self, time: f32) {
        self.data.time = time;
    }

    pub fn upload(&self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(&self.data));
    }

    pub fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Globals Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        })
    }
}
