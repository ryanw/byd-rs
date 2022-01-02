let gamma: f32 = 2.2;

struct Camera {
	view: mat4x4<f32>;
	projection: mat4x4<f32>;
	model: mat4x4<f32>;
};

struct VertexOutput {
	[[builtin(position)]] position: vec4<f32>;
	[[location(0)]] color: vec4<f32>;
};


[[group(0), binding(0)]]
var<uniform> camera: Camera;

[[stage(vertex)]]
fn vs_main(
	[[location(0)]] position: vec3<f32>,
	[[location(1)]] color: vec4<f32>,
) -> VertexOutput {
	var out: VertexOutput;

	var mvp: mat4x4<f32> = camera.projection * camera.view * camera.model;

	out.position = mvp * vec4<f32>(position, 1.0);
	out.color = color;

	return out;
}


[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
	return pow(in.color, vec4<f32>(gamma));
}
