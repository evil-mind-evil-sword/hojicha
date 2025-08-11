//! Resilient input handling with panic recovery
//!
//! This module provides a panic-safe input thread that can recover
//! from panics and continue processing terminal events.

use crossterm::event;
use std::panic::{self, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Duration;
use log::{error, warn, info, debug};

/// Statistics for monitoring input thread health
#[derive(Debug, Clone, Default)]
pub struct InputThreadStats {
    pub total_events: usize,
    pub panic_count: usize,
    pub error_count: usize,
    pub restart_count: usize,
}

/// Spawn a resilient input thread with automatic restart on panic
pub fn spawn_resilient_input_thread(
    running: Arc<AtomicBool>,
    force_quit: Arc<AtomicBool>,
    crossterm_tx: mpsc::SyncSender<event::Event>,
) -> thread::JoinHandle<InputThreadStats> {
    thread::spawn(move || {
        let mut stats = InputThreadStats::default();
        
        // Supervisor loop - restarts the input reader if it panics
        while running.load(Ordering::SeqCst) && !force_quit.load(Ordering::SeqCst) {
            info!("Starting input reader (attempt #{})", stats.restart_count + 1);
            
            // Run the actual input reading in a panic-safe wrapper
            let result = panic::catch_unwind(AssertUnwindSafe(|| {
                run_input_loop(
                    &running,
                    &force_quit,
                    &crossterm_tx,
                    &mut stats,
                )
            }));
            
            match result {
                Ok(()) => {
                    debug!("Input loop ended normally");
                    break; // Normal exit
                }
                Err(panic_info) => {
                    stats.panic_count += 1;
                    stats.restart_count += 1;
                    
                    // Log the panic
                    error!("Input thread panicked (panic #{}):", stats.panic_count);
                    if let Some(s) = panic_info.downcast_ref::<&str>() {
                        error!("  Panic message: {}", s);
                    } else if let Some(s) = panic_info.downcast_ref::<String>() {
                        error!("  Panic message: {}", s);
                    } else {
                        error!("  Unknown panic type");
                    }
                    
                    // Check if we should give up
                    if stats.panic_count > 10 {
                        error!("Too many panics in input thread, giving up");
                        break;
                    }
                    
                    // Brief pause before restart to avoid tight panic loops
                    thread::sleep(Duration::from_millis(100));
                    warn!("Restarting input thread after panic...");
                }
            }
        }
        
        info!("Input thread supervisor ending. Stats: {:?}", stats);
        stats
    })
}

/// The actual input reading loop with error handling
fn run_input_loop(
    running: &Arc<AtomicBool>,
    force_quit: &Arc<AtomicBool>,
    crossterm_tx: &mpsc::SyncSender<event::Event>,
    stats: &mut InputThreadStats,
) {
    let mut consecutive_errors = 0;
    
    loop {
        if !running.load(Ordering::SeqCst) || force_quit.load(Ordering::SeqCst) {
            debug!("Input loop stopping (running={}, force_quit={})",
                running.load(Ordering::SeqCst),
                force_quit.load(Ordering::SeqCst)
            );
            break;
        }
        
        // Poll for events with error handling
        match event::poll(Duration::from_millis(100)) {
            Ok(true) => {
                // Event is available, try to read it
                match event::read() {
                    Ok(evt) => {
                        consecutive_errors = 0; // Reset error counter on success
                        stats.total_events += 1;
                        
                        // Try to send the event
                        if let Err(e) = crossterm_tx.send(evt) {
                            debug!("Failed to send event (receiver disconnected): {:?}", e);
                            break; // Channel closed, exit gracefully
                        }
                    }
                    Err(e) => {
                        consecutive_errors += 1;
                        stats.error_count += 1;
                        
                        // Handle specific error types
                        use std::io::ErrorKind;
                        match e.kind() {
                            ErrorKind::Interrupted => {
                                // This is often harmless (e.g., from signals)
                                debug!("Input read interrupted, continuing...");
                                continue;
                            }
                            ErrorKind::WouldBlock => {
                                // This shouldn't happen after poll returned true
                                warn!("Unexpected WouldBlock after successful poll");
                                thread::sleep(Duration::from_millis(10));
                            }
                            _ => {
                                warn!("Error reading input: {}", e);
                                
                                // If we get too many consecutive errors, bail out
                                if consecutive_errors > 10 {
                                    error!("Too many consecutive input errors, stopping input thread");
                                    break;
                                }
                                
                                // Brief pause to avoid tight error loops
                                thread::sleep(Duration::from_millis(50));
                            }
                        }
                    }
                }
            }
            Ok(false) => {
                // No event available, this is normal
                consecutive_errors = 0;
            }
            Err(e) => {
                consecutive_errors += 1;
                stats.error_count += 1;
                
                warn!("Error polling for events: {}", e);
                
                if consecutive_errors > 10 {
                    error!("Too many consecutive polling errors, stopping input thread");
                    break;
                }
                
                // Longer pause for polling errors
                thread::sleep(Duration::from_millis(100));
            }
        }
    }
}

/// Alternative: Create a simple resilient wrapper for existing input code
pub fn wrap_with_panic_recovery<F>(
    name: &str,
    mut f: F,
) -> thread::JoinHandle<()>
where
    F: FnMut() -> bool + Send + 'static,
{
    let thread_name = name.to_string();
    
    thread::spawn(move || {
        let mut attempt = 0;
        
        loop {
            attempt += 1;
            info!("{}: Starting (attempt #{})", thread_name, attempt);
            
            let result = panic::catch_unwind(AssertUnwindSafe(|| f()));
            
            match result {
                Ok(should_continue) => {
                    if !should_continue {
                        info!("{}: Ending normally", thread_name);
                        break;
                    }
                }
                Err(panic_info) => {
                    error!("{}: Panicked!", thread_name);
                    if let Some(s) = panic_info.downcast_ref::<&str>() {
                        error!("  Panic message: {}", s);
                    }
                    
                    if attempt > 10 {
                        error!("{}: Too many panics, giving up", thread_name);
                        break;
                    }
                    
                    thread::sleep(Duration::from_millis(100));
                    warn!("{}: Restarting after panic...", thread_name);
                }
            }
        }
    })
}