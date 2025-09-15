use std::any::Any;
use crate::ecs::Component;
use super::{transform2d::Transform2d, vector2d::Vector2d, angle2d::Angle2d};

/// ECS Component wrapper for Transform2d
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)] // Core component for 2D transforms in the game engine
pub struct Transform2dComponent {
    transform: Transform2d,
}

#[allow(dead_code)] // Core component implementation for 2D transforms
impl Transform2dComponent {
    /// Creates a new Transform2dComponent with identity transform
    pub fn new() -> Self {
        Self {
            transform: Transform2d::identity(),
        }
    }

    /// Creates a Transform2dComponent from a Transform2d
    pub fn from_transform(transform: Transform2d) -> Self {
        Self { transform }
    }

    /// Creates a Transform2dComponent from translation, rotation, and scale
    pub fn from_trs(translation: Vector2d, rotation: Angle2d, scale: f32) -> Self {
        Self {
            transform: Transform2d::from_trs(translation, rotation, scale),
        }
    }

    /// Creates a Transform2dComponent with only translation
    pub fn from_translation(translation: Vector2d) -> Self {
        Self {
            transform: Transform2d::translation(translation),
        }
    }

    /// Creates a Transform2dComponent with only rotation
    pub fn from_rotation(rotation: Angle2d) -> Self {
        Self {
            transform: Transform2d::rotation(rotation),
        }
    }

    /// Creates a Transform2dComponent with only scale
    pub fn from_scale(scale: f32) -> Self {
        Self {
            transform: Transform2d::scale(scale),
        }
    }

    /// Gets the underlying transform
    pub fn transform(&self) -> Transform2d {
        self.transform
    }

    /// Sets the underlying transform
    pub fn set_transform(&mut self, transform: Transform2d) {
        self.transform = transform;
    }

    /// Gets the translation component
    pub fn translation(&self) -> Vector2d {
        self.transform.get_translation()
    }

    /// Sets the translation component
    pub fn set_translation(&mut self, translation: Vector2d) {
        let rotation = self.transform.get_rotation();
        let scale = self.transform.get_scale();
        self.transform = Transform2d::from_trs(translation, rotation, scale);
    }

    /// Gets the rotation component
    pub fn rotation(&self) -> Angle2d {
        self.transform.get_rotation()
    }

    /// Sets the rotation component
    pub fn set_rotation(&mut self, rotation: Angle2d) {
        let translation = self.transform.get_translation();
        let scale = self.transform.get_scale();
        self.transform = Transform2d::from_trs(translation, rotation, scale);
    }

    /// Gets the scale component
    pub fn scale(&self) -> f32 {
        self.transform.get_scale()
    }

    /// Sets the scale component
    pub fn set_scale(&mut self, scale: f32) {
        let translation = self.transform.get_translation();
        let rotation = self.transform.get_rotation();
        self.transform = Transform2d::from_trs(translation, rotation, scale);
    }

    /// Translates the transform by the given offset
    pub fn translate(&mut self, offset: Vector2d) {
        let current_translation = self.translation();
        self.set_translation(current_translation + offset);
    }

    /// Rotates the transform by the given angle
    pub fn rotate(&mut self, angle: Angle2d) {
        let current_rotation = self.rotation();
        self.set_rotation(current_rotation + angle);
    }

    /// Scales the transform by the given factor
    pub fn scale_by(&mut self, factor: f32) {
        let current_scale = self.scale();
        self.set_scale(current_scale * factor);
    }

    /// Transforms a point using this transform
    pub fn transform_point(&self, point: Vector2d) -> Vector2d {
        self.transform.transform_point(point)
    }

    /// Transforms a vector using this transform (ignores translation)
    pub fn transform_vector(&self, vector: Vector2d) -> Vector2d {
        self.transform.transform_vector(vector)
    }

    /// Gets the forward direction vector (positive x in local space)
    pub fn forward(&self) -> Vector2d {
        self.transform_vector(Vector2d::right())
    }

    /// Gets the right direction vector (positive y in local space)
    pub fn right(&self) -> Vector2d {
        self.transform_vector(Vector2d::up())
    }

