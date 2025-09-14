use std::any::Any;
use std::time::{Duration, Instant};
use crate::ecs::Component;
use serde::{Deserialize, Serialize};

/// Time component that stores delta time information for systems
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimeComponent {
    /// Delta time since last frame in seconds
    pub delta_time: f64,
    /// Total elapsed time since the start in seconds
    pub total_time: f64,
    /// Frame count since the start
    pub frame_count: u64,
    /// Time scale factor (1.0 = normal time, 0.5 = half speed, 2.0 = double speed)
    pub time_scale: f64,
    /// Whether time is paused
    pub is_paused: bool,
}

impl TimeComponent {
    /// Create a new time component with default values
    pub fn new() -> Self {
        Self {
            delta_time: 0.0,
            total_time: 0.0,
            frame_count: 0,
            time_scale: 1.0,
            is_paused: false,
        }
    }

    /// Get the scaled delta time (delta_time * time_scale)
    pub fn scaled_delta_time(&self) -> f64 {
        if self.is_paused {
            0.0
        } else {
            self.delta_time * self.time_scale
        }
    }

    /// Set the time scale factor
    pub fn set_time_scale(&mut self, scale: f64) {
        self.time_scale = scale.max(0.0); // Ensure non-negative
    }

    /// Pause the time
    pub fn pause(&mut self) {
        self.is_paused = true;
    }

    /// Resume the time
    pub fn resume(&mut self) {
        self.is_paused = false;
    }

    /// Toggle pause state
    pub fn toggle_pause(&mut self) {
        self.is_paused = !self.is_paused;
    }

    /// Update time component with new delta time
    pub fn update(&mut self, delta_time: f64) {
        self.delta_time = delta_time;
        if !self.is_paused {
            self.total_time += delta_time * self.time_scale;
        }
        self.frame_count += 1;
    }

    /// Get frames per second based on current delta time
    pub fn fps(&self) -> f64 {
        if self.delta_time > 0.0 {
            1.0 / self.delta_time
        } else {
            0.0
        }
    }
}

impl Default for TimeComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for TimeComponent {
    fn validate(&self) -> bool {
        self.delta_time >= 0.0 && 
        self.total_time >= 0.0 && 
        self.time_scale >= 0.0
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Component> {
        Box::new(self.clone())
    }
}

/// Time manager - a global service that provides current time information
pub struct TimeManager {
    start_time: Instant,
    last_frame_time: Instant,
    current_delta_time: Duration,
}

impl TimeManager {
    /// Create a new time manager
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            start_time: now,
            last_frame_time: now,
            current_delta_time: Duration::ZERO,
        }
    }

    /// Update the time manager - should be called once per frame
    pub fn update(&mut self) {
        let now = Instant::now();
        self.current_delta_time = now.duration_since(self.last_frame_time);
        self.last_frame_time = now;
    }

    /// Get the delta time since last frame in seconds
    pub fn delta_time_seconds(&self) -> f64 {
        self.current_delta_time.as_secs_f64()
    }

    /// Get the total elapsed time since start in seconds
    pub fn total_time_seconds(&self) -> f64 {
        self.start_time.elapsed().as_secs_f64()
    }

    /// Get the current delta time as Duration
    pub fn delta_time(&self) -> Duration {
        self.current_delta_time
    }

    /// Get the total elapsed time as Duration
    pub fn total_time(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Reset the time manager (useful for testing or restarting)
    pub fn reset(&mut self) {
        let now = Instant::now();
        self.start_time = now;
        self.last_frame_time = now;
        self.current_delta_time = Duration::ZERO;
    }
}

impl Default for TimeManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Global time manager instance
static mut GLOBAL_TIME_MANAGER: Option<TimeManager> = None;
static mut TIME_MANAGER_INITIALIZED: bool = false;

/// Initialize the global time manager
pub fn initialize_time_manager() {
    unsafe {
        GLOBAL_TIME_MANAGER = Some(TimeManager::new());
        TIME_MANAGER_INITIALIZED = true;
    }
}

