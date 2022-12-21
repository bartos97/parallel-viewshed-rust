#[allow(unused)]

use criterion::{ criterion_group, criterion_main, Criterion, BenchmarkId };
use parallel_viewshed_rust::mesh_splitter::MeshSplitter;

pub fn criterion_benchmark(c: &mut Criterion) {
    // c.bench_function("mesh_splitter wave 2x2", |bencher| {
    //     bencher.iter(|| {
    //         let mut splitter = MeshSplitter::new("resources/wave.obj", 2);
    //         splitter.run_splitter();
    //     })
    // });

    c.bench_function("mesh_splitter blob 10x10", |bencher| {
        bencher.iter(|| {
            let mut splitter = MeshSplitter::new("resources/blob.obj", 10);
            splitter.run_splitter();
        })
    });

    // let mut group = c.benchmark_group("mesh_splitter blob chunks range");
    // let chunks_range: [usize; 3] = [2, 5, 10];
    // for chunks in chunks_range.iter() {
    //     group.bench_with_input(BenchmarkId::from_parameter(chunks), chunks, |b, &chunks| {
    //         b.iter(|| {
    //             let mut splitter = MeshSplitter::new("resources/blob.obj", chunks);
    //             splitter.run_splitter();
    //         });
    //     });
    // }
    // group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);