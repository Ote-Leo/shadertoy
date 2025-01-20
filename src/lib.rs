// TODO: pass the following values to the fragment shader
//     - time
//     - drag_start
//     - drag_end
//     - mouse_left_pressed
//     - mouse_left_clicked
mod waker;

use waker::block_on;

use winit::{
	application::ApplicationHandler,
	dpi::{PhysicalPosition, PhysicalSize},
	event::{ElementState, WindowEvent},
	event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy},
	keyboard::{KeyCode, PhysicalKey},
	window::{Window, WindowId},
};

use anyhow::Context;
use std::io::Write;

const DEFAULT_SHADER_PATH: &str = "shaders/prelude.wgsl";

#[cfg(target_arch = "wasm32")]
type Rc<T> = std::rc::Rc<T>;
#[cfg(not(target_arch = "wasm32"))]
type Rc<T> = std::sync::Arc<T>;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct WindowData {
	resolution: [f32; 2],
	frame: u32,
	time: f32,
	cursor: [f32; 2],
	drag_start: [f32; 2],
	drag_end: [f32; 2],
	mouse_left_pressed: u32,
	mouse_left_clicked: u32,
}

struct Graphics {
	window: Rc<Window>,
	surface: wgpu::Surface<'static>,
	surface_config: wgpu::SurfaceConfiguration,
	device: wgpu::Device,
	queue: wgpu::Queue,

	render_pipeline_layout: wgpu::PipelineLayout,
	render_pipeline: wgpu::RenderPipeline,

	window_data: WindowData,
	window_data_buffer: wgpu::Buffer,
	window_data_bind_group: wgpu::BindGroup,

	vertex_shader: wgpu::ShaderModule,
}

async fn create_graphics(event_loop: &ActiveEventLoop) -> anyhow::Result<Graphics> {
	let window_attrs = Window::default_attributes();

	let window = Rc::new(
		event_loop
			.create_window(window_attrs)
			.context("creating window")?,
	);

	let instance = wgpu::Instance::default();
	let surface = instance
		.create_surface(window.clone())
		.context("creating window surface")?;

	let adapter = instance
		.request_adapter(&wgpu::RequestAdapterOptions {
			compatible_surface: Some(&surface),
			power_preference: wgpu::PowerPreference::None,
			force_fallback_adapter: false,
		})
		.await
		.context("requesting wgpu adapter")?;

	let (device, queue) = adapter
		.request_device(
			&wgpu::DeviceDescriptor {
				label: None,
				required_features: wgpu::Features::empty(),
				memory_hints: Default::default(),
				required_limits: wgpu::Limits::default(),
			},
			None,
		)
		.await
		.context("requesting wgpu device")?;

	device.push_error_scope(wgpu::ErrorFilter::Validation);

	let size = window.inner_size();
	let surface_config = surface
		.get_default_config(&adapter, size.width, size.height)
		.context("getting default surface configuration")?;

	surface.configure(&device, &surface_config);

	let vertex_shader = device.create_shader_module(wgpu::include_wgsl!("../shaders/vertex.wgsl"));

	let window_data_buffer = device.create_buffer(&wgpu::BufferDescriptor {
		label: Some("window_data_buffer"),
		size: std::mem::size_of::<WindowData>() as u64,
		usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
		mapped_at_creation: false,
	});

	let window_data_group_layout =
		device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			label: Some("window_data_group_layout"),
			entries: &[wgpu::BindGroupLayoutEntry {
				binding: 0,
				visibility: wgpu::ShaderStages::FRAGMENT,
				ty: wgpu::BindingType::Buffer {
					ty: wgpu::BufferBindingType::Uniform,
					has_dynamic_offset: false,
					min_binding_size: None,
				},
				count: None,
			}],
		});

	let window_data_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
		label: Some("window_data_bind_group"),
		layout: &window_data_group_layout,
		entries: &[wgpu::BindGroupEntry {
			binding: 0,
			resource: window_data_buffer.as_entire_binding(),
		}],
	});

	let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
		label: Some("render_pipeline_layout"),
		bind_group_layouts: &[&window_data_group_layout],
		push_constant_ranges: &[],
	});

	let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
		label: Some("render_pipeline"),
		layout: Some(&render_pipeline_layout),
		vertex: wgpu::VertexState {
			module: &vertex_shader,
			entry_point: None,
			compilation_options: Default::default(),
			buffers: &[],
		},
		fragment: None,
		primitive: Default::default(),
		multisample: Default::default(),
		cache: None,
		depth_stencil: None,
		multiview: None,
	});

	let window_data = WindowData {
		resolution: [size.width as f32, size.height as f32],
		frame: 0,
		time: 0.0,
		cursor: [0.0, 0.0],
		drag_start: [0.0, 0.0],
		drag_end: [0.0, 0.0],
		mouse_left_pressed: 0,
		mouse_left_clicked: 0,
	};

	Ok(Graphics {
		window,
		surface,
		surface_config,
		device,
		queue,

		render_pipeline_layout,
		render_pipeline,

		window_data,
		window_data_buffer,
		window_data_bind_group,

		vertex_shader,
	})
}

