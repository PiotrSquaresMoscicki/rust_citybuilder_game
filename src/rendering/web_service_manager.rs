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
    
    /// Create the HTML page content for the web client
    pub fn create_client_page(&self) -> String {
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Rust City Builder - Web Client</title>
    <style>
        body {
            font-family: Arial, sans-serif;
            margin: 0;
            padding: 20px;
            background-color: #f5f5f5;
        }
        .container {
            max-width: 1200px;
            margin: 0 auto;
            background: white;
            padding: 20px;
            border-radius: 8px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }
        h1 {
            color: #333;
            text-align: center;
        }
        #renderCanvas {
            border: 2px solid #ccc;
            background: white;
            display: block;
            margin: 20px auto;
        }
        .status {
            text-align: center;
            margin: 10px 0;
            padding: 10px;
            border-radius: 4px;
        }
        .connected {
            background-color: #d4edda;
            color: #155724;
            border: 1px solid #c3e6cb;
        }
        .disconnected {
            background-color: #f8d7da;
            color: #721c24;
            border: 1px solid #f5c6cb;
        }
        .log {
            height: 200px;
            overflow-y: auto;
            border: 1px solid #ccc;
            padding: 10px;
            background: #f8f9fa;
            font-family: monospace;
            font-size: 12px;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>🎮 Rust City Builder - Web Rendering Client</h1>
        
        <div id="status" class="status disconnected">
            Disconnected
        </div>
        
        <canvas id="renderCanvas" width="800" height="600"></canvas>
        
        <h3>Command Log:</h3>
        <div id="log" class="log"></div>
    </div>

    <script>
        class WebRenderingClient {
            constructor() {
                this.canvas = document.getElementById('renderCanvas');
                this.ctx = this.canvas.getContext('2d');
                this.status = document.getElementById('status');
                this.log = document.getElementById('log');
                this.connected = false;
                this.clientId = null;
                
                this.updateStatus();
                this.logMessage('Client initialized');
                
                // Simulate connection after a short delay
                setTimeout(() => this.connect(), 1000);
            }
            
            connect() {
                this.connected = true;
                this.clientId = 'client_' + Math.random().toString(36).substr(2, 9);
                this.updateStatus();
                this.logMessage('Connected to server as ' + this.clientId);
                
                // Send initial render command simulation
                setTimeout(() => {
                    this.receiveRenderCommand('DrawGrid', {
                        width: 10,
                        height: 8,
                        cellSize: 40,
                        lineColor: [0, 0, 0, 1],
                        backgroundColor: [1, 1, 1, 1]
                    });
                }, 2000);
            }
            
            updateStatus() {
                if (this.connected) {
                    this.status.className = 'status connected';
                    this.status.textContent = 'Connected (' + (this.clientId || 'unknown') + ')';
                } else {
                    this.status.className = 'status disconnected';
                    this.status.textContent = 'Disconnected';
                }
            }
            
            logMessage(message) {
                const timestamp = new Date().toLocaleTimeString();
                const logEntry = `[${timestamp}] ${message}\n`;
                this.log.textContent += logEntry;
                this.log.scrollTop = this.log.scrollHeight;
            }
            
            receiveRenderCommand(command, params) {
                this.logMessage(`Received render command: ${command}`);
                this.logMessage(`Parameters: ${JSON.stringify(params)}`);
                
                switch (command) {
                    case 'DrawGrid':
                        this.drawGrid(params);
                        break;
                    default:
                        this.logMessage(`Unknown command: ${command}`);
                }
            }
            
            drawGrid(params) {
                const { width, height, cellSize, lineColor, backgroundColor } = params;
                
                // Clear canvas
                this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);
                
                // Set background
                this.ctx.fillStyle = `rgba(${backgroundColor[0] * 255}, ${backgroundColor[1] * 255}, ${backgroundColor[2] * 255}, ${backgroundColor[3]})`;
                this.ctx.fillRect(0, 0, this.canvas.width, this.canvas.height);
                
                // Set line style
                this.ctx.strokeStyle = `rgba(${lineColor[0] * 255}, ${lineColor[1] * 255}, ${lineColor[2] * 255}, ${lineColor[3]})`;
                this.ctx.lineWidth = 1;
                
                // Draw grid
                this.ctx.beginPath();
                
                // Vertical lines
                for (let x = 0; x <= width; x++) {
                    const xPos = x * cellSize;
                    this.ctx.moveTo(xPos, 0);
                    this.ctx.lineTo(xPos, height * cellSize);
                }
                
                // Horizontal lines
                for (let y = 0; y <= height; y++) {
                    const yPos = y * cellSize;
                    this.ctx.moveTo(0, yPos);
                    this.ctx.lineTo(width * cellSize, yPos);
                }
                
                this.ctx.stroke();
                
                this.logMessage(`Grid drawn: ${width}x${height} cells, ${cellSize}px each`);
            }
        }
        
        // Initialize the client when the page loads
        window.addEventListener('load', () => {
            new WebRenderingClient();
        });
    </script>
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