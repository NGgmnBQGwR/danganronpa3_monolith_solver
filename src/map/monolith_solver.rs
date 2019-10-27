use super::monolith_map::{MonolithMap, SolvedPath, Tile};
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use crossbeam::queue::ArrayQueue;

/// Recursive Random Singlethreaded Unbounbed Bruteforce
pub fn solve_1(map: MonolithMap) -> Vec<Tile> {
    fn random_walk(
        results: &mut Vec<SolvedPath>,
        steps: Vec<Tile>,
        map: MonolithMap,
        rng: &mut ThreadRng,
    ) -> (u32, Vec<Tile>) {
        let mut groups = map.all_groups();
        if groups.is_empty() {
            let count = map.get_dead_tiles_count();
            (count, steps)
        } else {
            groups.shuffle(rng);
            let first_tile = groups[0][0];

            let mut new_map = map.clone();
            new_map.click(first_tile.0, first_tile.1);

            let new_steps = {
                let mut temp = steps.clone();
                temp.push(first_tile);
                temp
            };
            random_walk(results, new_steps, new_map, rng)
        }
    }

    let mut results: Vec<SolvedPath> = Vec::with_capacity(100);
    let map = map.clone();
    let mut rng = thread_rng();
    let start = Instant::now();
    loop {
        let new_map = map.clone();
        let (count, steps) = random_walk(&mut results, Vec::new(), new_map, &mut rng);
        if results.is_empty() || count < results.first().unwrap().0 {
            println!("Found result with {} tiles remaining.", count);
            results.push((count, steps));
            results.sort();
        };
        if count == 0 {
            break;
        } else {
            let best = results.first().unwrap().0;
            let elapsed = start.elapsed().as_secs();
            if elapsed > 10 && best < 5
                || elapsed > 30 && best < 8
                || elapsed > 60 && best < 10
                || elapsed > 120 && best < 15
                || elapsed > 300
            {
                break;
            }
        }
    }
    results.reverse();
    results.pop().unwrap_or_default().1
}

/// Recursive Sequential Singlethreaded Bounded Bruteforce
pub fn solve_2(map: MonolithMap) -> Vec<Tile> {
    fn work(
        results: &mut Vec<SolvedPath>,
        steps: Vec<Tile>,
        map: MonolithMap,
        dead_tiles_limit: u32,
    ) {
        let groups = map.all_groups();
        if groups.is_empty() {
            let count = map.get_dead_tiles_count();

            if results.is_empty() || count < results.first().unwrap().0 {
                results.push((count, steps));
                results.sort();
            }
        } else {
            for group in groups {
                let first_tile = group[0];

                let mut new_map = map.clone();
                new_map.click(first_tile.0, first_tile.1);
                if new_map.get_dead_tiles_count() > dead_tiles_limit {
                    continue;
                }

                let new_steps = {
                    let mut temp = steps.clone();
                    temp.push(first_tile);
                    temp
                };
                work(results, new_steps, new_map, dead_tiles_limit);
            }
        }
    }

    let mut results: Vec<SolvedPath> = Vec::with_capacity(100);
    for max_dead_tiles_allowed in [0u32, 5, 10, 15, 20].iter() {
        println!(
            "Trying to find solution with <= {} dead tiles.",
            max_dead_tiles_allowed
        );
        let map = map.clone();
        work(&mut results, Vec::new(), map, *max_dead_tiles_allowed);
        if !results.is_empty() {
            break;
        }
    }
    results.reverse();
    results.pop().unwrap_or_default().1
}

/// Recursive Singlethreaded N-Step Lookahead
pub fn solve_3(map: MonolithMap) -> Vec<Tile> {
    type SolvedPath = (u32, Vec<Tile>);

    fn walk(map: MonolithMap, steps: Vec<Tile>, depth_to_go: u32) -> SolvedPath {
        let groups = map.all_groups();
        if groups.is_empty() {
            let count = map.get_dead_tiles_count();
            (count, steps)
        } else {
            let mut results = Vec::<SolvedPath>::with_capacity(100);
            for group in groups {
                let first_tile = group[0];

                let mut new_map = map.clone();
                new_map.click(first_tile.0, first_tile.1);

                let new_steps = {
                    let mut temp = steps.clone();
                    temp.push(first_tile);
                    temp
                };

                if depth_to_go == 0 {
                    let count = map.get_dead_tiles_count();
                    results.push((count, new_steps));
                } else {
                    let (count, best_steps) = walk(new_map, new_steps, depth_to_go - 1);
                    results.push((count, best_steps));
                }
            }
            results.sort();
            results.reverse();
            results.pop().unwrap_or_default()
        }
    }
    let mut results = Vec::<SolvedPath>::with_capacity(5);

    for lookahead in &[0, 1, 2, 3, 4] {
        println!("Trying {}-step lookahead...", lookahead);
        let new_map = map.clone();
        let (count, steps) = walk(new_map, Vec::with_capacity(50), *lookahead);
        results.push((count, steps));
        println!("Found result with {} tiles remaining.", count);
    }
    results.sort();
    results.reverse();
    results.pop().unwrap_or_default().1
}

/// Recursive Sequential Multithreaded Bruteforce
pub fn solve_4(map: MonolithMap) -> Vec<Tile> {
    fn brute_solver(
        job_queue: Arc<ArrayQueue<(Vec<Tile>, MonolithMap)>>,
        result_queue: Arc<ArrayQueue<(u32, Vec<Tile>)>>,
    ) {
        let max_dead_tiles_allowed = 20;
        loop {
            let (steps, map) = match job_queue.pop() {
                Ok(job) => job,
                Err(_) => {
                    thread::sleep(Duration::from_millis(1_000));
                    match job_queue.pop() {
                        Ok(job) => job,
                        Err(_) => return,
                    }
                }
            };

            let groups = map.all_groups();
            if groups.is_empty() {
                let count = map.get_dead_tiles_count();

                if result_queue.is_empty() || count < max_dead_tiles_allowed {
                    let res = result_queue.push((count, steps));
                    if res.is_err() {
                        return;
                    }
                }
            } else {
                for group in groups {
                    let first_tile = group[0];
                    let mut new_steps = steps.clone();
                    new_steps.push(first_tile);
                    let mut new_map = map.clone();
                    new_map.click(first_tile.0, first_tile.1);
                    while job_queue.is_full() {
                        thread::sleep(Duration::from_millis(100));
                    }
                    job_queue
                        .push((new_steps, new_map))
                        .expect("Failed to push a new job.");
                }
            }
        }
    };
    let job_queue = Arc::new(ArrayQueue::new(1000));
    job_queue
        .push((Vec::<Tile>::new(), map))
        .expect("Failed to push starting value.");
    let result_queue = Arc::new(ArrayQueue::new(100));

    let workers: Vec<_> = (1..16)
        .map(|_| {
            let q1 = job_queue.clone();
            let q2 = result_queue.clone();
            thread::spawn(|| brute_solver(q1, q2))
        })
        .collect();

    for worker in workers {
        worker.join().expect("Failed to join on a thread handle.");
    }

    let mut results = Vec::new();
    while let Ok(value) = result_queue.pop() {
        results.push(value);
    }
    results.sort();
    results.reverse();
    results.pop().unwrap_or_default().1
}
