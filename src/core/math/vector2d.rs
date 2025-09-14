use std::ops::{Add, Sub, Mul, Div, Neg};

/// A 2D vector with basic mathematical operations
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector2d {
    pub x: f32,
    pub y: f32,
}

impl Vector2d {
    /// Creates a new Vector2d
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Creates a zero vector
    pub fn zero() -> Self {
        Self::new(0.0, 0.0)
    }

    /// Creates a unit vector pointing right
    pub fn right() -> Self {
        Self::new(1.0, 0.0)
    }

    /// Creates a unit vector pointing up
    pub fn up() -> Self {
        Self::new(0.0, 1.0)
    }

    /// Calculates the magnitude (length) of the vector
    pub fn magnitude(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    /// Calculates the squared magnitude (avoids sqrt for performance)
    pub fn magnitude_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    /// Returns a normalized version of the vector (unit vector)
    pub fn normalized(&self) -> Self {
        let mag = self.magnitude();
        if mag != 0.0 {
            Self::new(self.x / mag, self.y / mag)
        } else {
            Self::zero()
        }
    }

    /// Normalizes the vector in place
    pub fn normalize(&mut self) {
        let mag = self.magnitude();
        if mag != 0.0 {
            self.x /= mag;
            self.y /= mag;
        }
    }

    /// Calculates the dot product with another vector
    pub fn dot(&self, other: &Vector2d) -> f32 {
        self.x * other.x + self.y * other.y
    }

    /// Calculates the distance to another vector
    pub fn distance_to(&self, other: &Vector2d) -> f32 {
        (*other - *self).magnitude()
    }

    /// Calculates the squared distance to another vector
    pub fn distance_squared_to(&self, other: &Vector2d) -> f32 {
        (*other - *self).magnitude_squared()
    }

    /// Linear interpolation between this vector and another
    pub fn lerp(&self, other: &Vector2d, t: f32) -> Vector2d {
        *self + (*other - *self) * t
    }
}

impl Add for Vector2d {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub for Vector2d {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Mul<f32> for Vector2d {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

impl Mul<Vector2d> for f32 {
    type Output = Vector2d;

    fn mul(self, rhs: Vector2d) -> Self::Output {
        Vector2d::new(rhs.x * self, rhs.y * self)
    }
}

impl Div<f32> for Vector2d {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self::new(self.x / rhs, self.y / rhs)
    }
}

impl Neg for Vector2d {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_creation() {
        let v = Vector2d::new(3.0, 4.0);
        assert_eq!(v.x, 3.0);
        assert_eq!(v.y, 4.0);
    }

    #[test]
    fn test_vector_constants() {
        assert_eq!(Vector2d::zero(), Vector2d::new(0.0, 0.0));
        assert_eq!(Vector2d::right(), Vector2d::new(1.0, 0.0));
        assert_eq!(Vector2d::up(), Vector2d::new(0.0, 1.0));
    }

    #[test]
    fn test_magnitude() {
        let v = Vector2d::new(3.0, 4.0);
        assert_eq!(v.magnitude(), 5.0);
        assert_eq!(v.magnitude_squared(), 25.0);
    }

    #[test]
    fn test_normalization() {
        let v = Vector2d::new(3.0, 4.0);
        let normalized = v.normalized();
        assert!((normalized.magnitude() - 1.0).abs() < 0.001);
        
        let mut v_mut = Vector2d::new(3.0, 4.0);
        v_mut.normalize();
        assert!((v_mut.magnitude() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_arithmetic() {
        let v1 = Vector2d::new(1.0, 2.0);
        let v2 = Vector2d::new(3.0, 4.0);
        
        assert_eq!(v1 + v2, Vector2d::new(4.0, 6.0));
        assert_eq!(v2 - v1, Vector2d::new(2.0, 2.0));
        assert_eq!(v1 * 2.0, Vector2d::new(2.0, 4.0));
        assert_eq!(2.0 * v1, Vector2d::new(2.0, 4.0));
        assert_eq!(v1 / 2.0, Vector2d::new(0.5, 1.0));
    }

    #[test]
    fn test_dot_product() {
        let v1 = Vector2d::new(1.0, 2.0);
        let v2 = Vector2d::new(3.0, 4.0);
        assert_eq!(v1.dot(&v2), 11.0);
    }

    #[test]
    fn test_distance() {
        let v1 = Vector2d::new(0.0, 0.0);
        let v2 = Vector2d::new(3.0, 4.0);
        assert_eq!(v1.distance_to(&v2), 5.0);
        assert_eq!(v1.distance_squared_to(&v2), 25.0);
    }

    #[test]
    fn test_lerp() {
        let v1 = Vector2d::new(0.0, 0.0);
        let v2 = Vector2d::new(10.0, 10.0);
        let mid = v1.lerp(&v2, 0.5);
        assert_eq!(mid, Vector2d::new(5.0, 5.0));
    }
}