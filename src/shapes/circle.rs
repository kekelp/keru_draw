#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CircleData {
    pub center: [f32; 2],
    pub radii: [f32; 2],      // [inner_radius, outer_radius]
    pub angles: [f32; 2],     // [start_angle, end_angle] in radians
    pub x_clip: [f32; 2],
    pub y_clip: [f32; 2],
    pub color: [f32; 3],
    pub _padding: f32,
}

pub struct Circles {
    circles: Vec<CircleData>,
    pub buffer: wgpu::Buffer,
    buffer_capacity: usize,
}

impl Circles {
    pub fn new(device: &wgpu::Device) -> Self {
        let initial_capacity = 64;

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Circle Buffer"),
            size: (std::mem::size_of::<CircleData>() * initial_capacity) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            circles: Vec::new(),
            buffer,
            buffer_capacity: initial_capacity,
        }
    }

    pub fn clear(&mut self) {
        self.circles.clear();
    }

    pub fn push(&mut self, primitive: CircleData) -> usize {
        let index = self.circles.len();
        self.circles.push(primitive);
        index
    }

    pub fn len(&self) -> usize {
        self.circles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.circles.is_empty()
    }

    // Returns true if buffer was reallocated
    pub fn upload(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) -> bool {
        if self.circles.is_empty() {
            return false;
        }

        let mut buffer_recreated = false;

        // Grow buffer if needed
        if self.circles.len() > self.buffer_capacity {
            let new_capacity = self.circles.len().next_power_of_two();
            self.buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Circle Buffer"),
                size: (std::mem::size_of::<CircleData>() * new_capacity) as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.buffer_capacity = new_capacity;
            buffer_recreated = true;
        }

        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&self.circles));
        buffer_recreated
    }
}
