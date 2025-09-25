#![allow(dead_code)]

use alloc::vec::Vec;
use alloc::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputError {
    BufferFull,
    InvalidDevice,
    UnsupportedEvent,
}

pub type InputResult<T> = Result<T, InputError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputEventType {
    KeyPress,
    KeyRelease,
    MouseMove,
    MouseButtonPress,
    MouseButtonRelease,
    MouseWheel,
    Touch,
}

#[derive(Debug, Clone, Copy)]
pub struct InputEvent {
    pub event_type: InputEventType,
    pub timestamp: u64,
    pub device_id: u32,
    pub code: u32,
    pub value: i32,
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCode {
    Unknown = 0,
    Escape = 1,
    Num1 = 2,
    Num2 = 3,
    Num3 = 4,
    Num4 = 5,
    Num5 = 6,
    Num6 = 7,
    Num7 = 8,
    Num8 = 9,
    Num9 = 10,
    Num0 = 11,
    Q = 16,
    W = 17,
    E = 18,
    R = 19,
    T = 20,
    Y = 21,
    U = 22,
    I = 23,
    O = 24,
    P = 25,
    A = 30,
    S = 31,
    D = 32,
    F = 33,
    G = 34,
    H = 35,
    J = 36,
    K = 37,
    L = 38,
    Z = 44,
    X = 45,
    C = 46,
    V = 47,
    B = 48,
    N = 49,
    M = 50,
    Space = 57,
    Enter = 28,
    Backspace = 14,
    Tab = 15,
    LeftShift = 42,
    RightShift = 54,
    LeftCtrl = 29,
    RightCtrl = 97,
    LeftAlt = 56,
    RightAlt = 100,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left = 0,
    Right = 1,
    Middle = 2,
    Side = 3,
    Extra = 4,
}

#[derive(Debug)]
pub struct KeyboardState {
    pub pressed_keys: [bool; 256],
    pub modifiers: KeyModifiers,
}

#[derive(Debug, Clone, Copy)]
pub struct KeyModifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub super_key: bool,
    pub caps_lock: bool,
    pub num_lock: bool,
    pub scroll_lock: bool,
}

#[derive(Debug)]
pub struct MouseState {
    pub x: i32,
    pub y: i32,
    pub buttons: [bool; 8],
    pub wheel_delta: i32,
}

#[derive(Debug)]
pub struct TouchState {
    pub active_touches: Vec<TouchPoint>,
    pub max_touches: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct TouchPoint {
    pub id: u32,
    pub x: i32,
    pub y: i32,
    pub pressure: f32,
    pub size: f32,
}

#[derive(Debug)]
pub struct InputManager {
    event_queue: VecDeque<InputEvent>,
    keyboard: KeyboardState,
    mouse: MouseState,
    touch: TouchState,
    next_event_id: u64,
    focus_window: Option<u32>,
}

static mut INPUT_MANAGER: Option<InputManager> = None;

impl InputEvent {
    pub fn new(event_type: InputEventType, code: u32, value: i32) -> Self {
        InputEvent {
            event_type,
            timestamp: timer::get_ticks(),
            device_id: 0,
            code,
            value,
            x: 0,
            y: 0,
        }
    }

    pub fn with_position(mut self, x: i32, y: i32) -> Self {
        self.x = x;
        self.y = y;
        self
    }

    pub fn with_device(mut self, device_id: u32) -> Self {
        self.device_id = device_id;
        self
    }
}

impl KeyModifiers {
    pub fn new() -> Self {
        KeyModifiers {
            shift: false,
            ctrl: false,
            alt: false,
            super_key: false,
            caps_lock: false,
            num_lock: false,
            scroll_lock: false,
        }
    }

    pub fn clear(&mut self) {
        *self = Self::new();
    }
}

impl KeyboardState {
    pub fn new() -> Self {
        KeyboardState {
            pressed_keys: [false; 256],
            modifiers: KeyModifiers::new(),
        }
    }

    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.pressed_keys[key as usize]
    }

    pub fn press_key(&mut self, key: KeyCode) {
        self.pressed_keys[key as usize] = true;
        self.update_modifiers(key, true);
    }

    pub fn release_key(&mut self, key: KeyCode) {
        self.pressed_keys[key as usize] = false;
        self.update_modifiers(key, false);
    }

