mod ecs;
mod http_server;
mod enhanced_http_server;
mod core;
mod rendering;

use http_server::start_hello_world_server;
use enhanced_http_server::demonstrate_rendering_with_web_client;
use rendering::{WebServiceManager, WebClientRenderingDevice, initialize_global_rendering_manager, render_global_grid};
use rust_citybuilder_game::web_ecs_game::demonstrate_web_ecs_game;
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
            "render" => {
                println!("Demonstrating the Rendering System...\n");
                demonstrate_rendering_system();
            }
            "web-render" => {
                println!("Starting Web Rendering Client...\n");
                demonstrate_rendering_with_web_client();
            }
            "ecs-game" => {
                println!("Starting Web ECS Game Demo...\n");
                demonstrate_web_ecs_game();
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
        // Default behavior: start the ECS game
        println!("Starting Web ECS Game (default mode)...\n");
        println!("Use 'cargo run help' to see available commands.\n");
        
        demonstrate_web_ecs_game();
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
    println!("    render              Demonstrate Rendering System with Web Client");
    println!("    web-render          Start Interactive Web Rendering Client");
    println!("    ecs-game            Start Web ECS Game Demo (default)");
    println!("    help                Show this help message");
    println!("");
    println!("EXAMPLES:");
    println!("    cargo run                    # Start Web ECS game (default)");
    println!("    cargo run ecs-game           # Start Web ECS game explicitly");
    println!("    cargo run server             # Start HTTP server on localhost:8080");
    println!("    cargo run server 0.0.0.0:3000  # Start HTTP server on all interfaces, port 3000");
    println!("    cargo run render             # Run rendering system demonstration");
    println!("    cargo run web-render         # Start interactive web rendering client");
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
