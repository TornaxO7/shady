#version 450

layout(binding = 0) uniform float iTime;

// x: width
// y: height
layout(binding = 1) uniform vec2 iResolution;

// x: bass
// y: mid
// z: treble
layout(binding = 2) uniform vec3 iAudio;

layout(binding = 4) uniform double iFrame;

layout(location = 0) out vec4 fragColor;

void main() {
    // Normalized pixel coordinates (from 0 to 1)
    vec2 uv = gl_FragCoord.xy/iResolution.xy;

    // Time varying pixel color
    vec3 col = 0.5 + 0.5*cos(iTime+uv.xyx+vec3(0,2,4));

    // Output to screen
    fragColor = vec4(col,1.0);      
}
