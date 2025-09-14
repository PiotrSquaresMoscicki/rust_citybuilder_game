#[cfg(test)]
mod tests {
    use crate::ecs::{World, Mut};
    use crate::examples::{Position, Velocity, Health};
    use crate::diffing::{DebugTracker, WorldState};
    use std::any::TypeId;

    /// Helper function to create a world with test entities
    fn create_test_world() -> World {
        let mut world = World::new();
        
        // Enable debug tracking
        world.debug_tracker.enable();
        
        // Create test entities
        let entity1 = world.create_entity();
        world.add_component(entity1, Position::new(10.0, 20.0));
        world.add_component(entity1, Velocity::new(1.0, 2.0));
        world.add_component(entity1, Health::new(100));
        
        let entity2 = world.create_entity();
        world.add_component(entity2, Position::new(5.0, 15.0));
        world.add_component(entity2, Velocity::new(-1.0, 1.5));
        
        let entity3 = world.create_entity();
        world.add_component(entity3, Position::new(-2.0, 8.0));
        world.add_component(entity3, Health::new(50));
        
        world
    }
    
    /// Test system that modifies position based on velocity
    fn movement_system(world: &World) {
        let ent_it = world.iter_entities::<Position, Mut<Velocity>>();
        for (position, mut velocity) in ent_it {
            // Apply some simple physics
            velocity.dx *= 0.98; // Damping
            velocity.dy *= 0.98;
            
            // Add some force based on position
            if position.x > 0.0 {
                velocity.dx -= 0.1;
            } else {
                velocity.dx += 0.1;
            }
        }
    }
    
    /// Test system that modifies position
    fn position_update_system(world: &World) {
        let velocity_iter = world.iter_entities::<Velocity, Mut<Position>>();
        for (velocity, mut position) in velocity_iter {
            position.x += velocity.dx;
            position.y += velocity.dy;
        }
    }
    
    /// Test system that modifies health
    fn health_system(world: &World) {
        let ent_it = world.iter_entities::<Position, Mut<Health>>();
        for (position, mut health) in ent_it {
            // Damage entities in negative positions
            if position.x < 0.0 || position.y < 0.0 {
                health.damage(1);
            }
        }
    }
    
    #[test]
    fn test_replay_stop_and_rewind() {
        let mut world = create_test_world();
        
        // Capture initial state
        world.capture_world_state();
        let _initial_frame = world.debug_tracker.frame_number;
        
        // Run several simulation steps with tracking
        for _frame in 1..=5 {
            world.debug_tracker.next_frame();
            
            // Run systems with debug tracking
            world.run_system_with_debug("movement_system", movement_system, &[TypeId::of::<Velocity>()]);
            world.run_system_with_debug("position_update_system", position_update_system, &[TypeId::of::<Position>()]);
            world.run_system_with_debug("health_system", health_system, &[TypeId::of::<Health>()]);
            
            // Capture state every frame
            world.capture_world_state();
        }
        
        // Store final state for comparison
        let entity1 = world.get_entities()[0];
        let final_position = {
            let pos = world.get_component::<Position>(entity1).unwrap();
            (pos.x, pos.y)
        }; // Drop the borrow here
        
        // Rewind to frame 2
        let target_frame = 2;
        let success = world.restore_world_state(target_frame);
        assert!(success, "Failed to restore world state to frame {}", target_frame);
        
        // Verify we're back at frame 2 state
        let rewound_position = {
            let pos = world.get_component::<Position>(entity1).unwrap();
            (pos.x, pos.y)
        };
        
        // Position should be different from final state
        assert_ne!(rewound_position, final_position, 
                   "Position should be different after rewind");
        
        // Continue simulation from rewound state
        world.debug_tracker.frame_number = target_frame; // Reset frame counter
        for _frame in (target_frame + 1)..=5 {
            world.debug_tracker.next_frame();
            
            world.run_system_with_debug("movement_system", movement_system, &[TypeId::of::<Velocity>()]);
            world.run_system_with_debug("position_update_system", position_update_system, &[TypeId::of::<Position>()]);
            world.run_system_with_debug("health_system", health_system, &[TypeId::of::<Health>()]);
        }
        
        // Final position should match original final position (deterministic simulation)
        let replay_final_position = {
            let pos = world.get_component::<Position>(entity1).unwrap();
            (pos.x, pos.y)
        };
        
        // Due to deterministic nature, positions should be very close
        let tolerance = 0.001;
        assert!((replay_final_position.0 - final_position.0).abs() < tolerance,
                "X position mismatch: {} vs {}", replay_final_position.0, final_position.0);
        assert!((replay_final_position.1 - final_position.1).abs() < tolerance,
                "Y position mismatch: {} vs {}", replay_final_position.1, final_position.1);
    }
    
