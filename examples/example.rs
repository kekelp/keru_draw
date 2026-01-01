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
            "Bottom text o algo".to_owned(),
            (500.0, 400.0),
            (280.0, 150.0),
            0.0,
        );

        let svg_handle = renderer.image_renderer.load_svg(TIGER_SVG, 200, 200).unwrap();

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
        let width = self.size.width as f32;
        let height = self.size.height as f32;
        let clip_x = [0.0, width];
        let clip_y = [0.0, height];

        // Begin frame: prepare text and clear buffers
        self.renderer.begin_frame(width, height);

        // Draw various shapes to demonstrate the new shapes API

        // === ROW 1: Boxes with different corner radii ===
        // Sharp corners (filled)
        self.renderer.draw_box(
            [20.0, 20.0],
            [80.0, 60.0],
            0.0,
            0.0,
            [1.0, 0.3, 0.3, 1.0],
            clip_x,
            clip_y,
        );

        // Small rounded corners (filled)
        self.renderer.draw_box(
            [120.0, 20.0],
            [80.0, 60.0],
            5.0,
            0.0,
            [1.0, 0.5, 0.3, 1.0],
            clip_x,
            clip_y,
        );

        // Medium rounded corners (border only)
        self.renderer.draw_box(
            [220.0, 20.0],
            [80.0, 60.0],
            15.0,
            3.0,
            [1.0, 0.8, 0.3, 1.0],
            clip_x,
            clip_y,
        );

        // Very rounded (pill shape, border only)
        self.renderer.draw_box(
            [320.0, 20.0],
            [80.0, 60.0],
            30.0,
            5.0,
            [0.8, 1.0, 0.3, 1.0],
            clip_x,
            clip_y,
        );

        // === ROW 2: Circles of different sizes ===
        self.renderer.draw_circle(
            [50.0, 150.0],
            20.0,
            [0.3, 0.5, 1.0, 1.0],
            clip_x,
            clip_y,
        );

        self.renderer.draw_circle(
            [140.0, 150.0],
            30.0,
            [0.3, 0.7, 1.0, 1.0],
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

        // === ROW 2 continued: Rings with different thicknesses ===
        // Thin ring
        self.renderer.draw_ring(
            [360.0, 150.0],
            35.0,
            40.0,
            [1.0, 1.0, 0.3, 1.0],
            clip_x,
            clip_y,
        );

        // Medium ring
        self.renderer.draw_ring(
            [460.0, 150.0],
            30.0,
            45.0,
            [1.0, 0.8, 0.3, 1.0],
            clip_x,
            clip_y,
        );

        // Thick ring
        self.renderer.draw_ring(
            [560.0, 150.0],
            25.0,
            50.0,
            [1.0, 0.6, 0.3, 1.0],
            clip_x,
            clip_y,
        );

        // === ROW 3: Arcs at different angles ===
        // Quarter arc (90 degrees)
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

        // Half arc (180 degrees)
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

        // Three-quarter arc (270 degrees)
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

        // Rotated arc
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

        // === ROW 4: Pie slices at different angles ===
        // Small pie (45 degrees)
        self.renderer.draw_pie(
            [60.0, 400.0],
            45.0,
            0.0,
            std::f32::consts::PI * 0.25,
            [0.3, 1.0, 1.0, 1.0],
            clip_x,
            clip_y,
        );

        // Quarter pie (90 degrees)
        self.renderer.draw_pie(
            [170.0, 400.0],
            45.0,
            0.0,
            std::f32::consts::PI * 0.5,
            [0.3, 1.0, 0.8, 1.0],
            clip_x,
            clip_y,
        );

        // Half pie (180 degrees)
        self.renderer.draw_pie(
            [280.0, 400.0],
            45.0,
            0.0,
            std::f32::consts::PI,
            [0.3, 1.0, 0.6, 1.0],
            clip_x,
            clip_y,
        );

        // Large pie (270 degrees) - rotated
        self.renderer.draw_pie(
            [390.0, 400.0],
            45.0,
            std::f32::consts::PI * 0.5,
            std::f32::consts::PI * 2.0,
            [0.3, 1.0, 0.4, 1.0],
            clip_x,
            clip_y,
        );

        // // === ROW 5: Line segments at various angles and thicknesses ===
        // // Horizontal thin line
        self.renderer.draw_segment(
            [20.0, 520.0],
            [100.0, 520.0],
            3.0,
            [1.0, 0.5, 0.0, 1.0],
            clip_x,
            clip_y,
        );

        // Diagonal medium line
        self.renderer.draw_segment(
            [120.0, 500.0],
            [200.0, 540.0],
            6.0,
            [1.0, 0.6, 0.0, 1.0],
            clip_x,
            clip_y,
        );

        // Vertical thick line
        self.renderer.draw_segment(
            [230.0, 500.0],
            [230.0, 540.0],
            10.0,
            [1.0, 0.7, 0.0, 1.0],
            clip_x,
            clip_y,
        );

        // Diagonal line (other direction)
        self.renderer.draw_segment(
            [260.0, 540.0],
            [340.0, 500.0],
            8.0,
            [1.0, 0.8, 0.0, 1.0],
            clip_x,
            clip_y,
        );

        // Cross pattern
        self.renderer.draw_segment(
            [370.0, 500.0],
            [430.0, 540.0],
            5.0,
            [1.0, 0.9, 0.2, 0.7],
            clip_x,
            clip_y,
        );
        self.renderer.draw_segment(
            [370.0, 540.0],
            [430.0, 500.0],
            5.0,
            [1.0, 0.9, 0.2, 0.8],
            clip_x,
            clip_y,
        );

        // Draw retained text box
        self.renderer.draw_text_edit(&self.text_edit);

        // Draw tiger SVG
        self.renderer.draw_image(&self.svg_handle, 520.0, 150.0, 180.0, 180.0, 0.5);

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
                        Window::default_attributes()
                            .with_inner_size(PhysicalSize::new(800, 600))
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
