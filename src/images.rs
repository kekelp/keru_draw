use etagere::{Allocation, BucketedAtlasAllocator, size2};
use image::RgbaImage;
use wgpu::*;
use bytemuck::{Pod, Zeroable};
use std::collections::HashMap;

/// A quad for rendering an image instance (SVG or raster) on the GPU.
#[repr(C)]
#[derive(Clone, Copy, Debug, Zeroable, Pod)]
pub struct ImageQuad {
    pub pos_x: f32,
    pub pos_y: f32,
    pub width: f32,
    pub height: f32,
    pub uv_x: f32,
    pub uv_y: f32,
    pub uv_width: f32,
    pub uv_height: f32,
    pub page: u32,
    pub depth: f32,
    pub _padding0: u32,
    pub _padding1: u32,
}

/// A loaded image stored in the atlas.
#[derive(Clone, Debug)]
pub struct LoadedImage {
    pub page: u16,
    pub alloc: Allocation,
    pub width: u32,
    pub height: u32,
    /// Internal ID for looking up SVG data (0 for raster images)
    pub(crate) id: u64,
}

struct AtlasPage {
    packer: BucketedAtlasAllocator,
    image: RgbaImage,
    dirty: bool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
struct Params {
    screen_resolution_width: f32,
    screen_resolution_height: f32,
    _pad1: u32,
    _pad2: u32,
}

/// Image renderer that handles both SVG files and raster images using texture atlases.
pub struct ImageRenderer {
    atlas_size: u32,
    atlas_pages: Vec<AtlasPage>,

    quads: Vec<ImageQuad>,

    pub(crate) texture_array: Texture,
    pub(crate) sampler: Sampler,
    pub(crate) params_buffer: Buffer,
    params: Params,

    pub(crate) vertex_buffer: Buffer,

    needs_gpu_sync: bool,
    needs_texture_array_rebuild: bool,

    surface_is_srgb: bool,

    /// Cache of raw SVG data for rerasterization
    svg_data_cache: HashMap<u64, Vec<u8>>,
    /// Counter for generating unique IDs
    next_id: u64,
}

const INITIAL_BUFFER_SIZE: u64 = 1024 * 4;

impl ImageRenderer {
    /// Create a new ImageRenderer with default atlas size of 2048x2048
    pub fn new(device: &Device, _queue: &Queue, surface_format: TextureFormat) -> Self {
        Self::new_with_atlas_size(device, _queue, surface_format, 2048)
    }

