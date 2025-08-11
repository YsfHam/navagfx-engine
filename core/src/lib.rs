use std::time::{Duration, Instant};

pub mod application;
pub mod graphics;
pub mod assets;

pub mod export;


pub struct Timer {
    current_instant: Instant
}

impl Timer {
    pub fn new() -> Self {
        Self {
            current_instant: Instant::now()
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.current_instant.elapsed()
    }

    pub fn restart(&mut self) -> Duration {
        let elapsed = self.elapsed();
        self.current_instant = Instant::now();

        elapsed
    }
}