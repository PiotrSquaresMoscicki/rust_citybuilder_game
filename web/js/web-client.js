/**
 * Rust City Builder - Web Client Communication Library
 * 
 * This library handles communication between the web client and the Rust server,
 * managing connections, message parsing, and event handling.
 */

/**
 * Web client that communicates with the Rust server and processes render commands
 */
class WebRenderingClient {
    /**
     * Create a new web rendering client
     * @param {HTMLCanvasElement} canvas - Canvas element for rendering
     * @param {Object} options - Configuration options
     */
    constructor(canvas, options = {}) {
        this.canvas = canvas;
        this.renderingManager = new RenderingManager(canvas);
        this.options = {
            autoConnect: true,
            connectionDelay: 1000,
            logMessages: true,
            ...options
        };
        
        // Connection state
        this.connected = false;
        this.clientId = null;
        this.connectionAttempts = 0;
        this.maxConnectionAttempts = 5;
        
        // UI elements
        this.statusElement = null;
        this.logElement = null;
        
        // Event handlers
        this.onConnectionChange = null;
        this.onRenderCommand = null;
        this.onError = null;
        
        this.initialize();
    }
    
    /**
     * Initialize the client
     */
    initialize() {
        this.logMessage('Web rendering client initialized');
        
        if (this.options.autoConnect) {
            setTimeout(() => this.connect(), this.options.connectionDelay);
        }
    }
    
    /**
     * Set UI elements for status and logging
     * @param {HTMLElement} statusElement - Element to display connection status
     * @param {HTMLElement} logElement - Element to display log messages
     */
    setUIElements(statusElement, logElement) {
        this.statusElement = statusElement;
        this.logElement = logElement;
        this.updateStatus();
    }
    
    /**
     * Set event handlers
     * @param {Object} handlers - Object with event handler functions
     */
    setEventHandlers(handlers) {
        if (handlers.onConnectionChange) {
            this.onConnectionChange = handlers.onConnectionChange;
        }
        if (handlers.onRenderCommand) {
            this.onRenderCommand = handlers.onRenderCommand;
        }
        if (handlers.onError) {
            this.onError = handlers.onError;
        }
    }
    
    /**
     * Attempt to connect to the server
     */
    connect() {
        if (this.connected) {
            this.logMessage('Already connected');
            return;
        }
        
        this.connectionAttempts++;
        this.logMessage(`Attempting to connect (attempt ${this.connectionAttempts})...`);
        
        // Simulate connection process
        // In a real implementation, this would establish WebSocket or polling connection
        setTimeout(() => {
            if (this.connectionAttempts <= this.maxConnectionAttempts) {
                this.connected = true;
                this.clientId = this.generateClientId();
                this.updateStatus();
                this.logMessage(`Connected to server as ${this.clientId}`);
                
                if (this.onConnectionChange) {
                    this.onConnectionChange(true, this.clientId);
                }
                
                // Start receiving commands simulation
                this.startCommandSimulation();
            } else {
                this.logMessage('Failed to connect after maximum attempts');
                if (this.onError) {
                    this.onError('Connection failed');
                }
            }
        }, 500 + Math.random() * 1000);
    }
    
    /**
     * Disconnect from the server
     */
    disconnect() {
        if (!this.connected) {
            this.logMessage('Not connected');
            return;
        }
        
        this.connected = false;
        this.clientId = null;
        this.updateStatus();
        this.logMessage('Disconnected from server');
        
        if (this.onConnectionChange) {
            this.onConnectionChange(false, null);
        }
    }
    
    /**
     * Reconnect to the server
     */
    reconnect() {
        this.disconnect();
        this.connectionAttempts = 0;
        setTimeout(() => this.connect(), 1000);
    }
    
    /**
     * Update the connection status display
     */
    updateStatus() {
        if (!this.statusElement) return;
        
        if (this.connected) {
            this.statusElement.className = 'status connected';
            this.statusElement.textContent = `Connected (${this.clientId || 'unknown'})`;
        } else {
            this.statusElement.className = 'status disconnected';
            this.statusElement.textContent = 'Disconnected';
        }
    }
    
