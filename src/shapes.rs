use crate::gpu_vec::GpuVec;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BoxGpu {
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
    pub rounded_corners: u32, // bitflags: 1=top-left, 2=top-right, 4=bottom-left, 8=bottom-right
    pub pad: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CircleGpu {
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
pub struct SegmentGpu {
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

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GridGpu {
    pub top_left: [f32; 2],
    pub size: [f32; 2],
    pub x_clip: [f32; 2],
    pub y_clip: [f32; 2],
    pub offset: [f32; 2],
    pub lattice_size: f32,
    pub line_thickness: f32,
    pub color: [f32; 4],
    pub grid_type: u32, // 0=square, 1=hex
    pub pad: [f32; 3],
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TriangleGpu {
    pub p0: [f32; 2],
    pub p1: [f32; 2],
    pub p2: [f32; 2],
    pub x_clip: [f32; 2],
    pub y_clip: [f32; 2],
    pub gradient_direction: [f32; 2],
    pub color_start: [f32; 4],
    pub color_end: [f32; 4],
    pub gradient_type: u32, // 0=solid, 1=linear
    pub pad: [f32; 3],
}

pub struct Shapes {
    pub(crate) boxes: GpuVec<BoxGpu>,
    pub(crate) circles: GpuVec<CircleGpu>,
    pub(crate) segments: GpuVec<SegmentGpu>,
    pub(crate) grids: GpuVec<GridGpu>,
    pub(crate) triangles: GpuVec<TriangleGpu>,
}

impl Shapes {
    pub fn new(device: &wgpu::Device) -> Self {
        let boxes = GpuVec::new(device, 64, "keru_draw boxes");
        let circles = GpuVec::new(device, 64, "keru_draw circles");
        let segments = GpuVec::new(device, 64, "keru_draw segments");
        let grids = GpuVec::new(device, 64, "keru_draw grids");
        let triangles = GpuVec::new(device, 64, "keru_draw triangles");

        Self {
            boxes,
            circles,
            segments,
            grids,
            triangles,
        }
    }

    pub fn clear(&mut self) {
        self.boxes.clear();
        self.circles.clear();
        self.segments.clear();
        self.grids.clear();
        self.triangles.clear();
    }

    pub fn load_to_gpu(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) -> bool {
        let boxes_changed = self.boxes.load_to_gpu(device, queue);
        let circles_changed = self.circles.load_to_gpu(device, queue);
        let segments_changed = self.segments.load_to_gpu(device, queue);
        let grids_changed = self.grids.load_to_gpu(device, queue);
        let triangles_changed = self.triangles.load_to_gpu(device, queue);

        boxes_changed || circles_changed || segments_changed || grids_changed || triangles_changed
    }

    pub fn bind_group_layout_entries() -> Vec<wgpu::BindGroupLayoutEntry> {
        vec![
            GpuVec::<BoxGpu>::bind_group_layout_entry(0),
            GpuVec::<CircleGpu>::bind_group_layout_entry(1),
            GpuVec::<SegmentGpu>::bind_group_layout_entry(2),
            GpuVec::<GridGpu>::bind_group_layout_entry(3),
            GpuVec::<TriangleGpu>::bind_group_layout_entry(4),
        ]
    }
}
