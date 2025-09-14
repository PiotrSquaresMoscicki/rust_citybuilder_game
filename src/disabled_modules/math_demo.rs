use crate::ecs::{World, Entity};
use crate::core::math::{Vector2d, Angle2d, Transform2dComponent};

/// Demonstrates the math library usage with the ECS system
pub fn demonstrate_math_library() {
    println!("Math Library Demonstration");
    println!("=========================");
    
    let mut world = World::new();
    
    // Create some entities with transforms
    let entity1 = world.create_entity();
    let entity2 = world.create_entity();
    let entity3 = world.create_entity();
    
    // Add transform components with different configurations
    world.add_component(entity1, Transform2dComponent::from_translation(Vector2d::new(10.0, 20.0)));
    world.add_component(entity2, Transform2dComponent::from_rotation(Angle2d::from_degrees(45.0)));
    world.add_component(entity3, Transform2dComponent::from_trs(
        Vector2d::new(5.0, 5.0),
        Angle2d::from_degrees(90.0),
        2.0
    ));
    
    println!("\nInitial Transform States:");
    print_transform_info(&world, entity1, "Entity 1 (Translation only)");
    print_transform_info(&world, entity2, "Entity 2 (Rotation only)");
    print_transform_info(&world, entity3, "Entity 3 (Full TRS)");
    
    // Demonstrate vector operations
    println!("\nVector Operations:");
    let v1 = Vector2d::new(3.0, 4.0);
    let v2 = Vector2d::new(1.0, 2.0);
    println!("v1 = {:?}, v2 = {:?}", v1, v2);
    println!("v1 + v2 = {:?}", v1 + v2);
    println!("v1 magnitude = {:.2}", v1.magnitude());
    println!("v1 normalized = {:?}", v1.normalized());
    println!("v1 dot v2 = {:.2}", v1.dot(&v2));
    
    // Demonstrate angle operations
    println!("\nAngle Operations:");
    let angle1 = Angle2d::from_degrees(30.0);
    let angle2 = Angle2d::from_degrees(60.0);
    println!("angle1 = {:.1}°, angle2 = {:.1}°", angle1.degrees(), angle2.degrees());
    println!("angle1 + angle2 = {:.1}°", (angle1 + angle2).degrees());
    println!("sin(30°) = {:.3}", angle1.sin());
    println!("cos(30°) = {:.3}", angle1.cos());
    
    // Demonstrate transform operations on entities
    println!("\nTransform Operations:");
    
    // Move entity1
    if let Some(mut transform) = world.get_component_mut::<Transform2dComponent>(entity1) {
        transform.translate(Vector2d::new(5.0, 5.0));
        println!("Moved Entity 1 by (5, 5)");
    }
    
    // Rotate entity2
    if let Some(mut transform) = world.get_component_mut::<Transform2dComponent>(entity2) {
        transform.rotate(Angle2d::from_degrees(45.0));
        println!("Rotated Entity 2 by 45°");
    }
    
    // Scale entity3
    if let Some(mut transform) = world.get_component_mut::<Transform2dComponent>(entity3) {
        transform.scale_by(0.5);
        println!("Scaled Entity 3 by 0.5");
    }
    
    println!("\nUpdated Transform States:");
    print_transform_info(&world, entity1, "Entity 1 (After translation)");
    print_transform_info(&world, entity2, "Entity 2 (After rotation)");
    print_transform_info(&world, entity3, "Entity 3 (After scaling)");
    
    // Demonstrate point transformation
    println!("\nPoint Transformation:");
    let test_point = Vector2d::new(1.0, 0.0);
    println!("Original point: {:?}", test_point);
    
    if let Some(transform) = world.get_component::<Transform2dComponent>(entity3) {
        let transformed_point = transform.transform_point(test_point);
        println!("Transformed by Entity 3: {:?}", transformed_point);
        
        let forward = transform.forward();
        let right = transform.right();
        println!("Entity 3 forward direction: {:?}", forward);
        println!("Entity 3 right direction: {:?}", right);
    }
    
    // Demonstrate look-at functionality
    println!("\nLook-At Demonstration:");
    if let Some(mut transform) = world.get_component_mut::<Transform2dComponent>(entity1) {
        let target = Vector2d::new(100.0, 100.0);
        transform.look_at(target);
        println!("Entity 1 now looking at {:?}", target);
        println!("Entity 1 rotation: {:.1}°", transform.rotation().degrees());
    }
    
    println!("\nMath library demonstration complete!\n");
}

fn print_transform_info(world: &World, entity: Entity, label: &str) {
    if let Some(transform) = world.get_component::<Transform2dComponent>(entity) {
        println!("{}: pos=({:.1}, {:.1}), rot={:.1}°, scale={:.2}",
            label,
            transform.translation().x,
            transform.translation().y,
            transform.rotation().degrees(),
            transform.scale()
        );
    }
}