use crate::ecs::{World, EntityIterator, Mut, SingleIteratorSystem};
use crate::core::time::{TimeComponent, get_time_manager};

/// Time system that updates time components with delta time from the global time manager
pub fn time_system(time_iter: EntityIterator<Mut<TimeComponent>, Mut<TimeComponent>>) {
    // Get delta time from global time manager
    let delta_time = if let Some(manager) = get_time_manager() {
        manager.delta_time_seconds()
    } else {
        0.0 // Fallback if time manager not initialized
    };

    // Update all time components
    for (mut time_component, _) in time_iter {
        time_component.update(delta_time);
    }
}

/// Alternative time system for entities with only TimeComponent (no second component)
/// This is a helper function since the current ECS requires two components
pub fn update_time_components_in_world(world: &World) {
    if let Some(manager) = get_time_manager() {
        let delta_time = manager.delta_time_seconds();
        
        // Get all entities with TimeComponent
        let entities_with_time = world.entities_with_component::<TimeComponent>();
        
        for entity in entities_with_time {
            if let Some(mut time_comp) = world.get_component_mut::<TimeComponent>(entity) {
                time_comp.update(delta_time);
            }
        }
    }
}

/// Create a time system that can be added to the world
pub fn create_time_system() -> SingleIteratorSystem<Mut<TimeComponent>, Mut<TimeComponent>, impl Fn(EntityIterator<Mut<TimeComponent>, Mut<TimeComponent>>)> {
    SingleIteratorSystem::new(time_system, "time_system".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::time::{initialize_time_manager, update_global_time_manager};
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_time_system_updates_components() {
        // Initialize global time manager
        initialize_time_manager();
        
        // Create world and add time component
        let mut world = World::new();
        let entity = world.create_entity();
        world.add_component(entity, TimeComponent::new());
        
        // Simulate some time passing
        sleep(Duration::from_millis(1));
        update_global_time_manager();
        
        // Update time components
        update_time_components_in_world(&world);
        
        // Check that time component was updated
        {
            let time_comp = world.get_component::<TimeComponent>(entity);
            if let Some(time_comp) = time_comp {
                assert!(time_comp.delta_time > 0.0);
                assert!(time_comp.total_time > 0.0);
                assert_eq!(time_comp.frame_count, 1);
            } else {
                panic!("Time component should exist");
            }
        }
    }

    #[test]
    fn test_time_system_with_multiple_entities() {
        initialize_time_manager();
        
        let mut world = World::new();
        
        // Create multiple entities with time components
        let entity1 = world.create_entity();
        let entity2 = world.create_entity();
        world.add_component(entity1, TimeComponent::new());
        world.add_component(entity2, TimeComponent::new());
        
        sleep(Duration::from_millis(1));
        update_global_time_manager();
        
        update_time_components_in_world(&world);
        
        // Check both components were updated
        for &entity in &[entity1, entity2] {
            let time_comp = world.get_component::<TimeComponent>(entity);
            if let Some(time_comp) = time_comp {
                assert!(time_comp.delta_time > 0.0);
                assert!(time_comp.total_time > 0.0);
                assert_eq!(time_comp.frame_count, 1);
            } else {
                panic!("Time component should exist for entity {}", entity);
            }
        }
    }

    #[test]
    fn test_time_system_with_paused_component() {
        initialize_time_manager();
        
        let mut world = World::new();
        let entity = world.create_entity();
        let mut time_comp = TimeComponent::new();
        time_comp.pause();
        world.add_component(entity, time_comp);
        
        sleep(Duration::from_millis(1));
        update_global_time_manager();
        
        update_time_components_in_world(&world);
        
        {
            let time_comp = world.get_component::<TimeComponent>(entity);
            if let Some(time_comp) = time_comp {
                assert!(time_comp.delta_time > 0.0); // Delta time is still recorded
                assert_eq!(time_comp.total_time, 0.0); // But total time doesn't advance when paused
                assert_eq!(time_comp.frame_count, 1);
                assert!(time_comp.is_paused);
            } else {
                panic!("Time component should exist");
            }
        }
    }

    #[test]
    fn test_time_system_with_time_scale() {
        initialize_time_manager();
        
        let mut world = World::new();
        let entity = world.create_entity();
        let mut time_comp = TimeComponent::new();
        time_comp.set_time_scale(2.0);
        world.add_component(entity, time_comp);
        
        sleep(Duration::from_millis(1));
        update_global_time_manager();
        
        update_time_components_in_world(&world);
        
        {
            let time_comp = world.get_component::<TimeComponent>(entity);
            if let Some(time_comp) = time_comp {
                assert!(time_comp.delta_time > 0.0);
                assert!(time_comp.total_time > time_comp.delta_time); // Should be 2x the delta time
                assert_eq!(time_comp.time_scale, 2.0);
            } else {
                panic!("Time component should exist");
            }
        }
    }
}