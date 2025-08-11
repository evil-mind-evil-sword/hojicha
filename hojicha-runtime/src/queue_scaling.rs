//! Auto-scaling functionality for dynamic queue resizing
//!
//! This module provides automatic queue scaling based on load patterns,
//! allowing the queue to grow during high load and shrink during low load
//! to optimize memory usage.

use crate::priority_queue::{PriorityEventQueue, QueueStats};
use hojicha_core::core::Message;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Configuration for automatic queue scaling
#[derive(Debug, Clone)]
pub struct AutoScaleConfig {
    /// Minimum queue size (never shrink below)
    pub min_size: usize,

    /// Maximum queue size (never grow above)
    pub max_size: usize,

    /// Target utilization (0.0 to 1.0)
    pub target_utilization: f64,

    /// How often to evaluate scaling (in number of events)
    pub evaluation_interval: usize,

    /// Scaling strategy
    pub strategy: ScalingStrategy,

    /// Cooldown period between scaling operations
    pub cooldown: Duration,

    /// Enable debug logging
    pub debug: bool,
}

impl Default for AutoScaleConfig {
    fn default() -> Self {
        Self {
            min_size: 100,
            max_size: 10_000,
            target_utilization: 0.5,
            evaluation_interval: 100,
            strategy: ScalingStrategy::Conservative,
            cooldown: Duration::from_secs(5),
            debug: false,
        }
    }
}

/// Scaling strategies with different aggressiveness levels
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScalingStrategy {
    /// Conservative: Small incremental changes
    Conservative,

    /// Aggressive: Large rapid changes
    Aggressive,

    /// Predictive: Based on historical patterns
    Predictive,

    /// Adaptive: Adjusts strategy based on success rate
    Adaptive,
}

/// Decision made by the auto-scaler
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScalingDecision {
    /// Grow the queue by specified amount
    Grow(usize),

    /// Shrink the queue by specified amount
    Shrink(usize),

    /// No change needed
    NoChange,
}

/// Auto-scaler that manages dynamic queue resizing
pub struct QueueAutoScaler {
    config: AutoScaleConfig,

    /// History of utilization measurements
    utilization_history: VecDeque<f64>,

    /// History of scaling decisions and their outcomes
    scaling_history: VecDeque<ScalingOutcome>,

    /// Events processed since last evaluation
    events_since_evaluation: usize,

    /// Last time a scaling operation was performed
    last_scaling_time: Option<Instant>,

    /// Running average of event rate
    event_rate: EventRateTracker,

    /// Peak utilization seen
    peak_utilization: f64,
}

/// Tracks the outcome of a scaling decision
#[derive(Debug, Clone)]
struct ScalingOutcome {
    decision: ScalingDecision,
    #[allow(dead_code)]
    timestamp: Instant,
    utilization_before: f64,
    utilization_after: f64,
    dropped_events_before: usize,
    dropped_events_after: usize,
}

/// Tracks event processing rate over time
struct EventRateTracker {
    buckets: VecDeque<(Instant, usize)>,
    window: Duration,
}

impl EventRateTracker {
    fn new(window: Duration) -> Self {
        Self {
            buckets: VecDeque::new(),
            window,
        }
    }

    fn record_event(&mut self) {
        let now = Instant::now();

        // Remove old buckets outside the window
        while let Some((time, _)) = self.buckets.front() {
            if now.duration_since(*time) > self.window {
                self.buckets.pop_front();
            } else {
                break;
            }
        }

        // Add to current bucket or create new one
        if let Some((time, count)) = self.buckets.back_mut() {
            if now.duration_since(*time) < Duration::from_secs(1) {
                *count += 1;
            } else {
                self.buckets.push_back((now, 1));
            }
        } else {
            self.buckets.push_back((now, 1));
        }
    }

    fn events_per_second(&self) -> f64 {
        if self.buckets.is_empty() {
            return 0.0;
        }

        let total_events: usize = self.buckets.iter().map(|(_, c)| c).sum();
        let duration =
            if let (Some(first), Some(last)) = (self.buckets.front(), self.buckets.back()) {
                last.0.duration_since(first.0).as_secs_f64()
            } else {
                1.0
            };

        if duration > 0.0 {
            total_events as f64 / duration
        } else {
            total_events as f64
        }
    }

    fn is_increasing(&self) -> bool {
        if self.buckets.len() < 3 {
            return false;
        }

        let recent: Vec<_> = self.buckets.iter().rev().take(3).map(|(_, c)| *c).collect();
        recent.windows(2).all(|w| w[0] >= w[1])
    }
}

