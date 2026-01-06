#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BoxData {
    pub top_left: [f32; 2],
    pub size: [f32; 2],
    pub x_clip: [f32; 2],
    pub y_clip: [f32; 2],
    pub corner_radius: f32,
    pub border_thickness: f32,
    pub gradient_direction: [f32; 2],
    pub color_start: [f32; 4],
    pub color_end: [f32; 4],
    pub gradient_type: u32, // 0=solid, 1=linear
    pub pad: [f32; 3],
}

pub struct Boxes {
    boxes: Vec<BoxData>,
    pub buffer: wgpu::Buffer,
    buffer_capacity: usize,
}

impl Boxes {
    pub fn new(device: &wgpu::Device) -> Self {
        let initial_capacity = 64;

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Box Buffer"),
            size: (std::mem::size_of::<BoxData>() * initial_capacity) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            boxes: Vec::new(),
            buffer,
            buffer_capacity: initial_capacity,
        }
    }

    pub fn clear(&mut self) {
        self.boxes.clear();
    }

    pub fn push(&mut self, primitive: BoxData) -> usize {
        let index = self.boxes.len();
        self.boxes.push(primitive);
        index
    }

    pub fn len(&self) -> usize {
        self.boxes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.boxes.is_empty()
    }

    // Returns true if buffer was reallocated
    pub fn upload(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) -> bool {
        if self.boxes.is_empty() {
            return false;
        }

        let mut buffer_recreated = false;

        // Grow buffer if needed
        if self.boxes.len() > self.buffer_capacity {
            let new_capacity = self.boxes.len().next_power_of_two();
            self.buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Box Buffer"),
                size: (std::mem::size_of::<BoxData>() * new_capacity) as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.buffer_capacity = new_capacity;
            buffer_recreated = true;
        }

        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&self.boxes));
        buffer_recreated
    }
}
