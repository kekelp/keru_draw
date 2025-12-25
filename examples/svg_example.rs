use keru_draw::*;
use winit::{
    dpi::PhysicalSize, event::WindowEvent, event_loop::EventLoop, window::Window,
};

// A simple SVG circle
const SVG_CIRCLE: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
  <circle cx="50" cy="50" r="40" fill="blue" stroke="white" stroke-width="3"/>
</svg>"#;

// A simple SVG star
const SVG_STAR: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
  <polygon points="50,15 61,35 82,35 67,50 73,70 50,57 27,70 33,50 18,35 39,35" fill="gold" stroke="orange" stroke-width="2"/>
</svg>"#;

// A simple SVG heart
const SVG_HEART: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" viewBox="0 0 100 100">
  <path d="M 50,30 C 40,20 25,20 20,30 C 15,40 20,50 50,80 C 80,50 85,40 80,30 C 75,20 60,20 50,30 Z" fill="red" stroke="darkred" stroke-width="2"/>
</svg>"#;

struct App {
    window: Option<std::sync::Arc<Window>>,
    state: Option<State>,
}

struct State {
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
    renderer: Renderer,
    rotation: f32,
    // SVG handles - loaded once and reused
    circle_handle: SvgHandle,
    star_handle: SvgHandle,
    heart_handle: SvgHandle,
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
                required_features: wgpu::Features::empty(),
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
            .find(|f| f.is_srgb())
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

        let mut renderer = Renderer::new(device, queue, surface_format, size.width, size.height);

        // Load SVGs once at initialization
        let circle_handle = renderer.svg_renderer.load_svg(SVG_CIRCLE, 100, 100)
            .expect("Failed to load circle SVG");
        let star_handle = renderer.svg_renderer.load_svg(SVG_STAR, 100, 100)
            .expect("Failed to load star SVG");
        let heart_handle = renderer.svg_renderer.load_svg(SVG_HEART, 100, 100)
            .expect("Failed to load heart SVG");

        Self {
            surface,
            config,
            size,
            renderer,
            rotation: 0.0,
            circle_handle,
            star_handle,
            heart_handle,
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
        // Begin frame: prepare text and clear buffers
        self.renderer.begin_frame(self.size.width as f32, self.size.height as f32);

        // Update rotation
        self.rotation += 0.01;

        // Draw background rectangle
        self.renderer.draw_rectangle(RectangleData {
            top_left: [0.0, 0.0],
            size: [self.size.width as f32, self.size.height as f32],
            color: [0.1, 0.1, 0.15],
            corner_radius: 0.0,
            x_clip: [0.0, self.size.width as f32],
            y_clip: [0.0, self.size.height as f32],
        });

        // Draw static SVGs at different positions
        self.renderer.draw_svg(&self.circle_handle, 50.0, 50.0, 100.0, 100.0, 0.5);
        self.renderer.draw_svg(&self.star_handle, 200.0, 50.0, 120.0, 120.0, 0.5);
        self.renderer.draw_svg(&self.heart_handle, 370.0, 50.0, 100.0, 100.0, 0.5);

        // Draw SVGs with different sizes (same handle, scaled differently)
        self.renderer.draw_svg(&self.circle_handle, 50.0, 200.0, 80.0, 80.0, 0.5);
        self.renderer.draw_svg(&self.circle_handle, 150.0, 200.0, 120.0, 120.0, 0.5);
        self.renderer.draw_svg(&self.circle_handle, 300.0, 200.0, 60.0, 60.0, 0.5);

        // Draw animated SVG (moving in circle)
        let center_x = self.size.width as f32 / 2.0;
        let center_y = self.size.height as f32 / 2.0 + 100.0;
        let orbit_radius = 150.0;
        let x = center_x + orbit_radius * self.rotation.cos() - 50.0;
        let y = center_y + orbit_radius * self.rotation.sin() - 50.0;
        self.renderer.draw_svg(&self.heart_handle, x, y, 100.0, 100.0, 0.5);

        // Mix SVGs with other primitives
        self.renderer.draw_ellipse(EllipseData {
            top_left: [500.0, 50.0],
            size: [100.0, 100.0],
            color: [0.5, 0.0, 0.8],
            _padding: 0.0,
            x_clip: [0.0, self.size.width as f32],
            y_clip: [0.0, self.size.height as f32],
        });

        self.renderer.draw_rectangle(RectangleData {
            top_left: [500.0, 200.0],
            size: [100.0, 100.0],
            color: [0.0, 0.8, 0.5],
            corner_radius: 20.0,
            x_clip: [0.0, self.size.width as f32],
            y_clip: [0.0, self.size.height as f32],
        });

        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.renderer.render(&view);
        output.present();

        Ok(())
    }
}

impl winit::application::ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window.is_none() {
            let window = std::sync::Arc::new(
                event_loop
                    .create_window(
                        Window::default_attributes().with_inner_size(PhysicalSize::new(800, 600)),
                    )
                    .unwrap(),
            );
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
            match event {
                WindowEvent::CloseRequested => event_loop.exit(),
                WindowEvent::Resized(physical_size) => {
                    state.resize(physical_size);
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

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
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
