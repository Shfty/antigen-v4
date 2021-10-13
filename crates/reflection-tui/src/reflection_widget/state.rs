use crossterm::event::{Event, KeyCode, KeyEvent};

/// [`reflection::Data`] eqivalent to hold persistent widget state
pub enum ReflectionWidgetState {
    None,
    List {
        selected: usize,
        focused: bool,
        focused_field: Option<usize>,
        fields: Vec<ReflectionWidgetState>,
    },
    Struct {
        selected: usize,
        focused: bool,
        focused_field: Option<usize>,
        fields: Vec<(&'static str, ReflectionWidgetState)>,
    },
    Map {
        column: usize,
        row: usize,
        focused: bool,
        focused_field: Option<(usize, usize)>,
        fields: Vec<(ReflectionWidgetState, ReflectionWidgetState)>,
    },
}

impl ReflectionWidgetState {
    pub fn focus(&mut self) -> bool {
        match self {
            ReflectionWidgetState::None => false,
            ReflectionWidgetState::List {
                focused, fields, ..
            } => {
                if fields.len() > 0 {
                    *focused = true;
                    true
                } else {
                    false
                }
            }
            ReflectionWidgetState::Struct {
                focused, fields, ..
            } => {
                if fields.len() > 0 {
                    *focused = true;
                    true
                } else {
                    false
                }
            }
            ReflectionWidgetState::Map {
                focused, fields, ..
            } => {
                if fields.len() > 0 {
                    *focused = true;
                    true
                } else {
                    false
                }
            }
        }
    }

    pub fn defocus(&mut self) {
        match self {
            ReflectionWidgetState::None => (),
            ReflectionWidgetState::List { focused, .. } => *focused = false,
            ReflectionWidgetState::Struct { focused, .. } => *focused = false,
            ReflectionWidgetState::Map { focused, .. } => *focused = false,
        }
    }

    fn field_count(&mut self) -> usize {
        match self {
            ReflectionWidgetState::None => 0,
            ReflectionWidgetState::List { fields, .. } => fields.len(),
            ReflectionWidgetState::Struct { fields, .. } => fields.len(),
            ReflectionWidgetState::Map { fields, .. } => fields.len(),
        }
        
    }

    fn select_next(&mut self) {
        let field_count = self.field_count();
        match self {
            ReflectionWidgetState::List { selected, .. }
            | ReflectionWidgetState::Struct { selected, .. }
            | ReflectionWidgetState::Map { row: selected, .. } => {
                *selected = selected.wrapping_add(1);
                if *selected >= field_count {
                    *selected = 0
                }
            }
            _ => (),
        }
    }

    fn select_prev(&mut self) {
        let field_count = self.field_count();
        match self {
            ReflectionWidgetState::List { selected, .. }
            | ReflectionWidgetState::Struct { selected, .. }
            | ReflectionWidgetState::Map { row: selected, .. } => {
                *selected = selected.wrapping_sub(1);
                if *selected == usize::MAX {
                    *selected = field_count - 1
                }
            }
            _ => (),
        }
    }

    pub fn handle_input(&mut self, event: &Event) -> bool {
        match self {
            ReflectionWidgetState::None => false,
            ReflectionWidgetState::List {
                selected,
                focused,
                focused_field,
                fields,
                ..
            } => {
                if let Some(i) = focused_field {
                    if *i >= fields.len() {
                        *focused_field = None;
                        return true;
                    }

                    let field = fields.get_mut(*i).unwrap();
                    if !field.handle_input(event) {
                        field.defocus();
                        *focused_field = None;
                    }
                    true
                } else {
                    if let Event::Key(KeyEvent { code, .. }) = event {
                        match code {
                            KeyCode::Char('h') | KeyCode::Esc => false,
                            KeyCode::Char('j') => {
                                if !*focused {
                                    *focused = true;
                                } else {
                                    self.select_next();
                                }
                                true
                            }
                            KeyCode::Char('k') => {
                                if !*focused {
                                    *focused = true;
                                } else {
                                    self.select_prev();
                                }
                                true
                            }
                            KeyCode::Char('l') | KeyCode::Enter => {
                                let field = fields.get_mut(*selected).unwrap();
                                if field.focus() {
                                    *focused_field = Some(*selected);
                                }
                                true
                            }
                            _ => true,
                        }
                    } else {
                        true
                    }
                }
            }
            ReflectionWidgetState::Struct {
                selected,
                focused,
                focused_field,
                fields,
                ..
            } => {
                if let Some(i) = focused_field {
                    if *i >= fields.len() {
                        *focused_field = None;
                        return true;
                    }

                    let (_, field) = fields.get_mut(*i).unwrap();
                    if !field.handle_input(event) {
                        field.defocus();
                        *focused_field = None;
                    }
                    true
                } else {
                    if let Event::Key(KeyEvent { code, .. }) = event {
                        match code {
                            KeyCode::Char('h') | KeyCode::Esc => false,
                            KeyCode::Char('j') => {
                                if !*focused {
                                    *focused = true;
                                } else {
                                    self.select_next();
                                }
                                true
                            }
                            KeyCode::Char('k') => {
                                if !*focused {
                                    *focused = true;
                                } else {
                                    self.select_prev();
                                }
                                true
                            }
                            KeyCode::Char('l') | KeyCode::Enter => {
                                let (_, field) = fields.get_mut(*selected).unwrap();
                                if field.focus() {
                                    *focused_field = Some(*selected);
                                }
                                true
                            }
                            _ => true,
                        }
                    } else {
                        true
                    }
                }
            }
            ReflectionWidgetState::Map {
                column,
                row,
                focused,
                focused_field,
                fields,
                ..
            } => {
                if let Some((column, row)) = focused_field {
                    if *row >= fields.len() {
                        *focused_field = None;
                        return true;
                    }

                    let (key, value) = fields.get_mut(*row).unwrap();

                    let field = if *column == 0 { key } else { value };

                    if !field.handle_input(event) {
                        field.defocus();
                        *focused_field = None;
                    }
                    true
                } else {
                    if let Event::Key(KeyEvent { code, .. }) = event {
                        match code {
                            KeyCode::Esc => false,
                            KeyCode::Char('h') => {
                                if *column == 0 {
                                    false
                                } else {
                                    *column = 0;
                                    true
                                }
                            }
                            KeyCode::Char('j') => {
                                if !*focused {
                                    *focused = true;
                                } else {
                                    self.select_next();
                                }
                                true
                            }
                            KeyCode::Char('k') => {
                                if !*focused {
                                    *focused = true;
                                } else {
                                    self.select_prev();
                                }
                                true
                            }
                            KeyCode::Char('l') => {
                                *column = 1;
                                true
                            }
                            KeyCode::Enter => {
                                let (key, value) = fields.get_mut(*row).unwrap();
                                let field = if *column == 0 { key } else { value };

                                if field.focus() {
                                    *focused_field = Some((*column, *row));
                                }
                                true
                            }
                            _ => true,
                        }
                    } else {
                        true
                    }
                }
            }
        }
    }
}
