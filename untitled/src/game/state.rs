use std::vec::Vec;
use std::time::{Duration, Instant};

use crate::systems::{System};

pub struct GameState {
    tick_budget: Duration,
    systems: Vec<Box<dyn System>>,
}

impl GameState {
    pub fn new(tick_budget: Duration, systems: Vec<Box<dyn System>>) -> Self {
        Self { tick_budget, systems }
    }

    pub fn tick(&mut self, dt: f32) {
        let tick_start = Instant::now();
        let tick_deadline = tick_start + self.tick_budget;

        for s in &mut self.systems {
            s.as_mut().tick_start(dt);
        }

        let mut did_work = true;
        while did_work && Instant::now() < tick_deadline {
            did_work = false;
            for s in &mut self.systems {
                did_work &= s.as_mut().tick_work();
            }
        }

        for s in &mut self.systems {
            s.as_mut().tick_end();
        }
    }

    pub fn render(&self) {

    }

    pub fn entity_count(&self) -> u64 {
        0
    }
}
