mod ecs;
// mod examples;
mod http_server;
mod enhanced_http_server;
// mod mut_demo;
// mod diffing;
// mod diffing_demo;
// mod multiple_iterators_test;
// mod multiple_iterator_systems_test;
mod core;
// mod math_demo;
mod rendering;
// mod time_demo;
// mod system_object_example;

// #[cfg(test)]
// mod diffing_test;

// use examples::demonstrate_ecs_systems;
// use mut_demo::demonstrate_mut_requirement;
// use diffing_demo::demonstrate_diffing_system;
// use multiple_iterators_test::demonstrate_multiple_iterators;
// use multiple_iterator_systems_test::demonstrate_multiple_iterator_systems;
// use math_demo::demonstrate_math_library;
// use time_demo::run_time_demo;
use http_server::start_hello_world_server;
use enhanced_http_server::demonstrate_rendering_with_web_client;
use rendering::{WebServiceManager, WebClientRenderingDevice, initialize_global_rendering_manager, render_global_grid};
use rust_citybuilder_game::grid_game::{demonstrate_grid_game, run_interactive_grid_game};
use rust_citybuilder_game::web_grid_game::demonstrate_web_grid_game;
// use system_object_example::demonstrate_system_objects;
use std::env;

fn main() {
    println!("Welcome to Rust Citybuilder Game!");
    
    // Initialize the global rendering manager at program start
    let web_service = WebServiceManager::new("localhost:8081");
    let device = Box::new(WebClientRenderingDevice::new(web_service));
    
    if let Err(e) = initialize_global_rendering_manager(device) {
        eprintln!("Warning: Failed to initialize global rendering manager: {}", e);
    } else {
        println!("Global rendering manager initialized successfully");
    }
    
    // Check command line arguments to determine what to run
    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 {
        match args[1].as_str() {
            "ecs" => {
                println!("Demonstrating the Entity Component System...\n");
                // demonstrate_ecs_systems();
                println!("\n");
                // demonstrate_mut_requirement();
                println!("\n");
                // demonstrate_multiple_iterators();
            }
            "multi" => {
                println!("Demonstrating Multiple Iterators in Systems...\n");
                // demonstrate_multiple_iterators();
            }
            "multi-systems" => {
                println!("Demonstrating Multiple Iterator Systems...\n");
                // demonstrate_multiple_iterator_systems();
            }
            "diff" => {
                println!("Demonstrating the ECS Diffing System...\n");
                // demonstrate_diffing_system();
            }
            "math" => {
                println!("Demonstrating the Math Library...\n");
                // math_demo::demonstrate_math_library();
            }
            "time" => {
                println!("Demonstrating the Time Management System...\n");
                // run_time_demo();
            }
            "systems" => {
                println!("Demonstrating the New System Objects...\n");
                // demonstrate_system_objects();
            }
            "render" => {
                println!("Demonstrating the Rendering System...\n");
                demonstrate_rendering_system();
            }
            "web-render" => {
                println!("Starting Web Rendering Client...\n");
                demonstrate_rendering_with_web_client();
            }
            "game" => {
                println!("Starting 2D Grid Game...\n");
                demonstrate_grid_game();
            }
            "game-demo" => {
                println!("Running Interactive Grid Game Demo...\n");
                run_interactive_grid_game();
            }
            "web-game" => {
                println!("Starting Web Grid Game...\n");
                demonstrate_web_grid_game();
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
    println!("    multi               Demonstrate Multiple Iterators in Systems");
    println!("    multi-systems       Demonstrate Multiple Iterator Systems (new API)");
    println!("    diff                Demonstrate ECS Diffing System");
    println!("    math                Demonstrate Math Library (Vector2d, Angle2d, Transform2d)");
    println!("    time                Demonstrate Time Management System");
    println!("    systems             Demonstrate New System Objects (object-based systems)");
    println!("    render              Demonstrate Rendering System with Web Client");
    println!("    web-render          Start Interactive Web Rendering Client");
    println!("    game                Start 2D Grid Game");
    println!("    game-demo           Run Interactive Grid Game Demo");
    println!("    web-game            Start Web-based Grid Game");
    println!("    help                Show this help message");
    println!("");
    println!("EXAMPLES:");
    println!("    cargo run                    # Start HTTP server on localhost:8080");
    println!("    cargo run server             # Start HTTP server on localhost:8080");
    println!("    cargo run server 0.0.0.0:3000  # Start HTTP server on all interfaces, port 3000");
    println!("    cargo run ecs                # Run ECS demonstration");
    println!("    cargo run multi              # Run multiple iterators demonstration");
    println!("    cargo run diff               # Run ECS diffing demonstration");
    println!("    cargo run math               # Run math library demonstration");
    println!("    cargo run time               # Run time management demonstration");
    println!("    cargo run systems            # Run new system objects demonstration");
    println!("    cargo run render             # Run rendering system demonstration");
    println!("    cargo run web-render         # Start interactive web rendering client");
    println!("    cargo run game               # Start 2D grid game");
    println!("    cargo run game-demo          # Run interactive grid game demo");
    println!("    cargo run web-game           # Start web-based grid game");
    println!("");
}

fn demonstrate_rendering_system() {
    use std::thread;
    use std::time::Duration;
    
    println!("🎨 Rendering System Demonstration");
    println!("================================");
    
    // Test global rendering system
    if render_global_grid(10, 8, 32.0).is_ok() {
        println!("✅ Successfully sent grid rendering command to web client");
        println!("   Grid: 10x8 cells, 32px cell size");
        println!("   Colors: Black lines on white background");
    } else {
        println!("❌ Failed to send grid rendering command");
    }
    
    // Give some time for the command to be processed
    thread::sleep(Duration::from_millis(500));
    
    println!("\n📡 Web Service Information:");
    println!("   Web client available at: http://localhost:8081");
    println!("   Open this URL in your browser to see the rendered grid");
    
    println!("\n🔧 Technical Details:");
    println!("   - Rendering manager initialized globally at program start");
    println!("   - Web client rendering device communicates via HTTP");
    println!("   - Custom protocol sends JSON commands to web client");
    println!("   - Web client renders using HTML5 Canvas");
    
    println!("\nRendering system demonstration complete!");
}

// #[cfg(test)]
// mod main_test;
