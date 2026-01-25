use crate::*;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Box {
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

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Circle {
    pub center: [f32; 2],
    pub radii: [f32; 2],      // [inner_radius, outer_radius]
    pub angles: [f32; 2],     // [start_angle, end_angle] in radians
    pub x_clip: [f32; 2],
    pub y_clip: [f32; 2],
    pub gradient_direction: [f32; 2],
    pub color_start: [f32; 4],
    pub color_end: [f32; 4],
    pub gradient_type: u32, // 0=solid, 1=linear, 2=radial
    pub _padding: [f32; 3],
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Segment {
    pub start: [f32; 2],
    pub end: [f32; 2],
    pub x_clip: [f32; 2],
    pub y_clip: [f32; 2],
    pub color_start: [f32; 4],
    pub color_end: [f32; 4],
    pub thickness_dash: [f32; 4], // [thickness, dash_length, unused, unused]
    pub gradient_type: u32, // 0=solid, 1=linear along segment
    pub pad: [f32; 3],
}

use crate::gpu_vec::GpuVec;

pub struct Shapes {
    boxes: GpuVec<Box>,
    circles: GpuVec<Circle>,
    segments: GpuVec<Segment>,
    pub transforms: GpuVec<Transform>,
    pub bind_group: wgpu::BindGroup,
}

impl Shapes {
    pub fn new(device: &wgpu::Device) -> Self {
        let mut transforms = GpuVec::new(device, 64, "keru_draw transforms");
        // Push identity transform at index 0
        transforms.push(crate::Transform::identity());

        let boxes = GpuVec::new(device, 64, "keru_draw boxes");
        let circles = GpuVec::new(device, 64, "keru_draw circles");
        let segments = GpuVec::new(device, 64, "keru_draw segments");

        let bind_group_layout = Self::bind_group_layout(device);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Shapes Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                transforms.bind_group_entry(0),
                boxes.bind_group_entry(1),
                circles.bind_group_entry(2),
                segments.bind_group_entry(3),
            ],
        });

        Self {
            transforms,
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
        let index = self.boxes.len();
        self.boxes.push(Box {
            top_left,
            size,
            x_clip,
            y_clip,
            corner_radius,
            border_thickness,
            gradient_direction: [1.0, 0.0],
            color_start: color,
            color_end: color,
            gradient_type: 0, // solid
            pad: [0.0, 0.0, 0.0],
        });
        index
    }

    pub fn push_box_gradient(
        &mut self,
        top_left: [f32; 2],
        size: [f32; 2],
        corner_radius: f32,
        border_thickness: f32,
        color_start: [f32; 4],
        color_end: [f32; 4],
        gradient_angle: f32, // angle in radians
        x_clip: [f32; 2],
        y_clip: [f32; 2],
    ) -> usize {
        let gradient_direction = [gradient_angle.cos(), gradient_angle.sin()];
        let index = self.boxes.len();
        self.boxes.push(Box {
            top_left,
            size,
            x_clip,
            y_clip,
            corner_radius,
            border_thickness,
            gradient_direction,
            color_start,
            color_end,
            gradient_type: 1, // linear
            pad: [0.0, 0.0, 0.0],
        });
        index
    }


    pub fn push_circle(
        &mut self,
        center: [f32; 2],
        radius: f32,
        color: [f32; 4],
        x_clip: [f32; 2],
        y_clip: [f32; 2],
    ) -> usize {
        let index = self.circles.len();
        self.circles.push(Circle {
            center,
            radii: [0.0, radius],
            angles: [0.0, std::f32::consts::TAU],
            x_clip,
            y_clip,
            gradient_direction: [1.0, 0.0],
            color_start: color,
            color_end: color,
            gradient_type: 0, // solid
            _padding: [0.0, 0.0, 0.0],
        });
        index
    }

    pub fn push_circle_gradient(
        &mut self,
        center: [f32; 2],
        radius: f32,
        color_start: [f32; 4],
        color_end: [f32; 4],
        gradient_type: u32, // 1=linear, 2=radial
        gradient_angle: f32, // angle in radians (for linear)
        x_clip: [f32; 2],
        y_clip: [f32; 2],
    ) -> usize {
        let gradient_direction = [gradient_angle.cos(), gradient_angle.sin()];
        let index = self.circles.len();
        self.circles.push(Circle {
            center,
            radii: [0.0, radius],
            angles: [0.0, std::f32::consts::TAU],
            x_clip,
            y_clip,
            gradient_direction,
            color_start,
            color_end,
            gradient_type,
            _padding: [0.0, 0.0, 0.0],
        });
        index
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
        let index = self.circles.len();
        self.circles.push(Circle {
            center,
            radii: [inner_radius, outer_radius],
            angles: [0.0, std::f32::consts::TAU],
            x_clip,
            y_clip,
            gradient_direction: [1.0, 0.0],
            color_start: color,
            color_end: color,
            gradient_type: 0, // solid
            _padding: [0.0, 0.0, 0.0],
        });
        index
    }

    pub fn push_ring_gradient(
        &mut self,
        center: [f32; 2],
        inner_radius: f32,
        outer_radius: f32,
        color_start: [f32; 4],
        color_end: [f32; 4],
        gradient_type: u32, // 1=linear, 2=radial
        gradient_angle: f32, // angle in radians (for linear)
        x_clip: [f32; 2],
        y_clip: [f32; 2],
    ) -> usize {
        let gradient_direction = [gradient_angle.cos(), gradient_angle.sin()];
        let index = self.circles.len();
        self.circles.push(Circle {
            center,
            radii: [inner_radius, outer_radius],
            angles: [0.0, std::f32::consts::TAU],
            x_clip,
            y_clip,
            gradient_direction,
            color_start,
            color_end,
            gradient_type,
            _padding: [0.0, 0.0, 0.0],
        });
        index
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
        let index = self.circles.len();
        self.circles.push(Circle {
            center,
            radii: [radius - thickness * 0.5, radius + thickness * 0.5],
            angles: [start_angle, end_angle],
            x_clip,
            y_clip,
            gradient_direction: [1.0, 0.0],
            color_start: color,
            color_end: color,
            gradient_type: 0, // solid
            _padding: [0.0, 0.0, 0.0],
        });
        index
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
        let index = self.circles.len();
        self.circles.push(Circle {
            center,
            radii: [0.0, radius],
            angles: [start_angle, end_angle],
            x_clip,
            y_clip,
            gradient_direction: [1.0, 0.0],
            color_start: color,
            color_end: color,
            gradient_type: 0, // solid
            _padding: [0.0, 0.0, 0.0],
        });
        index
    }


    pub fn push_segment(
        &mut self,
        start: [f32; 2],
        end: [f32; 2],
        thickness: f32,
        color: [f32; 4],
        x_clip: [f32; 2],
        y_clip: [f32; 2],
        dash_length: Option<f32>,
    ) -> usize {
        let index = self.segments.len();
        self.segments.push(Segment {
            start,
            end,
            x_clip,
            y_clip,
            color_start: color,
            color_end: color,
            thickness_dash: [thickness, dash_length.unwrap_or(0.0), 1.0, 1.0],
            gradient_type: 0, // solid
            pad: [0.0, 0.0, 0.0],
        });
        index
    }

    pub fn push_segment_gradient(
        &mut self,
        start: [f32; 2],
        end: [f32; 2],
        thickness: f32,
        color_start: [f32; 4],
        color_end: [f32; 4],
        x_clip: [f32; 2],
        y_clip: [f32; 2],
        dash_length: Option<f32>,
    ) -> usize {
        let index = self.segments.len();
        self.segments.push(Segment {
            start,
            end,
            x_clip,
            y_clip,
            color_start,
            color_end,
            thickness_dash: [thickness, dash_length.unwrap_or(0.0), 1.0, 1.0],
            gradient_type: 1, // linear along segment
            pad: [0.0, 0.0, 0.0],
        });
        index
    }

    pub fn load_to_gpu(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let transforms_changed = self.transforms.load_to_gpu(device, queue);
        let boxes_changed = self.boxes.load_to_gpu(device, queue);
        let circles_changed = self.circles.load_to_gpu(device, queue);
        let segments_changed = self.segments.load_to_gpu(device, queue);

        if transforms_changed || boxes_changed || circles_changed || segments_changed {
            let bind_group_layout = Self::bind_group_layout(device);
            self.bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("keru_draw shapes bind group"),
                layout: &bind_group_layout,
                entries: &[
                    self.transforms.bind_group_entry(0),
                    self.boxes.bind_group_entry(1),
                    self.circles.bind_group_entry(2),
                    self.segments.bind_group_entry(3),
                ],
            });
        }
    }

    pub fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("keru_draw shapes bind group layout"),
            entries: &[
                GpuVec::<Transform>::bind_group_layout_entry(0),
                GpuVec::<Box>::bind_group_layout_entry(1),
                GpuVec::<Circle>::bind_group_layout_entry(2),
                GpuVec::<Segment>::bind_group_layout_entry(3),
            ],
        })
    }
}
