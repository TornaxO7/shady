# `shady-lib`

The main library which takes care of the uniform/storage buffers, vertices and templates.

The idea is that other applications who wish to include [shadertoy]-like shaders into their
application to use this library which takes care most of the data to be able to run those shaders.

# State

It's useable, however I'm a bit unsure about the architecture because I don't really know what
a good API looks like for a graphics-programmer.

# Example

A simple example can be seen here: <https://github.com/TornaxO7/shady/blob/main/shady-lib/examples/mini-simple.rs> if you want
to include it to your app. All relevant places where you have to "interact" with shady are annoted with the `// SHADY` comments.

[shadertoy]: https://www.youtube.com/watch?v=Xdbk1Pr5WXU&list=PLpM-Dvs8t0Vak1rrE2NJn8XYEJ5M7-BqT
