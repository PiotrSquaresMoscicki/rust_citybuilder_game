use std::any::Any;
use crate::ecs::Component;
use super::vector2d::Vector2d;

/// Color representation for sprites and shapes
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    /// Creates a new color with RGBA values (0.0 to 1.0)
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Creates a color with RGB values and alpha 1.0
    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self::new(r, g, b, 1.0)
    }

    /// Creates a white color
    pub fn white() -> Self {
        Self::rgb(1.0, 1.0, 1.0)
    }

    /// Creates a black color
    pub fn black() -> Self {
        Self::rgb(0.0, 0.0, 0.0)
    }

    /// Creates a red color
    pub fn red() -> Self {
        Self::rgb(1.0, 0.0, 0.0)
    }

    /// Creates a green color
    pub fn green() -> Self {
        Self::rgb(0.0, 1.0, 0.0)
    }

    /// Creates a blue color
    pub fn blue() -> Self {
        Self::rgb(0.0, 0.0, 1.0)
    }

    /// Creates a yellow color
    pub fn yellow() -> Self {
        Self::rgb(1.0, 1.0, 0.0)
    }

    /// Creates a transparent color
    pub fn transparent() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }

    /// Converts to RGBA tuple
    pub fn as_tuple(&self) -> (f32, f32, f32, f32) {
        (self.r, self.g, self.b, self.a)
    }
}

/// Sprite2d component for rendering 2D sprites
#[derive(Debug, Clone, PartialEq)]
pub struct Sprite2d {
    /// Texture/image identifier (could be a filename, ID, etc.)
    texture_id: String,
    /// Size of the sprite in world units
    size: Vector2d,
    /// Color tint applied to the sprite
    color: Color,
    /// Z-order for depth sorting (higher values render on top)
    z_order: i32,
    /// Whether the sprite is visible
    visible: bool,
    /// Texture coordinates (UV) for sprite atlases
    uv_rect: (Vector2d, Vector2d), // (min_uv, max_uv)
}

impl Sprite2d {
    /// Creates a new Sprite2d with default values
    pub fn new(texture_id: String, size: Vector2d) -> Self {
        Self {
            texture_id,
            size,
            color: Color::white(),
            z_order: 0,
            visible: true,
            uv_rect: (Vector2d::zero(), Vector2d::new(1.0, 1.0)),
        }
    }

    /// Creates a Sprite2d with specific color
    pub fn with_color(texture_id: String, size: Vector2d, color: Color) -> Self {
        Self {
            texture_id,
            size,
            color,
            z_order: 0,
            visible: true,
            uv_rect: (Vector2d::zero(), Vector2d::new(1.0, 1.0)),
        }
    }

    /// Gets the texture ID
    pub fn texture_id(&self) -> &str {
        &self.texture_id
    }

    /// Sets the texture ID
    pub fn set_texture_id(&mut self, texture_id: String) {
        self.texture_id = texture_id;
    }

    /// Gets the sprite size
    pub fn size(&self) -> Vector2d {
        self.size
    }

    /// Sets the sprite size
    pub fn set_size(&mut self, size: Vector2d) {
        self.size = size;
    }

    /// Gets the color tint
    pub fn color(&self) -> Color {
        self.color
    }

    /// Sets the color tint
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    /// Gets the z-order
    pub fn z_order(&self) -> i32 {
        self.z_order
    }

    /// Sets the z-order
    pub fn set_z_order(&mut self, z_order: i32) {
        self.z_order = z_order;
    }

    /// Gets visibility state
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Sets visibility state
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// Gets the UV rectangle for texture atlases
    pub fn uv_rect(&self) -> (Vector2d, Vector2d) {
        self.uv_rect
    }

    /// Sets the UV rectangle for texture atlases
    pub fn set_uv_rect(&mut self, min_uv: Vector2d, max_uv: Vector2d) {
        self.uv_rect = (min_uv, max_uv);
    }

    /// Gets the bounding radius for culling (half of diagonal)
    pub fn bounding_radius(&self) -> f32 {
        (self.size.x * self.size.x + self.size.y * self.size.y).sqrt() * 0.5
    }

    /// Gets the bounding box dimensions for culling
    pub fn bounding_box(&self) -> (f32, f32) {
        (self.size.x, self.size.y)
    }
}

