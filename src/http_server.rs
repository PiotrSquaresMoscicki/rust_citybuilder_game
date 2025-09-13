use tiny_http::{Server, Request, Response, Header};
use std::io;

/// Simple HTTP server that serves a hello world webpage
pub struct HelloWorldServer {
    server: Server,
}

impl HelloWorldServer {
    /// Create a new HTTP server listening on the specified address
    pub fn new(address: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let server = Server::http(address)
            .map_err(|e| format!("Failed to start server: {}", e))?;
        
        println!("HTTP server started on http://{}", address);
        println!("Visit http://{} in your browser to see the hello world page", address);
        
        Ok(Self { server })
    }
    
    /// Start the server and handle incoming requests
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Server is running... Press Ctrl+C to stop");
        
        for request in self.server.incoming_requests() {
            match self.handle_request(request) {
                Ok(_) => {},
                Err(e) => eprintln!("Error handling request: {}", e),
            }
        }
        
        Ok(())
    }
    
    /// Handle a single HTTP request
    fn handle_request(&self, request: Request) -> Result<(), io::Error> {
        let method = request.method();
        let url = request.url();
        
        println!("{} {}", method, url);
        
        // Create hello world HTML response
        let html_content = self.create_hello_world_html();
        
        // Create HTTP response with proper headers
        let header = Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..])
            .expect("Invalid header");
        
        let response = Response::from_string(html_content)
            .with_header(header);
        
        // Send the response
        request.respond(response)
    }
    
    /// Generate the hello world HTML content
    fn create_hello_world_html(&self) -> String {
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Hello World - Rust City Builder</title>
    <style>
        body {
            font-family: Arial, sans-serif;
            max-width: 800px;
            margin: 0 auto;
            padding: 2rem;
            background-color: #f5f5f5;
        }
        .container {
            background: white;
            padding: 2rem;
            border-radius: 8px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }
        h1 {
            color: #333;
            text-align: center;
            margin-bottom: 1rem;
        }
        .subtitle {
            color: #666;
            text-align: center;
            font-style: italic;
            margin-bottom: 2rem;
        }
        .info {
            background: #e7f3ff;
            padding: 1rem;
            border-left: 4px solid #2196F3;
            margin: 1rem 0;
        }
        .features {
            margin: 2rem 0;
        }
        .features ul {
            list-style-type: none;
            padding: 0;
        }
        .features li {
            background: #f8f9fa;
            margin: 0.5rem 0;
            padding: 0.5rem 1rem;
            border-left: 3px solid #28a745;
        }
        .footer {
            text-align: center;
            margin-top: 2rem;
            color: #888;
            font-size: 0.9rem;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>🎉 Hello World! 🎉</h1>
        <p class="subtitle">Welcome to the Rust City Builder Game HTTP Server</p>
        
        <div class="info">
            <strong>🚀 Server Status:</strong> Running successfully!<br>
            <strong>⚡ Technology:</strong> Rust with tiny_http<br>
            <strong>🎯 Purpose:</strong> Simple HTTP server demonstration
        </div>
        
        <div class="features">
            <h3>🏗️ Project Features:</h3>
            <ul>
                <li>✅ Entity Component System (ECS) library implemented</li>
                <li>✅ Simple HTTP server with hello world webpage</li>
                <li>✅ Rust-based city builder game foundation</li>
                <li>✅ Modular architecture for easy extension</li>
            </ul>
        </div>
        
        <div class="info">
            <h3>🎮 About This Project</h3>
            <p>This is a Rust-based city builder game that includes:</p>
            <ul>
                <li><strong>Entity Component System:</strong> A flexible ECS for game entities</li>
                <li><strong>HTTP Server:</strong> Simple web server for potential web interface</li>
                <li><strong>Modular Design:</strong> Clean separation of concerns</li>
            </ul>
        </div>
        
        <div class="footer">
            <p>Built with ❤️ using Rust</p>
            <p>Visit <a href="/">this page</a> to refresh</p>
        </div>
    </div>
</body>
</html>"#.to_string()
    }
}

/// Create and start the hello world HTTP server
pub fn start_hello_world_server(address: &str) -> Result<(), Box<dyn std::error::Error>> {
    let server = HelloWorldServer::new(address)?;
    server.run()
}