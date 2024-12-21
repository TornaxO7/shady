// Original author: https://www.youtube.com/watch?v=f4s1h2YETNY&pp=ygUPc2hhZGVyIHR1dG9yaWFs

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

@group(0) @binding(4)
var<uniform> iFrame: u32;

fn palette(t: f32) -> vec3<f32> {
    let a = vec3<f32>(0.5, 0.5, 0.5);
    let b = vec3<f32>(0.5, 0.5, 0.5);
    let c = vec3<f32>(1.0, 1.0, 1.0);
    let d = vec3<f32>(0.263, 0.416, 0.557);

    return a + b * cos(6.28318 * (c*t+d) );
}

@fragment
fn main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    var uv = (pos.xy * 2.0 - iResolution.xy) / iResolution.y;
    let uv0 = uv;
    var finalColor = vec3<f32>(0.0);
    
    for (var i: u32 = 0; i < 4; i++) {
        uv = fract(uv * 1.5) - 0.5;

        var d: f32 = length(uv) * exp(-length(uv0));

        let col = palette(length(uv0) + f32(i) * .4 + iTime * 0.4);

        d = sin(d*8. + iTime)/8.;
        d = abs(d);

        d = pow(0.01 / d, 1.2);

        finalColor += col * d;
    }

    // gamma color correction
    finalColor.x = pow(finalColor.x, 1.0 / 0.4);
    finalColor.y = pow(finalColor.y, 1.0 / 0.4);
    finalColor.z = pow(finalColor.z, 1.0 / 0.4);
        
    return vec4<f32>(finalColor, 1.0);
}