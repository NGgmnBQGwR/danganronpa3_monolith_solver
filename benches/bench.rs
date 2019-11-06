#![allow(unused_variables)]

use monolith_solver::map::MonolithMap;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn get_map() -> MonolithMap {
    MonolithMap{
        0: [// 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
            [2,2,4,4,4,3,3,1,4,3,4,2,3,4,0,3,3,4,3,4,1,4], // 0
            [4,3,3,1,3,4,2,4,4,4,2,1,2,1,0,4,4,4,3,2,3,4], // 1
            [1,3,4,3,2,3,2,1,2,3,3,1,3,3,0,2,3,0,0,0,0,1], // 2
            [2,2,1,1,2,1,1,4,1,1,3,2,1,1,0,1,1,0,2,1,2,2], // 3
            [2,3,4,4,1,4,1,4,3,4,3,4,3,4,0,4,4,0,0,0,1,4], // 4
            [3,4,4,1,4,3,3,4,4,1,3,3,4,4,0,2,3,3,2,0,2,3], // 5
            [3,2,3,2,2,1,2,1,2,4,2,2,2,3,0,1,1,4,1,0,2,4], // 6
            [1,2,1,1,2,3,3,2,1,2,1,1,1,2,1,0,0,0,0,0,1,3], // 7
            [4,3,1,3,4,3,3,2,3,3,1,2,4,4,2,0,3,4,4,3,1,3], // 8
            [4,4,2,1,3,4,1,4,4,4,1,4,2,4,0,3,1,4,3,2,4,2], // 9
            [1,3,2,2,2,2,1,1,3,1,2,2,1,0,1,1,2,2,1,1,2,4], // 10
        ]
    }
}

pub fn test1(c: &mut Criterion) {
    let map = get_map();
    // c.bench_function("cluster_before", |b| b.iter(|| map.get_tile_cluster(21,0)));
    // c.bench_function("cluster_after", |b| b.iter(|| map.get_tile_cluster2(21,0)));
    // c.bench_function("dead_before", |b| b.iter(|| map.get_dead_tiles_count()));
    // c.bench_function("dead_after", |b| b.iter(|| map.get_dead_tiles_count2()));
}
criterion_group!(benches, test1);
criterion_main!(benches);
