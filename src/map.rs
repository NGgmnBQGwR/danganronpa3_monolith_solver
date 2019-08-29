use std::collections::HashSet;

use serde::{Deserialize, Serialize};

type Tile = (usize, usize);

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct MonolithMap(pub [[u8; 22]; 11]);

impl Default for MonolithMap {
    fn default() -> Self {
        MonolithMap { 0: [[0; 22]; 11] }
    }
}

impl MonolithMap {
    pub fn solve(self) -> Vec<Tile> {
        vec![]
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
        for tile in group {
            for neighbor in self.get_neighbors(tile.0, tile.1) {
                self.advance(neighbor.0, neighbor.1);
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
        if y > 0 {
            neighbors.push((x, y - 1));
        }
        // below
        if y < max_y {
            neighbors.push((x, y + 1));
        }
        // left
        if x > 0 {
            neighbors.push((x - 1, y));
        }
        // right
        if x < max_x {
            neighbors.push((x + 1, y));
        }
        neighbors
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
        group.into_iter().collect()
    }
}

#[cfg(test)]
mod test {
    use super::MonolithMap;

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
        assert_eq!(steps, vec![(8, 6)]);
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
                [0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,1], // 10
            ]
        };

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
            let mut neighbors = map.get_neighbors(0, 21);
            neighbors.sort();
            assert_eq!(neighbors.len(), 2);
            assert_eq!(neighbors, vec![(0, 20), (1, 21)]);
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
    fn test_template(){
        let map = MonolithMap{
            0: [// 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 0
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 1
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 2
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 3
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 4
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 5
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 6
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 7
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 8
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 9
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], // 10
            ]
        };
    }
}
