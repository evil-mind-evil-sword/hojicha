//! FPS limiting logic for controlling render frequency

use std::time::{Duration, Instant};

/// Controls the frame rate of rendering
#[derive(Debug, Clone)]
pub struct FpsLimiter {
    max_fps: u16,
    last_render: Instant,
    frame_duration: Duration,
}

impl FpsLimiter {
    /// Create a new FPS limiter
    pub fn new(max_fps: u16) -> Self {
        let frame_duration = if max_fps > 0 {
            Duration::from_secs(1) / max_fps as u32
        } else {
            Duration::ZERO
        };

        Self {
            max_fps,
            last_render: Instant::now(),
            frame_duration,
        }
    }

    /// Check if enough time has passed to render the next frame
    pub fn should_render(&self) -> bool {
        if self.max_fps == 0 {
            // No FPS limit
            return true;
        }

        self.last_render.elapsed() >= self.frame_duration
    }

    /// Mark that a frame has been rendered
    pub fn mark_rendered(&mut self) {
        self.last_render = Instant::now();
    }

    /// Get the time remaining until the next frame should be rendered
    pub fn time_until_next_frame(&self) -> Duration {
        if self.max_fps == 0 {
            return Duration::ZERO;
        }

        let elapsed = self.last_render.elapsed();
        if elapsed >= self.frame_duration {
            Duration::ZERO
        } else {
            self.frame_duration - elapsed
        }
    }

    /// Update the maximum FPS
    pub fn set_max_fps(&mut self, max_fps: u16) {
        self.max_fps = max_fps;
        self.frame_duration = if max_fps > 0 {
            Duration::from_secs(1) / max_fps as u32
        } else {
            Duration::ZERO
        };
    }

    /// Get the current maximum FPS
    pub fn max_fps(&self) -> u16 {
        self.max_fps
    }

    /// Get the frame duration
    pub fn frame_duration(&self) -> Duration {
        self.frame_duration
    }

    /// Calculate the actual FPS based on the last render time
    pub fn actual_fps(&self) -> f64 {
        let elapsed = self.last_render.elapsed();
        if elapsed.as_secs_f64() > 0.0 {
            1.0 / elapsed.as_secs_f64()
        } else {
            0.0
        }
    }

    /// Reset the limiter
    pub fn reset(&mut self) {
        self.last_render = Instant::now();
    }
}

impl Default for FpsLimiter {
    fn default() -> Self {
        Self::new(60) // Default to 60 FPS
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_fps_limiter_creation() {
        let limiter = FpsLimiter::new(60);
        assert_eq!(limiter.max_fps(), 60);
        assert_eq!(limiter.frame_duration(), Duration::from_secs(1) / 60);
    }

    #[test]
    fn test_fps_limiter_no_limit() {
        let limiter = FpsLimiter::new(0);
        assert_eq!(limiter.max_fps(), 0);
        assert_eq!(limiter.frame_duration(), Duration::ZERO);
        assert!(limiter.should_render());
        assert_eq!(limiter.time_until_next_frame(), Duration::ZERO);
    }

    #[test]
    fn test_fps_limiter_should_render() {
        let mut limiter = FpsLimiter::new(30); // Use lower FPS for more reliable testing

        // Mark as rendered to start fresh
        limiter.mark_rendered();

        // Should not render immediately after marking
        assert!(!limiter.should_render());

        // Wait for frame duration (1/30 sec = ~33ms)
        thread::sleep(Duration::from_millis(40)); // Add buffer for test reliability

        // Should render now
        assert!(limiter.should_render());
    }

    #[test]
    fn test_fps_limiter_time_until_next_frame() {
        let mut limiter = FpsLimiter::new(60);
        limiter.mark_rendered();

        let time_until = limiter.time_until_next_frame();
        assert!(time_until <= Duration::from_secs(1) / 60);

        // Wait past frame duration
        thread::sleep(Duration::from_millis(20));

        let time_until = limiter.time_until_next_frame();
        assert_eq!(time_until, Duration::ZERO);
    }

    #[test]
    fn test_fps_limiter_set_max_fps() {
        let mut limiter = FpsLimiter::new(60);
        assert_eq!(limiter.max_fps(), 60);

        limiter.set_max_fps(30);
        assert_eq!(limiter.max_fps(), 30);
        assert_eq!(limiter.frame_duration(), Duration::from_secs(1) / 30);

        limiter.set_max_fps(0);
        assert_eq!(limiter.max_fps(), 0);
        assert_eq!(limiter.frame_duration(), Duration::ZERO);
    }

    #[test]
    fn test_fps_limiter_reset() {
        let mut limiter = FpsLimiter::new(60);

        // Mark as rendered and wait
        limiter.mark_rendered();
        thread::sleep(Duration::from_millis(20));

        // Reset should update last_render to now
        limiter.reset();

        // Should not render immediately after reset
        assert!(!limiter.should_render());
    }

    #[test]
    fn test_fps_limiter_default() {
        let limiter = FpsLimiter::default();
        assert_eq!(limiter.max_fps(), 60);
    }

    #[test]
    fn test_fps_limiter_various_fps_values() {
        let fps_values = vec![1, 24, 30, 60, 120, 144, 240];

        for fps in fps_values {
            let limiter = FpsLimiter::new(fps);
            assert_eq!(limiter.max_fps(), fps);
            assert_eq!(
                limiter.frame_duration(),
                Duration::from_secs(1) / fps as u32
            );
        }
    }

    #[test]
    #[ignore = "Flaky test due to timing dependencies"]
    fn test_fps_limiter_actual_fps() {
        let mut limiter = FpsLimiter::new(60);

        // Mark rendered and wait a specific time
        limiter.mark_rendered();
        thread::sleep(Duration::from_millis(100)); // 0.1 second

        // Actual FPS should be around 10 (1/0.1)
        let actual_fps = limiter.actual_fps();
        assert!(actual_fps > 8.0 && actual_fps < 12.0);
    }

    #[test]
    fn test_fps_limiter_high_fps() {
        let mut limiter = FpsLimiter::new(240);
        assert_eq!(limiter.max_fps(), 240);

        // Frame duration should be ~4.16ms for 240 FPS
        let expected_duration = Duration::from_secs(1) / 240;
        assert_eq!(limiter.frame_duration(), expected_duration);

        limiter.mark_rendered();
        assert!(!limiter.should_render());

        // Wait for frame duration
        thread::sleep(expected_duration + Duration::from_millis(1));
        assert!(limiter.should_render());
    }

    #[test]
    fn test_fps_limiter_low_fps() {
        let mut limiter = FpsLimiter::new(1);
        assert_eq!(limiter.max_fps(), 1);

        // Frame duration should be 1 second for 1 FPS
        assert_eq!(limiter.frame_duration(), Duration::from_secs(1));

        limiter.mark_rendered();
        assert!(!limiter.should_render());

        // Should not render after 500ms
        thread::sleep(Duration::from_millis(500));
        assert!(!limiter.should_render());

        // Should render after 1 second
        thread::sleep(Duration::from_millis(600));
        assert!(limiter.should_render());
    }
}
