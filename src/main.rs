mod ecs;
mod query;
mod examples;
mod query_examples;

use examples::demonstrate_ecs_systems;
use query_examples::{test_variable_arity_queries, system_with_variable_queries};

fn main() {
    println!("Welcome to Rust Citybuilder Game!");
    println!("Demonstrating the Entity Component System...\n");
    
    // Demonstrate the original ECS with the 2-component API
    println!("=== Original ECS API (2 components) ===");
    demonstrate_ecs_systems();
    
    println!("\n\n");
    
    // Demonstrate the new variable arity query system
    test_variable_arity_queries();
    
    println!();
    
    // Demonstrate systems using the new query API
    let world = query_examples::create_test_world();
    system_with_variable_queries(&world);
}

#[cfg(test)]
mod main_test;
