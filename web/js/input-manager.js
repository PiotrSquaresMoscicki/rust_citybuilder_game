/**
 * Rust City Builder - JavaScript Input Manager Library
 * 
 * This library provides a comprehensive input management system for web games,
 * handling keyboard, mouse, and touch events with a clean API that games can
 * use without implementing their own input handling.
 */

/**
 * Input event types that can be handled by the input manager
 */
const InputEventType = {
    KEY_DOWN: 'keydown',
    KEY_UP: 'keyup',
    MOUSE_DOWN: 'mousedown',
    MOUSE_UP: 'mouseup',
    MOUSE_MOVE: 'mousemove',
    MOUSE_WHEEL: 'wheel',
    TOUCH_START: 'touchstart',
    TOUCH_END: 'touchend',
    TOUCH_MOVE: 'touchmove'
};

/**
 * Standard key mappings for consistent key handling
 */
const KeyMap = {
    // Letters
    'KeyA': 'A', 'KeyB': 'B', 'KeyC': 'C', 'KeyD': 'D', 'KeyE': 'E',
    'KeyF': 'F', 'KeyG': 'G', 'KeyH': 'H', 'KeyI': 'I', 'KeyJ': 'J',
    'KeyK': 'K', 'KeyL': 'L', 'KeyM': 'M', 'KeyN': 'N', 'KeyO': 'O',
    'KeyP': 'P', 'KeyQ': 'Q', 'KeyR': 'R', 'KeyS': 'S', 'KeyT': 'T',
    'KeyU': 'U', 'KeyV': 'V', 'KeyW': 'W', 'KeyX': 'X', 'KeyY': 'Y', 'KeyZ': 'Z',
    
    // Numbers
    'Digit0': '0', 'Digit1': '1', 'Digit2': '2', 'Digit3': '3', 'Digit4': '4',
    'Digit5': '5', 'Digit6': '6', 'Digit7': '7', 'Digit8': '8', 'Digit9': '9',
    
    // Arrow keys
    'ArrowUp': 'ArrowUp', 'ArrowDown': 'ArrowDown',
    'ArrowLeft': 'ArrowLeft', 'ArrowRight': 'ArrowRight',
    
    // Special keys
    'Space': 'Space', 'Enter': 'Enter', 'Escape': 'Escape',
    'Tab': 'Tab', 'ShiftLeft': 'Shift', 'ShiftRight': 'Shift',
    'ControlLeft': 'Control', 'ControlRight': 'Control',
    'AltLeft': 'Alt', 'AltRight': 'Alt',
    'Backspace': 'Backspace', 'Delete': 'Delete',
    
    // Function keys
    'F1': 'F1', 'F2': 'F2', 'F3': 'F3', 'F4': 'F4',
    'F5': 'F5', 'F6': 'F6', 'F7': 'F7', 'F8': 'F8',
    'F9': 'F9', 'F10': 'F10', 'F11': 'F11', 'F12': 'F12'
};

/**
 * Mouse button mappings
 */
const MouseButton = {
    0: 'Left',
    1: 'Middle', 
    2: 'Right'
};

/**
 * Main Input Manager class
 * Provides a unified interface for handling all types of input events
 */
class InputManager {
    /**
     * Create a new input manager
     * @param {HTMLElement} targetElement - Element to attach input listeners to (defaults to document)
     * @param {Object} options - Configuration options
     */
    constructor(targetElement = document, options = {}) {
        this.targetElement = targetElement;
        this.options = {
            preventDefaultKeyboard: true,
            preventDefaultMouse: false,
            preventDefaultTouch: true,
            enablePointerLock: false,
            mouseWheelScale: 1.0,
            touchScale: 1.0,
            ...options
        };
        
        // Input state tracking
        this.keyStates = new Map();
        this.mouseButtonStates = new Map();
        this.mousePosition = { x: 0, y: 0 };
        this.mouseDelta = { x: 0, y: 0 };
        this.wheelDelta = 0;
        this.touches = new Map();
        
        // Event listeners and callbacks
        this.eventListeners = new Map();
        this.inputCallbacks = new Map();
        
        // Input history for debugging
        this.inputHistory = [];
        this.maxHistorySize = 100;
        
        // State flags
        this.isInitialized = false;
        this.isEnabled = true;
        this.pointerLocked = false;
        
        this.setupEventListeners();
    }
    
