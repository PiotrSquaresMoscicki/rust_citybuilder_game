use std::any::Any;
use crate::ecs::Component;
use super::{vector2d::Vector2d, sprite2d::Color};

/// Different types of 2D shapes that can be rendered
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)] // Core shape types for 2D rendering system
pub enum ShapeType {
    /// Circle with radius
    Circle { radius: f32 },
    /// Rectangle with width and height
    Rectangle { width: f32, height: f32 },
    /// Triangle with three vertices (relative to center)
    Triangle { 
        vertex1: Vector2d, 
        vertex2: Vector2d, 
        vertex3: Vector2d 
    },
    /// Line from start to end point (relative to center)
    Line { 
        start: Vector2d, 
        end: Vector2d, 
        thickness: f32 
    },
    /// Polygon with multiple vertices (relative to center)
    Polygon { vertices: Vec<Vector2d> },
}

impl ShapeType {
    /// Gets the bounding radius for culling purposes
    pub fn bounding_radius(&self) -> f32 {
        match self {
            ShapeType::Circle { radius } => *radius,
            ShapeType::Rectangle { width, height } => {
                (width * width + height * height).sqrt() * 0.5
            },
            ShapeType::Triangle { vertex1, vertex2, vertex3 } => {
                let max_dist = [vertex1, vertex2, vertex3]
                    .iter()
                    .map(|v| v.magnitude())
                    .fold(0.0, f32::max);
                max_dist
            },
            ShapeType::Line { start, end, thickness } => {
                let line_length = (*end - *start).magnitude();
                let max_radius = (line_length * 0.5).max(*thickness * 0.5);
                max_radius
            },
            ShapeType::Polygon { vertices } => {
                vertices.iter()
                    .map(|v| v.magnitude())
                    .fold(0.0, f32::max)
            },
        }
    }

    /// Gets the bounding box dimensions for culling
    pub fn bounding_box(&self) -> (f32, f32) {
        match self {
            ShapeType::Circle { radius } => (radius * 2.0, radius * 2.0),
            ShapeType::Rectangle { width, height } => (*width, *height),
            ShapeType::Triangle { vertex1, vertex2, vertex3 } => {
                let min_x = [vertex1.x, vertex2.x, vertex3.x].iter().fold(f32::INFINITY, |a, &b| a.min(b));
                let max_x = [vertex1.x, vertex2.x, vertex3.x].iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
                let min_y = [vertex1.y, vertex2.y, vertex3.y].iter().fold(f32::INFINITY, |a, &b| a.min(b));
                let max_y = [vertex1.y, vertex2.y, vertex3.y].iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
                (max_x - min_x, max_y - min_y)
            },
            ShapeType::Line { start, end, thickness } => {
                let line_vec = *end - *start;
                let width = line_vec.x.abs().max(*thickness);
                let height = line_vec.y.abs().max(*thickness);
                (width, height)
            },
            ShapeType::Polygon { vertices } => {
                if vertices.is_empty() {
                    return (0.0, 0.0);
                }
                let min_x = vertices.iter().map(|v| v.x).fold(f32::INFINITY, f32::min);
                let max_x = vertices.iter().map(|v| v.x).fold(f32::NEG_INFINITY, f32::max);
                let min_y = vertices.iter().map(|v| v.y).fold(f32::INFINITY, f32::min);
                let max_y = vertices.iter().map(|v| v.y).fold(f32::NEG_INFINITY, f32::max);
                (max_x - min_x, max_y - min_y)
            },
        }
    }
}

/// Fill style for shapes
#[derive(Debug, Clone, PartialEq)]
pub enum FillStyle {
    /// Solid fill with color
    Solid(Color),
    /// No fill (outline only)
    None,
}

/// Stroke style for shape outlines
#[derive(Debug, Clone, PartialEq)]
pub struct StrokeStyle {
    pub color: Color,
    pub width: f32,
}

impl StrokeStyle {
    pub fn new(color: Color, width: f32) -> Self {
        Self { color, width }
    }
}

/// Shape2d component for rendering 2D geometric shapes
#[derive(Debug, Clone, PartialEq)]
pub struct Shape2d {
    /// The type and geometry of the shape
    shape_type: ShapeType,
    /// Fill style for the shape interior
    fill: FillStyle,
    /// Stroke style for the shape outline (optional)
    stroke: Option<StrokeStyle>,
    /// Z-order for depth sorting (higher values render on top)
    z_order: i32,
    /// Whether the shape is visible
    visible: bool,
}

