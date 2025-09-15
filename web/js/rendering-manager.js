/**
 * Rust City Builder - JavaScript Rendering Manager Library
 * 
 * This library provides a JavaScript interface for rendering commands sent from
 * the Rust backend. It handles canvas operations, command parsing, and visual
 * feedback for the game rendering system.
 */

/**
 * Main rendering manager class that handles canvas rendering operations
 */
class RenderingManager {
    /**
     * Create a new rendering manager
     * @param {HTMLCanvasElement} canvas - The canvas element to render to
     */
    constructor(canvas) {
        if (!canvas || !(canvas instanceof HTMLCanvasElement)) {
            throw new Error('RenderingManager requires a valid canvas element');
        }
        
        this.canvas = canvas;
        this.ctx = canvas.getContext('2d');
        this.commandHistory = [];
        this.isReady = true;
        
        // Initialize with a clean canvas
        this.clear();
    }
    
    /**
     * Check if the rendering manager is ready to process commands
     * @returns {boolean} True if ready
     */
    isRenderingReady() {
        return this.isReady && this.canvas && this.ctx;
    }
    
    /**
     * Clear the canvas with optional background color
     * @param {number} r - Red component (0-1)
     * @param {number} g - Green component (0-1) 
     * @param {number} b - Blue component (0-1)
     * @param {number} a - Alpha component (0-1)
     */
    clear(r = 1, g = 1, b = 1, a = 1) {
        if (!this.isRenderingReady()) return;
        
        this.ctx.save();
        this.ctx.globalCompositeOperation = 'source-over';
        this.ctx.fillStyle = `rgba(${r * 255}, ${g * 255}, ${b * 255}, ${a})`;
        this.ctx.fillRect(0, 0, this.canvas.width, this.canvas.height);
        this.ctx.restore();
    }
    
    /**
     * Draw a grid with specified parameters
     * @param {Object} params - Grid parameters
     * @param {number} params.width - Number of grid columns
     * @param {number} params.height - Number of grid rows
     * @param {number} params.cellSize - Size of each cell in pixels
     * @param {Array} params.lineColor - Line color as [r, g, b, a] (0-1)
     * @param {Array} params.backgroundColor - Background color as [r, g, b, a] (0-1)
     */
    drawGrid(params) {
        if (!this.isRenderingReady()) return;
        
        const { width, height, cellSize, lineColor, backgroundColor } = params;
        
        // Clear and set background
        this.clear(backgroundColor[0], backgroundColor[1], backgroundColor[2], backgroundColor[3]);
        
        // Set line style
        this.ctx.strokeStyle = `rgba(${lineColor[0] * 255}, ${lineColor[1] * 255}, ${lineColor[2] * 255}, ${lineColor[3]})`;
        this.ctx.lineWidth = 1;
        
        this.ctx.beginPath();
        
        // Draw vertical lines
        for (let x = 0; x <= width; x++) {
            const xPos = x * cellSize;
            this.ctx.moveTo(xPos, 0);
            this.ctx.lineTo(xPos, height * cellSize);
        }
        
        // Draw horizontal lines
        for (let y = 0; y <= height; y++) {
            const yPos = y * cellSize;
            this.ctx.moveTo(0, yPos);
            this.ctx.lineTo(width * cellSize, yPos);
        }
        
        this.ctx.stroke();
    }
    
    /**
     * Draw a sprite with transform, size, color, and UV coordinates
     * @param {Object} params - Sprite parameters
     */
    drawSprite(params) {
        if (!this.isRenderingReady()) return;
        
        const { textureId, transform, size, color, zOrder, uvRect } = params;
        
        this.ctx.save();
        
        // Apply transform matrix (assuming 2D affine transform)
        if (transform && transform.length >= 6) {
            this.ctx.setTransform(transform[0], transform[1], transform[2], transform[3], transform[4], transform[5]);
        }
        
        // Set color/tint
        if (color) {
            this.ctx.fillStyle = `rgba(${color[0] * 255}, ${color[1] * 255}, ${color[2] * 255}, ${color[3] || 1})`;
        }
        
        // For now, draw a colored rectangle representing the sprite
        // In a full implementation, this would load and draw the actual texture
        const width = size ? size[0] : 32;
        const height = size ? size[1] : 32;
        
        this.ctx.fillRect(-width/2, -height/2, width, height);
        
        // Draw texture ID as text for debugging
        this.ctx.fillStyle = 'black';
        this.ctx.font = '12px monospace';
        this.ctx.textAlign = 'center';
        this.ctx.fillText(textureId || 'sprite', 0, 0);
        
        this.ctx.restore();
    }
    
