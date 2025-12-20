#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct AABB {
    pub min: [f32; 2],
    pub max: [f32; 2],
}
