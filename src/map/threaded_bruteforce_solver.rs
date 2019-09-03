use crate::map::{Tile, MonolithMap};

use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crossbeam::queue::ArrayQueue;


pub trait ThreadedBruteforce {
    fn solve_threaded_bruteforce(self) -> Vec<Tile>;
}

impl ThreadedBruteforce for MonolithMap {
    fn solve_threaded_bruteforce(self) -> Vec<Tile> {
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
            .push((Vec::<Tile>::new(), self))
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
}
