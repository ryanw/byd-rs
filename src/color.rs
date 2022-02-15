use cgmath::Vector4;

pub type Color = Vector4<f32>;

pub trait CreateColor {
	fn hsl(h: f32, s: f32, l: f32) -> Self;
}

impl CreateColor for Color {
	fn hsl(h: f32, s: f32, l: f32) -> Self {
		if s == 0.0 {
			return Color::new(l, l, l, 1.0);
		}

		let q = if l < 0.5 {
			l * (1.0 + s)
		} else {
			l + s - l * s
		};
		let p = 2.0 * l - q;

		let r = hue_to_rgb(p, q, h + 1.0 / 3.0);
		let g = hue_to_rgb(p, q, h);
		let b = hue_to_rgb(p, q, h - 1.0 / 3.0);
		Color::new(r, g, b, 1.0)
	}
}

fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
	if t < 0.0 {
		t += 1.0;
	}
	if t > 1.0 {
		t -= 1.0;
	}

	if t < 1.0 / 6.0 {
		return p + (q - p) * 6.0 * t;
	}

	if t < 1.0 / 2.0 {
		return q;
	}

	if t < 2.0 / 3.0 {
		return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
	}

	return p;
}