impl Shape2d {
    /// Creates a new Shape2d with solid fill
    pub fn new(shape_type: ShapeType, fill_color: Color) -> Self {
        Self {
            shape_type,
            fill: FillStyle::Solid(fill_color),
            stroke: None,
            z_order: 0,
            visible: true,
        }
    }

    /// Creates a new Shape2d with outline only
    pub fn outline_only(shape_type: ShapeType, stroke_color: Color, stroke_width: f32) -> Self {
        Self {
            shape_type,
            fill: FillStyle::None,
            stroke: Some(StrokeStyle::new(stroke_color, stroke_width)),
            z_order: 0,
            visible: true,
        }
    }

    /// Creates a new Shape2d with both fill and stroke
    pub fn with_stroke(shape_type: ShapeType, fill_color: Color, stroke_color: Color, stroke_width: f32) -> Self {
        Self {
            shape_type,
            fill: FillStyle::Solid(fill_color),
            stroke: Some(StrokeStyle::new(stroke_color, stroke_width)),
            z_order: 0,
            visible: true,
        }
    }

    /// Creates a circle shape
    pub fn circle(radius: f32, color: Color) -> Self {
        Self::new(ShapeType::Circle { radius }, color)
    }

    /// Creates a rectangle shape
    pub fn rectangle(width: f32, height: f32, color: Color) -> Self {
        Self::new(ShapeType::Rectangle { width, height }, color)
    }

    /// Creates a triangle shape
    pub fn triangle(v1: Vector2d, v2: Vector2d, v3: Vector2d, color: Color) -> Self {
        Self::new(ShapeType::Triangle { 
            vertex1: v1, 
            vertex2: v2, 
            vertex3: v3 
        }, color)
    }

    /// Creates a line shape
    pub fn line(start: Vector2d, end: Vector2d, thickness: f32, color: Color) -> Self {
        Self::new(ShapeType::Line { start, end, thickness }, color)
    }

    /// Gets the shape type
    pub fn shape_type(&self) -> &ShapeType {
        &self.shape_type
    }

    /// Sets the shape type
    pub fn set_shape_type(&mut self, shape_type: ShapeType) {
        self.shape_type = shape_type;
    }

    /// Gets the fill style
    pub fn fill(&self) -> &FillStyle {
        &self.fill
    }

    /// Sets the fill style
    pub fn set_fill(&mut self, fill: FillStyle) {
        self.fill = fill;
    }

    /// Gets the stroke style
    pub fn stroke(&self) -> Option<&StrokeStyle> {
        self.stroke.as_ref()
    }

