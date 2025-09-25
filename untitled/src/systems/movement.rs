use crate::systems::System;

pub struct MovementSystem {

}

impl MovementSystem {
    pub fn new() -> Self {
        Self {  }
    }
}

impl System for MovementSystem {
    fn tick_start(&mut self, dt: f32) {
        todo!()
    }
}
