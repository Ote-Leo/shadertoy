/**Input user fragment shader.
 * Each pixel on the window surface will run this shader.
 */

struct WindowData {
	// surface width × height
	resolution: vec2<f32>,
	// The number of frames that have been passed.
	frame: u32,
	// Number of seconds that have been elapsed
	time: f32,
	// Cursor position
	cursor: vec2<f32>,
	drag_start: vec2<f32>,
	drag_end: vec2<f32>,
	mouse_left_pressed: u32,
	mouse_left_clicked: u32,
}

@group(0) @binding(0)
var<uniform> WINDOW_DATA: WindowData;

fn palette(t: f32) -> vec3<f32> {
	let a = vec3(0.500, 0.500, 0.500);
	let b = vec3(0.500, 0.500, 0.500);
	let c = vec3(1.000, 1.000, 1.000);
	let d = vec3(0.263, 0.416, 0.557);

	return a + b * cos(6.28318 * (c * t + d));
}

@fragment
fn main(
	@builtin(position) coord: vec4<f32>,
) -> @location(0) vec4<f32> {
	let resolution = WINDOW_DATA.resolution;
	var uv = (coord.xy * 2.0 - resolution.xy) / resolution.y;
	var uv0 = uv;
	var final_color = vec3(0.0);

	let time = f32(WINDOW_DATA.frame)/50.0;

	for (var i = 0.0; i < 4.0; i=i+1.0) {
		uv = fract(uv * 1.5) - 0.5;

		var d = length(uv) * exp(-length(uv0));
		let col = palette(length(uv0) + i*0.4 + time*0.4);
		d = sin(d*8.0 + time)/8.0;
		d = abs(d);
		d = pow(0.01 / d, 1.2);

		final_color += col * d;
	}

	return vec4(final_color, 1.0);
}