/// Get a reference to the global time manager
pub fn get_time_manager() -> Option<&'static TimeManager> {
    unsafe {
        GLOBAL_TIME_MANAGER.as_ref()
    }
}

/// Get a mutable reference to the global time manager
pub fn get_time_manager_mut() -> Option<&'static mut TimeManager> {
    unsafe {
        GLOBAL_TIME_MANAGER.as_mut()
    }
}

/// Check if the time manager is initialized
pub fn is_time_manager_initialized() -> bool {
    unsafe {
        TIME_MANAGER_INITIALIZED
    }
}

/// Update the global time manager - should be called once per frame
pub fn update_global_time_manager() {
    if let Some(manager) = get_time_manager_mut() {
        manager.update();
    }
}

// Make TimeComponent diffable
// Temporarily disabled diffable macro
// crate::diffable!(TimeComponent { delta_time, total_time, frame_count, time_scale, is_paused });

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn test_time_component_creation() {
        let time_comp = TimeComponent::new();
        assert_eq!(time_comp.delta_time, 0.0);
        assert_eq!(time_comp.total_time, 0.0);
        assert_eq!(time_comp.frame_count, 0);
        assert_eq!(time_comp.time_scale, 1.0);
        assert!(!time_comp.is_paused);
        assert!(time_comp.validate());
    }

    #[test]
    fn test_time_component_update() {
        let mut time_comp = TimeComponent::new();
        time_comp.update(0.016); // ~60 FPS

        assert_eq!(time_comp.delta_time, 0.016);
        assert_eq!(time_comp.total_time, 0.016);
        assert_eq!(time_comp.frame_count, 1);
    }

    #[test]
    fn test_time_component_pause() {
        let mut time_comp = TimeComponent::new();
        time_comp.pause();
        time_comp.update(0.016);

        assert_eq!(time_comp.delta_time, 0.016);
        assert_eq!(time_comp.total_time, 0.0); // Should not advance when paused
        assert_eq!(time_comp.frame_count, 1);
        assert_eq!(time_comp.scaled_delta_time(), 0.0);
    }

    #[test]
    fn test_time_component_time_scale() {
        let mut time_comp = TimeComponent::new();
        time_comp.set_time_scale(2.0);
        time_comp.update(0.016);

        assert_eq!(time_comp.delta_time, 0.016);
        assert_eq!(time_comp.total_time, 0.032); // 2x speed
        assert_eq!(time_comp.scaled_delta_time(), 0.032);
    }

    #[test]
    fn test_time_component_fps() {
        let mut time_comp = TimeComponent::new();
        time_comp.update(0.016); // ~60 FPS

        let fps = time_comp.fps();
        assert!((fps - 62.5).abs() < 0.1); // 1.0 / 0.016 ≈ 62.5
    }

    #[test]
    fn test_time_manager_creation() {
        let manager = TimeManager::new();
        assert_eq!(manager.delta_time_seconds(), 0.0);
        assert!(manager.total_time_seconds() >= 0.0);
    }

    #[test]
    fn test_time_manager_update() {
        let mut manager = TimeManager::new();
        sleep(Duration::from_millis(1)); // Small delay
        manager.update();
        
        assert!(manager.delta_time_seconds() > 0.0);
        assert!(manager.total_time_seconds() > 0.0);
    }

    #[test]
    fn test_time_manager_reset() {
        let mut manager = TimeManager::new();
        sleep(Duration::from_millis(1));
        manager.update();
        
        let old_total = manager.total_time_seconds();
        manager.reset();
        
        assert!(manager.total_time_seconds() < old_total);
        assert_eq!(manager.delta_time_seconds(), 0.0);
    }

    #[test]
    fn test_global_time_manager() {
        initialize_time_manager();
        assert!(is_time_manager_initialized());
        
        update_global_time_manager();
        
        if let Some(manager) = get_time_manager() {
            assert!(manager.total_time_seconds() >= 0.0);
        } else {
            panic!("Global time manager should be initialized");
        }
    }
}