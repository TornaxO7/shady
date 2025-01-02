# Shady-toy

Basically a desktop version [shadertoy] _but_ `shady-toy` is _not_ compatible with its shaders.
`shady-toy` supports [glsl] _and_ [wgsl] shaders and it's rather _inspired_ by [shadertoy].

However, it's very easy to port a shader from [shadertoy] to `shady-toy`. Below is a section which explains that.

# Demo

[![](https://www.youtube.com/watch?v=twpxyUE5soE)](https://www.youtube.com/watch?v=twpxyUE5soE)

# Shader examples

You can find some shaders in the [shader-examples] directory if you want to try some shaders out.
Just provide the path to the file if you start `shady-toy` (but **_without_** the argument `--template`, otherwise you'll overwrite those files).

Obviously, credits are going to the original authors of those shaders. Source of those shaders are added at the top of the files.
I wish I could add some of my own, but I don't have this skill (yet?) :(

# Usage

### `nix` with flakes

```bash
nix run github:TornaxO7/shady#shady-toy -- <shady-toy args>
```

#### Example

- `nix run github:TornaxO7/shady#shady-toy -- /tmp/test.glsl --template` to start writing a `glsl` shader
- `nix run github:TornaxO7/shady#shady-toy -- /tmp/test.wgsl --template` to start writing a `wgsl` shader

### Build it yourself

I didn't try it myself but you just need vulkan and/or opengl and (of course) [rust].
Afterwards, navigate into this directory and execute

```bash
cargo run --release -- <shady-toy args>
```

#### Example

- `cargo run --release -- /tmp/test.glsl --template` to start writing a `glsl` shader
- `cargo run --release -- /tmp/test.wgsl --template` to start writing a `wgsl` shader

# Run shadertoy shaders

`shady-toy` implemented the following uniform/storage buffers:

- `iAudio`
- `iFrame`
- `iMouse`
- `iResolution`
- `iTime`

All you need to do to run (some) [shadertoy] shaders is:

1. Create a new template first (see installation-examples: provide the `--template` argument and a path with a `.glsl` extension).
2. Replace, _starting_ from the `main` function, the code with the code from the [shadertoy]-shader.
3. Rename `fragCoord` to `gl_FragCoord`
4. Change the entry-function signature from `void mainImage( out vec4 fragColor, in vec2 fragCoord )` to `void main()`

And you should be good to go.

# Other notes

`shady-toy` is not as mature as [shadertoy]. If you want to solid experience with many features and just want to write (epic) opengl shaders then [shadertoy]
is the way to go.

[shadertoy]: https://www.shadertoy.com/
[glsl]: https://www.khronos.org/opengl/wiki/Core_Language_(GLSL)
[wgsl]: https://www.w3.org/TR/WGSL/
[rust]: https://www.rust-lang.org/
[shader-examples]: https://github.com/TornaxO7/shady/tree/main/shady-toy/shader-examples
