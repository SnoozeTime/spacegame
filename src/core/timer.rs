use serde_derive::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Timer {
    /// Deadline in seconds.
    deadline: f32,

    /// how much time in seconds
    #[serde(default)]
    elapsed: f32,

    /// if true, then update will increase elapsed time.
    enabled: bool,
}

impl Timer {
    pub fn of_seconds(seconds: f32) -> Self {
        Self {
            deadline: seconds,
            elapsed: 0.0,
            enabled: false,
        }
    }

    pub fn start(&mut self) {
        self.enabled = true;
    }

    pub fn stop(&mut self) {
        self.enabled = false;
    }

    pub fn reset(&mut self) {
        self.elapsed = 0.0;
    }

    /// Update the timer
    pub fn tick(&mut self, dt: Duration) {
        self.elapsed += dt.as_secs_f32();
    }

    /// Returns true if the deadline has been reached.
    pub fn finished(&self) -> bool {
        self.elapsed >= self.deadline
    }
}
