pub mod input_device;
pub mod input_manager;
pub mod web_client_input_device;

pub use input_device::{
    InputDevice, InputEvent, InputResult, Key, MouseButton, GamepadButton, GamepadStick
};
pub use input_manager::{
    InputManager, initialize_global_input_manager, get_global_input_manager,
    add_global_input_device, poll_global_input_events, is_global_key_pressed,
    is_global_mouse_button_pressed, get_global_mouse_position, is_global_input_ready
};
pub use web_client_input_device::{WebClientInputDevice, InputMessage};