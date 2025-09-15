use tiny_http::{Server, Request, Response, Header};
use std::path::Path;
use std::fs;
use std::error::Error;
use std::thread;
use std::time::Duration;
use crate::rendering::*;

/// Enhanced HTTP server that can serve static files from the web directory
pub struct EnhancedHttpServer {
    address: String,
    web_root: String,
}

impl EnhancedHttpServer {
    /// Create a new enhanced HTTP server
    pub fn new(address: &str, web_root: &str) -> Self {
        Self {
            address: address.to_string(),
            web_root: web_root.to_string(),
        }
    }
    
    /// Start the HTTP server and handle requests
    pub fn start(&self) -> Result<(), Box<dyn Error>> {
        let server = Server::http(&self.address)
            .map_err(|e| format!("Failed to start server: {}", e))?;
        
        println!("🌐 Enhanced HTTP server started on http://{}", self.address);
        println!("📁 Serving files from: {}", self.web_root);
        println!("📡 Open http://{} in your browser to see the JavaScript rendering library client", self.address);
        println!("");
        
        for request in server.incoming_requests() {
            self.handle_request(request);
        }
        
        Ok(())
    }
    
    /// Handle an individual HTTP request
    fn handle_request(&self, request: Request) {
        let url = request.url();
        println!("Request: {} {}", request.method(), url);
        
        // Determine the file path
        let file_path = if url == "/" {
            format!("{}/index.html", self.web_root)
        } else {
            format!("{}{}", self.web_root, url)
        };
        
        // Serve the file
        match self.serve_file(&file_path) {
            Ok(response) => {
                if let Err(e) = request.respond(response) {
                    eprintln!("Failed to send response: {}", e);
                }
            }
            Err(e) => {
                eprintln!("Failed to serve file {}: {}", file_path, e);
                let response = self.create_error_response(404, "File not found", &file_path);
                if let Err(e) = request.respond(response) {
                    eprintln!("Failed to send error response: {}", e);
                }
            }
        }
    }
    
    /// Serve a file from the filesystem
    fn serve_file(&self, file_path: &str) -> Result<Response<std::fs::File>, Box<dyn Error>> {
        // Check if file exists
        if !Path::new(file_path).exists() {
            return Err(format!("File not found: {}", file_path).into());
        }
        
        let file = fs::File::open(file_path)
            .map_err(|e| format!("Cannot open file: {}", e))?;
        
        let content_type = self.get_content_type(file_path);
        let header = Header::from_bytes(&b"Content-Type"[..], content_type.as_bytes())
            .map_err(|_| "Cannot create header")?;
        
        Ok(Response::from_file(file).with_header(header))
    }
    
    /// Get the content type for a file based on its extension
    fn get_content_type(&self, file_path: &str) -> String {
        let path = Path::new(file_path);
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("html") => "text/html; charset=utf-8".to_string(),
            Some("js") => "application/javascript; charset=utf-8".to_string(),
            Some("css") => "text/css; charset=utf-8".to_string(),
            Some("json") => "application/json; charset=utf-8".to_string(),
            Some("png") => "image/png".to_string(),
            Some("jpg") | Some("jpeg") => "image/jpeg".to_string(),
            Some("gif") => "image/gif".to_string(),
            Some("svg") => "image/svg+xml".to_string(),
            Some("ico") => "image/x-icon".to_string(),
            _ => "text/plain; charset=utf-8".to_string(),
        }
    }
    
    /// Create an error response with fallback content
    fn create_error_response(&self, status_code: u16, message: &str, requested_path: &str) -> Response<std::io::Cursor<Vec<u8>>> {
        let web_service = WebServiceManager::new(&self.address);
        let fallback_content = if requested_path.ends_with("/index.html") || requested_path.ends_with("/") {
            // If the main page is missing, show fallback page
            web_service.create_fallback_client_page()
        } else {
            // For other files, show a simple error page
            format!(
                r#"<!DOCTYPE html>
<html>
<head>
    <title>Error {}</title>
    <style>
        body {{ font-family: Arial, sans-serif; text-align: center; padding: 50px; }}
        .error {{ color: #d73527; }}
        .container {{ max-width: 600px; margin: 0 auto; }}
    </style>
</head>
<body>
    <div class="container">
        <h1 class="error">Error {}</h1>
        <p>{}</p>
        <p><strong>Requested:</strong> <code>{}</code></p>
        <hr>
        <p><em>Rust City Builder - Enhanced HTTP Server</em></p>
        <p><a href="/">← Return to Home</a></p>
    </div>
</body>
</html>"#,
                status_code, status_code, message, requested_path
            )
        };
        
        let fallback_bytes = fallback_content.into_bytes();
        let header = Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..])
            .unwrap_or_else(|_| Header::from_bytes(&b"Content-Type"[..], &b"text/plain"[..]).unwrap());
        
        Response::from_data(fallback_bytes)
            .with_header(header)
            .with_status_code(status_code)
    }
}

/// Start the enhanced HTTP server that serves the JavaScript library files
pub fn start_rendering_server(address: &str) -> Result<(), Box<dyn Error>> {
    println!("🎮 Starting Rust City Builder with JavaScript Library Server");
    
    // Wait a moment for global rendering manager to be ready
    thread::sleep(Duration::from_millis(100));
    
    // Send initial grid render command using global manager
    if let Ok(result) = render_global_grid(12, 10, 35.0) {
        match result {
            RenderResult::Success => println!("✅ Initial grid rendering command sent"),
            RenderResult::Error(msg) => println!("⚠️  Grid rendering warning: {}", msg),
        }
    }
    
    let server = EnhancedHttpServer::new(address, "web");
    server.start()
}

/// Demonstrate the rendering system with a web client
pub fn demonstrate_rendering_with_web_client() {
    println!("🎨 Enhanced Rendering System with Web Client");
    println!("===========================================");
    
    // Start the rendering server
    if let Err(e) = start_rendering_server("localhost:8082") {
        eprintln!("Failed to start rendering server: {}", e);
    }
}