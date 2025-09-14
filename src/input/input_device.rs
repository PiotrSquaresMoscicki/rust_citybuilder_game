use std::error::Error;
use crate::core::math::Vector2d;

/// Types of input events that can be generated
#[derive(Debug, Clone, PartialEq)]
pub enum InputEvent {
    /// Keyboard key press event
    KeyPress { key: Key },
    /// Keyboard key release event  
    KeyRelease { key: Key },
    /// Mouse button press event
    MousePress { button: MouseButton, position: Vector2d },
    /// Mouse button release event
    MouseRelease { button: MouseButton, position: Vector2d },
    /// Mouse movement event
    MouseMove { position: Vector2d, delta: Vector2d },
    /// Mouse wheel scroll event
    MouseWheel { delta: f32, position: Vector2d },
    /// Gamepad button press event
    GamepadPress { button: GamepadButton, player_id: u32 },
    /// Gamepad button release event
    GamepadRelease { button: GamepadButton, player_id: u32 },
    /// Gamepad analog stick movement
    GamepadStick { stick: GamepadStick, value: Vector2d, player_id: u32 },
    /// Touch screen press event
    TouchPress { touch_id: u32, position: Vector2d },
    /// Touch screen release event
    TouchRelease { touch_id: u32, position: Vector2d },
    /// Touch screen movement event
    TouchMove { touch_id: u32, position: Vector2d, delta: Vector2d },
}

/// Keyboard key identifiers
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Key {
    // Letters
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    // Numbers
    Key0, Key1, Key2, Key3, Key4, Key5, Key6, Key7, Key8, Key9,
    // Arrow keys
    ArrowUp, ArrowDown, ArrowLeft, ArrowRight,
    // Function keys
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    // Special keys
    Space, Enter, Escape, Tab, Shift, Control, Alt, Backspace, Delete,
    // Custom key for unknown keys
    Unknown(String),
}

/// Mouse button identifiers
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u8),
}

/// Gamepad button identifiers
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GamepadButton {
    A, B, X, Y,
    DPadUp, DPadDown, DPadLeft, DPadRight,
    LeftShoulder, RightShoulder,
    LeftTrigger, RightTrigger,
    Start, Select,
    LeftStick, RightStick,
    Other(String),
}

/// Gamepad analog stick identifiers
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GamepadStick {
    LeftStick,
    RightStick,
}

/// Result of an input operation
#[derive(Debug, Clone)]
pub enum InputResult {
    Success,
    Error(String),
}

/// Trait defining the interface for input devices
/// Allows multiple implementations for different platforms (web, native, gamepad, etc.)
pub trait InputDevice: Send + Sync {
    /// Initialize the input device
    fn initialize(&mut self) -> Result<(), Box<dyn Error>>;
    
    /// Poll for new input events (non-blocking)
    /// Returns a vector of events that occurred since the last poll
    fn poll_events(&mut self) -> Result<Vec<InputEvent>, Box<dyn Error>>;
    
    /// Check if a specific key is currently pressed
    fn is_key_pressed(&self, key: &Key) -> bool;
    
    /// Check if a specific mouse button is currently pressed  
    fn is_mouse_button_pressed(&self, button: &MouseButton) -> bool;
    
    /// Get the current mouse position
    fn get_mouse_position(&self) -> Vector2d;
    
    /// Check if the device is ready to receive input
    fn is_ready(&self) -> bool;
    
    /// Get the name/type of this input device
    fn device_name(&self) -> &str;
    
    /// Get the unique device ID (for handling multiple devices of the same type)
    fn device_id(&self) -> u32;
    
    /// Shutdown the input device
    fn shutdown(&mut self) -> Result<(), Box<dyn Error>>;
}

