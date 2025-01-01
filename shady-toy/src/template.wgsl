@group(0) @binding(0)
var<uniform> iTime: f32;

// x: width
// y: height
@group(0) @binding(1)
var<uniform> iResolution: vec2<f32>;

@group(0) @binding(2)
var<storage, read> iAudio: array<f32, 10>;

// x: x-coord when the mouse is pressed
// y: y-coord when the mouse is pressed
// z: x-coord when the mouse is released
// w: y-coord when the mouse is released
@group(0) @binding(3)
var<uniform> iMouse: vec4<f32>;

@group(0) @binding(4)
var<uniform> iFrame: u32;

@fragment
fn main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = pos.xy/iResolution.xy;
    let col = 0.5 + 0.5 * cos(iTime + uv.xyx + vec3<f32>(0.0, 2.0, 4.0));

    return vec4<f32>(col, 1.0);
}