use hojicha::{
    core::{Cmd, Model},
    event::Event,
    program::{Program, ProgramOptions},
};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[derive(Clone)]
struct SimpleModel {
    count: Arc<AtomicUsize>,
}

#[derive(Debug, Clone)]
enum Msg {
    Inc,
    Quit,
}

impl Model for SimpleModel {
    type Message = Msg;

    fn init(&mut self) -> Cmd<Self::Message> {
        Cmd::none()
    }

    fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
        match event {
            Event::User(Msg::Inc) => {
                let new_count = self.count.fetch_add(1, Ordering::Relaxed) + 1;
                println!("Count: {}", new_count);
                if new_count >= 10 {
                    return hojicha::commands::quit(); // Quit after 10
                }
                Cmd::none()
            }
            Event::User(Msg::Quit) => hojicha::commands::quit(),
            _ => Cmd::none(),
        }
    }

    fn view(&self, _frame: &mut ratatui::Frame, _area: ratatui::layout::Rect) {}
}

#[test]
fn test_simple_async_bridge() {
    let model = SimpleModel {
        count: Arc::new(AtomicUsize::new(0)),
    };

    let count_clone = Arc::clone(&model.count);

    let options = ProgramOptions::default().headless();
    let mut program = Program::with_options(model, options).unwrap();

    program.init_async_bridge();
    let sender = program.sender().expect("bridge initialized");

    // Send messages from another thread
    thread::spawn(move || {
        for i in 0..20 {
            println!("Sending message {}", i);
            if sender.send(Event::User(Msg::Inc)).is_err() {
                println!("Send failed at {}", i);
                break;
            }
            thread::sleep(Duration::from_millis(1));
        }
        println!("Sender done");
    });

    program.run_with_timeout(Duration::from_secs(1)).unwrap();

    let final_count = count_clone.load(Ordering::Relaxed);
    println!("Final count: {}", final_count);
    assert!(
        final_count >= 10,
        "Should have processed at least 10 messages, got {}",
        final_count
    );
}
