use super::{vector2d::Vector2d, angle2d::Angle2d};

/// A 2D transformation matrix for translation, rotation, and scaling
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform2d {
    /// Matrix elements in column-major order:
    /// [m00, m10, m01, m11, m02, m12]
    /// represents:
    /// | m00  m01  m02 |
    /// | m10  m11  m12 |
    /// |  0    0    1  |
    matrix: [f32; 6],
}

impl Transform2d {
    /// Creates an identity transform
    pub fn identity() -> Self {
        Self {
            matrix: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
        }
    }

    /// Creates a translation transform
    pub fn translation(translation: Vector2d) -> Self {
        Self {
            matrix: [1.0, 0.0, 0.0, 1.0, translation.x, translation.y],
        }
    }

    /// Creates a rotation transform around the origin
    pub fn rotation(angle: Angle2d) -> Self {
        let cos = angle.cos();
        let sin = angle.sin();
        Self {
            matrix: [cos, sin, -sin, cos, 0.0, 0.0],
        }
    }

    /// Creates a uniform scale transform
    pub fn scale(scale: f32) -> Self {
        Self {
            matrix: [scale, 0.0, 0.0, scale, 0.0, 0.0],
        }
    }

    /// Creates a non-uniform scale transform
    pub fn scale_non_uniform(scale_x: f32, scale_y: f32) -> Self {
        Self {
            matrix: [scale_x, 0.0, 0.0, scale_y, 0.0, 0.0],
        }
    }

    /// Creates a transform from translation, rotation, and scale
    pub fn from_trs(translation: Vector2d, rotation: Angle2d, scale: f32) -> Self {
        let t = Self::translation(translation);
        let r = Self::rotation(rotation);
        let s = Self::scale(scale);
        t * r * s
    }

    /// Gets the translation component
    pub fn get_translation(&self) -> Vector2d {
        Vector2d::new(self.matrix[4], self.matrix[5])
    }

    /// Gets the rotation component (assuming no skew)
    pub fn get_rotation(&self) -> Angle2d {
        Angle2d::from_atan2(self.matrix[1], self.matrix[0])
    }

    /// Gets the scale component (assuming uniform scale and no skew)
    pub fn get_scale(&self) -> f32 {
        Vector2d::new(self.matrix[0], self.matrix[1]).magnitude()
    }

    /// Gets the scale components for x and y axes
    pub fn scale_components(&self) -> (f32, f32) {
        let scale_x = Vector2d::new(self.matrix[0], self.matrix[1]).magnitude();
        let scale_y = Vector2d::new(self.matrix[2], self.matrix[3]).magnitude();
        (scale_x, scale_y)
    }

    /// Transforms a point
    pub fn transform_point(&self, point: Vector2d) -> Vector2d {
        Vector2d::new(
            self.matrix[0] * point.x + self.matrix[2] * point.y + self.matrix[4],
            self.matrix[1] * point.x + self.matrix[3] * point.y + self.matrix[5],
        )
    }

    /// Transforms a vector (ignores translation)
    pub fn transform_vector(&self, vector: Vector2d) -> Vector2d {
        Vector2d::new(
            self.matrix[0] * vector.x + self.matrix[2] * vector.y,
            self.matrix[1] * vector.x + self.matrix[3] * vector.y,
        )
    }

    /// Computes the inverse transform
    pub fn inverse(&self) -> Option<Self> {
        let det = self.matrix[0] * self.matrix[3] - self.matrix[1] * self.matrix[2];
        
        if det.abs() < f32::EPSILON {
            return None; // Transform is not invertible
        }
        
        let inv_det = 1.0 / det;
        
        let inv_m00 = self.matrix[3] * inv_det;
        let inv_m01 = -self.matrix[2] * inv_det;
        let inv_m10 = -self.matrix[1] * inv_det;
        let inv_m11 = self.matrix[0] * inv_det;
        let inv_m02 = (self.matrix[2] * self.matrix[5] - self.matrix[3] * self.matrix[4]) * inv_det;
        let inv_m12 = (self.matrix[1] * self.matrix[4] - self.matrix[0] * self.matrix[5]) * inv_det;
        
        Some(Self {
            matrix: [inv_m00, inv_m10, inv_m01, inv_m11, inv_m02, inv_m12],
        })
    }

    /// Returns the raw matrix elements
    pub fn matrix(&self) -> [f32; 6] {
        self.matrix
    }

    /// Creates a transform from raw matrix elements
    pub fn from_matrix(matrix: [f32; 6]) -> Self {
        Self { matrix }
    }

    /// Linear interpolation between two transforms
    pub fn lerp(&self, other: &Transform2d, t: f32) -> Transform2d {
        let mut result = [0.0; 6];
        for i in 0..6 {
            result[i] = self.matrix[i] + (other.matrix[i] - self.matrix[i]) * t;
        }
        Transform2d::from_matrix(result)
    }
}

