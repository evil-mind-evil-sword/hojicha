//! Safe mutex wrapper that handles poison errors gracefully
//!
//! This module provides a mutex wrapper that recovers from poison errors
//! instead of panicking, allowing the TUI to continue running even if
//! a thread panics while holding a lock.

use log::warn;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

/// A mutex wrapper that automatically recovers from poison errors
pub struct SafeMutex<T> {
    inner: Mutex<T>,
}

impl<T> SafeMutex<T> {
    /// Create a new SafeMutex
    pub fn new(value: T) -> Self {
        Self {
            inner: Mutex::new(value),
        }
    }

    /// Lock the mutex, recovering from poison if necessary
    pub fn lock(&self) -> SafeGuard<T> {
        match self.inner.lock() {
            Ok(guard) => SafeGuard {
                guard: Ok(guard),
                recovered: false,
            },
            Err(poisoned) => {
                warn!("Mutex was poisoned, recovering...");
                let guard = poisoned.into_inner();
                SafeGuard {
                    guard: Ok(guard),
                    recovered: true,
                }
            }
        }
    }

    /// Try to lock the mutex without blocking
    pub fn try_lock(&self) -> Option<SafeGuard<T>> {
        match self.inner.try_lock() {
            Ok(guard) => Some(SafeGuard {
                guard: Ok(guard),
                recovered: false,
            }),
            Err(std::sync::TryLockError::Poisoned(poisoned)) => {
                warn!("Mutex was poisoned during try_lock, recovering...");
                let guard = poisoned.into_inner();
                Some(SafeGuard {
                    guard: Ok(guard),
                    recovered: true,
                })
            }
            Err(std::sync::TryLockError::WouldBlock) => None,
        }
    }
}

/// Guard for SafeMutex that tracks if recovery occurred
pub struct SafeGuard<'a, T> {
    guard: Result<MutexGuard<'a, T>, PoisonError<MutexGuard<'a, T>>>,
    recovered: bool,
}

impl<'a, T> SafeGuard<'a, T> {
    /// Check if this guard recovered from a poison error
    pub fn was_recovered(&self) -> bool {
        self.recovered
    }
}

impl<'a, T> Deref for SafeGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match &self.guard {
            Ok(guard) => guard.deref(),
            Err(poisoned) => poisoned.get_ref(),
        }
    }
}

impl<'a, T> DerefMut for SafeGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match &mut self.guard {
            Ok(guard) => guard.deref_mut(),
            Err(poisoned) => poisoned.get_mut(),
        }
    }
}

/// Arc wrapper around SafeMutex for convenience
pub type SafeArcMutex<T> = Arc<SafeMutex<T>>;

/// Create a new Arc<SafeMutex<T>>
pub fn safe_arc_mutex<T>(value: T) -> SafeArcMutex<T> {
    Arc::new(SafeMutex::new(value))
}
