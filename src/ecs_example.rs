use crate::ecs_new::*;
use std::any::Any;

// Example components for demonstration
#[derive(Clone, Debug)]
pub struct PositionComponent {
    pub x: f32,
    pub y: f32,
}

impl Component for PositionComponent {
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

#[derive(Clone, Debug)]
pub struct VelocityComponent {
    pub dx: f32,
    pub dy: f32,
}

impl Component for VelocityComponent {
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

#[derive(Clone, Debug)]
pub struct TimeComponent {
    pub delta_time: f32,
}

impl Component for TimeComponent {
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

#[derive(Clone, Debug)]
pub struct PhysicsComponent {
    pub mass: f32,
}

impl Component for PhysicsComponent {
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

#[derive(Clone, Debug)]
pub struct InputComponent {
    pub keys_pressed: Vec<String>,
}

impl Component for InputComponent {
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

#[derive(Clone, Debug)]
pub struct SampleConfigComponent {
    pub config_value: i32,
}

impl Component for SampleConfigComponent {
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

// System marker implementations for dependency tracking
pub struct TimeSystem;
impl SystemMarker for TimeSystem {
    fn name() -> &'static str { "TimeSystem" }
}

pub struct InputSystem;
impl SystemMarker for InputSystem {
    fn name() -> &'static str { "InputSystem" }
}

pub struct PhysicsSystem;
impl SystemMarker for PhysicsSystem {
    fn name() -> &'static str { "PhysicsSystem" }
}

// Example system implementation matching the target design
pub struct SampleSystem;

impl System for SampleSystem {
    type Dependencies = (TimeSystem, InputSystem, PhysicsSystem);
    type Iterators = (
        EntIt2<Mut<PositionComponent>, VelocityComponent>,
        EntIt3<TimeComponent, PhysicsComponent, InputComponent>,
    );

    fn update(&mut self, iterators: Self::Iterators) {
        let (position_velocity_iter, time_physics_input_iter) = iterators;
        
        // Implementation of the update logic
        println!("SampleSystem::update called with multiple iterators");
        
        // Since iterators return Entity IDs, we need access to the world to get components
        // This is a limitation of the current simplified design
        // In the complete implementation, iterators would return component references directly
        
        // Count entities with position and velocity
        let mut position_velocity_count = 0;
        for _entity in position_velocity_iter {
            position_velocity_count += 1;
        }
        println!("Found {} entities with Position and Velocity components", position_velocity_count);
        
        // Count entities with time, physics, and input
        let mut time_physics_input_count = 0;
        for _entity in time_physics_input_iter {
            time_physics_input_count += 1;
        }
        println!("Found {} entities with Time, Physics, and Input components", time_physics_input_count);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_ecs_design() {
        let mut world = World::new();
        
        // Create an entity with components
        let entity = world.create_entity();
        world.add_component(entity, PositionComponent { x: 0.0, y: 0.0 });
        world.add_component(entity, VelocityComponent { dx: 1.0, dy: 2.0 });
        
        // Create a time entity
        let time_entity = world.create_entity();
        world.add_component(time_entity, TimeComponent { delta_time: 0.016 });
        world.add_component(time_entity, PhysicsComponent { mass: 1.0 });
        world.add_component(time_entity, InputComponent { keys_pressed: vec![] });
        
        // Test component access
        assert!(world.has_component::<PositionComponent>(entity));
        assert!(world.has_component::<VelocityComponent>(entity));
        assert!(world.has_component::<TimeComponent>(time_entity));
        
        // Test component retrieval
        if let Some(pos) = world.get_component::<PositionComponent>(entity) {
            assert_eq!(pos.x, 0.0);
            assert_eq!(pos.y, 0.0);
        }
        
        // Test iterators
        let iter1 = world.iter_entities_2::<Mut<PositionComponent>, VelocityComponent>();
        let iter2 = world.iter_entities_3::<TimeComponent, PhysicsComponent, InputComponent>();
        
        // Create a sample system
        let mut sample_system = SampleSystem;
        
        // This demonstrates the target API design
        sample_system.update((iter1, iter2));
    }
}