    /**
     * Initialize the input manager
     */
    initialize() {
        if (this.isInitialized) {
            console.warn('InputManager already initialized');
            return;
        }
        
        this.isInitialized = true;
        this.isEnabled = true;
        
        console.log('InputManager initialized successfully');
        console.log('Target element:', this.targetElement === document ? 'document' : this.targetElement);
        console.log('Options:', this.options);
    }
    
    /**
     * Shutdown the input manager and remove all listeners
     */
    shutdown() {
        this.removeAllEventListeners();
        this.clearAllState();
        this.isInitialized = false;
        this.isEnabled = false;
        
        console.log('InputManager shut down');
    }
    
    /**
     * Enable or disable input processing
     * @param {boolean} enabled - Whether to enable input processing
     */
    setEnabled(enabled) {
        this.isEnabled = enabled;
        if (!enabled) {
            this.clearAllState();
        }
    }
    
    /**
     * Check if input processing is enabled
     * @returns {boolean} True if enabled
     */
    isInputEnabled() {
        return this.isInitialized && this.isEnabled;
    }
    
    // ============== EVENT LISTENER SETUP ==============
    
    /**
     * Set up all event listeners
     */
    setupEventListeners() {
        // Keyboard events
        this.addEventListener('keydown', this.handleKeyDown.bind(this));
        this.addEventListener('keyup', this.handleKeyUp.bind(this));
        
        // Mouse events
        this.addEventListener('mousedown', this.handleMouseDown.bind(this));
        this.addEventListener('mouseup', this.handleMouseUp.bind(this));
        this.addEventListener('mousemove', this.handleMouseMove.bind(this));
        this.addEventListener('wheel', this.handleMouseWheel.bind(this));
        this.addEventListener('contextmenu', this.handleContextMenu.bind(this));
        
        // Touch events
        this.addEventListener('touchstart', this.handleTouchStart.bind(this));
        this.addEventListener('touchend', this.handleTouchEnd.bind(this));
        this.addEventListener('touchmove', this.handleTouchMove.bind(this));
        
        // Pointer lock events
        document.addEventListener('pointerlockchange', this.handlePointerLockChange.bind(this));
        document.addEventListener('pointerlockerror', this.handlePointerLockError.bind(this));
        
        // Focus events to clear state when losing focus
        if (this.targetElement !== document) {
            this.addEventListener('blur', this.handleBlur.bind(this));
        } else {
            window.addEventListener('blur', this.handleBlur.bind(this));
        }
    }
    
    /**
     * Add an event listener to the target element
     * @param {string} eventType - Type of event to listen for
     * @param {Function} handler - Event handler function
     */
    addEventListener(eventType, handler) {
        this.targetElement.addEventListener(eventType, handler);
        
        // Store reference for removal later
        if (!this.eventListeners.has(eventType)) {
            this.eventListeners.set(eventType, []);
        }
        this.eventListeners.get(eventType).push(handler);
    }
    
    /**
     * Remove all event listeners
     */
    removeAllEventListeners() {
        for (const [eventType, handlers] of this.eventListeners) {
            for (const handler of handlers) {
                this.targetElement.removeEventListener(eventType, handler);
            }
        }
        this.eventListeners.clear();
        
        document.removeEventListener('pointerlockchange', this.handlePointerLockChange.bind(this));
        document.removeEventListener('pointerlockerror', this.handlePointerLockError.bind(this));
    }
    
    // ============== EVENT HANDLERS ==============
    
    /**
     * Handle keydown events
     * @param {KeyboardEvent} event - Keyboard event
     */
    handleKeyDown(event) {
        if (!this.isInputEnabled()) return;
        
        const key = this.normalizeKey(event.code || event.key);
        
        if (this.options.preventDefaultKeyboard) {
            event.preventDefault();
        }
        
        // Only register if not already pressed (prevent key repeat)
        if (!this.keyStates.get(key)) {
            this.keyStates.set(key, true);
            this.addToHistory('key_down', { key, timestamp: Date.now() });
            this.triggerCallback('keydown', { key, originalEvent: event });
        }
    }
    
    /**
     * Handle keyup events
     * @param {KeyboardEvent} event - Keyboard event
     */
    handleKeyUp(event) {
        if (!this.isInputEnabled()) return;
        
        const key = this.normalizeKey(event.code || event.key);
        
        if (this.options.preventDefaultKeyboard) {
            event.preventDefault();
        }
        
        this.keyStates.set(key, false);
        this.addToHistory('key_up', { key, timestamp: Date.now() });
        this.triggerCallback('keyup', { key, originalEvent: event });
    }
    
