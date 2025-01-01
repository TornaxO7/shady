# Shady

Shady is a rust-libraray to integrate [shadertoy](-like) easier into your app. It also ships with a default application
which uses the library: [shady-toy] which is basically like a local [shadertoy]-desktop-app.

It supports fragment shaders written in [wgsl] **_and_** [glsl]. The app determines which shader-language is used by
looking at the file-extension like `fragment.wgsl` or `fragment.glsl`.

There's also a [cava] like app: [shady-cli] which also displays bars on the terminal as [cava] but it's not as feature complete as [cava].

# Usage

In general `shady-toy <path> --template` will likely be what you want to execute. For example:

```bash
# if you want to write the fragment shader in `wgsl`
shady-toy /tmp/fragment.wgsl --template

# if you want to write the fragment shade rin `glsl`
shady-toy /tmp/fragment.glsl --template
```

## `nix`

If you are using `nix` with flakes: Simply run `nix run github:TornaxO7/shady -- <path to shader> --template` and you are good to go.

## Build it on your own

You need to install [rust] opengl/vulkan and then you can run:

```bash
cargo run --release -- <path> --template
```

# Features

- Live reloading (after save)
- Audio buffer

Currently implemented uniform buffers:

- `iTime`
- `iResolution`
- `iFrame`
- `iAudio` (with 10 elements, for the time being)
- `iMouse` (although not really tested)

# Demo

A demo can be seen [here](https://filebrowser.tornaxo7.de/api/public/dl/LB5bVE74?inline=true).

# Examples

- The template itself is an example if you run `shady <path> --template`, you can start writing your shader
- See in `shader-examples/` (run `shady-toy shader-examples/<file>`)

# Shadertoy

Currently, you can't just copy+paste the shaders from [shadertoy] due to some differences how [shady-toy] and [shadertoy] are using the fragment shaders.
So for the time being, here's a little (hopefully full) checklist/guide about how to get a shader from [shadertoy] up and running by using `shady-toy`

1. Make sure that the shader from [shadertoy] uses the uniform buffers listed in `# Features` _at most_.
2. Start by creating a `glsl` template by adding the `--template` argument (for example: `shady-toy /tmp/fragment.glsl --template`).
3. Open the fragment shader.
4. _Don't_ remove the `uniform` lines from the template!
5. Copy+Paste the [shadertoy]-fragment by replacing the `main` function.
6. Change the function `mainImage` to `void main() { ... }`
7. Replace `fragCoord` with `gl_FragCoord`.
8. You should be good to go (in most cases)

An example can be seen in `shader-examples`. Inside, there's a link to the original shader and author.

# Troubleshooting

## `shady` audio doesn't listen to my systems audio

Currently `shady` is listening to your default output device.
Take a look into your settings (for example with [pavucontrol], under "Recording") if shady is listening to the correct source.
For example on my system it looks like this (after starting [pavucontrol]):

![Example](./assets/shady_audio_settings.png)

# Status

You are able to write shaders similar to [shadertoy] and you have the uniform values `iTime` and `iResolution` but also `iAudio` if you want to create something with music visualisation.
However it's mostly unstable and unpolished (for example [gamma correction] is missing), that's why I'm not creating a release (yet?) for the lib but also for the app.

[shadertoy]: https://www.shadertoy.com/
[shady-toy]: https://github.com/TornaxO7/shady/tree/main/shady-toy
[wgsl]: https://www.w3.org/TR/WGSL/
[pavucontrol]: https://github.com/pulseaudio/pavucontrol
[gamma correction]: https://en.wikipedia.org/wiki/Gamma_correction
[rust]: https://www.rust-lang.org/
[glsl]: https://www.khronos.org/opengl/wiki/Core_Language_(GLSL)
[cava]: https://github.com/karlstav/cava
[shady-cli]: https://github.com/TornaxO7/shady/tree/main/shady-cli
