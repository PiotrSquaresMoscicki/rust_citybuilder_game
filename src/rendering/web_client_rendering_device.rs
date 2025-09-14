use std::error::Error;
use std::sync::{Arc, Mutex};
use super::{RenderingDevice, RenderCommand, RenderResult};
use super::web_service_manager::WebServiceManager;

/// Web client rendering device that communicates with a web client
/// via the WebServiceManager to tell it what to draw and where
pub struct WebClientRenderingDevice {
    web_service: Arc<Mutex<WebServiceManager>>,
    device_name: String,
    is_initialized: bool,
}

impl WebClientRenderingDevice {
    /// Create a new web client rendering device
    pub fn new(web_service_manager: WebServiceManager) -> Self {
        Self {
            web_service: Arc::new(Mutex::new(web_service_manager)),
            device_name: "WebClientRenderingDevice".to_string(),
            is_initialized: false,
        }
    }
    
    /// Get the web service manager for external access
    pub fn get_web_service(&self) -> Arc<Mutex<WebServiceManager>> {
        self.web_service.clone()
    }
    
    /// Check if there are connected clients
    pub fn has_connected_clients(&self) -> bool {
        if let Ok(service) = self.web_service.lock() {
            service.client_count() > 0
        } else {
            false
        }
    }
    
    /// Get the number of connected clients
    pub fn client_count(&self) -> usize {
        if let Ok(service) = self.web_service.lock() {
            service.client_count()
        } else {
            0
        }
    }
}

impl RenderingDevice for WebClientRenderingDevice {
    fn initialize(&mut self) -> Result<(), Box<dyn Error>> {
        if self.is_initialized {
            return Ok(());
        }
        
        let mut service = self.web_service.lock()
            .map_err(|e| format!("Failed to lock web service: {}", e))?;
        
        service.start()?;
        self.is_initialized = true;
        
        println!("WebClientRenderingDevice initialized successfully");
        Ok(())
    }
    
    fn execute_command(&mut self, command: RenderCommand) -> Result<RenderResult, Box<dyn Error>> {
        if !self.is_initialized {
            return Err("WebClientRenderingDevice not initialized".into());
        }
        
        let service = self.web_service.lock()
            .map_err(|e| format!("Failed to lock web service: {}", e))?;
        
        if !service.is_running() {
            return Err("Web service is not running".into());
        }
        
        // Convert RenderCommand to a JSON string for transmission to web client
        let command_json = match command {
            RenderCommand::Clear { r, g, b, a } => {
                format!(r#"{{"type":"Clear","params":{{"r":{},"g":{},"b":{},"a":{}}}}}"#, r, g, b, a)
            }
            RenderCommand::DrawGrid { width, height, cell_size, line_color, background_color } => {
                format!(
                    r#"{{"type":"DrawGrid","params":{{"width":{},"height":{},"cellSize":{},"lineColor":[{},{},{},{}],"backgroundColor":[{},{},{},{}]}}}}"#,
                    width, height, cell_size,
                    line_color.0, line_color.1, line_color.2, line_color.3,
                    background_color.0, background_color.1, background_color.2, background_color.3
                )
            }
        };
        
        // Send the command to all connected web clients
        service.send_render_command(&command_json)?;
        
        println!("Sent render command to web clients: {}", command_json);
        Ok(RenderResult::Success)
    }
    
    fn is_ready(&self) -> bool {
        if !self.is_initialized {
            return false;
        }
        
        if let Ok(service) = self.web_service.lock() {
            service.is_running() && service.client_count() > 0
        } else {
            false
        }
    }
    
    fn device_name(&self) -> &str {
        &self.device_name
    }
    
    fn shutdown(&mut self) -> Result<(), Box<dyn Error>> {
        if !self.is_initialized {
            return Ok(());
        }
        
        let mut service = self.web_service.lock()
            .map_err(|e| format!("Failed to lock web service: {}", e))?;
        
        service.stop()?;
        self.is_initialized = false;
        
        println!("WebClientRenderingDevice shut down successfully");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_web_client_rendering_device_creation() {
        let web_service = WebServiceManager::new("localhost:0");
        let device = WebClientRenderingDevice::new(web_service);
        
        assert_eq!(device.device_name(), "WebClientRenderingDevice");
        assert!(!device.is_ready());
        assert_eq!(device.client_count(), 0);
    }
    
    #[test]
    fn test_device_initialization() {
        let web_service = WebServiceManager::new("localhost:0");
        let mut device = WebClientRenderingDevice::new(web_service);
        
        // Device should not be ready before initialization
        assert!(!device.is_ready());
        
        // Initialize should succeed
        assert!(device.initialize().is_ok());
        
        // Should be able to shutdown after initialization
        assert!(device.shutdown().is_ok());
    }
}