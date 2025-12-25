use keru_draw::*;
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
    svg_handle: SvgHandle,
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

        let mut renderer = Renderer::new(device, queue, surface_format);

        let text_edit = renderer.text.add_text_edit(
            "Hello from keru_renderer!\nText rendering with clipped quads.".to_owned(),
            (100.0, 100.0),
            (300.0, 200.0),
            0.0,
        );

        let svg_handle = renderer.svg_renderer.load_svg(TIGER_SVG, 200, 200).unwrap();

        Self {
            surface,
            config,
            size,
            renderer,
            text_edit,
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
        // Begin frame: prepare text and clear buffers
        self.renderer.begin_frame(self.size.width as f32, self.size.height as f32);

        // Draw shapes for this frame
        self.renderer.draw_rectangle(RectangleData {
            top_left: [0.0, 0.0],
            size: [400.0, 400.0],
            color: [1.0, 0.0, 0.0],
            corner_radius: 60.0,
            x_clip: [0.0, 4000.0],
            y_clip: [30.0, 8000.0],
        });

        self.renderer.draw_ellipse(EllipseData {
            top_left: [400.0, 400.0],
            size: [400.0, 400.0],
            color: [0.0, 1.0, 0.0],
            _padding: 0.0,
            x_clip: [0.0, 750.0],
            y_clip: [50.0, 5000.0],
        });

        // Draw retained text box
        self.renderer.draw_text_edit(&self.text_edit);

        // Draw tiger SVG
        self.renderer.draw_svg(&self.svg_handle, 450.0, 350.0, 200.0, 200.0, 0.5);

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
                        Window::default_attributes().with_inner_size(PhysicalSize::new(800, 800)),
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
            // Register window with text system
            state.renderer.text_mut().handle_event(&event, window);

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