    fn update_modifiers(&mut self, key: KeyCode, pressed: bool) {
        match key {
            KeyCode::LeftShift | KeyCode::RightShift => self.modifiers.shift = pressed,
            KeyCode::LeftCtrl | KeyCode::RightCtrl => self.modifiers.ctrl = pressed,
            KeyCode::LeftAlt | KeyCode::RightAlt => self.modifiers.alt = pressed,
            _ => {}
        }
    }

    pub fn clear(&mut self) {
        self.pressed_keys = [false; 256];
        self.modifiers.clear();
    }
}

impl MouseState {
    pub fn new() -> Self {
        MouseState {
            x: 0,
            y: 0,
            buttons: [false; 8],
            wheel_delta: 0,
        }
    }

    pub fn is_button_pressed(&self, button: MouseButton) -> bool {
        self.buttons[button as usize]
    }

    pub fn press_button(&mut self, button: MouseButton) {
        self.buttons[button as usize] = true;
    }

    pub fn release_button(&mut self, button: MouseButton) {
        self.buttons[button as usize] = false;
    }

    pub fn move_to(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }

    pub fn get_position(&self) -> (i32, i32) {
        (self.x, self.y)
    }

    pub fn scroll(&mut self, delta: i32) {
        self.wheel_delta = delta;
    }
}

impl TouchState {
    pub fn new(max_touches: u32) -> Self {
        TouchState {
            active_touches: Vec::new(),
            max_touches,
        }
    }

    pub fn add_touch(&mut self, touch: TouchPoint) -> InputResult<()> {
        if self.active_touches.len() >= self.max_touches as usize {
            return Err(InputError::BufferFull);
        }
        self.active_touches.push(touch);
        Ok(())
    }

    pub fn remove_touch(&mut self, id: u32) {
        self.active_touches.retain(|touch| touch.id != id);
    }

    pub fn update_touch(&mut self, id: u32, x: i32, y: i32, pressure: f32, size: f32) {
        if let Some(touch) = self.active_touches.iter_mut().find(|t| t.id == id) {
            touch.x = x;
            touch.y = y;
            touch.pressure = pressure;
            touch.size = size;
        }
    }

    pub fn get_touch_count(&self) -> usize {
        self.active_touches.len()
    }
}

impl InputManager {
    pub fn new() -> Self {
        InputManager {
            event_queue: VecDeque::new(),
            keyboard: KeyboardState::new(),
            mouse: MouseState::new(),
            touch: TouchState::new(10), // Support up to 10 touch points
            next_event_id: 0,
            focus_window: None,
        }
    }

    pub fn push_event(&mut self, event: InputEvent) -> InputResult<()> {
        const MAX_EVENTS: usize = 1024;
        
        if self.event_queue.len() >= MAX_EVENTS {
            self.event_queue.pop_front(); // Remove oldest event
        }
        
        self.event_queue.push_back(event);
        self.process_event(&event);
        Ok(())
    }

    pub fn pop_event(&mut self) -> Option<InputEvent> {
        self.event_queue.pop_front()
    }

    pub fn peek_event(&self) -> Option<&InputEvent> {
        self.event_queue.front()
    }

    pub fn has_events(&self) -> bool {
        !self.event_queue.is_empty()
    }

    pub fn clear_events(&mut self) {
        self.event_queue.clear();
    }

    fn process_event(&mut self, event: &InputEvent) {
        match event.event_type {
            InputEventType::KeyPress => {
                if let Some(key) = KeyCode::from_u32(event.code) {
                    self.keyboard.press_key(key);
                }
            }
            InputEventType::KeyRelease => {
                if let Some(key) = KeyCode::from_u32(event.code) {
                    self.keyboard.release_key(key);
                }
            }
            InputEventType::MouseMove => {
                self.mouse.move_to(event.x, event.y);
            }
            InputEventType::MouseButtonPress => {
                if let Some(button) = MouseButton::from_u32(event.code) {
                    self.mouse.press_button(button);
                }
            }
            InputEventType::MouseButtonRelease => {
                if let Some(button) = MouseButton::from_u32(event.code) {
                    self.mouse.release_button(button);
                }
            }
            InputEventType::MouseWheel => {
                self.mouse.scroll(event.value);
            }
            InputEventType::Touch => {
                // Handle touch events
                let touch_point = TouchPoint {
                    id: event.code,
                    x: event.x,
                    y: event.y,
                    pressure: (event.value as f32) / 1000.0, // Convert to 0-1 range
                    size: 10.0, // Default size
                };
                
                if event.value > 0 {
                    let _ = self.touch.add_touch(touch_point);
                } else {
                    self.touch.remove_touch(event.code);
                }
            }
        }
    }

