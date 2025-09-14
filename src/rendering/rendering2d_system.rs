use crate::ecs::{World, Entity, EntityIterator};
use crate::core::math::{Camera2d, Sprite2d, Shape2d, Transform2dComponent, Transform2d};
use crate::rendering::{RenderCommand, get_global_rendering_manager};
use std::error::Error;

/// Data structure for visible entities that need to be rendered
#[derive(Debug, Clone)]
pub struct RenderableEntity {
    pub entity: Entity,
    pub transform: Transform2d,
    pub z_order: i32,
}

/// Data structure for visible sprites
#[derive(Debug, Clone)]
pub struct VisibleSprite {
    pub entity: Entity,
    pub transform: Transform2d,
    pub sprite: Sprite2d,
}

/// Data structure for visible shapes
#[derive(Debug, Clone)]
pub struct VisibleShape {
    pub entity: Entity,
    pub transform: Transform2d,
    pub shape: Shape2d,
}

/// The Rendering2D system that handles 2D rendering
/// This system finds all entities with Sprite2d or Shape2d components,
/// performs culling based on camera view, transforms them using the Camera2d,
/// and sends visible entities to the rendering manager.
pub struct Rendering2dSystem;

impl Rendering2dSystem {
    /// Execute the rendering system
    /// For now, we assume Camera2d entities also have Transform2dComponent
    pub fn execute(
        camera_iter: EntityIterator<Camera2d, Transform2dComponent>,
        sprite_iter: EntityIterator<Sprite2d, Transform2dComponent>,
        shape_iter: EntityIterator<Shape2d, Transform2dComponent>,
    ) -> Result<(), Box<dyn Error>> {
        // For now, we only support one camera component
        let camera_data = Self::find_camera(camera_iter)?;
        let (camera_entity, camera) = camera_data;

        // Collect visible sprites
        let visible_sprites = Self::cull_sprites(sprite_iter, &camera);
        
        // Collect visible shapes
        let visible_shapes = Self::cull_shapes(shape_iter, &camera);

        // Send rendering commands to the rendering manager
        Self::render_entities(visible_sprites, visible_shapes, &camera)?;

        Ok(())
    }

    /// Find the first (and for now, only) camera in the scene
    fn find_camera(mut camera_iter: EntityIterator<Camera2d, Transform2dComponent>) -> Result<(Entity, Camera2d, Transform2dComponent), Box<dyn Error>> {
        if let Some((camera, transform)) = camera_iter.next() {
            Ok((0, camera.clone(), transform.clone())) // Entity ID not available in iterator
        } else {
            Err("No camera found in the scene".into())
        }
    }

    /// Perform culling on sprites based on camera view
    fn cull_sprites(sprite_iter: EntityIterator<Sprite2d, Transform2dComponent>, camera: &Camera2d) -> Vec<VisibleSprite> {
        let mut visible_sprites = Vec::new();

        for (sprite, transform_component) in sprite_iter {
            if !sprite.is_visible() {
                continue;
            }

            let world_position = transform_component.translation();
            let (sprite_width, sprite_height) = sprite.bounding_box();
            
            // Check if sprite is visible in camera view
            if camera.is_rect_visible(world_position, sprite_width, sprite_height) {
                // Transform the sprite position using camera view
                let view_transform = camera.view_transform();
                let transformed = view_transform * transform_component.transform();
                
                visible_sprites.push(VisibleSprite {
                    entity: 0, // We don't have access to entity ID in this iterator pattern
                    transform: transformed,
                    sprite: sprite.clone(),
                });
            }
        }

        // Sort by z-order (back to front)
        visible_sprites.sort_by_key(|s| s.sprite.z_order());
        visible_sprites
    }

