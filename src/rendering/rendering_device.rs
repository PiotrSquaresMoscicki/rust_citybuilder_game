use std::error::Error;

/// Commands that can be sent to a rendering device
#[derive(Debug, Clone)]
pub enum RenderCommand {
    /// Clear the screen with a specified color
    Clear { r: f32, g: f32, b: f32, a: f32 },
    /// Draw a grid with specified parameters
    DrawGrid {
        width: u32,
        height: u32,
        cell_size: f32,
        line_color: (f32, f32, f32, f32),
        background_color: (f32, f32, f32, f32),
    },
}

/// Result of a rendering operation
#[derive(Debug, Clone)]
pub enum RenderResult {
    Success,
    Error(String),
}

/// Trait defining the interface for rendering devices
/// Allows multiple implementations for different platforms (web, native, etc.)
pub trait RenderingDevice: Send + Sync {
    /// Initialize the rendering device
    fn initialize(&mut self) -> Result<(), Box<dyn Error>>;
    
    /// Execute a rendering command
    fn execute_command(&mut self, command: RenderCommand) -> Result<RenderResult, Box<dyn Error>>;
    
    /// Check if the device is ready to receive commands
    fn is_ready(&self) -> bool;
    
    /// Get the name/type of this rendering device
    fn device_name(&self) -> &str;
    
    /// Shutdown the rendering device
    fn shutdown(&mut self) -> Result<(), Box<dyn Error>>;
}