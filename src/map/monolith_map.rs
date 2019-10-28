use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::convert::TryInto;

use super::monolith_solver;

pub type Tile = (usize, usize);

pub type SolvedPath = (u32, Vec<Tile>);

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct MonolithMap(pub [[u8; 22]; 11]);

impl Default for MonolithMap {
    fn default() -> Self {
        MonolithMap { 0: [[0; 22]; 11] }
    }
}

impl MonolithMap {
    pub fn solve(self) -> Vec<Tile> {
        monolith_solver::solve_1(self)
    }

    fn get(&self, x: usize, y: usize) -> u8 {
        self.0[y][x]
    }

    pub fn set(&mut self, x: usize, y: usize, value: u8) {
        debug_assert!(value <= 4);
        self.0[y][x] = value;
    }

    pub fn click(&mut self, x: usize, y: usize) {
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

    fn has_neighbors(&self, x: usize, y: usize) -> bool {
        let max_y = self.0.len() - 1;
        let max_x = self.0[0].len() - 1;

        self.get(x, y) != 0
            && ((y > 0 && self.get(x, y - 1) != 0)
                || (y < max_y && self.get(x, y + 1) != 0)
                || (x > 0 && self.get(x - 1, y) != 0)
                || (x < max_x && self.get(x + 1, y) != 0))
    }

    fn get_neighbors(&self, x: usize, y: usize) -> Vec<Tile> {
        let mut neighbors = Vec::with_capacity(4);
        let max_y = self.0.len() - 1;
        let max_x = self.0[0].len() - 1;

        // above
        if y > 0 && self.get(x, y - 1) != 0 {
            neighbors.push((x, y - 1));
        }
        // below
        if y < max_y && self.get(x, y + 1) != 0 {
            neighbors.push((x, y + 1));
        }
        // left
        if x > 0 && self.get(x - 1, y) != 0 {
            neighbors.push((x - 1, y));
        }
        // right
        if x < max_x && self.get(x + 1, y) != 0 {
            neighbors.push((x + 1, y));
        }
        neighbors
    }

    pub fn all_groups(&self) -> Vec<Vec<Tile>> {
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

    fn has_group(&self, x: usize, y: usize) -> bool {
        let group_type = self.get(x, y);
        if group_type == 0 {
            return false;
        }

        let max_y = self.0.len() - 1;
        let max_x = self.0[0].len() - 1;

        (y > 0 && self.get(x, y - 1) == group_type)
            || (y < max_y && self.get(x, y + 1) == group_type)
            || (x > 0 && self.get(x - 1, y) == group_type)
            || (x < max_x && self.get(x + 1, y) == group_type)
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

    fn get_tile_cluster(&self, x: usize, y: usize) -> Vec<Tile> {
        let mut group = HashSet::with_capacity(10);

        if self.get(x, y) == 0 {
            return Vec::new();
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
            if y > 0 && self.get(x, y - 1) != 0 && !group.contains(&(x, y - 1)) {
                todo.push((x, y - 1));
                group.insert((x, y - 1));
            }
            // below
            if y < max_y && self.get(x, y + 1) != 0 && !group.contains(&(x, y + 1)) {
                todo.push((x, y + 1));
                group.insert((x, y + 1));
            }
            // left
            if x > 0 && self.get(x - 1, y) != 0 && !group.contains(&(x - 1, y)) {
                todo.push((x - 1, y));
                group.insert((x - 1, y));
            }
            // right
            if x < max_x && self.get(x + 1, y) != 0 && !group.contains(&(x + 1, y)) {
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

    fn get_all_tiles(&self) -> Vec<Tile> {
        let max_y = self.0.len();
        let max_x = self.0[0].len();
        let mut result = Vec::with_capacity(264);

        for x in 0..max_x {
            for y in 0..max_y {
                if self.get(x, y) != 0 {
                    result.push((x, y));
                }
            }
        }
        result
    }

    fn get_single_tiles(&self) -> Vec<Tile> {
        let max_y = self.0.len();
        let max_x = self.0[0].len();
        let mut result = Vec::with_capacity(30);

        for x in 0..max_x {
            for y in 0..max_y {
                if self.get(x, y) != 0 {
                    let group = self.get_group(x, y);
                    if group.is_empty() {
                        result.push((x, y));
                    }
                }
            }
        }
        result
    }

    pub fn get_dead_tiles_count(&self) -> u32 {
        let max_y = self.0.len();
        let max_x = self.0[0].len();
        let mut counted = HashSet::with_capacity(100);

        for x in 0..max_x {
            for y in 0..max_y {
                if self.get(x, y) != 0 {
                    if counted.contains(&(x, y)) {
                        continue;
                    }

                    if !self.has_neighbors(x, y) {
                        counted.insert((x, y));
                        continue;
                    }

                    let tile_cluster = self.get_tile_cluster(x, y);
                    let mut dead_cluster = true;
                    for tile in &tile_cluster {
                        if dead_cluster && self.has_group(tile.0, tile.1) {
                            dead_cluster = false;
                            break;
                        }
                    }
                    if dead_cluster {
                        for tile in tile_cluster {
                            counted.insert((tile.0, tile.1));
                        }
                    }
                }
            }
        }
        counted.len().try_into().unwrap()
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
    fn test_has_neighbors(){
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
            let neighbors = map.has_neighbors(14, 10);
            assert_eq!(neighbors, false);
        }
        {
            let neighbors = map.has_neighbors(14, 9);
            assert_eq!(neighbors, false);
        }
        {
            let neighbors = map.has_neighbors(0, 0);
            assert_eq!(neighbors, true);
        }
        {
            let neighbors = map.has_neighbors(1, 9);
            assert_eq!(neighbors, true);
        }
        {
            let neighbors = map.has_neighbors(21, 10);
            assert_eq!(neighbors, true);
        }
        {
            let neighbors = map.has_neighbors(21, 0);
            assert_eq!(neighbors, true);
        }
        {
            let neighbors = map.has_neighbors(11, 7);
            assert_eq!(neighbors, true);
        }
    }

    #[test]
    fn test_has_group(){
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
            let group = map.has_group(0, 0);
            assert_eq!(group, true);
        }
        {
            let group = map.has_group(0, 1);
            assert_eq!(group, false);
        }
        {
            let group = map.has_group(5, 9);
            assert_eq!(group, false);
        }
        {
            let group = map.has_group(6, 10);
            assert_eq!(group, true);
        }
        {
            let group = map.has_group(21, 0);
            assert_eq!(group, true);
        }
        {
            let group = map.has_group(21, 1);
            assert_eq!(group, false);
        }
        {
            let group = map.has_group(21, 2);
            assert_eq!(group, false);
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
                [3,0,0,0,0,0,0,0,0,0,0,2,0,0,0,0,0,0,0,0,0,0], // 8
                [2,1,4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1], // 9
                [0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,1], // 10
            ]
        };

        let dead_tiles = map.get_dead_tiles_count();
        assert_eq!(dead_tiles, 6);
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
                [0,1,4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1], // 9
                [3,1,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,1,0], // 10
            ]
        };

        let dead_tiles = map.get_dead_tiles_count();
        assert_eq!(dead_tiles, 9);
    }

    #[test]
    fn test_get_all_tiles_count(){
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
        let all_tiles = map.get_all_tiles();
        assert_eq!(all_tiles.len(), 17);
    }

    #[test]
    fn test_get_tile_cluster(){
        let map = MonolithMap{
            0: [// 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
                [1,3,0,0,0,0,0,0,0,0,0,0,0,0,0,0,4,1,4,4,4,4], // 0
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
            let mut group = map.get_tile_cluster(0, 0);
            group.sort();
            assert_eq!(group.len(), 3);
            assert_eq!(group, vec![(0, 0), (0, 1), (1, 0),]);
        }
        {
            let mut group = map.get_tile_cluster(6, 9);
            group.sort();
            assert_eq!(group.len(), 11);
        }
        {
            let mut group = map.get_tile_cluster(20, 0);
            group.sort();
            assert_eq!(map.get(20, 0), 4);
            assert_eq!(group.len(), 24);
        }
    }
}
