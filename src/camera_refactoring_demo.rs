/// Test to demonstrate the Camera2d refactoring works correctly
#[cfg(test)]
mod camera_refactoring_demo {
    use crate::ecs::World;
    use crate::core::math::{Camera2d, Transform2dComponent, Vector2d, Angle2d};

    #[test]
    fn test_camera_refactoring_demo() {
        let mut world = World::new();
        
        // Create a camera entity with both Camera2d and Transform2dComponent
        let camera_entity = world.create_entity();
        
        // Camera2d now only contains scale and view dimensions
        let mut camera = Camera2d::new();
        camera.set_scale(2.0);
        camera.set_view_dimensions(800.0, 600.0);
        
        // Transform2dComponent handles position and rotation
        let mut camera_transform = Transform2dComponent::from_translation(Vector2d::new(100.0, 50.0));
        camera_transform.set_rotation(Angle2d::from_degrees(45.0));
        
        world.add_component(camera_entity, camera.clone());
        world.add_component(camera_entity, camera_transform.clone());
        
        // The camera methods now take position and rotation from the transform component
        let camera_position = camera_transform.translation();
        let camera_rotation = camera_transform.rotation();
        
        // Test world to camera transformation
        let world_point = Vector2d::new(150.0, 100.0);
        let camera_point = camera.world_to_camera(world_point, camera_position, camera_rotation);
        
        // Test camera to world transformation
        let back_to_world = camera.camera_to_world(camera_point, camera_position, camera_rotation);
        
        // Verify the transformations work (should get back close to original point)
        let diff = (world_point - back_to_world).magnitude();
        assert!(diff < 0.001, "Transform round-trip failed: diff = {}", diff);
        
        // Verify camera properties
        assert!((camera.scale() - 2.0).abs() < 0.001);
        assert!((camera_position.x - 100.0).abs() < 0.001);
        assert!((camera_position.y - 50.0).abs() < 0.001);
        assert!((camera_rotation.degrees() - 45.0).abs() < 0.001);
        
        // Test visibility check
        let test_point = Vector2d::new(120.0, 80.0);
        let _is_visible = camera.is_point_visible(test_point, camera_position, camera_rotation);
        
        println!("✅ Camera refactoring demo successful!");
        println!("   - Camera scale: {}", camera.scale());
        println!("   - Camera position from transform: {:?}", camera_position);
        println!("   - Camera rotation from transform: {:.2}°", camera_rotation.degrees());
        println!("   - Transform round-trip error: {:.6}", diff);
    }
}