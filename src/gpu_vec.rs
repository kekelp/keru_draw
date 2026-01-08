use std::mem::size_of;
use std::ops::{Index, IndexMut};
use wgpu::*;

pub struct GpuVec<T: Copy> {
    data: Vec<T>,
    buffer: wgpu::Buffer,
    buffer_capacity: usize,
    label: String,
    dirty: bool,
}

impl<T: Copy> GpuVec<T> {
    pub fn new(device: &wgpu::Device, capacity: usize, label: &str) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: (size_of::<T>() * capacity) as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            buffer,
            buffer_capacity: capacity,
            data: Vec::with_capacity(capacity),
            label: label.to_string(),
            dirty: false,
        }
    }

    /// Updates the underlying gpu buffer with self.data.
    /// Returns true if the buffer was reallocated.
    pub fn load_to_gpu(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) -> bool {
        if !self.dirty {
            return false;
        }

        let should_realloc = self.data.len() > self.buffer_capacity;
        if should_realloc {
            self.buffer_capacity = self.data.len().next_power_of_two();
            self.buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(self.label.as_str()),
                size: (size_of::<T>() * self.buffer_capacity) as u64,
                usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        if !self.data.is_empty() {
            let size = self.data.len() * size_of::<T>();
            queue.write_buffer(&self.buffer, 0, unsafe {
                std::slice::from_raw_parts_mut(self.data[..].as_ptr() as *mut u8, size)
            });
        }

        self.dirty = false;
        should_realloc
    }

    pub fn bind_group_layout_entry(binding_index: u32) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding: binding_index,
            visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: Some(std::num::NonZeroU64::new(size_of::<T>() as u64).unwrap()),
            },
            count: None,
        }
    }

    pub fn bind_group_entry(&self, binding_index: u32) -> wgpu::BindGroupEntry<'_> {
        wgpu::BindGroupEntry {
            binding: binding_index,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer: &self.buffer,
                offset: 0,
                size: None,
            }),
        }
    }

    pub fn clear(&mut self) {
        if !self.data.is_empty() {
            self.data.clear();
            self.dirty = true;
        }
    }

    pub fn push(&mut self, value: T) {
        self.data.push(value);
        self.dirty = true;
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }
}

impl<T: Copy> Index<usize> for GpuVec<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}
impl<T: Copy> IndexMut<usize> for GpuVec<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.dirty = true;
        &mut self.data[index]
    }
}
