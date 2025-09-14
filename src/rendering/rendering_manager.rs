use std::sync::{Arc, Mutex, OnceLock};
use std::error::Error;
use super::{RenderingDevice, RenderCommand, RenderResult};

/// Global rendering manager that can be accessed from anywhere in the application
/// This is not an ECS system - it's a globally accessible service
pub struct RenderingManager {
    device: Arc<Mutex<Box<dyn RenderingDevice>>>,
    is_initialized: bool,
}

impl RenderingManager {
    /// Create a new rendering manager with the specified device
    pub fn new(device: Box<dyn RenderingDevice>) -> Self {
        Self {
            device: Arc::new(Mutex::new(device)),
            is_initialized: false,
        }
    }
    
    /// Initialize the rendering manager and its device
    pub fn initialize(&mut self) -> Result<(), Box<dyn Error>> {
        if self.is_initialized {
            return Ok(());
        }
        
        let mut device = self.device.lock().map_err(|e| format!("Failed to lock device: {}", e))?;
        device.initialize()?;
        self.is_initialized = true;
        
        Ok(())
    }
    
    /// Execute a rendering command
    pub fn execute_command(&self, command: RenderCommand) -> Result<RenderResult, Box<dyn Error>> {
        if !self.is_initialized {
            return Err("Rendering manager not initialized".into());
        }
        
        let mut device = self.device.lock().map_err(|e| format!("Failed to lock device: {}", e))?;
        device.execute_command(command)
    }
    
    /// Check if the rendering system is ready
    pub fn is_ready(&self) -> bool {
        if !self.is_initialized {
            return false;
        }
        
        if let Ok(device) = self.device.lock() {
            device.is_ready()
        } else {
            false
        }
    }
    
    /// Get the device name
    pub fn device_name(&self) -> Result<String, Box<dyn Error>> {
        let device = self.device.lock().map_err(|e| format!("Failed to lock device: {}", e))?;
        Ok(device.device_name().to_string())
    }
    
    /// Render a black and white grid
    pub fn render_grid(&self, width: u32, height: u32, cell_size: f32) -> Result<RenderResult, Box<dyn Error>> {
        // Black and white grid: white background, black lines
        let command = RenderCommand::DrawGrid {
            width,
            height,
            cell_size,
            line_color: (0.0, 0.0, 0.0, 1.0),      // Black lines
            background_color: (1.0, 1.0, 1.0, 1.0), // White background
        };
        
        self.execute_command(command)
    }
    
    /// Shutdown the rendering manager
    pub fn shutdown(&mut self) -> Result<(), Box<dyn Error>> {
        if !self.is_initialized {
            return Ok(());
        }
        
        let mut device = self.device.lock().map_err(|e| format!("Failed to lock device: {}", e))?;
        device.shutdown()?;
        self.is_initialized = false;
        
        Ok(())
    }
}

// Global instance of the rendering manager
static GLOBAL_RENDERING_MANAGER: OnceLock<Arc<Mutex<RenderingManager>>> = OnceLock::new();

/// Initialize the global rendering manager with a specific device
pub fn initialize_global_rendering_manager(device: Box<dyn RenderingDevice>) -> Result<(), Box<dyn Error>> {
    let mut manager = RenderingManager::new(device);
    manager.initialize()?;
    
    let manager_arc = Arc::new(Mutex::new(manager));
    
    GLOBAL_RENDERING_MANAGER.set(manager_arc)
        .map_err(|_| "Global rendering manager already initialized")?;
    
    Ok(())
}

/// Get a reference to the global rendering manager
pub fn get_global_rendering_manager() -> Result<Arc<Mutex<RenderingManager>>, Box<dyn Error>> {
    GLOBAL_RENDERING_MANAGER.get()
        .ok_or("Global rendering manager not initialized".into())
        .map(|manager| manager.clone())
}

/// Convenience function to render a grid using the global manager
pub fn render_global_grid(width: u32, height: u32, cell_size: f32) -> Result<RenderResult, Box<dyn Error>> {
    let manager_arc = get_global_rendering_manager()?;
    let manager = manager_arc.lock().map_err(|e| format!("Failed to lock global manager: {}", e))?;
    manager.render_grid(width, height, cell_size)
}

/// Convenience function to check if the global rendering system is ready
pub fn is_global_rendering_ready() -> bool {
    if let Ok(manager_arc) = get_global_rendering_manager() {
        if let Ok(manager) = manager_arc.lock() {
            manager.is_ready()
        } else {
            false
        }
    } else {
        false
    }
}