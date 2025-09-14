use crate::ecs::{Component, World, EntityIterator, Mut};
use std::any::Any;
use serde::{Serialize, Deserialize};

/// Position component for entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl Position {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    
    pub fn set_position(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }
    
    pub fn get_position(&self) -> (f32, f32) {
        (self.x, self.y)
    }
}

impl Component for Position {
    fn validate(&self) -> bool {
        self.x.is_finite() && self.y.is_finite()
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

/// Velocity component for entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Velocity {
    pub dx: f32,
    pub dy: f32,
}

impl Velocity {
    pub fn new(dx: f32, dy: f32) -> Self {
        Self { dx, dy }
    }
    
    pub fn set_velocity(&mut self, dx: f32, dy: f32) {
        self.dx = dx;
        self.dy = dy;
    }
    
    pub fn get_velocity(&self) -> (f32, f32) {
        (self.dx, self.dy)
    }
}

impl Component for Velocity {
    fn validate(&self) -> bool {
        self.dx.is_finite() && self.dy.is_finite()
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

/// Health component for entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Health {
    pub current: i32,
    pub max: i32,
}

impl Health {
    pub fn new(max: i32) -> Self {
        Self {
            current: max,
            max,
        }
    }
    
    pub fn damage(&mut self, amount: i32) {
        self.current = (self.current - amount).max(0);
    }
    
    pub fn heal(&mut self, amount: i32) {
        self.current = (self.current + amount).min(self.max);
    }
    
    pub fn is_alive(&self) -> bool {
        self.current > 0
    }
    
    pub fn health_percentage(&self) -> f32 {
        if self.max > 0 {
            self.current as f32 / self.max as f32
        } else {
            0.0
        }
    }
}

impl Component for Health {
    fn validate(&self) -> bool {
        self.max >= 0 && self.current >= 0 && self.current <= self.max
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

/// Example system function that demonstrates the required API
/// This matches: system_func(ent_it: EntityIterator<ComponentType1, Mut<ComponentType2>>)
/// where Position is immutable and Mut<Velocity> is mutable
pub fn movement_system(ent_it: EntityIterator<Position, Mut<Velocity>>) {
    for (position, mut velocity) in ent_it {
        println!("Entity at ({}, {}) with velocity ({}, {})", 
                 position.x, position.y, velocity.dx, velocity.dy);
        
        // Demonstrate mutable access - apply some damping to velocity
        velocity.dx *= 0.99;
        velocity.dy *= 0.99;
        
        println!("  Applied damping, new velocity: ({}, {})", velocity.dx, velocity.dy);
    }
}

/// Another example system that modifies health components
pub fn health_damage_system(ent_it: EntityIterator<Position, Mut<Health>>) {
    for (position, mut health) in ent_it {
        // Entities take damage based on their position (example logic)
        let damage = if position.x < 0.0 || position.y < 0.0 { 5 } else { 1 };
        
        println!("Entity at ({}, {}) taking {} damage", position.x, position.y, damage);
        health.damage(damage);
        
        if health.is_alive() {
            println!("  Health: {}/{} ({}%)", 
                     health.current, health.max, 
                     (health.health_percentage() * 100.0) as i32);
        } else {
            println!("  Entity died!");
        }
    }
}

/// Helper function to create a simple example world
pub fn create_example_world() -> World {
    let mut world = World::new();
    
    // Create some entities with components
    let entity1 = world.create_entity();
    world.add_component(entity1, Position::new(10.0, 20.0));
    world.add_component(entity1, Velocity::new(1.5, -0.5));
    world.add_component(entity1, Health::new(100));
    
    let entity2 = world.create_entity();
    world.add_component(entity2, Position::new(5.0, 15.0));
    world.add_component(entity2, Velocity::new(-1.0, 2.0));
    
    let entity3 = world.create_entity();
    world.add_component(entity3, Position::new(-2.0, 8.0));
    world.add_component(entity3, Health::new(50));
    
    world
}

// Make Position diffable
crate::diffable!(Position { x, y });

// Make Velocity diffable  
crate::diffable!(Velocity { dx, dy });

// Make Health diffable
crate::diffable!(Health { current, max });

/// Demonstrate the system registration and execution pattern
pub fn demonstrate_ecs_systems() {
    let world = create_example_world();
    
    println!("=== Movement System (Position immutable, Mut<Velocity> mutable) ===");
    let ent_it = world.iter_entities::<Position, Mut<Velocity>>();
    movement_system(ent_it);
    
    println!("\n=== Health Damage System (Position immutable, Mut<Health> mutable) ===");
    let ent_it = world.iter_entities::<Position, Mut<Health>>();
    health_damage_system(ent_it);
}