//! Resource limits and monitoring for async task execution
//!
//! This module provides configurable limits and monitoring for system resources
//! to prevent exhaustion attacks and runaway resource consumption.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use log::{error, warn, info};
use tokio::sync::Semaphore;

/// Configuration for resource limits
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    /// Maximum number of concurrent async tasks (default: 1000)
    pub max_concurrent_tasks: usize,
    /// Maximum recursion depth for commands (default: 100)
    pub max_recursion_depth: usize,
    /// Warning threshold for concurrent tasks (default: 80% of max)
    pub task_warning_threshold: usize,
    /// Whether to log resource warnings (default: true)
    pub log_warnings: bool,
    /// Whether to reject new tasks when at limit (default: true)
    pub reject_when_full: bool,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        let max_tasks = 1000;
        Self {
            max_concurrent_tasks: max_tasks,
            max_recursion_depth: 100,
            task_warning_threshold: (max_tasks as f64 * 0.8) as usize,
            log_warnings: true,
            reject_when_full: true,
        }
    }
}

impl ResourceLimits {
    /// Create limits with a specific max concurrent tasks
    pub fn with_max_tasks(mut self, max: usize) -> Self {
        self.max_concurrent_tasks = max;
        self.task_warning_threshold = (max as f64 * 0.8) as usize;
        self
    }
    
    /// Set the recursion depth limit
    pub fn with_max_recursion(mut self, depth: usize) -> Self {
        self.max_recursion_depth = depth;
        self
    }
    
    /// Disable all limits (use with caution!)
    pub fn unlimited() -> Self {
        Self {
            max_concurrent_tasks: usize::MAX,
            max_recursion_depth: usize::MAX,
            task_warning_threshold: usize::MAX,
            log_warnings: false,
            reject_when_full: false,
        }
    }
}

/// Monitors and enforces resource limits
pub struct ResourceMonitor {
    limits: ResourceLimits,
    active_tasks: Arc<AtomicUsize>,
    total_spawned: Arc<AtomicUsize>,
    total_rejected: Arc<AtomicUsize>,
    peak_concurrent: Arc<AtomicUsize>,
    task_semaphore: Arc<Semaphore>,
    start_time: Instant,
}

impl ResourceMonitor {
    /// Create a new resource monitor with default limits
    pub fn new() -> Self {
        Self::with_limits(ResourceLimits::default())
    }
    
    /// Create a new resource monitor with custom limits
    pub fn with_limits(limits: ResourceLimits) -> Self {
        let semaphore = Arc::new(Semaphore::new(limits.max_concurrent_tasks));
        
        Self {
            limits,
            active_tasks: Arc::new(AtomicUsize::new(0)),
            total_spawned: Arc::new(AtomicUsize::new(0)),
            total_rejected: Arc::new(AtomicUsize::new(0)),
            peak_concurrent: Arc::new(AtomicUsize::new(0)),
            task_semaphore: semaphore,
            start_time: Instant::now(),
        }
    }
    
    /// Try to acquire a permit to spawn a new task
    /// 
    /// Returns Ok(permit) if under limits, Err if at capacity
    pub async fn try_acquire_task_permit(&self) -> Result<TaskPermit, ResourceExhausted> {
        // Try to acquire semaphore permit
        match self.task_semaphore.clone().try_acquire_owned() {
            Ok(permit) => {
                // Update counters
                let active = self.active_tasks.fetch_add(1, Ordering::SeqCst) + 1;
                self.total_spawned.fetch_add(1, Ordering::SeqCst);
                
                // Update peak if needed
                let mut peak = self.peak_concurrent.load(Ordering::SeqCst);
                while active > peak {
                    match self.peak_concurrent.compare_exchange_weak(
                        peak,
                        active,
                        Ordering::SeqCst,
                        Ordering::SeqCst,
                    ) {
                        Ok(_) => break,
                        Err(x) => peak = x,
                    }
                }
                
                // Log warning if approaching limit
                if self.limits.log_warnings && active >= self.limits.task_warning_threshold {
                    warn!(
                        "High async task count: {}/{} ({}% of limit)",
                        active,
                        self.limits.max_concurrent_tasks,
                        (active as f64 / self.limits.max_concurrent_tasks as f64 * 100.0) as u32
                    );
                }
                
                Ok(TaskPermit {
                    _permit: permit,
                    active_tasks: self.active_tasks.clone(),
                })
            }
            Err(_) => {
                self.total_rejected.fetch_add(1, Ordering::SeqCst);
                
                if self.limits.log_warnings {
                    error!(
                        "Task limit exceeded: {} active tasks (limit: {})",
                        self.active_tasks.load(Ordering::SeqCst),
                        self.limits.max_concurrent_tasks
                    );
                }
                
                if self.limits.reject_when_full {
                    Err(ResourceExhausted::TaskLimit(self.limits.max_concurrent_tasks))
                } else {
                    // If not rejecting, wait for a permit
                    let permit = self.task_semaphore.clone().acquire_owned().await
                        .map_err(|_| ResourceExhausted::TaskLimit(self.limits.max_concurrent_tasks))?;
                    
                    let active = self.active_tasks.fetch_add(1, Ordering::SeqCst) + 1;
                    self.total_spawned.fetch_add(1, Ordering::SeqCst);
                    
                    Ok(TaskPermit {
                        _permit: permit,
                        active_tasks: self.active_tasks.clone(),
                    })
                }
            }
        }
    }
    
