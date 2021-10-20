use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use crossbeam_channel::Sender;
use crossterm::event::{Event, KeyCode, KeyEvent};

#[profiling::function]
pub fn tui_input_thread(sender: Sender<Event>, main_loop_break: Arc<AtomicBool>) -> impl FnOnce() {
    move || loop {
        // Blocking read
        let event = crossterm::event::read().unwrap();

        // Quit on Ctrl-C
        if let Event::Key(KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: crossterm::event::KeyModifiers::CONTROL,
        }) = event
        {
            main_loop_break.store(true, Ordering::Relaxed);
            break;
        }

        // Send event to render thread
        sender.send(event).ok();
    }
}
