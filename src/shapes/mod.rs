mod box_shape;
mod circle;
mod segment;

pub use box_shape::{BoxData, Boxes};
pub use circle::{CircleData, Circles};
pub use segment::{SegmentData, Segments};

pub struct Shapes {
    boxes: Boxes,
    circles: Circles,
    segments: Segments,
    pub bind_group: wgpu::BindGroup,
}

impl Shapes {
    pub fn new(device: &wgpu::Device) -> Self {
        let boxes = Boxes::new(device);
        let circles = Circles::new(device);
        let segments = Segments::new(device);

        let bind_group_layout = Self::bind_group_layout(device);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Shapes Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: boxes.buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: circles.buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: segments.buffer.as_entire_binding(),
                },
            ],
        });

        Self {
            boxes,
            circles,
            segments,
            bind_group,
        }
    }

    pub fn clear(&mut self) {
        self.boxes.clear();
        self.circles.clear();
        self.segments.clear();
    }


    pub fn push_box(
        &mut self,
        top_left: [f32; 2],
        size: [f32; 2],
        corner_radius: f32,
        border_thickness: f32,
        color: [f32; 4],
        x_clip: [f32; 2],
        y_clip: [f32; 2],
    ) -> usize {
        self.boxes.push(BoxData {
            top_left,
            size,
            x_clip,
            y_clip,
            corner_radius,
            border_thickness,
            pad: [0.0, 0.0],
            color,
        })
    }


    pub fn push_circle(
        &mut self,
        center: [f32; 2],
        radius: f32,
        color: [f32; 4],
        x_clip: [f32; 2],
        y_clip: [f32; 2],
    ) -> usize {
        self.circles.push(CircleData {
            center,
            radii: [0.0, radius],
            angles: [0.0, std::f32::consts::TAU],
            x_clip,
            y_clip,
            _padding: [0.0, 0.0],
            color,
        })
    }


    pub fn push_ring(
        &mut self,
        center: [f32; 2],
        inner_radius: f32,
        outer_radius: f32,
        color: [f32; 4],
        x_clip: [f32; 2],
        y_clip: [f32; 2],
    ) -> usize {
        self.circles.push(CircleData {
            center,
            radii: [inner_radius, outer_radius],
            angles: [0.0, std::f32::consts::TAU],
            x_clip,
            y_clip,
            _padding: [0.0, 0.0],
            color,
        })
    }


    pub fn push_arc(
        &mut self,
        center: [f32; 2],
        radius: f32,
        start_angle: f32,
        end_angle: f32,
        thickness: f32,
        color: [f32; 4],
        x_clip: [f32; 2],
        y_clip: [f32; 2],
    ) -> usize {
        self.circles.push(CircleData {
            center,
            radii: [radius - thickness * 0.5, radius + thickness * 0.5],
            angles: [start_angle, end_angle],
            x_clip,
            y_clip,
            _padding: [0.0, 0.0],
            color,
        })
    }


    pub fn push_pie(
        &mut self,
        center: [f32; 2],
        radius: f32,
        start_angle: f32,
        end_angle: f32,
        color: [f32; 4],
        x_clip: [f32; 2],
        y_clip: [f32; 2],
    ) -> usize {
        self.circles.push(CircleData {
            center,
            radii: [0.0, radius],
            angles: [start_angle, end_angle],
            x_clip,
            y_clip,
            _padding: [0.0, 0.0],
            color,
        })
    }


    pub fn push_segment(
        &mut self,
        start: [f32; 2],
        end: [f32; 2],
        thickness: f32,
        color: [f32; 4],
        x_clip: [f32; 2],
        y_clip: [f32; 2],
    ) -> usize {
        self.segments.push(SegmentData {
            start,
            end,
            x_clip,
            y_clip,
            color,
            thickness_dash: [thickness, 1.0, 1.0, 1.0],
        })
    }

    pub fn upload(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let boxes_changed = self.boxes.upload(device, queue);
        let circles_changed = self.circles.upload(device, queue);
        let segments_changed = self.segments.upload(device, queue);

    
        if boxes_changed || circles_changed || segments_changed {
            let bind_group_layout = Self::bind_group_layout(device);
            self.bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Shapes Bind Group"),
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: self.boxes.buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: self.circles.buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: self.segments.buffer.as_entire_binding(),
                    },
                ],
            });
        }
    }

    pub fn boxes_len(&self) -> usize {
        self.boxes.len()
    }

    pub fn circles_len(&self) -> usize {
        self.circles.len()
    }

    pub fn segments_len(&self) -> usize {
        self.segments.len()
    }


    pub fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Shapes Bind Group Layout"),
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
            
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
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
