// TODO: Improve API
//       Structure
//       Report item querying
//          Logical / Physical / Normalized lookup for axes / hat switches
//          Convenience functions for indexed button checking
//       Formalize device polling
//          Event queue structure via shared state is preferable
//          Change events for individual items
//
// TODO: Fix USBDK lockup
//       Seems to have trouble with DualShock 4
//       May be better to go back to report reading being an optional codepath
//
// TODO: Load fallbacks from disk
//       Will either need to make a wrapper type, or make fallbacks own their report bytes
//
// TODO: Test with mouse
// TODO: Test with keyboard
//
// TODO: Provide means to access collection paths from ItemData
//       Can't use Vec since it's non-copy
//       May be better to use a similar solution to names and use a lookup table

mod error;
mod fallbacks;

#[cfg(test)]
mod tests;

use antigen_hid::{
    devices::{DeviceId, Devices},
    report::{
        global_state::UsagePage,
        local_state::{GenericDesktopUsage, Usage},
        input_report::{InputReport, ReportValue},
    },
};
use error::*;
use fallbacks::*;

use std::{
    collections::BTreeMap,
    fmt::Write,
    sync::{Arc, RwLock},
    time::Duration,
};

const USE_USBDK: bool = false;

fn main() -> Result<(), Error> {
    // Initialize rusb
    if !rusb::has_hid_access() {
        return Err("rusb library has no HID access".into());
    }

    let context = if USE_USBDK {
        rusb::Context::with_options(&[rusb::UsbOption::use_usbdk()])
    } else {
        rusb::Context::new()
    }?;

    // Enumerate valid devices
    let report_descriptors = fallback_t16000m();
    //report_descriptors.dump(&Path::new("."))?;

    let mut devices = Devices::new();
    devices.enumerate(context, &report_descriptors)?;

    // Shared state for transferring input data between threads
    let state: Arc<RwLock<BTreeMap<DeviceId, InputReport>>> = Default::default();

    // Spawn one thread per valid input device
    let (devices, device_names) = devices.take();
    for (device_id, device) in devices {
        let state = state.clone();
        std::thread::spawn(move || {
            let mut buf = vec![0; device.input_buffer_len()];
            let device_handle = device.open().unwrap();

            loop {
                device_handle
                    .read_interrupt(
                        device.interrupt_input(),
                        &mut buf,
                        Duration::from_secs(1000),
                    )
                    .unwrap();

                let result = device.parse(buf.iter().copied());

                state.write().unwrap().insert(device_id, result);
            }
        });
    }

    // Initialize TUI
    let stdout = std::io::stdout();
    let backend = tui::backend::CrosstermBackend::new(stdout);
    let mut terminal = tui::Terminal::new(backend)?;

    // Setup terminal
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::EnterAlternateScreen
    )?;
    crossterm::terminal::enable_raw_mode()?;
    terminal.clear()?;
    terminal.hide_cursor()?;

    // Spawn crossterm input thread
    let (crossterm_tx, crossterm_rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || loop {
        while crossterm::event::poll(Duration::default()).unwrap() {
            crossterm_tx
                .send(crossterm::event::read().unwrap())
                .unwrap();
        }
    });

    // Run main loop
    'main: loop {
        // Handle crossterm events
        while let Ok(event) = crossterm_rx.try_recv() {
            match event {
                crossterm::event::Event::Key(e) => match (e.code, e.modifiers) {
                    (
                        crossterm::event::KeyCode::Char('c'),
                        crossterm::event::KeyModifiers::CONTROL,
                    ) => break 'main,
                    _ => (),
                },
                _ => (),
            }
        }

        // Write TUI output to string buffer
        let mut buf = String::default();

        for (device_id, results) in state.read().unwrap().iter() {
            let device_name = device_names.get(device_id).unwrap();

            writeln!(
                buf,
                "{} {} (VID: 0x{:04x}, PID: 0x{:04x})",
                device_name.manufacturer(),
                device_name.product(),
                device_id.vid(),
                device_id.pid(),
            )
            .unwrap();

            let mut buttons = vec![];
            let mut axes = vec![];
            let mut hat_switches = vec![];
            let mut other = vec![];

            for input_value in results.iter() {
                let data_item = input_value.data_item();
                if let Some(UsagePage::Button) = data_item.global_state.usage_page {
                    buttons.push(input_value);
                    continue;
                } else if let Some(UsagePage::GenericDesktop) = data_item.global_state.usage_page {
                    match data_item.local_state.usage {
                        Some(usage) => match usage {
                            Usage::GenericDesktop(usage) => match usage {
                                GenericDesktopUsage::X
                                | GenericDesktopUsage::Y
                                | GenericDesktopUsage::Z
                                | GenericDesktopUsage::Rx
                                | GenericDesktopUsage::Ry
                                | GenericDesktopUsage::Rz
                                | GenericDesktopUsage::Slider
                                | GenericDesktopUsage::Dial
                                | GenericDesktopUsage::Wheel => {
                                    axes.push(input_value);
                                    continue;
                                }
                                GenericDesktopUsage::HatSwitch => {
                                    hat_switches.push(input_value);
                                    continue;
                                }
                                _ => (),
                            },
                            _ => (),
                        },
                        None => (),
                    }
                }

                other.push(input_value);
            }

            let mut buttons_string = String::from("Buttons: ");
            for (i, input_value) in buttons.iter().enumerate() {
                let report_value = input_value.report_value();
                if let ReportValue::Bool(true) = report_value {
                    write!(buttons_string, "{}, ", i)?;
                }
            }
            writeln!(buf, "{}", buttons_string)?;

            if axes.len() > 0 {
                writeln!(buf, "Axes:")?;
                for input_value in axes {
                    let data_item = input_value.data_item();
                    let value = input_value.report_value();
                    let name = format!("{}", data_item.local_state.usage.unwrap());
                    writeln!(buf, "{}: {:?}", name, value).unwrap();
                }
            }

            if hat_switches.len() > 0 {
                write!(buf, "Hat Switches: ")?;
                for input_value in hat_switches {
                    let value = input_value.report_value();
                    write!(buf, "{}, ", value).unwrap();
                }
            }

            if other.len() > 0 {
                writeln!(buf, "Other:")?;
                for input_value in other {
                    let data_item = input_value.data_item();
                    let value = input_value.report_value();

                    let name = if data_item.local_state.usage.is_some() {
                        format!("{:?}", data_item.local_state.usage.unwrap())
                    } else {
                        format!("{:?}", data_item.global_state.usage_page.unwrap())
                    };
                    writeln!(buf, "{}: {:?}", name, value).unwrap();
                }
            }

            writeln!(buf)?;
        }

        // Render TUI
        terminal.draw(|f| {
            let size = f.size();
            let block = tui::widgets::Block::default()
                .title("HID")
                .borders(tui::widgets::Borders::ALL);

            let inner = block.inner(size.into());
            f.render_widget(block, size);

            let output = tui::widgets::Paragraph::new(buf);
            f.render_widget(output, inner);
        })?;

        // Wait until the next frame
        std::thread::sleep(Duration::from_secs_f64(1.0 / 60.0));
    }

    // Cleanup terminal
    terminal.clear()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen
    )?;
    crossterm::terminal::disable_raw_mode()?;
    terminal.show_cursor()?;

    Ok(())
}
