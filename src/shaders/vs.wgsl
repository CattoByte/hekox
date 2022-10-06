[[block]]
struct Data {
	world:	mat4x4<f32>;
	view:	mat4x4<f32>;
	proj:	mat4x4<f32>;
};

struct VertexOutput {
	[[builtin(position)]] pos: vec4<f32>;
};

[[group(0), binding(0)]]
var<uniform> uniforms: Data;

[[stage(vertex)]]
fn main(
	[[location(0)]] pos: vec3<f32>,
) -> VertexOutput {
	let worldview:	mat4x4<f32> =	uniforms.view * uniforms.world;
	let wv3:	mat3x3<f32> =	mat3x3<f32>(worldview[0].xyz, worldview[1].xyz, worldview[2].xyz);
	let out_pos:	vec4<f32> =	uniforms.proj * worldview * vec4<f32>(pos, 1.0);
	return VertexOutput(out_pos);
}
