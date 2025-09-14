use std::any::Any;
use crate::ecs::Component;
use super::{vector2d::Vector2d, angle2d::Angle2d, transform2d::Transform2d};

/// Camera2d component that defines the view transformation for 2D rendering
#[derive(Debug, Clone, PartialEq)]
pub struct Camera2d {
    /// Position of the camera in world space
    position: Vector2d,
    /// Rotation of the camera
    rotation: Angle2d,
    /// Scale/zoom of the camera (higher values = zoomed in)
    scale: f32,
    /// View bounds for culling (in camera space)
    view_width: f32,
    view_height: f32,
}

impl Camera2d {
    /// Creates a new Camera2d with default values
    pub fn new() -> Self {
        Self {
            position: Vector2d::zero(),
            rotation: Angle2d::zero(),
            scale: 1.0,
            view_width: 1920.0,  // Default screen width
            view_height: 1080.0, // Default screen height
        }
    }

    /// Creates a Camera2d with specific position, rotation, and scale
    pub fn from_prs(position: Vector2d, rotation: Angle2d, scale: f32) -> Self {
        Self {
            position,
            rotation,
            scale,
            view_width: 1920.0,
            view_height: 1080.0,
        }
    }

    /// Gets the camera position
    pub fn position(&self) -> Vector2d {
        self.position
    }

    /// Sets the camera position
    pub fn set_position(&mut self, position: Vector2d) {
        self.position = position;
    }

    /// Gets the camera rotation
    pub fn rotation(&self) -> Angle2d {
        self.rotation
    }

    /// Sets the camera rotation
    pub fn set_rotation(&mut self, rotation: Angle2d) {
        self.rotation = rotation;
    }

    /// Gets the camera scale/zoom
    pub fn scale(&self) -> f32 {
        self.scale
    }

    /// Sets the camera scale/zoom
    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale.max(0.001); // Prevent zero or negative scale
    }

    /// Gets the view dimensions
    pub fn view_dimensions(&self) -> (f32, f32) {
        (self.view_width, self.view_height)
    }

    /// Sets the view dimensions for culling calculations
    pub fn set_view_dimensions(&mut self, width: f32, height: f32) {
        self.view_width = width;
        self.view_height = height;
    }

    /// Moves the camera by the given offset
    pub fn translate(&mut self, offset: Vector2d) {
        self.position = self.position + offset;
    }

    /// Rotates the camera by the given angle
    pub fn rotate(&mut self, angle: Angle2d) {
        self.rotation = self.rotation + angle;
    }

    /// Zooms the camera by the given factor
    pub fn zoom(&mut self, factor: f32) {
        self.scale *= factor;
        self.scale = self.scale.max(0.001);
    }

    /// Gets the view transform matrix (world to camera space)
    pub fn view_transform(&self) -> Transform2d {
        // Create inverse transform: translate to origin, then inverse rotate, then inverse scale
        let translation_to_origin = Transform2d::translation(-self.position);
        let inverse_rotation = Transform2d::rotation(Angle2d::from_radians(-self.rotation.radians()));
        let inverse_scale = Transform2d::scale(1.0 / self.scale);
        
        // Apply transformations in order: first translate, then rotate, then scale
        inverse_scale * inverse_rotation * translation_to_origin
    }

    /// Transforms a world point to camera space
    pub fn world_to_camera(&self, world_point: Vector2d) -> Vector2d {
        self.view_transform().transform_point(world_point)
    }

    /// Transforms a camera point to world space
    pub fn camera_to_world(&self, camera_point: Vector2d) -> Vector2d {
        let world_transform = Transform2d::from_trs(self.position, self.rotation, self.scale);
        world_transform.transform_point(camera_point)
    }

    /// Checks if a point is visible in the camera view
    pub fn is_point_visible(&self, world_point: Vector2d) -> bool {
        let camera_point = self.world_to_camera(world_point);
        let half_width = self.view_width * 0.5 / self.scale;
        let half_height = self.view_height * 0.5 / self.scale;
        
        camera_point.x >= -half_width && camera_point.x <= half_width &&
        camera_point.y >= -half_height && camera_point.y <= half_height
    }

    /// Checks if a circular area is visible in the camera view
    pub fn is_circle_visible(&self, center: Vector2d, radius: f32) -> bool {
        let camera_center = self.world_to_camera(center);
        let scaled_radius = radius / self.scale;
        let half_width = self.view_width * 0.5 / self.scale;
        let half_height = self.view_height * 0.5 / self.scale;
        
        // Check if circle intersects with view rectangle
        let closest_x = camera_center.x.clamp(-half_width, half_width);
        let closest_y = camera_center.y.clamp(-half_height, half_height);
        let closest_point = Vector2d::new(closest_x, closest_y);
        
        (camera_center - closest_point).magnitude() <= scaled_radius
    }

    /// Checks if a rectangular area is visible in the camera view
    pub fn is_rect_visible(&self, center: Vector2d, width: f32, height: f32) -> bool {
        let camera_center = self.world_to_camera(center);
        let scaled_width = width / self.scale;
        let scaled_height = height / self.scale;
        let half_view_width = self.view_width * 0.5 / self.scale;
        let half_view_height = self.view_height * 0.5 / self.scale;
        
        // AABB intersection test
        let rect_left = camera_center.x - scaled_width * 0.5;
        let rect_right = camera_center.x + scaled_width * 0.5;
        let rect_top = camera_center.y - scaled_height * 0.5;
        let rect_bottom = camera_center.y + scaled_height * 0.5;
        
        rect_left <= half_view_width && rect_right >= -half_view_width &&
        rect_top <= half_view_height && rect_bottom >= -half_view_height
    }
}