    /// Sets the position to look at a target position
    pub fn look_at(&mut self, target: Vector2d) {
        let current_pos = self.translation();
        let direction = (target - current_pos).normalized();
        let angle = Angle2d::from_vector(&direction);
        self.set_rotation(angle);
    }

    /// Linear interpolation to another transform
    pub fn lerp_to(&self, other: &Transform2dComponent, t: f32) -> Self {
        Self {
            transform: self.transform.lerp(&other.transform, t),
        }
    }
}

impl Component for Transform2dComponent {
    fn validate(&self) -> bool {
        // Check that the transform matrix is valid (no NaN or infinite values)
        let matrix = self.transform.matrix();
        matrix.iter().all(|&x| x.is_finite())
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

impl Default for Transform2dComponent {
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
    fn test_component_creation() {
        let component = Transform2dComponent::new();
        assert!(vector_approx_eq(component.translation(), Vector2d::zero()));
        assert!(approx_eq(component.rotation().radians(), 0.0));
        assert!(approx_eq(component.scale(), 1.0));
    }

    #[test]
    fn test_from_trs() {
        let translation = Vector2d::new(5.0, 3.0);
        let rotation = Angle2d::from_degrees(45.0);
        let scale = 2.0;

        let component = Transform2dComponent::from_trs(translation, rotation, scale);
        
        assert!(vector_approx_eq(component.translation(), translation));
        assert!(approx_eq(component.rotation().degrees(), rotation.degrees()));
        assert!(approx_eq(component.scale(), scale));
    }

    #[test]
    fn test_setters() {
        let mut component = Transform2dComponent::new();
        
        component.set_translation(Vector2d::new(1.0, 2.0));
        assert!(vector_approx_eq(component.translation(), Vector2d::new(1.0, 2.0)));
        
        component.set_rotation(Angle2d::from_degrees(90.0));
        assert!(approx_eq(component.rotation().degrees(), 90.0));
        
        component.set_scale(3.0);
        assert!(approx_eq(component.scale(), 3.0));
    }

    #[test]
    fn test_transform_operations() {
        let mut component = Transform2dComponent::from_translation(Vector2d::new(5.0, 5.0));
        
        component.translate(Vector2d::new(2.0, 3.0));
        assert!(vector_approx_eq(component.translation(), Vector2d::new(7.0, 8.0)));
        
        component.rotate(Angle2d::from_degrees(45.0));
        assert!(approx_eq(component.rotation().degrees(), 45.0));
        
        component.scale_by(2.0);
        assert!(approx_eq(component.scale(), 2.0));
    }

    #[test]
    fn test_point_transformation() {
        let component = Transform2dComponent::from_translation(Vector2d::new(1.0, 1.0));
        let point = Vector2d::new(2.0, 3.0);
        let transformed = component.transform_point(point);
        assert!(vector_approx_eq(transformed, Vector2d::new(3.0, 4.0)));
    }

    #[test]
    fn test_direction_vectors() {
        let component = Transform2dComponent::from_rotation(Angle2d::from_degrees(90.0));
        let forward = component.forward();
        let right = component.right();
        
        // After 90-degree rotation: forward should point up, right should point left
        assert!(vector_approx_eq(forward, Vector2d::new(0.0, 1.0)));
        assert!(vector_approx_eq(right, Vector2d::new(-1.0, 0.0)));
    }

    #[test]
    fn test_look_at() {
        let mut component = Transform2dComponent::from_translation(Vector2d::new(0.0, 0.0));
        component.look_at(Vector2d::new(1.0, 1.0));
        
        // Should be looking at 45 degrees
        assert!(approx_eq(component.rotation().degrees(), 45.0));
    }

    #[test]
    fn test_component_validation() {
        let component = Transform2dComponent::new();
        assert!(component.validate());
    }

    #[test]
    fn test_lerp() {
        let comp1 = Transform2dComponent::from_translation(Vector2d::new(0.0, 0.0));
        let comp2 = Transform2dComponent::from_translation(Vector2d::new(10.0, 10.0));
        let mid = comp1.lerp_to(&comp2, 0.5);
        
        assert!(vector_approx_eq(mid.translation(), Vector2d::new(5.0, 5.0)));
    }
}