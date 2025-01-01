#version 450

layout(binding = 0) uniform float iTime;

// x: width
// y: height
layout(binding = 1) uniform vec2 iResolution;

// 
layout(binding = 2) buffer iAudio {
    float freqs[10];
};

// x: x-coord when the mouse is pressed
// y: y-coord when the mouse is pressed
// z: x-coord when the mouse is released
// w: y-coord when the mouse is released
layout(binding = 3) uniform vec4 iMouse;

layout(binding = 4) uniform uint iFrame;

// the color which the pixel should have
layout(location = 0) out vec4 fragColor;

void main() {
    // Normalized pixel coordinates (from 0 to 1)
    vec2 uv = gl_FragCoord.xy/iResolution.xy;

    // Time varying pixel color
    vec3 col = 0.5 + 0.5*cos(iTime+uv.xyx+vec3(0,2,4));

    // Output to screen
    fragColor = vec4(col,1.0);      
}
