use crate::AttachContext;
use crate::{context::DrawContext, App, Event, Key, MouseButton, UpdateContext};
use futures::executor::block_on;
use std::{
	collections::{HashMap, HashSet},
	error::Error,
	sync::atomic::{AtomicUsize, Ordering},
	time::{self, Duration},
};
#[cfg(test)]
use winit::platform::unix::EventLoopExtUnix;
#[cfg(target_os = "linux")]
use winit::platform::unix::WindowBuilderExtUnix;
use winit::{
	dpi::PhysicalSize,
	event::{
		DeviceEvent, ElementState, Event as WinitEvent, KeyboardInput, MouseScrollDelta,
		WindowEvent,
	},
	event_loop::{ControlFlow, EventLoop},
	window::{Window as WinitWindow, WindowBuilder},
};

pub type PipelineID = usize;
pub static NEXT_PIPELINE_ID: AtomicUsize = AtomicUsize::new(1);

pub struct State {
	pub surface: wgpu::Surface,
	pub device: wgpu::Device,
	pub queue: wgpu::Queue,
	pub sc_desc: wgpu::SwapChainDescriptor,
	pub swap_chain: wgpu::SwapChain,
	pub size: winit::dpi::PhysicalSize<u32>,
	current_pipeline: Option<PipelineID>,
	pub(crate) pipelines: HashMap<PipelineID, wgpu::RenderPipeline>,
}

pub struct Window {
	event_loop: EventLoop<()>,
	winit: WinitWindow,
	state: State,
}

impl State {
	async fn new(window: &WinitWindow) -> Self {
		let size = window.inner_size();

		let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
		let surface = unsafe { instance.create_surface(window) };
		let adapter = instance
			.request_adapter(&wgpu::RequestAdapterOptions {
				power_preference: wgpu::PowerPreference::default(),
				compatible_surface: Some(&surface),
			})
			.await
			.unwrap();

		let (device, queue) = adapter
			.request_device(
				&wgpu::DeviceDescriptor {
					features: wgpu::Features::empty(),
					limits: wgpu::Limits::default(),
					label: None,
				},
				None, // Trace path
			)
			.await
			.unwrap();

		let sc_desc = wgpu::SwapChainDescriptor {
			usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
			format: adapter.get_swap_chain_preferred_format(&surface).unwrap(),
			width: size.width,
			height: size.height,
			present_mode: wgpu::PresentMode::Fifo,
		};
		let swap_chain = device.create_swap_chain(&surface, &sc_desc);

		Self {
			surface,
			device,
			queue,
			sc_desc,
			swap_chain,
			size,
			pipelines: HashMap::new(),
			current_pipeline: None,
		}
	}

	pub fn add_pipeline(&mut self, pipeline: wgpu::RenderPipeline) -> PipelineID {
		let id = NEXT_PIPELINE_ID.fetch_add(1, Ordering::Relaxed);
		self.pipelines.insert(id, pipeline);

		id
	}

	pub fn use_pipeline(&mut self, id: PipelineID) {
		self.current_pipeline = Some(id);
	}

	fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
		self.size = new_size;
		self.sc_desc.width = new_size.width;
		self.sc_desc.height = new_size.height;
		self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
	}

	fn input(&mut self, event: &WindowEvent) -> bool {
		false
	}

	fn update(&mut self) {
		// remove `todo!()`
	}

	fn render<A: App>(&mut self, app: &mut A) -> Result<(), wgpu::SwapChainError> {
		let frame = self.swap_chain.get_current_frame()?.output;
		let mut encoder = self
			.device
			.create_command_encoder(&wgpu::CommandEncoderDescriptor {
				label: Some("Render Encoder"),
			});
		{
			let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: Some("Render Pass"),
				color_attachments: &[wgpu::RenderPassColorAttachment {
					view: &frame.view,
					resolve_target: None,
					ops: wgpu::Operations {
						load: wgpu::LoadOp::Clear(wgpu::Color {
							r: 0.2,
							g: 0.1,
							b: 0.3,
							a: 1.0,
						}),
						store: true,
					},
				}],
				depth_stencil_attachment: None,
			});

			let mut ctx = DrawContext::new(self);
			app.draw(&mut ctx, &mut render_pass);
		}

		// submit will accept anything that implements IntoIter
		self.queue.submit(std::iter::once(encoder.finish()));

		Ok(())
	}
}

