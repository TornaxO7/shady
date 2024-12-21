@group(0) @binding(0)
var<uniform> iTime: f32;

// x: width
// y: height
@group(0) @binding(1)
var<uniform> iResolution: vec2<f32>;

// x: bass
// y: mid
// z: treble
@group(0) @binding(2)
var<uniform> iAudio: vec3<f32>;

@fragment
fn main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = pos.xy/iResolution.xy;
    let col = 0.5 + 0.5 * cos(iTime + uv.xyx + vec3<f32>(0.0, 2.0, 4.0));

    return vec4<f32>(col, 1.0);
}