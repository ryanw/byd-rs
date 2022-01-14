let gamma: f32 = 2.2;

struct Camera {
	view: mat4x4<f32>;
	projection: mat4x4<f32>;
};

struct Actor {
	color: vec4<f32>;
	model: mat4x4<f32>;
};

struct TextureEnabled {
	enabled: i32;
};

struct VertexOutput {
	[[builtin(position)]] position: vec4<f32>;
	[[location(0)]] world_position: vec3<f32>;
	[[location(1)]] normal: vec3<f32>;
	[[location(2)]] color: vec4<f32>;
	[[location(3)]] uv: vec2<f32>;
};


[[group(0), binding(0)]]
var<uniform> camera: Camera;
[[group(0), binding(1)]]
var<uniform> actor: Actor;

[[group(1), binding(0)]]
var<uniform> texture_enabled: TextureEnabled;
[[group(1), binding(1)]]
var t_diffuse: texture_2d<f32>;
[[group(1), binding(2)]]
var s_diffuse: sampler;

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
	out.color = actor.color;
	out.uv = uv;

	return out;
}


[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
	var light_pos = vec3<f32>(5.0, -4.0, 0.0);
	var light_dir = normalize(light_pos - in.world_position);
	var shade = clamp(dot(in.normal, light_dir), 0.0, 0.7) + 0.3;
	var color = in.color;
	if (texture_enabled.enabled > 0) {
		color = textureSample(t_diffuse, s_diffuse, in.uv);
	}
	return pow(color * shade, vec4<f32>(gamma));
}
