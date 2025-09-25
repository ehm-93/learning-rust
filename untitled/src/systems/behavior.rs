use crate::systems::System;

pub struct BehaviorSystem {

}

impl BehaviorSystem {
    pub fn new() -> Self {
        Self {  }
    }
}

impl System for BehaviorSystem {
    fn tick_start(&mut self, dt: f32) {
        todo!()
    }
}