    /// Get current resource statistics
    pub fn stats(&self) -> ResourceStats {
        ResourceStats {
            active_tasks: self.active_tasks.load(Ordering::SeqCst),
            total_spawned: self.total_spawned.load(Ordering::SeqCst),
            total_rejected: self.total_rejected.load(Ordering::SeqCst),
            peak_concurrent: self.peak_concurrent.load(Ordering::SeqCst),
            max_concurrent_tasks: self.limits.max_concurrent_tasks,
            uptime: self.start_time.elapsed(),
        }
    }
    
    /// Reset statistics (keeps limits)
    pub fn reset_stats(&self) {
        self.total_spawned.store(0, Ordering::SeqCst);
        self.total_rejected.store(0, Ordering::SeqCst);
        self.peak_concurrent.store(self.active_tasks.load(Ordering::SeqCst), Ordering::SeqCst);
    }
    
    /// Check if recursion depth exceeds limit
    pub fn check_recursion_depth(&self, depth: usize) -> Result<(), ResourceExhausted> {
        if depth > self.limits.max_recursion_depth {
            if self.limits.log_warnings {
                error!(
                    "Recursion depth limit exceeded: {} (limit: {})",
                    depth, self.limits.max_recursion_depth
                );
            }
            Err(ResourceExhausted::RecursionDepth(self.limits.max_recursion_depth))
        } else {
            Ok(())
        }
    }
}

/// A permit to spawn an async task
pub struct TaskPermit {
    _permit: tokio::sync::OwnedSemaphorePermit,
    active_tasks: Arc<AtomicUsize>,
}

impl Drop for TaskPermit {
    fn drop(&mut self) {
        self.active_tasks.fetch_sub(1, Ordering::SeqCst);
    }
}

/// Resource exhaustion error
#[derive(Debug, thiserror::Error)]
pub enum ResourceExhausted {
    #[error("Async task limit exceeded (limit: {0})")]
    TaskLimit(usize),
    
    #[error("Recursion depth limit exceeded (limit: {0})")]
    RecursionDepth(usize),
    
    #[error("Memory limit exceeded")]
    MemoryLimit,
}

/// Resource usage statistics
#[derive(Debug, Clone)]
pub struct ResourceStats {
    /// Currently active async tasks
    pub active_tasks: usize,
    /// Total tasks spawned since start
    pub total_spawned: usize,
    /// Total tasks rejected due to limits
    pub total_rejected: usize,
    /// Peak concurrent tasks seen
    pub peak_concurrent: usize,
    /// Configured maximum concurrent tasks
    pub max_concurrent_tasks: usize,
    /// Time since monitor started
    pub uptime: Duration,
}

impl ResourceStats {
    /// Display statistics as a formatted string
    pub fn display(&self) -> String {
        format!(
            "Resource Stats:\n\
             - Active tasks: {}/{} ({}%)\n\
             - Peak concurrent: {}\n\
             - Total spawned: {}\n\
             - Total rejected: {}\n\
             - Uptime: {:?}",
            self.active_tasks,
            self.max_concurrent_tasks,
            (self.active_tasks as f64 / self.max_concurrent_tasks as f64 * 100.0) as u32,
            self.peak_concurrent,
            self.total_spawned,
            self.total_rejected,
            self.uptime
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_task_limits() {
        let monitor = ResourceMonitor::with_limits(
            ResourceLimits::default().with_max_tasks(2)
        );
        
        // Should succeed for first two tasks
        let permit1 = monitor.try_acquire_task_permit().await;
        assert!(permit1.is_ok());
        
        let permit2 = monitor.try_acquire_task_permit().await;
        assert!(permit2.is_ok());
        
        // Third should fail
        let permit3 = monitor.try_acquire_task_permit().await;
        assert!(permit3.is_err());
        
        // Drop one permit
        drop(permit1);
        
        // Now should succeed again
        tokio::time::sleep(Duration::from_millis(10)).await;
        let permit4 = monitor.try_acquire_task_permit().await;
        assert!(permit4.is_ok());
    }
    
    #[test]
    fn test_recursion_limits() {
        let monitor = ResourceMonitor::with_limits(
            ResourceLimits::default().with_max_recursion(5)
        );
        
        assert!(monitor.check_recursion_depth(3).is_ok());
        assert!(monitor.check_recursion_depth(5).is_ok());
        assert!(monitor.check_recursion_depth(6).is_err());
    }
}