    /**
     * Draw a shape (circle, rectangle, triangle, etc.)
     * @param {Object} params - Shape parameters
     */
    drawShape(params) {
        if (!this.isRenderingReady()) return;
        
        const { shapeType, transform, fill, stroke, zOrder } = params;
        
        this.ctx.save();
        
        // Apply transform
        if (transform && transform.length >= 6) {
            this.ctx.setTransform(transform[0], transform[1], transform[2], transform[3], transform[4], transform[5]);
        }
        
        // Begin path for the shape
        this.ctx.beginPath();
        
        // Draw based on shape type
        if (shapeType.type === 'Circle') {
            this.ctx.arc(0, 0, shapeType.radius, 0, 2 * Math.PI);
        } else if (shapeType.type === 'Rectangle') {
            const w = shapeType.width;
            const h = shapeType.height;
            this.ctx.rect(-w/2, -h/2, w, h);
        } else if (shapeType.type === 'Triangle') {
            const vertices = shapeType.vertices;
            if (vertices && vertices.length >= 3) {
                this.ctx.moveTo(vertices[0][0], vertices[0][1]);
                this.ctx.lineTo(vertices[1][0], vertices[1][1]);
                this.ctx.lineTo(vertices[2][0], vertices[2][1]);
                this.ctx.closePath();
            }
        } else if (shapeType.type === 'Line') {
            const start = shapeType.start;
            const end = shapeType.end;
            if (start && end) {
                this.ctx.moveTo(start[0], start[1]);
                this.ctx.lineTo(end[0], end[1]);
                this.ctx.lineWidth = shapeType.thickness || 1;
            }
        } else if (shapeType.type === 'Polygon') {
            const vertices = shapeType.vertices;
            if (vertices && vertices.length > 0) {
                this.ctx.moveTo(vertices[0][0], vertices[0][1]);
                for (let i = 1; i < vertices.length; i++) {
                    this.ctx.lineTo(vertices[i][0], vertices[i][1]);
                }
                this.ctx.closePath();
            }
        }
        
        // Apply fill
        if (fill && fill.type === 'Solid') {
            const color = fill.color;
            this.ctx.fillStyle = `rgba(${color[0] * 255}, ${color[1] * 255}, ${color[2] * 255}, ${color[3] || 1})`;
            this.ctx.fill();
        }
        
        // Apply stroke
        if (stroke) {
            const color = stroke.color;
            this.ctx.strokeStyle = `rgba(${color[0] * 255}, ${color[1] * 255}, ${color[2] * 255}, ${color[3] || 1})`;
            this.ctx.lineWidth = stroke.width || 1;
            this.ctx.stroke();
        }
        
        this.ctx.restore();
    }
    
    /**
     * Process a render command from the server
     * @param {Object} command - The render command object
     */
    processRenderCommand(command) {
        if (!command || !command.type) {
            console.warn('Invalid render command:', command);
            return;
        }
        
        // Store command in history
        this.commandHistory.push({
            timestamp: Date.now(),
            command: command
        });
        
        // Keep only last 100 commands
        if (this.commandHistory.length > 100) {
            this.commandHistory.shift();
        }
        
        const { type, params } = command;
        
        try {
            switch (type) {
                case 'Clear':
                    this.clear(params.r, params.g, params.b, params.a);
                    break;
                
                case 'DrawGrid':
                    this.drawGrid(params);
                    break;
                
                case 'DrawSprite':
                    this.drawSprite(params);
                    break;
                
                case 'DrawShape':
                    this.drawShape(params);
                    break;
                
                default:
                    console.warn(`Unknown render command type: ${type}`);
            }
        } catch (error) {
            console.error(`Error processing render command ${type}:`, error);
        }
    }
    
    /**
     * Get the command history
     * @returns {Array} Array of recent commands
     */
    getCommandHistory() {
        return [...this.commandHistory];
    }
    
    /**
     * Clear the command history
     */
    clearCommandHistory() {
        this.commandHistory = [];
    }
    
    /**
     * Resize the canvas to fit its container
     * @param {number} width - New width (optional)
     * @param {number} height - New height (optional)
     */
    resize(width = null, height = null) {
        if (width !== null && height !== null) {
            this.canvas.width = width;
            this.canvas.height = height;
        } else {
            // Auto-resize to container
            const rect = this.canvas.getBoundingClientRect();
            this.canvas.width = rect.width;
            this.canvas.height = rect.height;
        }
    }
    
    /**
     * Get canvas dimensions
     * @returns {Object} Object with width and height properties
     */
    getDimensions() {
        return {
            width: this.canvas.width,
            height: this.canvas.height
        };
    }
}

// Export for use as ES6 module or global
if (typeof module !== 'undefined' && module.exports) {
    module.exports = RenderingManager;
} else if (typeof window !== 'undefined') {
    window.RenderingManager = RenderingManager;
}