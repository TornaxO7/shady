use criterion::{criterion_group, criterion_main, Criterion};
use shady::GlslFrontend;
use shady_app::renderer::Renderer;
use winit::event_loop::EventLoop;

fn octagrams() {
    let octagram = include_str!("octagrams.glsl");
    let mut renderer: Renderer<GlslFrontend> = Renderer::with_fragment(octagram);
}

fn benchmark(c: &mut Criterion) {
    c.bench_function("octagrams", |b| b.iter(|| octagrams()));
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
