use textslabs::Quad;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TextGlyphData {
    pub top_left: [f32; 2],
    pub size: [f32; 2],
    pub uv_origin: [f32; 2],
    pub color: [f32; 3],
    pub alpha: f32,
    pub x_clip: [f32; 2],
    pub y_clip: [f32; 2],
    pub page_index: u32,
    pub _padding1: f32,
    pub _padding2: f32,
    pub _padding3: f32,
}

pub struct TextGlyphs {
    glyphs: Vec<TextGlyphData>,
    buffer: wgpu::Buffer,
    buffer_capacity: usize,
    pub bind_group: wgpu::BindGroup,
}

impl TextGlyphs {
    pub fn new(device: &wgpu::Device) -> Self {
        let initial_capacity = 256;

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Text Glyph Buffer"),
            size: (std::mem::size_of::<TextGlyphData>() * initial_capacity) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = Self::bind_group_layout(device);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("TextGlyphs Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                },
            ],
        });

        Self {
            glyphs: Vec::new(),
            buffer,
            buffer_capacity: initial_capacity,
            bind_group,
        }
    }

    pub fn clear(&mut self) {
        self.glyphs.clear();
    }

    pub fn push(&mut self, glyph: TextGlyphData) -> usize {
        let index = self.glyphs.len();
        self.glyphs.push(glyph);
        index
    }

    pub fn len(&self) -> usize {
        self.glyphs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.glyphs.is_empty()
    }

    pub fn upload(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        if self.glyphs.is_empty() {
            return;
        }

        // Grow buffer if needed
        if self.glyphs.len() > self.buffer_capacity {
            let new_capacity = self.glyphs.len().next_power_of_two();
            self.buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Text Glyph Buffer"),
                size: (std::mem::size_of::<TextGlyphData>() * new_capacity) as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.buffer_capacity = new_capacity;

            // Recreate bind group with new buffer
            self.bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("TextGlyphs Bind Group"),
                layout: &Self::bind_group_layout(device),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: self.buffer.as_entire_binding(),
                    },
                ],
            });
        }

        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&self.glyphs));
    }

    pub fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("TextGlyphs Bind Group Layout"),
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

// Helper function to convert textslabs Quad to TextGlyphData
fn unpack_u32_to_f32_pair(packed: u32) -> [f32; 2] {
    let x = (packed & 0xFFFF) as u16;
    let y = ((packed >> 16) & 0xFFFF) as u16;
    // Interpret as i16 for signed positions
    let x_i16 = x as i16;
    let y_i16 = y as i16;
    [x_i16 as f32, y_i16 as f32]
}

fn unpack_clip_rect(packed: [u32; 2]) -> ([f32; 2], [f32; 2]) {
    let xy = unpack_u32_to_f32_pair(packed[0]);
    let wh = unpack_u32_to_f32_pair(packed[1]);
    ([xy[0], wh[0]], [xy[1], wh[1]])
}

fn unpack_color(packed: u32) -> ([f32; 3], f32) {
    let r = ((packed & 0xff000000) >> 24) as f32 / 255.0;
    let g = ((packed & 0x00ff0000) >> 16) as f32 / 255.0;
    let b = ((packed & 0x0000ff00) >> 8) as f32 / 255.0;
    let a = (packed & 0x000000ff) as f32 / 255.0;
    ([r, g, b], a)
}

fn unpack_page_index(flags_and_page: u32) -> u32 {
    (flags_and_page >> 24) & 0xFF
}

pub fn quad_to_glyph_data(quad: &Quad) -> TextGlyphData {
    let pos = unpack_u32_to_f32_pair(quad.pos_packed);
    let dim = unpack_u32_to_f32_pair(quad.dim_packed);
    let uv = unpack_u32_to_f32_pair(quad.uv_origin_packed);
    let (x_clip, y_clip) = unpack_clip_rect(quad.clip_rect_packed);
    let (color, alpha) = unpack_color(quad.color);
    let page_index = unpack_page_index(quad.flags_and_page);

    TextGlyphData {
        top_left: pos,
        size: dim,
        uv_origin: uv,
        color,
        alpha,
        x_clip,
        y_clip,
        page_index,
        _padding1: 0.0,
        _padding2: 0.0,
        _padding3: 0.0,
    }
}
