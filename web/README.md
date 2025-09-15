# JavaScript Libraries for Rust City Builder

This directory contains comprehensive JavaScript libraries that eliminate the need for games to implement their own HTML, CSS, or input handling systems.

## Overview

The library collection includes:
- **Input Manager** - Complete input handling system
- **Rendering Manager** - Canvas rendering and command processing
- **Web Client** - Server communication and connection management  
- **Game Templates** - Ready-to-use full-screen game interfaces

## New Libraries Added

### Input Manager Library (`js/input-manager.js`)

A comprehensive input management system that handles:

- **Keyboard Events**: Key press/release with normalized key mappings
- **Mouse Events**: Click, movement, wheel scrolling with position tracking
- **Touch Events**: Multi-touch support for mobile devices
- **State Management**: Real-time input state tracking
- **Event History**: Debugging support with input event history
- **Pointer Lock**: For first-person games requiring mouse capture

```javascript
// Create input manager targeting canvas
const inputManager = new InputManager(canvasElement, {
    preventDefaultKeyboard: true,
    preventDefaultMouse: false,
    preventDefaultTouch: true
});

// Set up event callbacks
inputManager.onInput('keydown', (event) => {
    console.log('Key pressed:', event.key);
});

// Query input state
if (inputManager.isKeyPressed('W')) {
    movePlayerForward();
}
```

### Full-Screen Game Template (`game-template.html`)

A complete game template featuring:

- **Full-Screen Canvas**: Automatically resizes to viewport (100vw x 100vh)
- **UI Overlay System**: Responsive panels that don't interfere with gameplay
- **Modern Styling**: Glass-morphism UI with backdrop blur effects
- **Debug Panel**: Toggleable debug information display
- **Connection Management**: Integrated server connection handling
- **Mobile Responsive**: Optimized for different screen sizes

## Core Libraries (Existing, Enhanced)

### Rendering Manager (`js/rendering-manager.js`)

Core rendering functionality:
- Handles canvas operations and drawing commands
- Supports grids, sprites, shapes, and other rendering primitives
- Manages command history and statistics
- Provides error handling and validation

### Web Client (`js/web-client.js`)

Client communication and management:
- Manages connection state with the Rust server
- Handles message parsing and event dispatching
- Provides UI integration and status management
- Simulates server communication for demo purposes

## Web Interfaces

### Full-Screen Template (`game-template.html`)
- **Purpose**: Complete game interface with full-screen canvas
- **Features**: Modern UI overlay, input handling, responsive design
- **Use Case**: New games that want a professional, full-screen experience

### Enhanced Original Template (`index.html`)
- **Purpose**: Updated version of the original interface
- **Features**: Full-screen canvas, integrated input manager
- **Use Case**: Existing games transitioning to the new system

## Implementation Benefits

### For Game Developers:

1. **No Input Implementation Required**: Focus on game logic, not input handling
2. **No HTML/CSS Knowledge Needed**: Templates provide complete web interfaces
3. **Cross-Platform Input**: Unified API for keyboard, mouse, and touch
4. **Professional UI**: Modern, responsive design out of the box
5. **Debug Support**: Built-in debugging tools and input history

### For the Rust Backend:

1. **Standardized Interface**: Consistent communication protocol
2. **Reduced Complexity**: Frontend handles all input normalization
3. **Better Testing**: JavaScript libraries can be tested independently
4. **Modular Architecture**: Libraries can be mixed and matched

## Usage Examples

### Basic Game Setup:

```javascript
class MyGame {
    constructor() {
        this.canvas = document.getElementById('gameCanvas');
        this.inputManager = new InputManager(this.canvas);
        this.renderingClient = new WebRenderingClient(this.canvas);
        
        this.setupInput();
        this.inputManager.initialize();
    }
    
    setupInput() {
        this.inputManager.onInput('keydown', (event) => {
            if (event.key === 'Space') {
                this.jump();
            }
        });
        
        this.inputManager.onInput('mousemove', (event) => {
            this.aimAt(event.position);
        });
    }
    
    update() {
        // Check continuous input
        if (this.inputManager.isKeyPressed('W')) {
            this.moveForward();
        }
        
        // Reset deltas each frame
        this.inputManager.resetMouseDelta();
    }
}
```

### Advanced Input Handling:

