use crate::map::{MonolithMap, Tile};
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::time::{Duration, Instant};

pub trait RecursiveRandomBruteforce {
    fn solve_recursive_random_bruteforce(self) -> Vec<Tile>;
}

impl RecursiveRandomBruteforce for MonolithMap {
    fn solve_recursive_random_bruteforce(self) -> Vec<Tile> {
        type SolvedPath = (u32, Vec<Tile>);

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
        let map = self.clone();
        let mut rng = thread_rng();
        let start = Instant::now();
        loop {
            let new_map = map.clone();
            let (count, steps) = random_walk(&mut results, Vec::new(), new_map, &mut rng);
            if results.is_empty() || count < results.first().unwrap().0 {
                println!("Found result with {} dead tiles remaining.", count);
                results.push((count, steps));
                results.sort();
            };
            if count == 0 {
                break;
            } else {
                let best = results.first().unwrap().0;
                let elapsed = start.elapsed().as_secs();
                if elapsed > 10 && best < 5 || elapsed > 20 && best < 8 || elapsed > 30 {
                    break;
                }
            }
        }
        results.reverse();
        results.pop().unwrap_or_default().1
    }
}