struct GraphicsBuilder {
	event_loop_proxy: Option<EventLoopProxy<Graphics>>,
}

impl GraphicsBuilder {
	fn new(event_loop_proxy: EventLoopProxy<Graphics>) -> Self {
		Self {
			event_loop_proxy: Some(event_loop_proxy),
		}
	}

	fn build_and_send(&mut self, event_loop: &ActiveEventLoop) {
		let Some(event_loop_proxy) = self.event_loop_proxy.take() else {
			return;
		};

		#[cfg(target_arch = "wasm32")]
		{
			let mut gfx = create_graphics(event_loop);
			gfx.reload_shader().expect("failed to reload shader");
			wasm_bindgen_futures::spawn_local(async move {
				let gfx = gfx.await.expect("failed to create graphics context");
				assert!(event_loop_proxy.send_event(gfx).is_ok());
			});
		}
		#[cfg(not(target_arch = "wasm32"))]
		{
			let mut gfx =
				block_on(create_graphics(event_loop)).expect("failed to create graphics context");
			gfx.reload_shader().expect("failed to reload shader");
			assert!(event_loop_proxy.send_event(gfx).is_ok());
		}
	}
}

fn display_seconds(duration: u64) -> String {
	const MINUTE: u64 = 60;
	const HOUR: u64 = 60 * MINUTE;
	const DAY: u64 = 24 * HOUR;

	let days = duration / DAY;
	let hours = duration % DAY / HOUR;
	let minutes = duration % DAY % HOUR / MINUTE;
	let seconds = duration % DAY % HOUR % MINUTE;

	let mut tokens = vec![];
	if days > 0 {
		tokens.push(format!("{days} d"));
	}
	if hours > 0 {
		tokens.push(format!("{hours} h"));
	}
	if minutes > 0 {
		tokens.push(format!("{minutes} m"));
	}
	tokens.push(format!("{seconds} s"));

	return tokens.join(" ")
}

impl Graphics {
	fn draw(&mut self) {
		let frame = self.surface.get_current_texture().unwrap();
		let view = frame.texture.create_view(&Default::default());
		let mut encoder = self.device.create_command_encoder(&Default::default());

		{
			let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				color_attachments: &[Some(wgpu::RenderPassColorAttachment {
					view: &view,
					resolve_target: None,
					ops: wgpu::Operations {
						load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
						store: wgpu::StoreOp::Store,
					},
				})],
				..Default::default()
			});

			render_pass.set_bind_group(0, Some(&self.window_data_bind_group), &[]);
			render_pass.set_pipeline(&self.render_pipeline);
			render_pass.draw(0..3, 0..1);
		}

		let command_buffer = encoder.finish();

		self.queue
			.write_buffer(&self.window_data_buffer, 0 as wgpu::BufferAddress, unsafe {
				core::slice::from_raw_parts(
					[self.window_data].as_ptr() as *const u8,
					std::mem::size_of::<WindowData>(),
				)
			});
		self.queue.submit([command_buffer]);
		frame.present();

		self.window_data.frame = self.window_data.frame.wrapping_add(1);
	}

	fn resize(&mut self, size: PhysicalSize<u32>) {
		self.surface_config.width = size.width;
		self.surface_config.height = size.height;
		self.window_data.resolution = [size.width as f32, size.height as f32];
		self.surface.configure(&self.device, &self.surface_config);
	}

	fn update_cursor_position(&mut self, position: PhysicalPosition<f64>) {
		self.window_data.cursor = [position.x as f32, position.y as f32];
	}

	fn reload_shader(&mut self) -> anyhow::Result<()> {
		let start = std::time::Instant::now();

		let shader_src = std::fs::read_to_string(DEFAULT_SHADER_PATH)
			.context("reading vertex prelude shader")?;
		let shader = self
			.device
			.create_shader_module(wgpu::ShaderModuleDescriptor {
				label: Some("shader"),
				source: wgpu::ShaderSource::Wgsl(shader_src.into()),
			});

		print!("\x1b[2J\x1b[H");
		std::io::stdout().flush().context("clearing stdout")?;

		let mut successful_compilation = true;

		block_on(async {
			let compilation_info = shader.get_compilation_info().await;
			for msg in compilation_info.messages.iter() {
				successful_compilation = false;
				if msg.message_type != wgpu::CompilationMessageType::Info {
					eprintln!("{}", msg.message);
				}
			}
		});

		self.render_pipeline =
			self.device
				.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
					label: Some("render_pipeline"),
					layout: Some(&self.render_pipeline_layout),
					vertex: wgpu::VertexState {
						module: &self.vertex_shader,
						entry_point: None,
						compilation_options: wgpu::PipelineCompilationOptions::default(),
						buffers: &[],
					},
					primitive: wgpu::PrimitiveState::default(),
					depth_stencil: None,
					multisample: wgpu::MultisampleState::default(),
					fragment: Some(wgpu::FragmentState {
						module: &shader,
						entry_point: None,
						compilation_options: wgpu::PipelineCompilationOptions::default(),
						targets: &[Some(wgpu::ColorTargetState {
							format: self.surface_config.format,
							blend: Some(wgpu::BlendState::REPLACE),
							write_mask: wgpu::ColorWrites::ALL,
						})],
					}),
					multiview: None,
					cache: None,
				});

		if successful_compilation {
			let duration = std::time::Instant::now().duration_since(start).as_secs();
		    println!("reloaded prelude shader successfully in {}", display_seconds(duration));
		}

		Ok(())
	}
}

