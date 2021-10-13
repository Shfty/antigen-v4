// TODO: Use layouts to compose list, struct, map, etc
//
// TODO: Horizontal list support
// 
// TODO: Tabs widget - like struct, but without the value column
//
// TODO: Major cleanup sweep
//       Duplication
//          Method logic (lists, structs, etc)
//          Type definitions
//       Naming
//          Widget predicate should probably be called widget rules
//       Structure
//          State handling
//          Need to be able to extend for custom widgets
//
mod pong;

pub use pong::*;

use reflection::data::Data;
use reflection_tui::*;

use std::{any::TypeId, borrow::Cow, collections::BTreeMap, io::Stdout, time::{Duration, Instant}};

use tui::backend::CrosstermBackend;

// Test datatypes
#[derive(serde::Serialize)]
enum TestEnum {
    UnitVariant,
    NewtypeVariant(PongState),
    TupleVariant(i8, i16, i32, i64, i128),
    StructVariant {
        uint32: u32,
        cow: Cow<'static, str>,
        test_enum: Box<TestEnum>,
    },
}

#[derive(serde::Serialize)]
struct UnitStruct;

#[derive(serde::Serialize)]
struct NewtypeStruct(&'static str);

#[derive(serde::Serialize)]
struct TupleStruct(i8, i16, i32, i64, i128);

#[derive(serde::Serialize)]
struct TestStructA {
    boolean: bool,
    iint8: i8,
    iint16: i16,
    iint32: i32,
    iint64: i64,
    iint128: i128,
    uint8: u8,
    uint16: u16,
    uint32: u32,
    uint64: u64,
    uint128: u128,
    float32: f32,
    float64: f64,
    char: char,
    str: &'static str,
    #[serde(with = "serde_bytes")]
    byte_array: Vec<u8>,
    option: Option<TestStructB>,
    unit: (),
    unit_struct: UnitStruct,
    unit_variant: TestEnum,
    newtype_struct: NewtypeStruct,
    newtype_variant: TestEnum,
    seq: Vec<u16>,
    tuple: (u8, u16, u32, u64, u128),
    tuple_struct: TupleStruct,
    tuple_variant: TestEnum,
    structure: TestStructB,
    structure_variant: TestEnum,
    map: BTreeMap<&'static str, String>,
}

#[derive(serde::Serialize)]
struct TestStructB {
    uint64: u64,
    float64: f64,
    string: String,
    option_u128: Option<u128>,
    test_enum: TestEnum,
}


// Test logic
fn start_tui() -> tui::Terminal<CrosstermBackend<Stdout>> {
    // Add a panic hook wrapper to disable TUI before print
    // (This prevents loss of panic data printed to the alternate screen)
    let panic_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        stop_tui();
        panic_hook(info);
    }));

    crossterm::terminal::enable_raw_mode().unwrap();

    crossterm::execute!(
        std::io::stdout(),
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )
    .unwrap();

    let backend = CrosstermBackend::new(std::io::stdout());
    let mut terminal = tui::Terminal::new(backend).unwrap();

    terminal.clear().unwrap();

    crossterm::execute!(std::io::stdout(), crossterm::cursor::Hide).unwrap();

    terminal
}

fn stop_tui() {
    // Cleanup TUI
    crossterm::execute!(
        std::io::stdout(),
        crossterm::cursor::Show,
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )
    .unwrap();
    crossterm::terminal::disable_raw_mode().unwrap();
}

