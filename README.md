# Shady

Shady is a shader-sandbox app and library which is similar to [shadertoy] but instead of just allowing to write shaders with GLSL you are able to write
shaders in [glsl] _and_ [wgsl] (thanks to [naga]).

This repository includes more than one app since they are somewhat related. Feel free to click one of the links to know more about them and how to use them:

- [shady-toy]: Basically a desktop version of [shadertoy].
- [shady-cli]: A [cava] like audio visualizer in the terminal.

# Troubleshooting

## `shady` audio doesn't listen to my systems audio

[shady-toy] and [shady-cli] are both using your default output device for the audio.
Check, if [shady] is listening to the correct audio source for example with [pavucontrol] in the "Recording" tab.
For example on my system it looks like this (after starting [pavucontrol]):

![Example](./assets/shady_audio_settings.png)

# Sources/Similar projects

Here are some other sources/similar projects if you're interested:

- Other music visualizers:
  - https://github.com/phip1611/spectrum-analyzer
  - https://github.com/BrunoWallner/crav
  - https://github.com/karlstav/cava
- Tscoding implementing [musializer] https://www.youtube.com/watch?v=Xdbk1Pr5WXU&list=PLpM-Dvs8t0Vak1rrE2NJn8XYEJ5M7-BqT
- WGPU tutorials:
  - https://sotrh.github.io/learn-wgpu/
  - https://webgpufundamentals.org/

[shadertoy]: https://www.shadertoy.com/
[pavucontrol]: https://github.com/pulseaudio/pavucontrol
[naga]: https://crates.io/crates/naga
[shady-toy]: https://github.com/TornaxO7/shady/tree/main/shady-toy
[shady-cli]: https://github.com/TornaxO7/shady/tree/main/shady-cli
[glsl]: https://www.khronos.org/opengl/wiki/Core_Language_(GLSL)
[wgsl]: https://www.w3.org/TR/WGSL/
[musializer]: https://github.com/tsoding/musializer
[cava]: https://github.com/karlstav/cava
