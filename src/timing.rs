use std::{
    cmp,
    time::{Duration, Instant},
};

use spin_sleep::SpinSleeper;

pub struct Timing {
    pub tickrate: u64,
    pub framerate: u64,
    last_tick: Instant,
    last_frame: Instant,
    sleeper: SpinSleeper,
}

impl Timing {
    pub fn new(tickrate: u64, framerate: u64) -> Self {
        let now = Instant::now();
        Self {
            tickrate,
            framerate,
            last_tick: now,
            last_frame: now,
            sleeper: SpinSleeper::default(),
        }
    }

    pub fn should_tick(&self) -> bool {
        self.calc_next_tick() == 0
    }
    pub fn should_draw(&self) -> bool {
        self.calc_next_frame() == 0
    }

    pub fn mark_tick(&mut self) {
        self.last_tick = Instant::now();
    }
    pub fn mark_draw(&mut self) {
        self.last_frame = Instant::now();
    }

    pub fn try_sleep(&self) {
        let sleep_for = self.calc_sleep_duration();
        if sleep_for > 0 {
            // accounts for platform dependent sleep resolution
            self.sleeper.sleep(Duration::from_millis(sleep_for));
        }
    }

    fn calc_next_tick(&self) -> u64 {
        calc_next_timeout(&self.last_tick, 1000 / self.tickrate)
    }

    fn calc_next_frame(&self) -> u64 {
        calc_next_timeout(&self.last_frame, 1000 / self.framerate)
    }

    fn calc_sleep_duration(&self) -> u64 {
        cmp::min(self.calc_next_frame(), self.calc_next_tick())
    }
}

#[inline]
fn calc_next_timeout(last: &Instant, timeout: u64) -> u64 {
    // Thats 5849424 centuries of sleeping, give or take
    let elapsed = last.elapsed().as_millis() as u64;
    if timeout > elapsed {
        timeout - elapsed
    } else {
        0
    }
}