impl Component for Sprite2d {
    fn validate(&self) -> bool {
        // Check that size values are finite and positive
        self.size.x.is_finite() && self.size.y.is_finite() &&
        self.size.x > 0.0 && self.size.y > 0.0 &&
        // Check that color values are finite and in valid range
        self.color.r.is_finite() && self.color.g.is_finite() &&
        self.color.b.is_finite() && self.color.a.is_finite() &&
        self.color.r >= 0.0 && self.color.r <= 1.0 &&
        self.color.g >= 0.0 && self.color.g <= 1.0 &&
        self.color.b >= 0.0 && self.color.b <= 1.0 &&
        self.color.a >= 0.0 && self.color.a <= 1.0 &&
        // Check UV coordinates are finite
        self.uv_rect.0.x.is_finite() && self.uv_rect.0.y.is_finite() &&
        self.uv_rect.1.x.is_finite() && self.uv_rect.1.y.is_finite()
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

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < 0.001
    }

    #[test]
    fn test_color_creation() {
        let color = Color::new(0.5, 0.3, 0.8, 0.9);
        assert!(approx_eq(color.r, 0.5));
        assert!(approx_eq(color.g, 0.3));
        assert!(approx_eq(color.b, 0.8));
        assert!(approx_eq(color.a, 0.9));
    }

    #[test]
    fn test_color_presets() {
        let white = Color::white();
        assert_eq!(white.as_tuple(), (1.0, 1.0, 1.0, 1.0));

        let black = Color::black();
        assert_eq!(black.as_tuple(), (0.0, 0.0, 0.0, 1.0));

        let red = Color::red();
        assert_eq!(red.as_tuple(), (1.0, 0.0, 0.0, 1.0));
    }

    #[test]
    fn test_sprite_creation() {
        let sprite = Sprite2d::new("test_texture".to_string(), Vector2d::new(64.0, 64.0));
        assert_eq!(sprite.texture_id(), "test_texture");
        assert_eq!(sprite.size(), Vector2d::new(64.0, 64.0));
        assert_eq!(sprite.color(), Color::white());
        assert_eq!(sprite.z_order(), 0);
        assert!(sprite.is_visible());
    }

    #[test]
    fn test_sprite_setters() {
        let mut sprite = Sprite2d::new("test".to_string(), Vector2d::new(32.0, 32.0));
        
        sprite.set_color(Color::red());
        assert_eq!(sprite.color(), Color::red());
        
        sprite.set_z_order(5);
        assert_eq!(sprite.z_order(), 5);
        
        sprite.set_visible(false);
        assert!(!sprite.is_visible());
        
        sprite.set_size(Vector2d::new(128.0, 64.0));
        assert_eq!(sprite.size(), Vector2d::new(128.0, 64.0));
    }

    #[test]
    fn test_sprite_uv_rect() {
        let mut sprite = Sprite2d::new("atlas".to_string(), Vector2d::new(64.0, 64.0));
        
        let min_uv = Vector2d::new(0.0, 0.0);
        let max_uv = Vector2d::new(0.5, 0.5);
        sprite.set_uv_rect(min_uv, max_uv);
        
        let (uv_min, uv_max) = sprite.uv_rect();
        assert_eq!(uv_min, min_uv);
        assert_eq!(uv_max, max_uv);
    }

    #[test]
    fn test_bounding_calculations() {
        let sprite = Sprite2d::new("test".to_string(), Vector2d::new(6.0, 8.0));
        
        let radius = sprite.bounding_radius();
        let expected_radius = (6.0_f32 * 6.0 + 8.0 * 8.0).sqrt() * 0.5;
        assert!(approx_eq(radius, expected_radius));
        
        let (width, height) = sprite.bounding_box();
        assert!(approx_eq(width, 6.0));
        assert!(approx_eq(height, 8.0));
    }

    #[test]
    fn test_sprite_validation() {
        let valid_sprite = Sprite2d::new("test".to_string(), Vector2d::new(64.0, 64.0));
        assert!(valid_sprite.validate());
        
        let mut invalid_sprite = Sprite2d::new("test".to_string(), Vector2d::new(-64.0, 64.0));
        assert!(!invalid_sprite.validate());
        
        invalid_sprite.set_size(Vector2d::new(64.0, 64.0));
        invalid_sprite.set_color(Color::new(2.0, 0.5, 0.5, 1.0)); // Invalid color value
        assert!(!invalid_sprite.validate());
    }
}