/// Helper functions for string parsing
impl Key {
    /// Parse a key from a string representation
    pub fn from_string(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "a" => Key::A, "b" => Key::B, "c" => Key::C, "d" => Key::D, "e" => Key::E,
            "f" => Key::F, "g" => Key::G, "h" => Key::H, "i" => Key::I, "j" => Key::J,
            "k" => Key::K, "l" => Key::L, "m" => Key::M, "n" => Key::N, "o" => Key::O,
            "p" => Key::P, "q" => Key::Q, "r" => Key::R, "s" => Key::S, "t" => Key::T,
            "u" => Key::U, "v" => Key::V, "w" => Key::W, "x" => Key::X, "y" => Key::Y, "z" => Key::Z,
            "0" => Key::Key0, "1" => Key::Key1, "2" => Key::Key2, "3" => Key::Key3, "4" => Key::Key4,
            "5" => Key::Key5, "6" => Key::Key6, "7" => Key::Key7, "8" => Key::Key8, "9" => Key::Key9,
            "arrowup" | "up" => Key::ArrowUp,
            "arrowdown" | "down" => Key::ArrowDown,
            "arrowleft" | "left" => Key::ArrowLeft,
            "arrowright" | "right" => Key::ArrowRight,
            "space" => Key::Space,
            "enter" => Key::Enter,
            "escape" | "esc" => Key::Escape,
            "tab" => Key::Tab,
            "shift" => Key::Shift,
            "control" | "ctrl" => Key::Control,
            "alt" => Key::Alt,
            "backspace" => Key::Backspace,
            "delete" | "del" => Key::Delete,
            "f1" => Key::F1, "f2" => Key::F2, "f3" => Key::F3, "f4" => Key::F4,
            "f5" => Key::F5, "f6" => Key::F6, "f7" => Key::F7, "f8" => Key::F8,
            "f9" => Key::F9, "f10" => Key::F10, "f11" => Key::F11, "f12" => Key::F12,
            _ => Key::Unknown(s.to_string()),
        }
    }
    
    /// Convert a key to its string representation
    pub fn to_string(&self) -> String {
        match self {
            Key::A => "A".to_string(), Key::B => "B".to_string(), Key::C => "C".to_string(),
            Key::D => "D".to_string(), Key::E => "E".to_string(), Key::F => "F".to_string(),
            Key::G => "G".to_string(), Key::H => "H".to_string(), Key::I => "I".to_string(),
            Key::J => "J".to_string(), Key::K => "K".to_string(), Key::L => "L".to_string(),
            Key::M => "M".to_string(), Key::N => "N".to_string(), Key::O => "O".to_string(),
            Key::P => "P".to_string(), Key::Q => "Q".to_string(), Key::R => "R".to_string(),
            Key::S => "S".to_string(), Key::T => "T".to_string(), Key::U => "U".to_string(),
            Key::V => "V".to_string(), Key::W => "W".to_string(), Key::X => "X".to_string(),
            Key::Y => "Y".to_string(), Key::Z => "Z".to_string(),
            Key::Key0 => "0".to_string(), Key::Key1 => "1".to_string(), Key::Key2 => "2".to_string(),
            Key::Key3 => "3".to_string(), Key::Key4 => "4".to_string(), Key::Key5 => "5".to_string(),
            Key::Key6 => "6".to_string(), Key::Key7 => "7".to_string(), Key::Key8 => "8".to_string(),
            Key::Key9 => "9".to_string(),
            Key::ArrowUp => "ArrowUp".to_string(),
            Key::ArrowDown => "ArrowDown".to_string(),
            Key::ArrowLeft => "ArrowLeft".to_string(),
            Key::ArrowRight => "ArrowRight".to_string(),
            Key::Space => "Space".to_string(),
            Key::Enter => "Enter".to_string(),
            Key::Escape => "Escape".to_string(),
            Key::Tab => "Tab".to_string(),
            Key::Shift => "Shift".to_string(),
            Key::Control => "Control".to_string(),
            Key::Alt => "Alt".to_string(),
            Key::Backspace => "Backspace".to_string(),
            Key::Delete => "Delete".to_string(),
            Key::F1 => "F1".to_string(), Key::F2 => "F2".to_string(), Key::F3 => "F3".to_string(),
            Key::F4 => "F4".to_string(), Key::F5 => "F5".to_string(), Key::F6 => "F6".to_string(),
            Key::F7 => "F7".to_string(), Key::F8 => "F8".to_string(), Key::F9 => "F9".to_string(),
            Key::F10 => "F10".to_string(), Key::F11 => "F11".to_string(), Key::F12 => "F12".to_string(),
            Key::Unknown(s) => s.clone(),
        }
    }
}

impl MouseButton {
    /// Parse a mouse button from a string representation
    pub fn from_string(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "left" | "0" => MouseButton::Left,
            "right" | "2" => MouseButton::Right,
            "middle" | "1" => MouseButton::Middle,
            _ => {
                if let Ok(button_code) = s.parse::<u8>() {
                    MouseButton::Other(button_code)
                } else {
                    MouseButton::Other(255) // Unknown button
                }
            }
        }
    }
}