    /// Create a new ImageRenderer with custom atlas size
    pub fn new_with_atlas_size(device: &Device, _queue: &Queue, surface_format: TextureFormat, atlas_size: u32) -> Self {
        let surface_is_srgb = surface_format.is_srgb();

        let params = Params {
            screen_resolution_width: 800.0,
            screen_resolution_height: 600.0,
            _pad1: 0,
            _pad2: 0,
        };

        let params_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Image Params Buffer"),
            size: std::mem::size_of::<Params>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let vertex_buffer = create_vertex_buffer(device, INITIAL_BUFFER_SIZE);

        // Create initial texture array with 1 layer
        let texture_array = create_texture_array(device, atlas_size, 1, surface_is_srgb);

        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("Image Atlas Sampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });

        // Create initial atlas page
        let atlas_pages = vec![AtlasPage {
            image: RgbaImage::new(atlas_size, atlas_size),
            packer: BucketedAtlasAllocator::new(size2(atlas_size as i32, atlas_size as i32)),
            dirty: false,
        }];

        Self {
            atlas_size,
            atlas_pages,
            quads: Vec::new(),
            texture_array,
            sampler,
            params_buffer,
            params,
            vertex_buffer,
            needs_gpu_sync: false,
            needs_texture_array_rebuild: false,
            surface_is_srgb,
            svg_data_cache: HashMap::new(),
            next_id: 1,
        }
    }

    /// Update the screen resolution
    pub fn update_resolution(&mut self, width: f32, height: f32) {
        self.params.screen_resolution_width = width;
        self.params.screen_resolution_height = height;
    }

    /// Clear all quads for the next frame
    pub fn clear(&mut self) {
        self.quads.clear();
        self.needs_gpu_sync = true;
    }

    /// Load and rasterize an SVG, returning the loaded image.
    ///
    /// Returns None if the SVG couldn't be loaded/rasterized.
    pub fn load_svg(&mut self, svg_data: &[u8], width: u32, height: u32) -> Option<LoadedImage> {
        self.rasterize_and_store(svg_data, width, height)
    }

    /// Draw a previously loaded SVG.
    ///
    /// If `rerasterize` is true, the SVG will be rerasterized if the desired size
    /// (width x height) differs from the currently stored rasterization size.
    pub fn draw_svg(&mut self, loaded: &mut LoadedImage, x: f32, y: f32, width: f32, height: f32, depth: f32, rerasterize: bool) {
        // Check if rerasterization is needed
        if rerasterize && loaded.id != 0 {
            let desired_width = width.round() as u32;
            let desired_height = height.round() as u32;

            if desired_width != loaded.width || desired_height != loaded.height {
                // Get the cached SVG data
                if let Some(svg_data) = self.svg_data_cache.get(&loaded.id).cloned() {
                    // Deallocate old texture space
                    self.atlas_pages[loaded.page as usize].packer.deallocate(loaded.alloc.id);

                    // Rerasterize at new size
                    if let Some(new_loaded) = self.rasterize_and_store_with_id(&svg_data, desired_width, desired_height, loaded.id) {
                        *loaded = new_loaded;
                    }
                }
            }
        }

        self.add_quad_from_loaded_image(loaded, x, y, width, height, depth);
    }

    /// Remove a loaded SVG and free its atlas space.
    pub fn unload_svg(&mut self, loaded: &LoadedImage) {
        self.atlas_pages[loaded.page as usize].packer.deallocate(loaded.alloc.id);
        // Remove cached SVG data
        if loaded.id != 0 {
            self.svg_data_cache.remove(&loaded.id);
        }
    }

    /// Load a raster image from raw RGBA8 bytes, returning the loaded image.
    ///
    /// Returns None if the image couldn't be stored in the atlas.
    pub fn load_image(&mut self, rgba_data: &[u8], width: u32, height: u32) -> Option<LoadedImage> {
        self.store_image_data(rgba_data, width, height)
    }

    /// Load a raster image from encoded bytes (PNG, JPEG, etc.), returning the loaded image.
    ///
    /// Returns None if the image couldn't be decoded or stored.
    pub fn load_image_from_bytes(&mut self, image_data: &[u8]) -> Option<LoadedImage> {
        let img = image::load_from_memory(image_data).ok()?;
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        self.load_image(rgba.as_raw(), width, height)
    }

    /// Draw a previously loaded raster image
    pub fn draw_image(&mut self, loaded: &LoadedImage, x: f32, y: f32, width: f32, height: f32, depth: f32) {
        self.add_quad_from_loaded_image(loaded, x, y, width, height, depth);
    }

    /// Remove a loaded image and free its atlas space
    pub fn unload_image(&mut self, loaded: &LoadedImage) {
        self.atlas_pages[loaded.page as usize].packer.deallocate(loaded.alloc.id);
    }

    /// Get the quads for external rendering
    pub fn quads(&self) -> &[ImageQuad] {
        &self.quads
    }

    /// Upload all quads and textures to GPU
    pub(crate) fn load_to_gpu(&mut self, device: &Device, queue: &Queue) -> bool {
        if !self.needs_gpu_sync && !self.needs_texture_array_rebuild {
            return false;
        }

        // Update params buffer
        let bytes: &[u8] = bytemuck::cast_slice(std::slice::from_ref(&self.params));
        queue.write_buffer(&self.params_buffer, 0, bytes);

        // Rebuild or update texture array
        if self.needs_texture_array_rebuild {
            self.rebuild_texture_array(device, queue);
            self.needs_texture_array_rebuild = false;
        } else {
            self.update_texture_array(queue);
        }

        // Update vertex buffer
        let required_size = (self.quads.len() * std::mem::size_of::<ImageQuad>()) as u64;
        if self.vertex_buffer.size() < required_size {
            let new_size = (required_size * 3 / 2).max(INITIAL_BUFFER_SIZE);
            self.vertex_buffer = create_vertex_buffer(device, new_size);
        }

        if !self.quads.is_empty() {
            let bytes: &[u8] = bytemuck::cast_slice(&self.quads);
            queue.write_buffer(&self.vertex_buffer, 0, bytes);
        }

        self.needs_gpu_sync = false;
        self.needs_texture_array_rebuild
    }

    // Internal methods

    fn add_quad_from_loaded_image(
        &mut self,
        loaded: &LoadedImage,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        depth: f32,
    ) {
        let uv_x = loaded.alloc.rectangle.min.x as f32;
        let uv_y = loaded.alloc.rectangle.min.y as f32;
        let uv_width = loaded.width as f32;
        let uv_height = loaded.height as f32;

        self.quads.push(ImageQuad {
            pos_x: x,
            pos_y: y,
            width,
            height,
            uv_x,
            uv_y,
            uv_width,
            uv_height,
            page: loaded.page as u32,
            depth,
            _padding0: 0,
            _padding1: 0,
        });

        self.needs_gpu_sync = true;
    }

    fn rasterize_and_store(&mut self, svg_data: &[u8], width: u32, height: u32) -> Option<LoadedImage> {
        // Generate unique ID and store SVG data
        let id = self.next_id;
        self.next_id += 1;
        self.svg_data_cache.insert(id, svg_data.to_vec());

        self.rasterize_and_store_with_id(svg_data, width, height, id)
    }

    fn rasterize_and_store_with_id(&mut self, svg_data: &[u8], width: u32, height: u32, id: u64) -> Option<LoadedImage> {
        // Parse SVG
        let opt = usvg::Options::default();
        let tree = usvg::Tree::from_data(svg_data, &opt).ok()?;

        // Create pixmap for rasterization
        // we could keep a scratch buffer but it probably happens rarely enough that it's better to keep the memory. maybe
        let mut pixmap = tiny_skia::Pixmap::new(width, height)?;

        // Calculate scale to fit SVG in requested dimensions
        let svg_size = tree.size();
        let scale_x = width as f32 / svg_size.width();
        let scale_y = height as f32 / svg_size.height();
        let scale = scale_x.min(scale_y);

        let transform = tiny_skia::Transform::from_scale(scale, scale);

        // Render
        resvg::render(&tree, transform, &mut pixmap.as_mut());

        // Get raw RGBA data
        let rgba_data = pixmap.take();

        self.store_image_data_with_id(&rgba_data, width, height, id)
    }

    fn store_image_data(
        &mut self,
        rgba_data: &[u8],
        width: u32,
        height: u32,
    ) -> Option<LoadedImage> {
        self.store_image_data_with_id(rgba_data, width, height, 0)
    }

    fn store_image_data_with_id(
        &mut self,
        rgba_data: &[u8],
        width: u32,
        height: u32,
        id: u64,
    ) -> Option<LoadedImage> {
        // Verify data size
        if rgba_data.len() != (width * height * 4) as usize {
            return None;
        }

        // Try to allocate in existing pages
        for page_idx in 0..self.atlas_pages.len() {
            if let Some(alloc) = self.atlas_pages[page_idx].packer.allocate(size2(width as i32, height as i32)) {
                return Some(self.store_in_atlas(rgba_data, alloc, page_idx, width, height, id));
            }
        }

        // Create new page
        let new_page_idx = self.make_new_page();
        if let Some(alloc) = self.atlas_pages[new_page_idx].packer.allocate(size2(width as i32, height as i32)) {
            return Some(self.store_in_atlas(rgba_data, alloc, new_page_idx, width, height, id));
        }

        // Image too large even for new page
        None
    }

    fn store_in_atlas(
        &mut self,
        rgba_data: &[u8],
        alloc: Allocation,
        page_idx: usize,
        width: u32,
        height: u32,
        id: u64,
    ) -> LoadedImage {
        // Copy image data to atlas
        let dst_x = alloc.rectangle.min.x as u32;
        let dst_y = alloc.rectangle.min.y as u32;

        for y in 0..height {
            for x in 0..width {
                let offset = ((y * width + x) * 4) as usize;
                let pixel = image::Rgba([
                    rgba_data[offset],
                    rgba_data[offset + 1],
                    rgba_data[offset + 2],
                    rgba_data[offset + 3],
                ]);
                self.atlas_pages[page_idx].image.put_pixel(dst_x + x, dst_y + y, pixel);
            }
        }

        self.atlas_pages[page_idx].dirty = true;

        LoadedImage {
            page: page_idx as u16,
            alloc,
            width,
            height,
            id,
        }
    }

    fn make_new_page(&mut self) -> usize {
        self.atlas_pages.push(AtlasPage {
            image: RgbaImage::new(self.atlas_size, self.atlas_size),
            packer: BucketedAtlasAllocator::new(size2(self.atlas_size as i32, self.atlas_size as i32)),
            dirty: true,
        });
        self.needs_texture_array_rebuild = true;
        self.atlas_pages.len() - 1
    }

    fn rebuild_texture_array(&mut self, device: &Device, queue: &Queue) {
        let num_pages = self.atlas_pages.len().max(1);

        self.texture_array = create_texture_array(device, self.atlas_size, num_pages as u32, self.surface_is_srgb);

        // Upload all pages
        for (page_idx, page) in self.atlas_pages.iter_mut().enumerate() {
            upload_texture_page(queue, &self.texture_array, &page.image, page_idx as u32, self.atlas_size);
            page.dirty = false;
        }
    }

    fn update_texture_array(&mut self, queue: &Queue) {
        for (page_idx, page) in self.atlas_pages.iter_mut().enumerate() {
            if page.dirty {
                upload_texture_page(queue, &self.texture_array, &page.image, page_idx as u32, self.atlas_size);
                page.dirty = false;
            }
        }
    }
}

fn create_vertex_buffer(device: &Device, size: u64) -> Buffer {
    device.create_buffer(&BufferDescriptor {
        label: Some("Image Vertex Buffer"),
        size,
        usage: BufferUsages::VERTEX | BufferUsages::STORAGE | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn create_texture_array(device: &Device, size: u32, layers: u32, surface_is_srgb: bool) -> Texture {

    let format = if surface_is_srgb {
        TextureFormat::Rgba8UnormSrgb
    } else {
        TextureFormat::Rgba8Unorm
    };

    device.create_texture(&TextureDescriptor {
        label: Some("Image Atlas Texture Array"),
        size: Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: layers,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format,
        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        view_formats: &[],
    })
}

fn upload_texture_page(queue: &Queue, texture: &Texture, image: &RgbaImage, layer: u32, size: u32) {
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d { z: layer, ..Default::default() },
            aspect: wgpu::TextureAspect::All,
        },
        image.as_raw(),
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * size),
            rows_per_image: None,
        },
        wgpu::Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
    );
}
