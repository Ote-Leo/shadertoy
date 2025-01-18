@vertex
fn main(@builtin(vertex_index) idx: u32) -> @builtin(position) vec4<f32> {
	var positions = array<vec2<f32>, 3>(
		vec2(-1.0, -1.0),
		vec2( 3.0, -1.0),
		vec2(-1.0,  3.0),
	);

	return vec4(positions[idx], 0.0, 1.0);
}