    /**
     * Handle mousedown events
     * @param {MouseEvent} event - Mouse event
     */
    handleMouseDown(event) {
        if (!this.isInputEnabled()) return;
        
        const button = MouseButton[event.button] || `Button${event.button}`;
        const position = this.getMousePosition(event);
        
        if (this.options.preventDefaultMouse) {
            event.preventDefault();
        }
        
        this.mouseButtonStates.set(button, true);
        this.mousePosition = position;
        
        this.addToHistory('mouse_down', { button, position, timestamp: Date.now() });
        this.triggerCallback('mousedown', { button, position, originalEvent: event });
    }
    
    /**
     * Handle mouseup events
     * @param {MouseEvent} event - Mouse event
     */
    handleMouseUp(event) {
        if (!this.isInputEnabled()) return;
        
        const button = MouseButton[event.button] || `Button${event.button}`;
        const position = this.getMousePosition(event);
        
        if (this.options.preventDefaultMouse) {
            event.preventDefault();
        }
        
        this.mouseButtonStates.set(button, false);
        this.mousePosition = position;
        
        this.addToHistory('mouse_up', { button, position, timestamp: Date.now() });
        this.triggerCallback('mouseup', { button, position, originalEvent: event });
    }
    
    /**
     * Handle mousemove events
     * @param {MouseEvent} event - Mouse event
     */
    handleMouseMove(event) {
        if (!this.isInputEnabled()) return;
        
        const position = this.getMousePosition(event);
        const delta = {
            x: event.movementX || (position.x - this.mousePosition.x),
            y: event.movementY || (position.y - this.mousePosition.y)
        };
        
        this.mouseDelta = delta;
        this.mousePosition = position;
        
        this.triggerCallback('mousemove', { position, delta, originalEvent: event });
    }
    
    /**
     * Handle wheel events
     * @param {WheelEvent} event - Wheel event
     */
    handleMouseWheel(event) {
        if (!this.isInputEnabled()) return;
        
        const delta = -event.deltaY * this.options.mouseWheelScale;
        const position = this.getMousePosition(event);
        
        event.preventDefault();
        
        this.wheelDelta = delta;
        this.mousePosition = position;
        
        this.addToHistory('mouse_wheel', { delta, position, timestamp: Date.now() });
        this.triggerCallback('wheel', { delta, position, originalEvent: event });
    }
    
    /**
     * Handle context menu events (right-click menu)
     * @param {MouseEvent} event - Context menu event
     */
    handleContextMenu(event) {
        if (this.options.preventDefaultMouse) {
            event.preventDefault();
        }
    }
    
    /**
     * Handle touch start events
     * @param {TouchEvent} event - Touch event
     */
    handleTouchStart(event) {
        if (!this.isInputEnabled()) return;
        
        if (this.options.preventDefaultTouch) {
            event.preventDefault();
        }
        
        for (const touch of event.changedTouches) {
            const position = this.getTouchPosition(touch);
            this.touches.set(touch.identifier, {
                position,
                startPosition: position,
                startTime: Date.now()
            });
            
            this.addToHistory('touch_start', { 
                touchId: touch.identifier, 
                position, 
                timestamp: Date.now() 
            });
            this.triggerCallback('touchstart', { 
                touchId: touch.identifier, 
                position, 
                originalEvent: event 
            });
        }
    }
    
    /**
     * Handle touch end events
     * @param {TouchEvent} event - Touch event
     */
    handleTouchEnd(event) {
        if (!this.isInputEnabled()) return;
        
        if (this.options.preventDefaultTouch) {
            event.preventDefault();
        }
        
        for (const touch of event.changedTouches) {
            const touchData = this.touches.get(touch.identifier);
            const position = this.getTouchPosition(touch);
            
            if (touchData) {
                this.touches.delete(touch.identifier);
                
                this.addToHistory('touch_end', { 
                    touchId: touch.identifier, 
                    position, 
                    duration: Date.now() - touchData.startTime,
                    timestamp: Date.now() 
                });
                this.triggerCallback('touchend', { 
                    touchId: touch.identifier, 
                    position, 
                    startPosition: touchData.startPosition,
                    duration: Date.now() - touchData.startTime,
                    originalEvent: event 
                });
            }
        }
    }
    
