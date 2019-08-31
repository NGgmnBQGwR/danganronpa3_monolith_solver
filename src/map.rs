use std::collections::HashSet;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crossbeam::queue::ArrayQueue;
use serde::{Deserialize, Serialize};

pub type Tile = (usize, usize);

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct MonolithMap(pub [[u8; 22]; 11]);

impl Default for MonolithMap {
    fn default() -> Self {
        MonolithMap { 0: [[0; 22]; 11] }
    }
}

impl MonolithMap {
    pub fn solve(self) -> Vec<Tile> {
        self.solve_recursive_bruteforce()
    }

    fn get(&self, x: usize, y: usize) -> u8 {
        self.0[y][x]
    }

    pub fn set(&mut self, x: usize, y: usize, value: u8) {
        debug_assert!(value <= 4);
        self.0[y][x] = value;
    }

    fn click(&mut self, x: usize, y: usize) {
        if self.get(x, y) == 0 {
            return;
        }

        let group = self.get_group(x, y);
        if group.len() == 1 {
            return;
        }

        for tile in &group {
            self.set(tile.0, tile.1, 0);
        }
        let mut advanced = HashSet::with_capacity(50);
        for tile in group {
            for neighbor in self.get_neighbors(tile.0, tile.1) {
                if !advanced.contains(&(neighbor.0, neighbor.1)) {
                    self.advance(neighbor.0, neighbor.1);
                    advanced.insert((neighbor.0, neighbor.1));
                }
            }
        }
    }

    fn advance(&mut self, x: usize, y: usize) {
        match self.get(x, y) {
            1 => self.set(x, y, 2),
            2 => self.set(x, y, 3),
            3 => self.set(x, y, 4),
            4 => self.set(x, y, 1),
            0 => (),
            _ => unreachable!(),
        }
    }

    fn get_neighbors(&self, x: usize, y: usize) -> Vec<Tile> {
        let mut neighbors = Vec::with_capacity(4);
        let max_y = self.0.len() - 1;
        let max_x = self.0[0].len() - 1;

        // above
        if y > 0 && self.get(x, y-1) != 0 {
            neighbors.push((x, y - 1));
        }
        // below
        if y < max_y && self.get(x, y+1) != 0 {
            neighbors.push((x, y + 1));
        }
        // left
        if x > 0 && self.get(x-1, y) != 0 {
            neighbors.push((x - 1, y));
        }
        // right
        if x < max_x && self.get(x+1, y) != 0 {
            neighbors.push((x + 1, y));
        }
        neighbors
    }

    fn all_groups(&self) -> Vec<Vec<Tile>> {
        let mut groups = Vec::with_capacity(30);
        let mut visited = HashSet::with_capacity(200);
        let max_y = self.0.len();
        let max_x = self.0[0].len();

        for x in 0..max_x {
            for y in 0..max_y {
                if self.get(x, y) != 0 && !visited.contains(&(x, y)) {
                    let group = self.get_group(x, y);
                    if !group.is_empty() {
                        for (x, y) in group.iter() {
                            visited.insert((*x, *y));
                        }
                        groups.push(group);
                    }
                }
            }
        }
        groups
    }

    fn get_group(&self, x: usize, y: usize) -> Vec<Tile> {
        let mut group = HashSet::with_capacity(10);

        let group_type = self.get(x, y);
        if group_type == 0 {
            return group.into_iter().collect();
        }

        let mut todo = Vec::with_capacity(10);
        todo.push((x, y));
        group.insert((x, y));
        let max_y = self.0.len() - 1;
        let max_x = self.0[0].len() - 1;

        while let Some(tile) = todo.pop() {
            let x = tile.0;
            let y = tile.1;
            // above
            if y > 0 && self.get(x, y - 1) == group_type && !group.contains(&(x, y - 1)) {
                todo.push((x, y - 1));
                group.insert((x, y - 1));
            }
            // below
            if y < max_y && self.get(x, y + 1) == group_type && !group.contains(&(x, y + 1)) {
                todo.push((x, y + 1));
                group.insert((x, y + 1));
            }
            // left
            if x > 0 && self.get(x - 1, y) == group_type && !group.contains(&(x - 1, y)) {
                todo.push((x - 1, y));
                group.insert((x - 1, y));
            }
            // right
            if x < max_x && self.get(x + 1, y) == group_type && !group.contains(&(x + 1, y)) {
                todo.push((x + 1, y));
                group.insert((x + 1, y));
            }
        }
        let group: Vec<Tile> = group.into_iter().collect();
        if group.len() == 1 {
            Vec::new()
        } else {
            group
        }
    }

