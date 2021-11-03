use std::time::Duration;

use crossterm::event::{KeyCode, KeyModifiers};

use crate::CrosstermEventQueue;

#[profiling::function]
pub fn crossterm_poll_input(sender: &crossbeam_channel::Sender<crossterm::event::Event>) {
    while let Ok(true) = crossterm::event::poll(Duration::default()) {
        let event = crossterm::event::read().unwrap();
        sender.send(event).ok();
    }
}

#[profiling::function]
pub fn crossterm_input_buffer_fill(
    receiver: &crossbeam_channel::Receiver<crossterm::event::Event>,
    events: &mut CrosstermEventQueue,
) {
    while let Ok(event) = receiver.try_recv() {
        events.push(event);
    }
}

#[profiling::function]
pub fn crossterm_input_buffer_clear(events: &mut CrosstermEventQueue) {
    events.clear();
}

#[profiling::function]
pub fn crossterm_quit_on_ctrl_c(events: &CrosstermEventQueue) -> bool {
    for event in events.iter() {
        if let crossterm::event::Event::Key(crossterm::event::KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
        }) = event
        {
            return true;
        }
    }

    false
}