fn main_thread(
    r: crossbeam_channel::Receiver<crossterm::event::Event>,
    mut terminal: tui::Terminal<CrosstermBackend<Stdout>>,
) {
    let mut test_struct = TestStructA {
        boolean: true,
        iint8: i8::MIN,
        iint16: i16::MIN,
        iint32: i32::MIN,
        iint64: i64::MIN,
        iint128: i128::MIN,
        uint8: u8::MAX,
        uint16: u16::MAX,
        uint32: u32::MAX,
        uint64: u64::MAX,
        uint128: u128::MAX,
        float32: core::f32::consts::PI,
        float64: core::f64::consts::TAU,
        char: '@',
        str: "Hello World",
        byte_array: vec![1, 2, 3, 4],
        option: Some(TestStructB {
            uint64: 20,
            float64: 2.5,
            string: "Lorem Ipsum".to_string(),
            option_u128: Some(1234),
            test_enum: TestEnum::StructVariant {
                uint32: 30,
                cow: Cow::from("Dolor Sit Amet"),
                test_enum: Box::new(TestEnum::UnitVariant),
            },
        }),
        unit: (),
        unit_struct: UnitStruct,
        unit_variant: TestEnum::UnitVariant,
        newtype_struct: NewtypeStruct("Vulkan Lives!"),
        newtype_variant: TestEnum::NewtypeVariant(Default::default()),
        seq: vec![16, 32, 64, 128],
        tuple: (1, 2, 4, 8, 16),
        tuple_struct: TupleStruct(-32, -64, -128, -256, -512),
        tuple_variant: TestEnum::TupleVariant(-102, -204, -409, -906, -1024),
        map: vec![
            ("Praise the", "Omnissiah".into()),
            ("Utter the", "Canticles of Battle".into()),
            ("Calm the", "Machine Spirit".into()),
        ]
        .into_iter()
        .collect(),
        structure: TestStructB {
            uint64: 20,
            float64: 2.5,
            string: "Lorem Ipsum".to_string(),
            option_u128: None,
            test_enum: TestEnum::StructVariant {
                uint32: 30,
                cow: Cow::from("Dolor Sit Amet"),
                test_enum: Box::new(TestEnum::UnitVariant),
            },
        },
        structure_variant: TestEnum::StructVariant {
            uint32: 30,
            cow: Cow::from("Dolor Sit Amet"),
            test_enum: Box::new(TestEnum::UnitVariant),
        },
    };

    let mut state = ReflectionWidgetState::None;

    'main_loop: loop {
        let timestamp = Instant::now();

        while let Ok(event) = r.try_recv() {
            if let crossterm::event::Event::Key(crossterm::event::KeyEvent {
                code: crossterm::event::KeyCode::Char('c'),
                modifiers: crossterm::event::KeyModifiers::CONTROL,
            }) = event
            {
                break 'main_loop;
            }

            state.handle_input(&event);
        }

        if let TestEnum::NewtypeVariant(v) = &mut test_struct.newtype_variant {
            v.tick();
        }

        let mut data = reflection::to_data(&test_struct, false).unwrap();

        terminal
            .draw(|f| {
                f.render_stateful_widget(
                    ReflectionWidget::new(&mut data, &widget_rules),
                    f.size(),
                    &mut state,
                )
            })
            .unwrap();

        while timestamp.elapsed() <= Duration::from_millis(64) {}
    }
}

pub fn widget_rules(data: &mut Data, parent_type: TypeId) -> Option<Box<dyn DataWidget + '_>> {
    if let Data::Struct { name: "PongState", .. } = data {
        if parent_type == TypeId::of::<StructDetailSlot>() {
            return Some(Box::new(PongWidget::new(data)))
        }
    }

    standard_widgets(&widget_rules)(data, parent_type)
}

fn tui_input_thread(s: crossbeam_channel::Sender<crossterm::event::Event>) -> impl FnOnce() {
    move || 'tui_input_loop: loop {
        let timestamp = Instant::now();
        if let Ok(true) = crossterm::event::poll(Default::default()) {
            while let Ok(event) = crossterm::event::read() {
                if let Err(_) = s.send(event) {
                    break 'tui_input_loop;
                }
            }
        }
        while timestamp.elapsed() <= Duration::from_millis(1) {}
    }
}

fn main() {
    let terminal = start_tui();

    let (s, r) = crossbeam_channel::unbounded();

    std::thread::spawn(tui_input_thread(s));
    main_thread(r, terminal);

    stop_tui();
}