    #[test]
    fn test_replay_in_new_world() {
        // Create baseline world and run simulation
        let mut baseline_world = create_test_world();
        
        // Capture initial state
        baseline_world.capture_world_state();
        
        // Clone initial state info
        let initial_entities = baseline_world.get_entities().clone();
        
        // Run baseline simulation for 3 frames
        for _frame in 1..=3 {
            baseline_world.debug_tracker.next_frame();
            
            baseline_world.run_system_with_debug("movement_system", movement_system, &[TypeId::of::<Velocity>()]);
            baseline_world.run_system_with_debug("position_update_system", position_update_system, &[TypeId::of::<Position>()]);
            baseline_world.run_system_with_debug("health_system", health_system, &[TypeId::of::<Health>()]);
            
            // Capture state after each frame
            baseline_world.capture_world_state();
        }
        
        // Continue baseline simulation for 2 more frames
        for _frame in 4..=5 {
            baseline_world.debug_tracker.next_frame();
            
            baseline_world.run_system_with_debug("movement_system", movement_system, &[TypeId::of::<Velocity>()]);
            baseline_world.run_system_with_debug("position_update_system", position_update_system, &[TypeId::of::<Position>()]);
            baseline_world.run_system_with_debug("health_system", health_system, &[TypeId::of::<Health>()]);
        }
        
        // Store baseline final state
        let entity1 = initial_entities[0];
        let baseline_final_position = {
            let pos = baseline_world.get_component::<Position>(entity1).unwrap();
            (pos.x, pos.y)
        };
        
        // Store diff history and world states for replay
        let diff_history = baseline_world.debug_tracker.diff_history.clone();
        let world_states = baseline_world.debug_tracker.world_states.clone();
        
        // Create new world and replay changes up to frame 3
        let mut new_world = create_test_world();
        new_world.debug_tracker.enable();
        
        // Copy the diff history and world states from baseline
        new_world.debug_tracker.diff_history = diff_history;
        new_world.debug_tracker.world_states = world_states;
        
        // Replay up to frame 3
        let replay_success = new_world.replay_to_frame(3);
        assert!(replay_success, "Failed to replay to frame 3");
        
        // Set frame number to continue simulation
        new_world.debug_tracker.frame_number = 3;
        
        // Continue simulation from frame 3 to frame 5
        for _frame in 4..=5 {
            new_world.debug_tracker.next_frame();
            
            new_world.run_system_with_debug("movement_system", movement_system, &[TypeId::of::<Velocity>()]);
            new_world.run_system_with_debug("position_update_system", position_update_system, &[TypeId::of::<Position>()]);
            new_world.run_system_with_debug("health_system", health_system, &[TypeId::of::<Health>()]);
        }
        
        // Compare final states
        let new_world_entity1 = new_world.get_entities()[0];
        let new_world_final_position = {
            let pos = new_world.get_component::<Position>(new_world_entity1).unwrap();
            (pos.x, pos.y)
        };
        
        // Final states should match (deterministic simulation)
        let tolerance = 0.001;
        assert!((new_world_final_position.0 - baseline_final_position.0).abs() < tolerance,
                "X position mismatch: {} vs {}", new_world_final_position.0, baseline_final_position.0);
        assert!((new_world_final_position.1 - baseline_final_position.1).abs() < tolerance,
                "Y position mismatch: {} vs {}", new_world_final_position.1, baseline_final_position.1);
    }
    
    #[test]
    fn test_world_state_capture_and_restore() {
        let mut world = create_test_world();
        
        // Get initial values
        let entity1 = world.get_entities()[0];
        let initial_value = {
            let pos = world.get_component::<Position>(entity1).unwrap();
            (pos.x, pos.y)
        };
        
        // Capture initial state after getting values
        world.capture_world_state();
        
        // Modify the world
        {
            let mut position = world.get_component_mut::<Position>(entity1).unwrap();
            position.x = 999.0;
            position.y = 888.0;
        }
        
        // Verify modification
        let modified_value = {
            let pos = world.get_component::<Position>(entity1).unwrap();
            (pos.x, pos.y)
        };
        assert_eq!(modified_value, (999.0, 888.0));
        
        // Restore to initial state
        let restore_success = world.restore_world_state(0);
        assert!(restore_success, "Failed to restore world state");
        
        // Verify restoration
        let restored_value = {
            let pos = world.get_component::<Position>(entity1).unwrap();
            (pos.x, pos.y)
        };
        
        assert_eq!(restored_value, initial_value, 
                   "Position should be restored to initial value");
    }
    
    #[test]
    fn test_diff_recording_and_replay() {
        let mut world = create_test_world();
        
        // Capture initial state first
        world.capture_world_state();
        
        // Run a system and record diffs
        world.debug_tracker.next_frame();
        world.run_system_with_debug("movement_system", movement_system, &[TypeId::of::<Velocity>()]);
        
        // Verify that diffs were recorded
        assert!(!world.debug_tracker.diff_history.is_empty(), 
                "Diff history should not be empty after running system");
        
        let diff_record = &world.debug_tracker.diff_history[0];
        assert_eq!(diff_record.system_name, "movement_system");
        assert_eq!(diff_record.frame_number, 1);
        assert!(!diff_record.component_diffs.is_empty(), 
                "Component diffs should not be empty");
        
        // Check that velocity components were modified
        let velocity_diffs: Vec<_> = diff_record.component_diffs.iter()
            .filter(|d| d.component_type == "Velocity")
            .collect();
        assert!(!velocity_diffs.is_empty(), "Should have velocity diffs");
        
        // Print diff history for debugging
        let formatted_history = world.debug_tracker.get_diff_history_formatted();
        println!("Diff history:\n{}", formatted_history);
    }
}