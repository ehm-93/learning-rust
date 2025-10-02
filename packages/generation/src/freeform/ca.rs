pub fn cellular_automata(map: &mut Vec<Vec<bool>>, iterations: usize, progress: fn(usize, &Vec<Vec<bool>>) -> ()) {
    let width = map[0].len();
    let height = map.len();

    for i in 0..iterations {
        let mut new_map = map.clone();

        for y in 1..height-1 {
            for x in 1..width-1 {
                let floor_count = count_floors(map, x, y);
                if floor_count >= 5 {
                    new_map[y][x] = true;
                }
            }
        }

        *map = new_map;
        progress(i, map);
    }
}

fn count_floors(map: &[Vec<bool>], x: usize, y: usize) -> usize {
    let mut count = 0;
    for dy in -1..=1_i32 {
        for dx in -1..=1_i32 {
            if dx == 0 && dy == 0 { continue; }
            let nx = (x as i32 + dx) as usize;
            let ny = (y as i32 + dy) as usize;
            if map[ny][nx] { count += 1; } // true = floor, false = wall
        }
    }
    count
}
