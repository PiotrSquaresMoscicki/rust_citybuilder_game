# JavaScript Rendering Manager Library

This directory contains the JavaScript Rendering Manager Library for the Rust City Builder game. The library provides a modular, reusable interface for processing rendering commands sent from the Rust backend.

## Overview

The JavaScript library replaces the previous embedded HTML/JavaScript approach with a proper modular architecture consisting of separate, reusable JavaScript files that can be served as static assets.

## Files

### Core Libraries

- **`rendering-manager.js`** - Core rendering functionality
  - Handles canvas operations and drawing commands
  - Supports grids, sprites, shapes, and other rendering primitives
  - Manages command history and statistics
  - Provides error handling and validation

- **`web-client.js`** - Client communication and management
  - Manages connection state with the Rust server
  - Handles message parsing and event dispatching
  - Provides UI integration and status management
  - Simulates server communication for demo purposes

### Web Interface

- **`index.html`** - Main client web page
  - Responsive design with modern UI
  - Real-time statistics and activity logging
  - Interactive controls for testing rendering features
  - Professional styling and user experience

## Architecture

### RenderingManager Class

The core rendering class that handles all canvas operations:

```javascript
const canvas = document.getElementById('myCanvas');
const renderingManager = new RenderingManager(canvas);

// Process a render command
renderingManager.processRenderCommand({
    type: 'DrawGrid',
    params: {
        width: 10,
        height: 8,
        cellSize: 40,
        lineColor: [0, 0, 0, 1],
        backgroundColor: [1, 1, 1, 1]
    }
});
```

### WebRenderingClient Class

The client communication class that manages server interaction:

```javascript
const canvas = document.getElementById('myCanvas');
const client = new WebRenderingClient(canvas, {
    autoConnect: true,
    logMessages: true
});

// Set up UI elements
client.setUIElements(statusElement, logElement);

// Set up event handlers
client.setEventHandlers({
    onConnectionChange: (connected, clientId) => {
        console.log(`Connection changed: ${connected}`);
    },
    onRenderCommand: (type, params) => {
        console.log(`Received command: ${type}`);
    }
});
```

## Supported Render Commands

### DrawGrid
Renders a grid with specified dimensions and styling:
```json
{
    "type": "DrawGrid",
    "params": {
        "width": 12,
        "height": 10,
        "cellSize": 35,
        "lineColor": [0.2, 0.4, 0.8, 1],
        "backgroundColor": [0.95, 0.95, 1, 1]
    }
}
```

### DrawShape
Renders geometric shapes with fills and strokes:
```json
{
    "type": "DrawShape",
    "params": {
        "shapeType": {"type": "Circle", "radius": 30},
        "transform": [1, 0, 0, 1, 150, 150],
        "fill": {"type": "Solid", "color": [0.8, 0.2, 0.2, 0.7]},
        "stroke": {"color": [0, 0, 0, 1], "width": 3}
    }
}
```

### DrawSprite
Renders textured sprites with transforms:
```json
{
    "type": "DrawSprite", 
    "params": {
        "textureId": "player",
        "transform": [1, 0, 0, 1, 200, 200],
        "size": [32, 32],
        "color": [1, 1, 1, 1]
    }
}
```

### Clear
Clears the canvas with optional background color:
```json
{
    "type": "Clear",
    "params": {
        "r": 1, "g": 1, "b": 1, "a": 1
    }
}
```

## Features

### Modular Architecture
- Separate concerns between rendering and communication
- Reusable components that can be integrated into different projects
- Clean API boundaries and interfaces

### Canvas Rendering
- High-performance HTML5 Canvas rendering
- Support for complex shapes, sprites, and effects
- Automatic scaling and coordinate system management

### Real-time Statistics
- Command processing counters
- Connection status monitoring
- Performance metrics and debugging information

### Error Handling
- Graceful degradation for unsupported commands
- Connection failure recovery
- Detailed logging and debugging support

### Responsive Design
- Mobile-friendly interface
- Adaptive layout for different screen sizes
- Professional styling and user experience

## Integration with Rust Backend

The JavaScript library is designed to work seamlessly with the Rust rendering system:

1. **Rust Server** - Sends JSON render commands via HTTP
2. **Enhanced HTTP Server** - Serves the JavaScript library files as static assets
3. **JavaScript Client** - Processes commands and renders to canvas
4. **Web Interface** - Provides user interaction and status monitoring

## Usage

### Running the Server

```bash
# Start the enhanced web server with JavaScript library
cargo run web-render
```

### Accessing the Client

Open your browser to `http://localhost:8082` to access the JavaScript rendering client.

### Testing Features

The web interface provides several test buttons:

- **Connect/Disconnect** - Manage server connection
- **Test Grid** - Render a sample grid
- **Test Shape** - Render geometric shapes
- **Clear Canvas** - Clear the rendering surface
- **Resize Canvas** - Change canvas dimensions

## Development

### Adding New Render Commands

1. Add the command definition to `RenderingManager.processRenderCommand()`
2. Implement the rendering logic in the appropriate method
3. Update the Rust backend to send the new command format
4. Test the integration end-to-end

### Extending the Client

1. Modify the `WebRenderingClient` class for new communication features
2. Add UI elements to `index.html` for new functionality
3. Update event handlers and status management as needed

## Browser Compatibility

The library supports modern browsers with HTML5 Canvas support:
- Chrome 50+
- Firefox 45+
- Safari 10+
- Edge 79+

## License

This library is part of the Rust City Builder project and follows the same licensing terms.