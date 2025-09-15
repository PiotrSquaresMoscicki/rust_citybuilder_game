pub mod input_device;
pub mod input_manager;
pub mod web_client_input_device;

pub use input_device::{
    InputDevice, InputEvent, Key, MouseButton
};
pub use input_manager::{
    initialize_global_input_manager, get_global_input_manager,
    add_global_input_device, poll_global_input_events, is_global_key_pressed
};
pub use web_client_input_device::WebClientInputDevice;