    /// Perform culling on shapes based on camera view
    fn cull_shapes(shape_iter: EntityIterator<Shape2d, Transform2dComponent>, camera: &Camera2d) -> Vec<VisibleShape> {
        let mut visible_shapes = Vec::new();

        for (shape, transform_component) in shape_iter {
            if !shape.is_visible() {
                continue;
            }

            let world_position = transform_component.translation();
            let (shape_width, shape_height) = shape.bounding_box();
            
            // Check if shape is visible in camera view
            if camera.is_rect_visible(world_position, shape_width, shape_height) {
                // Transform the shape position using camera view
                let view_transform = camera.view_transform();
                let transformed = view_transform * transform_component.transform();
                
                visible_shapes.push(VisibleShape {
                    entity: 0, // We don't have access to entity ID in this iterator pattern
                    transform: transformed,
                    shape: shape.clone(),
                });
            }
        }

        // Sort by z-order (back to front)
        visible_shapes.sort_by_key(|s| s.shape.z_order());
        visible_shapes
    }

    /// Send rendering commands to the rendering manager
    fn render_entities(
        visible_sprites: Vec<VisibleSprite>,
        visible_shapes: Vec<VisibleShape>,
        camera: &Camera2d,
    ) -> Result<(), Box<dyn Error>> {
        let manager_arc = get_global_rendering_manager()?;
        let manager = manager_arc.lock().map_err(|e| format!("Failed to lock rendering manager: {}", e))?;

        // Clear the screen first
        let clear_command = RenderCommand::Clear { r: 0.2, g: 0.2, b: 0.2, a: 1.0 };
        manager.execute_command(clear_command)?;

        // Combine sprites and shapes into a single sorted list
        let mut all_renderables: Vec<(i32, RenderCommand)> = Vec::new();

        // Add sprite commands
        for visible_sprite in visible_sprites {
            let command = RenderCommand::DrawSprite {
                texture_id: visible_sprite.sprite.texture_id().to_string(),
                transform: visible_sprite.transform,
                size: visible_sprite.sprite.size(),
                color: visible_sprite.sprite.color(),
                z_order: visible_sprite.sprite.z_order(),
                uv_rect: visible_sprite.sprite.uv_rect(),
            };
            all_renderables.push((visible_sprite.sprite.z_order(), command));
        }

        // Add shape commands
        for visible_shape in visible_shapes {
            let command = RenderCommand::DrawShape {
                shape_type: visible_shape.shape.shape_type().clone(),
                transform: visible_shape.transform,
                fill: visible_shape.shape.fill().clone(),
                stroke: visible_shape.shape.stroke().cloned(),
                z_order: visible_shape.shape.z_order(),
            };
            all_renderables.push((visible_shape.shape.z_order(), command));
        }

        // Sort by z-order and execute commands
        all_renderables.sort_by_key(|(z_order, _)| *z_order);
        
        for (_, command) in all_renderables {
            manager.execute_command(command)?;
        }

        Ok(())
    }

    /// Convenience function to run the rendering system with a World reference
    pub fn run_with_world(world: &World) -> Result<(), Box<dyn Error>> {
        let camera_iter = world.iter_entities::<Camera2d, Transform2dComponent>();
        let sprite_iter = world.iter_entities::<Sprite2d, Transform2dComponent>();
        let shape_iter = world.iter_entities::<Shape2d, Transform2dComponent>();
        
        Self::execute(camera_iter, sprite_iter, shape_iter)
    }
}

