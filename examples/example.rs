use std::{borrow::Cow, f32::consts::PI};

use keru_draw::*;
use keru_draw::{GridType, GradientType, Fill};
use textslabs::{ColorBrush, TextStyle2, Transform2D, parley::{FontFamily, FontStack}};
use winit::{
    dpi::PhysicalSize, event::WindowEvent, event_loop::EventLoop, window::Window,
};

const TIGER_SVG: &[u8] = include_bytes!("tiger.svg");

struct App {
    window: Option<std::sync::Arc<Window>>,
    state: Option<State>,
}

struct State {
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
    renderer: Renderer,
    text_edit: TextEditHandle,
    text_box1: TextBoxHandle,
    text_box2: TextBoxHandle,
    text_box3: TextBoxHandle,
    svg_handle: LoadedImage,
}

impl State {
    async fn new(window: std::sync::Arc<Window>) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::TIMESTAMP_QUERY | wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS,
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::default(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                trace: wgpu::Trace::Off,
            })
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| ! f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let mut renderer = Renderer::new(device, queue.clone(), surface_format);

        let style = renderer.text.add_style(
            TextStyle2 {
                brush: ColorBrush([0, 0, 0, 255]),
                font_stack: FontStack::Single(FontFamily::Named(Cow::Borrowed("sans-serif"))),
                ..Default::default()
            },
            None,
        );
        let text_edit = renderer.text.add_text_edit(
            "🌈Bottom text".to_owned(),
            (500.0, 400.0),
            (280.0, 150.0),
            0.0,
        );
        renderer.text.get_text_edit_mut(&text_edit).set_style(&style);

        let text_box1 = renderer.text.add_text_box(
            "Using rotation or zoom on text and SVGs can look quite disappointing, because they are pre-rasterized on the CPU.".to_owned(),
            (10.0, 530.0),
            (200.0, 50.0),
            0.0,
        );
        renderer.text.get_text_box_mut(&text_box1).set_style(&style);
        renderer.text.get_text_box_mut(&text_box1).set_transform(Transform2D {
            translation: (20.0, 530.0),
            rotation: std::f32::consts::PI * -0.15,
            scale: 1.5,
        });

        let text_box2 = renderer.text.add_text_box(
            "90 degree rotation".to_owned(),
            (0.0, 0.0),
            (200.0, 60.0),
            0.0,
        );
        renderer.text.get_text_box_mut(&text_box2).set_style(&style);
        renderer.text.get_text_box_mut(&text_box2).set_transform(Transform2D {
            translation: (400.0, 550.0),
            rotation: std::f32::consts::PI * 0.5,
            scale: 1.0,
        });

        let text_box3 = renderer.text.add_text_box(
            "What kind of renderer doesn't have a hex grid primitive?".to_owned(),
            (750.0, 600.0),
            (250.0, 100.0),
            0.0,
        );
        renderer.text.get_text_box_mut(&text_box3).set_style(&style);


        let svg_handle = renderer.image_renderer.load_svg(TIGER_SVG, 200, 200).unwrap();

        Self {
            surface,
            config,
            size,
            renderer,
            text_edit,
            text_box1,
            text_box2,
            text_box3,
            svg_handle,
        }
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(self.renderer.device(), &self.config);
            self.renderer.resize(new_size.width, new_size.height);
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let width = self.size.width as f32;
        let height = self.size.height as f32;
        let clip_x = [0.0, width];
        let clip_y = [0.0, height];

        self.renderer.begin_frame(width, height);

        // Gradient box - horizontal
        self.renderer.draw_box(Box {
            top_left: [20.0, 20.0],
            size: [80.0, 60.0],
            corner_radius: 0.0,
            border_thickness: 0.0,
            fill: Fill::Gradient {
                color_start: [1.0, 0.3, 0.3, 1.0],
                color_end: [0.3, 0.3, 1.0, 1.0],
                gradient_type: GradientType::Linear,
                angle: 0.0,
            },
            x_clip: clip_x,
            y_clip: clip_y,
        });

        // Gradient box - diagonal
        self.renderer.draw_box(Box {
            top_left: [120.0, 20.0],
            size: [80.0, 60.0],
            corner_radius: 5.0,
            border_thickness: 0.0,
            fill: Fill::Gradient {
                color_start: [1.0, 0.5, 0.3, 1.0],
                color_end: [0.3, 1.0, 0.5, 1.0],
                gradient_type: GradientType::Linear,
                angle: std::f32::consts::PI * 0.25,
            },
            x_clip: clip_x,
            y_clip: clip_y,
        });

        self.renderer.draw_box(Box {
            top_left: [220.0, 20.0],
            size: [80.0, 60.0],
            corner_radius: 16.0,
            border_thickness: 5.0,
            fill: Fill::Solid([0.015686, 0.666667, 0.427451, 1.0]),
            x_clip: clip_x,
            y_clip: clip_y,
        });

        self.renderer.draw_box(Box {
            top_left: [320.0, 20.0],
            size: [80.0, 60.0],
            corner_radius: 30.0,
            border_thickness: 5.0,
            fill: Fill::Solid([0.8, 1.0, 0.3, 1.0]),
            x_clip: clip_x,
            y_clip: clip_y,
        });

        // Radial gradient circle
        self.renderer.draw_circle(Circle {
            center: [50.0, 150.0],
            radius: 20.0,
            fill: Fill::Gradient {
                color_start: [1.0, 1.0, 0.3, 1.0],
                color_end: [1.0, 0.3, 0.3, 1.0],
                gradient_type: GradientType::Radial,
                angle: 0.0,
            },
            x_clip: clip_x,
            y_clip: clip_y,
        });

        // Linear gradient circle
        self.renderer.draw_circle(Circle {
            center: [140.0, 150.0],
            radius: 30.0,
            fill: Fill::Gradient {
                color_start: [0.3, 0.7, 1.0, 1.0],
                color_end: [1.0, 0.3, 0.7, 1.0],
                gradient_type: GradientType::Linear,
                angle: std::f32::consts::PI * 0.5,
            },
            x_clip: clip_x,
            y_clip: clip_y,
        });

        self.renderer.draw_circle(Circle {
            center: [250.0, 150.0],
            radius: 40.0,
            fill: Fill::Solid([0.3, 0.9, 1.0, 1.0]),
            x_clip: clip_x,
            y_clip: clip_y,
        });

        self.renderer.draw_ring(Ring {
            center: [360.0, 150.0],
            inner_radius: 35.0,
            outer_radius: 40.0,
            fill: Fill::Solid([1.0, 1.0, 0.3, 1.0]),
            x_clip: clip_x,
            y_clip: clip_y,
        });

        self.renderer.draw_ring(Ring {
            center: [460.0, 150.0],
            inner_radius: 30.0,
            outer_radius: 45.0,
            fill: Fill::Solid([1.0, 0.8, 0.3, 1.0]),
            x_clip: clip_x,
            y_clip: clip_y,
        });

        self.renderer.draw_ring(Ring {
            center: [560.0, 150.0],
            inner_radius: 25.0,
            outer_radius: 50.0,
            fill: Fill::Solid([1.0, 0.6, 0.3, 1.0]),
            x_clip: clip_x,
            y_clip: clip_y,
        });

        self.renderer.draw_arc(Arc {
            center: [60.0, 280.0],
            radius: 40.0,
            start_angle: 0.0,
            end_angle: std::f32::consts::PI * 0.5,
            thickness: 8.0,
            fill: Fill::Solid([1.0, 0.3, 1.0, 1.0]),
            x_clip: clip_x,
            y_clip: clip_y,
        });

        self.renderer.draw_arc(Arc {
            center: [170.0, 280.0],
            radius: 40.0,
            start_angle: 0.0,
            end_angle: std::f32::consts::PI,
            thickness: 8.0,
            fill: Fill::Solid([0.8, 0.3, 1.0, 1.0]),
            x_clip: clip_x,
            y_clip: clip_y,
        });

        self.renderer.draw_arc(Arc {
            center: [280.0, 280.0],
            radius: 40.0,
            start_angle: 0.0,
            end_angle: std::f32::consts::PI * 1.5,
            thickness: 8.0,
            fill: Fill::Solid([0.6, 0.3, 1.0, 1.0]),
            x_clip: clip_x,
            y_clip: clip_y,
        });

        self.renderer.draw_arc(Arc {
            center: [390.0, 280.0],
            radius: 40.0,
            start_angle: std::f32::consts::PI * 0.25,
            end_angle: std::f32::consts::PI * 1.25,
            thickness: 8.0,
            fill: Fill::Solid([0.4, 0.3, 1.0, 1.0]),
            x_clip: clip_x,
            y_clip: clip_y,
        });

        self.renderer.draw_pie(Pie {
            center: [60.0, 400.0],
            radius: 45.0,
            start_angle: 0.0,
            end_angle: std::f32::consts::PI * 0.25,
            fill: Fill::Solid([0.3, 1.0, 1.0, 1.0]),
            x_clip: clip_x,
            y_clip: clip_y,
        });

        self.renderer.draw_pie(Pie {
            center: [170.0, 400.0],
            radius: 45.0,
            start_angle: 0.0,
            end_angle: std::f32::consts::PI * 0.5,
            fill: Fill::Solid([0.3, 1.0, 0.8, 1.0]),
            x_clip: clip_x,
            y_clip: clip_y,
        });

        self.renderer.draw_pie(Pie {
            center: [280.0, 400.0],
            radius: 45.0,
            start_angle: 0.0,
            end_angle: std::f32::consts::PI,
            fill: Fill::Solid([0.3, 1.0, 0.6, 1.0]),
            x_clip: clip_x,
            y_clip: clip_y,
        });

        self.renderer.draw_pie(Pie {
            center: [390.0, 400.0],
            radius: 45.0,
            start_angle: std::f32::consts::PI * 0.5,
            end_angle: std::f32::consts::PI * 2.0,
            fill: Fill::Solid([0.3, 1.0, 0.4, 1.0]),
            x_clip: clip_x,
            y_clip: clip_y,
        });

        self.renderer.draw_segment(Segment {
            start: [20.0, 520.0],
            end: [100.0, 520.0],
            thickness: 3.0,
            fill: Fill::Solid([1.0, 0.5, 0.0, 1.0]),
            x_clip: clip_x,
            y_clip: clip_y,
            dash_length: None,
        });

        self.renderer.draw_segment(Segment {
            start: [120.0, 500.0],
            end: [200.0, 540.0],
            thickness: 6.0,
            fill: Fill::Solid([1.0, 0.6, 0.0, 1.0]),
            x_clip: clip_x,
            y_clip: clip_y,
            dash_length: Some(10.0),
        });

        self.renderer.draw_segment(Segment {
            start: [230.0, 500.0],
            end: [230.0, 540.0],
            thickness: 10.0,
            fill: Fill::Solid([1.0, 0.7, 0.0, 1.0]),
            x_clip: clip_x,
            y_clip: clip_y,
            dash_length: Some(15.0),
        });

        self.renderer.draw_segment(Segment {
            start: [260.0, 540.0],
            end: [340.0, 500.0],
            thickness: 8.0,
            fill: Fill::Solid([1.0, 0.8, 0.0, 1.0]),
            x_clip: clip_x,
            y_clip: clip_y,
            dash_length: Some(5.0),
        });

        // Gradient segments forming an X
        self.renderer.draw_segment(Segment {
            start: [370.0, 500.0],
            end: [430.0, 540.0],
            thickness: 5.0,
            fill: Fill::Gradient {
                color_start: [1.0, 0.9, 0.2, 0.7],
                color_end: [0.2, 0.9, 1.0, 0.7],
                gradient_type: GradientType::Linear,
                angle: 0.0, // angle is ignored for segments
            },
            x_clip: clip_x,
            y_clip: clip_y,
            dash_length: Some(8.0),
        });
        self.renderer.draw_segment(Segment {
            start: [370.0, 540.0],
            end: [430.0, 500.0],
            thickness: 5.0,
            fill: Fill::Gradient {
                color_start: [1.0, 0.2, 0.9, 0.8],
                color_end: [0.2, 1.0, 0.9, 0.8],
                gradient_type: GradientType::Linear,
                angle: 0.0, // angle is ignored for segments
            },
            x_clip: clip_x,
            y_clip: clip_y,
            dash_length: None,
        });

        // Square grid
        self.renderer.draw_grid(Grid {
            top_left: [750.0, 20.0],
            size: [200.0, 200.0],
            lattice_size: 20.0,
            offset: [0.0, 0.0],
            line_thickness: 1.5,
            color: [0.5, 0.5, 1.0, 0.5],
            grid_type: GridType::Square,
            x_clip: clip_x,
            y_clip: clip_y,
        });

        // Hexagonal grid
        self.renderer.draw_grid(Grid {
            top_left: [750.0, 250.0],
            size: [300.0, 300.0],
            lattice_size: 30.0,
            offset: [0.0, 0.0],
            line_thickness: 2.0,
            color: [1.0, 0.0, 0.0, 0.5],
            grid_type: GridType::Hexagonal,
            x_clip: clip_x,
            y_clip: clip_y,
        });

        self.renderer.draw_text_box(&self.text_box3);
        
        self.renderer.draw_text_edit(&self.text_edit);

        self.renderer.draw_image(&self.svg_handle, 520.0, 150.0, 180.0, 180.0, 0.5);

        // Rotated text
        self.renderer.text.get_text_box_mut(&self.text_box1).set_screen_space_clip_rect(Some((0.0, 0.0, 100.0, 1000000.0)));
        self.renderer.draw_text_box(&self.text_box1);
        
        self.renderer.draw_text_box(&self.text_box2);

        self.renderer.push_transform(
            Transform::translation(600.0, 630.0)
                .then_scale(1.25, 1.25)
                .then_rotate(euclid::Angle::radians(PI * 0.3))
        );

        self.renderer.draw_image(&self.svg_handle, -50.0, -50.0, 100.0, 100.0, 0.5);

        self.renderer.pop_transform();
        
        self.renderer.prepare_text_decorations();

        self.renderer.autorender(&self.surface, wgpu::Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 });

        Ok(())
    }
}

impl winit::application::ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window.is_none() {
            let window = std::sync::Arc::new(
                event_loop
                    .create_window(
                        Window::default_attributes()
                            .with_inner_size(PhysicalSize::new(1200, 800))
                            .with_title("example"),
                    )
                    .unwrap(),
            );
            window.set_ime_allowed(true);
            let state = pollster::block_on(State::new(window.clone()));
            self.window = Some(window);
            self.state = Some(state);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        if let (Some(window), Some(state)) = (&self.window, &mut self.state) {
            // Register window with text system
            state.renderer.text.handle_event(&event, window);

            match event {
                WindowEvent::CloseRequested => event_loop.exit(),
                WindowEvent::Resized(physical_size) => {
                    state.resize(physical_size);
                    window.request_redraw();
                }
                WindowEvent::RedrawRequested => {
                    match state.render() {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                        Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                        Err(e) => eprintln!("{:?}", e),
                    }
                    window.request_redraw();
                }
                _ => {}
            }
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();

    let mut app = App {
        window: None,
        state: None,
    };

    event_loop.run_app(&mut app).unwrap();
}
