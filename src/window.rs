use crate::{Event, Key, MouseButton};
use std::{collections::HashSet, time::Instant};
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

pub struct Window {
	event_loop: EventLoop<Event>,
	pub(crate) winit: WinitWindow,
}

impl Window {
	pub fn new(width: u32, height: u32) -> Self {
		let event_loop: EventLoop<Event> = EventLoop::with_user_event();

		let wb = WindowBuilder::new()
			.with_inner_size(PhysicalSize::new(width, height))
			.with_title("Test WGPU")
			.with_transparent(false);
		#[cfg(target_os = "linux")]
		let wb = wb.with_class("".into(), "Byd".into());
		let window = wb.build(&event_loop).unwrap();

		Self {
			event_loop,
			winit: window,
		}
	}

	pub fn run<F>(self, mut event_handler: F)
	where
		F: 'static + FnMut(Event, &mut ControlFlow),
	{
		let event_loop = self.event_loop;
		let window = self.winit;

		let grabbed = false;
		let mut held_keys: HashSet<Key> = HashSet::new();
		let mut held_buttons: HashSet<MouseButton> = HashSet::new();

		let mut mouse_pos = (0.0, 0.0);
		let mut last_update_at = Instant::now();
		let event_proxy = event_loop.create_proxy();
		event_loop.run(move |event, _, control_flow| {
			*control_flow = ControlFlow::Poll;
			match event {
				WinitEvent::RedrawEventsCleared => {
					window.request_redraw();
				}

				WinitEvent::RedrawRequested(_) => {
					event_proxy
						.send_event(Event::Draw(last_update_at.elapsed()))
						.expect("Failed to send event");
					last_update_at = Instant::now();
				}

				WinitEvent::DeviceEvent { event, .. } => match event {
					DeviceEvent::MouseMotion { delta: (x, y) } => {
						if grabbed {
							event_proxy
								.send_event(Event::MouseMotion(x as _, y as _))
								.expect("Failed to send event");
						}
						for button in &held_buttons {
							event_proxy
								.send_event(Event::MouseDrag(button.clone(), x as _, y as _))
								.expect("Failed to send event");
						}
					}
					_ => {}
				},

				WinitEvent::WindowEvent { event, .. } => match event {
					WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
					WindowEvent::Resized(size) => {
						event_proxy
							.send_event(Event::WindowResize(size.width, size.height))
							.expect("Failed to send event");
					}

					WindowEvent::CursorMoved { position, .. } => {
						mouse_pos = (position.x, position.y);
						event_proxy
							.send_event(Event::MouseMove(position.x as _, position.y as _))
							.expect("Failed to send event");
					}

					WindowEvent::MouseWheel {
						delta: MouseScrollDelta::LineDelta(x, y),
						..
					} => {
						event_proxy
							.send_event(Event::MouseWheel(x, y))
							.expect("Failed to send event");
					}

					WindowEvent::MouseInput { state, button, .. } => match state {
						ElementState::Pressed => {
							held_buttons.insert(button.into());
							event_proxy
								.send_event(Event::MouseDown(
									button.into(),
									mouse_pos.0 as _,
									mouse_pos.1 as _,
								))
								.expect("Failed to send event");
						}
						ElementState::Released => {
							held_buttons.remove(&button.into());
							event_proxy
								.send_event(Event::MouseUp(
									button.into(),
									mouse_pos.0 as _,
									mouse_pos.1 as _,
								))
								.expect("Failed to send event");
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
							let key = keycode.into();
							let is_repeat = held_keys.contains(&key);
							held_keys.insert(key.clone());
							if is_repeat {
								event_proxy
									.send_event(Event::KeyRepeat(key))
									.expect("Failed to send event");
							} else {
								event_proxy
									.send_event(Event::KeyDown(key))
									.expect("Failed to send event");
							}
						}
					}

					WindowEvent::ReceivedCharacter(ch) => {
						event_proxy
							.send_event(Event::ReceivedCharacter(ch))
							.expect("Failed to send event");
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
							let key = Key::from(key);
							held_keys.remove(&key);
							event_proxy
								.send_event(Event::KeyUp(key))
								.expect("Failed to send event");
						}
					}

					_ => {}
				},

				WinitEvent::UserEvent(user_event) => {
					event_handler(user_event, control_flow);
				}
				_ => {}
			};
		});
	}
}
