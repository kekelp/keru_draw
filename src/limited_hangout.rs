use crate::*;

impl Renderer {
    /// Get a restricted drawing context that only exposes the drawing methods, but not the methods like [`Self::begin_frame()`] and [`Self::clear_for_new_frame()`].
    pub fn get_draw_context(&mut self) -> DrawContext<'_> {
        DrawContext { renderer: self }
    }
}


/// A context for custom drawing.
/// 
/// This is a limited version of the `keru_draw` [`Renderer`]
pub struct DrawContext<'a> {
    renderer: &'a mut Renderer,
}

impl<'a> DrawContext<'a> {
    /// Draw a box/rectangle.
    pub fn draw_box(&mut self, params: Box) {
        self.renderer.draw_box(params);
    }

    /// Draw an image.
    pub fn draw_image(
        &mut self,
        image: LoadedImage,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        x_clip: [f32; 2],
        y_clip: [f32; 2],
    ) {
        self.renderer.draw_image(image, x, y, width, height, x_clip, y_clip);
    }

    /// Draw a filled circle.
    pub fn draw_circle(&mut self, params: Circle) {
        self.renderer.draw_circle(params);
    }

    /// Draw a ring (hollow circle).
    pub fn draw_ring(&mut self, params: CircleRing) {
        self.renderer.draw_ring(params);
    }

    /// Draw an arc.
    pub fn draw_arc(&mut self, params: CircleArc) {
        self.renderer.draw_arc(params);
    }

    /// Draw a pie slice.
    pub fn draw_pie(&mut self, params: CirclePie) {
        self.renderer.draw_pie(params);
    }

    /// Draw a line segment.
    pub fn draw_segment(&mut self, params: Segment) {
        self.renderer.draw_segment(params);
    }

    /// Draw a grid.
    pub fn draw_grid(&mut self, params: Grid) {
        self.renderer.draw_grid(params);
    }

    /// Draw a triangle.
    pub fn draw_triangle(&mut self, params: Triangle) {
        self.renderer.draw_triangle(params);
    }

    /// Draw a hexagon.
    pub fn draw_hexagon(&mut self, params: Hexagon) {
        self.renderer.draw_hexagon(params);
    }

    /// Draw a quadratic bezier curve.
    pub fn draw_quadratic_bezier(&mut self, params: QuadraticBezier) {
        self.renderer.draw_quadratic_bezier(params);
    }

    /// Draw a dashed box outline.
    pub fn draw_dashed_box_outline(&mut self, params: DashedBoxOutline) {
        self.renderer.draw_dashed_box_outline(params);
    }

    /// Draw a dashed hexagon outline.
    pub fn draw_dashed_hexagon_outline(&mut self, params: DashedHexagonOutline) {
        self.renderer.draw_dashed_hexagon_outline(params);
    }

    /// Set a transform for subsequent draw calls.
    pub fn set_transform(&mut self, transform: Transform) {
        let handle = self.renderer.insert_transform(transform);
        self.renderer.set_current_transform(handle);
    }

    /// Clear the current transform, resetting to identity.
    pub fn clear_transform(&mut self) {
        self.renderer.clear_current_transform();
    }
}
