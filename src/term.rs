use std::{thread, time};

use crate::{App, AttachContext, State};
use futures::executor::block_on;
use mutunga::{Cell, Color, Event, TerminalCanvas};

const FPS: u64 = 30;

pub struct Term {
	state: State,
	output_buffer: wgpu::Buffer,
}

impl Term {
	pub fn new() -> Self {
		// FIXME get term size
		let size = (128u32, 128);
		let state = block_on(State::new(None));
		let device = &state.device;
		let output_buffer_size = (4 * size.0 * size.1) as wgpu::BufferAddress;
		let output_buffer_desc = wgpu::BufferDescriptor {
			size: output_buffer_size,
			usage: wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::MAP_READ,
			label: None,
			mapped_at_creation: false,
		};
		let output_buffer = device.create_buffer(&output_buffer_desc);

		Self {
			state,
			output_buffer,
		}
	}

	pub fn device(&self) -> &wgpu::Device {
		&self.state.device
	}

	pub fn run(self, mut app: impl App + 'static) {
		let mut state = self.state;
		let mut output_buffer = self.output_buffer;

		app.attach(&mut AttachContext::new(&mut state));

		let mut term = TerminalCanvas::new();
		let width = term.width();
		let height = term.height();
		term.attach().unwrap();

		'foo: loop {
			let current_start = time::Instant::now();

			// Handle terminal events
			while let Ok(event) = term.next_event() {
				match event {
					// Resize our 3D canvas to match the terminal size
					Event::Resize(width, height) => {
						// TODO
					}
					// Ignore any other events
					_ => {}
				}
			}

			// Render the 3D scene to buffer
			state
				.render_to_buffer(&mut output_buffer, &mut app)
				.unwrap();

			{
				let buffer_slice = output_buffer.slice(..);

				// NOTE: We have to create the mapping THEN device.poll() before await
				// the future. Otherwise the application will freeze.
				let mapping = buffer_slice.map_async(wgpu::MapMode::Read);
				state.device.poll(wgpu::Maintain::Wait);
				block_on(mapping).unwrap();

				let data = buffer_slice.get_mapped_range();

				// Draw each pixel to the terminal
				for y in 0..height as usize {
					for x in 0..width as usize {
						let i = (x + y * 128) * 4;
						if x >= 128 || y >= 128 {
							continue;
						}
						let r = data[i];
						let g = data[i + 1];
						let b = data[i + 2];
						let a = data[i + 3];
						let color = Color::rgba(r, g, b, a);

						term.set_cell(
							x as i32,
							y as i32,
							Cell {
								fg: Color::transparent(),
								bg: color,
								symbol: ' ',
							},
						);
					}
				}
				term.present().unwrap();

				// Draw at fixed framerate
				let wait = time::Duration::from_millis(1000 / FPS);
				let elapsed = current_start.elapsed();
				if elapsed < wait {
					thread::sleep(wait - elapsed);
				}
			}
			output_buffer.unmap();
		}
	}
}
