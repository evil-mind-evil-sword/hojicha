//! Event processing logic extracted from Program for testability

use crate::event::{Event, KeyEvent};
use crossterm::event::{Event as CrosstermEvent, KeyEventKind};
use std::sync::mpsc;
use std::time::Duration;

/// Processes raw crossterm events into boba events
pub struct EventProcessor;

impl EventProcessor {
    /// Process a crossterm event into a boba event
    pub fn process_crossterm_event(event: CrosstermEvent) -> Option<Event<()>> {
        match event {
            CrosstermEvent::Key(key) if key.kind == KeyEventKind::Press => {
                Some(Event::Key(key.into()))
            }
            CrosstermEvent::Mouse(mouse) => Some(Event::Mouse(mouse.into())),
            CrosstermEvent::Resize(width, height) => Some(Event::Resize { width, height }),
            CrosstermEvent::Paste(data) => Some(Event::Paste(data)),
            CrosstermEvent::FocusGained => Some(Event::Focus),
            CrosstermEvent::FocusLost => Some(Event::Blur),
            _ => None,
        }
    }

    /// Coalesce multiple resize events into one
    pub fn coalesce_resize_events(
        initial_width: u16,
        initial_height: u16,
        rx: &mpsc::Receiver<CrosstermEvent>,
    ) -> (u16, u16) {
        let mut width = initial_width;
        let mut height = initial_height;

        // Drain any additional resize events
        while let Ok(CrosstermEvent::Resize(w, h)) = rx.try_recv() {
            width = w;
            height = h;
        }

        (width, height)
    }

    /// Check if an event is a quit event (Ctrl+Q)
    pub fn is_quit_event<M>(event: &Event<M>) -> bool {
        if let Event::Key(KeyEvent { key: crate::event::Key::Char('q'), modifiers }) = event {
            return modifiers.contains(crossterm::event::KeyModifiers::CONTROL);
        }
        false
    }

    /// Check if an event is a suspend event (Ctrl+Z)
    pub fn is_suspend_event<M>(event: &Event<M>) -> bool {
        if let Event::Key(KeyEvent { key: crate::event::Key::Char('z'), modifiers }) = event {
            return modifiers.contains(crossterm::event::KeyModifiers::CONTROL);
        }
        false
    }

    /// Prioritize and merge events from multiple channels
    pub fn prioritize_events<M>(
        message_rx: &mpsc::Receiver<Event<M>>,
        crossterm_rx: &mpsc::Receiver<CrosstermEvent>,
        tick_rate: Duration,
    ) -> Option<Event<M>>
    where
        M: Clone,
    {
        // First check for available user messages
        if let Ok(msg) = message_rx.try_recv() {
            return Some(msg);
        }

        // If no user messages, check for crossterm events
        match crossterm_rx.recv_timeout(tick_rate) {
            Ok(ct_event) => {
                // Special handling for resize to coalesce multiple events
                if let CrosstermEvent::Resize(width, height) = ct_event {
                    let (final_width, final_height) =
                        Self::coalesce_resize_events(width, height, crossterm_rx);
                    return Some(Event::Resize {
                        width: final_width,
                        height: final_height,
                    });
                }

                // Convert crossterm event to boba event
                Self::process_crossterm_event(ct_event)
                    .map(|e| unsafe { std::mem::transmute_copy(&e) })
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                // Check one more time for messages before generating tick
                if let Ok(msg) = message_rx.try_recv() {
                    Some(msg)
                } else {
                    Some(Event::Tick)
                }
            }
            _ => None,
        }
    }