    pub fn set_focus_window(&mut self, window_id: Option<u32>) {
        self.focus_window = window_id;
    }

    pub fn get_focus_window(&self) -> Option<u32> {
        self.focus_window
    }

    pub fn get_keyboard_state(&self) -> &KeyboardState {
        &self.keyboard
    }

    pub fn get_mouse_state(&self) -> &MouseState {
        &self.mouse
    }

    pub fn get_touch_state(&self) -> &TouchState {
        &self.touch
    }

    pub fn inject_key_event(&mut self, key: KeyCode, pressed: bool) -> InputResult<()> {
        let event_type = if pressed { InputEventType::KeyPress } else { InputEventType::KeyRelease };
        let event = InputEvent::new(event_type, key as u32, if pressed { 1 } else { 0 });
        self.push_event(event)
    }

    pub fn inject_mouse_move(&mut self, x: i32, y: i32) -> InputResult<()> {
        let event = InputEvent::new(InputEventType::MouseMove, 0, 0).with_position(x, y);
        self.push_event(event)
    }

    pub fn inject_mouse_button(&mut self, button: MouseButton, pressed: bool) -> InputResult<()> {
        let event_type = if pressed { InputEventType::MouseButtonPress } else { InputEventType::MouseButtonRelease };
        let event = InputEvent::new(event_type, button as u32, if pressed { 1 } else { 0 });
        self.push_event(event)
    }
}

impl KeyCode {
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            1 => Some(KeyCode::Escape),
            2 => Some(KeyCode::Num1),
            3 => Some(KeyCode::Num2),
            16 => Some(KeyCode::Q),
            17 => Some(KeyCode::W),
            18 => Some(KeyCode::E),
            19 => Some(KeyCode::R),
            28 => Some(KeyCode::Enter),
            57 => Some(KeyCode::Space),
            _ => None,
        }
    }
}

impl MouseButton {
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(MouseButton::Left),
            1 => Some(MouseButton::Right),
            2 => Some(MouseButton::Middle),
            _ => None,
        }
    }
}

// Module for timer functions (simplified for this example)
mod timer {
    static mut TICK_COUNT: u64 = 0;
    
    pub fn get_ticks() -> u64 {
        unsafe { TICK_COUNT }
    }
    
    pub fn increment_ticks() {
        unsafe { TICK_COUNT += 1; }
    }
}

// Public API functions
pub fn input_init() -> InputResult<()> {
    unsafe {
        if INPUT_MANAGER.is_some() {
            return Err(InputError::InvalidDevice);
        }
        
        let manager = InputManager::new();
        INPUT_MANAGER = Some(manager);
    }
    Ok(())
}

pub fn input_get_manager() -> Option<&'static mut InputManager> {
    unsafe { INPUT_MANAGER.as_mut() }
}

pub fn input_push_event(event: InputEvent) -> InputResult<()> {
    let manager = input_get_manager().ok_or(InputError::InvalidDevice)?;
    manager.push_event(event)
}

pub fn input_pop_event() -> Option<InputEvent> {
    let manager = input_get_manager()?;
    manager.pop_event()
}

pub fn input_has_events() -> bool {
    let manager = input_get_manager();
    manager.map_or(false, |m| m.has_events())
}

pub fn input_set_focus_window(window_id: Option<u32>) -> InputResult<()> {
    let manager = input_get_manager().ok_or(InputError::InvalidDevice)?;
    manager.set_focus_window(window_id);
    Ok(())
}

pub fn input_get_keyboard_state() -> Option<&'static KeyboardState> {
    let manager = input_get_manager()?;
    Some(manager.get_keyboard_state())
}

pub fn input_get_mouse_state() -> Option<&'static MouseState> {
    let manager = input_get_manager()?;
    Some(manager.get_mouse_state())
}