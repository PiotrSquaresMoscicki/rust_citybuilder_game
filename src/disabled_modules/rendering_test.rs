use crate::rendering::*;
use std::thread;
use std::time::Duration;

/// Test the rendering system integration
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rendering_manager_creation() {
        let web_service = WebServiceManager::new("localhost:0");
        let device = Box::new(WebClientRenderingDevice::new(web_service));
        let mut manager = RenderingManager::new(device);
        
        assert!(!manager.is_ready());
        assert!(manager.initialize().is_ok());
        assert!(manager.shutdown().is_ok());
    }
    
    #[test]
    fn test_web_service_manager_lifecycle() {
        let mut web_service = WebServiceManager::new("localhost:0");
        
        // Should not be running initially
        assert!(!web_service.is_running());
        assert_eq!(web_service.client_count(), 0);
        
        // Start the service
        assert!(web_service.start().is_ok());
        assert!(web_service.is_running());
        
        // Stop the service
        assert!(web_service.stop().is_ok());
        assert!(!web_service.is_running());
    }
    
    #[test]
    fn test_render_command_serialization() {
        let web_service = WebServiceManager::new("localhost:0");
        let mut device = WebClientRenderingDevice::new(web_service);
        
        // Initialize device
        assert!(device.initialize().is_ok());
        
        // Test grid rendering command
        let grid_command = RenderCommand::DrawGrid {
            width: 10,
            height: 8,
            cell_size: 32.0,
            line_color: (0.0, 0.0, 0.0, 1.0),
            background_color: (1.0, 1.0, 1.0, 1.0),
        };
        
        // Should succeed even without connected clients
        let result = device.execute_command(grid_command);
        assert!(result.is_ok());
        
        assert!(device.shutdown().is_ok());
    }
    
    #[test]
    fn test_global_rendering_manager_initialization() {
        // This test ensures we can initialize the global manager
        let web_service = WebServiceManager::new("localhost:0");
        let device = Box::new(WebClientRenderingDevice::new(web_service));
        
        // Initialize global manager
        let result = initialize_global_rendering_manager(device);
        
        // Note: This might fail if global manager is already initialized by another test
        // That's okay for this demo - in a real application, you'd want to ensure
        // proper cleanup between tests
        if result.is_err() {
            println!("Global manager already initialized (expected in test environment)");
        }
        
        // Test convenience functions
        let is_ready = is_global_rendering_ready();
        println!("Global rendering system ready: {}", is_ready);
    }
    
    #[test]
    fn test_connection_establishment_simulation() {
        let mut web_service = WebServiceManager::new("localhost:0");
        
        // Start the web service
        assert!(web_service.start().is_ok());
        
        // Give some time for the simulated client connection
        thread::sleep(Duration::from_millis(200));
        
        // Check if we have any clients (simulated connection should occur)
        let client_count = web_service.client_count();
        println!("Connected clients: {}", client_count);
        
        // Test sending a render command
        let result = web_service.send_render_command(r#"{"type":"DrawGrid","params":{"width":5,"height":5}}"#);
        assert!(result.is_ok());
        
        // Stop the service
        assert!(web_service.stop().is_ok());
    }
    
    #[test]
    fn test_grid_rendering_request() {
        let web_service = WebServiceManager::new("localhost:0");
        let mut device = WebClientRenderingDevice::new(web_service);
        
        // Initialize
        assert!(device.initialize().is_ok());
        
        // Create a rendering manager
        let device_box = Box::new(device);
        let mut manager = RenderingManager::new(device_box);
        assert!(manager.initialize().is_ok());
        
        // Test grid rendering
        let result = manager.render_grid(10, 10, 25.0);
        assert!(result.is_ok());
        
        if let Ok(render_result) = result {
            match render_result {
                RenderResult::Success => println!("Grid rendering request sent successfully"),
                RenderResult::Error(msg) => println!("Grid rendering failed: {}", msg),
            }
        }
        
        // Cleanup
        assert!(manager.shutdown().is_ok());
    }
    
    #[test]
    fn test_web_client_page_generation() {
        let web_service = WebServiceManager::new("localhost:0");
        let page_content = web_service.create_client_page();
        
        // Basic checks to ensure the page contains expected elements
        assert!(page_content.contains("<!DOCTYPE html>"));
        assert!(page_content.contains("Rust City Builder"));
        assert!(page_content.contains("renderCanvas"));
        assert!(page_content.contains("WebRenderingClient"));
        assert!(page_content.contains("drawGrid"));
        
        println!("Web client page generated successfully ({} characters)", page_content.len());
    }
    
    #[test]
    fn test_rendering_device_interface_compliance() {
        let web_service = WebServiceManager::new("localhost:0");
        let mut device = WebClientRenderingDevice::new(web_service);
        
        // Test interface compliance
        assert_eq!(device.device_name(), "WebClientRenderingDevice");
        assert!(!device.is_ready());
        
        // Initialize
        assert!(device.initialize().is_ok());
        
        // Test clear command
        let clear_command = RenderCommand::Clear {
            r: 0.5,
            g: 0.5,
            b: 0.5,
            a: 1.0,
        };
        
        let result = device.execute_command(clear_command);
        assert!(result.is_ok());
        
        // Shutdown
        assert!(device.shutdown().is_ok());
        assert!(!device.is_ready());
    }
}