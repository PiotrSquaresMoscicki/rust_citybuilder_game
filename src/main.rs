mod ecs;
mod examples;

use examples::demonstrate_ecs_systems;

fn main() {
    println!("Welcome to Rust Citybuilder Game!");
    println!("Demonstrating the Entity Component System...\n");
    
    // Demonstrate the ECS with the requested API
    demonstrate_ecs_systems();
}

#[cfg(test)]
mod main_test;