impl QueueAutoScaler {
    /// Create a new auto-scaler with the given configuration
    pub fn new(config: AutoScaleConfig) -> Self {
        Self {
            config,
            utilization_history: VecDeque::with_capacity(100),
            scaling_history: VecDeque::with_capacity(50),
            events_since_evaluation: 0,
            last_scaling_time: None,
            event_rate: EventRateTracker::new(Duration::from_secs(60)),
            peak_utilization: 0.0,
        }
    }

    /// Process an event and potentially trigger scaling
    pub fn on_event_processed<M: Message>(
        &mut self,
        queue: &mut PriorityEventQueue<M>,
    ) -> Option<ScalingDecision> {
        self.events_since_evaluation += 1;
        self.event_rate.record_event();

        // Check if it's time to evaluate scaling
        if self.events_since_evaluation >= self.config.evaluation_interval {
            self.events_since_evaluation = 0;
            return self.evaluate_scaling(queue);
        }

        None
    }

    /// Evaluate whether scaling is needed
    pub fn evaluate_scaling<M: Message>(
        &mut self,
        queue: &mut PriorityEventQueue<M>,
    ) -> Option<ScalingDecision> {
        let stats = queue.stats();

        // Update history
        self.utilization_history.push_back(stats.utilization);
        if self.utilization_history.len() > 100 {
            self.utilization_history.pop_front();
        }

        self.peak_utilization = self.peak_utilization.max(stats.utilization);

        // Check cooldown
        if let Some(last_time) = self.last_scaling_time {
            if Instant::now().duration_since(last_time) < self.config.cooldown {
                return None;
            }
        }

        // Make scaling decision based on strategy
        let decision = match self.config.strategy {
            ScalingStrategy::Conservative => self.conservative_scaling(&stats),
            ScalingStrategy::Aggressive => self.aggressive_scaling(&stats),
            ScalingStrategy::Predictive => self.predictive_scaling(&stats),
            ScalingStrategy::Adaptive => self.adaptive_scaling(&stats),
        };

        // Apply the decision if there is one
        if decision != ScalingDecision::NoChange {
            let utilization_before = stats.utilization;
            let dropped_before = stats.dropped_events;

            let result = match decision {
                ScalingDecision::Grow(amount) => {
                    let new_size = (stats.max_size + amount).min(self.config.max_size);
                    queue.resize(new_size)
                }
                ScalingDecision::Shrink(amount) => {
                    let new_size =
                        (stats.max_size.saturating_sub(amount)).max(self.config.min_size);
                    queue.resize(new_size)
                }
                ScalingDecision::NoChange => Ok(()),
            };

            if result.is_ok() {
                self.last_scaling_time = Some(Instant::now());

                let new_stats = queue.stats();
                self.scaling_history.push_back(ScalingOutcome {
                    decision,
                    timestamp: Instant::now(),
                    utilization_before,
                    utilization_after: new_stats.utilization,
                    dropped_events_before: dropped_before,
                    dropped_events_after: new_stats.dropped_events,
                });

                if self.scaling_history.len() > 50 {
                    self.scaling_history.pop_front();
                }

                if self.config.debug {
                    log::debug!(
                        "Queue scaling: {:?} (size: {} -> {}, util: {:.1}% -> {:.1}%)",
                        decision,
                        stats.max_size,
                        new_stats.max_size,
                        utilization_before * 100.0,
                        new_stats.utilization * 100.0
                    );
                }

                return Some(decision);
            }
        }

        None
    }

    /// Conservative scaling strategy - small incremental changes
    fn conservative_scaling(&self, stats: &QueueStats) -> ScalingDecision {
        let avg_utilization = self.average_utilization();

        if stats.utilization > 0.9 || stats.backpressure_active {
            // High load - grow by 20%
            let growth = (stats.max_size as f64 * 0.2) as usize;
            ScalingDecision::Grow(growth.max(10))
        } else if avg_utilization < 0.2 && stats.max_size > self.config.min_size {
            // Very low load for sustained period - shrink by 10%
            let shrink = (stats.max_size as f64 * 0.1) as usize;
            ScalingDecision::Shrink(shrink.max(10))
        } else {
            ScalingDecision::NoChange
        }
    }

    /// Aggressive scaling strategy - large rapid changes
    fn aggressive_scaling(&self, stats: &QueueStats) -> ScalingDecision {
        if stats.utilization > 0.8 {
            // High load - double the size
            let growth = stats.max_size;
            ScalingDecision::Grow(growth)
        } else if stats.utilization < 0.1 && stats.max_size > self.config.min_size {
            // Very low load - halve the size
            let shrink = stats.max_size / 2;
            ScalingDecision::Shrink(shrink)
        } else if stats.utilization > 0.6 {
            // Moderate load - grow by 50%
            let growth = (stats.max_size as f64 * 0.5) as usize;
            ScalingDecision::Grow(growth)
        } else {
            ScalingDecision::NoChange
        }
    }

