//! Debug version of program.rs input thread with logging

use crossterm::event;
use std::sync::mpsc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::panic;

pub fn spawn_debug_input_thread(
    running: Arc<AtomicBool>,
    force_quit: Arc<AtomicBool>,
    crossterm_tx: mpsc::SyncSender<event::Event>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        // Set up panic handler for this thread
        let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
            eprintln!("DEBUG: Input thread started");
            let mut iteration = 0;
            
            loop {
                iteration += 1;
                
                if !running.load(Ordering::SeqCst) || force_quit.load(Ordering::SeqCst) {
                    eprintln!("DEBUG: Input thread stopping (running={}, force_quit={})",
                        running.load(Ordering::SeqCst),
                        force_quit.load(Ordering::SeqCst)
                    );
                    break;
                }
                
                // Try to poll for events
                match event::poll(Duration::from_millis(100)) {
                    Ok(true) => {
                        eprintln!("DEBUG: Event available, attempting to read...");
                        
                        // Try to read the event
                        match event::read() {
                            Ok(evt) => {
                                eprintln!("DEBUG: Read event successfully: {:?}", evt);
                                
                                // Try to send it
                                match crossterm_tx.send(evt) {
                                    Ok(()) => {
                                        eprintln!("DEBUG: Event sent successfully");
                                    }
                                    Err(e) => {
                                        eprintln!("ERROR: Failed to send event: {:?}", e);
                                        break; // Channel closed, exit thread
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("ERROR: Failed to read event: {:?}", e);
                                // Don't break here, continue trying
                            }
                        }
                    }
                    Ok(false) => {
                        // No event available, this is normal
                        if iteration % 50 == 0 {
                            eprintln!("DEBUG: Input thread alive, iteration {}", iteration);
                        }
                    }
                    Err(e) => {
                        eprintln!("ERROR: Failed to poll for events: {:?}", e);
                        // Don't break, continue trying
                    }
                }
            }
            
            eprintln!("DEBUG: Input thread loop ended normally");
        }));
        
        match result {
            Ok(()) => {
                eprintln!("DEBUG: Input thread ended without panic");
            }
            Err(e) => {
                eprintln!("ERROR: Input thread PANICKED: {:?}", e);
            }
        }
    })
}