    fn get_single_tiles(&self) -> Vec<Tile> {
        let max_y = self.0.len();
        let max_x = self.0[0].len();
        let mut result = Vec::with_capacity(30);

        for x in 0..max_x {
            for y in 0..max_y {
                if self.get(x, y) != 0 {
                    let group = self.get_group(x, y);
                    if group.is_empty(){
                        result.push((x,y));
                    }
                }
            }
        }
        result
    }

    fn get_dead_tiles_count(&self) -> u32 {
        let max_y = self.0.len();
        let max_x = self.0[0].len();
        let mut count = 0;

        for x in 0..max_x {
            for y in 0..max_y {
                if self.get(x, y) != 0 {
                    let neighbors = self.get_neighbors(x, y);
                    if neighbors.is_empty(){
                        count += 1;
                    }
                }
            }
        }
        count
    }

    pub fn solve_recursive_bruteforce(self) -> Vec<Tile> {
        type SolvedPath = (u32, Vec<Tile>);

        fn work(
            results: &mut Vec<SolvedPath>,
            steps: Vec<Tile>,
            map: MonolithMap,
            dead_cells_limit: u32,
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
                    if new_map.get_dead_tiles_count() > dead_cells_limit {
                        continue;
                    }

                    let new_steps = {
                        let mut temp = steps.clone();
                        temp.push(first_tile);
                        temp
                    };
                    work(results, new_steps, new_map, dead_cells_limit);
                }
            }
        }

        let mut results: Vec<SolvedPath> = Vec::with_capacity(100);
        for max_dead_cells_allowed in [5u32, 10, 15, 20].iter() {
            let map = self.clone();
            work(&mut results, Vec::new(), map, *max_dead_cells_allowed);
            if !results.is_empty() {
                break;
            }
        }
        results.reverse();
        results.pop().unwrap_or_default().1
    }

    pub fn solve_threaded_bruteforce(self) -> Vec<Tile> {
        fn brute_solver(
            job_queue: Arc<ArrayQueue<(Vec<Tile>, MonolithMap)>>,
            result_queue: Arc<ArrayQueue<(u32, Vec<Tile>)>>,
        ) {
            let max_dead_cells_allowed = 20;
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

                    if result_queue.is_empty() || count < max_dead_cells_allowed {
                        let res = result_queue.push((count, steps));
                        if res.is_err(){
                            return
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
                        job_queue.push((new_steps, new_map)).expect("Failed to push a new job.");
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

#[cfg(test)]
mod test {
    use super::{MonolithMap, Tile};

    #[test]
    fn test_solve_1_step(){
        let map = MonolithMap{
            0: [// 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 0
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 1
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 2
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 3
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 4
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 5
                [0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0], // 6
                [0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0], // 7
                [0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0], // 8
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 9
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 10
            ]
        };
        let steps = map.solve();
        assert_eq!(steps.len(), 1);
        let correct_step_1: Vec<Tile> = vec![(8, 8), (8, 7), (8, 6)];
        assert!(correct_step_1.contains(&steps[0]));
    }

    #[test]
    fn test_solve_2_step(){
        let map = MonolithMap{
            0: [// 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 0
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 1
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 2
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 3
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 4
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 5
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 6
                [0,0,0,0,0,0,0,3,2,0,0,0,0,0,0,0,0,0,0,0,0,0], // 7
                [0,0,0,0,0,0,0,0,1,1,0,0,0,0,0,0,0,0,0,0,0,0], // 8
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 9
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 10
            ]
        };
        let steps = map.solve();
        assert_eq!(steps.len(), 2);
        let correct_step_1: Vec<Tile> = vec![(8, 8), (9, 8)];
        assert!(correct_step_1.contains(&steps[0]));
        let correct_step_2: Vec<Tile> = vec![(7, 7), (8, 7)];
        assert!(correct_step_2.contains(&steps[1]));
    }


    #[test]
    fn test_solve_2_step_with_alternative(){
        let map = MonolithMap{
            0: [// 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 0
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 1
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 2
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 3
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 4
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 5
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 6
                [0,0,0,0,0,0,3,3,2,0,0,0,0,0,0,0,0,0,0,0,0,0], // 7
                [0,0,0,0,0,0,0,0,1,1,0,0,0,0,0,0,0,2,4,2,0,0], // 8
                [0,0,3,0,0,0,0,0,3,2,0,0,0,0,0,0,0,0,0,0,0,0], // 9
                [0,0,4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 10
            ]
        };
        let steps = map.solve();
        assert_eq!(steps.len(), 2);
        let correct_step_1: Vec<Tile> = vec![(8, 8), (9, 8)];
        assert!(correct_step_1.contains(&steps[0]));
        let correct_step_2: Vec<Tile> = vec![(6, 7), (7, 7), (8, 7)];
        assert!(correct_step_2.contains(&steps[1]));
    }

    #[test]
    fn test_advance_1_tile(){
        let mut map = MonolithMap{
            0: [// 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 0
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 1
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 2
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 3
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 4
                [0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0], // 5
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 6
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 7
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 8
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 9
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 10
            ]
        };

        map.advance(10, 5);

        let map_after = MonolithMap{
            0: [// 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 0
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 1
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 2
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 3
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 4
                [0,0,0,0,0,0,0,0,0,0,2,0,0,0,0,0,0,0,0,0,0,0], // 5
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 6
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 7
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 8
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 9
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 10
            ]
        };
        assert_eq!(map, map_after);
    }

    #[test]
    fn test_click_tile(){
        let mut map = MonolithMap{
            0: [// 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 0
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 1
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 2
                [0,0,0,0,0,0,0,0,0,4,4,0,0,0,0,0,0,0,0,0,0,0], // 3
                [0,0,0,0,0,0,0,0,0,2,1,3,0,0,0,0,0,0,0,0,0,0], // 4
                [0,0,0,0,0,0,0,0,0,2,1,3,0,0,0,0,0,0,0,0,0,0], // 5
                [0,0,0,0,0,0,0,0,0,0,2,0,0,0,0,0,0,0,0,0,0,0], // 6
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 7
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 8
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 9
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 10
            ]
        };

        map.click(10, 4);

        let mut map_after = MonolithMap{
            0: [// 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 0
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 1
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 2
                [0,0,0,0,0,0,0,0,0,4,1,0,0,0,0,0,0,0,0,0,0,0], // 3
                [0,0,0,0,0,0,0,0,0,3,0,4,0,0,0,0,0,0,0,0,0,0], // 4
                [0,0,0,0,0,0,0,0,0,3,0,4,0,0,0,0,0,0,0,0,0,0], // 5
                [0,0,0,0,0,0,0,0,0,0,3,0,0,0,0,0,0,0,0,0,0,0], // 6
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 7
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 8
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 9
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 10
            ]
        };
        assert_eq!(map, map_after);

        map_after.click(9, 5);

        let map_after_after = MonolithMap{
            0: [// 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 0
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 1
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 2
                [0,0,0,0,0,0,0,0,0,1,1,0,0,0,0,0,0,0,0,0,0,0], // 3
                [0,0,0,0,0,0,0,0,0,0,0,4,0,0,0,0,0,0,0,0,0,0], // 4
                [0,0,0,0,0,0,0,0,0,0,0,4,0,0,0,0,0,0,0,0,0,0], // 5
                [0,0,0,0,0,0,0,0,0,0,3,0,0,0,0,0,0,0,0,0,0,0], // 6
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 7
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 8
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 9
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 10
            ]
        };
        assert_eq!(map_after, map_after_after);
    }


    #[test]
    fn test_click_tile_corner(){
        let mut map = MonolithMap{
            0: [// 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
                [1,4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 0
                [4,4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 1
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 2
                [0,0,0,0,0,0,0,0,0,4,4,0,0,0,0,0,0,0,0,0,0,0], // 3
                [0,0,0,0,0,0,0,0,0,2,1,3,0,0,0,0,0,0,0,0,0,0], // 4
                [0,0,0,0,0,0,0,0,0,2,1,3,0,0,0,0,0,0,0,0,0,0], // 5
                [0,0,0,0,0,0,0,0,0,0,2,0,0,0,0,0,0,0,0,0,0,0], // 6
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 7
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 8
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 9
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 10
            ]
        };

        map.click(1, 0);

        let map_after = MonolithMap{
            0: [// 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
                [2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 0
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 1
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 2
                [0,0,0,0,0,0,0,0,0,4,4,0,0,0,0,0,0,0,0,0,0,0], // 3
                [0,0,0,0,0,0,0,0,0,2,1,3,0,0,0,0,0,0,0,0,0,0], // 4
                [0,0,0,0,0,0,0,0,0,2,1,3,0,0,0,0,0,0,0,0,0,0], // 5
                [0,0,0,0,0,0,0,0,0,0,2,0,0,0,0,0,0,0,0,0,0,0], // 6
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 7
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 8
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 9
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 10
            ]
        };
        assert_eq!(map, map_after);
    }

    #[test]
    fn test_get_neighbors(){
        let map = MonolithMap{
            0: [// 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
                [3,2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2,1], // 0
                [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,3], // 1
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 2
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 3
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 4
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 5
                [0,0,0,0,0,0,0,0,0,0,0,2,0,0,0,0,0,0,0,0,0,0], // 6
                [0,0,0,0,0,0,0,0,0,0,2,2,2,0,0,0,0,0,0,0,0,0], // 7
                [0,3,0,0,0,0,0,0,0,0,0,2,0,0,0,0,0,0,0,0,0,0], // 8
                [2,1,4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1], // 9
                [0,1,0,0,0,0,0,0,0,0,0,0,0,0,3,0,0,0,0,0,1,1], // 10
            ]
        };

        {
            let neighbors = map.get_neighbors(14, 10);
            assert_eq!(neighbors.len(), 0);
        }
        {
            let mut neighbors = map.get_neighbors(0, 0);
            neighbors.sort();
            assert_eq!(neighbors.len(), 2);
            assert_eq!(neighbors, vec![(0, 1), (1, 0)]);
        }
        {
            let mut neighbors = map.get_neighbors(1, 9);
            neighbors.sort();
            assert_eq!(neighbors.len(), 4);
            assert_eq!(neighbors, vec![(0, 9), (1, 8), (1, 10), (2, 9)]);
        }
        {
            let mut neighbors = map.get_neighbors(21, 10);
            neighbors.sort();
            assert_eq!(neighbors.len(), 2);
            assert_eq!(neighbors, vec![(20, 10), (21, 9)]);
        }
        {
            let mut neighbors = map.get_neighbors(21, 0);
            neighbors.sort();
            assert_eq!(neighbors.len(), 2);
            assert_eq!(neighbors, vec![(20, 0), (21, 1)]);
        }
        {
            let mut neighbors = map.get_neighbors(11, 7);
            neighbors.sort();
            assert_eq!(neighbors.len(), 4);
            assert_eq!(neighbors, vec![(10, 7), (11, 6), (11, 8), (12, 7)]);
        }
    }

    #[test]
    fn test_get_group(){
        let map = MonolithMap{
            0: [// 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
                [1,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,4,1,4,4,4,4], // 0
                [2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,3,4,4,2,4,2], // 1
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,4,2,4,4,4,3], // 2
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,4,1,4,0], // 3
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,4,1,0,4], // 4
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2,0,0,0], // 5
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 6
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 7
                [0,0,0,0,0,0,2,2,2,0,0,0,0,0,0,0,0,0,0,0,0,0], // 8
                [0,0,0,0,0,2,3,3,3,2,0,0,0,0,0,0,0,0,0,0,0,0], // 9
                [0,0,0,0,0,0,2,2,2,0,0,0,0,0,0,0,0,0,0,0,0,0], // 10
            ]
        };
        {
            let mut group = map.get_group(0, 0);
            group.sort();
            assert_eq!(map.get(0, 0), 1);
            assert_eq!(group.len(), 2);
            assert_eq!(group, vec![(0, 0), (1, 0)]);
            assert!(group.iter().all(|x| map.get(x.0, x.1) == 1));
        }
        {
            let mut group = map.get_group(6, 9);
            group.sort();
            assert_eq!(map.get(6, 9), 3);
            assert_eq!(group.len(), 3);
            assert!(group.iter().all(|x| map.get(x.0, x.1) == 3));
            assert_eq!(group, vec![(6, 9), (7, 9), (8, 9)]);
        }
        {
            let mut group = map.get_group(20, 0);
            group.sort();
            assert_eq!(map.get(20, 0), 4);
            assert_eq!(group.len(), 13);
            assert!(group.iter().all(|x| map.get(x.0, x.1) == 4));
        }
    }

    #[test]
    fn test_all_groups(){
        let map = MonolithMap{
            0: [// 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
                [3,2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2,1], // 0
                [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,3], // 1
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 2
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 3
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 4
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 5
                [0,0,0,0,0,0,0,0,0,0,0,2,0,0,0,0,0,0,0,0,0,0], // 6
                [0,0,0,0,0,0,0,0,0,0,2,2,2,0,0,0,0,0,0,0,0,0], // 7
                [0,3,0,0,0,0,0,0,0,0,0,2,0,0,0,0,0,0,0,0,0,0], // 8
                [2,1,4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1], // 9
                [0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,1], // 10
            ]
        };

        let groups = map.all_groups();
        assert_eq!(groups.len(), 3);
    }

    #[test]
    fn test_get_single_tiles() {
        let map = MonolithMap{
            0: [// 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
                [3,2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2,1], // 0
                [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,3], // 1
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 2
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 3
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 4
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 5
                [0,0,0,0,0,0,0,0,0,0,0,2,0,0,0,0,0,0,0,0,0,0], // 6
                [0,0,0,0,0,0,0,0,0,0,2,2,2,0,0,0,0,0,0,0,0,0], // 7
                [0,3,0,0,0,0,0,0,0,0,0,2,0,0,0,0,0,0,0,0,0,0], // 8
                [2,1,4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1], // 9
                [0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,1], // 10
            ]
        };

        let single_tiles = map.get_single_tiles().len();
        assert_eq!(single_tiles, 9);
    }

    #[test]
    fn test_get_dead_tiles_count_1(){
        let map = MonolithMap{
            0: [// 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
                [3,2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2,1], // 0
                [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,3], // 1
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 2
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 3
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 4
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 5
                [0,0,0,0,0,0,0,0,0,0,0,2,0,0,0,0,0,0,0,0,0,0], // 6
                [0,0,0,0,0,0,0,0,0,0,2,2,2,0,0,0,0,0,0,0,0,0], // 7
                [0,3,0,0,0,0,0,0,0,0,0,2,0,0,0,0,0,0,0,0,0,0], // 8
                [2,1,4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1], // 9
                [0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,1], // 10
            ]
        };

        let dead_tiles = map.get_dead_tiles_count();
        assert_eq!(dead_tiles, 0);
    }

    #[test]
    fn test_get_dead_tiles_count_2(){
        let map = MonolithMap{
            0: [// 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
                [3,2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2,1], // 0
                [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,3], // 1
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 2
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 3
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 4
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 5
                [0,0,0,0,0,0,0,0,0,0,0,2,0,0,0,0,0,0,0,0,0,0], // 6
                [0,0,0,0,0,0,0,0,0,0,2,2,2,0,0,0,0,0,0,0,0,0], // 7
                [0,3,0,0,0,0,0,0,0,0,0,2,0,0,0,0,0,0,0,0,0,0], // 8
                [0,0,4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1], // 9
                [3,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,1,0], // 10
            ]
        };

        let dead_tiles = map.get_dead_tiles_count();
        assert_eq!(dead_tiles, 6);
    }
}