impl Window {
	pub fn new() -> Self {
		#[cfg(not(test))]
		let event_loop = EventLoop::new();
		#[cfg(test)]
		let event_loop = EventLoop::new_any_thread();

		let wb = WindowBuilder::new()
			.with_inner_size(PhysicalSize::new(1920, 1080))
			.with_title("Test WGPU")
			.with_transparent(false);
		#[cfg(target_os = "linux")]
		let wb = wb.with_class("".into(), "Byd".into());
		let window = wb.build(&event_loop).unwrap();
		let state = block_on(State::new(&window));

		Self {
			event_loop,
			winit: window,
			state,
		}
	}

	pub fn device(&self) -> &wgpu::Device {
		&self.state.device
	}

	pub fn run(self, mut app: impl App + 'static) {
		let event_loop = self.event_loop;
		let window = self.winit;
		let mut state = self.state;

		app.attach(&mut AttachContext::new(&mut state));

		let grabbed = false;
		let mut held_keys: HashSet<Key> = HashSet::new();
		let mut held_buttons: HashSet<MouseButton> = HashSet::new();

		let mut mouse_pos = (0.0, 0.0);
		let start_at = time::Instant::now();
		let mut last_frame_at = time::Instant::now();
		event_loop.run(move |event, _, control_flow| {
			*control_flow = ControlFlow::Poll;
			match event {
				WinitEvent::RedrawRequested(_) => {
					state.update();
					match state.render(&mut app) {
						Ok(_) => {}
						// Recreate the swap_chain if lost
						Err(wgpu::SwapChainError::Lost) => state.resize(state.size),
						// The system is out of memory, we should probably quit
						Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
						// All other errors (Outdated, Timeout) should be resolved by the next frame
						Err(e) => eprintln!("{:?}", e),
					}

					window.request_redraw();
				}

				WinitEvent::DeviceEvent { event, .. } => match event {
					DeviceEvent::MouseMotion { delta: (x, y) } => {
						if grabbed {
							app.event(&Event::MouseMotion(x as _, y as _));
						}
						for button in &held_buttons {
							app.event(&Event::MouseDrag(button.clone(), x as _, y as _));
						}
					}
					_ => {}
				},

				WinitEvent::WindowEvent { event, .. } => match event {
					WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
					WindowEvent::Resized(size) => {
						state.resize(size);
						// FIXME resize wgpu surface
						app.event(&Event::WindowResize(size.width, size.height));
					}

					WindowEvent::CursorMoved { position, .. } => {
						mouse_pos = (position.x, position.y);
						app.event(&Event::MouseMove(position.x as _, position.y as _));
					}

					WindowEvent::MouseWheel {
						delta: MouseScrollDelta::LineDelta(x, y),
						..
					} => {
						app.event(&Event::MouseWheel(x, y));
					}

					WindowEvent::MouseInput { state, button, .. } => match state {
						ElementState::Pressed => {
							held_buttons.insert(button.into());
							app.event(&Event::MouseDown(
								button.into(),
								mouse_pos.0 as _,
								mouse_pos.1 as _,
							));
						}
						ElementState::Released => {
							held_buttons.remove(&button.into());
							app.event(&Event::MouseUp(
								button.into(),
								mouse_pos.0 as _,
								mouse_pos.1 as _,
							));
						}
					},

					WindowEvent::KeyboardInput {
						input:
							KeyboardInput {
								virtual_keycode,
								state: ElementState::Pressed,
								..
							},
						..
					} => {
						if let Some(keycode) = virtual_keycode {
							let key: Key = keycode.into();
							let is_repeat = held_keys.contains(&key);
							held_keys.insert(key.clone());
							if is_repeat {
								app.event(&Event::KeyRepeat(key));
							} else {
								app.event(&Event::KeyDown(key));
							}
						}
					}

					WindowEvent::ReceivedCharacter(ch) => {
						app.event(&Event::ReceivedCharacter(ch));
					}

					WindowEvent::KeyboardInput {
						input:
							KeyboardInput {
								virtual_keycode,
								state: ElementState::Released,
								..
							},
						..
					} => {
						if let Some(key) = virtual_keycode {
							let key: Key = key.into();
							held_keys.remove(&key);
							app.event(&Event::KeyUp(key));
						}
					}

					_ => {}
				},

				_ => {}
			};
		});
	}
}