    /**
     * Handle touch move events
     * @param {TouchEvent} event - Touch event
     */
    handleTouchMove(event) {
        if (!this.isInputEnabled()) return;
        
        if (this.options.preventDefaultTouch) {
            event.preventDefault();
        }
        
        for (const touch of event.changedTouches) {
            const touchData = this.touches.get(touch.identifier);
            const position = this.getTouchPosition(touch);
            
            if (touchData) {
                const delta = {
                    x: position.x - touchData.position.x,
                    y: position.y - touchData.position.y
                };
                
                touchData.position = position;
                
                this.triggerCallback('touchmove', { 
                    touchId: touch.identifier, 
                    position, 
                    delta,
                    startPosition: touchData.startPosition,
                    originalEvent: event 
                });
            }
        }
    }
    
    /**
     * Handle blur events (clear input state when losing focus)
     * @param {FocusEvent} event - Focus event
     */
    handleBlur(event) {
        this.clearAllState();
        this.triggerCallback('blur', { originalEvent: event });
    }
    
    /**
     * Handle pointer lock change events
     */
    handlePointerLockChange() {
        this.pointerLocked = document.pointerLockElement === this.targetElement;
        this.triggerCallback('pointerlockchange', { locked: this.pointerLocked });
    }
    
    /**
     * Handle pointer lock error events
     */
    handlePointerLockError() {
        console.error('Pointer lock failed');
        this.triggerCallback('pointerlockerror', {});
    }
    
    // ============== INPUT STATE QUERIES ==============
    
    /**
     * Check if a key is currently pressed
     * @param {string} key - Key to check (normalized key name)
     * @returns {boolean} True if the key is pressed
     */
    isKeyPressed(key) {
        return this.keyStates.get(this.normalizeKey(key)) || false;
    }
    
    /**
     * Check if any of the specified keys are pressed
     * @param {string[]} keys - Array of keys to check
     * @returns {boolean} True if any key is pressed
     */
    isAnyKeyPressed(keys) {
        return keys.some(key => this.isKeyPressed(key));
    }
    
    /**
     * Check if all of the specified keys are pressed
     * @param {string[]} keys - Array of keys to check
     * @returns {boolean} True if all keys are pressed
     */
    areAllKeysPressed(keys) {
        return keys.every(key => this.isKeyPressed(key));
    }
    
    /**
     * Check if a mouse button is currently pressed
     * @param {string} button - Button to check ('Left', 'Right', 'Middle')
     * @returns {boolean} True if the button is pressed
     */
    isMouseButtonPressed(button) {
        return this.mouseButtonStates.get(button) || false;
    }
    
    /**
     * Get the current mouse position
     * @returns {Object} Object with x and y coordinates
     */
    getMousePosition() {
        return { ...this.mousePosition };
    }
    
    /**
     * Get the mouse movement delta since last frame
     * @returns {Object} Object with x and y delta values
     */
    getMouseDelta() {
        return { ...this.mouseDelta };
    }
    
    /**
     * Get the mouse wheel delta
     * @returns {number} Wheel delta value
     */
    getWheelDelta() {
        return this.wheelDelta;
    }
    
    /**
     * Get information about current touches
     * @returns {Map} Map of touch IDs to touch data
     */
    getCurrentTouches() {
        return new Map(this.touches);
    }
    
    /**
     * Check if there are any active touches
     * @returns {boolean} True if there are active touches
     */
    hasTouches() {
        return this.touches.size > 0;
    }
    
    // ============== CALLBACK SYSTEM ==============
    
    /**
     * Register a callback for input events
     * @param {string} eventType - Type of event ('keydown', 'keyup', 'mousedown', etc.)
     * @param {Function} callback - Callback function to register
     */
    onInput(eventType, callback) {
        if (!this.inputCallbacks.has(eventType)) {
            this.inputCallbacks.set(eventType, []);
        }
        this.inputCallbacks.get(eventType).push(callback);
    }
    
    /**
     * Remove a callback for input events
     * @param {string} eventType - Type of event
     * @param {Function} callback - Callback function to remove
     */
    offInput(eventType, callback) {
        if (this.inputCallbacks.has(eventType)) {
            const callbacks = this.inputCallbacks.get(eventType);
            const index = callbacks.indexOf(callback);
            if (index > -1) {
                callbacks.splice(index, 1);
            }
        }
    }
    
    /**
     * Trigger callbacks for an event type
     * @param {string} eventType - Type of event
     * @param {Object} eventData - Event data to pass to callbacks
     */
    triggerCallback(eventType, eventData) {
        if (this.inputCallbacks.has(eventType)) {
            for (const callback of this.inputCallbacks.get(eventType)) {
                try {
                    callback(eventData);
                } catch (error) {
                    console.error(`Error in input callback for ${eventType}:`, error);
                }
            }
        }
    }
    
