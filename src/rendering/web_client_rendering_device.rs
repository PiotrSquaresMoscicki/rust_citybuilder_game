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
            RenderCommand::DrawSprite { 
                texture_id, 
                transform, 
                size, 
                color, 
                z_order, 
                uv_rect 
            } => {
                let matrix = transform.matrix();
                let (uv_min, uv_max) = uv_rect;
                format!(
                    r#"{{"type":"DrawSprite","params":{{"textureId":"{}","transform":[{},{},{},{},{},{}],"size":[{},{}],"color":[{},{},{},{}],"zOrder":{},"uvRect":[{},{},{},{}]}}}}"#,
                    texture_id,
                    matrix[0], matrix[1], matrix[2], matrix[3], matrix[4], matrix[5],
                    size.x, size.y,
                    color.r, color.g, color.b, color.a,
                    z_order,
                    uv_min.x, uv_min.y, uv_max.x, uv_max.y
                )
            }
            RenderCommand::DrawShape { 
                shape_type, 
                transform, 
                fill, 
                stroke, 
                z_order 
            } => {
                let matrix = transform.matrix();
                let shape_json = Self::serialize_shape_type(&shape_type);
                let fill_json = Self::serialize_fill_style(&fill);
                let stroke_json = if let Some(s) = stroke {
                    format!(r#"{{"color":[{},{},{},{}],"width":{}}}"#, s.color.r, s.color.g, s.color.b, s.color.a, s.width)
                } else {
                    "null".to_string()
                };
                
                format!(
                    r#"{{"type":"DrawShape","params":{{"shapeType":{},"transform":[{},{},{},{},{},{}],"fill":{},"stroke":{},"zOrder":{}}}}}"#,
                    shape_json,
                    matrix[0], matrix[1], matrix[2], matrix[3], matrix[4], matrix[5],
                    fill_json,
                    stroke_json,
                    z_order
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

impl WebClientRenderingDevice {
    /// Helper function to serialize ShapeType to JSON
    fn serialize_shape_type(shape_type: &crate::core::math::ShapeType) -> String {
        use crate::core::math::ShapeType;
        match shape_type {
            ShapeType::Circle { radius } => {
                format!(r#"{{"type":"Circle","radius":{}}}"#, radius)
            }
            ShapeType::Rectangle { width, height } => {
                format!(r#"{{"type":"Rectangle","width":{},"height":{}}}"#, width, height)
            }
            ShapeType::Triangle { vertex1, vertex2, vertex3 } => {
                format!(
                    r#"{{"type":"Triangle","vertices":[[{},{}],[{},{}],[{},{}]]}}"#,
                    vertex1.x, vertex1.y, vertex2.x, vertex2.y, vertex3.x, vertex3.y
                )
            }
            ShapeType::Line { start, end, thickness } => {
                format!(
                    r#"{{"type":"Line","start":[{},{}],"end":[{},{}],"thickness":{}}}"#,
                    start.x, start.y, end.x, end.y, thickness
                )
            }
            ShapeType::Polygon { vertices } => {
                let vertices_json: Vec<String> = vertices.iter()
                    .map(|v| format!("[{},{}]", v.x, v.y))
                    .collect();
                format!(r#"{{"type":"Polygon","vertices":[{}]}}"#, vertices_json.join(","))
            }
        }
    }
    
    /// Helper function to serialize FillStyle to JSON
    fn serialize_fill_style(fill_style: &crate::core::math::FillStyle) -> String {
        use crate::core::math::FillStyle;
        match fill_style {
            FillStyle::Solid(color) => {
                format!(r#"{{"type":"Solid","color":[{},{},{},{}]}}"#, color.r, color.g, color.b, color.a)
            }
            FillStyle::None => {
                r#"{"type":"None"}"#.to_string()
            }
        }
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