    /// Predictive scaling strategy - based on historical patterns
    fn predictive_scaling(&self, stats: &QueueStats) -> ScalingDecision {
        let event_rate = self.event_rate.events_per_second();
        let is_rate_increasing = self.event_rate.is_increasing();

        // Predict future needs based on event rate trend
        if is_rate_increasing && stats.utilization > 0.5 {
            // Rate is increasing and we're above 50% - proactively grow
            let predicted_need = (event_rate * 10.0) as usize; // Predict 10 seconds ahead
            let growth = predicted_need.saturating_sub(stats.current_size);
            if growth > 0 {
                return ScalingDecision::Grow(growth);
            }
        }

        // Use peak utilization for decisions
        if self.peak_utilization > 0.95 && stats.utilization > 0.7 {
            // We've hit peak before and are climbing - grow early
            let growth = (stats.max_size as f64 * 0.3) as usize;
            ScalingDecision::Grow(growth)
        } else if stats.utilization < 0.15 && !is_rate_increasing {
            // Low utilization and rate not increasing - safe to shrink
            let shrink = (stats.max_size as f64 * 0.2) as usize;
            ScalingDecision::Shrink(shrink)
        } else {
            ScalingDecision::NoChange
        }
    }

    /// Adaptive scaling strategy - adjusts based on past success
    fn adaptive_scaling(&self, stats: &QueueStats) -> ScalingDecision {
        // Analyze recent scaling outcomes
        let recent_successes = self
            .scaling_history
            .iter()
            .rev()
            .take(5)
            .filter(|outcome| {
                // Consider it successful if utilization improved without dropping events
                let util_improved = match outcome.decision {
                    ScalingDecision::Grow(_) => {
                        outcome.utilization_after < outcome.utilization_before
                    }
                    ScalingDecision::Shrink(_) => outcome.utilization_after < 0.8,
                    ScalingDecision::NoChange => true,
                };
                let no_new_drops = outcome.dropped_events_after == outcome.dropped_events_before;
                util_improved && no_new_drops
            })
            .count();

        let success_rate = if self.scaling_history.len() >= 5 {
            recent_successes as f64 / 5.0
        } else {
            0.5 // Assume neutral if not enough history
        };

        // Adjust aggressiveness based on success rate
        if success_rate > 0.8 {
            // High success - be more aggressive
            self.aggressive_scaling(stats)
        } else if success_rate < 0.4 {
            // Low success - be more conservative
            self.conservative_scaling(stats)
        } else {
            // Medium success - use predictive
            self.predictive_scaling(stats)
        }
    }

    /// Get the average utilization over recent history
    fn average_utilization(&self) -> f64 {
        if self.utilization_history.is_empty() {
            return 0.0;
        }

        let sum: f64 = self.utilization_history.iter().sum();
        sum / self.utilization_history.len() as f64
    }

    /// Get current scaling metrics for monitoring
    pub fn metrics(&self) -> ScalingMetrics {
        ScalingMetrics {
            average_utilization: self.average_utilization(),
            peak_utilization: self.peak_utilization,
            events_per_second: self.event_rate.events_per_second(),
            scaling_operations: self.scaling_history.len(),
            last_scaling: self.last_scaling_time,
        }
    }
}

/// Metrics about the auto-scaling behavior
#[derive(Debug, Clone)]
pub struct ScalingMetrics {
    /// Average queue utilization over the monitoring period
    pub average_utilization: f64,
    /// Peak queue utilization observed
    pub peak_utilization: f64,
    /// Current event processing rate in events per second
    pub events_per_second: f64,
    /// Total number of scaling operations performed
    pub scaling_operations: usize,
    /// Timestamp of the last scaling operation
    pub last_scaling: Option<Instant>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_rate_tracker() {
        let mut tracker = EventRateTracker::new(Duration::from_secs(10));

        // Record some events
        for _ in 0..10 {
            tracker.record_event();
        }

        // Should have non-zero rate
        assert!(tracker.events_per_second() > 0.0);
    }

    #[test]
    fn test_scaling_strategies() {
        let config = AutoScaleConfig::default();
        let scaler = QueueAutoScaler::new(config);

        let stats = QueueStats {
            current_size: 90,
            max_size: 100,
            utilization: 0.9,
            backpressure_active: true,
            dropped_events: 0,
        };

        // Conservative should suggest growth
        let decision = scaler.conservative_scaling(&stats);
        assert!(matches!(decision, ScalingDecision::Grow(_)));

        // Aggressive should suggest larger growth
        let aggressive = scaler.aggressive_scaling(&stats);
        if let (ScalingDecision::Grow(c), ScalingDecision::Grow(a)) = (decision, aggressive) {
            assert!(a > c);
        }
    }
}
