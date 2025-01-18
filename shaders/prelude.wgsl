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

fn min2(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
	return vec2<f32>(
		min(a.x, b.x),
		min(a.y, b.y),
	);
}

fn min3(a: vec3<f32>, b: vec3<f32>) -> vec3<f32> {
	return vec3<f32>(
		min(a.x, b.x),
		min(a.y, b.y),
		min(a.z, b.z),
	);
}

fn min4(a: vec4<f32>, b: vec4<f32>) -> vec4<f32> {
	return vec4<f32>(
		min(a.x, b.x),
		min(a.y, b.y),
		min(a.z, b.z),
		min(a.w, b.w),
	);
}

fn max2(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
	return vec2<f32>(
		max(a.x, b.x),
		max(a.y, b.y),
	);
}

fn max3(a: vec3<f32>, b: vec3<f32>) -> vec3<f32> {
	return vec3<f32>(
		max(a.x, b.x),
		max(a.y, b.y),
		max(a.z, b.z),
	);
}

fn max4(a: vec4<f32>, b: vec4<f32>) -> vec4<f32> {
	return vec4<f32>(
		max(a.x, b.x),
		max(a.y, b.y),
		max(a.z, b.z),
		max(a.w, b.w),
	);
}

fn splat2(v: f32) -> vec2<f32> {
	return vec2<f32>(v, v);
}

fn splat3(v: f32) -> vec3<f32> {
	return vec3<f32>(v, v, v);
}

fn splat4(v: f32) -> vec4<f32> {
	return vec4<f32>(v, v, v, v);
}

fn mix3(a: vec3<f32>, b: vec3<f32>, d: f32) -> vec3<f32> {
	return vec3<f32>(
		mix(a.x, b.x, d),
		mix(a.y, b.y, d),
		mix(a.z, b.z, d)
	);
}

fn sd_round_box(p: vec2<f32>, b: vec2<f32>, in_r: vec4<f32>) -> f32 {
	var r: vec4<f32>;

	r = in_r;

	if (p.x > 0.0) {
		r.x = r.x;
		r.y = r.y;
	} else {
		r.x = r.z;
		r.y = r.w;
	}

	let q = abs(p) - b + vec2<f32>(r.x, r.x);
	return min(max(q.x, q.y), 0.0) + length(max2(q, splat2(0.0))) - r.x;
}

@fragment
fn main(
	@builtin(position) coord: vec4<f32>,
) -> @location(0) vec4<f32> {

	let time = f32(WINDOW_DATA.frame)/100.0;

    let p = (splat2(2.0) * coord.xy - WINDOW_DATA.resolution) / splat2(WINDOW_DATA.resolution.y);
    let si = vec2<f32>(0.9, 0.6);
    let ra = splat4(0.3) + splat4(0.3) * cos(splat4(time) * vec4<f32>(0.0, 1.0, 2.0, 3.0));
    let d = sd_round_box(p, si, ra);
    let col = splat3(1.0) - sign(d)  * vec3<f32>(0.1, 0.4, 0.7);
    let col1 = col * splat3(1.0) - splat3(exp(-3.0 * abs(d)));
    let col2 = col1 * splat3(0.8) * splat3(0.2) * cos(150.0 * d);
    let col3 = mix3(col2, splat3(1.0), 1.0 - smoothstep(0.0, 0.02, abs(d)));

    return vec4<f32>(col3, 1.0);
}
