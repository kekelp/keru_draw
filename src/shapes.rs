use crate::gpu_vec::GpuVec;
use crate::Color;


#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Default)]
pub struct GradientGpu {
    pub color_start: Color,
    pub color_end: Color,
    pub gradient_direction: [f32; 2], // normalized direction for linear gradient
    pub gradient_type: u32, // 0=solid, 1=linear, 2=radial
    pub _pad: u32,
}












#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Default)]
pub struct RectangleGpu {
    pub top_left: [f32; 2],
    pub size: [f32; 2],
    pub nine_slice_l: f32,           // left inset in pixels (0 = no nine-slice)
    pub nine_slice_r: f32,           // right inset in pixels
    pub nine_slice_t: f32,           // top inset in pixels
    pub nine_slice_b: f32,           // bottom inset in pixels
    pub corner_radius: f32,
    pub border_thickness: f32,
    pub gradient_direction: [f32; 2],
    pub color_start: Color,
    pub color_end: Color,
    pub gradient_index: u32, // index into gradients buffer
    pub rounded_corners: u32, // bitflags: 1=top-left, 2=top-right, 4=bottom-left, 8=bottom-right
    pub texture_uv_origin: [f32; 2], // pixel coords in atlas (top-left corner)
    pub texture_uv_size: [f32; 2],   // pixel dimensions in atlas
    pub texture_page: u32,           // atlas layer, u32::MAX = no texture
    pub blur_radius: f32,
    pub nine_slice_tiling: u32,      // bit 0=enabled; bits 1-2=h mode; bits 3-4=v mode (0=stretch,1=tile,2=tile_fit)
    pub _ns_pad: [f32; 3],           // padding to 128 bytes (16-byte aligned)
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CircleGpu {
    pub color_start: Color,
    pub color_end: Color,
    pub center: [f32; 2],
    pub radii: [f32; 2],      // [inner_radius, outer_radius]
    pub angles: [f32; 2],     // [start_angle, end_angle] in radians
    pub gradient_direction: [f32; 2],
    pub gradient_index: u32, // index into gradients buffer
    pub texture_page: u32,           // atlas layer, u32::MAX = no texture
    pub texture_uv_origin: [f32; 2], // pixel coords in atlas (top-left corner)
    pub texture_uv_size: [f32; 2],   // pixel dimensions in atlas
    pub dash_length: f32,            // 0 = no dashing, >0 = dash length in pixels
    pub dash_offset: f32,            // offset for dash pattern alignment
    pub blur_radius: f32,
    pub nine_slice_l: f32,           // left inset in pixels (0 = no nine-slice)
    pub nine_slice_r: f32,           // right inset in pixels
    pub nine_slice_t: f32,           // top inset in pixels
    pub nine_slice_b: f32,           // bottom inset in pixels
    pub nine_slice_tiling: u32,      // bit 0=enabled; bits 1-2=h mode; bits 3-4=v mode (0=stretch,1=tile,2=tile_fit)
    pub _ns_pad: [f32; 2],           // padding to 128 bytes (16-byte aligned)
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SegmentGpu {
    pub start: [f32; 2],
    pub end: [f32; 2],
    pub color_start: Color,
    pub color_end: Color,
    pub thickness_dash: [f32; 4], // [thickness, dash_length, dash_offset, unused]
    pub gradient_index: u32, // index into gradients buffer
    pub texture_page: u32,           // atlas layer, u32::MAX = no texture
    pub texture_uv_origin: [f32; 2], // pixel coords in atlas (top-left corner)
    pub texture_uv_size: [f32; 2],   // pixel dimensions in atlas
    pub blur_radius: f32,
    pub nine_slice_l: f32,           // left inset in pixels (0 = no nine-slice)
    pub nine_slice_r: f32,           // right inset in pixels
    pub nine_slice_t: f32,           // top inset in pixels
    pub nine_slice_b: f32,           // bottom inset in pixels
    pub nine_slice_tiling: u32,      // bit 0=enabled; bits 1-2=h mode; bits 3-4=v mode (0=stretch,1=tile,2=tile_fit)
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GridGpu {
    pub color_start: Color,
    pub color_end: Color,
    pub top_left: [f32; 2],
    pub size: [f32; 2],
    pub offset: [f32; 2],
    pub lattice_size: f32,
    pub line_thickness: f32,
    pub gradient_direction: [f32; 2],
    pub _pad: f32,
    pub gradient_index: u32, // index into gradients buffer
    pub grid_type: u32, // 0=square, 1=hex
    pub texture_page: u32,           // atlas layer, u32::MAX = no texture
    pub texture_uv_origin: [f32; 2], // pixel coords in atlas (top-left corner)
    pub texture_uv_size: [f32; 2],   // pixel dimensions in atlas
    pub blur_radius: f32,
    pub nine_slice_l: f32,           // left inset in pixels (0 = no nine-slice)
    pub nine_slice_r: f32,           // right inset in pixels
    pub nine_slice_t: f32,           // top inset in pixels
    pub nine_slice_b: f32,           // bottom inset in pixels
    pub nine_slice_tiling: u32,      // bit 0=enabled; bits 1-2=h mode; bits 3-4=v mode (0=stretch,1=tile,2=tile_fit)
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TriangleGpu {
    pub p0: [f32; 2],
    pub p1: [f32; 2],
    pub p2: [f32; 2],
    pub gradient_direction: [f32; 2],
    pub color_start: Color,
    pub color_end: Color,
    pub gradient_index: u32, // index into gradients buffer
    pub texture_page: u32,           // atlas layer, u32::MAX = no texture
    pub texture_uv_origin: [f32; 2], // pixel coords in atlas (top-left corner)
    pub texture_uv_size: [f32; 2],   // pixel dimensions in atlas
    pub blur_radius: f32,
    pub nine_slice_l: f32,           // left inset in pixels (0 = no nine-slice)
    pub nine_slice_r: f32,           // right inset in pixels
    pub nine_slice_t: f32,           // top inset in pixels
    pub nine_slice_b: f32,           // bottom inset in pixels
    pub nine_slice_tiling: u32,      // bit 0=enabled; bits 1-2=h mode; bits 3-4=v mode (0=stretch,1=tile,2=tile_fit)
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct HexagonGpu {
    pub center: [f32; 2],
    pub size: f32,
    pub rotation: f32,
    pub gradient_direction: [f32; 2],
    pub stroke_thickness: f32,
    pub texture_page: u32,
    pub color_start: Color,
    pub color_end: Color,
    pub gradient_index: u32, // index into gradients buffer
    pub nine_slice_l: f32,           // left inset in pixels (0 = no nine-slice)
    pub texture_uv_origin: [f32; 2],
    pub texture_uv_size: [f32; 2],
    pub blur_radius: f32,
    pub nine_slice_r: f32,           // right inset in pixels
    pub nine_slice_t: f32,           // top inset in pixels
    pub nine_slice_b: f32,           // bottom inset in pixels
    pub nine_slice_tiling: u32,      // bit 0=enabled; bits 1-2=h mode; bits 3-4=v mode (0=stretch,1=tile,2=tile_fit)
    pub _ns_pad: f32,                // padding to 112 bytes (16-byte aligned)
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct QuadraticBezierGpu {
    pub p0: [f32; 2],
    pub p1: [f32; 2],
    pub p2: [f32; 2],
    pub thickness: f32,
    pub blur_radius: f32,
    pub gradient_index: u32, // index into gradients buffer (replaces color: Color, same 16 bytes)
    pub _color_unused: [f32; 3],
}

pub struct Shapes {
    pub(crate) gradient_indices: Vec<usize>,
    pub(crate) boxes: GpuVec<RectangleGpu>,
    pub(crate) circles: GpuVec<CircleGpu>,
    pub(crate) segments: GpuVec<SegmentGpu>,
    pub(crate) grids: GpuVec<GridGpu>,
    pub(crate) triangles: GpuVec<TriangleGpu>,
    pub(crate) hexagons: GpuVec<HexagonGpu>,
    pub(crate) quadratic_beziers: GpuVec<QuadraticBezierGpu>,
}

impl Shapes {
    pub fn new(device: &wgpu::Device) -> Self {
        let boxes = GpuVec::new(device, 64, "keru_draw boxes");
        let circles = GpuVec::new(device, 64, "keru_draw circles");
        let segments = GpuVec::new(device, 64, "keru_draw segments");
        let grids = GpuVec::new(device, 64, "keru_draw grids");
        let triangles = GpuVec::new(device, 64, "keru_draw triangles");
        let hexagons = GpuVec::new(device, 64, "keru_draw hexagons");
        let quadratic_beziers = GpuVec::new(device, 64, "keru_draw quadratic_beziers");

        Self {
            gradient_indices: Vec::new(),
            boxes,
            circles,
            segments,
            grids,
            triangles,
            hexagons,
            quadratic_beziers,
        }
    }

    pub fn clear(&mut self) {
        // gradient_indices are drained by Renderer::clear_for_new_frame before calling this
        self.boxes.clear();
        self.circles.clear();
        self.segments.clear();
        self.grids.clear();
        self.triangles.clear();
        self.hexagons.clear();
        self.quadratic_beziers.clear();
    }

    pub fn load_to_gpu(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) -> bool {
        let boxes_changed = self.boxes.load_to_gpu(device, queue);
        let circles_changed = self.circles.load_to_gpu(device, queue);
        let segments_changed = self.segments.load_to_gpu(device, queue);
        let grids_changed = self.grids.load_to_gpu(device, queue);
        let triangles_changed = self.triangles.load_to_gpu(device, queue);
        let hexagons_changed = self.hexagons.load_to_gpu(device, queue);
        let quadratic_beziers_changed = self.quadratic_beziers.load_to_gpu(device, queue);

        boxes_changed || circles_changed || segments_changed || grids_changed || triangles_changed || hexagons_changed || quadratic_beziers_changed
    }
}