enum MaybeGraphics {
	Builder(GraphicsBuilder),
	Graphics(Graphics),
}

struct Application {
	graphics: MaybeGraphics,
}

impl Application {
	fn new(event_loop: &EventLoop<Graphics>) -> Self {
		Self {
			graphics: MaybeGraphics::Builder(GraphicsBuilder::new(event_loop.create_proxy())),
		}
	}

	fn draw(&mut self) {
		let MaybeGraphics::Graphics(gfx) = &mut self.graphics else {
			return;
		};
		gfx.draw()
	}
}

impl ApplicationHandler<Graphics> for Application {
	fn window_event(
		&mut self,
		event_loop: &ActiveEventLoop,
		window_id: WindowId,
		event: WindowEvent,
	) {
		use winit::{
			event::KeyEvent,
			keyboard::{Key, NamedKey},
		};

		let MaybeGraphics::Graphics(gfx) = &mut self.graphics else {
			return;
		};

		if gfx.window.id() != window_id {
			return;
		}

		match event {
			WindowEvent::Resized(size) => gfx.resize(size),
			WindowEvent::CloseRequested
			| WindowEvent::KeyboardInput {
				event: KeyEvent {
					logical_key: Key::Named(NamedKey::Escape),
					..
				},
				..
			} => event_loop.exit(),
			WindowEvent::CursorMoved { position, .. } => gfx.update_cursor_position(position),
			WindowEvent::KeyboardInput {
				event:
					KeyEvent {
						physical_key: PhysicalKey::Code(KeyCode::KeyR),
						state: ElementState::Pressed,
						..
					},
				..
			} => gfx.reload_shader().expect("failed to reload shader"),
			_ => (),
		}
	}

	fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
		self.draw()
	}

	fn resumed(&mut self, event_loop: &ActiveEventLoop) {
		if let MaybeGraphics::Builder(builder) = &mut self.graphics {
			builder.build_and_send(event_loop);
		}
	}

	fn user_event(&mut self, _event_loop: &ActiveEventLoop, graphics: Graphics) {
		self.graphics = MaybeGraphics::Graphics(graphics);
	}
}

pub fn run() {
	let event_loop = EventLoop::with_user_event().build().unwrap();
	let mut app = Application::new(&event_loop);
	event_loop.set_control_flow(ControlFlow::Poll);
	event_loop.run_app(&mut app).unwrap();
}

#[cfg(target_arch = "wasm32")]
const CANVAS_ID: &str = "wgpu-canvas";

#[cfg(target_arch = "wasm32")]
pub fn run_web() {
	let window = web_sys::window().unwrap_throw();
	let document = window.document().unwrap_throw();

	let canvas = documnet.create_element("canvas").unwrap_throw();
	canvas.set_id(CANVAS_ID);
	canvas.set_attribute("width", "500").unwrap_throw();
	canvas.set_attribute("height", "500").unwrap_throw();

	let body = document
		.get_element_by_tag_name("body")
		.item(0)
		.unwrap_throw();

	body.append_with_node_1(canvas.unchecked_ref())
		.unwrap_throw();

	run();
}
