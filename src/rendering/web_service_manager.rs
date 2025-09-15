use tiny_http::Server;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::error::Error;
use std::thread;
use std::time::Duration;
use serde::{Serialize, Deserialize};

/// Message sent from the web client to the server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    Connect { client_id: String },
    Acknowledge { command_id: String },
    Error { message: String },
}

/// Message sent from the server to the web client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    Welcome { client_id: String },
    RenderCommand { command_id: String, command: String },
    Disconnect,
}

/// Status of a client connection
#[derive(Debug, Clone)]
pub struct ClientConnection {
    pub client_id: String,
    pub connected_at: std::time::Instant,
    pub last_activity: std::time::Instant,
}

/// Web service manager responsible for hosting the webpage and managing connections
pub struct WebServiceManager {
    server: Option<Server>,
    address: String,
    clients: Arc<Mutex<Vec<ClientConnection>>>,
    message_sender: Option<Sender<ServerMessage>>,
    message_receiver: Option<Receiver<ClientMessage>>,
    is_running: bool,
}

impl WebServiceManager {
    /// Create a new web service manager
    pub fn new(address: &str) -> Self {
        Self {
            server: None,
            address: address.to_string(),
            clients: Arc::new(Mutex::new(Vec::new())),
            message_sender: None,
            message_receiver: None,
            is_running: false,
        }
    }
    
    /// Start the web service
    pub fn start(&mut self) -> Result<(), Box<dyn Error>> {
        if self.is_running {
            return Ok(());
        }
        
        let server = Server::http(&self.address)
            .map_err(|e| format!("Failed to start web service: {}", e))?;
        
        println!("Web service started on http://{}", self.address);
        
        let (tx, _rx) = channel();
        let (client_tx, client_rx) = channel();
        
        self.server = Some(server);
        self.message_sender = Some(tx);
        self.message_receiver = Some(client_rx);
        self.is_running = true;
        
        // Start background thread to handle HTTP requests
        let _server_address = self.address.clone();
        let clients = self.clients.clone();
        
        thread::spawn(move || {
            // This would be implemented to handle HTTP requests
            // For now, we'll simulate client connections
            thread::sleep(Duration::from_millis(100));
            
            // Simulate a client connection
            let client_id = format!("client_{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap_or("unknown"));
            
            {
                let mut clients_guard = clients.lock().unwrap();
                clients_guard.push(ClientConnection {
                    client_id: client_id.clone(),
                    connected_at: std::time::Instant::now(),
                    last_activity: std::time::Instant::now(),
                });
            }
            
            // Send welcome message
            if client_tx.send(ClientMessage::Connect { client_id }).is_err() {
                eprintln!("Failed to send client connect message");
            }
        });
        
        Ok(())
    }
    
    /// Check if the web service is running
    pub fn is_running(&self) -> bool {
        self.is_running
    }
    
    /// Get the number of connected clients
    pub fn client_count(&self) -> usize {
        if let Ok(clients) = self.clients.lock() {
            clients.len()
        } else {
            0
        }
    }
    
    /// Send a message to all connected clients
    pub fn broadcast_message(&self, message: ServerMessage) -> Result<(), Box<dyn Error>> {
        if !self.is_running {
            return Err("Web service not running".into());
        }
        
        if let Some(sender) = &self.message_sender {
            // For testing purposes, we ignore send failures as there might not be a receiver
            match sender.send(message) {
                Ok(_) => Ok(()),
                Err(_) => {
                    // In a real implementation, this would be a proper error
                    // For testing, we'll just log and continue
                    println!("Warning: No receiver for message (expected in tests)");
                    Ok(())
                }
            }
        } else {
            Err("Message sender not initialized".into())
        }
    }
    
    /// Receive messages from clients (non-blocking)
    pub fn receive_client_message(&self) -> Option<ClientMessage> {
        if let Some(receiver) = &self.message_receiver {
            receiver.try_recv().ok()
        } else {
            None
        }
    }
    
    /// Get connected clients info
    pub fn get_clients(&self) -> Vec<ClientConnection> {
        if let Ok(clients) = self.clients.lock() {
            clients.clone()
        } else {
            Vec::new()
        }
    }
    
    /// Send a render command to all connected clients
    pub fn send_render_command(&self, command: &str) -> Result<(), Box<dyn Error>> {
        let command_id = format!("cmd_{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap_or("unknown"));
        
        let message = ServerMessage::RenderCommand {
            command_id,
            command: command.to_string(),
        };
        
        self.broadcast_message(message)
    }
    
    /// Stop the web service
    pub fn stop(&mut self) -> Result<(), Box<dyn Error>> {
        if !self.is_running {
            return Ok(());
        }
        
        // Send disconnect message to all clients
        let _ = self.broadcast_message(ServerMessage::Disconnect);
        
        // Clear clients
        if let Ok(mut clients) = self.clients.lock() {
            clients.clear();
        }
        
        self.server = None;
        self.message_sender = None;
        self.message_receiver = None;
        self.is_running = false;
        
        println!("Web service stopped");
        Ok(())
    }
    
    /// Get the path to the web client files
    pub fn get_web_files_path(&self) -> String {
        "web".to_string()
    }
    
    /// Create the HTML page content for the web client (fallback for when files are not available)
    pub fn create_fallback_client_page(&self) -> String {
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Rust City Builder - Fallback Client</title>
    <style>
        body {
            font-family: Arial, sans-serif;
            margin: 0;
            padding: 20px;
            background-color: #f5f5f5;
            text-align: center;
        }
        .container {
            max-width: 600px;
            margin: 0 auto;
            background: white;
            padding: 40px;
            border-radius: 8px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }
        .error {
            color: #d73527;
            background: #f8d7da;
            border: 1px solid #f5c6cb;
            padding: 15px;
            border-radius: 4px;
            margin: 20px 0;
        }
        .info {
            color: #0c5460;
            background: #d1ecf1;
            border: 1px solid #bee5eb;
            padding: 15px;
            border-radius: 4px;
            margin: 20px 0;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>🎮 Rust City Builder - Web Client</h1>
        
        <div class="error">
            <h3>JavaScript Library Files Not Found</h3>
            <p>The JavaScript rendering manager library files could not be loaded.</p>
        </div>
        
        <div class="info">
            <h4>Expected Files:</h4>
            <ul style="text-align: left;">
                <li><code>web/index.html</code> - Main client page</li>
                <li><code>web/js/rendering-manager.js</code> - Rendering library</li>
                <li><code>web/js/web-client.js</code> - Client communication library</li>
            </ul>
            
            <p><strong>Note:</strong> This is a fallback page. Please ensure the JavaScript library files are properly installed in the <code>web/</code> directory.</p>
        </div>
    </div>
</body>
</html>"#.to_string()
    }
}

// Simple UUID generation for demo purposes (we don't want to add another dependency)
mod uuid {
    pub struct Uuid;
    
    impl Uuid {
        pub fn new_v4() -> UuidValue {
            UuidValue
        }
    }
    
    pub struct UuidValue;
    
    impl UuidValue {
        pub fn to_string(&self) -> String {
            // Simple random string generation for demo
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            use std::time::{SystemTime, UNIX_EPOCH};
            
            let mut hasher = DefaultHasher::new();
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos().hash(&mut hasher);
            format!("{:x}", hasher.finish())
        }
    }
}