use crate::systems::System;

pub struct CombatSystem {

}

impl CombatSystem {
    pub fn new() -> Self {
        Self {  }
    }
}

impl System for CombatSystem {
    fn tick_start(&mut self, dt: f32) {
        todo!()
    }
}