impl std::ops::Mul for Transform2d {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        // Matrix multiplication for 2D affine transforms
        let a = &self.matrix;
        let b = &rhs.matrix;
        
        Self {
            matrix: [
                a[0] * b[0] + a[2] * b[1],                    // m00
                a[1] * b[0] + a[3] * b[1],                    // m10
                a[0] * b[2] + a[2] * b[3],                    // m01
                a[1] * b[2] + a[3] * b[3],                    // m11
                a[0] * b[4] + a[2] * b[5] + a[4],            // m02
                a[1] * b[4] + a[3] * b[5] + a[5],            // m12
            ],
        }
    }
}

impl Default for Transform2d {
    fn default() -> Self {
        Self::identity()
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
    fn test_identity() {
        let identity = Transform2d::identity();
        let point = Vector2d::new(3.0, 4.0);
        let transformed = identity.transform_point(point);
        assert!(vector_approx_eq(transformed, point));
    }

    #[test]
    fn test_translation() {
        let translation = Transform2d::translation(Vector2d::new(5.0, 3.0));
        let point = Vector2d::new(1.0, 1.0);
        let transformed = translation.transform_point(point);
        assert!(vector_approx_eq(transformed, Vector2d::new(6.0, 4.0)));
        
        // Test that translation doesn't affect vectors
        let vector = Vector2d::new(1.0, 1.0);
        let transformed_vec = translation.transform_vector(vector);
        assert!(vector_approx_eq(transformed_vec, vector));
    }

    #[test]
    fn test_rotation() {
        let rotation = Transform2d::rotation(Angle2d::from_degrees(90.0));
        let point = Vector2d::new(1.0, 0.0);
        let transformed = rotation.transform_point(point);
        assert!(vector_approx_eq(transformed, Vector2d::new(0.0, 1.0)));
    }

    #[test]
    fn test_scale() {
        let scale = Transform2d::scale(2.0);
        let point = Vector2d::new(3.0, 4.0);
        let transformed = scale.transform_point(point);
        assert!(vector_approx_eq(transformed, Vector2d::new(6.0, 8.0)));
    }

    #[test]
    fn test_non_uniform_scale() {
        let scale = Transform2d::scale_non_uniform(2.0, 3.0);
        let point = Vector2d::new(1.0, 1.0);
        let transformed = scale.transform_point(point);
        assert!(vector_approx_eq(transformed, Vector2d::new(2.0, 3.0)));
    }

    #[test]
    fn test_from_trs() {
        let transform = Transform2d::from_trs(
            Vector2d::new(1.0, 2.0),
            Angle2d::from_degrees(90.0),
            2.0,
        );
        
        let point = Vector2d::new(1.0, 0.0);
        let transformed = transform.transform_point(point);
        // Scale by 2, rotate 90 degrees, then translate
        // (1,0) -> (2,0) -> (0,2) -> (1,4)
        assert!(vector_approx_eq(transformed, Vector2d::new(1.0, 4.0)));
    }

    #[test]
    fn test_component_extraction() {
        let translation = Vector2d::new(5.0, 3.0);
        let rotation = Angle2d::from_degrees(45.0);
        let scale = 2.0;
        
        let transform = Transform2d::from_trs(translation, rotation, scale);
        
        assert!(vector_approx_eq(transform.get_translation(), translation));
        assert!(approx_eq(transform.get_rotation().degrees(), rotation.degrees()));
        assert!(approx_eq(transform.get_scale(), scale));
    }

    #[test]
    fn test_matrix_multiplication() {
        let t1 = Transform2d::translation(Vector2d::new(1.0, 2.0));
        let t2 = Transform2d::rotation(Angle2d::from_degrees(90.0));
        
        let combined = t1 * t2;
        let point = Vector2d::new(1.0, 0.0);
        let transformed = combined.transform_point(point);
        
        // First rotate, then translate: (1,0) -> (0,1) -> (1,3)
        assert!(vector_approx_eq(transformed, Vector2d::new(1.0, 3.0)));
    }

    #[test]
    fn test_inverse() {
        let transform = Transform2d::from_trs(
            Vector2d::new(3.0, 4.0),
            Angle2d::from_degrees(45.0),
            2.0,
        );
        
        let inverse = transform.inverse().unwrap();
        let combined = transform * inverse;
        
        // Should be close to identity
        let point = Vector2d::new(5.0, 7.0);
        let transformed = combined.transform_point(point);
        assert!(vector_approx_eq(transformed, point));
    }

    #[test]
    fn test_lerp() {
        let t1 = Transform2d::translation(Vector2d::new(0.0, 0.0));
        let t2 = Transform2d::translation(Vector2d::new(10.0, 10.0));
        let mid = t1.lerp(&t2, 0.5);
        
        let point = Vector2d::new(0.0, 0.0);
        let transformed = mid.transform_point(point);
        assert!(vector_approx_eq(transformed, Vector2d::new(5.0, 5.0)));
    }
}