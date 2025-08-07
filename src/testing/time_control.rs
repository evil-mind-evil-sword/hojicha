//! Time control utilities for deterministic testing
//!
//! This module provides utilities for controlling time in tests, similar to
//! Tokio's `time::pause()` functionality.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

/// A controller for virtual time in tests
#[derive(Clone)]
pub struct TimeController {
    /// Whether time is paused
    paused: Arc<AtomicBool>,
    /// Current virtual time in milliseconds
    current_ms: Arc<AtomicU64>,
    /// Time scale factor (1.0 = real time, 0.0 = paused, >1.0 = fast forward)
    time_scale: Arc<AtomicU64>, // Stored as fixed point (1000 = 1.0)
}

impl TimeController {
    /// Create a new time controller with paused time
    pub fn new_paused() -> Self {
        Self {
            paused: Arc::new(AtomicBool::new(true)),
            current_ms: Arc::new(AtomicU64::new(0)),
            time_scale: Arc::new(AtomicU64::new(0)), // 0.0 = paused
        }
    }

    /// Create a new time controller with real time
    pub fn new_real() -> Self {
        Self {
            paused: Arc::new(AtomicBool::new(false)),
            current_ms: Arc::new(AtomicU64::new(0)),
            time_scale: Arc::new(AtomicU64::new(1000)), // 1.0 = real time
        }
    }

    /// Pause time progression
    pub fn pause(&self) {
        self.paused.store(true, Ordering::SeqCst);
        self.time_scale.store(0, Ordering::SeqCst);
    }

    /// Resume time at normal speed
    pub fn resume(&self) {
        self.paused.store(false, Ordering::SeqCst);
        self.time_scale.store(1000, Ordering::SeqCst);
    }

    /// Set time scale (0.0 = paused, 1.0 = normal, 2.0 = double speed)
    pub fn set_scale(&self, scale: f64) {
        let scale_fixed = (scale * 1000.0) as u64;
        self.time_scale.store(scale_fixed, Ordering::SeqCst);
        self.paused.store(scale == 0.0, Ordering::SeqCst);
    }

    /// Advance time by the given duration (only works when paused)
    pub fn advance(&self, duration: Duration) -> Result<(), &'static str> {
        if !self.paused.load(Ordering::SeqCst) {
            return Err("Cannot advance time when not paused");
        }
        
        let ms = duration.as_millis() as u64;
        self.current_ms.fetch_add(ms, Ordering::SeqCst);
        Ok(())
    }

    /// Get the current virtual time
    pub fn now(&self) -> Duration {
        let ms = self.current_ms.load(Ordering::SeqCst);
        Duration::from_millis(ms)
    }

    /// Sleep for the given duration
    /// In paused mode: advances virtual time instantly
    /// In real mode: actually sleeps
    pub async fn sleep(&self, duration: Duration) {
        if self.paused.load(Ordering::SeqCst) {
            // Just advance virtual time
            let _ = self.advance(duration);
        } else {
            // Apply time scale
            let scale = self.time_scale.load(Ordering::SeqCst) as f64 / 1000.0;
            if scale > 0.0 {
                let scaled_duration = Duration::from_secs_f64(duration.as_secs_f64() / scale);
                tokio::time::sleep(scaled_duration).await;
            }
        }
    }

    /// Block the current thread for the given duration
    /// In paused mode: advances virtual time instantly
    /// In real mode: actually blocks
    pub fn sleep_blocking(&self, duration: Duration) {
        if self.paused.load(Ordering::SeqCst) {
            // Just advance virtual time
            let _ = self.advance(duration);
        } else {
            // Apply time scale
            let scale = self.time_scale.load(Ordering::SeqCst) as f64 / 1000.0;
            if scale > 0.0 {
                let scaled_duration = Duration::from_secs_f64(duration.as_secs_f64() / scale);
                std::thread::sleep(scaled_duration);
            }
        }
    }
}

/// Global time controller for tests
static GLOBAL_TIME: once_cell::sync::Lazy<TimeController> = 
    once_cell::sync::Lazy::new(|| TimeController::new_real());

/// Pause global time (for use in tests)
pub fn pause() {
    GLOBAL_TIME.pause();
}

/// Resume global time
pub fn resume() {
    GLOBAL_TIME.resume();
}

/// Advance global time by the given duration
pub fn advance(duration: Duration) -> Result<(), &'static str> {
    GLOBAL_TIME.advance(duration)
}

/// Get a handle to the global time controller
pub fn controller() -> TimeController {
    GLOBAL_TIME.clone()
}

/// Macro for tests with paused time
#[macro_export]
macro_rules! test_with_paused_time {
    ($name:ident, $body:block) => {
        #[test]
        fn $name() {
            let _guard = $crate::testing::time_control::PausedTimeGuard::new();
            $body
        }
    };
}

/// RAII guard that pauses time and restores it when dropped
pub struct PausedTimeGuard {
    was_paused: bool,
}

impl PausedTimeGuard {
    pub fn new() -> Self {
        let was_paused = GLOBAL_TIME.paused.load(Ordering::SeqCst);
        pause();
        Self { was_paused }
    }
}

impl Drop for PausedTimeGuard {
    fn drop(&mut self) {
        if !self.was_paused {
            resume();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_controller_pause_advance() {
        let controller = TimeController::new_paused();
        
        assert_eq!(controller.now(), Duration::ZERO);
        
        controller.advance(Duration::from_secs(1)).unwrap();
        assert_eq!(controller.now(), Duration::from_secs(1));
        
        controller.advance(Duration::from_millis(500)).unwrap();
        assert_eq!(controller.now(), Duration::from_millis(1500));
    }

    #[test]
    fn test_time_controller_cannot_advance_when_running() {
        let controller = TimeController::new_real();
        
        let result = controller.advance(Duration::from_secs(1));
        assert!(result.is_err());
    }

    #[test]
    fn test_time_scale() {
        let controller = TimeController::new_paused();
        
        controller.set_scale(2.0); // Double speed
        assert!(!controller.paused.load(Ordering::SeqCst));
        
        controller.set_scale(0.0); // Paused
        assert!(controller.paused.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_virtual_sleep() {
        let controller = TimeController::new_paused();
        
        let start = controller.now();
        controller.sleep(Duration::from_secs(10)).await;
        let end = controller.now();
        
        // Should have advanced by 10 seconds instantly
        assert_eq!(end - start, Duration::from_secs(10));
    }

    test_with_paused_time!(test_macro_paused_time, {
        advance(Duration::from_secs(1)).unwrap();
        assert_eq!(controller().now(), Duration::from_secs(1));
    });
}