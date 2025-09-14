/// Web-render sample that demonstrates the 2D rendering system
/// This sample creates a scene with various shapes and colors,
/// spawns a camera entity, and demonstrates the rendering pipeline.

use crate::ecs::World;
use crate::core::math::{
    Camera2d, Sprite2d, Shape2d, Transform2dComponent, Vector2d, Color, 
    ShapeType, FillStyle, StrokeStyle, Angle2d
};
use crate::rendering::{
    initialize_global_rendering_manager, 
    WebClientRenderingDevice, 
    WebServiceManager
};
use std::error::Error;

/// Initialize the web rendering system
pub fn initialize_web_render_system() -> Result<(), Box<dyn Error>> {
    // Create web service manager
    let web_service = WebServiceManager::new("localhost:8080");
    
    // Create web client rendering device
    let web_device = WebClientRenderingDevice::new(web_service);
    
    // Initialize global rendering manager with web device
    initialize_global_rendering_manager(Box::new(web_device))?;
    
    println!("Web render system initialized on http://localhost:8080");
    Ok(())
}

/// Create a sample world with various entities for rendering
pub fn create_sample_world() -> World {
    let mut world = World::new();
    
    // Create camera entity
    let camera_entity = world.create_entity();
    let mut camera = Camera2d::new();
    camera.set_view_dimensions(800.0, 600.0);
    camera.set_scale(1.0);
    let camera_transform = Transform2dComponent::from_translation(Vector2d::new(0.0, 0.0));
    world.add_component(camera_entity, camera);
    world.add_component(camera_entity, camera_transform);
    println!("Created camera entity at origin");
    
    // Create various sprite entities
    create_sprite_entities(&mut world);
    
    // Create various shape entities
    create_shape_entities(&mut world);
    
    println!("Sample world created with sprites and shapes");
    world
}

/// Create various sprite entities with different textures and colors
fn create_sprite_entities(world: &mut World) {
    // Sprite 1: Player character
    let player_entity = world.create_entity();
    let player_sprite = Sprite2d::with_color(
        "player.png".to_string(),
        Vector2d::new(64.0, 64.0),
        Color::white()
    );
    let player_transform = Transform2dComponent::from_translation(Vector2d::new(-200.0, 0.0));
    world.add_component(player_entity, player_sprite);
    world.add_component(player_entity, player_transform);
    
    // Sprite 2: Enemy with red tint
    let enemy_entity = world.create_entity();
    let mut enemy_sprite = Sprite2d::with_color(
        "enemy.png".to_string(),
        Vector2d::new(48.0, 48.0),
        Color::red()
    );
    enemy_sprite.set_z_order(1);
    let enemy_transform = Transform2dComponent::from_translation(Vector2d::new(150.0, -100.0));
    world.add_component(enemy_entity, enemy_sprite);
    world.add_component(enemy_entity, enemy_transform);
    
    // Sprite 3: Collectible item
    let item_entity = world.create_entity();
    let mut item_sprite = Sprite2d::with_color(
        "coin.png".to_string(),
        Vector2d::new(32.0, 32.0),
        Color::new(1.0, 0.9, 0.0, 1.0) // Golden color
    );
    item_sprite.set_z_order(2);
    let item_transform = Transform2dComponent::from_translation(Vector2d::new(100.0, 150.0));
    world.add_component(item_entity, item_sprite);
    world.add_component(item_entity, item_transform);
    
    // Sprite 4: Background tile
    let bg_entity = world.create_entity();
    let mut bg_sprite = Sprite2d::with_color(
        "background.png".to_string(),
        Vector2d::new(800.0, 600.0),
        Color::new(0.8, 0.8, 0.8, 0.7) // Semi-transparent background
    );
    bg_sprite.set_z_order(-1); // Behind everything
    let bg_transform = Transform2dComponent::from_translation(Vector2d::zero());
    world.add_component(bg_entity, bg_sprite);
    world.add_component(bg_entity, bg_transform);
    
    println!("Created 4 sprite entities: player, enemy, item, background");
}

