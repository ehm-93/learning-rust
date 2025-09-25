use crate::systems::System;

pub struct CollisionSystem {

}

impl CollisionSystem {
    pub fn new() -> Self {
        Self {  }
    }
}

impl System for CollisionSystem {
    fn tick_start(&mut self, dt: f32) {
        todo!()
    }
}
