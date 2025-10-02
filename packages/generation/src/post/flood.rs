use std::collections::VecDeque;
use image::{ImageBuffer, Rgb};

pub fn flood_fill(map: &Vec<Vec<bool>>, start: (usize, usize)) -> Vec<Vec<usize>> {
    let (width, height) = (map[0].len(), map.len());
    let mut distances = vec![vec![usize::MAX; width]; height];
    let mut queue = VecDeque::from([start]);
    distances[start.1][start.0] = 0;

    while let Some((x, y)) = queue.pop_front() {
        let current_distance = distances[y][x];

        for (nx, ny) in neighbors((x, y), width, height) {
            if map[ny][nx] && distances[ny][nx] == usize::MAX {
                distances[ny][nx] = current_distance + 1;
                queue.push_back((nx, ny));
            }
        }
    }

    distances
}

fn neighbors((x, y): (usize, usize), width: usize, height: usize) -> Vec<(usize, usize)> {
    let mut result = Vec::new();
    if x > 0 { result.push((x - 1, y)); }
    if x < width - 1 { result.push((x + 1, y)); }
    if y > 0 { result.push((x, y - 1)); }
    if y < height - 1 { result.push((x, y + 1)); }
    result
}

pub fn write_choropleth(values: &Vec<Vec<usize>>, filename: &str) -> std::io::Result<()> {
    let height = values.len() as u32;
    let width = values[0].len() as u32;
    let max_value = values.iter().flatten().max().unwrap_or(&1);

    let mut img = ImageBuffer::new(width, height);

    for (y, row) in values.iter().enumerate() {
        for (x, &value) in row.iter().enumerate() {
            let intensity = (value as f32 / *max_value as f32 * 255.0) as u8;
            img.put_pixel(x as u32, y as u32, Rgb([intensity, intensity, intensity]));
        }
    }

    img.save(filename)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}