/// System function compatible with the ECS framework
/// This version uses multiple entity iterators as the system signature
pub fn rendering2d_system(
    camera_iter: EntityIterator<Camera2d, Transform2dComponent>,
    sprite_iter: EntityIterator<Sprite2d, Transform2dComponent>,
    shape_iter: EntityIterator<Shape2d, Transform2dComponent>,
) -> Result<(), Box<dyn Error>> {
    Rendering2dSystem::execute(camera_iter, sprite_iter, shape_iter)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::math::{Color, ShapeType, FillStyle, Angle2d};

    fn create_test_world_with_entities() -> World {
        let mut world = World::new();

        // Create camera entity
        let camera_entity = world.create_entity();
        let mut camera = Camera2d::new();
        camera.set_view_dimensions(800.0, 600.0);
        let camera_transform = Transform2dComponent::new();
        world.add_component(camera_entity, camera);
        world.add_component(camera_entity, camera_transform);

        // Create sprite entity
        let sprite_entity = world.create_entity();
        let sprite = Sprite2d::new("test_texture".to_string(), crate::core::math::Vector2d::new(64.0, 64.0));
        let transform = Transform2dComponent::from_translation(crate::core::math::Vector2d::new(100.0, 100.0));
        world.add_component(sprite_entity, sprite);
        world.add_component(sprite_entity, transform);

        // Create shape entity
        let shape_entity = world.create_entity();
        let shape = Shape2d::circle(32.0, Color::red());
        let shape_transform = Transform2dComponent::from_translation(crate::core::math::Vector2d::new(200.0, 150.0));
        world.add_component(shape_entity, shape);
        world.add_component(shape_entity, shape_transform);

        world
    }

    #[test]
    fn test_camera_finding() {
        let world = create_test_world_with_entities();
        let camera_iter = world.iter_entities::<Camera2d, Transform2dComponent>();
        
        let result = Rendering2dSystem::find_camera(camera_iter);
        assert!(result.is_ok());
    }

    #[test]
    fn test_sprite_culling() {
        let world = create_test_world_with_entities();
        let camera_iter = world.iter_entities::<Camera2d, Transform2dComponent>();
        let sprite_iter = world.iter_entities::<Sprite2d, Transform2dComponent>();
        
        let (_, camera) = Rendering2dSystem::find_camera(camera_iter).unwrap();
        let visible_sprites = Rendering2dSystem::cull_sprites(sprite_iter, &camera);
        
        // Should have at least one visible sprite
        assert!(!visible_sprites.is_empty());
    }

    #[test]
    fn test_shape_culling() {
        let world = create_test_world_with_entities();
        let camera_iter = world.iter_entities::<Camera2d, Transform2dComponent>();
        let shape_iter = world.iter_entities::<Shape2d, Transform2dComponent>();
        
        let (_, camera) = Rendering2dSystem::find_camera(camera_iter).unwrap();
        let visible_shapes = Rendering2dSystem::cull_shapes(shape_iter, &camera);
        
        // Should have at least one visible shape
        assert!(!visible_shapes.is_empty());
    }

    #[test]
    fn test_z_order_sorting() {
        let mut world = World::new();

        // Create camera
        let camera_entity = world.create_entity();
        let camera = Camera2d::new();
        let camera_transform = Transform2dComponent::new();
        world.add_component(camera_entity, camera);
        world.add_component(camera_entity, camera_transform);

        // Create multiple sprites with different z-orders
        for i in 0..3 {
            let entity = world.create_entity();
            let mut sprite = Sprite2d::new("test".to_string(), crate::core::math::Vector2d::new(32.0, 32.0));
            sprite.set_z_order(2 - i); // Z-orders: 2, 1, 0
            let transform = Transform2dComponent::from_translation(crate::core::math::Vector2d::new(50.0, 50.0));
            world.add_component(entity, sprite);
            world.add_component(entity, transform);
        }

        let camera_iter = world.iter_entities::<Camera2d, Transform2dComponent>();
        let sprite_iter = world.iter_entities::<Sprite2d, Transform2dComponent>();
        
        let (_, camera) = Rendering2dSystem::find_camera(camera_iter).unwrap();
        let visible_sprites = Rendering2dSystem::cull_sprites(sprite_iter, &camera);
        
        // Check that sprites are sorted by z-order
        for i in 1..visible_sprites.len() {
            assert!(visible_sprites[i-1].sprite.z_order() <= visible_sprites[i].sprite.z_order());
        }
    }
}