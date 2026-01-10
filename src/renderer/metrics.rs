//! Performance metrics for the renderer.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Tracks render performance metrics over time.
pub struct RenderMetrics {
    frame_times: VecDeque<Duration>,
    last_frame_start: Instant,
    pub entity_count: usize,
    pub draw_calls: u32,
    pub buffer_uploads: u32,
}

impl Default for RenderMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderMetrics {
    /// Create a new metrics tracker.
    pub fn new() -> Self {
        Self {
            frame_times: VecDeque::with_capacity(120),
            last_frame_start: Instant::now(),
            entity_count: 0,
            draw_calls: 0,
            buffer_uploads: 0,
        }
    }

    /// Call at the start of each frame.
    pub fn begin_frame(&mut self) {
        self.last_frame_start = Instant::now();
        self.draw_calls = 0;
        self.buffer_uploads = 0;
    }

    /// Call at the end of each frame.
    pub fn end_frame(&mut self) {
        let elapsed = self.last_frame_start.elapsed();
        self.frame_times.push_back(elapsed);
        if self.frame_times.len() > 120 {
            self.frame_times.pop_front();
        }
    }

    /// Record a draw call.
    pub fn record_draw_call(&mut self) {
        self.draw_calls += 1;
    }

    /// Record a buffer upload.
    pub fn record_buffer_upload(&mut self) {
        self.buffer_uploads += 1;
    }

    /// Get average frame time in milliseconds (over last 120 frames).
    pub fn avg_frame_time_ms(&self) -> f32 {
        if self.frame_times.is_empty() {
            return 0.0;
        }
        let sum: Duration = self.frame_times.iter().sum();
        sum.as_secs_f32() * 1000.0 / self.frame_times.len() as f32
    }

    /// Get current FPS (based on average frame time).
    pub fn fps(&self) -> f32 {
        let ms = self.avg_frame_time_ms();
        if ms > 0.0 {
            1000.0 / ms
        } else {
            0.0
        }
    }

    /// Get min frame time in milliseconds.
    pub fn min_frame_time_ms(&self) -> f32 {
        self.frame_times
            .iter()
            .min()
            .map(|d| d.as_secs_f32() * 1000.0)
            .unwrap_or(0.0)
    }

    /// Get max frame time in milliseconds.
    pub fn max_frame_time_ms(&self) -> f32 {
        self.frame_times
            .iter()
            .max()
            .map(|d| d.as_secs_f32() * 1000.0)
            .unwrap_or(0.0)
    }

    /// Get last frame time in milliseconds.
    pub fn last_frame_time_ms(&self) -> f32 {
        self.frame_times
            .back()
            .map(|d| d.as_secs_f32() * 1000.0)
            .unwrap_or(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_metrics_recording() {
        let mut metrics = RenderMetrics::new();

        metrics.begin_frame();
        metrics.record_draw_call();
        metrics.record_draw_call();
        metrics.record_buffer_upload();
        thread::sleep(Duration::from_millis(1));
        metrics.end_frame();

        assert_eq!(metrics.draw_calls, 2);
        assert_eq!(metrics.buffer_uploads, 1);
        assert!(metrics.last_frame_time_ms() >= 1.0);
    }

    #[test]
    fn test_fps_calculation() {
        let mut metrics = RenderMetrics::new();

        // Simulate ~60fps (16.67ms per frame)
        for _ in 0..10 {
            metrics.begin_frame();
            thread::sleep(Duration::from_millis(16));
            metrics.end_frame();
        }

        let fps = metrics.fps();
        // Should be roughly 60fps, allow wide tolerance due to sleep precision
        assert!(fps > 30.0 && fps < 100.0, "FPS was {}", fps);
    }
}