/// Create various shape entities with different colors and types
fn create_shape_entities(world: &mut World) {
    // Shape 1: Circle (sun or health orb)
    let circle_entity = world.create_entity();
    let circle_shape = Shape2d::with_stroke(
        ShapeType::Circle { radius: 40.0 },
        Color::new(1.0, 0.8, 0.0, 0.8), // Golden fill
        Color::new(1.0, 0.5, 0.0, 1.0), // Orange stroke
        3.0
    );
    let circle_transform = Transform2dComponent::from_translation(Vector2d::new(-100.0, 200.0));
    world.add_component(circle_entity, circle_shape);
    world.add_component(circle_entity, circle_transform);
    
    // Shape 2: Rectangle (platform or wall)
    let rect_entity = world.create_entity();
    let mut rect_shape = Shape2d::with_stroke(
        ShapeType::Rectangle { width: 120.0, height: 20.0 },
        Color::new(0.4, 0.2, 0.1, 1.0), // Brown fill
        Color::black(),
        2.0
    );
    rect_shape.set_z_order(0);
    let rect_transform = Transform2dComponent::from_translation(Vector2d::new(50.0, -200.0));
    world.add_component(rect_entity, rect_shape);
    world.add_component(rect_entity, rect_transform);
    
    // Shape 3: Triangle (arrow or marker)
    let triangle_entity = world.create_entity();
    let triangle_shape = Shape2d::triangle(
        Vector2d::new(0.0, 30.0),   // Top vertex
        Vector2d::new(-20.0, -15.0), // Bottom left
        Vector2d::new(20.0, -15.0),  // Bottom right
        Color::green()
    );
    let mut triangle_transform = Transform2dComponent::from_translation(Vector2d::new(200.0, 100.0));
    triangle_transform.set_rotation(Angle2d::from_degrees(45.0)); // Rotate 45 degrees
    world.add_component(triangle_entity, triangle_shape);
    world.add_component(triangle_entity, triangle_transform);
    
    // Shape 4: Line (laser beam or connection)
    let line_entity = world.create_entity();
    let mut line_shape = Shape2d::line(
        Vector2d::new(-50.0, 0.0),
        Vector2d::new(50.0, 0.0),
        4.0,
        Color::new(0.0, 1.0, 1.0, 0.8) // Cyan with transparency
    );
    line_shape.set_z_order(3); // On top
    let line_transform = Transform2dComponent::from_translation(Vector2d::new(-150.0, -150.0));
    world.add_component(line_entity, line_shape);
    world.add_component(line_entity, line_transform);
    
    // Shape 5: Pentagon (special item or UI element)
    let pentagon_entity = world.create_entity();
    let pentagon_vertices = vec![
        Vector2d::new(0.0, 25.0),      // Top
        Vector2d::new(24.0, 8.0),      // Top right
        Vector2d::new(15.0, -20.0),    // Bottom right
        Vector2d::new(-15.0, -20.0),   // Bottom left
        Vector2d::new(-24.0, 8.0),     // Top left
    ];
    let pentagon_shape = Shape2d::with_stroke(
        ShapeType::Polygon { vertices: pentagon_vertices },
        Color::new(0.5, 0.0, 0.8, 0.7), // Purple fill
        Color::new(0.8, 0.0, 1.0, 1.0), // Magenta stroke
        2.0
    );
    let pentagon_transform = Transform2dComponent::from_translation(Vector2d::new(250.0, -50.0));
    world.add_component(pentagon_entity, pentagon_shape);
    world.add_component(pentagon_entity, pentagon_transform);
    
    println!("Created 5 shape entities: circle, rectangle, triangle, line, pentagon");
}

/// Run the rendering system on the sample world
pub fn render_sample_scene(world: &World) -> Result<(), Box<dyn Error>> {
    // Run the rendering2d system
    crate::rendering::Rendering2dSystem::run_with_world(world)?;
    println!("Rendered sample scene with all entities");
    Ok(())
}

/// Print statistics about the sample world
pub fn print_world_statistics(world: &World) {
    let mut camera_count = 0;
    let mut sprite_count = 0;
    let mut shape_count = 0;
    let mut transform_count = 0;
    
    // Count entities with each component type
    for (_camera, _transform) in world.iter_entities::<Camera2d, Transform2dComponent>() {
        camera_count += 1;
    }
    
    for (_sprite, _transform) in world.iter_entities::<Sprite2d, Transform2dComponent>() {
        sprite_count += 1;
    }
    
    for (_shape, _transform) in world.iter_entities::<Shape2d, Transform2dComponent>() {
        shape_count += 1;
    }
    
    println!("\n=== World Statistics ===");
    println!("Cameras: {}", camera_count);
    println!("Sprites: {}", sprite_count);
    println!("Shapes: {}", shape_count);
    println!("Total entities: {}", camera_count + sprite_count + shape_count);
    println!("========================\n");
}

/// Complete demo function that sets up everything and runs the sample
pub fn run_web_render_demo() -> Result<(), Box<dyn Error>> {
    println!("Starting Web Render Demo...\n");
    
    // Initialize the web rendering system
    initialize_web_render_system()?;
    
    // Create the sample world
    let world = create_sample_world();
    
    // Print world statistics
    print_world_statistics(&world);
    
    // Render the scene
    render_sample_scene(&world)?;
    
    println!("Web Render Demo completed successfully!");
    println!("Visit http://localhost:8080 to see the rendered scene");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_world_creation() {
        let world = create_sample_world();
        
        // Verify we have the expected entities
        let camera_count = world.iter_entities::<Camera2d, Transform2dComponent>().count();
        let sprite_count = world.iter_entities::<Sprite2d, Transform2dComponent>().count();
        let shape_count = world.iter_entities::<Shape2d, Transform2dComponent>().count();
        
        assert_eq!(camera_count, 1, "Should have exactly 1 camera");
        assert_eq!(sprite_count, 4, "Should have exactly 4 sprites");
        assert_eq!(shape_count, 5, "Should have exactly 5 shapes");
    }
    
    #[test]
    fn test_entity_components() {
        let world = create_sample_world();
        
        // Test that each sprite entity has both Sprite2d and Transform2dComponent
        for (_sprite, _transform) in world.iter_entities::<Sprite2d, Transform2dComponent>() {
            // This iteration confirms both components exist together
        }
        
        // Test that each shape entity has both Shape2d and Transform2dComponent
        for (_shape, _transform) in world.iter_entities::<Shape2d, Transform2dComponent>() {
            // This iteration confirms both components exist together
        }
    }
    
    #[test]
    fn test_z_order_variety() {
        let world = create_sample_world();
        
        let mut z_orders = Vec::new();
        
        // Collect sprite z-orders
        for (sprite, _transform) in world.iter_entities::<Sprite2d, Transform2dComponent>() {
            z_orders.push(sprite.z_order());
        }
        
        // Collect shape z-orders
        for (shape, _transform) in world.iter_entities::<Shape2d, Transform2dComponent>() {
            z_orders.push(shape.z_order());
        }
        
        // Should have a variety of z-orders including negative and positive
        z_orders.sort();
        assert!(z_orders.len() > 0);
        assert!(z_orders.contains(&-1)); // Background sprite
        assert!(z_orders.iter().any(|&z| z > 0)); // Some entities in front
    }
}