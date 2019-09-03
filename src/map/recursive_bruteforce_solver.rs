use crate::map::{MonolithMap, Tile};

pub trait RecursiveBruteforce {
    fn solve_recursive_bruteforce(self) -> Vec<Tile>;
}

impl RecursiveBruteforce for MonolithMap {
    fn solve_recursive_bruteforce(self) -> Vec<Tile> {
        type SolvedPath = (u32, Vec<Tile>);

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
            println!("Trying to find solution with <= {} dead tiles.", max_dead_tiles_allowed);
            let map = self.clone();
            work(&mut results, Vec::new(), map, *max_dead_tiles_allowed);
            if !results.is_empty() {
                break;
            }
        }
        results.reverse();
        results.pop().unwrap_or_default().1
    }
}