    /**
     * Log a message to the console and UI
     * @param {string} message - Message to log
     */
    logMessage(message) {
        if (this.options.logMessages) {
            console.log(`[WebRenderingClient] ${message}`);
        }
        
        if (this.logElement) {
            const timestamp = new Date().toLocaleTimeString();
            const logEntry = `[${timestamp}] ${message}\n`;
            this.logElement.textContent += logEntry;
            this.logElement.scrollTop = this.logElement.scrollHeight;
        }
    }
    
    /**
     * Process a render command received from the server
     * @param {string} commandType - Type of the command
     * @param {Object} params - Command parameters
     */
    receiveRenderCommand(commandType, params) {
        if (!this.connected) {
            this.logMessage('Cannot process command: not connected');
            return;
        }
        
        this.logMessage(`Received render command: ${commandType}`);
        if (this.options.logMessages) {
            this.logMessage(`Parameters: ${JSON.stringify(params)}`);
        }
        
        const command = {
            type: commandType,
            params: params
        };
        
        // Process the command through the rendering manager
        this.renderingManager.processRenderCommand(command);
        
        // Call custom handler if provided
        if (this.onRenderCommand) {
            this.onRenderCommand(commandType, params);
        }
    }
    
    /**
     * Parse and process a JSON render command
     * @param {string} jsonCommand - JSON string containing the command
     */
    processJSONCommand(jsonCommand) {
        try {
            const command = JSON.parse(jsonCommand);
            this.receiveRenderCommand(command.type, command.params);
        } catch (error) {
            this.logMessage(`Error parsing JSON command: ${error.message}`);
            if (this.onError) {
                this.onError(`JSON parse error: ${error.message}`);
            }
        }
    }
    
    /**
     * Send an acknowledgment to the server
     * @param {string} commandId - ID of the command to acknowledge
     */
    sendAcknowledgment(commandId) {
        if (!this.connected) return;
        
        // In a real implementation, this would send the acknowledgment to the server
        this.logMessage(`Sent acknowledgment for command: ${commandId}`);
    }
    
    /**
     * Get client statistics
     * @returns {Object} Object with client statistics
     */
    getStatistics() {
        const history = this.renderingManager.getCommandHistory();
        const dimensions = this.renderingManager.getDimensions();
        
        return {
            connected: this.connected,
            clientId: this.clientId,
            connectionAttempts: this.connectionAttempts,
            commandsProcessed: history.length,
            canvasDimensions: dimensions,
            lastCommand: history.length > 0 ? history[history.length - 1] : null
        };
    }
    
    /**
     * Clear the canvas and command history
     */
    clearAll() {
        this.renderingManager.clear();
        this.renderingManager.clearCommandHistory();
        this.logMessage('Canvas and command history cleared');
    }
    
    /**
     * Resize the canvas
     * @param {number} width - New width
     * @param {number} height - New height
     */
    resizeCanvas(width, height) {
        this.renderingManager.resize(width, height);
        this.logMessage(`Canvas resized to ${width}x${height}`);
    }
    
    /**
     * Generate a unique client ID
     * @returns {string} Unique client ID
     */
    generateClientId() {
        return 'client_' + Math.random().toString(36).substr(2, 9);
    }
    
    /**
     * Simulate receiving commands (for demo purposes)
     */
    startCommandSimulation() {
        if (!this.connected) return;
        
        // Send a grid command after a delay
        setTimeout(() => {
            if (this.connected) {
                this.receiveRenderCommand('DrawGrid', {
                    width: 10,
                    height: 8,
                    cellSize: 40,
                    lineColor: [0, 0, 0, 1],
                    backgroundColor: [1, 1, 1, 1]
                });
            }
        }, 2000);
        
        // Send additional demo commands
        setTimeout(() => {
            if (this.connected) {
                this.receiveRenderCommand('DrawShape', {
                    shapeType: { type: 'Circle', radius: 20 },
                    transform: [1, 0, 0, 1, 100, 100],
                    fill: { type: 'Solid', color: [1, 0, 0, 0.8] },
                    stroke: { color: [0, 0, 0, 1], width: 2 },
                    zOrder: 1
                });
            }
        }, 4000);
    }
    
    /**
     * Stop the client and clean up resources
     */
    shutdown() {
        this.disconnect();
        this.renderingManager = null;
        this.logMessage('Client shut down');
    }
}

// Export for use as ES6 module or global
if (typeof module !== 'undefined' && module.exports) {
    module.exports = WebRenderingClient;
} else if (typeof window !== 'undefined') {
    window.WebRenderingClient = WebRenderingClient;
}