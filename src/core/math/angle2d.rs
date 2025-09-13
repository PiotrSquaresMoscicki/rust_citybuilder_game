use std::f32::consts::PI;
use super::vector2d::Vector2d;

/// Represents a 2D angle with conversion and operation utilities
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Angle2d {
    radians: f32,
}

impl Angle2d {
    /// Creates a new Angle2d from radians
    pub fn from_radians(radians: f32) -> Self {
        Self { radians }
    }

    /// Creates a new Angle2d from degrees
    pub fn from_degrees(degrees: f32) -> Self {
        Self {
            radians: degrees * PI / 180.0,
        }
    }

    /// Creates a zero angle
    pub fn zero() -> Self {
        Self::from_radians(0.0)
    }

    /// Creates a 90-degree angle
    pub fn quarter_turn() -> Self {
        Self::from_radians(PI / 2.0)
    }

    /// Creates a 180-degree angle
    pub fn half_turn() -> Self {
        Self::from_radians(PI)
    }

    /// Creates a 270-degree angle
    pub fn three_quarter_turn() -> Self {
        Self::from_radians(3.0 * PI / 2.0)
    }

    /// Creates a full 360-degree turn
    pub fn full_turn() -> Self {
        Self::from_radians(2.0 * PI)
    }

    /// Gets the angle in radians
    pub fn radians(&self) -> f32 {
        self.radians
    }

    /// Gets the angle in degrees
    pub fn degrees(&self) -> f32 {
        self.radians * 180.0 / PI
    }

    /// Normalizes the angle to be between 0 and 2π
    pub fn normalized(&self) -> Self {
        let mut angle = self.radians % (2.0 * PI);
        if angle < 0.0 {
            angle += 2.0 * PI;
        }
        Self::from_radians(angle)
    }

    /// Normalizes the angle to be between -π and π
    pub fn normalized_signed(&self) -> Self {
        let mut angle = self.radians % (2.0 * PI);
        if angle > PI {
            angle -= 2.0 * PI;
        } else if angle < -PI {
            angle += 2.0 * PI;
        }
        Self::from_radians(angle)
    }

    /// Returns the sine of the angle
    pub fn sin(&self) -> f32 {
        self.radians.sin()
    }

    /// Returns the cosine of the angle
    pub fn cos(&self) -> f32 {
        self.radians.cos()
    }

    /// Returns the tangent of the angle
    pub fn tan(&self) -> f32 {
        self.radians.tan()
    }

    /// Creates an angle from the arctangent of y/x
    pub fn from_atan2(y: f32, x: f32) -> Self {
        Self::from_radians(y.atan2(x))
    }

    /// Creates an angle from a direction vector
    pub fn from_vector(vector: &Vector2d) -> Self {
        Self::from_atan2(vector.y, vector.x)
    }

    /// Converts the angle to a unit direction vector
    pub fn to_vector(&self) -> Vector2d {
        Vector2d::new(self.cos(), self.sin())
    }

    /// Rotates this angle by another angle
    pub fn rotate_by(&self, other: &Angle2d) -> Self {
        Self::from_radians(self.radians + other.radians)
    }

    /// Returns the difference between two angles (shortest path)
    pub fn difference(&self, other: &Angle2d) -> Self {
        let diff = (other.radians - self.radians).abs();
        let shorter_diff = (2.0 * PI - diff).min(diff);
        Self::from_radians(shorter_diff)
    }

    /// Linear interpolation between two angles (takes shortest path)
    pub fn lerp(&self, other: &Angle2d, t: f32) -> Self {
        let self_norm = self.normalized_signed();
        let other_norm = other.normalized_signed();
        
        let mut diff = other_norm.radians - self_norm.radians;
        
        // Take the shorter path
        if diff > PI {
            diff -= 2.0 * PI;
        } else if diff < -PI {
            diff += 2.0 * PI;
        }
        
        Self::from_radians(self_norm.radians + diff * t)
    }
}

