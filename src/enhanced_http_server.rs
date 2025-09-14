use crate::rendering::*;
use std::thread;
use std::time::Duration;

/// Enhanced HTTP server that includes rendering capabilities
pub fn start_rendering_server(address: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("🎮 Starting Rust City Builder with Rendering Server");
    
    // Create a web service just for serving the page (separate from global rendering)
    let web_service_for_page = WebServiceManager::new(address);
    let page_content = web_service_for_page.create_client_page();
    
    println!("✅ Using existing global rendering manager");
    
    // Wait a moment for initialization
    thread::sleep(Duration::from_millis(100));
    
    // Send initial grid render command using global manager
    if let Ok(result) = render_global_grid(12, 10, 35.0) {
        match result {
            RenderResult::Success => println!("✅ Initial grid rendering command sent"),
            RenderResult::Error(msg) => println!("⚠️  Grid rendering warning: {}", msg),
        }
    }
    
    // Start basic HTTP server to serve the client page
    use tiny_http::{Server, Response, Header};
    
    let server = Server::http(address)
        .map_err(|e| format!("Failed to start HTTP server: {}", e))?;
    
    println!("🌐 HTTP server started on http://{}", address);
    println!("📡 Open http://{} in your browser to see the web rendering client", address);
    println!("🎯 The client will automatically render a black and white grid");
    println!("");
    println!("Press Ctrl+C to stop the server");
    
    for request in server.incoming_requests() {
        let method = request.method();
        let url = request.url();
        
        println!("{} {}", method, url);
        
        // Serve the rendering client page for all requests
        let header = Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..])
            .expect("Invalid header");
        
        let response = Response::from_string(&page_content)
            .with_header(header);
        
        if let Err(e) = request.respond(response) {
            eprintln!("Error responding to request: {}", e);
        }
        
        // Send another grid render command for each request to demonstrate
        thread::spawn(|| {
            thread::sleep(Duration::from_millis(1000));
            let _ = render_global_grid(8, 6, 50.0);
        });
    }
    
    Ok(())
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