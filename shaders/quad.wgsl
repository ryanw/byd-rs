let gamma: f32 = 2.2;

[[group(0), binding(0)]]
var t_diffuse: texture_2d<f32>;
[[group(0), binding(1)]]
var s_diffuse: sampler;

struct VertexOutput {
	[[builtin(position)]] position: vec4<f32>;
	[[location(0)]] uv: vec2<f32>;
};


[[stage(vertex)]]
fn vs_main(
	[[location(0)]] position: vec3<f32>,
) -> VertexOutput {
	var out: VertexOutput;

	out.position = vec4<f32>(position, 1.0);
	out.uv = position.xy * 0.5 + 0.5;

	return out;
}


[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
	return textureSample(t_diffuse, s_diffuse, in.uv);
}
