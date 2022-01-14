let gamma: f32 = 2.2;

struct Camera {
	view: mat4x4<f32>;
	projection: mat4x4<f32>;
};

struct Actor {
	color: vec4<f32>;
	model: mat4x4<f32>;
};

struct VertexOutput {
	[[builtin(position)]] position: vec4<f32>;
	[[location(0)]] world_position: vec3<f32>;
	[[location(1)]] normal: vec3<f32>;
	[[location(2)]] color: vec4<f32>;
};


[[group(0), binding(0)]]
var<uniform> camera: Camera;

[[group(0), binding(1)]]
var<uniform> actor: Actor;

[[stage(vertex)]]
fn vs_main(
	[[location(0)]] position: vec3<f32>,
	[[location(1)]] normal: vec3<f32>,
	[[location(2)]] uv: vec2<f32>,
) -> VertexOutput {
	var out: VertexOutput;

	var mvp: mat4x4<f32> = camera.projection * camera.view * actor.model;

	out.position = mvp * vec4<f32>(position, 1.0);
	out.world_position = (actor.model * vec4<f32>(position, 1.0)).xyz;
	out.normal = normalize((actor.model * vec4<f32>(normal, 0.0)).xyz);
	out.color = vec4<f32>(1.0, 1.0, 0.0, 1.0);

	return out;
}


[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
	return pow(in.color, vec4<f32>(gamma));
}