```javascript
// Multi-key combinations
if (inputManager.areAllKeysPressed(['Shift', 'W'])) {
    this.sprint();
}

// Touch support for mobile
inputManager.onInput('touchstart', (event) => {
    const { touchId, position } = event;
    this.handleTouch(touchId, position);
});

// Mouse wheel for zoom
inputManager.onInput('wheel', (event) => {
    this.adjustZoom(event.delta);
});

// Debugging
const stats = inputManager.getStatistics();
console.log('Active keys:', stats.keysPressed);
console.log('Mouse position:', stats.mousePosition);
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

## Input Event Types Supported

The Input Manager handles all major input types:

### Keyboard Events
- `keydown` / `keyup` with normalized key names (A-Z, 0-9, ArrowUp, Space, etc.)
- Multi-key state tracking
- Key combination detection

### Mouse Events  
- `mousedown` / `mouseup` with button identification (Left, Right, Middle)
- `mousemove` with position and delta tracking
- `wheel` events with scroll delta
- Pointer lock support for FPS games

### Touch Events
- `touchstart` / `touchend` / `touchmove` with multi-touch support
- Touch ID tracking for complex gestures
- Position and delta calculation

## Template Customization

The templates are designed to be easily customizable:

### CSS Variables for Theming:
```css
:root {
    --primary-color: #667eea;
    --background-gradient: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    --ui-background: rgba(0, 0, 0, 0.8);
    --ui-border: rgba(255, 255, 255, 0.2);
}
```

### Responsive Breakpoints:
```css
@media (max-width: 768px) {
    .ui-panel {
        padding: 10px;
        font-size: 14px;
    }
}
```

## Architecture

### Full-Screen Canvas Layout
```html
<!-- Canvas takes full viewport -->
<div id="gameContainer">
    <canvas id="gameCanvas"></canvas>
    
    <!-- UI overlay with selective pointer events -->
    <div id="gameUI">
        <div class="ui-panel">...</div>
    </div>
</div>
```

### Input Manager Integration
```javascript
// Initialize input manager
const inputManager = new InputManager(canvas, options);

// Set up rendering client  
const renderingClient = new WebRenderingClient(canvas);

// Connect input to game logic
inputManager.onInput('keydown', handleKeyPress);
inputManager.onInput('mousedown', handleMouseClick);
```

## Features

### Modular Architecture
- Separate concerns between rendering, input, and communication
- Reusable components that can be integrated into different projects
- Clean API boundaries and interfaces

### Full-Screen Gaming Experience
- Viewport-based sizing (100vw, 100vh)
- Automatic canvas resizing on window resize
- Layered UI with selective pointer events
- Professional gaming aesthetic

### Cross-Platform Input Support
- Unified API for keyboard, mouse, and touch
- Normalized key mappings across browsers
- Mobile-friendly touch handling
- Gamepad support ready for extension

### Real-time Statistics & Debugging
- Input event history tracking
- Performance metrics and FPS counter
- Connection status monitoring
- Debug panel with real-time state display

### Error Handling
- Graceful degradation for unsupported commands
- Connection failure recovery
- Detailed logging and debugging support

### Responsive Design
- Mobile-friendly interface
- Adaptive layout for different screen sizes
- Touch-optimized controls

## Integration with Rust Backend

The JavaScript library works seamlessly with the existing Rust input system:

1. **Input Events**: JavaScript normalizes and sends events to Rust
2. **State Synchronization**: Both systems maintain consistent input state
3. **Command Protocol**: Standardized JSON messages for communication
4. **Error Handling**: Graceful degradation when connection is lost

The integration flow:
1. **Rust Server** - Sends JSON render commands via HTTP
2. **Enhanced HTTP Server** - Serves the JavaScript library files as static assets
3. **JavaScript Client** - Processes commands and renders to canvas
4. **Input Manager** - Captures and normalizes input events
5. **Web Interface** - Provides user interaction and status monitoring

## Usage

### Running the Server

```bash
# Start the enhanced web server with JavaScript library
cargo run
```

### Accessing the Client

Open your browser to:
- `http://localhost:8081` - Enhanced original interface
- `http://localhost:8081/game-template.html` - Full-screen game template

### Testing Features

The web interfaces provide several test buttons:

- **Connect/Disconnect** - Manage server connection
- **Test Grid** - Render a sample grid
- **Test Shape** - Render geometric shapes
- **Clear Canvas** - Clear the rendering surface
- **Fullscreen** - Toggle fullscreen mode
- **Debug** - Toggle debug information panel

## Development

### Adding New Render Commands

1. Add the command definition to `RenderingManager.processRenderCommand()`
2. Implement the rendering logic in the appropriate method
3. Update the Rust backend to send the new command format
4. Test the integration end-to-end

### Adding New Input Types

1. Extend the `InputManager` class with new event handlers
2. Add normalized mappings in the KeyMap/MouseButton constants
3. Update the callback system for new event types
4. Test across different browsers and devices

### Extending the Templates

1. Modify CSS for new UI elements or styling
2. Add JavaScript event handlers for new functionality
3. Update responsive breakpoints as needed
4. Test on mobile and desktop

## Browser Compatibility

The libraries support modern browsers with HTML5 Canvas support:
- Chrome 50+
- Firefox 45+
- Safari 10+
- Edge 79+

## License

This library is part of the Rust City Builder project and follows the same licensing terms.

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