    /// Sets the stroke style
    pub fn set_stroke(&mut self, stroke: Option<StrokeStyle>) {
        self.stroke = stroke;
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

    /// Gets the bounding radius for culling
    pub fn bounding_radius(&self) -> f32 {
        let shape_radius = self.shape_type.bounding_radius();
        if let Some(stroke) = &self.stroke {
            shape_radius + stroke.width * 0.5
        } else {
            shape_radius
        }
    }

    /// Gets the bounding box dimensions for culling
    pub fn bounding_box(&self) -> (f32, f32) {
        let (mut width, mut height) = self.shape_type.bounding_box();
        if let Some(stroke) = &self.stroke {
            width += stroke.width;
            height += stroke.width;
        }
        (width, height)
    }
}

impl Component for Shape2d {
    fn validate(&self) -> bool {
        // Validate shape type
        let shape_valid = match &self.shape_type {
            ShapeType::Circle { radius } => radius.is_finite() && *radius > 0.0,
            ShapeType::Rectangle { width, height } => {
                width.is_finite() && height.is_finite() && *width > 0.0 && *height > 0.0
            },
            ShapeType::Triangle { vertex1, vertex2, vertex3 } => {
                vertex1.x.is_finite() && vertex1.y.is_finite() &&
                vertex2.x.is_finite() && vertex2.y.is_finite() &&
                vertex3.x.is_finite() && vertex3.y.is_finite()
            },
            ShapeType::Line { start, end, thickness } => {
                start.x.is_finite() && start.y.is_finite() &&
                end.x.is_finite() && end.y.is_finite() &&
                thickness.is_finite() && *thickness > 0.0
            },
            ShapeType::Polygon { vertices } => {
                vertices.len() >= 3 && vertices.iter().all(|v| v.x.is_finite() && v.y.is_finite())
            },
        };

        // Validate fill
        let fill_valid = match &self.fill {
            FillStyle::Solid(color) => {
                color.r.is_finite() && color.g.is_finite() &&
                color.b.is_finite() && color.a.is_finite() &&
                color.r >= 0.0 && color.r <= 1.0 &&
                color.g >= 0.0 && color.g <= 1.0 &&
                color.b >= 0.0 && color.b <= 1.0 &&
                color.a >= 0.0 && color.a <= 1.0
            },
            FillStyle::None => true,
        };

        // Validate stroke
        let stroke_valid = if let Some(stroke) = &self.stroke {
            stroke.color.r.is_finite() && stroke.color.g.is_finite() &&
            stroke.color.b.is_finite() && stroke.color.a.is_finite() &&
            stroke.color.r >= 0.0 && stroke.color.r <= 1.0 &&
            stroke.color.g >= 0.0 && stroke.color.g <= 1.0 &&
            stroke.color.b >= 0.0 && stroke.color.b <= 1.0 &&
            stroke.color.a >= 0.0 && stroke.color.a <= 1.0 &&
            stroke.width.is_finite() && stroke.width > 0.0
        } else {
            true
        };

        shape_valid && fill_valid && stroke_valid
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
    fn test_shape_type_circle() {
        let circle = ShapeType::Circle { radius: 5.0 };
        assert!(approx_eq(circle.bounding_radius(), 5.0));
        let (width, height) = circle.bounding_box();
        assert!(approx_eq(width, 10.0));
        assert!(approx_eq(height, 10.0));
    }

    #[test]
    fn test_shape_type_rectangle() {
        let rect = ShapeType::Rectangle { width: 6.0, height: 8.0 };
        let expected_radius = (6.0_f32 * 6.0 + 8.0 * 8.0).sqrt() * 0.5;
        assert!(approx_eq(rect.bounding_radius(), expected_radius));
        let (width, height) = rect.bounding_box();
        assert!(approx_eq(width, 6.0));
        assert!(approx_eq(height, 8.0));
    }

    #[test]
    fn test_shape_creation() {
        let circle = Shape2d::circle(10.0, Color::red());
        assert!(matches!(circle.shape_type(), ShapeType::Circle { radius } if *radius == 10.0));
        assert_eq!(circle.z_order(), 0);
        assert!(circle.is_visible());
    }

    #[test]
    fn test_shape_with_stroke() {
        let rect = Shape2d::with_stroke(
            ShapeType::Rectangle { width: 20.0, height: 10.0 },
            Color::blue(),
            Color::black(),
            2.0
        );
        
        assert!(matches!(rect.fill(), FillStyle::Solid(color) if *color == Color::blue()));
        assert!(rect.stroke().is_some());
        
        if let Some(stroke) = rect.stroke() {
            assert_eq!(stroke.color, Color::black());
            assert!(approx_eq(stroke.width, 2.0));
        }
    }

    #[test]
    fn test_outline_only_shape() {
        let line = Shape2d::outline_only(
            ShapeType::Line { 
                start: Vector2d::zero(), 
                end: Vector2d::new(10.0, 0.0), 
                thickness: 1.0 
            },
            Color::green(),
            1.0
        );
        
        assert!(matches!(line.fill(), FillStyle::None));
        assert!(line.stroke().is_some());
    }

    #[test]
    fn test_bounding_calculations_with_stroke() {
        let circle = Shape2d::with_stroke(
            ShapeType::Circle { radius: 5.0 },
            Color::red(),
            Color::black(),
            2.0
        );
        
        // Base radius + half stroke width
        let expected_radius = 5.0 + 1.0;
        assert!(approx_eq(circle.bounding_radius(), expected_radius));
        
        let (width, height) = circle.bounding_box();
        assert!(approx_eq(width, 12.0)); // 10.0 + 2.0 stroke
        assert!(approx_eq(height, 12.0));
    }

    #[test]
    fn test_triangle_shape() {
        let triangle = Shape2d::triangle(
            Vector2d::new(0.0, 5.0),
            Vector2d::new(-4.0, -3.0),
            Vector2d::new(4.0, -3.0),
            Color::yellow()
        );
        
        assert!(matches!(triangle.shape_type(), ShapeType::Triangle { .. }));
        assert!(triangle.validate());
    }

    #[test]
    fn test_shape_validation() {
        let valid_shape = Shape2d::circle(5.0, Color::red());
        assert!(valid_shape.validate());
        
        let invalid_shape = Shape2d::circle(-1.0, Color::red());
        assert!(!invalid_shape.validate());
        
        let invalid_color_shape = Shape2d::circle(5.0, Color::new(2.0, 0.5, 0.5, 1.0));
        assert!(!invalid_color_shape.validate());
    }
}