impl std::ops::Add for Angle2d {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::from_radians(self.radians + rhs.radians)
    }
}

impl std::ops::Sub for Angle2d {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::from_radians(self.radians - rhs.radians)
    }
}

impl std::ops::Mul<f32> for Angle2d {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::from_radians(self.radians * rhs)
    }
}

impl std::ops::Div<f32> for Angle2d {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self::from_radians(self.radians / rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_angle_creation() {
        let angle_rad = Angle2d::from_radians(PI / 2.0);
        assert!((angle_rad.radians() - PI / 2.0).abs() < 0.001);
        
        let angle_deg = Angle2d::from_degrees(90.0);
        assert!((angle_deg.degrees() - 90.0).abs() < 0.001);
    }

    #[test]
    fn test_angle_constants() {
        assert!((Angle2d::zero().radians() - 0.0).abs() < 0.001);
        assert!((Angle2d::quarter_turn().radians() - PI / 2.0).abs() < 0.001);
        assert!((Angle2d::half_turn().radians() - PI).abs() < 0.001);
        assert!((Angle2d::full_turn().radians() - 2.0 * PI).abs() < 0.001);
    }

    #[test]
    fn test_conversion() {
        let angle = Angle2d::from_degrees(90.0);
        assert!((angle.radians() - PI / 2.0).abs() < 0.001);
        assert!((angle.degrees() - 90.0).abs() < 0.001);
    }

    #[test]
    fn test_trig_functions() {
        let angle = Angle2d::from_degrees(90.0);
        assert!((angle.sin() - 1.0).abs() < 0.001);
        assert!(angle.cos().abs() < 0.001);
        
        let angle45 = Angle2d::from_degrees(45.0);
        assert!((angle45.sin() - 0.7071).abs() < 0.001);
        assert!((angle45.cos() - 0.7071).abs() < 0.001);
    }

    #[test]
    fn test_normalization() {
        let angle = Angle2d::from_radians(3.0 * PI);
        let normalized = angle.normalized();
        assert!((normalized.radians() - PI).abs() < 0.001);
        
        let angle_neg = Angle2d::from_radians(-PI / 2.0);
        let norm_signed = angle_neg.normalized_signed();
        assert!((norm_signed.radians() + PI / 2.0).abs() < 0.001);
    }

    #[test]
    fn test_vector_conversion() {
        let angle = Angle2d::from_degrees(0.0);
        let vector = angle.to_vector();
        assert!((vector.x - 1.0).abs() < 0.001);
        assert!(vector.y.abs() < 0.001);
        
        let vector_up = Vector2d::new(0.0, 1.0);
        let angle_from_vec = Angle2d::from_vector(&vector_up);
        assert!((angle_from_vec.degrees() - 90.0).abs() < 0.001);
    }

    #[test]
    fn test_angle_arithmetic() {
        let a1 = Angle2d::from_degrees(30.0);
        let a2 = Angle2d::from_degrees(60.0);
        
        let sum = a1 + a2;
        assert!((sum.degrees() - 90.0).abs() < 0.001);
        
        let diff = a2 - a1;
        assert!((diff.degrees() - 30.0).abs() < 0.001);
        
        let scaled = a1 * 2.0;
        assert!((scaled.degrees() - 60.0).abs() < 0.001);
    }

    #[test]
    fn test_angle_lerp() {
        let a1 = Angle2d::from_degrees(10.0);
        let a2 = Angle2d::from_degrees(50.0);
        let mid = a1.lerp(&a2, 0.5);
        assert!((mid.degrees() - 30.0).abs() < 0.001);
        
        // Test crossing 0 degrees
        let a1 = Angle2d::from_degrees(350.0);
        let a2 = Angle2d::from_degrees(10.0);
        let mid = a1.lerp(&a2, 0.5);
        assert!((mid.normalized().degrees() - 0.0).abs() < 0.001 || (mid.normalized().degrees() - 360.0).abs() < 0.001);
    }
}