    // ============== UTILITY METHODS ==============
    
    /**
     * Normalize a key name for consistent handling
     * @param {string} key - Raw key value
     * @returns {string} Normalized key name
     */
    normalizeKey(key) {
        return KeyMap[key] || key;
    }
    
    /**
     * Get mouse position relative to the target element
     * @param {MouseEvent} event - Mouse event
     * @returns {Object} Position object with x and y coordinates
     */
    getMousePosition(event) {
        if (this.targetElement === document) {
            return { x: event.clientX, y: event.clientY };
        } else {
            const rect = this.targetElement.getBoundingClientRect();
            return {
                x: event.clientX - rect.left,
                y: event.clientY - rect.top
            };
        }
    }
    
    /**
     * Get touch position relative to the target element
     * @param {Touch} touch - Touch object
     * @returns {Object} Position object with x and y coordinates
     */
    getTouchPosition(touch) {
        if (this.targetElement === document) {
            return { x: touch.clientX, y: touch.clientY };
        } else {
            const rect = this.targetElement.getBoundingClientRect();
            return {
                x: (touch.clientX - rect.left) * this.options.touchScale,
                y: (touch.clientY - rect.top) * this.options.touchScale
            };
        }
    }
    
    /**
     * Clear all input state
     */
    clearAllState() {
        this.keyStates.clear();
        this.mouseButtonStates.clear();
        this.touches.clear();
        this.mouseDelta = { x: 0, y: 0 };
        this.wheelDelta = 0;
    }
    
    /**
     * Add an event to the input history for debugging
     * @param {string} eventType - Type of event
     * @param {Object} eventData - Event data
     */
    addToHistory(eventType, eventData) {
        this.inputHistory.push({ type: eventType, data: eventData });
        
        // Keep history size manageable
        if (this.inputHistory.length > this.maxHistorySize) {
            this.inputHistory.shift();
        }
    }
    
    /**
     * Get the input history for debugging
     * @returns {Array} Array of historical input events
     */
    getInputHistory() {
        return [...this.inputHistory];
    }
    
    /**
     * Clear the input history
     */
    clearInputHistory() {
        this.inputHistory = [];
    }
    
    /**
     * Request pointer lock on the target element
     */
    requestPointerLock() {
        if (this.targetElement !== document && this.targetElement.requestPointerLock) {
            this.targetElement.requestPointerLock();
        }
    }
    
    /**
     * Exit pointer lock
     */
    exitPointerLock() {
        if (document.exitPointerLock) {
            document.exitPointerLock();
        }
    }
    
    /**
     * Check if pointer is currently locked
     * @returns {boolean} True if pointer is locked
     */
    isPointerLocked() {
        return this.pointerLocked;
    }
    
    /**
     * Reset mouse delta (call this each frame after reading delta)
     */
    resetMouseDelta() {
        this.mouseDelta = { x: 0, y: 0 };
    }
    
    /**
     * Reset wheel delta (call this each frame after reading delta)
     */
    resetWheelDelta() {
        this.wheelDelta = 0;
    }
    
    /**
     * Get statistics about input manager state
     * @returns {Object} Statistics object
     */
    getStatistics() {
        return {
            initialized: this.isInitialized,
            enabled: this.isEnabled,
            keysPressed: Array.from(this.keyStates.entries()).filter(([_, pressed]) => pressed).map(([key, _]) => key),
            mouseButtonsPressed: Array.from(this.mouseButtonStates.entries()).filter(([_, pressed]) => pressed).map(([button, _]) => button),
            mousePosition: this.getMousePosition(),
            activeTouches: this.touches.size,
            historySize: this.inputHistory.length,
            pointerLocked: this.pointerLocked,
            callbacksRegistered: Array.from(this.inputCallbacks.entries()).map(([type, callbacks]) => ({ type, count: callbacks.length }))
        };
    }
}

// Export for use as ES6 module or global
if (typeof module !== 'undefined' && module.exports) {
    module.exports = { InputManager, InputEventType, KeyMap, MouseButton };
} else if (typeof window !== 'undefined') {
    window.InputManager = InputManager;
    window.InputEventType = InputEventType;
    window.KeyMap = KeyMap;
    window.MouseButton = MouseButton;
}