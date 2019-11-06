use super::monolith_map::{MonolithMap, SolvedPath, Tile};
use crossbeam::queue::ArrayQueue;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::borrow::Borrow;
use std::convert::TryInto;
use std::io::Read;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// Recursive Random Singlethreaded Unbounbed Bruteforce
pub fn solve_1(map: MonolithMap) -> Vec<Tile> {
    fn random_walk(
        results: &mut Vec<SolvedPath>,
        steps: &mut Vec<Tile>,
        map: &mut MonolithMap,
        rng: &mut ThreadRng,
    ) -> u32 {
        let mut groups = map.all_groups();
        if groups.is_empty() {
            map.get_dead_tiles_count()
        } else {
            groups.shuffle(rng);
            let first_tile = groups[0][0];
            map.click(first_tile.0, first_tile.1);
            steps.push(first_tile);
            random_walk(results, steps, map, rng)
        }
    }

    let mut results: Vec<SolvedPath> = Vec::with_capacity(100);
    let mut rng = thread_rng();
    let start = Instant::now();
    loop {
        let mut steps = Vec::with_capacity(100);
        let count = random_walk(&mut results, &mut steps, &mut map.clone(), &mut rng);
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

/// Recursive Sequential Multithreaded Bruteforce
pub fn solve_5(map: MonolithMap) -> Vec<Tile> {
    fn timer_thread(exit_flag: Arc<AtomicBool>, current_best: Arc<AtomicU32>) {
        let start = Instant::now();
        loop {
            if start.elapsed().as_secs() > 60 || current_best.load(Ordering::Relaxed) == 0 {
                println!("Stopping solver.");
                exit_flag.store(true, Ordering::Release);
                break;
            }
            thread::sleep(std::time::Duration::from_millis(100));
        }
    }
    fn brute_solver(
        job_queue: Arc<ArrayQueue<(Vec<Tile>, MonolithMap)>>,
        result: Arc<Mutex<Vec<SolvedPath>>>,
        current_best: Arc<AtomicU32>,
        exit_flag: Arc<AtomicBool>,
    ) {
        fn work(
            result: &Mutex<Vec<SolvedPath>>,
            steps: Vec<Tile>,
            map: MonolithMap,
            current_best: &AtomicU32,
            exit_flag: &AtomicBool,
        ) {
            if exit_flag.load(Ordering::Acquire) {
                return;
            }

            let groups = map.all_groups();
            if groups.is_empty() {
                let count = map.get_dead_tiles_count();

                if count < current_best.load(Ordering::Acquire) {
                    result.lock().unwrap().push((count, steps));
                    current_best.store(count, Ordering::Release);
                    println!("Current best result is: {} tiles remaining.", count);
                }
            } else {
                for group in groups {
                    let first_tile = group[0];

                    let mut new_map = map.clone();
                    new_map.click(first_tile.0, first_tile.1);
                    if new_map.get_dead_tiles_count() >= current_best.load(Ordering::Acquire) {
                        continue;
                    }

                    let new_steps = {
                        let mut temp = steps.clone();
                        temp.push(first_tile);
                        temp
                    };
                    work(result, new_steps, new_map, current_best, exit_flag);
                }
            }
        }

        loop {
            let (steps, map) = match job_queue.pop() {
                Ok(job) => {
                    println!("Took new job, {} remains.", job_queue.len());
                    job
                }
                Err(_) => {
                    thread::sleep(Duration::from_millis(1_000));
                    match job_queue.pop() {
                        Ok(job) => job,
                        Err(_) => return,
                    }
                }
            };
            work(
                result.borrow(),
                steps,
                map,
                current_best.borrow(),
                exit_flag.borrow(),
            );
        }
    };

    let job_queue = Arc::new(ArrayQueue::new(200));
    let result = Arc::new(Mutex::new(Vec::with_capacity(100)));
    let current_best = Arc::new(AtomicU32::new(22 * 11));
    let exit_flag = Arc::new(AtomicBool::new(false));
    {
        let groups = map.all_groups();
        for group in groups {
            let first_tile = group[0];
            let mut new_steps = Vec::with_capacity(100);
            new_steps.push(first_tile);
            let mut new_map = map.clone();
            new_map.click(first_tile.0, first_tile.1);
            job_queue
                .push((new_steps, new_map))
                .expect("Failed to push starting value.");
        }
    }

    let timer_handle = {
        let exit_flag_clone = exit_flag.clone();
        let best_clone = current_best.clone();
        thread::spawn(|| timer_thread(exit_flag_clone, best_clone))
    };
    let workers: Vec<_> = (0..8)
        .map(|_| {
            let job_clone = job_queue.clone();
            let result_clone = result.clone();
            let best_clone = current_best.clone();
            let exit_flag_clone = exit_flag.clone();
            thread::spawn(|| brute_solver(job_clone, result_clone, best_clone, exit_flag_clone))
        })
        .collect();

    timer_handle
        .join()
        .expect("Failed to join on a timer thread handle.");
    for worker in workers {
        worker.join().expect("Failed to join on a thread handle.");
    }

    let mut results = result.lock().unwrap();
    results.sort();
    results.reverse();
    results.pop().unwrap_or_default().1
}

/// Recursive Random Multithreaded Bounbed Bruteforce
pub fn solve_6(map: MonolithMap) -> Vec<Tile> {
    fn timer_thread(exit_flag: Arc<AtomicBool>, current_best: Arc<AtomicU32>) {
        let start = Instant::now();
        loop {
            if start.elapsed().as_secs() > 60 || current_best.load(Ordering::Relaxed) == 0 {
                println!("Stopping solver.");
                exit_flag.store(true, Ordering::Relaxed);
                break;
            }
            thread::sleep(std::time::Duration::from_millis(100));
        }
    }
    fn brute_solver(
        map: MonolithMap,
        result: Arc<Mutex<Vec<SolvedPath>>>,
        current_best: Arc<AtomicU32>,
        exit_flag: Arc<AtomicBool>,
    ) {
        fn work(
            result: &Mutex<Vec<SolvedPath>>,
            steps: Vec<Tile>,
            map: MonolithMap,
            current_best: &AtomicU32,
            exit_flag: &AtomicBool,
            rng: &mut ThreadRng,
        ) {
            if exit_flag.load(Ordering::Acquire) {
                return;
            }

            let mut groups = map.all_groups();
            if groups.is_empty() {
                let count = map.get_dead_tiles_count();

                if count < current_best.load(Ordering::Acquire) {
                    result.lock().unwrap().push((count, steps));
                    current_best.store(count, Ordering::Release);
                    println!("Current best result is: {} tiles remaining.", count);
                }
            } else {
                groups.shuffle(rng);
                for group in groups {
                    let first_tile = group[0];

                    let mut new_map = map.clone();
                    new_map.click(first_tile.0, first_tile.1);
                    if new_map.get_dead_tiles_count() >= current_best.load(Ordering::Acquire) {
                        continue;
                    }

                    let new_steps = {
                        let mut temp = steps.clone();
                        temp.push(first_tile);
                        temp
                    };
                    work(result, new_steps, new_map, current_best, exit_flag, rng);
                }
            }
        }

        work(
            result.borrow(),
            Vec::new(),
            map,
            current_best.borrow(),
            exit_flag.borrow(),
            &mut thread_rng(),
        );
    };

    let result = Arc::new(Mutex::new(Vec::with_capacity(100)));
    let current_best = Arc::new(AtomicU32::new(22 * 11));
    let exit_flag = Arc::new(AtomicBool::new(false));

    let timer_handle = {
        let exit_flag_clone = exit_flag.clone();
        let best_clone = current_best.clone();
        thread::spawn(|| timer_thread(exit_flag_clone, best_clone))
    };
    let workers: Vec<_> = (0..8)
        .map(|_| {
            let map = map.clone();
            let result_clone = result.clone();
            let best_clone = current_best.clone();
            let exit_flag_clone = exit_flag.clone();
            thread::spawn(|| brute_solver(map, result_clone, best_clone, exit_flag_clone))
        })
        .collect();

    timer_handle
        .join()
        .expect("Failed to join on a timer thread handle.");
    for worker in workers {
        worker.join().expect("Failed to join on a thread handle.");
    }

    let mut results = result.lock().unwrap();
    results.sort();
    results.reverse();
    results.pop().unwrap_or_default().1
}

/// Recursive Random SingleGroup Multithreaded Bounbed Bruteforce
pub fn solve_7(map: MonolithMap) -> Vec<Tile> {
    fn timer_thread(exit_flag: Arc<AtomicBool>, current_best: Arc<AtomicU32>) {
        let start = Instant::now();
        loop {
            if start.elapsed().as_secs() > 60 || current_best.load(Ordering::Relaxed) == 0 {
                println!("Stopping solver.");
                exit_flag.store(true, Ordering::Release);
                break;
            }
            thread::sleep(std::time::Duration::from_millis(100));
        }
    }
    fn brute_solver(
        map: MonolithMap,
        result: Arc<Mutex<Vec<SolvedPath>>>,
        current_best: Arc<AtomicU32>,
        exit_flag: Arc<AtomicBool>,
    ) {
        fn random_walk(steps: &mut Vec<Tile>, map: &mut MonolithMap, rng: &mut ThreadRng) -> u32 {
            let mut groups = map.all_groups();
            if groups.is_empty() {
                map.get_dead_tiles_count()
            } else {
                groups.shuffle(rng);
                let first_tile = groups[0][0];
                map.click(first_tile.0, first_tile.1);
                steps.push(first_tile);
                random_walk(steps, map, rng)
            }
        }
        let mut steps = Vec::with_capacity(100);
        let mut rng = thread_rng();
        loop {
            if exit_flag.load(Ordering::Acquire) {
                return;
            }
            let count = random_walk(&mut steps, &mut map.clone(), &mut rng);

            if count < current_best.load(Ordering::Acquire) {
                result.lock().unwrap().push((count, steps.clone()));
                current_best.store(count, Ordering::Release);
                println!("Current best result is: {} tiles remaining.", count);
            }
            steps.clear();
        }
    };

    let result = Arc::new(Mutex::new(Vec::with_capacity(100)));
    let current_best = Arc::new(AtomicU32::new(22 * 11));
    let exit_flag = Arc::new(AtomicBool::new(false));

    let timer_handle = {
        let exit_flag_clone = exit_flag.clone();
        let best_clone = current_best.clone();
        thread::spawn(|| timer_thread(exit_flag_clone, best_clone))
    };
    let workers: Vec<_> = (0..8)
        .map(|_| {
            let map = map.clone();
            let result_clone = result.clone();
            let best_clone = current_best.clone();
            let exit_flag_clone = exit_flag.clone();
            thread::spawn(|| brute_solver(map, result_clone, best_clone, exit_flag_clone))
        })
        .collect();

    timer_handle
        .join()
        .expect("Failed to join on a timer thread handle.");
    for worker in workers {
        worker.join().expect("Failed to join on a thread handle.");
    }

    let mut results = result.lock().unwrap();
    results.sort();
    results.reverse();
    results.pop().unwrap_or_default().1
}

pub fn solve_8(map: MonolithMap) -> Vec<Tile> {
    fn cluster_solver(map_queue: Arc<ArrayQueue<MonolithMap>>, result: Arc<Mutex<Vec<Tile>>>) {
        fn work(
            results: &mut Vec<SolvedPath>,
            steps: Vec<Tile>,
            map: MonolithMap,
            current_best: &mut u32,
        ) {
            let groups = map.all_groups();
            if groups.is_empty() {
                let count = map.get_dead_tiles_count();

                if count < *current_best {
                    results.push((count, steps));
                    *current_best = count;
                }
            } else {
                for group in groups {
                    let first_tile = group[0];

                    let mut new_map = map.clone();
                    new_map.click(first_tile.0, first_tile.1);
                    if new_map.get_dead_tiles_count() >= *current_best {
                        continue;
                    }

                    let new_steps = {
                        let mut temp = steps.clone();
                        temp.push(first_tile);
                        temp
                    };
                    work(results, new_steps, new_map, current_best);
                }
            }
        };

        loop {
            let map = match map_queue.pop() {
                Ok(x) => x,
                Err(_) => {
                    thread::sleep(Duration::from_millis(1_000));
                    match map_queue.pop() {
                        Ok(x) => x,
                        Err(_) => break,
                    }
                }
            };
            let tile_count = map.get_all_tiles_count();
            println!(
                "Took cluster ({} tiles, {} groups), {} left.",
                tile_count,
                map.all_groups().len(),
                map_queue.len()
            );

            let mut results = Vec::with_capacity(50);
            let mut best_result = tile_count;
            work(&mut results, Vec::with_capacity(50), map, &mut best_result);
            if !results.is_empty() {
                results.sort();
                results.reverse();
                let (count, path) = results.pop().unwrap_or_default();
                assert_eq!(count, best_result);
                println!(
                    "Best result for cluster ({} tiles) is {} tiles remaining.",
                    tile_count, count
                );

                {
                    let mut result_vec = result.lock().unwrap();
                    for step in path {
                        result_vec.push(step);
                    }
                }
            }
        }
    };

    let map_queue = Arc::new(ArrayQueue::new(50));
    let clusters = map.all_tile_clusters();
    for (index, cluster) in clusters.into_iter().enumerate() {
        let cluster_map = map.create_map_from_cluster(&cluster);
        let all_groups = cluster_map.all_groups();

        if all_groups.is_empty() {
            continue;
        }

        if all_groups.len() >= 12 {
            println!(
                "Cluster #{} ({} tiles) has too many groups in it ({}).",
                index,
                cluster.len(),
                all_groups.len()
            );
            return Vec::new();
        }

        map_queue
            .push(cluster_map)
            .expect("Failed to push a starting cluster map.");
    }
    let result = Arc::new(Mutex::new(Vec::with_capacity(100)));

    let workers: Vec<_> = (0..8)
        .map(|_| {
            let q1 = map_queue.clone();
            let q2 = result.clone();
            thread::spawn(|| cluster_solver(q1, q2))
        })
        .collect();

    for worker in workers {
        worker.join().expect("Failed to join on a thread handle.");
    }

    Arc::try_unwrap(result)
        .expect("Arc had several owners.")
        .into_inner()
        .expect("Mutex was poisoned.")
}

/// Find Solutions Where That Spot Is Empty
pub fn solve_9(map: MonolithMap) -> Vec<Tile> {
    fn load_target_tiles() -> Vec<Tile> {
        let mut file = std::fs::File::open("tiles.txt").expect("Failed to open 'tiles.txt'.");
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)
            .expect("Failed to read from file.");
        let mut tiles =
            serde_json::from_str::<Vec<(usize, usize)>>(&buffer).expect("Failed to parse JSON.");
        tiles.sort();
        tiles.dedup();
        tiles
    }

    fn get_map_diff_score(current_map: &MonolithMap, target_tiles: &[Tile]) -> u32 {
        let mut score = 0;
        for tile in target_tiles {
            if current_map.get(tile.0, tile.1) == 0 {
                score += 1;
            }
        }
        score
    }

    fn timer_thread(exit_flag: Arc<AtomicBool>) {
        let start = Instant::now();
        loop {
            if exit_flag.load(Ordering::Acquire) {
                break;
            }
            if start.elapsed().as_secs() > 60 {
                println!("Stopping solver.");
                exit_flag.store(true, Ordering::Release);
                break;
            }
            thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    fn random_walk(
        steps: &mut Vec<Tile>,
        map: &mut MonolithMap,
        target: &[Tile],
        rng: &mut ThreadRng,
    ) -> u32 {
        let mut groups = map.all_groups();
        if groups.is_empty() {
            get_map_diff_score(map, target)
        } else {
            groups.shuffle(rng);
            let first_tile = groups[0][0];
            map.click(first_tile.0, first_tile.1);
            steps.push(first_tile);
            random_walk(steps, map, target, rng)
        }
    }

    fn brute_solver(
        map: MonolithMap,
        target: Vec<Tile>,
        result: Arc<Mutex<Vec<SolvedPath>>>,
        current_best: Arc<AtomicU32>,
        exit_flag: Arc<AtomicBool>,
    ) {
        let mut steps = Vec::with_capacity(100);
        let mut rng = thread_rng();
        loop {
            if exit_flag.load(Ordering::Acquire) {
                return;
            }
            let count = random_walk(&mut steps, &mut map.clone(), &target, &mut rng);

            if count > current_best.load(Ordering::Acquire) {
                let target_len: u32 = target
                    .len()
                    .try_into()
                    .expect("Failed to convert usize to u32");
                result.lock().unwrap().push((count, steps.clone()));
                current_best.store(count, Ordering::Release);
                println!(
                    "Current best result is: {}/{} tiles freed.",
                    count, target_len
                );
                if count == target_len {
                    exit_flag.store(true, Ordering::Release);
                    break;
                }
            }
            steps.clear();
        }
    };

    let exit_flag = Arc::new(AtomicBool::new(false));

    let timer_handle = {
        let exit_flag_clone = exit_flag.clone();
        thread::spawn(|| timer_thread(exit_flag_clone))
    };

    let target_tiles = load_target_tiles();
    let result = Arc::new(Mutex::new(Vec::with_capacity(100)));
    let current_best = Arc::new(AtomicU32::new(0));

    let workers: Vec<_> = (0..8)
        .map(|_| {
            let map = map.clone();
            let target = target_tiles.clone();
            let result_clone = result.clone();
            let best_clone = current_best.clone();
            let exit_flag_clone = exit_flag.clone();
            thread::spawn(|| brute_solver(map, target, result_clone, best_clone, exit_flag_clone))
        })
        .collect();

    timer_handle
        .join()
        .expect("Failed to join on a timer thread handle.");
    for worker in workers {
        worker.join().expect("Failed to join on a thread handle.");
    }

    let mut results = result.lock().unwrap();
    results.sort();
    results.pop().unwrap_or_default().1
}

/// Using special function to find best groups to click
pub fn solve_10(map: MonolithMap) -> Vec<Tile> {
    fn get_group_score(original_map: &MonolithMap, group: &[Tile]) -> f64 {
        let mut new_map = original_map.clone();
        let first_tile = group[0];
        new_map.click(first_tile.0, first_tile.1);
        let new_groups = new_map.all_groups();
        let groups_total_size = new_groups.iter().fold(0, |sum, e| sum + e.len());
        // new_groups.len() as f64 // ?
        // groups_total_size as f64 // ?
        groups_total_size as f64 / new_groups.len() as f64 // ?
    }
    fn cmp_f64(a: f64, b: f64) -> std::cmp::Ordering {
        if a < b {
            return std::cmp::Ordering::Less;
        } else if a > b {
            return std::cmp::Ordering::Greater;
        }
        std::cmp::Ordering::Equal
    }
    fn walk(steps: &mut Vec<Tile>, map: &mut MonolithMap) {
        let groups = map.all_groups();
        if groups.is_empty() {
        } else if groups.len() == 1 {
            let first_tile = groups[0][0];
            map.click(first_tile.0, first_tile.1);
            steps.push(first_tile);
            walk(steps, map)
        } else {
            let best_group = {
                let mut best_result = groups
                    .into_iter()
                    .map(|x| (get_group_score(map, &x), x))
                    .collect::<Vec<_>>();
                best_result.sort_by(|a, b| cmp_f64(a.0, b.0));
                best_result.pop().unwrap().1
            };
            let first_tile = best_group[0];
            map.click(first_tile.0, first_tile.1);
            steps.push(first_tile);
            walk(steps, map)
        }
    }

    let mut steps = Vec::with_capacity(100);
    walk(&mut steps, &mut map.clone());
    steps
}
