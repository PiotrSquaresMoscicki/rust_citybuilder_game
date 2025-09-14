use std::sync::{Arc, Mutex, OnceLock};
use std::error::Error;
use std::collections::HashMap;
use super::{InputDevice, InputEvent, Key, MouseButton};
use crate::core::math::Vector2d;

/// Global input manager that can be accessed from anywhere in the application
/// This is not an ECS system - it's a globally accessible service
/// Can handle multiple input devices for split-screen games or multiple input sources
pub struct InputManager {
    devices: Vec<Arc<Mutex<Box<dyn InputDevice>>>>,
    device_map: HashMap<u32, usize>, // device_id -> index in devices vector
    is_initialized: bool,
    event_buffer: Vec<InputEvent>,
    key_states: HashMap<Key, bool>,
    mouse_button_states: HashMap<MouseButton, bool>,
    mouse_position: Vector2d,
}

impl InputManager {
    /// Create a new input manager
    pub fn new() -> Self {
        Self {
            devices: Vec::new(),
            device_map: HashMap::new(),
            is_initialized: false,
            event_buffer: Vec::new(),
            key_states: HashMap::new(),
            mouse_button_states: HashMap::new(),
            mouse_position: Vector2d::new(0.0, 0.0),
        }
    }
    
    /// Add an input device to the manager
    pub fn add_device(&mut self, device: Box<dyn InputDevice>) -> Result<u32, Box<dyn Error>> {
        let device_id = device.device_id();
        
        if self.device_map.contains_key(&device_id) {
            return Err(format!("Device with ID {} already exists", device_id).into());
        }
        
        let device_arc = Arc::new(Mutex::new(device));
        let index = self.devices.len();
        
        self.devices.push(device_arc);
        self.device_map.insert(device_id, index);
        
        println!("Added input device with ID: {}", device_id);
        Ok(device_id)
    }
    
    /// Initialize the input manager and all its devices
    pub fn initialize(&mut self) -> Result<(), Box<dyn Error>> {
        if self.is_initialized {
            return Ok(());
        }
        
        for device in &self.devices {
            let mut device = device.lock().map_err(|e| format!("Failed to lock device: {}", e))?;
            device.initialize()?;
        }
        
        self.is_initialized = true;
        println!("Input manager initialized with {} devices", self.devices.len());
        Ok(())
    }
    
    /// Poll for input events from all devices
    pub fn poll_events(&mut self) -> Result<Vec<InputEvent>, Box<dyn Error>> {
        if !self.is_initialized {
            return Ok(Vec::new());
        }
        
        self.event_buffer.clear();
        
        // Collect events from all devices first
        let mut all_events = Vec::new();
        
        for device in &self.devices {
            let mut device = device.lock().map_err(|e| format!("Failed to lock device: {}", e))?;
            
            if device.is_ready() {
                let events = device.poll_events()?;
                all_events.extend(events);
            }
        }
        
        // Update internal state based on events
        for event in &all_events {
            self.update_state_from_event(event);
        }
        
        self.event_buffer = all_events.clone();
        Ok(all_events)
    }
    
    /// Check if a specific key is currently pressed on any device
    pub fn is_key_pressed(&self, key: &Key) -> bool {
        // Check internal state first (from processed events)
        if let Some(&pressed) = self.key_states.get(key) {
            return pressed;
        }
        
        // Fallback: check all devices directly
        for device in &self.devices {
            if let Ok(device) = device.lock() {
                if device.is_ready() && device.is_key_pressed(key) {
                    return true;
                }
            }
        }
        
        false
    }
    
    /// Check if a specific mouse button is currently pressed on any device
    pub fn is_mouse_button_pressed(&self, button: &MouseButton) -> bool {
        // Check internal state first (from processed events)
        if let Some(&pressed) = self.mouse_button_states.get(button) {
            return pressed;
        }
        
        // Fallback: check all devices directly
        for device in &self.devices {
            if let Ok(device) = device.lock() {
                if device.is_ready() && device.is_mouse_button_pressed(button) {
                    return true;
                }
            }
        }
        
        false
    }
    
    /// Get the current mouse position (from the last device that reported it)
    pub fn get_mouse_position(&self) -> Vector2d {
        self.mouse_position
    }
    
    /// Check if the input system is ready (at least one device is ready)
    pub fn is_ready(&self) -> bool {
        if !self.is_initialized {
            return false;
        }
        
        for device in &self.devices {
            if let Ok(device) = device.lock() {
                if device.is_ready() {
                    return true;
                }
            }
        }
        
        false
    }
    
    /// Get the number of connected input devices
    pub fn device_count(&self) -> usize {
        self.devices.len()
    }
    
    /// Get device names and their status
    pub fn get_device_info(&self) -> Vec<(String, bool, u32)> {
        let mut info = Vec::new();
        
        for device in &self.devices {
            if let Ok(device) = device.lock() {
                info.push((
                    device.device_name().to_string(),
                    device.is_ready(),
                    device.device_id(),
                ));
            }
        }
        
        info
    }
    
