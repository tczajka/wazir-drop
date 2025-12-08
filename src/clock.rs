use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct Stopwatch {
    snapshot: Duration,
    start_instant: Option<Instant>,
}

impl Stopwatch {
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

    pub fn instant_at(&self, t: Duration) -> Instant {
        let start_instant = self.start_instant.expect("Stopwatch not running");
        start_instant + t.saturating_sub(self.snapshot)
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

    pub fn get_used(&self) -> Duration {
        self.stopwatch.get()
    }

    pub fn instant_at(&self, t: Duration) -> Instant {
        self.stopwatch.instant_at(self.initial.saturating_sub(t))
    }
}
