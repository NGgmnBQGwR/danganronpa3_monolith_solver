use crate::map::{MonolithMap, Tile};

pub trait RecursiveLesserThanNDead {
    fn solve_recursive_lesser_n_dead(self) -> Vec<Tile>;
}

impl RecursiveLesserThanNDead for MonolithMap {
    fn solve_recursive_lesser_n_dead(self) -> Vec<Tile> {
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
            let new_map = self.clone();
            let (count, steps) = walk(new_map, Vec::with_capacity(50), *lookahead);
            results.push((count, steps));
            println!("Found result with {} tiles remaining.", count);
        }
        results.sort();
        results.reverse();
        results.pop().unwrap_or_default().1
    }
}