    /// Remove an input device by ID
    pub fn remove_device(&mut self, device_id: u32) -> Result<(), Box<dyn Error>> {
        if let Some(&index) = self.device_map.get(&device_id) {
            // Shutdown the device before removing
            {
                let mut device = self.devices[index].lock()
                    .map_err(|e| format!("Failed to lock device: {}", e))?;
                device.shutdown()?;
            }
            
            // Remove from devices vector and update indices in device_map
            self.devices.remove(index);
            self.device_map.remove(&device_id);
            
            // Update indices for devices that were shifted
            for (_id, idx) in self.device_map.iter_mut() {
                if *idx > index {
                    *idx -= 1;
                }
            }
            
            println!("Removed input device with ID: {}", device_id);
            Ok(())
        } else {
            Err(format!("Device with ID {} not found", device_id).into())
        }
    }
    
    /// Shutdown the input manager and all devices
    pub fn shutdown(&mut self) -> Result<(), Box<dyn Error>> {
        if !self.is_initialized {
            return Ok(());
        }
        
        for device in &self.devices {
            let mut device = device.lock().map_err(|e| format!("Failed to lock device: {}", e))?;
            device.shutdown()?;
        }
        
        self.devices.clear();
        self.device_map.clear();
        self.event_buffer.clear();
        self.key_states.clear();
        self.mouse_button_states.clear();
        self.is_initialized = false;
        
        println!("Input manager shut down successfully");
        Ok(())
    }
    
    /// Update internal state based on incoming events
    fn update_state_from_event(&mut self, event: &InputEvent) {
        match event {
            InputEvent::KeyPress { key } => {
                self.key_states.insert(key.clone(), true);
            }
            InputEvent::KeyRelease { key } => {
                self.key_states.insert(key.clone(), false);
            }
            InputEvent::MousePress { button, position } => {
                self.mouse_button_states.insert(button.clone(), true);
                self.mouse_position = *position;
            }
            InputEvent::MouseRelease { button, position } => {
                self.mouse_button_states.insert(button.clone(), false);
                self.mouse_position = *position;
            }
            InputEvent::MouseMove { position, .. } => {
                self.mouse_position = *position;
            }
            InputEvent::MouseWheel { position, .. } => {
                self.mouse_position = *position;
            }
            _ => {
                // Other event types don't affect key/mouse state
            }
        }
    }
}

// Global instance of the input manager
static GLOBAL_INPUT_MANAGER: OnceLock<Arc<Mutex<InputManager>>> = OnceLock::new();

/// Initialize the global input manager
pub fn initialize_global_input_manager() -> Result<(), Box<dyn Error>> {
    let mut manager = InputManager::new();
    manager.initialize()?;
    
    let manager_arc = Arc::new(Mutex::new(manager));
    
    GLOBAL_INPUT_MANAGER.set(manager_arc)
        .map_err(|_| "Global input manager already initialized")?;
    
    Ok(())
}

/// Get a reference to the global input manager
pub fn get_global_input_manager() -> Result<Arc<Mutex<InputManager>>, Box<dyn Error>> {
    GLOBAL_INPUT_MANAGER.get()
        .ok_or("Global input manager not initialized".into())
        .map(|manager| manager.clone())
}

/// Add a device to the global input manager
pub fn add_global_input_device(device: Box<dyn InputDevice>) -> Result<u32, Box<dyn Error>> {
    let manager_arc = get_global_input_manager()?;
    let mut manager = manager_arc.lock().map_err(|e| format!("Failed to lock global manager: {}", e))?;
    manager.add_device(device)
}

/// Poll events from the global input manager
pub fn poll_global_input_events() -> Result<Vec<InputEvent>, Box<dyn Error>> {
    let manager_arc = get_global_input_manager()?;
    let mut manager = manager_arc.lock().map_err(|e| format!("Failed to lock global manager: {}", e))?;
    manager.poll_events()
}

/// Check if a key is pressed using the global input manager
pub fn is_global_key_pressed(key: &Key) -> bool {
    if let Ok(manager_arc) = get_global_input_manager() {
        if let Ok(manager) = manager_arc.lock() {
            manager.is_key_pressed(key)
        } else {
            false
        }
    } else {
        false
    }
}

/// Check if a mouse button is pressed using the global input manager
pub fn is_global_mouse_button_pressed(button: &MouseButton) -> bool {
    if let Ok(manager_arc) = get_global_input_manager() {
        if let Ok(manager) = manager_arc.lock() {
            manager.is_mouse_button_pressed(button)
        } else {
            false
        }
    } else {
        false
    }
}

/// Get the global mouse position
pub fn get_global_mouse_position() -> Vector2d {
    if let Ok(manager_arc) = get_global_input_manager() {
        if let Ok(manager) = manager_arc.lock() {
            manager.get_mouse_position()
        } else {
            Vector2d::new(0.0, 0.0)
        }
    } else {
        Vector2d::new(0.0, 0.0)
    }
}

/// Check if the global input system is ready
pub fn is_global_input_ready() -> bool {
    if let Ok(manager_arc) = get_global_input_manager() {
        if let Ok(manager) = manager_arc.lock() {
            manager.is_ready()
        } else {
            false
        }
    } else {
        false
    }
}