mod ecs;
mod query;
mod examples;
mod query_examples;
mod http_server;

use examples::demonstrate_ecs_systems;
use query_examples::{test_variable_arity_queries, system_with_variable_queries};
use http_server::start_hello_world_server;
use std::env;

fn main() {
    // Demonstrate systems using the new query API
    let world = query_examples::create_test_world();
    system_with_variable_queries(&world);
  
    // Check command line arguments to determine what to run
    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 {
        match args[1].as_str() {
            "ecs" => {
                println!("Demonstrating the Entity Component System...\n");
                test_variable_arity_queries();
            }
            "server" => {
                println!("Starting HTTP server...\n");
                let address = args.get(2).map(|s| s.as_str()).unwrap_or("localhost:8080");
                
                if let Err(e) = start_hello_world_server(address) {
                    eprintln!("Server error: {}", e);
                    std::process::exit(1);
                }
            }
            "help" | "--help" | "-h" => {
                print_help();
            }
            _ => {
                println!("Unknown command: {}", args[1]);
                print_help();
            }
        }
    } else {
        // Default behavior: start the HTTP server
        println!("Starting HTTP server (default mode)...\n");
        println!("Use 'cargo run ecs' to run ECS demo instead.\n");
        
        if let Err(e) = start_hello_world_server("localhost:8080") {
            eprintln!("Server error: {}", e);
            std::process::exit(1);
        }
    }
}

fn print_help() {
    println!("Rust City Builder Game");
    println!("");
    println!("USAGE:");
    println!("    cargo run [COMMAND] [OPTIONS]");
    println!("");
    println!("COMMANDS:");
    println!("    server [ADDRESS]    Start HTTP server (default: localhost:8080)");
    println!("    ecs                 Demonstrate Entity Component System");
    println!("    help                Show this help message");
    println!("");
    println!("EXAMPLES:");
    println!("    cargo run                    # Start HTTP server on localhost:8080");
    println!("    cargo run server             # Start HTTP server on localhost:8080");
    println!("    cargo run server 0.0.0.0:3000  # Start HTTP server on all interfaces, port 3000");
    println!("    cargo run ecs                # Run ECS demonstration");
    println!("");
}

#[cfg(test)]
mod main_test;
