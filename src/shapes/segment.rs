#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SegmentData {
    pub start: [f32; 2],
    pub end: [f32; 2],
    pub x_clip: [f32; 2],
    pub y_clip: [f32; 2],
    pub color_start: [f32; 4],
    pub color_end: [f32; 4],
    pub thickness_dash: [f32; 4],
    pub gradient_type: u32, // 0=solid, 1=linear along segment
    pub pad: [f32; 3],
}

pub struct Segments {
    segments: Vec<SegmentData>,
    pub buffer: wgpu::Buffer,
    buffer_capacity: usize,
}

impl Segments {
    pub fn new(device: &wgpu::Device) -> Self {
        let initial_capacity = 64;

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Segment Buffer"),
            size: (std::mem::size_of::<SegmentData>() * initial_capacity) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            segments: Vec::new(),
            buffer,
            buffer_capacity: initial_capacity,
        }
    }

    pub fn clear(&mut self) {
        self.segments.clear();
    }

    pub fn push(&mut self, primitive: SegmentData) -> usize {
        let index = self.segments.len();
        self.segments.push(primitive);
        index
    }

    pub fn len(&self) -> usize {
        self.segments.len()
    }

    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    // Returns true if buffer was reallocated
    pub fn upload(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) -> bool {
        if self.segments.is_empty() {
            return false;
        }

        let mut buffer_recreated = false;

        // Grow buffer if needed
        if self.segments.len() > self.buffer_capacity {
            let new_capacity = self.segments.len().next_power_of_two();
            self.buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Segment Buffer"),
                size: (std::mem::size_of::<SegmentData>() * new_capacity) as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.buffer_capacity = new_capacity;
            buffer_recreated = true;
        }

        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&self.segments));
        buffer_recreated
    }
}