    /// Prioritize events for headless mode (no crossterm events)
    pub fn prioritize_events_headless<M>(
        message_rx: &mpsc::Receiver<Event<M>>,
        tick_rate: Duration,
    ) -> Option<Event<M>>
    where
        M: Clone,
    {
        // Try to receive a message with timeout
        match message_rx.recv_timeout(tick_rate) {
            Ok(msg) => Some(msg),
            Err(mpsc::RecvTimeoutError::Timeout) => Some(Event::Tick),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyModifiers, MouseButton, MouseEventKind};

    #[test]
    fn test_process_key_event() {
        let key_event = CrosstermEvent::Key(crossterm::event::KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        });

        let result = EventProcessor::process_crossterm_event(key_event);
        assert!(result.is_some());
        if let Some(Event::Key(key)) = result {
            assert_eq!(key.key, crate::event::Key::Char('a'));
        } else {
            panic!("Expected Key event");
        }
    }

    #[test]
    fn test_process_mouse_event() {
        let mouse_event = CrosstermEvent::Mouse(crossterm::event::MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 10,
            row: 20,
            modifiers: KeyModifiers::empty(),
        });

        let result = EventProcessor::process_crossterm_event(mouse_event);
        assert!(result.is_some());
        if let Some(Event::Mouse(mouse)) = result {
            assert_eq!(mouse.column, 10);
            assert_eq!(mouse.row, 20);
        } else {
            panic!("Expected Mouse event");
        }
    }

    #[test]
    fn test_process_resize_event() {
        let resize_event = CrosstermEvent::Resize(80, 24);

        let result = EventProcessor::process_crossterm_event(resize_event);
        assert!(result.is_some());
        if let Some(Event::Resize { width, height }) = result {
            assert_eq!(width, 80);
            assert_eq!(height, 24);
        } else {
            panic!("Expected Resize event");
        }
    }

    #[test]
    fn test_process_paste_event() {
        let paste_event = CrosstermEvent::Paste("test".to_string());

        let result = EventProcessor::process_crossterm_event(paste_event);
        assert!(result.is_some());
        if let Some(Event::Paste(text)) = result {
            assert_eq!(text, "test");
        } else {
            panic!("Expected Paste event");
        }
    }

    #[test]
    fn test_process_focus_events() {
        let focus_gained = CrosstermEvent::FocusGained;
        let result = EventProcessor::process_crossterm_event(focus_gained);
        assert!(matches!(result, Some(Event::Focus)));

        let focus_lost = CrosstermEvent::FocusLost;
        let result = EventProcessor::process_crossterm_event(focus_lost);
        assert!(matches!(result, Some(Event::Blur)));
    }

    #[test]
    fn test_coalesce_resize_events() {
        let (tx, rx) = mpsc::channel();

        // Send multiple resize events
        tx.send(CrosstermEvent::Resize(100, 30)).unwrap();
        tx.send(CrosstermEvent::Resize(110, 35)).unwrap();
        tx.send(CrosstermEvent::Resize(120, 40)).unwrap();

        let (width, height) = EventProcessor::coalesce_resize_events(80, 24, &rx);

        // Should get the last resize event
        assert_eq!(width, 120);
        assert_eq!(height, 40);
    }

    #[test]
    fn test_is_quit_event() {
        let quit_event: Event<()> = Event::Key(KeyEvent {
            key: crate::event::Key::Char('q'),
            modifiers: KeyModifiers::CONTROL,
        });
        assert!(EventProcessor::is_quit_event(&quit_event));

        let non_quit_event: Event<()> = Event::Key(KeyEvent {
            key: crate::event::Key::Char('q'),
            modifiers: KeyModifiers::empty(),
        });
        assert!(!EventProcessor::is_quit_event(&non_quit_event));
    }

    #[test]
    fn test_is_suspend_event() {
        let suspend_event: Event<()> = Event::Key(KeyEvent {
            key: crate::event::Key::Char('z'),
            modifiers: KeyModifiers::CONTROL,
        });
        assert!(EventProcessor::is_suspend_event(&suspend_event));

        let non_suspend_event: Event<()> = Event::Key(KeyEvent {
            key: crate::event::Key::Char('z'),
            modifiers: KeyModifiers::empty(),
        });
        assert!(!EventProcessor::is_suspend_event(&non_suspend_event));
    }
}
