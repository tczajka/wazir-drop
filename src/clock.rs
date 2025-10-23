use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct Stopwatch {
    snapshot: Duration,
    start_instant: Option<Instant>,
}

impl Stopwatch {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            snapshot: Duration::ZERO,
            start_instant: None,
        }
    }

    pub fn start(&mut self) {
        assert!(self.start_instant.is_none(), "Stopwatch already running");
        self.start_instant = Some(Instant::now());
    }

    pub fn stop(&mut self) {
        self.snapshot += self.start_instant.expect("Stopwatch not running").elapsed();
        self.start_instant = None;
    }

    pub fn get(&self) -> Duration {
        match self.start_instant {
            Some(start_instant) => self.snapshot + start_instant.elapsed(),
            None => self.snapshot,
        }
    }
}

#[derive(Debug)]
pub struct Timer {
    stopwatch: Stopwatch,
    initial: Duration,
}

impl Timer {
    pub fn new(initial: Duration) -> Self {
        Self {
            stopwatch: Stopwatch::new(),
            initial,
        }
    }

    pub fn start(&mut self) {
        self.stopwatch.start();
    }

    pub fn stop(&mut self) {
        self.stopwatch.stop();
    }

    pub fn get(&self) -> Duration {
        self.initial.saturating_sub(self.stopwatch.get())
    }
}
