mod ecs;
mod examples;
mod http_server;
mod mut_demo;
mod diffing;
mod diffing_demo;

#[cfg(test)]
mod diffing_test;

use examples::demonstrate_ecs_systems;
use mut_demo::demonstrate_mut_requirement;
use diffing_demo::demonstrate_diffing_system;
use http_server::start_hello_world_server;
use std::env;

fn main() {
    println!("Welcome to Rust Citybuilder Game!");
    
    // Check command line arguments to determine what to run
    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 {
        match args[1].as_str() {
            "ecs" => {
                println!("Demonstrating the Entity Component System...\n");
                demonstrate_ecs_systems();
                println!("\n");
                demonstrate_mut_requirement();
            }
            "diff" => {
                println!("Demonstrating the ECS Diffing System...\n");
                demonstrate_diffing_system();
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
    println!("    diff                Demonstrate ECS Diffing System");
    println!("    help                Show this help message");
    println!("");
    println!("EXAMPLES:");
    println!("    cargo run                    # Start HTTP server on localhost:8080");
    println!("    cargo run server             # Start HTTP server on localhost:8080");
    println!("    cargo run server 0.0.0.0:3000  # Start HTTP server on all interfaces, port 3000");
    println!("    cargo run ecs                # Run ECS demonstration");
    println!("    cargo run diff               # Run ECS diffing demonstration");
    println!("");
}

#[cfg(test)]
mod main_test;