impl Component for Camera2d {
    fn validate(&self) -> bool {
        // Check that all values are finite and scale is positive
        self.position.x.is_finite() && self.position.y.is_finite() &&
        self.rotation.radians().is_finite() &&
        self.scale.is_finite() && self.scale > 0.0 &&
        self.view_width.is_finite() && self.view_width > 0.0 &&
        self.view_height.is_finite() && self.view_height > 0.0
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

impl Default for Camera2d {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < 0.001
    }

    fn vector_approx_eq(a: Vector2d, b: Vector2d) -> bool {
        approx_eq(a.x, b.x) && approx_eq(a.y, b.y)
    }

    #[test]
    fn test_camera_creation() {
        let camera = Camera2d::new();
        assert!(vector_approx_eq(camera.position(), Vector2d::zero()));
        assert!(approx_eq(camera.rotation().radians(), 0.0));
        assert!(approx_eq(camera.scale(), 1.0));
    }

    #[test]
    fn test_camera_setters() {
        let mut camera = Camera2d::new();
        
        camera.set_position(Vector2d::new(5.0, 3.0));
        assert!(vector_approx_eq(camera.position(), Vector2d::new(5.0, 3.0)));
        
        camera.set_rotation(Angle2d::from_degrees(45.0));
        assert!(approx_eq(camera.rotation().degrees(), 45.0));
        
        camera.set_scale(2.0);
        assert!(approx_eq(camera.scale(), 2.0));
    }

    #[test]
    fn test_camera_transforms() {
        let mut camera = Camera2d::new();
        camera.set_position(Vector2d::new(10.0, 5.0));
        camera.set_scale(2.0);
        
        let world_point = Vector2d::new(12.0, 7.0);
        let camera_point = camera.world_to_camera(world_point);
        let back_to_world = camera.camera_to_world(camera_point);
        
        // Should get back the original point
        assert!(vector_approx_eq(world_point, back_to_world));
    }

    #[test]
    fn test_point_visibility() {
        let mut camera = Camera2d::new();
        camera.set_view_dimensions(100.0, 100.0);
        camera.set_position(Vector2d::zero());
        camera.set_scale(1.0);
        
        // Point at origin should be visible
        assert!(camera.is_point_visible(Vector2d::zero()));
        
        // Point within view should be visible
        assert!(camera.is_point_visible(Vector2d::new(40.0, 40.0)));
        
        // Point outside view should not be visible
        assert!(!camera.is_point_visible(Vector2d::new(100.0, 100.0)));
    }

    #[test]
    fn test_circle_visibility() {
        let mut camera = Camera2d::new();
        camera.set_view_dimensions(100.0, 100.0);
        camera.set_position(Vector2d::zero());
        camera.set_scale(1.0);
        
        // Circle at origin should be visible
        assert!(camera.is_circle_visible(Vector2d::zero(), 10.0));
        
        // Circle partially outside should still be visible
        assert!(camera.is_circle_visible(Vector2d::new(45.0, 45.0), 10.0));
        
        // Circle far outside should not be visible
        assert!(!camera.is_circle_visible(Vector2d::new(200.0, 200.0), 10.0));
    }

    #[test]
    fn test_rect_visibility() {
        let mut camera = Camera2d::new();
        camera.set_view_dimensions(100.0, 100.0);
        camera.set_position(Vector2d::zero());
        camera.set_scale(1.0);
        
        // Rect at origin should be visible
        assert!(camera.is_rect_visible(Vector2d::zero(), 20.0, 20.0));
        
        // Rect partially outside should still be visible
        assert!(camera.is_rect_visible(Vector2d::new(45.0, 45.0), 20.0, 20.0));
        
        // Rect far outside should not be visible
        assert!(!camera.is_rect_visible(Vector2d::new(200.0, 200.0), 20.0, 20.0));
    }

    #[test]
    fn test_camera_validation() {
        let camera = Camera2d::new();
        assert!(camera.validate());
        
        // Create an invalid camera with direct field access to test validation
        let invalid_camera = Camera2d {
            position: Vector2d::new(f32::NAN, 0.0),
            rotation: Angle2d::zero(),
            scale: 1.0,
            view_width: 100.0,
            view_height: 100.0,
        };
        assert!(!invalid_camera.validate());
    }
}