use std::{borrow::Cow, f32::consts::PI};

use keru_draw::*;
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
            "Using rotation or zoom on text and SVGs can look quite disappointing, because they are pre-rasterized on the CPU.Using rotation or zoom on text and SVGs can look quite disappointing, because they are pre-rasterized on the CPU.Using rotation or zoom on text and SVGs can look quite disappointing, because they are pre-rasterized on the CPU.Using rotation or zoom on text and SVGs can look quite disappointing, because they are pre-rasterized on the CPU.Using rotation or zoom on text and SVGs can look quite disappointing, because they are pre-rasterized on the CPU.Using rotation or zoom on text and SVGs can look quite disappointing, because they are pre-rasterized on the CPU.".to_owned(),
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

        let svg_handle = renderer.image_renderer.load_svg(TIGER_SVG, 200, 200).unwrap();

        Self {
            surface,
            config,
            size,
            renderer,
            text_edit,
            text_box1,
            text_box2,
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
        self.renderer.draw_box_gradient(
            [20.0, 20.0],
            [80.0, 60.0],
            0.0,
            0.0,
            [1.0, 0.3, 0.3, 1.0],
            [0.3, 0.3, 1.0, 1.0],
            0.0, // horizontal gradient
            clip_x,
            clip_y,
        );

        // Gradient box - diagonal
        self.renderer.draw_box_gradient(
            [120.0, 20.0],
            [80.0, 60.0],
            5.0,
            0.0,
            [1.0, 0.5, 0.3, 1.0],
            [0.3, 1.0, 0.5, 1.0],
            std::f32::consts::PI * 0.25, // 45 degrees
            clip_x,
            clip_y,
        );

        self.renderer.draw_box(
            [220.0, 20.0],
            [80.0, 60.0],
            16.0,
            5.0,
            [0.015686, 0.666667, 0.427451, 1.0],
            clip_x,
            clip_y,
        );

        self.renderer.draw_box(
            [320.0, 20.0],
            [80.0, 60.0],
            30.0,
            5.0,
            [0.8, 1.0, 0.3, 1.0],
            clip_x,
            clip_y,
        );

        // Radial gradient circle
        self.renderer.draw_circle_gradient(
            [50.0, 150.0],
            20.0,
            [1.0, 1.0, 0.3, 1.0],
            [1.0, 0.3, 0.3, 1.0],
            2, // radial
            0.0,
            clip_x,
            clip_y,
        );

        // Linear gradient circle
        self.renderer.draw_circle_gradient(
            [140.0, 150.0],
            30.0,
            [0.3, 0.7, 1.0, 1.0],
            [1.0, 0.3, 0.7, 1.0],
            1, // linear
            std::f32::consts::PI * 0.5, // vertical
            clip_x,
            clip_y,
        );

        self.renderer.draw_circle(
            [250.0, 150.0],
            40.0,
            [0.3, 0.9, 1.0, 1.0],
            clip_x,
            clip_y,
        );

        self.renderer.draw_ring(
            [360.0, 150.0],
            35.0,
            40.0,
            [1.0, 1.0, 0.3, 1.0],
            clip_x,
            clip_y,
        );

        self.renderer.draw_ring(
            [460.0, 150.0],
            30.0,
            45.0,
            [1.0, 0.8, 0.3, 1.0],
            clip_x,
            clip_y,
        );

        self.renderer.draw_ring(
            [560.0, 150.0],
            25.0,
            50.0,
            [1.0, 0.6, 0.3, 1.0],
            clip_x,
            clip_y,
        );

        self.renderer.draw_arc(
            [60.0, 280.0],
            40.0,
            0.0,
            std::f32::consts::PI * 0.5,
            8.0,
            [1.0, 0.3, 1.0, 1.0],
            clip_x,
            clip_y,
        );

        self.renderer.draw_arc(
            [170.0, 280.0],
            40.0,
            0.0,
            std::f32::consts::PI,
            8.0,
            [0.8, 0.3, 1.0, 1.0],
            clip_x,
            clip_y,
        );

        self.renderer.draw_arc(
            [280.0, 280.0],
            40.0,
            0.0,
            std::f32::consts::PI * 1.5,
            8.0,
            [0.6, 0.3, 1.0, 1.0],
            clip_x,
            clip_y,
        );

        self.renderer.draw_arc(
            [390.0, 280.0],
            40.0,
            std::f32::consts::PI * 0.25,
            std::f32::consts::PI * 1.25,
            8.0,
            [0.4, 0.3, 1.0, 1.0],
            clip_x,
            clip_y,
        );

        self.renderer.draw_pie(
            [60.0, 400.0],
            45.0,
            0.0,
            std::f32::consts::PI * 0.25,
            [0.3, 1.0, 1.0, 1.0],
            clip_x,
            clip_y,
        );

        self.renderer.draw_pie(
            [170.0, 400.0],
            45.0,
            0.0,
            std::f32::consts::PI * 0.5,
            [0.3, 1.0, 0.8, 1.0],
            clip_x,
            clip_y,
        );

        self.renderer.draw_pie(
            [280.0, 400.0],
            45.0,
            0.0,
            std::f32::consts::PI,
            [0.3, 1.0, 0.6, 1.0],
            clip_x,
            clip_y,
        );

        self.renderer.draw_pie(
            [390.0, 400.0],
            45.0,
            std::f32::consts::PI * 0.5,
            std::f32::consts::PI * 2.0,
            [0.3, 1.0, 0.4, 1.0],
            clip_x,
            clip_y,
        );

        self.renderer.draw_segment(
            [20.0, 520.0],
            [100.0, 520.0],
            3.0,
            [1.0, 0.5, 0.0, 1.0],
            clip_x,
            clip_y,
            None,
        );

        self.renderer.draw_segment(
            [120.0, 500.0],
            [200.0, 540.0],
            6.0,
            [1.0, 0.6, 0.0, 1.0],
            clip_x,
            clip_y,
            Some(10.0),
        );

        self.renderer.draw_segment(
            [230.0, 500.0],
            [230.0, 540.0],
            10.0,
            [1.0, 0.7, 0.0, 1.0],
            clip_x,
            clip_y,
            Some(15.0),
        );

        self.renderer.draw_segment(
            [260.0, 540.0],
            [340.0, 500.0],
            8.0,
            [1.0, 0.8, 0.0, 1.0],
            clip_x,
            clip_y,
            Some(5.0),
        );

        // Gradient segments forming an X
        self.renderer.draw_segment_gradient(
            [370.0, 500.0],
            [430.0, 540.0],
            5.0,
            [1.0, 0.9, 0.2, 0.7],
            [0.2, 0.9, 1.0, 0.7],
            clip_x,
            clip_y,
            Some(8.0),
        );
        self.renderer.draw_segment_gradient(
            [370.0, 540.0],
            [430.0, 500.0],
            5.0,
            [1.0, 0.2, 0.9, 0.8],
            [0.2, 1.0, 0.9, 0.8],
            clip_x,
            clip_y,
            None,
        );

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
