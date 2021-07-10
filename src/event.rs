use winit::event::VirtualKeyCode;

use crate::context::DrawContext;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum MouseButton {
	Left,
	Right,
	Middle,
	WheelUp,
	WheelDown,
	Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Key {
	Unknown,
	LShift,
	RShift,
	Space,
	LControl,
	RControl,
	Tab,
	Backspace,
	Delete,
	Home,
	End,
	PageUp,
	PageDown,
	Insert,
	Left,
	Right,
	Up,
	Down,
	F1,
	F2,
	F3,
	F4,
	F5,
	F6,
	F7,
	F8,
	F9,
	F10,
	F11,
	F12,
	A,
	B,
	C,
	D,
	E,
	F,
	G,
	H,
	I,
	J,
	K,
	L,
	M,
	N,
	O,
	P,
	Q,
	R,
	S,
	T,
	U,
	V,
	W,
	X,
	Y,
	Z,
}

pub enum Event {
	KeyUp(Key),
	KeyDown(Key),
	KeyRepeat(Key),
	MouseDown(MouseButton, f32, f32),
	MouseUp(MouseButton, f32, f32),
	MouseMove(f32, f32),
	MouseMotion(f32, f32),
	MouseWheel(f32, f32),
	MouseDrag(MouseButton, f32, f32),
	WindowResize(u32, u32),
	ReceivedCharacter(char),
}

impl From<VirtualKeyCode> for Key {
	fn from(other: VirtualKeyCode) -> Key {
		match other {
			VirtualKeyCode::Left => Key::Left,
			VirtualKeyCode::Right => Key::Right,
			VirtualKeyCode::Up => Key::Up,
			VirtualKeyCode::Down => Key::Down,
			VirtualKeyCode::Back => Key::Backspace,
			VirtualKeyCode::Space => Key::Space,
			VirtualKeyCode::LShift => Key::LShift,
			VirtualKeyCode::RShift => Key::RShift,
			VirtualKeyCode::LControl => Key::LControl,
			VirtualKeyCode::RControl => Key::RControl,
			VirtualKeyCode::A => Key::A,
			VirtualKeyCode::B => Key::B,
			VirtualKeyCode::C => Key::C,
			VirtualKeyCode::D => Key::D,
			VirtualKeyCode::E => Key::E,
			VirtualKeyCode::F => Key::F,
			VirtualKeyCode::G => Key::G,
			VirtualKeyCode::H => Key::H,
			VirtualKeyCode::I => Key::I,
			VirtualKeyCode::J => Key::J,
			VirtualKeyCode::K => Key::K,
			VirtualKeyCode::L => Key::L,
			VirtualKeyCode::M => Key::M,
			VirtualKeyCode::N => Key::N,
			VirtualKeyCode::O => Key::O,
			VirtualKeyCode::P => Key::P,
			VirtualKeyCode::Q => Key::Q,
			VirtualKeyCode::R => Key::R,
			VirtualKeyCode::S => Key::S,
			VirtualKeyCode::T => Key::T,
			VirtualKeyCode::U => Key::U,
			VirtualKeyCode::V => Key::V,
			VirtualKeyCode::W => Key::W,
			VirtualKeyCode::X => Key::X,
			VirtualKeyCode::Y => Key::Y,
			VirtualKeyCode::Z => Key::Z,
			_ => {
				log::warn!("Unknown key: {:?}", other);
				Key::Unknown
			}
		}
	}
}

impl From<winit::event::MouseButton> for MouseButton {
	fn from(other: winit::event::MouseButton) -> MouseButton {
		match other {
			winit::event::MouseButton::Left => MouseButton::Left,
			winit::event::MouseButton::Right => MouseButton::Right,
			winit::event::MouseButton::Middle => MouseButton::Middle,
			winit::event::MouseButton::Other(_) => MouseButton::Other,
		}
	}
}
