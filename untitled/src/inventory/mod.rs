struct Inventory {
    grid: Vec<Vec<Option<usize>>>, // cell -> item ID
    items: Vec<PlacedItem>,
}

impl Inventory {
    fn new(width: usize, height: usize) -> Self {
        Self {
            grid: vec![vec![None; width]; height],
            items: Vec::new(),
        }
    }

    fn can_place(&self, item: &dyn Item, x: usize, y: usize, orientation: Orientation) -> bool {
        let mask = rotate_mask(item.mask(), orientation);
        for (dy, row) in mask.iter().enumerate() {
            for (dx, &cell) in row.iter().enumerate() {
                if cell {
                    let gx = x + dx;
                    let gy = y + dy;
                    if gy >= self.grid.len() || gx >= self.grid[0].len() || self.grid[gy][gx].is_some() {
                        return false;
                    }
                }
            }
        }
        true
    }

    fn place(&mut self, item: Box<dyn Item>, x: usize, y: usize, orientation: Orientation) -> Result<(), &'static str> {
        if !self.can_place(&*item, x, y, orientation) {
            return Err("Cannot place item here");
        }
        let id = self.items.len();
        let placed_item = PlacedItem { item, x, y, orientation };
        let mask = placed_item.rotated_mask();
        for (dy, row) in mask.iter().enumerate() {
            for (dx, &cell) in row.iter().enumerate() {
                if cell {
                    self.grid[y + dy][x + dx] = Some(id);
                }
            }
        }
        self.items.push(placed_item);
        Ok(())
    }

    fn remove(&mut self, id: usize) -> Result<(), &'static str> {
        if id >= self.items.len() {
            return Err("Invalid item ID");
        }
        let placed_item = &self.items[id];
        let mask = placed_item.rotated_mask();
        for (dy, row) in mask.iter().enumerate() {
            for (dx, &cell) in row.iter().enumerate() {
                if cell {
                    self.grid[placed_item.y + dy][placed_item.x + dx] = None;
                }
            }
        }
        self.items.remove(id);
        // Update grid references
        for row in &mut self.grid {
            for cell in row.iter_mut() {
                if let Some(cell_id) = cell {
                    if *cell_id > id {
                        *cell_id -= 1;
                    }
                }
            }
        }
    }
}

trait Item {
    fn mask(&self) -> &[Vec<bool>]; // base shape, Up orientation
}

struct PlacedItem {
    item: Box<dyn Item>,
    x: usize,
    y: usize,
    orientation: Orientation,
}

impl PlacedItem {
    fn rotated_mask(&self) -> Vec<Vec<bool>> {
        rotate_mask(self.item.mask(), self.orientation)
    }
}

fn rotate_mask(mask: &[Vec<bool>], orientation: Orientation) -> Vec<Vec<bool>> {
    match orientation {
        Orientation::Up => mask.to_vec(),
        Orientation::Right => rotate_90_cw(mask),
        Orientation::Down => rotate_180(mask),
        Orientation::Left => rotate_90_ccw(mask),
    }
}

fn rotate_90_cw(mask: &[Vec<bool>]) -> Vec<Vec<bool>> {
    let mut rotated = vec![vec![false; mask.len()]; mask[0].len()];
    for i in 0..mask.len() {
        for j in 0..mask[i].len() {
            rotated[j][mask.len() - 1 - i] = mask[i][j];
        }
    }
    rotated
}

fn rotate_180(mask: &[Vec<bool>]) -> Vec<Vec<bool>> {
    let mut rotated = mask.to_vec();
    rotated.reverse();
    for row in &mut rotated {
        row.reverse();
    }
    rotated
}

fn rotate_90_ccw(mask: &[Vec<bool>]) -> Vec<Vec<bool>> {
    let mut rotated = vec![vec![false; mask.len()]; mask[0].len()];
    for i in 0..mask.len() {
        for j in 0..mask[i].len() {
            rotated[mask[0].len() - 1 - j][i] = mask[i][j];
        }
    }
    rotated
}
